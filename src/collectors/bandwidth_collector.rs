use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sysinfo::Networks;

/// Represents bandwidth statistics for a single network interface at a specific point in time
/// This struct contains both cumulative totals and calculated speeds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthStats {
    /// UTC timestamp when these statistics were collected
    pub timestamp: DateTime<Utc>,
    /// Name of the network interface (e.g., "eth0", "wlan0", "en0")
    pub interface_name: String,
    /// Total bytes received since interface startup
    pub bytes_received: u64,
    /// Total bytes sent since interface startup
    pub bytes_sent: u64,
    /// Total packets received since interface startup
    pub packets_received: u64,
    /// Total packets sent since interface startup
    pub packets_sent: u64,
    /// Current download speed in bytes per second
    pub download_speed_bps: f64,
    /// Current upload speed in bytes per second
    pub upload_speed_bps: f64,
}

/// Collects bandwidth statistics from network interfaces
/// Maintains previous readings to calculate speed deltas
#[derive(Debug)]
pub struct BandwidthCollector {
    /// System network interfaces manager from sysinfo crate
    networks: Networks,
    /// Cache of previous readings for speed calculation
    /// Maps interface name to (bytes_received, bytes_sent, timestamp)
    previous_stats: HashMap<String, (u64, u64, DateTime<Utc>)>,
}

impl BandwidthCollector {
    /// Creates a new bandwidth collector with fresh network interface list
    pub fn new() -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
            previous_stats: HashMap::new(),
        }
    }

    /// Collects current bandwidth statistics from all network interfaces
    /// Returns a vector of BandwidthStats, one per active interface
    pub fn collect(&mut self) -> Result<Vec<BandwidthStats>> {
        // Refresh network statistics from the system
        self.networks.refresh(false);
        let now = Utc::now();
        let mut stats = Vec::new();

        // Process each network interface
        for (interface_name, network) in &self.networks {
            let bytes_received = network.received();
            let bytes_sent = network.transmitted();
            let packets_received = network.packets_received();
            let packets_sent = network.packets_transmitted();

            // Calculate speeds based on previous readings
            let (download_speed_bps, upload_speed_bps) = if let Some((prev_rx, prev_tx, prev_time)) = 
                self.previous_stats.get(interface_name) {
                let time_diff = (now - prev_time).num_milliseconds() as f64 / 1000.0;
                if time_diff > 0.0 {
                    // Calculate bytes per second using saturating subtraction to prevent underflow
                    let download_speed = bytes_received.saturating_sub(*prev_rx) as f64 / time_diff;
                    let upload_speed = bytes_sent.saturating_sub(*prev_tx) as f64 / time_diff;
                    (download_speed, upload_speed)
                } else {
                    (0.0, 0.0)
                }
            } else {
                // First reading for this interface - no speed calculation possible
                (0.0, 0.0)
            };

            // Store current readings for next speed calculation
            self.previous_stats.insert(
                interface_name.to_string(),
                (bytes_received, bytes_sent, now),
            );

            stats.push(BandwidthStats {
                timestamp: now,
                interface_name: interface_name.to_string(),
                bytes_received,
                bytes_sent,
                packets_received,
                packets_sent,
                download_speed_bps,
                upload_speed_bps,
            });
        }

        Ok(stats)
    }

    /// Returns total bandwidth usage across all interfaces
    /// Returns tuple of (total_download_bytes, total_upload_bytes)
    pub fn get_total_bandwidth(&self) -> (f64, f64) {
        let stats: Vec<_> = self.previous_stats.values()
            .map(|(rx, tx, _)| (*rx, *tx))
            .collect();
        
        let total_download = stats.iter().map(|(rx, _)| rx).sum::<u64>() as f64;
        let total_upload = stats.iter().map(|(_, tx)| tx).sum::<u64>() as f64;
        
        (total_download, total_upload)
    }
}

/// Formats byte values into human-readable format
/// Converts bytes to appropriate unit (B, KB, MB, GB, TB)
pub fn format_bytes(bytes: f64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes;
    let mut unit_index = 0;

    // Convert to higher units while value is >= 1024
    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", value, UNITS[unit_index])
}

/// Formats speed values into human-readable format
/// Converts bytes per second to appropriate unit (B/s, KB/s, MB/s, GB/s)
pub fn format_speed(bytes_per_second: f64) -> String {
    const UNITS: &[&str] = &["B/s", "KB/s", "MB/s", "GB/s"];
    let mut value = bytes_per_second;
    let mut unit_index = 0;

    // Convert to higher units while value is >= 1024
    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", value, UNITS[unit_index])
}