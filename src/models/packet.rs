// Core packet data models for network monitoring
// Defines structures for representing captured network packets and related statistics

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

/// Core data structure representing a captured network packet
/// Contains all relevant metadata for analysis and storage
/// 
/// # Examples
/// 
/// ```rust
/// let packet = NetworkPacket::new(
///     "eth0".to_string(),
///     1500,
///     PacketProtocol::IPv4,
///     PacketDirection::Outbound
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPacket {
    /// When the packet was captured (local system time)
    pub timestamp: DateTime<Local>,
    /// Network interface name where packet was captured (e.g., "eth0", "wlan0")
    pub interface: String,
    /// Total packet size in bytes including headers
    pub size_bytes: u64,
    /// Network layer protocol (IPv4, IPv6, etc.)
    pub protocol: PacketProtocol,
    /// Transport layer protocol (TCP, UDP, ICMP, etc.)
    pub transport_protocol: TransportProtocol,
    /// Source IP address (None for non-IP packets)
    pub source_addr: Option<IpAddr>,
    /// Destination IP address (None for non-IP packets)
    pub dest_addr: Option<IpAddr>,
    /// Source port number (None for protocols without ports)
    pub source_port: Option<u16>,
    /// Destination port number (None for protocols without ports)
    pub dest_port: Option<u16>,
    /// Traffic direction relative to the monitoring system
    pub direction: PacketDirection,
}

/// Represents the network layer protocol of a captured packet
/// Used to classify packets at the IP level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PacketProtocol {
    /// Ethernet frame (Layer 2)
    Ethernet,
    /// Internet Protocol version 4
    IPv4,
    /// Internet Protocol version 6
    IPv6,
    /// Address Resolution Protocol
    ARP,
    /// Other protocol types identified by EtherType
    Other(u16),
}

/// Represents the transport layer protocol of a captured packet
/// Used to classify packets at the TCP/UDP level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportProtocol {
    /// Transmission Control Protocol
    TCP,
    /// User Datagram Protocol
    UDP,
    /// Internet Control Message Protocol
    ICMP,
    /// Internet Control Message Protocol version 6
    ICMPv6,
    /// Other transport protocols identified by IP protocol number
    Other(u8),
}

/// Indicates the direction of packet flow relative to the monitoring system
/// Used for traffic analysis and bandwidth calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketDirection {
    /// Traffic coming into the system
    Inbound,
    /// Traffic going out from the system
    Outbound,
    /// Traffic between local interfaces (loopback)
    Local,
}

/// Comprehensive statistics for a time period of packet capture
/// Aggregates multiple metrics for performance analysis and reporting
/// 
/// # Usage
/// 
/// This structure is used to store periodic snapshots of network activity,
/// typically collected every few seconds during monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketStatistics {
    /// Total number of packets captured in this period
    pub total_packets: u64,
    /// Total bytes captured in this period
    pub total_bytes: u64,
    /// Average packets per second during this period
    pub packets_per_second: f64,
    /// Average bytes per second during this period (bandwidth)
    pub bytes_per_second: f64,
    /// Breakdown of traffic by transport protocol
    pub protocol_distribution: ProtocolDistribution,
    /// Most active network connections during this period
    pub top_connections: Vec<ConnectionInfo>,
    /// Start of the measurement period
    pub start_time: DateTime<Local>,
    /// End of the measurement period
    pub end_time: DateTime<Local>,
}

/// Statistics tracking the distribution of different transport protocols
/// Used for aggregating packet counts and byte totals by protocol type
/// 
/// # Fields
/// 
/// All fields track both packet counts and byte totals for each protocol
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProtocolDistribution {
    /// Number of TCP packets captured
    pub tcp_packets: u64,
    /// Number of UDP packets captured
    pub udp_packets: u64,
    /// Number of ICMP packets captured
    pub icmp_packets: u64,
    /// Number of packets using other protocols
    pub other_packets: u64,
    /// Total bytes transferred via TCP
    pub tcp_bytes: u64,
    /// Total bytes transferred via UDP
    pub udp_bytes: u64,
    /// Total bytes transferred via ICMP
    pub icmp_bytes: u64,
    /// Total bytes transferred via other protocols
    pub other_bytes: u64,
}

/// Information about a network connection derived from packet analysis
/// Used to track and display the most active connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Source address and port in "IP:port" format
    pub source: String,
    /// Destination address and port in "IP:port" format
    pub destination: String,
    /// Transport protocol used by this connection
    pub protocol: TransportProtocol,
    /// Total number of packets in this connection
    pub packets: u64,
    /// Total bytes transferred in this connection
    pub bytes: u64,
}

/// Represents a known application-layer protocol
/// Used for protocol identification during packet analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationProtocol {
    /// Human-readable name of the protocol (e.g., "HTTP", "HTTPS")
    pub name: String,
    /// Well-known port number for this protocol
    pub port: u16,
    /// Underlying transport protocol (TCP/UDP)
    pub transport: TransportProtocol,
}

impl NetworkPacket {
    /// Creates a new NetworkPacket with basic information
    /// 
    /// # Arguments
    /// 
    /// * `interface` - Network interface name (e.g., "eth0")
    /// * `size_bytes` - Total packet size including headers
    /// * `protocol` - Network layer protocol
    /// * `direction` - Traffic direction (inbound/outbound/local)
    /// 
    /// # Returns
    /// 
    /// A new NetworkPacket with timestamp set to current time
    /// and optional fields (addresses, ports) set to None
    pub fn new(
        interface: String,
        size_bytes: u64,
        protocol: PacketProtocol,
        direction: PacketDirection,
    ) -> Self {
        Self {
            timestamp: Local::now(),
            interface,
            size_bytes,
            protocol,
            transport_protocol: TransportProtocol::Other(0),
            source_addr: None,
            dest_addr: None,
            source_port: None,
            dest_port: None,
            direction,
        }
    }

