// Tsubasa (翼) — Tracker Auto-Append
// Fetches and caches a tracker list from a remote URL.
// Appended to all new torrents when enabled.

use std::time::{Duration, Instant};

use parking_lot::RwLock;

/// Auto-append tracker list manager.
/// Fetches from a configurable URL (default: ngosang/trackerslist).
pub struct TrackerAutoAppend {
    /// Cached tracker URLs.
    cached_list: RwLock<Vec<String>>,
    /// When the list was last fetched.
    last_fetch: RwLock<Option<Instant>>,
    /// Refresh interval (default: 24 hours).
    refresh_interval: Duration,
}

impl TrackerAutoAppend {
    pub fn new() -> Self {
        Self {
            cached_list: RwLock::new(Vec::new()),
            last_fetch: RwLock::new(None),
            refresh_interval: Duration::from_secs(24 * 60 * 60),
        }
    }

    /// Get the cached tracker list. Returns empty if not fetched yet.
    pub fn get_list(&self) -> Vec<String> {
        self.cached_list.read().clone()
    }

    /// Check if the list needs refreshing.
    pub fn needs_refresh(&self) -> bool {
        match *self.last_fetch.read() {
            Some(t) => t.elapsed() > self.refresh_interval,
            None => true,
        }
    }

    /// Fetch tracker list from URL and cache it.
    pub async fn refresh(&self, url: &str, client: &reqwest::Client) -> Result<usize, String> {
        let resp = client
            .get(url)
            .header("User-Agent", "Tsubasa/0.1.0")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch tracker list: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Tracker list HTTP {}", resp.status()));
        }

        let body = resp.text().await
            .map_err(|e| format!("Failed to read tracker list: {e}"))?;

        let trackers: Vec<String> = body.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && (l.starts_with("udp://") || l.starts_with("http://") || l.starts_with("https://")))
            .collect();

        let count = trackers.len();
        *self.cached_list.write() = trackers;
        *self.last_fetch.write() = Some(Instant::now());

        tracing::info!(count, "Refreshed auto-append tracker list");
        Ok(count)
    }

    /// Set the cached list directly (e.g., from persisted settings).
    pub fn set_list(&self, list: Vec<String>) {
        *self.cached_list.write() = list;
        *self.last_fetch.write() = Some(Instant::now());
    }
}
