use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPacket {
    pub timestamp: DateTime<Local>,
    pub interface: String,
    pub size_bytes: u64,
    pub protocol: PacketProtocol,
    pub transport_protocol: TransportProtocol,
    pub source_addr: Option<IpAddr>,
    pub dest_addr: Option<IpAddr>,
    pub source_port: Option<u16>,
    pub dest_port: Option<u16>,
    pub direction: PacketDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PacketProtocol {
    Ethernet,
    IPv4,
    IPv6,
    ARP,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportProtocol {
    TCP,
    UDP,
    ICMP,
    ICMPv6,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketDirection {
    Inbound,
    Outbound,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketStatistics {
    pub total_packets: u64,
    pub total_bytes: u64,
    pub packets_per_second: f64,
    pub bytes_per_second: f64,
    pub protocol_distribution: ProtocolDistribution,
    pub top_connections: Vec<ConnectionInfo>,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProtocolDistribution {
    pub tcp_packets: u64,
    pub udp_packets: u64,
    pub icmp_packets: u64,
    pub other_packets: u64,
    pub tcp_bytes: u64,
    pub udp_bytes: u64,
    pub icmp_bytes: u64,
    pub other_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub source: String,
    pub destination: String,
    pub protocol: TransportProtocol,
    pub packets: u64,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationProtocol {
    pub name: String,
    pub port: u16,
    pub transport: TransportProtocol,
}

impl NetworkPacket {
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