    pub fn is_tcp(&self) -> bool {
        matches!(self.transport_protocol, TransportProtocol::TCP)
    }

    pub fn is_udp(&self) -> bool {
        matches!(self.transport_protocol, TransportProtocol::UDP)
    }

    pub fn is_icmp(&self) -> bool {
        matches!(
            self.transport_protocol,
            TransportProtocol::ICMP | TransportProtocol::ICMPv6
        )
    }

    pub fn connection_string(&self) -> String {
        match (self.source_addr, self.source_port, self.dest_addr, self.dest_port) {
            (Some(src_ip), Some(src_port), Some(dst_ip), Some(dst_port)) => {
                format!("{src_ip}:{src_port} -> {dst_ip}:{dst_port}")
            }
            (Some(src_ip), None, Some(dst_ip), None) => {
                format!("{src_ip} -> {dst_ip}")
            }
            _ => "Unknown".to_string(),
        }
    }
}


impl ProtocolDistribution {
    pub fn add_packet(&mut self, packet: &NetworkPacket) {
        match packet.transport_protocol {
            TransportProtocol::TCP => {
                self.tcp_packets += 1;
                self.tcp_bytes += packet.size_bytes;
            }
            TransportProtocol::UDP => {
                self.udp_packets += 1;
                self.udp_bytes += packet.size_bytes;
            }
            TransportProtocol::ICMP | TransportProtocol::ICMPv6 => {
                self.icmp_packets += 1;
                self.icmp_bytes += packet.size_bytes;
            }
            TransportProtocol::Other(_) => {
                self.other_packets += 1;
                self.other_bytes += packet.size_bytes;
            }
        }
    }

    pub fn total_packets(&self) -> u64 {
        self.tcp_packets + self.udp_packets + self.icmp_packets + self.other_packets
    }

    pub fn total_bytes(&self) -> u64 {
        self.tcp_bytes + self.udp_bytes + self.icmp_bytes + self.other_bytes
    }
}

pub fn common_application_protocols() -> HashMap<u16, ApplicationProtocol> {
    let mut protocols = HashMap::new();
    
    protocols.insert(80, ApplicationProtocol {
        name: "HTTP".to_string(),
        port: 80,
        transport: TransportProtocol::TCP,
    });
    
    protocols.insert(443, ApplicationProtocol {
        name: "HTTPS".to_string(),
        port: 443,
        transport: TransportProtocol::TCP,
    });
    
    protocols.insert(22, ApplicationProtocol {
        name: "SSH".to_string(),
        port: 22,
        transport: TransportProtocol::TCP,
    });
    
    protocols.insert(53, ApplicationProtocol {
        name: "DNS".to_string(),
        port: 53,
        transport: TransportProtocol::UDP,
    });
    
    protocols.insert(25, ApplicationProtocol {
        name: "SMTP".to_string(),
        port: 25,
        transport: TransportProtocol::TCP,
    });
    
    protocols.insert(110, ApplicationProtocol {
        name: "POP3".to_string(),
        port: 110,
        transport: TransportProtocol::TCP,
    });
    
    protocols.insert(143, ApplicationProtocol {
        name: "IMAP".to_string(),
        port: 143,
        transport: TransportProtocol::TCP,
    });
    
    protocols.insert(3389, ApplicationProtocol {
        name: "RDP".to_string(),
        port: 3389,
        transport: TransportProtocol::TCP,
    });
    
    protocols.insert(21, ApplicationProtocol {
        name: "FTP".to_string(),
        port: 21,
        transport: TransportProtocol::TCP,
    });
    
    protocols.insert(67, ApplicationProtocol {
        name: "DHCP".to_string(),
        port: 67,
        transport: TransportProtocol::UDP,
    });
    
    protocols
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_creation() {
        let packet = NetworkPacket::new(
            "eth0".to_string(),
            1500,
            PacketProtocol::IPv4,
            PacketDirection::Inbound,
        );
        
        assert_eq!(packet.interface, "eth0");
        assert_eq!(packet.size_bytes, 1500);
        assert_eq!(packet.protocol, PacketProtocol::IPv4);
        assert_eq!(packet.direction, PacketDirection::Inbound);
    }

    #[test]
    fn test_protocol_distribution() {
        let mut dist = ProtocolDistribution::default();
        
        let mut packet = NetworkPacket::new(
            "eth0".to_string(),
            100,
            PacketProtocol::IPv4,
            PacketDirection::Inbound,
        );
        packet.transport_protocol = TransportProtocol::TCP;
        
        dist.add_packet(&packet);
        
        assert_eq!(dist.tcp_packets, 1);
        assert_eq!(dist.tcp_bytes, 100);
        assert_eq!(dist.total_packets(), 1);
        assert_eq!(dist.total_bytes(), 100);
    }

    #[test]
    fn test_connection_string() {
        use std::net::{IpAddr, Ipv4Addr};
        
        let mut packet = NetworkPacket::new(
            "eth0".to_string(),
            100,
            PacketProtocol::IPv4,
            PacketDirection::Outbound,
        );
        
        packet.source_addr = Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));
        packet.source_port = Some(12345);
        packet.dest_addr = Some(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        packet.dest_port = Some(443);
        
        assert_eq!(packet.connection_string(), "192.168.1.100:12345 -> 8.8.8.8:443");
    }
}