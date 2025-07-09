use anyhow::{Context, Result};
use chrono::Local;
use log::{error, info, warn};
use pnet::datalink::{self, Channel::Ethernet, NetworkInterface};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

use crate::models::{
    NetworkPacket, PacketDirection, PacketProtocol, PacketStatistics, ProtocolDistribution,
    TransportProtocol,
};

/// High-performance packet collector for network monitoring
/// 
/// Uses libpnet for raw packet capture with platform-specific optimizations.
/// Implements producer-consumer pattern with bounded channels for memory safety.
/// 
/// # Architecture
/// 
/// ```text
/// Raw Packets -> pnet capture -> Channel -> PacketCollector -> NetworkPacket
/// ```
/// 
/// # Platform Requirements
/// 
/// - Linux: Requires CAP_NET_RAW capability or root privileges
/// - macOS: Requires root privileges for BPF access
/// - Windows: Requires Administrator privileges and Npcap
/// 
/// # Example
/// 
/// ```rust
/// let mut collector = PacketCollector::new("eth0".to_string())?;
/// collector.start().await?;
/// 
/// while let Some(packet) = collector.receive_packet().await {
///     println!("Captured {} bytes", packet.size_bytes);
/// }
/// ```
pub struct PacketCollector {
    /// Network interface to monitor (e.g., "eth0", "wlan0")
    interface_name: String,
    /// Channel sender for captured packets (producer side)
    packet_sender: Sender<NetworkPacket>,
    /// Channel receiver for captured packets (consumer side)
    packet_receiver: Arc<Mutex<Receiver<NetworkPacket>>>,
    /// Shared statistics for monitoring capture performance
    stats: Arc<Mutex<PacketStatistics>>,
    /// Atomic flag to control capture loop execution
    running: Arc<Mutex<bool>>,
}

