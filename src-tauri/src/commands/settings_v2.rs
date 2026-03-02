// Tsubasa (翼) — Grouped Settings Commands (v2)
// IPC handlers for the new 12-group settings system.
// Coexists with legacy settings commands for backward compatibility.

use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::error::TsubasaError;
use crate::settings::schema::*;

// ─── Get all settings ───────────────────────────────────────

#[tauri::command]
pub async fn get_all_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<AllSettings, TsubasaError> {
    Ok(state.settings_manager.all())
}

// ─── Per-group getters ──────────────────────────────────────

#[tauri::command]
pub async fn get_behavior_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<BehaviorSettings, TsubasaError> {
    Ok(state.settings_manager.behavior())
}

#[tauri::command]
pub async fn get_download_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<DownloadSettings, TsubasaError> {
    Ok(state.settings_manager.downloads())
}

#[tauri::command]
pub async fn get_connection_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<ConnectionSettings, TsubasaError> {
    Ok(state.settings_manager.connections())
}

#[tauri::command]
pub async fn get_speed_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<SpeedSettings, TsubasaError> {
    Ok(state.settings_manager.speed())
}

#[tauri::command]
pub async fn get_bittorrent_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<BitTorrentSettings, TsubasaError> {
    Ok(state.settings_manager.bittorrent())
}

#[tauri::command]
pub async fn get_queue_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<QueueSettings, TsubasaError> {
    Ok(state.settings_manager.queue())
}

#[tauri::command]
pub async fn get_seeding_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<SeedingSettings, TsubasaError> {
    Ok(state.settings_manager.seeding())
}

#[tauri::command]
pub async fn get_cloud_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<CloudSettings, TsubasaError> {
    Ok(state.settings_manager.cloud())
}

// ─── Per-group setters ──────────────────────────────────────

#[tauri::command]
pub async fn update_behavior_settings(
    state: State<'_, Arc<AppState>>,
    settings: BehaviorSettings,
) -> Result<(), TsubasaError> {
    state.settings_manager.set_behavior(settings)?;
    tracing::info!("Behavior settings updated");
    Ok(())
}

#[tauri::command]
pub async fn update_download_settings(
    state: State<'_, Arc<AppState>>,
    settings: DownloadSettings,
) -> Result<(), TsubasaError> {
    state.settings_manager.set_downloads(settings)?;
    tracing::info!("Download settings updated");
    Ok(())
}

#[tauri::command]
pub async fn update_connection_settings(
    state: State<'_, Arc<AppState>>,
    settings: ConnectionSettings,
) -> Result<(), TsubasaError> {
    state.settings_manager.set_connections(settings)?;
    tracing::info!("Connection settings updated");
    Ok(())
}

#[tauri::command]
pub async fn update_speed_settings(
    state: State<'_, Arc<AppState>>,
    settings: SpeedSettings,
) -> Result<(), TsubasaError> {
    state.settings_manager.set_speed(settings.clone())?;

    // Apply speed limits to running engine
    let engine_clone = {
        let guard = state.engine.read();
        guard.clone()
    };
    if let Some(engine) = engine_clone {
        let bandwidth = crate::bandwidth::BandwidthConfig {
            download_limit: settings.global_dl_limit,
            upload_limit: settings.global_ul_limit,
            ..Default::default()
        };
        engine.update_bandwidth(&bandwidth);
    }

    tracing::info!("Speed settings updated");
    Ok(())
}

#[tauri::command]
pub async fn update_bittorrent_settings(
    state: State<'_, Arc<AppState>>,
    settings: BitTorrentSettings,
) -> Result<(), TsubasaError> {
    state.settings_manager.set_bittorrent(settings)?;
    tracing::info!("BitTorrent settings updated");
    Ok(())
}

#[tauri::command]
pub async fn update_queue_settings(
    state: State<'_, Arc<AppState>>,
    settings: QueueSettings,
) -> Result<(), TsubasaError> {
    state.settings_manager.set_queue(settings)?;
    tracing::info!("Queue settings updated");
    Ok(())
}

#[tauri::command]
pub async fn update_seeding_settings(
    state: State<'_, Arc<AppState>>,
    settings: SeedingSettings,
) -> Result<(), TsubasaError> {
    state.settings_manager.set_seeding(settings)?;
    tracing::info!("Seeding settings updated");
    Ok(())
}

#[tauri::command]
pub async fn update_cloud_settings(
    state: State<'_, Arc<AppState>>,
    settings: CloudSettings,
) -> Result<(), TsubasaError> {
    // Update API keys in the cloud manager
    if let Some(ref key) = settings.torbox_api_key {
        state.cloud_manager.torbox.set_api_key(key.clone());
    }
    if let Some(ref key) = settings.realdebrid_api_key {
        state.cloud_manager.real_debrid.set_api_key(key.clone());
    }

    state.settings_manager.set_cloud(settings)?;
    tracing::info!("Cloud settings updated");
    Ok(())
}
