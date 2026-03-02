// Tsubasa (翼) — Tracker Manager
// Per-torrent tracker state: tiers, announce status, health scoring.
// Integrates with librqbit's tracker system.

use std::collections::HashMap;
use std::time::Instant;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

// ─── Types ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackerStatus {
    Working,
    Updating,
    NotContacted,
    Error,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerEntry {
    pub url: String,
    pub tier: u8,
    pub status: TrackerStatus,
    pub seeders: u32,
    pub leechers: u32,
    pub downloaded: u32,
    pub last_announce_secs_ago: Option<u64>,
    pub next_announce_secs: Option<u64>,
    pub last_error: Option<String>,
    pub announce_count: u32,
    pub fail_count: u32,
    pub consecutive_fails: u32,
    /// Health score: (success * 10) - (fail * 5) - (consecutive_fails * 20)
    pub health_score: i32,
}

impl TrackerEntry {
    pub fn new(url: String, tier: u8) -> Self {
        Self {
            url,
            tier,
            status: TrackerStatus::NotContacted,
            seeders: 0,
            leechers: 0,
            downloaded: 0,
            last_announce_secs_ago: None,
            next_announce_secs: None,
            last_error: None,
            announce_count: 0,
            fail_count: 0,
            consecutive_fails: 0,
            health_score: 0,
        }
    }

    /// Update after a successful announce.
    pub fn record_success(&mut self, seeders: u32, leechers: u32, downloaded: u32) {
        self.status = TrackerStatus::Working;
        self.seeders = seeders;
        self.leechers = leechers;
        self.downloaded = downloaded;
        self.announce_count += 1;
        self.consecutive_fails = 0;
        self.last_error = None;
        self.recalc_health();
    }

    /// Update after a failed announce.
    pub fn record_failure(&mut self, error: String) {
        self.status = TrackerStatus::Error;
        self.fail_count += 1;
        self.consecutive_fails += 1;
        self.last_error = Some(error);
        self.recalc_health();

        // Auto-disable after too many consecutive failures
        if self.health_score < -50 {
            self.status = TrackerStatus::Disabled;
        }
    }

    fn recalc_health(&mut self) {
        self.health_score = (self.announce_count as i32 * 10)
            - (self.fail_count as i32 * 5)
            - (self.consecutive_fails as i32 * 20);
    }
}

// ─── Tracker Manager ────────────────────────────────────────

/// Manages tracker state for all torrents.
pub struct TrackerManager {
    /// Per-torrent tracker lists. Key = torrent ID.
    trackers: RwLock<HashMap<String, Vec<TrackerEntry>>>,
}

impl TrackerManager {
    pub fn new() -> Self {
        Self {
            trackers: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize trackers for a torrent.
    pub fn register_torrent(&self, torrent_id: &str, tracker_urls: Vec<(String, u8)>) {
        let entries: Vec<TrackerEntry> = tracker_urls.into_iter()
            .map(|(url, tier)| TrackerEntry::new(url, tier))
            .collect();
        self.trackers.write().insert(torrent_id.to_string(), entries);
    }

    /// Remove tracker state for a torrent.
    pub fn unregister_torrent(&self, torrent_id: &str) {
        self.trackers.write().remove(torrent_id);
    }

    /// Get tracker list for a torrent.
    pub fn get_trackers(&self, torrent_id: &str) -> Vec<TrackerEntry> {
        self.trackers.read()
            .get(torrent_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Add a tracker to a torrent.
    pub fn add_tracker(&self, torrent_id: &str, url: String, tier: u8) {
        let mut trackers = self.trackers.write();
        let list = trackers.entry(torrent_id.to_string()).or_default();

        // Don't add duplicates
        if !list.iter().any(|t| t.url == url) {
            list.push(TrackerEntry::new(url, tier));
        }
    }

    /// Remove a tracker from a torrent.
    pub fn remove_tracker(&self, torrent_id: &str, url: &str) {
        let mut trackers = self.trackers.write();
        if let Some(list) = trackers.get_mut(torrent_id) {
            list.retain(|t| t.url != url);
        }
    }

    /// Record a successful announce.
    pub fn record_success(
        &self,
        torrent_id: &str,
        tracker_url: &str,
        seeders: u32,
        leechers: u32,
        downloaded: u32,
    ) {
        let mut trackers = self.trackers.write();
        if let Some(list) = trackers.get_mut(torrent_id) {
            if let Some(entry) = list.iter_mut().find(|t| t.url == tracker_url) {
                entry.record_success(seeders, leechers, downloaded);
            }
        }
    }

    /// Record a failed announce.
    pub fn record_failure(&self, torrent_id: &str, tracker_url: &str, error: String) {
        let mut trackers = self.trackers.write();
        if let Some(list) = trackers.get_mut(torrent_id) {
            if let Some(entry) = list.iter_mut().find(|t| t.url == tracker_url) {
                entry.record_failure(error);
            }
        }
    }

    /// Re-enable a disabled tracker.
    pub fn enable_tracker(&self, torrent_id: &str, tracker_url: &str) {
        let mut trackers = self.trackers.write();
        if let Some(list) = trackers.get_mut(torrent_id) {
            if let Some(entry) = list.iter_mut().find(|t| t.url == tracker_url) {
                entry.status = TrackerStatus::NotContacted;
                entry.consecutive_fails = 0;
                entry.recalc_health();
            }
        }
    }

    /// Get aggregate seeder/leecher counts for a torrent (from all working trackers).
    pub fn get_swarm_info(&self, torrent_id: &str) -> (u32, u32) {
        let trackers = self.trackers.read();
        let list = match trackers.get(torrent_id) {
            Some(l) => l,
            None => return (0, 0),
        };

        let seeders = list.iter()
            .filter(|t| t.status == TrackerStatus::Working)
            .map(|t| t.seeders)
            .max()
            .unwrap_or(0);

        let leechers = list.iter()
            .filter(|t| t.status == TrackerStatus::Working)
            .map(|t| t.leechers)
            .max()
            .unwrap_or(0);

        (seeders, leechers)
    }
}