impl PacketCollector {
    /// Creates a new packet collector for the specified interface
    /// 
    /// # Arguments
    /// 
    /// * `interface_name` - Network interface to capture packets from
    /// 
    /// # Returns
    /// 
    /// A new PacketCollector instance with bounded channel (10,000 packets)
    /// and initialized statistics. The collector is created in stopped state.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let collector = PacketCollector::new("eth0".to_string())?;
    /// ```
    pub fn new(interface_name: String) -> Result<Self> {
        // Create bounded channel to prevent memory exhaustion under high traffic
        // Buffer size of 10,000 packets provides good balance between
        // responsiveness and memory usage
        let (sender, receiver) = mpsc::channel(10000);
        
        // Initialize packet statistics with zero values
        // These will be updated as packets are captured and processed
        let stats = PacketStatistics {
            total_packets: 0,
            total_bytes: 0,
            packets_per_second: 0.0,
            bytes_per_second: 0.0,
            protocol_distribution: ProtocolDistribution::default(),
            top_connections: Vec::new(),
            start_time: Local::now(),
            end_time: Local::now(),
        };

        Ok(Self {
            interface_name,
            packet_sender: sender,
            packet_receiver: Arc::new(Mutex::new(receiver)),
            stats: Arc::new(Mutex::new(stats)),
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// Starts packet capture on the configured interface
    /// 
    /// This method spawns a background task that performs the actual packet capture
    /// using libpnet. The captured packets are parsed and sent through the internal
    /// channel for consumption by `receive_packet()`.
    /// 
    /// # Errors
    /// 
    /// - Returns error if interface doesn't exist or lacks permissions
    /// - Returns error if packet capture cannot be initialized
    /// - On Windows, requires Npcap to be installed
    /// 
    /// # Platform Notes
    /// 
    /// - Linux: Requires CAP_NET_RAW or root privileges
    /// - macOS: Requires root privileges for BPF device access
    /// - Windows: Requires Administrator privileges and Npcap driver
    pub async fn start(&self) -> Result<()> {
        // Check if capture is already running to prevent duplicate tasks
        let mut running = self.running.lock().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        // Locate the specified network interface
        // This validates that the interface exists and is available for capture
        let interface = self
            .find_interface(&self.interface_name)
            .context("Failed to find network interface")?;

        info!("Starting packet capture on interface: {}", interface.name);

        // Clone shared references for use in the capture task
        let stats_clone = Arc::clone(&self.stats);
        let running_clone = Arc::clone(&self.running);
        let sender = self.packet_sender.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::capture_loop(interface, sender, stats_clone, running_clone).await {
                error!("Packet capture error: {}", e);
            }
        });

        let stats_clone = Arc::clone(&self.stats);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            let mut last_packets = 0u64;
            let mut last_bytes = 0u64;

            loop {
                interval.tick().await;
                let mut stats = stats_clone.lock().await;
                let current_packets = stats.total_packets;
                let current_bytes = stats.total_bytes;
                
                stats.packets_per_second = (current_packets - last_packets) as f64;
                stats.bytes_per_second = (current_bytes - last_bytes) as f64;
                stats.end_time = Local::now();
                
                last_packets = current_packets;
                last_bytes = current_bytes;
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        *running = false;
        info!("Stopping packet capture");
        Ok(())
    }

    pub async fn get_stats(&self) -> PacketStatistics {
        self.stats.lock().await.clone()
    }

    pub async fn receive_packet(&self) -> Option<NetworkPacket> {
        self.packet_receiver.lock().await.recv().await
    }

    fn find_interface(&self, name: &str) -> Option<NetworkInterface> {
        datalink::interfaces()
            .into_iter()
            .find(|iface| iface.name == name || (name == "any" && iface.is_up() && !iface.is_loopback()))
    }

    async fn capture_loop(
        interface: NetworkInterface,
        sender: Sender<NetworkPacket>,
        stats: Arc<Mutex<PacketStatistics>>,
        running: Arc<Mutex<bool>>,
    ) -> Result<()> {
        let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
            Ok(Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => return Err(anyhow::anyhow!("Unsupported channel type")),
            Err(e) => {
                if e.to_string().contains("Permission denied") {
                    return Err(anyhow::anyhow!(
                        "Permission denied. Packet capture requires elevated privileges (sudo/administrator)"
                    ));
                }
                return Err(anyhow::anyhow!("Failed to create datalink channel: {}", e));
            }
        };

        let interface_name = interface.name.clone();
        let local_ips: Vec<IpAddr> = interface
            .ips
            .iter()
            .filter_map(|ip| match ip.ip() {
                IpAddr::V4(addr) if !addr.is_loopback() => Some(IpAddr::V4(addr)),
                IpAddr::V6(addr) if !addr.is_loopback() => Some(IpAddr::V6(addr)),
                _ => None,
            })
            .collect();

        info!("Capturing packets on {} with IPs: {:?}", interface_name, local_ips);

        while *running.lock().await {
            match rx.next() {
                Ok(packet) => {
                    if let Some(ethernet) = EthernetPacket::new(packet) {
                        if let Some(network_packet) = Self::process_ethernet_packet(
                            &ethernet,
                            &interface_name,
                            &local_ips,
                        ) {
                            let mut stats_guard = stats.lock().await;
                            stats_guard.total_packets += 1;
                            stats_guard.total_bytes += network_packet.size_bytes;
                            stats_guard.protocol_distribution.add_packet(&network_packet);
                            drop(stats_guard);

                            if let Err(e) = sender.send(network_packet).await {
                                warn!("Failed to send packet to receiver: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error receiving packet: {}", e);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }

        Ok(())
    }

    fn process_ethernet_packet(
        ethernet: &EthernetPacket,
        interface_name: &str,
        local_ips: &[IpAddr],
    ) -> Option<NetworkPacket> {
        let mut packet = NetworkPacket::new(
            interface_name.to_string(),
            ethernet.packet().len() as u64,
            PacketProtocol::Ethernet,
            PacketDirection::Local,
        );

        match ethernet.get_ethertype() {
            EtherTypes::Ipv4 => {
                if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                    packet.protocol = PacketProtocol::IPv4;
                    packet.source_addr = Some(IpAddr::V4(ipv4.get_source()));
                    packet.dest_addr = Some(IpAddr::V4(ipv4.get_destination()));
                    
                    packet.direction = Self::determine_direction(
                        packet.source_addr.unwrap(),
                        packet.dest_addr.unwrap(),
                        local_ips,
                    );

                    match ipv4.get_next_level_protocol() {
                        IpNextHeaderProtocols::Tcp => {
                            packet.transport_protocol = TransportProtocol::TCP;
                            if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                                packet.source_port = Some(tcp.get_source());
                                packet.dest_port = Some(tcp.get_destination());
                            }
                        }
                        IpNextHeaderProtocols::Udp => {
                            packet.transport_protocol = TransportProtocol::UDP;
                            if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                                packet.source_port = Some(udp.get_source());
                                packet.dest_port = Some(udp.get_destination());
                            }
                        }
                        IpNextHeaderProtocols::Icmp => {
                            packet.transport_protocol = TransportProtocol::ICMP;
                        }
                        _ => {
                            packet.transport_protocol = TransportProtocol::Other(ipv4.get_next_level_protocol().0);
                        }
                    }
                }
            }
            EtherTypes::Ipv6 => {
                if let Some(ipv6) = Ipv6Packet::new(ethernet.payload()) {
                    packet.protocol = PacketProtocol::IPv6;
                    packet.source_addr = Some(IpAddr::V6(ipv6.get_source()));
                    packet.dest_addr = Some(IpAddr::V6(ipv6.get_destination()));
                    
                    packet.direction = Self::determine_direction(
                        packet.source_addr.unwrap(),
                        packet.dest_addr.unwrap(),
                        local_ips,
                    );

                    match ipv6.get_next_header() {
                        IpNextHeaderProtocols::Tcp => {
                            packet.transport_protocol = TransportProtocol::TCP;
                            if let Some(tcp) = TcpPacket::new(ipv6.payload()) {
                                packet.source_port = Some(tcp.get_source());
                                packet.dest_port = Some(tcp.get_destination());
                            }
                        }
                        IpNextHeaderProtocols::Udp => {
                            packet.transport_protocol = TransportProtocol::UDP;
                            if let Some(udp) = UdpPacket::new(ipv6.payload()) {
                                packet.source_port = Some(udp.get_source());
                                packet.dest_port = Some(udp.get_destination());
                            }
                        }
                        IpNextHeaderProtocols::Icmpv6 => {
                            packet.transport_protocol = TransportProtocol::ICMPv6;
                        }
                        _ => {
                            packet.transport_protocol = TransportProtocol::Other(ipv6.get_next_header().0);
                        }
                    }
                }
            }
            EtherTypes::Arp => {
                packet.protocol = PacketProtocol::ARP;
            }
            _ => {
                packet.protocol = PacketProtocol::Other(ethernet.get_ethertype().0);
            }
        }

        Some(packet)
    }

    fn determine_direction(
        source: IpAddr,
        dest: IpAddr,
        local_ips: &[IpAddr],
    ) -> PacketDirection {
        let source_is_local = local_ips.contains(&source) || source.is_loopback();
        let dest_is_local = local_ips.contains(&dest) || dest.is_loopback();

        match (source_is_local, dest_is_local) {
            (true, true) => PacketDirection::Local,
            (true, false) => PacketDirection::Outbound,
            (false, true) => PacketDirection::Inbound,
            (false, false) => PacketDirection::Local,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_packet_collector_creation() {
        let collector = PacketCollector::new("eth0".to_string()).unwrap();
        let stats = collector.get_stats().await;
        assert_eq!(stats.total_packets, 0);
        assert_eq!(stats.total_bytes, 0);
    }

    #[test]
    fn test_direction_determination() {
        use std::net::{Ipv4Addr, Ipv6Addr};

        let local_ips = vec![
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1)),
        ];

        let dir = PacketCollector::determine_direction(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            &local_ips,
        );
        assert_eq!(dir, PacketDirection::Outbound);

        let dir = PacketCollector::determine_direction(
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            &local_ips,
        );
        assert_eq!(dir, PacketDirection::Inbound);

        let dir = PacketCollector::determine_direction(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            &local_ips,
        );
        assert_eq!(dir, PacketDirection::Local);
    }
}