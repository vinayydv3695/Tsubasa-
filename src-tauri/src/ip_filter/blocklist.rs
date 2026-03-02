// Tsubasa (翼) — Blocklist Parser
// Parses PeerGuardian .dat and eMule .p2p IP range files.
// Formats:
//   PeerGuardian .dat:  "Description:start_ip-end_ip"
//   eMule .p2p:         "start_ip - end_ip , level , description"

use std::net::Ipv4Addr;
use std::path::Path;

/// An IP range entry from a blocklist file.
#[derive(Debug, Clone)]
pub struct IpRange {
    pub start: u32,
    pub end: u32,
}

impl IpRange {
    pub fn new(start: Ipv4Addr, end: Ipv4Addr) -> Self {
        Self {
            start: u32::from(start),
            end: u32::from(end),
        }
    }
}

/// Parse a PeerGuardian .dat format blocklist file.
/// Format: "Description:start_ip-end_ip"
pub fn parse_dat_file(path: &Path) -> Result<Vec<IpRange>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read .dat file: {e}"))?;

    let mut ranges = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Format: "Description:start_ip-end_ip"
        if let Some(colon_pos) = line.rfind(':') {
            let ip_part = &line[colon_pos + 1..];
            if let Some(dash_pos) = ip_part.find('-') {
                let start_str = ip_part[..dash_pos].trim();
                let end_str = ip_part[dash_pos + 1..].trim();

                match (start_str.parse::<Ipv4Addr>(), end_str.parse::<Ipv4Addr>()) {
                    (Ok(start), Ok(end)) => {
                        ranges.push(IpRange::new(start, end));
                    }
                    _ => {
                        tracing::trace!(line = line_num, "Skipping invalid IP range in .dat");
                    }
                }
            }
        }
    }

    tracing::info!(count = ranges.len(), path = %path.display(), "Parsed PeerGuardian .dat blocklist");
    Ok(ranges)
}

/// Parse an eMule .p2p format blocklist file.
/// Format: "start_ip - end_ip , level , description"
pub fn parse_p2p_file(path: &Path) -> Result<Vec<IpRange>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read .p2p file: {e}"))?;

    let mut ranges = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Format: "start_ip - end_ip , level , description"
        let parts: Vec<&str> = line.split(',').collect();
        if parts.is_empty() { continue; }

        let ip_range = parts[0].trim();
        if let Some(dash_pos) = ip_range.find(" - ") {
            let start_str = ip_range[..dash_pos].trim();
            let end_str = ip_range[dash_pos + 3..].trim();

            match (start_str.parse::<Ipv4Addr>(), end_str.parse::<Ipv4Addr>()) {
                (Ok(start), Ok(end)) => {
                    ranges.push(IpRange::new(start, end));
                }
                _ => {
                    tracing::trace!(line = line_num, "Skipping invalid IP range in .p2p");
                }
            }
        }
    }

    tracing::info!(count = ranges.len(), path = %path.display(), "Parsed eMule .p2p blocklist");
    Ok(ranges)
}

/// Parse a CIDR notation string (e.g., "192.168.1.0/24") into an IpRange.
pub fn parse_cidr(cidr: &str) -> Result<IpRange, String> {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid CIDR: {}", cidr));
    }

    let base: Ipv4Addr = parts[0].parse()
        .map_err(|e| format!("Invalid IP in CIDR: {e}"))?;
    let prefix_len: u32 = parts[1].parse()
        .map_err(|e| format!("Invalid prefix length: {e}"))?;

    if prefix_len > 32 {
        return Err(format!("Prefix length {} > 32", prefix_len));
    }

    let base_u32 = u32::from(base);
    let mask = if prefix_len == 0 { 0 } else { !0u32 << (32 - prefix_len) };
    let start = base_u32 & mask;
    let end = start | !mask;

    Ok(IpRange { start, end })
}
