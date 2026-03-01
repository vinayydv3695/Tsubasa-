// Tsubasa (翼) — Debrid Provider Trait
// All cloud torrent providers implement this interface.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Unique identifier for a torrent on a cloud provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudTorrentId(pub String);

/// Status of a torrent on a cloud provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudStatus {
    /// Torrent is queued on the provider
    Queued,
    /// Provider is downloading the torrent
    Downloading { progress: f64 },
    /// Provider has fully downloaded the torrent
    Completed,
    /// The torrent is cached (instant availability)
    Cached,
    /// Download failed on the provider side
    Failed { reason: String },
    /// Unknown status
    Unknown,
}

/// A direct download link from the cloud provider's CDN.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectLink {
    pub filename: String,
    pub url: String,
    pub size_bytes: u64,
}

/// Cloud provider account information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub provider: String,
    pub username: String,
    pub plan: String,
    pub expiry: Option<String>,
    pub storage_used: u64,
    pub storage_total: u64,
    pub points_used: Option<u64>,
    pub points_total: Option<u64>,
}

/// Source of a torrent for cloud submission.
#[derive(Debug, Clone)]
pub enum TorrentSource {
    MagnetUri(String),
    InfoHash(String),
    TorrentFile(Vec<u8>),
}

/// The abstract interface all debrid providers implement.
#[async_trait]
pub trait DebridProvider: Send + Sync {
    /// Human-readable provider name.
    fn name(&self) -> &str;

    /// Check if the provider is configured (has API key).
    fn is_configured(&self) -> bool;

    /// Submit a torrent for cloud downloading.
    async fn add_torrent(
        &self,
        source: &TorrentSource,
    ) -> crate::error::Result<CloudTorrentId>;

    /// Check the status of a submitted torrent.
    async fn check_status(
        &self,
        id: &CloudTorrentId,
    ) -> crate::error::Result<CloudStatus>;

    /// Get direct download links for a completed torrent.
    async fn get_download_links(
        &self,
        id: &CloudTorrentId,
    ) -> crate::error::Result<Vec<DirectLink>>;

    /// Check if a torrent is cached (instant download).
    async fn check_cached(
        &self,
        info_hash: &str,
    ) -> crate::error::Result<bool>;

    /// Get account information (quota, plan, etc.).
    async fn account_info(&self) -> crate::error::Result<AccountInfo>;

    /// Delete a torrent from the cloud provider.
    async fn delete_torrent(
        &self,
        id: &CloudTorrentId,
    ) -> crate::error::Result<()>;
}
