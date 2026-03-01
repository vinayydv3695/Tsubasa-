// Tsubasa (翼) — Torrent Commands
// IPC handlers for torrent operations.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app_state::AppState;
use crate::download::state_machine::{DownloadPolicy, TorrentState};
use crate::error::TsubasaError;
use crate::events::TsubasaEvent;
use crate::storage::models::{
    TorrentFileInfo, TorrentPeerInfo, TorrentRecord, TorrentSummary, TorrentTrackerInfo,
};

/// Request to add a new torrent.
#[derive(Debug, Deserialize)]
pub struct AddTorrentRequest {
    /// Magnet URI or file path
    pub source: String,
    /// Override download directory (optional)
    pub save_path: Option<String>,
    /// Download policy override (optional)
    pub policy: Option<DownloadPolicy>,
}

/// Response after adding a torrent.
#[derive(Debug, Serialize)]
pub struct AddTorrentResponse {
    pub id: String,
    pub name: String,
    pub info_hash: String,
}

#[tauri::command]
pub async fn add_torrent(
    state: State<'_, Arc<AppState>>,
    request: AddTorrentRequest,
) -> Result<AddTorrentResponse, TsubasaError> {
    let id = uuid::Uuid::new_v4().to_string();
    let settings = state.db.load_settings()?;

    let save_path = request
        .save_path
        .unwrap_or_else(|| settings.download_dir.clone());
    let policy = request.policy.unwrap_or(settings.default_policy);

    // Extract preliminary metadata from the source
    let name = if request.source.starts_with("magnet:") {
        extract_magnet_name(&request.source).unwrap_or_else(|| format!("Torrent {}", &id[..8]))
    } else {
        std::path::Path::new(&request.source)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    };

    let info_hash = extract_info_hash(&request.source).unwrap_or_else(|| id[..16].to_string());
    let now = chrono::Utc::now().to_rfc3339();

    // Create database record
    let record = TorrentRecord {
        id: id.clone(),
        info_hash: info_hash.clone(),
        name: name.clone(),
        state: TorrentState::Pending,
        policy,
        total_bytes: 0,
        downloaded_bytes: 0,
        uploaded_bytes: 0,
        save_path: save_path.clone(),
        magnet_uri: if request.source.starts_with("magnet:") {
            Some(request.source.clone())
        } else {
            None
        },
        added_at: now,
        completed_at: None,
        download_speed_limit: 0,
        upload_speed_limit: 0,
        max_peers: 0,
        ratio_limit: settings.default_ratio_limit,
        error_message: None,
    };

    state.db.insert_torrent(&record)?;

    // Route through the orchestrator based on policy.
    // Clone the Arc<TorrentEngine> in a sync scope, then pass to orchestrator.
    let engine_clone = {
        let guard = state.engine.read();
        guard.clone()
    };

    match state
        .orchestrator
        .start_download(&id, &request.source, policy, &save_path, engine_clone)
        .await
    {
        Ok(result) => {
            let _ = state.db.update_torrent_state(&id, "downloading", None);
            state.event_bus.publish(TsubasaEvent::TorrentStateChanged {
                id: id.clone(),
                from: "pending".to_string(),
                to: "downloading".to_string(),
            });

            // For cloud-only, the engine doesn't emit TorrentAdded, so we do it manually
            if !result.local_started && result.cloud_started {
                state.event_bus.publish(TsubasaEvent::TorrentAdded {
                    id: id.clone(),
                    name: name.clone(),
                    info_hash: info_hash.clone(),
                });
            }

            // Use orchestrator-returned values when available, fall back to magnet-parsed
            let final_name = if result.name.is_empty() {
                name
            } else {
                result.name
            };
            let final_hash = if result.info_hash.is_empty() {
                info_hash
            } else {
                result.info_hash
            };

            return Ok(AddTorrentResponse {
                id,
                name: final_name,
                info_hash: final_hash,
            });
        }
        Err(e) => {
            let err_msg = e.to_string();
            state
                .db
                .update_torrent_state(&id, "errored", Some(&err_msg))?;
            state.event_bus.publish(TsubasaEvent::Error {
                torrent_id: Some(id.clone()),
                message: err_msg,
                recoverable: true,
            });
        }
    }

    Ok(AddTorrentResponse {
        id,
        name,
        info_hash,
    })
}

