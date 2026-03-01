// Tsubasa (翼) — Settings Commands
// IPC handlers for application settings.

use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::bandwidth::BandwidthConfig;
use crate::error::TsubasaError;
use crate::storage::models::AppSettings;

#[tauri::command]
pub async fn get_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<AppSettings, TsubasaError> {
    state.db.load_settings()
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, Arc<AppState>>,
    settings: AppSettings,
) -> Result<(), TsubasaError> {
    state.db.save_settings(&settings)?;

    // Apply bandwidth limits to the running engine if available
    let engine_clone = {
        let guard = state.engine.read();
        guard.clone()
    };
    if let Some(engine) = engine_clone {
        let bandwidth = BandwidthConfig {
            download_limit: settings.global_download_limit,
            upload_limit: settings.global_upload_limit,
            ..Default::default()
        };
        engine.update_bandwidth(&bandwidth);
    }

    tracing::info!("Settings updated");
    Ok(())
}

#[tauri::command]
pub async fn get_setting(
    state: State<'_, Arc<AppState>>,
    key: String,
) -> Result<Option<String>, TsubasaError> {
    state.db.get_setting(&key)
}

#[tauri::command]
pub async fn set_setting(
    state: State<'_, Arc<AppState>>,
    key: String,
    value: String,
) -> Result<(), TsubasaError> {
    state.db.set_setting(&key, &value)
}
