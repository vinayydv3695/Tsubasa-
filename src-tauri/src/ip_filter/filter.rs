// Tsubasa (翼) — IP Filter
// Sorted range-based IP filter with O(log n) lookup.
// Supports 1M+ ranges from PeerGuardian/eMule blocklists.

use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;
use std::sync::Arc;

use parking_lot::RwLock;

use super::blocklist::{self, IpRange};
use crate::settings::schema::IPFilterSettings;

/// Thread-safe IP filter with sorted ranges for fast lookup.
pub struct IPFilter {
    /// Sorted, non-overlapping IP ranges (merged on load).
    ranges: Vec<(u32, u32)>,
    /// Manually banned individual IPs.
    manual_bans: HashSet<u32>,
    /// Number of ranges loaded.
    range_count: usize,
}

impl IPFilter {
    pub fn new() -> Self {
        Self {
            ranges: Vec::new(),
            manual_bans: HashSet::new(),
            range_count: 0,
        }
    }

    /// Check if an IP address is blocked.
    /// O(log n) via binary search on sorted ranges + O(1) for manual bans.
    pub fn is_blocked(&self, ip: IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                let ip_u32 = u32::from(ipv4);

                // Check manual bans first (O(1))
                if self.manual_bans.contains(&ip_u32) {
                    return true;
                }

                // Binary search on sorted ranges (O(log n))
                self.ranges.binary_search_by(|&(start, end)| {
                    if ip_u32 < start {
                        std::cmp::Ordering::Greater
                    } else if ip_u32 > end {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }).is_ok()
            }
            IpAddr::V6(_) => {
                // IPv6 filtering not implemented yet (very rare in torrents)
                false
            }
        }
    }

    /// Load ranges from a PeerGuardian .dat file.
    pub fn load_dat_file(&mut self, path: &Path) -> Result<usize, String> {
        let ranges = blocklist::parse_dat_file(path)?;
        let count = ranges.len();
        self.add_ranges(ranges);
        Ok(count)
    }

    /// Load ranges from an eMule .p2p file.
    pub fn load_p2p_file(&mut self, path: &Path) -> Result<usize, String> {
        let ranges = blocklist::parse_p2p_file(path)?;
        let count = ranges.len();
        self.add_ranges(ranges);
        Ok(count)
    }

    /// Add a CIDR range.
    pub fn add_cidr(&mut self, cidr: &str) -> Result<(), String> {
        let range = blocklist::parse_cidr(cidr)?;
        self.add_ranges(vec![range]);
        Ok(())
    }

    /// Manually ban an IP address.
    pub fn ban_ip(&mut self, ip: Ipv4Addr) {
        self.manual_bans.insert(u32::from(ip));
    }

    /// Remove a manual IP ban.
    pub fn unban_ip(&mut self, ip: Ipv4Addr) {
        self.manual_bans.remove(&u32::from(ip));
    }

    /// Get the number of loaded ranges.
    pub fn range_count(&self) -> usize {
        self.range_count
    }

    /// Get the number of manual bans.
    pub fn manual_ban_count(&self) -> usize {
        self.manual_bans.len()
    }

    /// Clear all ranges and bans.
    pub fn clear(&mut self) {
        self.ranges.clear();
        self.manual_bans.clear();
        self.range_count = 0;
    }

    /// Add ranges, merge overlaps, and re-sort.
    fn add_ranges(&mut self, new_ranges: Vec<IpRange>) {
        for r in new_ranges {
            self.ranges.push((r.start, r.end));
        }

        // Sort by start IP
        self.ranges.sort_by_key(|&(start, _)| start);

        // Merge overlapping ranges
        let mut merged: Vec<(u32, u32)> = Vec::new();
        for &(start, end) in &self.ranges {
            if let Some(last) = merged.last_mut() {
                if start <= last.1 + 1 {
                    // Overlapping or adjacent — extend
                    last.1 = last.1.max(end);
                    continue;
                }
            }
            merged.push((start, end));
        }

        self.range_count = merged.len();
        self.ranges = merged;
    }
}

/// Thread-safe wrapper around IPFilter.
pub struct IPFilterManager {
    filter: Arc<RwLock<IPFilter>>,
    config: Arc<RwLock<IPFilterSettings>>,
}

impl IPFilterManager {
    pub fn new(config: Arc<RwLock<IPFilterSettings>>) -> Self {
        Self {
            filter: Arc::new(RwLock::new(IPFilter::new())),
            config,
        }
    }

    /// Check if IP filtering is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.read().enabled
    }

    /// Check if an IP is blocked (returns false if filtering is disabled).
    pub fn is_blocked(&self, ip: IpAddr) -> bool {
        if !self.is_enabled() {
            return false;
        }
        self.filter.read().is_blocked(ip)
    }

    /// Load the configured blocklist file.
    pub fn load_blocklist(&self) -> Result<usize, String> {
        let config = self.config.read().clone();
        let path = config.blocklist_path
            .ok_or("No blocklist path configured")?;

        let path = Path::new(&path);
        if !path.exists() {
            return Err(format!("Blocklist file not found: {}", path.display()));
        }

        let mut filter = self.filter.write();
        filter.clear();

        // Detect format by extension
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "dat" => filter.load_dat_file(path),
            "p2p" => filter.load_p2p_file(path),
            _ => {
                // Try both formats
                filter.load_dat_file(path)
                    .or_else(|_| filter.load_p2p_file(path))
            }
        }
    }

    /// Manually ban an IP.
    pub fn ban_ip(&self, ip: Ipv4Addr) {
        self.filter.write().ban_ip(ip);
    }

    /// Remove a manual IP ban.
    pub fn unban_ip(&self, ip: Ipv4Addr) {
        self.filter.write().unban_ip(ip);
    }

    /// Get filter statistics.
    pub fn stats(&self) -> (usize, usize) {
        let filter = self.filter.read();
        (filter.range_count(), filter.manual_ban_count())
    }

    /// Update configuration.
    pub fn update_config(&self, new: IPFilterSettings) {
        *self.config.write() = new;
    }
}
