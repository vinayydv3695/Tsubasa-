// Tsubasa (翼) — System Commands
// IPC handlers for app info, logs, diagnostics.

use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::app_state::AppState;
use crate::error::TsubasaError;
use crate::logging::ring_buffer::LogEntry;

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub version: String,
    pub engine_ready: bool,
    pub uptime_seconds: u64,
}

#[tauri::command]
pub async fn get_app_info(
    state: State<'_, Arc<AppState>>,
) -> Result<AppInfo, TsubasaError> {
    let uptime = state.started_at.elapsed().as_secs();

    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        engine_ready: state.engine.read().is_some(),
        uptime_seconds: uptime,
    })
}

#[tauri::command]
pub async fn get_logs(
    state: State<'_, Arc<AppState>>,
    since_index: Option<usize>,
) -> Result<Vec<LogEntry>, TsubasaError> {
    let entries = match since_index {
        Some(idx) => state.log_buffer.get_since(idx),
        None => state.log_buffer.get_all(),
    };
    Ok(entries)
}

#[tauri::command]
pub async fn clear_logs(
    state: State<'_, Arc<AppState>>,
) -> Result<(), TsubasaError> {
    state.log_buffer.clear();
    Ok(())
}
