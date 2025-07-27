//! Bandwidth statistics and data structures
//!
//! This module contains the BandwidthStats struct and related data types
//! for representing bandwidth measurements and interface information.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Indicates the reliability of speed calculations
/// Used to inform users about the quality of bandwidth measurements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CalculationConfidence {
    /// High confidence: Normal operation with sufficient data and stable conditions
    High,
    /// Medium confidence: Recent interface changes, short time intervals, or minor anomalies
    Medium,
    /// Low confidence: Counter resets, time anomalies, or data validation issues detected
    Low,
    /// No confidence: First reading, insufficient data, or critical errors
    None,
}

/// Network interface types for better categorization and filtering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterfaceType {
    /// Physical Ethernet connection
    Ethernet,
    /// Wireless network interface
    WiFi,
    /// Loopback interface (localhost)
    Loopback,
    /// Virtual interface (VPN, container, etc.)
    Virtual,
    /// Unknown or unclassified interface type
    Unknown,
}

/// Network interface operational states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterfaceState {
    /// Interface is up and operational
    Up,
    /// Interface is down or disconnected
    Down,
    /// Interface state is unknown or transitioning
    Unknown,
}

/// Represents bandwidth statistics for a single network interface at a specific point in time
/// This struct contains both cumulative totals and calculated speeds with enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthStats {
    /// UTC timestamp when these statistics were collected
    pub timestamp: DateTime<Utc>,
    /// Name of the network interface (e.g., "eth0", "wlan0", "en0")
    pub interface_name: String,
    /// Type of network interface for better categorization
    pub interface_type: InterfaceType,
    /// Current operational state of the interface
    pub interface_state: InterfaceState,
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
    /// Confidence level of the speed calculations
    pub calculation_confidence: CalculationConfidence,
    /// Time elapsed since the last successful update for this interface (in seconds)
    pub time_since_last_update: f64,
}

/// Utility functions for bandwidth statistics
impl BandwidthStats {
    /// Formats speed values with appropriate units (B/s, KB/s, MB/s, GB/s)
    pub fn format_speed(speed_bps: f64) -> String {
        crate::collectors::bandwidth::formatting::format_speed(speed_bps)
    }

    /// Formats byte values with appropriate units (B, KB, MB, GB, TB)
    pub fn format_bytes(bytes: f64) -> String {
        crate::collectors::bandwidth::formatting::format_bytes(bytes)
    }

    /// Gets the total bandwidth (download + upload) in bytes per second
    pub fn total_bandwidth_bps(&self) -> f64 {
        self.download_speed_bps + self.upload_speed_bps
    }

    /// Gets the total bytes transferred (received + sent)
    pub fn total_bytes(&self) -> u64 {
        self.bytes_received + self.bytes_sent
    }

    /// Gets the total packets transferred (received + sent)
    pub fn total_packets(&self) -> u64 {
        self.packets_received + self.packets_sent
    }

    /// Calculates the average packet size for received data
    pub fn average_rx_packet_size(&self) -> Option<f64> {
        if self.packets_received > 0 {
            Some(self.bytes_received as f64 / self.packets_received as f64)
        } else {
            None
        }
    }

    /// Calculates the average packet size for sent data
    pub fn average_tx_packet_size(&self) -> Option<f64> {
        if self.packets_sent > 0 {
            Some(self.bytes_sent as f64 / self.packets_sent as f64)
        } else {
            None
        }
    }

    /// Checks if the interface has any activity (non-zero speeds)
    pub fn has_activity(&self) -> bool {
        self.download_speed_bps > 0.0 || self.upload_speed_bps > 0.0
    }

