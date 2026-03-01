// Tsubasa (翼) — Bandwidth Management
// Delegates rate limiting to librqbit's built-in governor-based token bucket.
// Provides conversion between Tsubasa config and librqbit LimitsConfig,
// and methods to apply limits at session-wide and per-torrent levels.

use std::num::NonZeroU32;

use librqbit::limits::LimitsConfig;
use serde::{Deserialize, Serialize};

/// Global bandwidth configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthConfig {
    /// Global download speed limit in bytes/sec, 0 = unlimited
    pub download_limit: u64,
    /// Global upload speed limit in bytes/sec, 0 = unlimited
    pub upload_limit: u64,
    /// Maximum number of global connections
    pub max_connections: u32,
    /// Maximum connections per torrent
    pub max_connections_per_torrent: u32,
}

impl Default for BandwidthConfig {
    fn default() -> Self {
        Self {
            download_limit: 0,
            upload_limit: 0,
            max_connections: 500,
            max_connections_per_torrent: 100,
        }
    }
}

impl BandwidthConfig {
    /// Convert to librqbit's LimitsConfig for session-wide rate limiting.
    /// A value of 0 means unlimited (None in LimitsConfig).
    pub fn to_librqbit_limits(&self) -> LimitsConfig {
        LimitsConfig {
            download_bps: NonZeroU32::new(self.download_limit as u32),
            upload_bps: NonZeroU32::new(self.upload_limit as u32),
        }
    }
}

/// Per-torrent bandwidth override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentBandwidthConfig {
    /// Download speed limit in bytes/sec for this torrent, 0 = use global
    pub download_limit: u64,
    /// Upload speed limit in bytes/sec for this torrent, 0 = use global
    pub upload_limit: u64,
}

impl Default for TorrentBandwidthConfig {
    fn default() -> Self {
        Self {
            download_limit: 0,
            upload_limit: 0,
        }
    }
}

impl TorrentBandwidthConfig {
    /// Convert to librqbit's LimitsConfig for per-torrent rate limiting.
    pub fn to_librqbit_limits(&self) -> LimitsConfig {
        LimitsConfig {
            download_bps: NonZeroU32::new(self.download_limit as u32),
            upload_bps: NonZeroU32::new(self.upload_limit as u32),
        }
    }
}

/// Apply bandwidth limits to a running librqbit session.
/// This updates the session's rate limiters at runtime (atomic swap, no restart needed).
pub fn apply_session_limits(session: &librqbit::Session, config: &BandwidthConfig) {
    let dl = NonZeroU32::new(config.download_limit as u32);
    let ul = NonZeroU32::new(config.upload_limit as u32);

    session.ratelimits.set_download_bps(dl);
    session.ratelimits.set_upload_bps(ul);

    tracing::info!(
        download_limit = ?config.download_limit,
        upload_limit = ?config.upload_limit,
        "Session bandwidth limits updated"
    );
}
