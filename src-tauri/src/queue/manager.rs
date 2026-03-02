// Tsubasa (翼) — Queue Manager
// Active torrent limits with slot allocation algorithm.
// Checks every 2 seconds, promotes queued → active as slots free.

use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::Notify;

use crate::settings::schema::QueueSettings;

// ─── Types ──────────────────────────────────────────────────

/// Queue state for a single torrent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueuePosition {
    /// Actively downloading/seeding (has a slot).
    Active,
    /// Waiting for a slot.
    Queued(u32), // position in queue (1-based)
    /// User forced start — bypasses queue limits.
    ForceStarted,
    /// Paused by user.
    Paused,
}

/// Info tracked per torrent for queue decisions.
#[derive(Debug, Clone)]
pub struct QueuedTorrent {
    pub id: String,
    pub is_downloading: bool,
    pub is_seeding: bool,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub priority: i32,          // higher = dequeued first
    pub added_at: Instant,
    pub queue_position: QueuePosition,
    pub force_started: bool,
}

// ─── Queue Manager ──────────────────────────────────────────

/// Manages active download/upload/total limits.
/// Call `evaluate()` periodically (every 2s) to promote queued torrents.
pub struct QueueManager {
    config: Arc<RwLock<QueueSettings>>,
    torrents: RwLock<Vec<QueuedTorrent>>,
    notify: Notify,
}

impl QueueManager {
    pub fn new(config: Arc<RwLock<QueueSettings>>) -> Self {
        Self {
            config,
            torrents: RwLock::new(Vec::new()),
            notify: Notify::new(),
        }
    }

    /// Register a torrent with the queue manager.
    pub fn register(&self, torrent: QueuedTorrent) {
        self.torrents.write().push(torrent);
        self.notify.notify_one();
    }

    /// Remove a torrent from the queue manager.
    pub fn unregister(&self, id: &str) {
        self.torrents.write().retain(|t| t.id != id);
    }

    /// Update torrent stats (called periodically from the polling loop).
    pub fn update_stats(&self, id: &str, dl_speed: f64, ul_speed: f64, is_downloading: bool, is_seeding: bool) {
        let mut torrents = self.torrents.write();
        if let Some(t) = torrents.iter_mut().find(|t| t.id == id) {
            t.download_speed = dl_speed;
            t.upload_speed = ul_speed;
            t.is_downloading = is_downloading;
            t.is_seeding = is_seeding;
        }
    }

    /// Force start a torrent (bypass queue limits).
    pub fn force_start(&self, id: &str) {
        let mut torrents = self.torrents.write();
        if let Some(t) = torrents.iter_mut().find(|t| t.id == id) {
            t.force_started = true;
            t.queue_position = QueuePosition::ForceStarted;
        }
    }

    /// Set priority for a torrent (higher = dequeued first).
    pub fn set_priority(&self, id: &str, priority: i32) {
        let mut torrents = self.torrents.write();
        if let Some(t) = torrents.iter_mut().find(|t| t.id == id) {
            t.priority = priority;
        }
    }

    /// Evaluate the queue and return actions to take.
    ///
    /// Returns:
    /// - `to_start`: torrent IDs that should be started (promoted from queue)
    /// - `to_queue`: torrent IDs that should be queued (demoted from active)
    pub fn evaluate(&self) -> QueueActions {
        let config = self.config.read().clone();
        let mut torrents = self.torrents.write();
        let mut actions = QueueActions::default();

        // Count active torrents (excluding force-started)
        let (mut active_dl, mut active_ul, mut active_total) = (0u32, 0u32, 0u32);

        for t in torrents.iter() {
            if t.force_started {
                continue; // Force-started don't count against limits
            }
            match t.queue_position {
                QueuePosition::Active => {
                    let is_slow = self.is_slow(t, &config);
                    if config.exclude_slow_from_count && is_slow {
                        continue; // Slow torrents don't count
                    }
                    if t.is_downloading { active_dl += 1; }
                    if t.is_seeding { active_ul += 1; }
                    active_total += 1;
                }
                _ => {}
            }
        }

        // Sort queued torrents by priority (highest first), then by added_at (oldest first)
        let mut queued: Vec<usize> = torrents.iter().enumerate()
            .filter(|(_, t)| matches!(t.queue_position, QueuePosition::Queued(_)))
            .map(|(i, _)| i)
            .collect();

        queued.sort_by(|&a, &b| {
            let ta = &torrents[a];
            let tb = &torrents[b];
            tb.priority.cmp(&ta.priority)
                .then(ta.added_at.cmp(&tb.added_at))
        });

        // Promote queued torrents to active if slots available
        for &idx in &queued {
            let t = &torrents[idx];

            // Check total limit
            if active_total >= config.max_active_total {
                break;
            }

            // Check per-type limits
            if t.is_downloading && active_dl >= config.max_active_downloads {
                continue;
            }
            if t.is_seeding && active_ul >= config.max_active_uploads {
                continue;
            }

            // Promote
            actions.to_start.push(torrents[idx].id.clone());
            if torrents[idx].is_downloading { active_dl += 1; }
            if torrents[idx].is_seeding { active_ul += 1; }
            active_total += 1;
        }

        // Update queue positions
        for id in &actions.to_start {
            if let Some(t) = torrents.iter_mut().find(|t| t.id == *id) {
                t.queue_position = QueuePosition::Active;
            }
        }

        // Renumber remaining queued torrents
        let mut pos = 1u32;
        for t in torrents.iter_mut() {
            if matches!(t.queue_position, QueuePosition::Queued(_)) {
                t.queue_position = QueuePosition::Queued(pos);
                pos += 1;
            }
        }

        actions
    }

    /// Check if a torrent is "slow" per the configured thresholds.
    fn is_slow(&self, torrent: &QueuedTorrent, config: &QueueSettings) -> bool {
        if torrent.is_downloading {
            return torrent.download_speed < config.slow_torrent_dl_threshold as f64;
        }
        if torrent.is_seeding {
            return torrent.upload_speed < config.slow_torrent_ul_threshold as f64;
        }
        false
    }

    /// Get the queue position for a torrent.
    pub fn get_position(&self, id: &str) -> Option<QueuePosition> {
        self.torrents.read().iter()
            .find(|t| t.id == id)
            .map(|t| t.queue_position)
    }

    /// Get all queue info (for frontend display).
    pub fn get_all_positions(&self) -> Vec<(String, QueuePosition)> {
        self.torrents.read().iter()
            .map(|t| (t.id.clone(), t.queue_position))
            .collect()
    }

    /// Update configuration live.
    pub fn update_config(&self, new_config: QueueSettings) {
        *self.config.write() = new_config;
        self.notify.notify_one();
    }
}

/// Actions the queue manager wants the engine to take.
#[derive(Debug, Default)]
pub struct QueueActions {
    /// Torrent IDs to start (promote from queue).
    pub to_start: Vec<String>,
    /// Torrent IDs to queue (demote from active).
    pub to_queue: Vec<String>,
}