    /// Checks if the interface is considered active (has recent activity)
    pub fn is_active(&self) -> bool {
        self.interface_state == InterfaceState::Up && self.has_activity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Helper function to create a test BandwidthStats instance
    fn create_test_stats() -> BandwidthStats {
        BandwidthStats {
            timestamp: Utc::now(),
            interface_name: "eth0".to_string(),
            interface_type: InterfaceType::Ethernet,
            interface_state: InterfaceState::Up,
            bytes_received: 1048576,
            bytes_sent: 524288,
            packets_received: 1000,
            packets_sent: 500,
            download_speed_bps: 1024.0,
            upload_speed_bps: 512.0,
            calculation_confidence: CalculationConfidence::High,
            time_since_last_update: 1.0,
        }
    }

    #[test]
    fn test_bandwidth_stats_creation() {
        let stats = create_test_stats();

        assert_eq!(stats.interface_name, "eth0");
        assert_eq!(stats.interface_type, InterfaceType::Ethernet);
        assert_eq!(stats.interface_state, InterfaceState::Up);
        assert_eq!(stats.bytes_received, 1048576);
        assert_eq!(stats.bytes_sent, 524288);
        assert_eq!(stats.packets_received, 1000);
        assert_eq!(stats.packets_sent, 500);
        assert_eq!(stats.download_speed_bps, 1024.0);
        assert_eq!(stats.upload_speed_bps, 512.0);
        assert_eq!(stats.calculation_confidence, CalculationConfidence::High);
        assert_eq!(stats.time_since_last_update, 1.0);
    }

    #[test]
    fn test_bandwidth_stats_utility_methods() {
        let stats = create_test_stats();

        // Test total bandwidth calculation
        assert_eq!(stats.total_bandwidth_bps(), 1536.0); // 1024.0 + 512.0

        // Test total bytes calculation
        assert_eq!(stats.total_bytes(), 1572864); // 1048576 + 524288

        // Test total packets calculation
        assert_eq!(stats.total_packets(), 1500); // 1000 + 500

        // Test average packet sizes
        assert_eq!(stats.average_rx_packet_size(), Some(1048.576)); // 1048576 / 1000
        assert_eq!(stats.average_tx_packet_size(), Some(1048.576)); // 524288 / 500

        // Test activity detection
        assert!(stats.has_activity());
        assert!(stats.is_active());
    }

    #[test]
    fn test_bandwidth_stats_edge_cases() {
        let mut stats = create_test_stats();

        // Test zero packets case
        stats.packets_received = 0;
        stats.packets_sent = 0;
        assert_eq!(stats.average_rx_packet_size(), None);
        assert_eq!(stats.average_tx_packet_size(), None);

        // Test zero speed case
        stats.download_speed_bps = 0.0;
        stats.upload_speed_bps = 0.0;
        assert!(!stats.has_activity());
        assert!(!stats.is_active()); // Still up but no activity

        // Test down interface
        stats.interface_state = InterfaceState::Down;
        stats.download_speed_bps = 1024.0; // Even with speed, down interface is not active
        assert!(!stats.is_active());
    }

    #[test]
    fn test_bandwidth_stats_formatting_methods() {
        // Test static formatting methods
        assert_eq!(BandwidthStats::format_speed(1024.0), "1.00 KB/s");
        assert_eq!(BandwidthStats::format_bytes(1048576.0), "1.00 MB");

        // Test edge cases
        assert_eq!(BandwidthStats::format_speed(0.0), "0.00 B/s");
        assert_eq!(BandwidthStats::format_bytes(0.0), "0 B");
    }

    #[test]
    fn test_bandwidth_stats_serialization() {
        let stats = create_test_stats();

        // Test that stats can be serialized/deserialized
        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: BandwidthStats = serde_json::from_str(&serialized).unwrap();

        assert_eq!(stats.interface_name, deserialized.interface_name);
        assert_eq!(stats.interface_type, deserialized.interface_type);
        assert_eq!(stats.interface_state, deserialized.interface_state);
        assert_eq!(stats.bytes_received, deserialized.bytes_received);
        assert_eq!(stats.bytes_sent, deserialized.bytes_sent);
        assert_eq!(stats.packets_received, deserialized.packets_received);
        assert_eq!(stats.packets_sent, deserialized.packets_sent);
        assert_eq!(stats.download_speed_bps, deserialized.download_speed_bps);
        assert_eq!(stats.upload_speed_bps, deserialized.upload_speed_bps);
        assert_eq!(
            stats.calculation_confidence,
            deserialized.calculation_confidence
        );
        assert_eq!(
            stats.time_since_last_update,
            deserialized.time_since_last_update
        );
    }

    #[test]
    fn test_calculation_confidence_enum() {
        // Test all variants
        let high = CalculationConfidence::High;
        let medium = CalculationConfidence::Medium;
        let low = CalculationConfidence::Low;
        let none = CalculationConfidence::None;

        // Test equality
        assert_eq!(high, CalculationConfidence::High);
        assert_ne!(high, medium);

        // Test serialization/deserialization
        let serialized = serde_json::to_string(&high).unwrap();
        let deserialized: CalculationConfidence = serde_json::from_str(&serialized).unwrap();
        assert_eq!(high, deserialized);

        // Test all variants can be serialized
        assert!(serde_json::to_string(&medium).is_ok());
        assert!(serde_json::to_string(&low).is_ok());
        assert!(serde_json::to_string(&none).is_ok());
    }

    #[test]
    fn test_interface_type_enum() {
        // Test all variants
        let ethernet = InterfaceType::Ethernet;
        let wifi = InterfaceType::WiFi;
        let loopback = InterfaceType::Loopback;
        let virtual_if = InterfaceType::Virtual;
        let unknown = InterfaceType::Unknown;

        // Test equality
        assert_eq!(ethernet, InterfaceType::Ethernet);
        assert_ne!(ethernet, wifi);

        // Test serialization/deserialization
        let serialized = serde_json::to_string(&ethernet).unwrap();
        let deserialized: InterfaceType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(ethernet, deserialized);

        // Test all variants can be serialized
        assert!(serde_json::to_string(&wifi).is_ok());
        assert!(serde_json::to_string(&loopback).is_ok());
        assert!(serde_json::to_string(&virtual_if).is_ok());
        assert!(serde_json::to_string(&unknown).is_ok());
    }

    #[test]
    fn test_interface_state_enum() {
        // Test all variants
        let up = InterfaceState::Up;
        let down = InterfaceState::Down;
        let unknown = InterfaceState::Unknown;

        // Test equality
        assert_eq!(up, InterfaceState::Up);
        assert_ne!(up, down);

        // Test serialization/deserialization
        let serialized = serde_json::to_string(&up).unwrap();
        let deserialized: InterfaceState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(up, deserialized);

        // Test all variants can be serialized
        assert!(serde_json::to_string(&down).is_ok());
        assert!(serde_json::to_string(&unknown).is_ok());
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that the data structures maintain backward compatibility
        // by ensuring they can be created with all expected fields
        let stats = BandwidthStats {
            timestamp: Utc::now(),
            interface_name: "test".to_string(),
            interface_type: InterfaceType::Unknown,
            interface_state: InterfaceState::Unknown,
            bytes_received: 0,
            bytes_sent: 0,
            packets_received: 0,
            packets_sent: 0,
            download_speed_bps: 0.0,
            upload_speed_bps: 0.0,
            calculation_confidence: CalculationConfidence::None,
            time_since_last_update: 0.0,
        };

        // Verify all fields are accessible
        assert_eq!(stats.interface_name, "test");
        assert_eq!(stats.interface_type, InterfaceType::Unknown);
        assert_eq!(stats.interface_state, InterfaceState::Unknown);
        assert_eq!(stats.calculation_confidence, CalculationConfidence::None);

        // Verify utility methods work
        assert_eq!(stats.total_bandwidth_bps(), 0.0);
        assert_eq!(stats.total_bytes(), 0);
        assert_eq!(stats.total_packets(), 0);
        assert!(!stats.has_activity());
        assert!(!stats.is_active());
    }

    #[test]
    fn test_enum_variants_completeness() {
        // Ensure all enum variants are tested and accessible

        // CalculationConfidence variants
        let _high = CalculationConfidence::High;
        let _medium = CalculationConfidence::Medium;
        let _low = CalculationConfidence::Low;
        let _none = CalculationConfidence::None;

        // InterfaceType variants
        let _ethernet = InterfaceType::Ethernet;
        let _wifi = InterfaceType::WiFi;
        let _loopback = InterfaceType::Loopback;
        let _virtual = InterfaceType::Virtual;
        let _unknown = InterfaceType::Unknown;

        // InterfaceState variants
        let _up = InterfaceState::Up;
        let _down = InterfaceState::Down;
        let _unknown_state = InterfaceState::Unknown;

        // If this test compiles, all variants are accessible
        assert!(true);
    }
}
