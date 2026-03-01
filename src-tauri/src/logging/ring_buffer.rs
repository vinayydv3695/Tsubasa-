// Tsubasa (翼) — Log Ring Buffer
// In-memory circular buffer for the in-app log viewer.

use parking_lot::Mutex;
use serde::Serialize;
use std::collections::VecDeque;

const DEFAULT_CAPACITY: usize = 5000;

/// A single log entry for display in the UI.
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

/// Thread-safe ring buffer that holds the most recent N log entries.
pub struct LogRingBuffer {
    entries: Mutex<VecDeque<LogEntry>>,
    capacity: usize,
}

impl LogRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    /// Push a new log entry. Evicts oldest if at capacity.
    pub fn push(&self, entry: LogEntry) {
        let mut entries = self.entries.lock();
        if entries.len() >= self.capacity {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    /// Get all entries (for initial load in UI).
    pub fn get_all(&self) -> Vec<LogEntry> {
        self.entries.lock().iter().cloned().collect()
    }

    /// Get entries since a given index (for incremental updates).
    pub fn get_since(&self, index: usize) -> Vec<LogEntry> {
        let entries = self.entries.lock();
        if index >= entries.len() {
            return Vec::new();
        }
        entries.iter().skip(index).cloned().collect()
    }

    /// Get the current count of entries.
    pub fn len(&self) -> usize {
        self.entries.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.lock().is_empty()
    }

    /// Clear all entries.
    pub fn clear(&self) {
        self.entries.lock().clear();
    }
}

impl Default for LogRingBuffer {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}
