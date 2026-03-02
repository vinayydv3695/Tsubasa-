// Tsubasa — Application State
// Central state container shared across all Tauri commands and background tasks.

use std::sync::Arc;
use std::time::Instant;

use parking_lot::RwLock;

use crate::cloud::CloudManager;
use crate::download::DownloadOrchestrator;
use crate::engine::TorrentEngine;
use crate::events::EventBus;
use crate::logging::ring_buffer::LogRingBuffer;
use crate::search::SearchEngine;
use crate::settings::SettingsManager;
use crate::storage::database::Database;
use crate::storage::session::SessionManager;

/// Shared application state accessible from all Tauri commands.
/// Wraps all subsystems and provides thread-safe access.
pub struct AppState {
    /// SQLite database handle.
    pub db: Database,

    /// Central event bus.
    pub event_bus: EventBus,

    /// Torrent engine (initialized asynchronously after app starts).
    pub engine: RwLock<Option<Arc<TorrentEngine>>>,

    /// Session file manager.
    pub session_manager: RwLock<Option<SessionManager>>,

    /// Cloud debrid provider manager.
    pub cloud_manager: Arc<CloudManager>,

    /// Download orchestrator — routes downloads by policy.
    pub orchestrator: Arc<DownloadOrchestrator>,

    /// Torrent search engine (Torbox Search API).
    pub search_engine: SearchEngine,

    /// In-memory log ring buffer for the UI viewer.
    pub log_buffer: Arc<LogRingBuffer>,

    /// Grouped settings manager (v2).
    pub settings_manager: Arc<SettingsManager>,

    /// Timestamp when the app started.
    pub started_at: Instant,
}

impl AppState {
    /// Create a new AppState with all subsystems.
    pub fn new(
        db: Database,
        log_buffer: Arc<LogRingBuffer>,
        cloud_manager: Arc<CloudManager>,
        event_bus: EventBus,
        orchestrator: DownloadOrchestrator,
        search_engine: SearchEngine,
        settings_manager: Arc<SettingsManager>,
    ) -> Self {
        Self {
            db,
            event_bus,
            engine: RwLock::new(None),
            session_manager: RwLock::new(None),
            cloud_manager,
            orchestrator: Arc::new(orchestrator),
            search_engine,
            log_buffer,
            settings_manager,
            started_at: Instant::now(),
        }
    }
}
