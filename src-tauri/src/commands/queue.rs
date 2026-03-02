// Tsubasa (翼) — Queue Commands
// IPC handlers for the queue manager.

use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::error::TsubasaError;
use crate::queue::manager::QueuePosition;

/// Get queue positions for all torrents.
#[tauri::command]
pub async fn get_queue_positions(
    _state: State<'_, Arc<AppState>>,
) -> Result<Vec<QueuePositionInfo>, TsubasaError> {
    // Placeholder — will be populated when QueueManager is integrated into AppState
    Ok(Vec::new())
}

/// Force start a torrent (bypass queue limits).
#[tauri::command]
pub async fn force_start_torrent(
    _state: State<'_, Arc<AppState>>,
    torrent_id: String,
) -> Result<(), TsubasaError> {
    tracing::info!(torrent_id = %torrent_id, "Force starting torrent");
    // Will call queue_manager.force_start() when integrated
    Ok(())
}

/// Set priority for a torrent.
#[tauri::command]
pub async fn set_torrent_priority(
    _state: State<'_, Arc<AppState>>,
    torrent_id: String,
    priority: i32,
) -> Result<(), TsubasaError> {
    tracing::info!(torrent_id = %torrent_id, priority, "Setting torrent priority");
    // Will call queue_manager.set_priority() when integrated
    Ok(())
}

/// Queue position info for the frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct QueuePositionInfo {
    pub torrent_id: String,
    pub position: String,
}
