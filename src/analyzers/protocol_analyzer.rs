// ProtocolAnalyzer: Advanced packet analysis and protocol identification
// Performs deep packet inspection to identify application protocols and security patterns
// Maintains connection state and generates security alerts

use crate::models::{
    common_application_protocols, ApplicationProtocol, NetworkPacket, TransportProtocol,
};
use anyhow::Result;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr};

/// Advanced protocol analyzer for network traffic inspection
/// 
/// Performs deep packet inspection to identify application-layer protocols,
/// track network connections, and detect security anomalies. Uses heuristic
/// analysis based on port numbers, packet patterns, and traffic characteristics.
/// 
/// # Features
/// 
/// - Application protocol identification (HTTP, HTTPS, DNS, etc.)
/// - Connection state tracking with automatic cleanup
/// - Security pattern detection (suspicious ports, unencrypted sensitive data)
/// - Traffic classification (Web, Email, P2P, etc.)
/// - Geolocation analysis (planned)
/// 
/// # Example
/// 
/// ```rust
/// let mut analyzer = ProtocolAnalyzer::new();
/// let analysis = analyzer.analyze_packet(&packet)?;
/// 
/// if let Some(protocol) = analysis.application_protocol {
///     println!("Detected {protocol} traffic");
/// }
/// ```
pub struct ProtocolAnalyzer {
    /// Database of well-known protocols mapped by port number
    known_protocols: HashMap<u16, ApplicationProtocol>,
    /// Aggregate statistics for all analyzed protocols
    protocol_stats: ProtocolStats,
    /// Active connection tracking for state analysis
    connection_tracker: ConnectionTracker,
}

/// Statistical counters for protocol analysis
/// Tracks various metrics to provide insights into network traffic patterns
#[derive(Debug, Clone, Default)]
pub struct ProtocolStats {
    /// Total number of TCP connections observed
    pub tcp_connections: u64,
    /// Total number of UDP sessions tracked
    pub udp_sessions: u64,
    /// Total ICMP packets processed
    pub icmp_packets: u64,
    /// DNS queries detected
    pub dns_queries: u64,
    /// HTTP requests identified
    pub http_requests: u64,
    pub https_connections: u64,
    pub ssh_connections: u64,
    pub other_protocols: u64,
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub source: IpAddr,
    pub destination: IpAddr,
    pub source_port: Option<u16>,
    pub dest_port: Option<u16>,
    pub protocol: TransportProtocol,
    pub packets: u64,
    pub bytes: u64,
    pub first_seen: chrono::DateTime<chrono::Local>,
    pub last_seen: chrono::DateTime<chrono::Local>,
    pub application_protocol: Option<String>,
}

#[derive(Debug, Default)]
pub struct ConnectionTracker {
    connections: HashMap<String, ConnectionInfo>,
    max_connections: usize,
}

impl ConnectionTracker {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: HashMap::new(),
            max_connections,
        }
    }

    pub fn track_connection(&mut self, packet: &NetworkPacket) {
        if let (Some(src), Some(dst)) = (packet.source_addr, packet.dest_addr) {
            let connection_key = format!(
                "{}:{}-{}:{}",
                src,
                packet.source_port.unwrap_or(0),
                dst,
                packet.dest_port.unwrap_or(0)
            );

            if let Some(connection) = self.connections.get_mut(&connection_key) {
                connection.packets += 1;
                connection.bytes += packet.size_bytes;
                connection.last_seen = packet.timestamp;
            } else {
                if self.connections.len() >= self.max_connections {
                    self.cleanup_old_connections();
                }

                let connection = ConnectionInfo {
                    source: src,
                    destination: dst,
                    source_port: packet.source_port,
                    dest_port: packet.dest_port,
                    protocol: packet.transport_protocol,
                    packets: 1,
                    bytes: packet.size_bytes,
                    first_seen: packet.timestamp,
                    last_seen: packet.timestamp,
                    application_protocol: None,
                };

                self.connections.insert(connection_key, connection);
            }
        }
    }

    pub fn get_top_connections(&self, limit: usize) -> Vec<&ConnectionInfo> {
        let mut connections: Vec<&ConnectionInfo> = self.connections.values().collect();
        connections.sort_by(|a, b| b.bytes.cmp(&a.bytes));
        connections.into_iter().take(limit).collect()
    }

    pub fn get_connection_count(&self) -> usize {
        self.connections.len()
    }

    fn cleanup_old_connections(&mut self) {
        let cutoff_time = chrono::Local::now() - chrono::Duration::minutes(5);
        self.connections.retain(|_, connection| connection.last_seen > cutoff_time);
    }
}

