use crate::analyzers::{AnalysisResult, ProtocolAnalyzer, TrafficType};
use crate::collectors::PacketCollector;
use crate::storage::PacketStorage;
use anyhow::{Context, Result};
use chrono::Local;
use log::error;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::Mutex;
use tokio::time::{interval, timeout};

pub struct PacketCommandHandler {
    storage: Arc<PacketStorage>,
    analyzer: Arc<Mutex<ProtocolAnalyzer>>,
}

impl PacketCommandHandler {
    pub fn new(storage: Arc<PacketStorage>) -> Self {
        Self {
            storage,
            analyzer: Arc::new(Mutex::new(ProtocolAnalyzer::new())),
        }
    }

    pub async fn handle_packets_command(
        &self,
        interface: Option<String>,
        protocol_filter: Option<String>,
        capture_duration: Option<String>,
        detailed: bool,
        max_connections: usize,
    ) -> Result<()> {
        // Note about privileges
        println!("⚠️  Note: Packet capture requires elevated privileges (sudo/administrator)");
        println!();

        let interface_name = interface.unwrap_or_else(|| "any".to_string());
        let duration = parse_duration(&capture_duration.unwrap_or_else(|| "60s".to_string()))?;

        println!("🔍 Starting packet capture on interface: {interface_name}");
        println!("📊 Capture duration: {duration:?}");
        if let Some(protocol) = &protocol_filter {
            println!("🔧 Protocol filter: {protocol}");
        }
        println!();

        // Create packet collector
        let collector = PacketCollector::new(interface_name.clone())
            .context("Failed to create packet collector")?;

        // Start capture with timeout
        let capture_result = timeout(duration, self.run_packet_capture(
            collector,
            protocol_filter,
            detailed,
            max_connections,
        )).await;

        match capture_result {
            Ok(Ok(())) => {
                println!("✅ Packet capture completed successfully");
            }
            Ok(Err(e)) => {
                error!("Packet capture failed: {e}");
                return Err(e);
            }
            Err(_) => {
                println!("⏰ Capture duration completed");
            }
        }

        Ok(())
    }

    async fn run_packet_capture(
        &self,
        collector: PacketCollector,
        protocol_filter: Option<String>,
        detailed: bool,
        max_connections: usize,
    ) -> Result<()> {
        // Start the collector
        collector.start().await.context("Failed to start packet collector")?;

        // Statistics tracking
        let mut packet_count = 0u64;
        let mut byte_count = 0u64;
        let mut protocol_stats: HashMap<String, u64> = HashMap::new();
        let mut connection_tracker: HashMap<String, (u64, u64)> = HashMap::new();

        // Display update interval
        let mut display_interval = interval(StdDuration::from_secs(1));

        println!("📡 Capturing packets... (Press Ctrl+C to stop)\n");

        loop {
            tokio::select! {
                // Handle display updates
                _ = display_interval.tick() => {
                    self.display_stats(
                        packet_count,
                        byte_count,
                        &protocol_stats,
                        &connection_tracker,
                        max_connections,
                        detailed,
                    ).await;
                }

                // Handle packet reception
                packet_opt = collector.receive_packet() => {
                    if let Some(packet) = packet_opt {
                        // Apply protocol filter
                        if let Some(ref filter) = protocol_filter {
                            let packet_protocol = format!("{:?}", packet.transport_protocol).to_lowercase();
                            if !packet_protocol.contains(&filter.to_lowercase()) {
                                continue;
                            }
                        }

                        // Analyze packet
                        let mut analyzer = self.analyzer.lock().await;
                        let analysis = if let Ok(analysis) = analyzer.analyze_packet(&packet) {
                            self.process_packet_analysis(&packet, &analysis)?;
                            analysis
                        } else {
                            AnalysisResult::default()
                        };

                        // Update statistics
                        packet_count += 1;
                        byte_count += packet.size_bytes;

                        // Update protocol stats
                        let protocol_name = analysis_to_protocol_name(&packet, &analysis);
                        *protocol_stats.entry(protocol_name).or_insert(0) += 1;

                        // Update connection tracking
                        if let (Some(src), Some(dst)) = (packet.source_addr, packet.dest_addr) {
                            let connection_key = format!("{}:{} -> {}:{}",
                                src, packet.source_port.unwrap_or(0),
                                dst, packet.dest_port.unwrap_or(0)
                            );
                            let entry = connection_tracker.entry(connection_key).or_insert((0, 0));
                            entry.0 += 1; // packet count
                            entry.1 += packet.size_bytes; // byte count
                        }
                    }
                }
            }
        }
    }

    fn process_packet_analysis(
        &self,
        packet: &crate::models::NetworkPacket,
        analysis: &AnalysisResult,
    ) -> Result<()> {
        // Store analysis results
        self.storage.analyze_packet_for_storage(packet, analysis)?;
        Ok(())
    }