#[tauri::command]
pub async fn get_torrents(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<TorrentSummary>, TsubasaError> {
    let records = state.db.get_all_torrents()?;

    // Merge live stats from the engine where available
    let summaries: Vec<TorrentSummary> = records
        .into_iter()
        .map(|r| {
            let (download_speed, upload_speed, peers_connected, seeds_connected, eta_seconds, live_progress, live_total) = {
                let guard = state.engine.read();
                if let Some(ref engine) = *guard {
                    if let Some(stats) = engine.get_torrent_stats(&r.id) {
                        let (dl, ul, peers, seeds, eta) = if let Some(ref live) = stats.live {
                            let dl = live.download_speed.mbps * 1_000_000.0 / 8.0;
                            let ul = live.upload_speed.mbps * 1_000_000.0 / 8.0;
                            let peers = live.snapshot.peer_stats.live as u32;
                            let remaining = stats.total_bytes.saturating_sub(stats.progress_bytes);
                            let eta = if dl > 0.0 && remaining > 0 {
                                Some((remaining as f64 / dl) as u64)
                            } else {
                                None
                            };
                            (dl, ul, peers, 0u32, eta)
                        } else {
                            (0.0, 0.0, 0, 0, None)
                        };
                        (dl, ul, peers, seeds, eta, stats.progress_bytes, stats.total_bytes)
                    } else {
                        (0.0, 0.0, 0, 0, None, r.downloaded_bytes, r.total_bytes)
                    }
                } else {
                    (0.0, 0.0, 0, 0, None, r.downloaded_bytes, r.total_bytes)
                }
            };

            let progress = if live_total > 0 {
                live_progress as f64 / live_total as f64
            } else {
                0.0
            };
            let ratio = if r.downloaded_bytes > 0 {
                r.uploaded_bytes as f64 / r.downloaded_bytes as f64
            } else {
                0.0
            };

            TorrentSummary {
                id: r.id,
                info_hash: r.info_hash,
                name: r.name,
                state: r.state,
                policy: r.policy,
                progress,
                total_bytes: live_total,
                downloaded_bytes: live_progress,
                uploaded_bytes: r.uploaded_bytes,
                download_speed,
                upload_speed,
                peers_connected,
                seeds_connected,
                eta_seconds,
                ratio,
                added_at: r.added_at,
                save_path: r.save_path,
                error_message: r.error_message,
            }
        })
        .collect();

    Ok(summaries)
}

#[tauri::command]
pub async fn pause_torrent(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), TsubasaError> {
    let record = state.db.get_torrent(&id)?;
    let new_state = record.state.transition_to(TorrentState::Paused)?;

    // Clone the Arc<TorrentEngine> in a sync scope, then call async without holding the lock.
    let engine_clone = {
        let guard = state.engine.read();
        guard.clone()
    };

    if let Some(engine) = engine_clone {
        engine.pause_torrent(&id).await?;
    }

    state
        .db
        .update_torrent_state(&id, &new_state.to_string(), None)?;

    state.event_bus.publish(TsubasaEvent::TorrentStateChanged {
        id: id.clone(),
        from: record.state.to_string(),
        to: new_state.to_string(),
    });

    Ok(())
}

#[tauri::command]
pub async fn resume_torrent(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), TsubasaError> {
    let record = state.db.get_torrent(&id)?;
    let new_state = record.state.transition_to(TorrentState::Downloading)?;

    // Clone the Arc<TorrentEngine> in a sync scope, then call async without holding the lock.
    let engine_clone = {
        let guard = state.engine.read();
        guard.clone()
    };

    if let Some(engine) = engine_clone {
        engine.resume_torrent(&id).await?;
    }

    state
        .db
        .update_torrent_state(&id, &new_state.to_string(), None)?;

    state.event_bus.publish(TsubasaEvent::TorrentStateChanged {
        id: id.clone(),
        from: record.state.to_string(),
        to: new_state.to_string(),
    });

    Ok(())
}

#[tauri::command]
pub async fn remove_torrent(
    state: State<'_, Arc<AppState>>,
    id: String,
    delete_files: bool,
) -> Result<(), TsubasaError> {
    let record = state.db.get_torrent(&id)?;

    // Cancel any active cloud tasks for this torrent
    state.orchestrator.cancel_cloud_tasks(&id);

    // Clone the Arc<TorrentEngine> in a sync scope, then call async without holding the lock.
    let engine_clone = {
        let guard = state.engine.read();
        guard.clone()
    };

    if let Some(engine) = engine_clone {
        if let Err(e) = engine.delete_torrent(&id, delete_files).await {
            tracing::warn!(torrent_id = %id, "Failed to delete from engine: {e:#}");
        }
    } else if delete_files {
        // Engine not available — try manual file deletion
        let save_path = std::path::Path::new(&record.save_path);
        if save_path.exists() {
            let _ = std::fs::remove_dir_all(save_path);
        }
    }

    // Remove session data
    if let Some(ref session_mgr) = *state.session_manager.read() {
        let _ = session_mgr.remove_session(&record.info_hash);
    }

    state.db.remove_torrent(&id)?;

    state.event_bus.publish(TsubasaEvent::TorrentRemoved {
        id: id.clone(),
    });

    Ok(())
}

#[tauri::command]
pub async fn get_torrent_details(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<TorrentRecord, TsubasaError> {
    state.db.get_torrent(&id)
}

#[tauri::command]
pub async fn get_torrent_files(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Vec<TorrentFileInfo>, TsubasaError> {
    let engine_clone = {
        let guard = state.engine.read();
        guard.clone()
    };

    match engine_clone {
        Some(engine) => engine.get_torrent_files(&id),
        None => Err(TsubasaError::Engine(
            crate::error::EngineError::Operation("Engine not initialized".to_string()),
        )),
    }
}

#[tauri::command]
pub async fn get_torrent_peers(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Vec<TorrentPeerInfo>, TsubasaError> {
    let engine_clone = {
        let guard = state.engine.read();
        guard.clone()
    };

    match engine_clone {
        Some(engine) => engine.get_torrent_peers(&id),
        None => Err(TsubasaError::Engine(
            crate::error::EngineError::Operation("Engine not initialized".to_string()),
        )),
    }
}

#[tauri::command]
pub async fn get_torrent_trackers(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Vec<TorrentTrackerInfo>, TsubasaError> {
    let engine_clone = {
        let guard = state.engine.read();
        guard.clone()
    };

    match engine_clone {
        Some(engine) => engine.get_torrent_trackers(&id),
        None => Err(TsubasaError::Engine(
            crate::error::EngineError::Operation("Engine not initialized".to_string()),
        )),
    }
}

// ─── Helpers ────────────────────────────────────────

fn extract_magnet_name(magnet: &str) -> Option<String> {
    magnet
        .split('&')
        .find(|part| part.starts_with("dn=") || part.contains("dn="))
        .and_then(|part| {
            part.split('=')
                .nth(1)
                .map(|name| urlencoding::decode(name).unwrap_or_default().into_owned())
        })
}

fn extract_info_hash(source: &str) -> Option<String> {
    if source.starts_with("magnet:") {
        source
            .split('&')
            .find(|part| part.contains("xt=urn:btih:"))
            .and_then(|part| part.split("btih:").nth(1))
            .map(|hash| hash.to_lowercase())
    } else {
        None
    }
}
