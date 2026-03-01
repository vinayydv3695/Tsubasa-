// Tsubasa (翼) — Download Orchestrator
// Decides download path based on policy and manages the download lifecycle.
// Routes torrents to local engine, cloud providers, or both (hybrid/race mode).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use super::state_machine::DownloadPolicy;
use crate::cloud::download_driver::download_all_cloud_files;
use crate::cloud::provider::{CloudStatus, CloudTorrentId, DebridProvider, TorrentSource};
use crate::cloud::CloudManager;
use crate::engine::TorrentEngine;
use crate::error::{DownloadError, TsubasaError};
use crate::events::TsubasaEvent;
use crate::retry::{retry_with_backoff, RetryConfig};
use crate::storage::database::Database;

/// Preferred download path when in Hybrid mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreferredPath {
    /// Prefer local swarm, fallback to cloud
    PreferLocal,
    /// Prefer cloud, fallback to local
    PreferCloud,
    /// Race both simultaneously
    Race,
}

impl Default for PreferredPath {
    fn default() -> Self {
        PreferredPath::Race
    }
}

/// Configuration for a specific torrent's download behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub policy: DownloadPolicy,
    pub preferred_path: PreferredPath,
    /// Speed limit in bytes/sec, 0 = unlimited
    pub speed_limit_down: u64,
    /// Upload speed limit in bytes/sec, 0 = unlimited
    pub speed_limit_up: u64,
    /// Max peers for this torrent, 0 = use global default
    pub max_peers: u32,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            policy: DownloadPolicy::default(),
            preferred_path: PreferredPath::default(),
            speed_limit_down: 0,
            speed_limit_up: 0,
            max_peers: 0,
        }
    }
}

/// Tracks an active cloud lifecycle task so it can be cancelled.
struct CloudLifecycleTask {
    cancel: CancellationToken,
    provider_name: String,
    cloud_torrent_id: Option<CloudTorrentId>,
}

/// Result of starting a download through the orchestrator.
#[derive(Debug, Clone, Serialize)]
pub struct StartResult {
    pub info_hash: String,
    pub name: String,
    pub total_bytes: u64,
    pub local_started: bool,
    pub cloud_started: bool,
}

/// The Download Orchestrator.
/// Routes torrents to local engine, cloud providers, or both based on policy.
pub struct DownloadOrchestrator {
    cloud_manager: Arc<CloudManager>,
    event_tx: broadcast::Sender<TsubasaEvent>,
    db: Database,
    active_tasks: Arc<parking_lot::RwLock<HashMap<String, CloudLifecycleTask>>>,
}

