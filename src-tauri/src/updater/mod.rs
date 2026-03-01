// Tsubasa (翼) — Updater
// Tauri silent auto-update integration.
// Provides a command to check for updates and a background check loop.

use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

/// Check for available updates.
/// Returns Some((version, body)) if an update is available, None otherwise.
pub async fn check_for_update(
    app: &AppHandle,
) -> crate::error::Result<Option<(String, Option<String>)>> {
    let updater = app.updater().map_err(|e| {
        crate::error::TsubasaError::Internal(format!("Updater not available: {e}"))
    })?;

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            let body = update.body.clone();
            tracing::info!(version = %version, "Update available");
            Ok(Some((version, body)))
        }
        Ok(None) => {
            tracing::debug!("No update available");
            Ok(None)
        }
        Err(e) => {
            // Don't treat update check failures as fatal
            tracing::warn!("Update check failed: {e}");
            Ok(None)
        }
    }
}
