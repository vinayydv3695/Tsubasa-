// Tsubasa (翼) — Speed Graph Collector
// Circular buffer for bandwidth sampling.
// Records 1-second resolution samples, keeps 5 minutes of history.
// Low overhead — just pushes to a VecDeque and pops old entries.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use serde::Serialize;

/// A single speed sample at a point in time.
#[derive(Debug, Clone, Serialize)]
pub struct SpeedSample {
    /// Seconds since app start (for frontend graphing).
    pub timestamp_secs: f64,
    /// Download speed in bytes/second.
    pub download_speed: f64,
    /// Upload speed in bytes/second.
    pub upload_speed: f64,
}

/// Circular buffer speed graph collector.
/// Stores up to `max_samples` at 1-second resolution.
pub struct SpeedGraphCollector {
    buffer: VecDeque<SpeedSample>,
    max_samples: usize,
    start_time: Instant,
}

impl SpeedGraphCollector {
    /// Create a new collector.
    /// `max_samples` = 300 gives 5 minutes of 1-second history.
    pub fn new(max_samples: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_samples),
            max_samples,
            start_time: Instant::now(),
        }
    }

    /// Record a speed sample. Called every second by the sampling task.
    pub fn record(&mut self, download_speed: f64, upload_speed: f64) {
        let timestamp_secs = self.start_time.elapsed().as_secs_f64();

        let sample = SpeedSample {
            timestamp_secs,
            download_speed,
            upload_speed,
        };

        if self.buffer.len() >= self.max_samples {
            self.buffer.pop_front();
        }
        self.buffer.push_back(sample);
    }

    /// Get all samples within the given time window (from now).
    pub fn get_window(&self, window: Duration) -> Vec<SpeedSample> {
        let now = self.start_time.elapsed().as_secs_f64();
        let cutoff = now - window.as_secs_f64();

        self.buffer.iter()
            .filter(|s| s.timestamp_secs >= cutoff)
            .cloned()
            .collect()
    }

    /// Get the last N samples.
    pub fn get_last_n(&self, n: usize) -> Vec<SpeedSample> {
        let skip = if self.buffer.len() > n { self.buffer.len() - n } else { 0 };
        self.buffer.iter().skip(skip).cloned().collect()
    }

    /// Get average speeds over a time window.
    pub fn average(&self, window: Duration) -> (f64, f64) {
        let samples = self.get_window(window);
        if samples.is_empty() {
            return (0.0, 0.0);
        }
        let count = samples.len() as f64;
        let dl_avg = samples.iter().map(|s| s.download_speed).sum::<f64>() / count;
        let ul_avg = samples.iter().map(|s| s.upload_speed).sum::<f64>() / count;
        (dl_avg, ul_avg)
    }

    /// Get peak speeds over a time window.
    pub fn peak(&self, window: Duration) -> (f64, f64) {
        let samples = self.get_window(window);
        let dl_peak = samples.iter().map(|s| s.download_speed).fold(0.0f64, f64::max);
        let ul_peak = samples.iter().map(|s| s.upload_speed).fold(0.0f64, f64::max);
        (dl_peak, ul_peak)
    }

    /// Total number of samples stored.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

/// Per-torrent speed graph collection.
/// Maps torrent IDs to individual collectors.
pub struct PerTorrentGraphs {
    graphs: std::collections::HashMap<String, SpeedGraphCollector>,
    max_samples: usize,
}

impl PerTorrentGraphs {
    pub fn new(max_samples: usize) -> Self {
        Self {
            graphs: std::collections::HashMap::new(),
            max_samples,
        }
    }

    /// Record speed for a specific torrent.
    pub fn record(&mut self, id: &str, dl: f64, ul: f64) {
        let collector = self.graphs
            .entry(id.to_string())
            .or_insert_with(|| SpeedGraphCollector::new(self.max_samples));
        collector.record(dl, ul);
    }

    /// Remove a torrent's graph data.
    pub fn remove(&mut self, id: &str) {
        self.graphs.remove(id);
    }

    /// Get samples for a specific torrent.
    pub fn get_window(&self, id: &str, window: Duration) -> Vec<SpeedSample> {
        self.graphs.get(id)
            .map(|c| c.get_window(window))
            .unwrap_or_default()
    }
}
