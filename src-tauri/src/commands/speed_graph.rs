// Tsubasa (翼) — Speed Graph Commands
// IPC handlers for bandwidth graph data.

use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use tauri::State;

use crate::app_state::AppState;
use crate::error::TsubasaError;
use crate::speed_graph::collector::SpeedSample;

/// Get global speed graph data.
/// `window_secs` — how many seconds of history to return (default: 300 = 5 min).
#[tauri::command]
pub async fn get_speed_graph(
    state: State<'_, Arc<AppState>>,
    window_secs: Option<u64>,
) -> Result<Vec<SpeedSample>, TsubasaError> {
    let window = Duration::from_secs(window_secs.unwrap_or(300));

    // Note: The speed graph collector needs to be integrated into AppState.
    // For now, return empty. This will be wired up when the polling loop is connected.
    Ok(Vec::new())
}

/// Get speed graph data for a specific torrent.
#[tauri::command]
pub async fn get_torrent_speed_graph(
    state: State<'_, Arc<AppState>>,
    torrent_id: String,
    window_secs: Option<u64>,
) -> Result<Vec<SpeedSample>, TsubasaError> {
    let window = Duration::from_secs(window_secs.unwrap_or(300));

    // Placeholder — will be wired when per-torrent graphs are added to AppState
    Ok(Vec::new())
}