impl DownloadOrchestrator {
    /// Create a new orchestrator.
    pub fn new(
        cloud_manager: Arc<CloudManager>,
        event_tx: broadcast::Sender<TsubasaEvent>,
        db: Database,
    ) -> Self {
        Self {
            cloud_manager,
            event_tx,
            db,
            active_tasks: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    /// Main entry point: start a download based on the given policy.
    pub async fn start_download(
        &self,
        torrent_id: &str,
        source: &str,
        policy: DownloadPolicy,
        save_path: &str,
        engine: Option<Arc<TorrentEngine>>,
    ) -> crate::error::Result<StartResult> {
        match policy {
            DownloadPolicy::LocalOnly => {
                self.start_local_only(torrent_id, source, engine).await
            }
            DownloadPolicy::CloudOnly => {
                self.start_cloud_only(torrent_id, source, save_path).await
            }
            DownloadPolicy::Hybrid => {
                self.start_hybrid(torrent_id, source, save_path, engine)
                    .await
            }
        }
    }

    /// Local-only: delegate entirely to the torrent engine.
    async fn start_local_only(
        &self,
        torrent_id: &str,
        source: &str,
        engine: Option<Arc<TorrentEngine>>,
    ) -> crate::error::Result<StartResult> {
        let engine = engine.ok_or_else(|| {
            TsubasaError::Engine(crate::error::EngineError::Operation(
                "Engine not initialized".to_string(),
            ))
        })?;

        let (info_hash, name, total_bytes) =
            engine.add_torrent(source, torrent_id.to_string()).await?;

        Ok(StartResult {
            info_hash,
            name,
            total_bytes,
            local_started: true,
            cloud_started: false,
        })
    }

    /// Cloud-only: pick the best provider and spawn the cloud lifecycle.
    async fn start_cloud_only(
        &self,
        torrent_id: &str,
        source: &str,
        save_path: &str,
    ) -> crate::error::Result<StartResult> {
        let (provider, provider_name) = self.pick_provider(source).await?;

        let cancel = CancellationToken::new();

        // Track the task before spawning
        {
            let mut tasks = self.active_tasks.write();
            tasks.insert(
                torrent_id.to_string(),
                CloudLifecycleTask {
                    cancel: cancel.clone(),
                    provider_name: provider_name.clone(),
                    cloud_torrent_id: None,
                },
            );
        }

        // Spawn the cloud lifecycle as a background task
        self.spawn_cloud_lifecycle(
            torrent_id.to_string(),
            source.to_string(),
            save_path.to_string(),
            provider,
            provider_name,
            cancel,
        );

        Ok(StartResult {
            info_hash: String::new(),
            name: String::new(),
            total_bytes: 0,
            local_started: false,
            cloud_started: true,
        })
    }

    /// Hybrid (race mode): start both local and cloud simultaneously.
    /// Both run in parallel; the cloud lifecycle will emit its own events.
    /// The local engine handles its own completion detection via the progress loop.
    async fn start_hybrid(
        &self,
        torrent_id: &str,
        source: &str,
        save_path: &str,
        engine: Option<Arc<TorrentEngine>>,
    ) -> crate::error::Result<StartResult> {
        let mut local_started = false;
        let mut cloud_started = false;
        let mut info_hash = String::new();
        let mut name = String::new();
        let mut total_bytes = 0u64;

        // Try to start local
        if let Some(engine) = engine {
            match engine.add_torrent(source, torrent_id.to_string()).await {
                Ok((ih, n, tb)) => {
                    info_hash = ih;
                    name = n;
                    total_bytes = tb;
                    local_started = true;
                }
                Err(e) => {
                    tracing::warn!(
                        torrent_id = %torrent_id,
                        "Hybrid: local engine failed, continuing with cloud only: {e}"
                    );
                }
            }
        }

        // Try to start cloud
        if self.cloud_manager.any_configured() {
            match self.pick_provider(source).await {
                Ok((provider, provider_name)) => {
                    let cancel = CancellationToken::new();

                    {
                        let mut tasks = self.active_tasks.write();
                        tasks.insert(
                            torrent_id.to_string(),
                            CloudLifecycleTask {
                                cancel: cancel.clone(),
                                provider_name: provider_name.clone(),
                                cloud_torrent_id: None,
                            },
                        );
                    }

                    self.spawn_cloud_lifecycle(
                        torrent_id.to_string(),
                        source.to_string(),
                        save_path.to_string(),
                        provider,
                        provider_name,
                        cancel,
                    );

                    cloud_started = true;
                }
                Err(e) => {
                    tracing::warn!(
                        torrent_id = %torrent_id,
                        "Hybrid: cloud provider selection failed: {e}"
                    );
                }
            }
        }

        if !local_started && !cloud_started {
            return Err(TsubasaError::Download(DownloadError::NoPath));
        }

        Ok(StartResult {
            info_hash,
            name,
            total_bytes,
            local_started,
            cloud_started,
        })
    }

    /// Cancel any active cloud tasks for the given torrent.
    pub fn cancel_cloud_tasks(&self, torrent_id: &str) {
        let task = self.active_tasks.write().remove(torrent_id);
        if let Some(task) = task {
            tracing::info!(
                torrent_id = %torrent_id,
                provider = %task.provider_name,
                "Cancelling cloud lifecycle task"
            );
            task.cancel.cancel();
        }
    }

    /// Cancel all active cloud lifecycle tasks (used during shutdown).
    pub fn cancel_all(&self) {
        let mut tasks = self.active_tasks.write();
        let count = tasks.len();
        for (torrent_id, task) in tasks.drain() {
            tracing::info!(
                torrent_id = %torrent_id,
                provider = %task.provider_name,
                "Cancelling cloud lifecycle task (shutdown)"
            );
            task.cancel.cancel();
        }
        if count > 0 {
            tracing::info!(count = count, "Cancelled all active cloud tasks");
        }
    }

    /// Pick the best cloud provider for a given source.
    /// Prefers a provider that has the torrent cached (instant availability).
    /// Falls back to the first configured provider (Torbox preferred).
    async fn pick_provider(
        &self,
        source: &str,
    ) -> crate::error::Result<(Arc<dyn DebridProvider>, String)> {
        let info_hash = extract_info_hash_from_source(source);

        // Collect configured providers (Torbox first, then Real-Debrid)
        let mut providers: Vec<(Arc<dyn DebridProvider>, String)> = Vec::new();
        if self.cloud_manager.torbox.is_configured() {
            providers.push((
                self.cloud_manager.torbox.clone() as Arc<dyn DebridProvider>,
                "torbox".to_string(),
            ));
        }
        if self.cloud_manager.realdebrid.is_configured() {
            providers.push((
                self.cloud_manager.realdebrid.clone() as Arc<dyn DebridProvider>,
                "realdebrid".to_string(),
            ));
        }

        if providers.is_empty() {
            return Err(TsubasaError::Download(DownloadError::NoPath));
        }

        // If we have an info_hash, check cache on all providers
        if let Some(ref hash) = info_hash {
            for (provider, name) in &providers {
                match provider.check_cached(hash).await {
                    Ok(true) => {
                        tracing::info!(
                            provider = %name,
                            info_hash = %hash,
                            "Torrent is cached on provider, selecting it"
                        );
                        return Ok((provider.clone(), name.clone()));
                    }
                    Ok(false) => {}
                    Err(e) => {
                        tracing::warn!(
                            provider = %name,
                            "Cache check failed: {e}"
                        );
                    }
                }
            }
        }

        // No cache hit — return the first configured provider
        let (provider, name) = providers.into_iter().next().unwrap();
        Ok((provider, name))
    }

    /// Spawn the cloud lifecycle as a background tokio task.
    fn spawn_cloud_lifecycle(
        &self,
        torrent_id: String,
        source: String,
        save_path: String,
        provider: Arc<dyn DebridProvider>,
        provider_name: String,
        cancel: CancellationToken,
    ) {
        let event_tx = self.event_tx.clone();
        let db = self.db.clone();
        let http_client = self.cloud_manager.http_client.clone();
        let active_tasks = self.active_tasks.clone();

        tokio::spawn(async move {
            let result = run_cloud_lifecycle(
                &torrent_id,
                &source,
                &save_path,
                provider.as_ref(),
                &provider_name,
                &event_tx,
                &db,
                &http_client,
                &cancel,
                &active_tasks,
            )
            .await;

            // Clean up tracking on completion
            active_tasks.write().remove(&torrent_id);

            match result {
                Ok(()) => {
                    tracing::info!(
                        torrent_id = %torrent_id,
                        provider = %provider_name,
                        "Cloud lifecycle completed successfully"
                    );
                }
                Err(TsubasaError::Download(DownloadError::Cancelled)) => {
                    tracing::info!(
                        torrent_id = %torrent_id,
                        "Cloud lifecycle cancelled"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        torrent_id = %torrent_id,
                        provider = %provider_name,
                        "Cloud lifecycle failed: {e}"
                    );

                    let _ = db.update_torrent_state(&torrent_id, "errored", Some(&e.to_string()));

                    let _ = event_tx.send(TsubasaEvent::Error {
                        torrent_id: Some(torrent_id.clone()),
                        message: format!("Cloud download failed: {e}"),
                        recoverable: true,
                    });

                    let _ = event_tx.send(TsubasaEvent::TorrentStateChanged {
                        id: torrent_id,
                        from: "downloading".to_string(),
                        to: "errored".to_string(),
                    });
                }
            }
        });
    }
}

/// The full cloud download lifecycle, run inside a spawned task.
///
/// Pipeline:
/// 1. Determine TorrentSource from source string
/// 2. Submit to provider
/// 3. Poll status every 3s until Completed/Cached
/// 4. Get download links
/// 5. Download all files via HTTP
/// 6. Update DB state to completed
/// 7. Emit completion events
async fn run_cloud_lifecycle(
    torrent_id: &str,
    source: &str,
    save_path: &str,
    provider: &dyn DebridProvider,
    provider_name: &str,
    event_tx: &broadcast::Sender<TsubasaEvent>,
    db: &Database,
    http_client: &reqwest::Client,
    cancel: &CancellationToken,
    active_tasks: &parking_lot::RwLock<HashMap<String, CloudLifecycleTask>>,
) -> crate::error::Result<()> {
    // Step 1: Build TorrentSource
    let torrent_source = if source.starts_with("magnet:") {
        TorrentSource::MagnetUri(source.to_string())
    } else {
        // It's a .torrent file path — read the bytes
        let bytes = tokio::fs::read(source).await.map_err(|e| {
            TsubasaError::Download(DownloadError::FileWrite(format!(
                "Failed to read torrent file: {e}"
            )))
        })?;
        TorrentSource::TorrentFile(bytes)
    };

    // Step 2: Submit to provider (with retry)
    let retry_config = RetryConfig::cloud_api();
    let cloud_id = retry_with_backoff(
        &retry_config,
        "cloud_add_torrent",
        || async { provider.add_torrent(&torrent_source).await },
    )
    .await?;

    tracing::info!(
        torrent_id = %torrent_id,
        provider = %provider_name,
        cloud_id = %cloud_id.0,
        "Torrent submitted to cloud provider"
    );

    // Store the cloud torrent ID so it can be cleaned up on cancel
    {
        let mut tasks = active_tasks.write();
        if let Some(task) = tasks.get_mut(torrent_id) {
            task.cloud_torrent_id = Some(cloud_id.clone());
        }
    }

    let _ = event_tx.send(TsubasaEvent::CloudStatusChanged {
        torrent_id: torrent_id.to_string(),
        provider: provider_name.to_string(),
        status: "queued".to_string(),
    });

    // Step 3: Poll status until completion
    let poll_interval = std::time::Duration::from_secs(3);
    loop {
        // Check cancellation
        if cancel.is_cancelled() {
            // Clean up: delete from cloud provider
            if let Err(e) = provider.delete_torrent(&cloud_id).await {
                tracing::warn!(
                    torrent_id = %torrent_id,
                    "Failed to delete cloud torrent on cancel: {e}"
                );
            }
            return Err(TsubasaError::Download(DownloadError::Cancelled));
        }

        let status = retry_with_backoff(
            &retry_config,
            "cloud_check_status",
            || async { provider.check_status(&cloud_id).await },
        )
        .await?;

        match status {
            CloudStatus::Completed | CloudStatus::Cached => {
                let _ = event_tx.send(TsubasaEvent::CloudStatusChanged {
                    torrent_id: torrent_id.to_string(),
                    provider: provider_name.to_string(),
                    status: "completed".to_string(),
                });
                break;
            }
            CloudStatus::Downloading { progress } => {
                let _ = event_tx.send(TsubasaEvent::CloudDownloadProgress {
                    torrent_id: torrent_id.to_string(),
                    provider: provider_name.to_string(),
                    progress_pct: progress * 100.0,
                });
            }
            CloudStatus::Queued => {
                // Still queued, keep polling
            }
            CloudStatus::Failed { reason } => {
                return Err(TsubasaError::Cloud {
                    provider: provider_name.to_string(),
                    source: crate::error::CloudError::DownloadFailed(reason),
                });
            }
            CloudStatus::Unknown => {
                tracing::warn!(
                    torrent_id = %torrent_id,
                    "Cloud provider returned unknown status, continuing to poll"
                );
            }
        }

        tokio::select! {
            _ = cancel.cancelled() => {
                if let Err(e) = provider.delete_torrent(&cloud_id).await {
                    tracing::warn!(
                        torrent_id = %torrent_id,
                        "Failed to delete cloud torrent on cancel: {e}"
                    );
                }
                return Err(TsubasaError::Download(DownloadError::Cancelled));
            }
            _ = tokio::time::sleep(poll_interval) => {}
        }
    }

    // Step 4: Get download links (with retry)
    let links = retry_with_backoff(
        &retry_config,
        "cloud_get_download_links",
        || async { provider.get_download_links(&cloud_id).await },
    )
    .await?;

    if links.is_empty() {
        return Err(TsubasaError::Cloud {
            provider: provider_name.to_string(),
            source: crate::error::CloudError::DownloadFailed(
                "No download links returned".to_string(),
            ),
        });
    }

    let total_bytes: u64 = links.iter().map(|l| l.size_bytes).sum();

    tracing::info!(
        torrent_id = %torrent_id,
        files = links.len(),
        total_bytes = total_bytes,
        "Got download links from cloud provider"
    );

    // Update DB with total size info
    let _ = db.update_torrent_progress(torrent_id, 0, 0, total_bytes);

    // Step 5: Download all files via HTTP
    let save_dir = PathBuf::from(save_path);

    let results = download_all_cloud_files(
        http_client,
        &links,
        &save_dir,
        torrent_id,
        provider_name,
        event_tx,
        cancel,
    )
    .await?;

    let downloaded_total: u64 = results.iter().map(|r| r.total_bytes).sum();

    // Step 6: Update DB state to completed
    let _ = db.update_torrent_progress(torrent_id, downloaded_total, 0, total_bytes);
    let _ = db.update_torrent_state(torrent_id, "completed", None);

    // Step 7: Emit completion events
    let _ = event_tx.send(TsubasaEvent::DownloadComplete {
        id: torrent_id.to_string(),
        name: links
            .first()
            .map(|l| l.filename.clone())
            .unwrap_or_default(),
        path: save_dir,
        size_bytes: downloaded_total,
    });

    let _ = event_tx.send(TsubasaEvent::TorrentStateChanged {
        id: torrent_id.to_string(),
        from: "downloading".to_string(),
        to: "completed".to_string(),
    });

    // Clean up: delete from cloud provider to free quota
    if let Err(e) = provider.delete_torrent(&cloud_id).await {
        tracing::warn!(
            torrent_id = %torrent_id,
            "Failed to clean up cloud torrent after download: {e}"
        );
    }

    Ok(())
}

/// Extract an info hash from a magnet URI or return None for .torrent file paths.
fn extract_info_hash_from_source(source: &str) -> Option<String> {
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
