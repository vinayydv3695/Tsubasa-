// Tsubasa — Library Root
// Module declarations and public API.

pub mod app_state;
pub mod commands;
pub mod error;
pub mod events;
pub mod engine;
pub mod download;
pub mod storage;
pub mod cloud;
pub mod bandwidth;
pub mod search;
pub mod logging;
pub mod settings;
pub mod queue;
pub mod speed_graph;
pub mod seeding;
pub mod tracker;
pub mod connection;
pub mod ip_filter;
pub mod crypto;
pub mod retry;
pub mod updater;

use std::path::PathBuf;
use std::sync::Arc;

use tauri::Emitter;
use tokio_util::sync::CancellationToken;

use app_state::AppState;
use cloud::CloudManager;
use download::DownloadOrchestrator;
use engine::TorrentEngine;
use events::{EventBus, TsubasaEvent};
use search::SearchEngine;
use storage::database::Database;
use storage::session::SessionManager;

/// Get the application data directory.
fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tsubasa")
}

/// Main entry point for the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_dir = app_data_dir();
    std::fs::create_dir_all(&app_dir).expect("Failed to create app data directory");

    // Create shared log buffer (used by both logging and AppState)
    let log_buffer = Arc::new(logging::ring_buffer::LogRingBuffer::default());

    // Initialize logging with the ring buffer
    let log_dir = app_dir.join("logs");
    if let Err(e) = logging::init_logging(log_dir, Some(log_buffer.clone())) {
        eprintln!("Failed to initialize logging: {e}");
    }

    tracing::info!("Tsubasa starting up, data dir: {}", app_dir.display());

    // Open database
    let db_path = app_dir.join("tsubasa.db");
    let db = Database::open(&db_path).expect("Failed to open database");

    // Load settings to initialize cloud providers with saved API keys
    let (torbox_key, rd_key) = match db.load_settings() {
        Ok(settings) => (settings.torbox_api_key, settings.realdebrid_api_key),
        Err(e) => {
            tracing::warn!("Failed to load settings for cloud init: {e}");
            (None, None)
        }
    };

    // Create event bus externally so orchestrator can hold a sender
    let event_bus = EventBus::new();

    // Create cloud manager as Arc
    let cloud_manager = Arc::new(CloudManager::new(torbox_key, rd_key));
    tracing::info!(
        providers = ?cloud_manager.configured_providers(),
        "Cloud manager initialized"
    );

    // Create download orchestrator with cloned handles
    let orchestrator = DownloadOrchestrator::new(
        cloud_manager.clone(),
        event_bus.sender(),
        db.clone(),
    );

    // Create search engine
    let search_engine = SearchEngine::new(cloud_manager.clone(), db.clone());

    // Create grouped settings manager (v2)
    let settings_manager = Arc::new(
        settings::SettingsManager::new(db.clone())
            .expect("Failed to initialize settings manager")
    );
    tracing::info!("Settings manager initialized (v2 grouped schema)");

    // Create application state
    let state = Arc::new(AppState::new(db, log_buffer, cloud_manager, event_bus, orchestrator, search_engine, settings_manager));

    // Clone state for the shutdown window event handler
    let shutdown_state = state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![
            // Torrent commands
            commands::torrent::add_torrent,
            commands::torrent::get_torrents,
            commands::torrent::pause_torrent,
            commands::torrent::resume_torrent,
            commands::torrent::remove_torrent,
            commands::torrent::get_torrent_details,
            commands::torrent::get_torrent_files,
            commands::torrent::get_torrent_peers,
            commands::torrent::get_torrent_trackers,
            // Settings commands (legacy)
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::get_setting,
            commands::settings::set_setting,
            // Settings commands (v2 grouped)
            commands::settings_v2::get_all_settings,
            commands::settings_v2::get_behavior_settings,
            commands::settings_v2::get_download_settings,
            commands::settings_v2::get_connection_settings,
            commands::settings_v2::get_speed_settings,
            commands::settings_v2::get_bittorrent_settings,
            commands::settings_v2::get_queue_settings,
            commands::settings_v2::get_seeding_settings,
            commands::settings_v2::get_cloud_settings,
            commands::settings_v2::update_behavior_settings,
            commands::settings_v2::update_download_settings,
            commands::settings_v2::update_connection_settings,
            commands::settings_v2::update_speed_settings,
            commands::settings_v2::update_bittorrent_settings,
            commands::settings_v2::update_queue_settings,
            commands::settings_v2::update_seeding_settings,
            commands::settings_v2::update_cloud_settings,
            // Cloud commands
            commands::cloud::get_cloud_status,
            commands::cloud::cloud_add_torrent,
            commands::cloud::cloud_check_status,
            commands::cloud::cloud_get_links,
            commands::cloud::cloud_check_cached,
            commands::cloud::cloud_account_info,
            commands::cloud::cloud_delete_torrent,
            commands::cloud::cloud_download_file,
            // System commands
            commands::system::get_app_info,
            commands::system::get_logs,
            commands::system::clear_logs,
            // Search commands (Torbox API)
            commands::search::search_torrents,
            commands::search::save_search_history,
            commands::search::get_search_history,
            commands::search::clear_search_history,
            // Search commands (Aggregator)
            commands::search_aggregator::aggregator_search,
            commands::search_aggregator::get_search_plugins,
            // Speed graph commands
            commands::speed_graph::get_speed_graph,
            commands::speed_graph::get_torrent_speed_graph,
            // Queue commands
            commands::queue::get_queue_positions,
            commands::queue::force_start_torrent,
            commands::queue::set_torrent_priority,
        ])
        .on_window_event(move |_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                tracing::info!("Window destroyed, initiating graceful shutdown");

                // 1. Emit shutdown event to frontend
                shutdown_state.event_bus.publish(TsubasaEvent::EngineShuttingDown);

                // 2. Cancel all active cloud lifecycle tasks
                shutdown_state.orchestrator.cancel_all();

                // 3. Final database progress sync
                {
                    let guard = shutdown_state.engine.read();
                    if let Some(ref engine) = *guard {
                        let progress = engine.all_torrent_progress();
                        for (id, downloaded, uploaded, total) in progress {
                            if let Err(e) = shutdown_state.db.update_torrent_progress(
                                &id, downloaded, uploaded, total,
                            ) {
                                tracing::warn!(
                                    torrent_id = %id,
                                    "Failed to sync progress during shutdown: {e}"
                                );
                            }
                        }
                        tracing::info!("Final DB progress sync complete");

                        // 4. Shut down the torrent engine
                        engine.shutdown();
                    }
                }

                tracing::info!("Graceful shutdown complete");
            }
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let state_clone = state.clone();
            let app_dir_clone = app_dir.clone();

            // Spawn async initialization
            tauri::async_runtime::spawn(async move {
                // Initialize session manager
                let session_dir = app_dir_clone.join("sessions");
                match SessionManager::new(session_dir) {
                    Ok(mgr) => {
                        *state_clone.session_manager.write() = Some(mgr);
                        tracing::info!("Session manager initialized");
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize session manager: {e}");
                    }
                }

                // Load settings for engine config
                let settings = match state_clone.db.load_settings() {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("Failed to load settings: {e}");
                        return;
                    }
                };

                let download_dir = PathBuf::from(&settings.download_dir);
                if let Err(e) = std::fs::create_dir_all(&download_dir) {
                    tracing::error!("Failed to create download directory: {e}");
                    return;
                }

                let cancel_token = CancellationToken::new();
                let db_sync_token = cancel_token.clone();

                // Build bandwidth config from settings
                let bandwidth = bandwidth::BandwidthConfig {
                    download_limit: settings.global_download_limit,
                    upload_limit: settings.global_upload_limit,
                    ..Default::default()
                };

                match TorrentEngine::new(
                    download_dir,
                    state_clone.event_bus.sender(),
                    cancel_token,
                    bandwidth,
                    app_dir_clone.join("sessions"),
                )
                .await
                {
                    Ok(engine) => {
                        let engine = Arc::new(engine);
                        *state_clone.engine.write() = Some(engine.clone());
                        state_clone.event_bus.publish(TsubasaEvent::EngineReady);
                        tracing::info!("Torrent engine initialized");

                        // Spawn periodic database sync task (every 30 seconds)
                        let db_sync_state = state_clone.clone();
                        let db_sync_engine = engine.clone();
                        tokio::spawn(async move {
                            let mut interval = tokio::time::interval(
                                std::time::Duration::from_secs(30),
                            );
                            interval.set_missed_tick_behavior(
                                tokio::time::MissedTickBehavior::Skip,
                            );
                            loop {
                                tokio::select! {
                                    _ = db_sync_token.cancelled() => {
                                        tracing::info!("DB sync task shutting down");
                                        break;
                                    }
                                    _ = interval.tick() => {
                                        let progress = db_sync_engine.all_torrent_progress();
                                        for (id, downloaded, uploaded, total) in progress {
                                            if let Err(e) = db_sync_state.db.update_torrent_progress(
                                                &id, downloaded, uploaded, total,
                                            ) {
                                                tracing::warn!(
                                                    torrent_id = %id,
                                                    "Failed to sync progress to DB: {e}"
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize torrent engine: {e}");
                        state_clone.event_bus.publish(TsubasaEvent::Error {
                            torrent_id: None,
                            message: format!("Engine initialization failed: {e}"),
                            recoverable: false,
                        });
                    }
                }

                // Start event bridge to frontend
                let mut event_rx = state_clone.event_bus.subscribe();
                let handle = app_handle.clone();
                tokio::spawn(async move {
                    loop {
                        match event_rx.recv().await {
                            Ok(event) => {
                                let _ = handle.emit("tsubasa-event", &event);
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                                tracing::warn!("Event bridge lagged by {n} events");
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                tracing::info!("Event bus closed, stopping bridge");
                                break;
                            }
                        }
                    }
                });
            });

            Ok(())
        })
        .run(tauri::generate_context!())
            .expect("Failed to run the app")
}
