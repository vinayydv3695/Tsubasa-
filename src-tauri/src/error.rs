// Tsubasa (翼) — Error Types
// Unified error hierarchy for the entire application.

use std::time::Duration;

/// Top-level application error type.
/// Every subsystem error funnels through here.
#[derive(Debug, thiserror::Error)]
pub enum TsubasaError {
    #[error("Torrent engine error: {0}")]
    Engine(#[from] EngineError),

    #[error("Cloud provider error ({provider}): {source}")]
    Cloud {
        provider: String,
        #[source]
        source: CloudError,
    },

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Invalid state transition: {from} -> {to}")]
    InvalidTransition { from: String, to: String },

    #[error("Operation timed out after {duration:?}: {operation}")]
    Timeout {
        operation: String,
        duration: Duration,
    },

    #[error("Download error: {0}")]
    Download(#[from] DownloadError),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Internal(String),
}

/// Torrent engine specific errors.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Failed to initialize torrent session: {0}")]
    SessionInit(String),

    #[error("Invalid torrent source: {0}")]
    InvalidSource(String),

    #[error("Torrent not found: {0}")]
    TorrentNotFound(String),

    #[error("Peer connection failed: {0}")]
    PeerConnection(String),

    #[error("Tracker announce failed: {0}")]
    TrackerAnnounce(String),

    #[error("Piece verification failed for piece {piece_index}")]
    PieceVerification { piece_index: u32 },

    #[error("Disk I/O error: {0}")]
    DiskIo(String),

    #[error("Metadata fetch failed: {0}")]
    MetadataFetch(String),

    #[error("Engine operation error: {0}")]
    Operation(String),
}

/// Cloud/debrid provider errors.
#[derive(Debug, thiserror::Error)]
pub enum CloudError {
    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("API request failed: {0}")]
    ApiRequest(String),

    #[error("Rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("Torrent not cached on cloud")]
    NotCached,

    #[error("Cloud download failed: {0}")]
    DownloadFailed(String),

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Quota exceeded")]
    QuotaExceeded,

    #[error("Provider unavailable: {0}")]
    Unavailable(String),
}

/// Database errors.
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Migration failed: {0}")]
    Migration(String),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Constraint violation: {0}")]
    Constraint(String),
}

/// Download orchestration errors.
#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("No download path available for policy")]
    NoPath,

    #[error("HTTP download failed: {0}")]
    HttpFetch(String),

    #[error("File write failed: {0}")]
    FileWrite(String),

    #[error("Checksum mismatch")]
    ChecksumMismatch,

    #[error("Cancelled")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, TsubasaError>;

/// Make TsubasaError serializable for Tauri IPC.
impl serde::Serialize for TsubasaError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
