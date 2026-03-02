// Tsubasa (翼) — Seeding Rules Engine
// Checks ratio, time, and inactivity limits for seeding torrents.
// Returns actions to take when limits are hit.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::settings::schema::{SeedingAction, SeedingSettings, TorrentOverrides};

// ─── Types ──────────────────────────────────────────────────

/// Information about a seeding torrent needed for rule evaluation.
#[derive(Debug, Clone)]
pub struct SeedingTorrentInfo {
    pub id: String,
    pub name: String,
    /// Current share ratio (uploaded / downloaded).
    pub ratio: f64,
    /// Time spent seeding since completion.
    pub seed_time: Duration,
    /// Time since last upload activity.
    pub inactive_time: Duration,
    /// Current upload speed (bytes/sec).
    pub upload_speed: f64,
    /// Per-torrent overrides (if any).
    pub overrides: Option<TorrentOverrides>,
}

/// Action to take on a torrent that hit a seeding limit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedingRuleAction {
    pub torrent_id: String,
    pub torrent_name: String,
    pub action: SeedingAction,
    pub reason: SeedingLimitReason,
}

/// Why the seeding limit was triggered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeedingLimitReason {
    RatioReached { current: f64, limit: f64 },
    TimeLimitReached { minutes_seeded: u64, limit_minutes: u64 },
    InactiveTimeout { inactive_minutes: u64, limit_minutes: u64 },
}

impl std::fmt::Display for SeedingLimitReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RatioReached { current, limit } =>
                write!(f, "Ratio {:.2} reached limit {:.2}", current, limit),
            Self::TimeLimitReached { minutes_seeded, limit_minutes } =>
                write!(f, "Seeded {}m, limit {}m", minutes_seeded, limit_minutes),
            Self::InactiveTimeout { inactive_minutes, limit_minutes } =>
                write!(f, "Inactive {}m, timeout {}m", inactive_minutes, limit_minutes),
        }
    }
}

// ─── Rules Engine ───────────────────────────────────────────

/// Evaluate seeding rules for a list of seeding torrents.
/// Returns actions for any torrents that have hit their limits.
pub fn evaluate_seeding_rules(
    torrents: &[SeedingTorrentInfo],
    global_settings: &SeedingSettings,
) -> Vec<SeedingRuleAction> {
    let mut actions = Vec::new();

    for torrent in torrents {
        // Per-torrent overrides take priority over global settings
        let ratio_limit = torrent.overrides.as_ref()
            .and_then(|o| o.ratio_limit)
            .or(global_settings.global_ratio_limit);

        let time_limit_mins = torrent.overrides.as_ref()
            .and_then(|o| o.time_limit_mins)
            .or(global_settings.global_time_limit_mins);

        let inactive_timeout_mins = global_settings.inactive_timeout_mins;

        // Check ratio limit
        if let Some(limit) = ratio_limit {
            if torrent.ratio >= limit {
                actions.push(SeedingRuleAction {
                    torrent_id: torrent.id.clone(),
                    torrent_name: torrent.name.clone(),
                    action: global_settings.action_on_limit,
                    reason: SeedingLimitReason::RatioReached {
                        current: torrent.ratio,
                        limit,
                    },
                });
                continue; // Only one action per torrent
            }
        }

        // Check time limit
        if let Some(limit) = time_limit_mins {
            let seeded_mins = torrent.seed_time.as_secs() / 60;
            if seeded_mins >= limit {
                actions.push(SeedingRuleAction {
                    torrent_id: torrent.id.clone(),
                    torrent_name: torrent.name.clone(),
                    action: global_settings.action_on_limit,
                    reason: SeedingLimitReason::TimeLimitReached {
                        minutes_seeded: seeded_mins,
                        limit_minutes: limit,
                    },
                });
                continue;
            }
        }

        // Check inactivity timeout
        if let Some(timeout) = inactive_timeout_mins {
            let inactive_mins = torrent.inactive_time.as_secs() / 60;
            if inactive_mins >= timeout && torrent.upload_speed == 0.0 {
                actions.push(SeedingRuleAction {
                    torrent_id: torrent.id.clone(),
                    torrent_name: torrent.name.clone(),
                    action: global_settings.action_on_limit,
                    reason: SeedingLimitReason::InactiveTimeout {
                        inactive_minutes: inactive_mins,
                        limit_minutes: timeout,
                    },
                });
            }
        }
    }

    actions
}