impl Default for ProtocolAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolAnalyzer {
    pub fn new() -> Self {
        Self {
            known_protocols: common_application_protocols(),
            protocol_stats: ProtocolStats::default(),
            connection_tracker: ConnectionTracker::new(10000),
        }
    }

    pub fn analyze_packet(&mut self, packet: &NetworkPacket) -> Result<AnalysisResult> {
        self.connection_tracker.track_connection(packet);
        
        let mut result = AnalysisResult {
            application_protocol: self.identify_application_protocol(packet),
            is_encrypted: self.is_encrypted_traffic(packet),
            traffic_type: self.classify_traffic_type(packet),
            security_flags: self.check_security_flags(packet),
            flow_direction: FlowDirection::Local, // Will be updated below
            geolocation: None, // Will be updated below
        };

        self.update_stats(packet, &result);

        result.flow_direction = self.determine_flow_direction(packet);
        result.geolocation = self.get_geolocation_info(packet);

        Ok(result)
    }

    pub fn get_stats(&self) -> &ProtocolStats {
        &self.protocol_stats
    }

    pub fn get_top_connections(&self, limit: usize) -> Vec<&ConnectionInfo> {
        self.connection_tracker.get_top_connections(limit)
    }

    pub fn get_connection_count(&self) -> usize {
        self.connection_tracker.get_connection_count()
    }

    fn identify_application_protocol(&self, packet: &NetworkPacket) -> Option<String> {
        if let Some(port) = packet.dest_port.or(packet.source_port) {
            if let Some(protocol) = self.known_protocols.get(&port) {
                return Some(protocol.name.clone());
            }
        }

        match packet.transport_protocol {
            TransportProtocol::TCP => self.analyze_tcp_payload(packet),
            TransportProtocol::UDP => self.analyze_udp_payload(packet),
            _ => None,
        }
    }

    fn analyze_tcp_payload(&self, packet: &NetworkPacket) -> Option<String> {
        match (packet.source_port, packet.dest_port) {
            (Some(80), _) | (_, Some(80)) => Some("HTTP".to_string()),
            (Some(443), _) | (_, Some(443)) => Some("HTTPS".to_string()),
            (Some(22), _) | (_, Some(22)) => Some("SSH".to_string()),
            (Some(25), _) | (_, Some(25)) => Some("SMTP".to_string()),
            (Some(110), _) | (_, Some(110)) => Some("POP3".to_string()),
            (Some(143), _) | (_, Some(143)) => Some("IMAP".to_string()),
            (Some(993), _) | (_, Some(993)) => Some("IMAPS".to_string()),
            (Some(995), _) | (_, Some(995)) => Some("POP3S".to_string()),
            (Some(21), _) | (_, Some(21)) => Some("FTP".to_string()),
            (Some(23), _) | (_, Some(23)) => Some("Telnet".to_string()),
            _ => None,
        }
    }

    fn analyze_udp_payload(&self, packet: &NetworkPacket) -> Option<String> {
        match (packet.source_port, packet.dest_port) {
            (Some(53), _) | (_, Some(53)) => Some("DNS".to_string()),
            (Some(67), _) | (_, Some(67)) => Some("DHCP".to_string()),
            (Some(68), _) | (_, Some(68)) => Some("DHCP".to_string()),
            (Some(123), _) | (_, Some(123)) => Some("NTP".to_string()),
            (Some(161), _) | (_, Some(161)) => Some("SNMP".to_string()),
            (Some(162), _) | (_, Some(162)) => Some("SNMP".to_string()),
            (Some(514), _) | (_, Some(514)) => Some("Syslog".to_string()),
            _ => None,
        }
    }

    fn is_encrypted_traffic(&self, packet: &NetworkPacket) -> bool {
        match (packet.source_port, packet.dest_port) {
            (Some(443), _) | (_, Some(443)) => true, // HTTPS
            (Some(993), _) | (_, Some(993)) => true, // IMAPS
            (Some(995), _) | (_, Some(995)) => true, // POP3S
            (Some(22), _) | (_, Some(22)) => true,   // SSH
            (Some(990), _) | (_, Some(990)) => true, // FTPS
            _ => false,
        }
    }

    fn classify_traffic_type(&self, packet: &NetworkPacket) -> TrafficType {
        if self.is_local_traffic(packet) {
            return TrafficType::Local;
        }

        if self.is_web_traffic(packet) {
            return TrafficType::Web;
        }

        if self.is_email_traffic(packet) {
            return TrafficType::Email;
        }

        if self.is_file_transfer(packet) {
            return TrafficType::FileTransfer;
        }

        if self.is_streaming_traffic(packet) {
            return TrafficType::Streaming;
        }

        TrafficType::Other
    }

