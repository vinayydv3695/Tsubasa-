// Tsubasa (翼) — Database Models
// Type-safe representations of database rows.

use serde::{Deserialize, Serialize};

use crate::download::state_machine::{DownloadPolicy, TorrentState};

/// A torrent record as stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentRecord {
    pub id: String,
    pub info_hash: String,
    pub name: String,
    pub state: TorrentState,
    pub policy: DownloadPolicy,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub uploaded_bytes: u64,
    pub save_path: String,
    pub magnet_uri: Option<String>,
    pub added_at: String,
    pub completed_at: Option<String>,
    pub download_speed_limit: u64,
    pub upload_speed_limit: u64,
    pub max_peers: u32,
    pub ratio_limit: f64,
    pub error_message: Option<String>,
}

/// Application settings as stored in key-value pairs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingRecord {
    pub key: String,
    pub value: String,
}

/// Global application settings with typed fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_dir: String,
    pub default_policy: DownloadPolicy,
    pub global_download_limit: u64,
    pub global_upload_limit: u64,
    pub max_active_torrents: u32,
    pub max_active_downloads: u32,
    pub max_active_seeds: u32,
    pub default_ratio_limit: f64,
    pub enable_dht: bool,
    pub enable_pex: bool,
    pub listen_port: u16,
    pub torbox_api_key: Option<String>,
    pub realdebrid_api_key: Option<String>,
    pub onboarding_completed: bool,
    pub theme: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
            .to_string_lossy()
            .to_string();

        Self {
            download_dir,
            default_policy: DownloadPolicy::LocalOnly,
            global_download_limit: 0,
            global_upload_limit: 0,
            max_active_torrents: 10,
            max_active_downloads: 5,
            max_active_seeds: 5,
            default_ratio_limit: 2.0,
            enable_dht: true,
            enable_pex: true,
            listen_port: 6881,
            torbox_api_key: None,
            realdebrid_api_key: None,
            onboarding_completed: false,
            theme: "dark".to_string(),
        }
    }
}

/// Torrent statistics for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentStats {
    pub id: String,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub peers_connected: u32,
    pub seeds_connected: u32,
    pub availability: f64,
    pub progress: f64,
    pub eta_seconds: Option<u64>,
}

// ─── Detail Panel Response Types ────────────────────────

/// Information about a single file within a torrent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFileInfo {
    /// Index of the file within the torrent (0-based).
    pub index: usize,
    /// Relative file path within the torrent (e.g. "folder/subfolder/file.mkv").
    pub path: String,
    /// File name (last component of the path).
    pub name: String,
    /// Total size in bytes.
    pub size: u64,
    /// Bytes downloaded for this file.
    pub downloaded: u64,
    /// Download progress as a fraction (0.0 to 1.0).
    pub progress: f64,
}

/// Information about a connected peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentPeerInfo {
    /// Peer address as "ip:port".
    pub address: String,
    /// Peer connection state (e.g. "live", "connecting", "queued").
    pub state: String,
    /// Total bytes downloaded from this peer.
    pub downloaded_bytes: u64,
    /// Total bytes uploaded to this peer.
    pub uploaded_bytes: u64,
    /// Number of connection attempts to this peer.
    pub connection_attempts: u32,
    /// Number of errors with this peer.
    pub errors: u32,
}

/// Information about a tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentTrackerInfo {
    /// Tracker URL.
    pub url: String,
    /// Tracker status. librqbit 8.1.1 does not expose per-tracker announce status,
    /// so this will always be "unknown".
    pub status: String,
}

/// Summary view of a torrent for the frontend table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentSummary {
    pub id: String,
    pub info_hash: String,
    pub name: String,
    pub state: TorrentState,
    pub policy: DownloadPolicy,
    pub progress: f64,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub uploaded_bytes: u64,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub peers_connected: u32,
    pub seeds_connected: u32,
    pub eta_seconds: Option<u64>,
    pub ratio: f64,
    pub added_at: String,
    pub save_path: String,
    pub error_message: Option<String>,
}