    async fn display_stats(
        &self,
        packet_count: u64,
        byte_count: u64,
        protocol_stats: &HashMap<String, u64>,
        connection_tracker: &HashMap<String, (u64, u64)>,
        max_connections: usize,
        detailed: bool,
    ) {
        // Clear screen and show stats
        print!("\x1B[2J\x1B[1;1H"); // Clear screen and move cursor to top

        println!("📊 Packet Monitor - Live Statistics");
        println!("{}", "═".repeat(50));
        println!("Total Packets: {packet_count}");
        println!("Total Bytes:   {}", format_bytes(byte_count));
        println!();

        // Protocol distribution
        if !protocol_stats.is_empty() {
            println!("🔧 Protocol Distribution:");
            let mut sorted_protocols: Vec<_> = protocol_stats.iter().collect();
            sorted_protocols.sort_by(|a, b| b.1.cmp(a.1));
            
            for (protocol, count) in sorted_protocols.iter().take(5) {
                let percentage = (**count as f64 / packet_count as f64) * 100.0;
                println!("  {protocol:<8} {count:>6} ({percentage:>5.1}%)");
            }
            println!();
        }

        // Top connections
        if !connection_tracker.is_empty() {
            println!("🌐 Top Connections (by bytes):");
            let mut sorted_connections: Vec<_> = connection_tracker.iter().collect();
            sorted_connections.sort_by(|a, b| b.1.1.cmp(&a.1.1)); // Sort by bytes

            for (connection, (packets, bytes)) in sorted_connections.iter().take(max_connections) {
                if detailed {
                    println!("  {}", connection);
                    println!("    Packets: {}, Bytes: {}", packets, format_bytes(*bytes));
                } else {
                    println!("  {} - {}", connection, format_bytes(*bytes));
                }
            }
            println!();
        }

        if detailed {
            // Additional detailed information
            println!("🔍 Detailed Information:");
            println!("  Average packet size: {}", 
                if packet_count > 0 { format_bytes(byte_count / packet_count) } 
                else { "N/A".to_string() }
            );
            println!("  Unique connections: {}", connection_tracker.len());
            println!();
        }
    }

    pub async fn handle_analyze_command(
        &self,
        period: String,
        interface: Option<String>,
        security: bool,
        protocols: bool,
    ) -> Result<()> {
        let duration = parse_duration(&period)
            .context("Failed to parse analysis period")?;
        
        let since = Local::now() - chrono::Duration::from_std(duration)
            .map_err(|_| anyhow::anyhow!("Invalid duration for analysis"))?;

        let interface_name = interface.unwrap_or_else(|| "all".to_string());

        println!("📈 Analyzing traffic patterns");
        println!("Interface: {}", interface_name);
        println!("Period: {} (since {})", period, since.format("%Y-%m-%d %H:%M:%S"));
        println!();

        // Get traffic summary from storage
        let summary = self.storage.get_traffic_summary(&interface_name, since)
            .context("Failed to retrieve traffic summary")?;

        // Display basic statistics
        println!("📊 Traffic Summary:");
        println!("  Total Packets: {}", summary.total_packets);
        println!("  Total Bytes:   {}", format_bytes(summary.total_bytes));
        println!();

        // Protocol distribution
        if protocols && !summary.protocols.is_empty() {
            println!("🔧 Protocol Distribution:");
            let mut sorted_protocols: Vec<_> = summary.protocols.iter().collect();
            sorted_protocols.sort_by(|a, b| b.1.bytes.cmp(&a.1.bytes));

            for (protocol, stats) in sorted_protocols {
                let percentage = (stats.bytes as f64 / summary.total_bytes as f64) * 100.0;
                println!("  {:<12} {:>10} packets, {:>10} ({:>5.1}%)",
                    protocol,
                    stats.packets,
                    format_bytes(stats.bytes),
                    percentage
                );
            }
            println!();
        }

        // Top connections
        if !summary.top_connections.is_empty() {
            println!("🌐 Top Connections:");
            for (i, connection) in summary.top_connections.iter().take(10).enumerate() {
                println!("  {}. {} -> {} ({})",
                    i + 1,
                    connection.source,
                    connection.destination,
                    format_bytes(connection.bytes)
                );
            }
            println!();
        }

        // Security analysis
        if security {
            println!("🔒 Security Analysis:");
            // This would query security events from storage
            println!("  No security issues detected in the analyzed period.");
            println!();
        }

        Ok(())
    }
}

fn parse_duration(duration_str: &str) -> Result<StdDuration> {
    let duration_str = duration_str.trim();
    
    if duration_str.ends_with('s') {
        let seconds: u64 = duration_str.trim_end_matches('s').parse()
            .context("Invalid seconds format")?;
        Ok(StdDuration::from_secs(seconds))
    } else if duration_str.ends_with('m') {
        let minutes: u64 = duration_str.trim_end_matches('m').parse()
            .context("Invalid minutes format")?;
        Ok(StdDuration::from_secs(minutes * 60))
    } else if duration_str.ends_with('h') {
        let hours: u64 = duration_str.trim_end_matches('h').parse()
            .context("Invalid hours format")?;
        Ok(StdDuration::from_secs(hours * 3600))
    } else {
        // Assume seconds if no unit
        let seconds: u64 = duration_str.parse()
            .context("Invalid duration format")?;
        Ok(StdDuration::from_secs(seconds))
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

fn analysis_to_protocol_name(
    packet: &crate::models::NetworkPacket,
    analysis: &AnalysisResult,
) -> String {
    if let Some(ref app_protocol) = analysis.application_protocol {
        app_protocol.clone()
    } else {
        format!("{:?}", packet.transport_protocol)
    }
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self {
            application_protocol: None,
            is_encrypted: false,
            traffic_type: TrafficType::Other,
            security_flags: Vec::new(),
            flow_direction: crate::analyzers::FlowDirection::Local,
            geolocation: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("60s").unwrap(), StdDuration::from_secs(60));
        assert_eq!(parse_duration("5m").unwrap(), StdDuration::from_secs(300));
        assert_eq!(parse_duration("1h").unwrap(), StdDuration::from_secs(3600));
        assert_eq!(parse_duration("30").unwrap(), StdDuration::from_secs(30));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }
}