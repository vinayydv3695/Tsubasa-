// Tsubasa (翼) — Event Bus
// Central event system using tokio broadcast channels.
// All subsystems publish events, the Tauri bridge forwards them to the frontend.

use serde::Serialize;
use std::path::PathBuf;
use tokio::sync::broadcast;

/// Capacity of the event broadcast channel.
/// Events beyond this are dropped for slow receivers.
const EVENT_CHANNEL_CAPACITY: usize = 4096;

/// All events that flow through the Tsubasa event system.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum TsubasaEvent {
    // Torrent lifecycle
    TorrentAdded {
        id: String,
        name: String,
        info_hash: String,
    },
    TorrentRemoved {
        id: String,
    },
    TorrentStateChanged {
        id: String,
        from: String,
        to: String,
    },

    // Progress
    ProgressUpdate {
        id: String,
        downloaded_bytes: u64,
        total_bytes: u64,
        download_speed: f64,
        upload_speed: f64,
        peers_connected: u32,
        seeds_connected: u32,
    },

    // Peer events
    PeerConnected {
        torrent_id: String,
        address: String,
        client: String,
    },
    PeerDisconnected {
        torrent_id: String,
        address: String,
    },

    // Cloud events
    CloudStatusChanged {
        torrent_id: String,
        provider: String,
        status: String,
    },
    CloudDownloadProgress {
        torrent_id: String,
        provider: String,
        progress_pct: f64,
    },

    // Completion
    DownloadComplete {
        id: String,
        name: String,
        path: PathBuf,
        size_bytes: u64,
    },

    // Errors
    Error {
        torrent_id: Option<String>,
        message: String,
        recoverable: bool,
    },

    // Logging
    LogEntry {
        level: String,
        target: String,
        message: String,
        timestamp: String,
    },

    // Engine status
    EngineReady,
    EngineShuttingDown,

    // Stats
    GlobalStats {
        total_download_speed: f64,
        total_upload_speed: f64,
        active_torrents: u32,
        total_peers: u32,
    },
}

/// Central event bus for the application.
/// Uses tokio broadcast for multi-consumer event distribution.
#[derive(Debug)]
pub struct EventBus {
    sender: broadcast::Sender<TsubasaEvent>,
}

impl EventBus {
    /// Create a new event bus.
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self { sender }
    }

    /// Publish an event to all subscribers.
    /// Returns Ok(num_receivers) or silently succeeds with 0 if no receivers.
    pub fn publish(&self, event: TsubasaEvent) -> usize {
        // If no receivers are listening, send returns an error.
        // This is not a failure condition — it just means nobody cares yet.
        self.sender.send(event).unwrap_or(0)
    }

    /// Subscribe to all events.
    pub fn subscribe(&self) -> broadcast::Receiver<TsubasaEvent> {
        self.sender.subscribe()
    }

    /// Get a clone of the sender for subsystems to hold.
    pub fn sender(&self) -> broadcast::Sender<TsubasaEvent> {
        self.sender.clone()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
