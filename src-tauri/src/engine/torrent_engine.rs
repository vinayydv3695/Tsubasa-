// Tsubasa (翼) — Torrent Engine
// Wraps librqbit::Session to provide Tsubasa-specific torrent management.
// Includes a progress polling loop that emits ProgressUpdate and GlobalStats events.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use librqbit::session_stats::snapshot::SessionStatsSnapshot;
use librqbit::{
    AddTorrent, AddTorrentOptions, AddTorrentResponse, ManagedTorrent as LibManagedTorrent,
    Session, SessionOptions, SessionPersistenceConfig,
};
use parking_lot::RwLock;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::bandwidth::BandwidthConfig;
use crate::download::state_machine::TorrentState;
use crate::error::{EngineError, TsubasaError};
use crate::events::TsubasaEvent;

/// How often the progress polling loop runs (milliseconds).
const POLL_INTERVAL_MS: u64 = 500;

/// Internal tracking of an active torrent within the engine.
pub struct TrackedTorrent {
    pub id: String,
    pub librqbit_id: usize,
    pub info_hash: String,
    pub name: String,
    pub state: TorrentState,
    pub handle: Arc<LibManagedTorrent>,
    pub total_bytes: u64,
    /// Whether we already emitted a DownloadComplete event for this torrent.
    pub completion_notified: bool,
}

/// The Tsubasa torrent engine.
/// Manages the librqbit session and maps Tsubasa torrent IDs to librqbit handles.
pub struct TorrentEngine {
    session: Arc<Session>,
    torrents: Arc<RwLock<HashMap<String, TrackedTorrent>>>,
    event_tx: broadcast::Sender<TsubasaEvent>,
    cancel_token: CancellationToken,
}

