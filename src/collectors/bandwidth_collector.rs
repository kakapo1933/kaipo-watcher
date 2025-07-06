use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sysinfo::Networks;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthStats {
    pub timestamp: DateTime<Utc>,
    pub interface_name: String,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub packets_received: u64,
    pub packets_sent: u64,
    pub download_speed_bps: f64,
    pub upload_speed_bps: f64,
}

#[derive(Debug)]
pub struct BandwidthCollector {
    networks: Networks,
    previous_stats: HashMap<String, (u64, u64, DateTime<Utc>)>,
}

impl BandwidthCollector {
    pub fn new() -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
            previous_stats: HashMap::new(),
        }
    }

    pub fn collect(&mut self) -> Result<Vec<BandwidthStats>> {
        self.networks.refresh(false);
        let now = Utc::now();
        let mut stats = Vec::new();

        for (interface_name, network) in &self.networks {
            let bytes_received = network.received();
            let bytes_sent = network.transmitted();
            let packets_received = network.packets_received();
            let packets_sent = network.packets_transmitted();

            let (download_speed_bps, upload_speed_bps) = if let Some((prev_rx, prev_tx, prev_time)) = 
                self.previous_stats.get(interface_name) {
                let time_diff = (now - prev_time).num_milliseconds() as f64 / 1000.0;
                if time_diff > 0.0 {
                    let download_speed = bytes_received.saturating_sub(*prev_rx) as f64 / time_diff;
                    let upload_speed = bytes_sent.saturating_sub(*prev_tx) as f64 / time_diff;
                    (download_speed, upload_speed)
                } else {
                    (0.0, 0.0)
                }
            } else {
                (0.0, 0.0)
            };

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

    pub fn get_total_bandwidth(&self) -> (f64, f64) {
        let stats: Vec<_> = self.previous_stats.values()
            .map(|(rx, tx, _)| (*rx, *tx))
            .collect();
        
        let total_download = stats.iter().map(|(rx, _)| rx).sum::<u64>() as f64;
        let total_upload = stats.iter().map(|(_, tx)| tx).sum::<u64>() as f64;
        
        (total_download, total_upload)
    }
}

pub fn format_bytes(bytes: f64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes;
    let mut unit_index = 0;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", value, UNITS[unit_index])
}

pub fn format_speed(bytes_per_second: f64) -> String {
    const UNITS: &[&str] = &["B/s", "KB/s", "MB/s", "GB/s"];
    let mut value = bytes_per_second;
    let mut unit_index = 0;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", value, UNITS[unit_index])
}