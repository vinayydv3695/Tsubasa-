// Tsubasa (翼) — Torrent State Machine
// Strict state transitions — no freeform state assignment allowed.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::TsubasaError;

/// All possible states a torrent can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TorrentState {
    /// Just added, parsing metadata / fetching info from magnet
    Pending,
    /// Checking existing files on disk
    Checking,
    /// Queued, waiting for a download slot
    Queued,
    /// Actively downloading (local, cloud, or hybrid)
    Downloading,
    /// User or system paused the download
    Paused,
    /// All pieces downloaded and verified
    Completed,
    /// Uploading to peers after completion
    Seeding,
    /// Stopped by user or ratio limit met
    Stopped,
    /// Recoverable error state
    Errored,
}

impl TorrentState {
    /// Check if a transition from `self` to `target` is valid.
    pub fn can_transition_to(&self, target: TorrentState) -> bool {
        use TorrentState::*;
        matches!(
            (self, target),
            // From Pending
            (Pending, Checking)
                | (Pending, Downloading)
                | (Pending, Queued)
                | (Pending, Errored)
                | (Pending, Paused)
                // From Checking
                | (Checking, Downloading)
                | (Checking, Completed) // already have all pieces
                | (Checking, Queued)
                | (Checking, Errored)
                | (Checking, Paused)
                // From Queued
                | (Queued, Downloading)
                | (Queued, Paused)
                | (Queued, Errored)
                // From Downloading
                | (Downloading, Paused)
                | (Downloading, Completed)
                | (Downloading, Errored)
                | (Downloading, Stopped)
                // From Paused
                | (Paused, Downloading)
                | (Paused, Queued)
                | (Paused, Stopped)
                | (Paused, Checking)
                // From Completed
                | (Completed, Seeding)
                | (Completed, Stopped)
                // From Seeding
                | (Seeding, Stopped)
                | (Seeding, Paused)
                | (Seeding, Errored)
                // From Errored (retry paths)
                | (Errored, Pending)
                | (Errored, Downloading)
                | (Errored, Queued)
                | (Errored, Stopped)
                | (Errored, Paused)
                // From Stopped (restart paths)
                | (Stopped, Downloading)
                | (Stopped, Queued)
                | (Stopped, Seeding)
                | (Stopped, Checking)
        )
    }

    /// Attempt a state transition. Returns the new state or an error.
    pub fn transition_to(self, target: TorrentState) -> crate::error::Result<TorrentState> {
        if self == target {
            return Ok(self); // No-op transitions are fine
        }
        if self.can_transition_to(target) {
            Ok(target)
        } else {
            Err(TsubasaError::InvalidTransition {
                from: self.to_string(),
                to: target.to_string(),
            })
        }
    }

    /// Is this state considered "active" (consuming resources)?
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            TorrentState::Downloading
                | TorrentState::Checking
                | TorrentState::Seeding
                | TorrentState::Pending
        )
    }

    /// Is this state terminal (no further progress expected)?
    pub fn is_terminal(&self) -> bool {
        matches!(self, TorrentState::Stopped | TorrentState::Errored)
    }
}

impl fmt::Display for TorrentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TorrentState::Pending => "pending",
            TorrentState::Checking => "checking",
            TorrentState::Queued => "queued",
            TorrentState::Downloading => "downloading",
            TorrentState::Paused => "paused",
            TorrentState::Completed => "completed",
            TorrentState::Seeding => "seeding",
            TorrentState::Stopped => "stopped",
            TorrentState::Errored => "errored",
        };
        write!(f, "{}", s)
    }
}

/// Download policy — determines which path a torrent uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadPolicy {
    LocalOnly,
    CloudOnly,
    Hybrid,
}

impl Default for DownloadPolicy {
    fn default() -> Self {
        DownloadPolicy::LocalOnly
    }
}

impl fmt::Display for DownloadPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DownloadPolicy::LocalOnly => write!(f, "local_only"),
            DownloadPolicy::CloudOnly => write!(f, "cloud_only"),
            DownloadPolicy::Hybrid => write!(f, "hybrid"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transitions() {
        let state = TorrentState::Pending;
        assert!(state.can_transition_to(TorrentState::Downloading));
        assert!(state.can_transition_to(TorrentState::Checking));
        assert!(state.can_transition_to(TorrentState::Errored));
    }

    #[test]
    fn invalid_transitions() {
        let state = TorrentState::Pending;
        assert!(!state.can_transition_to(TorrentState::Seeding));
        assert!(!state.can_transition_to(TorrentState::Completed));
    }

    #[test]
    fn noop_transition() {
        let state = TorrentState::Downloading;
        assert_eq!(
            state.transition_to(TorrentState::Downloading).unwrap(),
            TorrentState::Downloading
        );
    }

    #[test]
    fn full_lifecycle() {
        let mut state = TorrentState::Pending;
        state = state.transition_to(TorrentState::Downloading).unwrap();
        state = state.transition_to(TorrentState::Completed).unwrap();
        state = state.transition_to(TorrentState::Seeding).unwrap();
        state = state.transition_to(TorrentState::Stopped).unwrap();
        assert_eq!(state, TorrentState::Stopped);
    }
}