    fn is_local_traffic(&self, packet: &NetworkPacket) -> bool {
        if let (Some(src), Some(dst)) = (packet.source_addr, packet.dest_addr) {
            match (src, dst) {
                (IpAddr::V4(src_v4), IpAddr::V4(dst_v4)) => {
                    src_v4.is_private() && dst_v4.is_private()
                }
                (IpAddr::V6(src_v6), IpAddr::V6(dst_v6)) => {
                    (src_v6.is_loopback() || Self::is_private_ipv6(&src_v6))
                        && (dst_v6.is_loopback() || Self::is_private_ipv6(&dst_v6))
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn is_private_ipv6(addr: &Ipv6Addr) -> bool {
        addr.segments()[0] & 0xfe00 == 0xfc00 || // Unique local addresses
        addr.segments()[0] & 0xffc0 == 0xfe80     // Link-local addresses
    }

    fn is_streaming_traffic(&self, packet: &NetworkPacket) -> bool {
        match (packet.source_port, packet.dest_port) {
            (Some(1935), _) | (_, Some(1935)) => true, // RTMP
            _ => packet.size_bytes > 1024, // Large packets often indicate streaming
        }
    }

    fn is_web_traffic(&self, packet: &NetworkPacket) -> bool {
        match (packet.source_port, packet.dest_port) {
            (Some(80), _) | (_, Some(80)) => true,   // HTTP
            (Some(443), _) | (_, Some(443)) => true, // HTTPS
            (Some(8080), _) | (_, Some(8080)) => true, // Alt HTTP
            (Some(8443), _) | (_, Some(8443)) => true, // Alt HTTPS
            _ => false,
        }
    }

    fn is_email_traffic(&self, packet: &NetworkPacket) -> bool {
        match (packet.source_port, packet.dest_port) {
            (Some(25), _) | (_, Some(25)) => true,   // SMTP
            (Some(110), _) | (_, Some(110)) => true, // POP3
            (Some(143), _) | (_, Some(143)) => true, // IMAP
            (Some(993), _) | (_, Some(993)) => true, // IMAPS
            (Some(995), _) | (_, Some(995)) => true, // POP3S
            _ => false,
        }
    }

    fn is_file_transfer(&self, packet: &NetworkPacket) -> bool {
        match (packet.source_port, packet.dest_port) {
            (Some(21), _) | (_, Some(21)) => true,   // FTP
            (Some(22), _) | (_, Some(22)) => true,   // SFTP/SCP
            (Some(990), _) | (_, Some(990)) => true, // FTPS
            _ => false,
        }
    }

    fn check_security_flags(&self, packet: &NetworkPacket) -> Vec<SecurityFlag> {
        let mut flags = Vec::new();

        if self.is_suspicious_port(packet) {
            flags.push(SecurityFlag::SuspiciousPort);
        }

        if self.is_unencrypted_sensitive(packet) {
            flags.push(SecurityFlag::UnencryptedSensitive);
        }

        if self.is_high_frequency(packet) {
            flags.push(SecurityFlag::HighFrequency);
        }

        flags
    }

    fn is_suspicious_port(&self, packet: &NetworkPacket) -> bool {
        let suspicious_ports = [1337, 31337, 12345, 54321, 9999];
        if let Some(port) = packet.dest_port.or(packet.source_port) {
            suspicious_ports.contains(&port)
        } else {
            false
        }
    }

    fn is_unencrypted_sensitive(&self, packet: &NetworkPacket) -> bool {
        match (packet.source_port, packet.dest_port) {
            (Some(23), _) | (_, Some(23)) => true, // Telnet
            (Some(21), _) | (_, Some(21)) => true, // FTP
            (Some(110), _) | (_, Some(110)) => true, // POP3
            (Some(143), _) | (_, Some(143)) => true, // IMAP
            _ => false,
        }
    }

    fn is_high_frequency(&self, _packet: &NetworkPacket) -> bool {
        false
    }

    fn determine_flow_direction(&self, packet: &NetworkPacket) -> FlowDirection {
        match packet.direction {
            crate::models::PacketDirection::Inbound => FlowDirection::Inbound,
            crate::models::PacketDirection::Outbound => FlowDirection::Outbound,
            crate::models::PacketDirection::Local => FlowDirection::Local,
        }
    }

    fn get_geolocation_info(&self, packet: &NetworkPacket) -> Option<GeolocationInfo> {
        if let Some(remote_addr) = self.get_remote_address(packet) {
            match remote_addr {
                IpAddr::V4(ipv4) if ipv4.is_private() => Some(GeolocationInfo {
                    country: "Private".to_string(),
                    region: "LAN".to_string(),
                    is_private: true,
                }),
                _ => None,
            }
        } else {
            None
        }
    }

    fn get_remote_address(&self, packet: &NetworkPacket) -> Option<IpAddr> {
        match packet.direction {
            crate::models::PacketDirection::Inbound => packet.source_addr,
            crate::models::PacketDirection::Outbound => packet.dest_addr,
            crate::models::PacketDirection::Local => None,
        }
    }

    fn update_stats(&mut self, packet: &NetworkPacket, result: &AnalysisResult) {
        match packet.transport_protocol {
            TransportProtocol::TCP => self.protocol_stats.tcp_connections += 1,
            TransportProtocol::UDP => self.protocol_stats.udp_sessions += 1,
            TransportProtocol::ICMP | TransportProtocol::ICMPv6 => {
                self.protocol_stats.icmp_packets += 1
            }
            _ => self.protocol_stats.other_protocols += 1,
        }

        if let Some(ref protocol) = result.application_protocol {
            match protocol.as_str() {
                "DNS" => self.protocol_stats.dns_queries += 1,
                "HTTP" => self.protocol_stats.http_requests += 1,
                "HTTPS" => self.protocol_stats.https_connections += 1,
                "SSH" => self.protocol_stats.ssh_connections += 1,
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub application_protocol: Option<String>,
    pub is_encrypted: bool,
    pub traffic_type: TrafficType,
    pub security_flags: Vec<SecurityFlag>,
    pub flow_direction: FlowDirection,
    pub geolocation: Option<GeolocationInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrafficType {
    Web,
    Email,
    FileTransfer,
    Streaming,
    Gaming,
    VoIP,
    Local,
    Other,
}

#[derive(Debug, Clone)]
pub enum SecurityFlag {
    SuspiciousPort,
    UnencryptedSensitive,
    HighFrequency,
    UnknownProtocol,
    LargePayload,
}

#[derive(Debug, Clone)]
pub enum FlowDirection {
    Inbound,
    Outbound,
    Local,
}

#[derive(Debug, Clone)]
pub struct GeolocationInfo {
    pub country: String,
    pub region: String,
    pub is_private: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{NetworkPacket, PacketDirection, PacketProtocol};
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_protocol_analyzer_creation() {
        let analyzer = ProtocolAnalyzer::new();
        assert_eq!(analyzer.get_connection_count(), 0);
    }

    #[test]
    fn test_http_identification() {
        let mut analyzer = ProtocolAnalyzer::new();
        
        let mut packet = NetworkPacket::new(
            "eth0".to_string(),
            1500,
            PacketProtocol::IPv4,
            PacketDirection::Outbound,
        );
        packet.dest_port = Some(80);
        packet.transport_protocol = TransportProtocol::TCP;
        
        let result = analyzer.analyze_packet(&packet).unwrap();
        assert_eq!(result.application_protocol, Some("HTTP".to_string()));
        assert!(!result.is_encrypted);
        assert_eq!(result.traffic_type, TrafficType::Web);
    }

    #[test]
    fn test_https_identification() {
        let mut analyzer = ProtocolAnalyzer::new();
        
        let mut packet = NetworkPacket::new(
            "eth0".to_string(),
            1500,
            PacketProtocol::IPv4,
            PacketDirection::Outbound,
        );
        packet.dest_port = Some(443);
        packet.transport_protocol = TransportProtocol::TCP;
        
        let result = analyzer.analyze_packet(&packet).unwrap();
        assert_eq!(result.application_protocol, Some("HTTPS".to_string()));
        assert!(result.is_encrypted);
        assert_eq!(result.traffic_type, TrafficType::Web);
    }

    #[test]
    fn test_local_traffic_detection() {
        let mut analyzer = ProtocolAnalyzer::new();
        
        let mut packet = NetworkPacket::new(
            "eth0".to_string(),
            1500,
            PacketProtocol::IPv4,
            PacketDirection::Local,
        );
        packet.source_addr = Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        packet.dest_addr = Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)));
        
        let result = analyzer.analyze_packet(&packet).unwrap();
        assert_eq!(result.traffic_type, TrafficType::Local);
    }

    #[test]
    fn test_connection_tracking() {
        let mut analyzer = ProtocolAnalyzer::new();
        
        let mut packet = NetworkPacket::new(
            "eth0".to_string(),
            1500,
            PacketProtocol::IPv4,
            PacketDirection::Outbound,
        );
        packet.source_addr = Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        packet.dest_addr = Some(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        packet.source_port = Some(12345);
        packet.dest_port = Some(80);
        
        analyzer.analyze_packet(&packet).unwrap();
        assert_eq!(analyzer.get_connection_count(), 1);
        
        analyzer.analyze_packet(&packet).unwrap();
        assert_eq!(analyzer.get_connection_count(), 1); // Same connection
        
        let connections = analyzer.get_top_connections(5);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].packets, 2);
        assert_eq!(connections[0].bytes, 3000);
    }
}