impl TorrentEngine {
    /// Initialize the torrent engine with the given download directory.
    /// Enables librqbit's built-in session persistence and fastresume.
    /// Applies initial bandwidth limits from the config.
    /// Spawns a background progress polling loop.
    pub async fn new(
        download_dir: PathBuf,
        event_tx: broadcast::Sender<TsubasaEvent>,
        cancel_token: CancellationToken,
        bandwidth: BandwidthConfig,
        session_dir: PathBuf,
    ) -> crate::error::Result<Self> {
        let session = Session::new_with_opts(
            download_dir,
            SessionOptions {
                disable_dht: false,
                disable_dht_persistence: false,
                fastresume: true,
                persistence: Some(SessionPersistenceConfig::Json {
                    folder: Some(session_dir),
                }),
                cancellation_token: Some(cancel_token.clone()),
                ratelimits: bandwidth.to_librqbit_limits(),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| TsubasaError::Engine(EngineError::SessionInit(e.to_string())))?;

        let torrents = Arc::new(RwLock::new(HashMap::new()));

        let engine = Self {
            session,
            torrents: torrents.clone(),
            event_tx: event_tx.clone(),
            cancel_token: cancel_token.clone(),
        };

        // Spawn the progress polling loop
        Self::spawn_progress_loop(torrents, event_tx, cancel_token);

        Ok(engine)
    }

    /// Spawns a tokio task that polls all tracked torrents at a fixed interval
    /// and emits ProgressUpdate events + GlobalStats.
    fn spawn_progress_loop(
        torrents: Arc<RwLock<HashMap<String, TrackedTorrent>>>,
        event_tx: broadcast::Sender<TsubasaEvent>,
        cancel_token: CancellationToken,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(POLL_INTERVAL_MS));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("Progress polling loop shutting down");
                        break;
                    }
                    _ = interval.tick() => {
                        Self::poll_progress(&torrents, &event_tx);
                    }
                }
            }
        });
    }

    /// Poll all tracked torrents for progress and emit events.
    fn poll_progress(
        torrents: &Arc<RwLock<HashMap<String, TrackedTorrent>>>,
        event_tx: &broadcast::Sender<TsubasaEvent>,
    ) {
        let mut total_download_speed: f64 = 0.0;
        let mut total_upload_speed: f64 = 0.0;
        let mut active_torrents: u32 = 0;
        let mut total_peers: u32 = 0;

        // Collect progress data while holding the read lock briefly
        let updates: Vec<(String, TsubasaEvent)> = {
            let guard = torrents.read();
            guard
                .iter()
                .filter(|(_, t)| t.state.is_active())
                .map(|(id, tracked)| {
                    let stats = tracked.handle.stats();

                    let (dl_speed, ul_speed, peers, seeds, _eta) =
                        if let Some(ref live) = stats.live {
                            let dl = live.download_speed.mbps * 1_000_000.0 / 8.0; // Mbps -> bytes/s
                            let ul = live.upload_speed.mbps * 1_000_000.0 / 8.0;
                            let peer_count = live.snapshot.peer_stats.live as u32;
                            let seed_count = 0u32;
                            // Compute ETA from remaining bytes and download speed
                            let remaining = stats.total_bytes.saturating_sub(stats.progress_bytes);
                            let eta_secs = if dl > 0.0 && remaining > 0 {
                                Some((remaining as f64 / dl) as u64)
                            } else {
                                None
                            };
                            (dl, ul, peer_count, seed_count, eta_secs)
                        } else {
                            (0.0, 0.0, 0, 0, None)
                        };

                    total_download_speed += dl_speed;
                    total_upload_speed += ul_speed;
                    total_peers += peers;
                    if !stats.finished {
                        active_torrents += 1;
                    }

                    let event = TsubasaEvent::ProgressUpdate {
                        id: id.clone(),
                        downloaded_bytes: stats.progress_bytes,
                        total_bytes: stats.total_bytes,
                        download_speed: dl_speed,
                        upload_speed: ul_speed,
                        peers_connected: peers,
                        seeds_connected: seeds,
                    };

                    (id.clone(), event)
                })
                .collect()
        };

        // Emit per-torrent progress events
        for (_id, event) in &updates {
            let _ = event_tx.send(event.clone());
        }

        // Check for completions — needs write lock
        {
            let mut guard = torrents.write();
            for (id, _event) in &updates {
                if let Some(tracked) = guard.get_mut(id) {
                    let stats = tracked.handle.stats();
                    if stats.finished && !tracked.completion_notified {
                        tracked.completion_notified = true;
                        tracked.state = TorrentState::Seeding;

                        let _ = event_tx.send(TsubasaEvent::DownloadComplete {
                            id: id.clone(),
                            name: tracked.name.clone(),
                            path: PathBuf::from(""), // actual path managed by librqbit session
                            size_bytes: stats.total_bytes,
                        });

                        let _ = event_tx.send(TsubasaEvent::TorrentStateChanged {
                            id: id.clone(),
                            from: "downloading".to_string(),
                            to: "seeding".to_string(),
                        });

                        tracing::info!(
                            torrent_id = %id,
                            name = %tracked.name,
                            "Torrent download complete, now seeding"
                        );
                    }
                }
            }
        }

        // Emit global stats
        let _ = event_tx.send(TsubasaEvent::GlobalStats {
            total_download_speed,
            total_upload_speed,
            active_torrents,
            total_peers,
        });
    }

    /// Add a torrent from a magnet URI or .torrent file path.
    pub async fn add_torrent(
        &self,
        source: &str,
        tsubasa_id: String,
    ) -> crate::error::Result<(String, String, u64)> {
        let add_torrent = if source.starts_with("magnet:") {
            AddTorrent::from_url(source)
        } else {
            let bytes = std::fs::read(source)
                .map_err(|e| TsubasaError::Engine(EngineError::InvalidSource(e.to_string())))?;
            AddTorrent::from_bytes(bytes)
        };

        let opts = Some(AddTorrentOptions {
            overwrite: true,
            ..Default::default()
        });

        let response = self
            .session
            .add_torrent(add_torrent, opts)
            .await
            .map_err(|e| TsubasaError::Engine(EngineError::Operation(format!("{e:#}"))))?;

        let (librqbit_id, handle) = match response {
            AddTorrentResponse::Added(id, handle) => (id, handle),
            AddTorrentResponse::AlreadyManaged(id, handle) => (id, handle),
            AddTorrentResponse::ListOnly(_) => {
                return Err(TsubasaError::Engine(EngineError::Operation(
                    "Torrent was list-only, not added".to_string(),
                )));
            }
        };

        let info_hash = handle.info_hash().as_string();
        let name = handle
            .name()
            .unwrap_or_else(|| format!("Torrent {}", &tsubasa_id[..8.min(tsubasa_id.len())]));
        let total_bytes = handle.stats().total_bytes;

        let managed = TrackedTorrent {
            id: tsubasa_id.clone(),
            librqbit_id,
            info_hash: info_hash.clone(),
            name: name.clone(),
            state: TorrentState::Downloading,
            handle,
            total_bytes,
            completion_notified: false,
        };

        self.torrents.write().insert(tsubasa_id.clone(), managed);

        let _ = self.event_tx.send(TsubasaEvent::TorrentAdded {
            id: tsubasa_id,
            name: name.clone(),
            info_hash: info_hash.clone(),
        });

        Ok((info_hash, name, total_bytes))
    }

    /// Pause a torrent. Updates both the librqbit session and internal tracking.
    pub async fn pause_torrent(&self, tsubasa_id: &str) -> crate::error::Result<()> {
        let handle = {
            let guard = self.torrents.read();
            guard
                .get(tsubasa_id)
                .map(|t| t.handle.clone())
                .ok_or_else(|| {
                    TsubasaError::Engine(EngineError::TorrentNotFound(tsubasa_id.to_string()))
                })?
        };

        self.session.pause(&handle).await.map_err(|e| {
            TsubasaError::Engine(EngineError::Operation(format!("Failed to pause: {e:#}")))
        })?;

        // Update internal state
        if let Some(tracked) = self.torrents.write().get_mut(tsubasa_id) {
            tracked.state = TorrentState::Paused;
        }

        Ok(())
    }

    /// Resume a paused torrent.
    pub async fn resume_torrent(&self, tsubasa_id: &str) -> crate::error::Result<()> {
        let handle = {
            let guard = self.torrents.read();
            guard
                .get(tsubasa_id)
                .map(|t| t.handle.clone())
                .ok_or_else(|| {
                    TsubasaError::Engine(EngineError::TorrentNotFound(tsubasa_id.to_string()))
                })?
        };

        self.session.unpause(&handle).await.map_err(|e| {
            TsubasaError::Engine(EngineError::Operation(format!("Failed to resume: {e:#}")))
        })?;

        // Update internal state
        if let Some(tracked) = self.torrents.write().get_mut(tsubasa_id) {
            tracked.state = TorrentState::Downloading;
        }

        Ok(())
    }

    /// Remove a torrent from the engine and optionally delete its files.
    pub async fn delete_torrent(
        &self,
        tsubasa_id: &str,
        delete_files: bool,
    ) -> crate::error::Result<Option<TrackedTorrent>> {
        let tracked = self.torrents.write().remove(tsubasa_id);

        if let Some(ref t) = tracked {
            // Use librqbit_id (TorrentId = usize) to delete from session
            if let Err(e) = self
                .session
                .delete(librqbit::api::TorrentIdOrHash::Id(t.librqbit_id), delete_files)
                .await
            {
                tracing::warn!(
                    torrent_id = %tsubasa_id,
                    "Failed to delete from librqbit session: {e:#}"
                );
            }
        }

        Ok(tracked)
    }

    /// Get stats for a specific torrent by Tsubasa ID.
    pub fn get_torrent_stats(
        &self,
        tsubasa_id: &str,
    ) -> Option<librqbit::TorrentStats> {
        let guard = self.torrents.read();
        guard.get(tsubasa_id).map(|t| t.handle.stats())
    }

    /// Get session-level statistics.
    pub fn get_session_stats(&self) -> SessionStatsSnapshot {
        self.session.stats_snapshot()
    }

    /// Get the inner librqbit session for direct access when needed.
    pub fn session(&self) -> &Arc<Session> {
        &self.session
    }

    /// Get a managed torrent handle by Tsubasa ID.
    pub fn get_torrent(&self, id: &str) -> Option<Arc<LibManagedTorrent>> {
        self.torrents.read().get(id).map(|t| t.handle.clone())
    }

    /// Get a snapshot of all tracked torrent IDs.
    pub fn tracked_ids(&self) -> Vec<String> {
        self.torrents.read().keys().cloned().collect()
    }

    /// Get a snapshot of progress for all tracked torrents.
    /// Returns (tsubasa_id, downloaded_bytes, uploaded_bytes, total_bytes) tuples.
    pub fn all_torrent_progress(&self) -> Vec<(String, u64, u64, u64)> {
        let guard = self.torrents.read();
        guard
            .values()
            .map(|t| {
                let stats = t.handle.stats();
                (
                    t.id.clone(),
                    stats.progress_bytes,
                    stats.uploaded_bytes,
                    stats.total_bytes,
                )
            })
            .collect()
    }

    /// Remove a torrent from the engine's tracking only (no librqbit deletion).
    pub fn remove_torrent(&self, id: &str) -> Option<TrackedTorrent> {
        self.torrents.write().remove(id)
    }

    /// Signal the engine to shut down gracefully.
    pub fn shutdown(&self) {
        self.cancel_token.cancel();
        let _ = self.event_tx.send(TsubasaEvent::EngineShuttingDown);
        tracing::info!("Torrent engine shutdown requested");
    }

    /// Update session-wide bandwidth limits at runtime.
    /// Changes take effect immediately (atomic swap of rate limiters).
    pub fn update_bandwidth(&self, config: &BandwidthConfig) {
        crate::bandwidth::apply_session_limits(&self.session, config);
    }

    /// Get the file list for a torrent, including per-file download progress.
    /// Returns None if the torrent is not tracked.
    pub fn get_torrent_files(
        &self,
        tsubasa_id: &str,
    ) -> crate::error::Result<Vec<crate::storage::models::TorrentFileInfo>> {
        let guard = self.torrents.read();
        let tracked = guard.get(tsubasa_id).ok_or_else(|| {
            TsubasaError::Engine(EngineError::TorrentNotFound(tsubasa_id.to_string()))
        })?;

        let stats = tracked.handle.stats();
        let file_progress = &stats.file_progress;

        // Get file metadata via with_metadata
        let file_infos = tracked
            .handle
            .with_metadata(|m| m.file_infos.clone())
            .map_err(|e| {
                TsubasaError::Engine(EngineError::Operation(format!(
                    "Metadata not available: {e:#}"
                )))
            })?;

        let result = file_infos
            .iter()
            .enumerate()
            .map(|(i, fi)| {
                let path = fi.relative_filename.to_string_lossy().to_string();
                let name = fi
                    .relative_filename
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone());
                let downloaded = file_progress.get(i).copied().unwrap_or(0);
                let progress = if fi.len > 0 {
                    downloaded as f64 / fi.len as f64
                } else {
                    1.0
                };

                crate::storage::models::TorrentFileInfo {
                    index: i,
                    path,
                    name,
                    size: fi.len,
                    downloaded,
                    progress,
                }
            })
            .collect();

        Ok(result)
    }

    /// Get the connected peers for a torrent.
    /// Returns an empty list if the torrent has no live state (e.g. paused, initializing).
    pub fn get_torrent_peers(
        &self,
        tsubasa_id: &str,
    ) -> crate::error::Result<Vec<crate::storage::models::TorrentPeerInfo>> {
        let guard = self.torrents.read();
        let tracked = guard.get(tsubasa_id).ok_or_else(|| {
            TsubasaError::Engine(EngineError::TorrentNotFound(tsubasa_id.to_string()))
        })?;

        let live = match tracked.handle.live() {
            Some(live) => live,
            None => return Ok(Vec::new()),
        };

        // PeerStatsFilter is not publicly re-exported from librqbit 8.1.1,
        // but it derives Default. Rust infers the type from the method signature.
        let snapshot = live.per_peer_stats_snapshot(Default::default());

        let peers = snapshot
            .peers
            .into_iter()
            .map(|(addr, stats)| crate::storage::models::TorrentPeerInfo {
                address: addr,
                state: stats.state.to_string(),
                downloaded_bytes: stats.counters.fetched_bytes,
                uploaded_bytes: 0, // PeerCounters doesn't expose uploaded_bytes per-peer
                connection_attempts: stats.counters.connection_attempts,
                errors: stats.counters.errors,
            })
            .collect();

        Ok(peers)
    }

    /// Get the tracker list for a torrent.
    /// Note: librqbit 8.1.1 only exposes tracker URLs, not per-tracker announce status.
    pub fn get_torrent_trackers(
        &self,
        tsubasa_id: &str,
    ) -> crate::error::Result<Vec<crate::storage::models::TorrentTrackerInfo>> {
        let guard = self.torrents.read();
        let tracked = guard.get(tsubasa_id).ok_or_else(|| {
            TsubasaError::Engine(EngineError::TorrentNotFound(tsubasa_id.to_string()))
        })?;

        let trackers = tracked
            .handle
            .shared()
            .trackers
            .iter()
            .map(|url| crate::storage::models::TorrentTrackerInfo {
                url: url.to_string(),
                status: "unknown".to_string(),
            })
            .collect();

        Ok(trackers)
    }
}
