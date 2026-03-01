// Tsubasa — Cloud Commands
// IPC handlers for debrid/cloud operations.

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio_util::sync::CancellationToken;

use crate::app_state::AppState;
use crate::cloud::download_driver;
use crate::cloud::provider::{
    AccountInfo, CloudStatus, CloudTorrentId, DebridProvider, DirectLink, TorrentSource,
};
use crate::error::TsubasaError;

// ─── Response types ─────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CloudProviderStatus {
    pub name: String,
    pub configured: bool,
    pub connected: bool,
}

#[derive(Debug, Serialize)]
pub struct CloudAddResult {
    pub cloud_id: String,
    pub provider: String,
}

#[derive(Debug, Deserialize)]
pub struct CloudDownloadRequest {
    pub url: String,
    pub filename: String,
    pub save_dir: String,
    pub torrent_id: String,
    pub provider: String,
}

// ─── Commands ───────────────────────────────────────────

/// Get the configuration & connection status of all cloud providers.
/// Now does a live connection test by calling account_info.
#[tauri::command]
pub async fn get_cloud_status(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<CloudProviderStatus>, TsubasaError> {
    let cloud = &state.cloud_manager;

    // Test Torbox connectivity
    let torbox_connected = if cloud.torbox.is_configured() {
        cloud.torbox.account_info().await.is_ok()
    } else {
        false
    };

    // Test Real-Debrid connectivity
    let rd_connected = if cloud.realdebrid.is_configured() {
        cloud.realdebrid.account_info().await.is_ok()
    } else {
        false
    };

    Ok(vec![
        CloudProviderStatus {
            name: "Torbox".to_string(),
            configured: cloud.torbox.is_configured(),
            connected: torbox_connected,
        },
        CloudProviderStatus {
            name: "Real-Debrid".to_string(),
            configured: cloud.realdebrid.is_configured(),
            connected: rd_connected,
        },
    ])
}

/// Submit a torrent to a cloud debrid provider.
#[tauri::command]
pub async fn cloud_add_torrent(
    state: State<'_, Arc<AppState>>,
    source: String,
    provider: String,
) -> Result<CloudAddResult, TsubasaError> {
    let cloud = &state.cloud_manager;

    let prov = cloud.get_provider(&provider).ok_or_else(|| {
        TsubasaError::Cloud {
            provider: provider.clone(),
            source: crate::error::CloudError::Unavailable(format!(
                "Unknown provider: {provider}"
            )),
        }
    })?;

    // Determine the source type (magnet URI or info hash)
    let torrent_source = if source.starts_with("magnet:") {
        TorrentSource::MagnetUri(source)
    } else if source.len() == 40 && source.chars().all(|c| c.is_ascii_hexdigit()) {
        TorrentSource::InfoHash(source)
    } else {
        // Assume it's a magnet URI if it doesn't look like a plain hash
        TorrentSource::MagnetUri(source)
    };

    let cloud_id = prov.add_torrent(&torrent_source).await?;

    // Emit cloud status changed event
    state.event_bus.publish(crate::events::TsubasaEvent::CloudStatusChanged {
        torrent_id: cloud_id.0.clone(),
        provider: prov.name().to_string(),
        status: "queued".to_string(),
    });

    Ok(CloudAddResult {
        cloud_id: cloud_id.0,
        provider: prov.name().to_string(),
    })
}

/// Check the status of a torrent on a cloud provider.
#[tauri::command]
pub async fn cloud_check_status(
    state: State<'_, Arc<AppState>>,
    cloud_id: String,
    provider: String,
) -> Result<CloudStatus, TsubasaError> {
    let cloud = &state.cloud_manager;

    let prov = cloud.get_provider(&provider).ok_or_else(|| {
        TsubasaError::Cloud {
            provider: provider.clone(),
            source: crate::error::CloudError::Unavailable(format!(
                "Unknown provider: {provider}"
            )),
        }
    })?;

    let status = prov.check_status(&CloudTorrentId(cloud_id)).await?;
    Ok(status)
}

/// Get direct download links for a completed cloud torrent.
#[tauri::command]
pub async fn cloud_get_links(
    state: State<'_, Arc<AppState>>,
    cloud_id: String,
    provider: String,
) -> Result<Vec<DirectLink>, TsubasaError> {
    let cloud = &state.cloud_manager;

    let prov = cloud.get_provider(&provider).ok_or_else(|| {
        TsubasaError::Cloud {
            provider: provider.clone(),
            source: crate::error::CloudError::Unavailable(format!(
                "Unknown provider: {provider}"
            )),
        }
    })?;

    let links = prov
        .get_download_links(&CloudTorrentId(cloud_id))
        .await?;
    Ok(links)
}

/// Check if a torrent is cached (instant availability) on all configured providers.
/// Returns a map of provider name -> cached boolean.
#[tauri::command]
pub async fn cloud_check_cached(
    state: State<'_, Arc<AppState>>,
    info_hash: String,
) -> Result<Vec<(String, bool)>, TsubasaError> {
    let cloud = &state.cloud_manager;
    let mut results = Vec::new();

    if cloud.torbox.is_configured() {
        let cached = cloud.torbox.check_cached(&info_hash).await.unwrap_or(false);
        results.push(("Torbox".to_string(), cached));
    }

    if cloud.realdebrid.is_configured() {
        let cached = cloud
            .realdebrid
            .check_cached(&info_hash)
            .await
            .unwrap_or(false);
        results.push(("Real-Debrid".to_string(), cached));
    }

    Ok(results)
}

/// Get account information for a specific cloud provider.
#[tauri::command]
pub async fn cloud_account_info(
    state: State<'_, Arc<AppState>>,
    provider: String,
) -> Result<AccountInfo, TsubasaError> {
    let cloud = &state.cloud_manager;

    let prov = cloud.get_provider(&provider).ok_or_else(|| {
        TsubasaError::Cloud {
            provider: provider.clone(),
            source: crate::error::CloudError::Unavailable(format!(
                "Unknown provider: {provider}"
            )),
        }
    })?;

    prov.account_info().await
}

/// Delete a torrent from a cloud provider.
#[tauri::command]
pub async fn cloud_delete_torrent(
    state: State<'_, Arc<AppState>>,
    cloud_id: String,
    provider: String,
) -> Result<(), TsubasaError> {
    let cloud = &state.cloud_manager;

    let prov = cloud.get_provider(&provider).ok_or_else(|| {
        TsubasaError::Cloud {
            provider: provider.clone(),
            source: crate::error::CloudError::Unavailable(format!(
                "Unknown provider: {provider}"
            )),
        }
    })?;

    prov.delete_torrent(&CloudTorrentId(cloud_id)).await
}

/// Download a file from a cloud provider's CDN to local disk.
/// This is a long-running operation — progress is reported via events.
#[tauri::command]
pub async fn cloud_download_file(
    state: State<'_, Arc<AppState>>,
    request: CloudDownloadRequest,
) -> Result<download_driver::CloudDownloadResult, TsubasaError> {
    let cloud = &state.cloud_manager;
    let event_tx = state.event_bus.sender();
    let cancel = CancellationToken::new();

    let result = download_driver::download_cloud_file(
        &cloud.http_client,
        &request.url,
        &request.filename,
        &PathBuf::from(&request.save_dir),
        &request.torrent_id,
        &request.provider,
        &event_tx,
        &cancel,
    )
    .await?;

    Ok(result)
}
