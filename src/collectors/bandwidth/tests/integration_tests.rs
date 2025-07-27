//! Integration tests for bandwidth collection
//!
//! This module contains integration tests that verify cross-module
//! functionality and end-to-end behavior.

#[cfg(test)]
mod tests {
    use crate::collectors::bandwidth::errors::BandwidthError;
    use crate::collectors::bandwidth::formatting::{format_bytes, format_speed};
    use crate::collectors::bandwidth::stats::{
        BandwidthStats, CalculationConfidence, InterfaceState, InterfaceType,
    };
    use crate::collectors::bandwidth::validation::validate_interface_data;
    use chrono::Utc;

    #[test]
    fn test_interface_data_validation() {
        // Test case 1: Valid data should pass
        let result = validate_interface_data("eth0", 1000, 500, 10, 5, 1);
        assert!(result.is_ok());

        // Test case 2: Packets without bytes (invalid)
        let result = validate_interface_data("eth0", 0, 500, 10, 5, 1);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("packets received but 0 bytes")
        );

        let result = validate_interface_data("eth0", 1000, 0, 10, 5, 1);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("packets sent but 0 bytes")
        );

        // Test case 3: Unreasonably large packet sizes
        let result = validate_interface_data("eth0", 1000000, 500, 10, 5, 1); // 100KB per packet
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Average RX packet size too large")
        );

        let result = validate_interface_data("eth0", 1000, 1000000, 10, 5, 1); // 200KB per packet
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Average TX packet size too large")
        );

        // Test case 4: Empty interface name
        let result = validate_interface_data("", 1000, 500, 10, 5, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty string"));

        // Test case 5: Interface name too long
        let long_name = "a".repeat(100);
        let result = validate_interface_data(&long_name, 1000, 500, 10, 5, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));

        // Test case 6: Zero packets with bytes (valid - could be large packets)
        let result = validate_interface_data("eth0", 1000, 500, 0, 0, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_handling_and_recovery() {
        // Test BandwidthError creation and properties
        let refresh_error = BandwidthError::RefreshFailed {
            message: "Test error".to_string(),
            retry_attempts: 3,
        };
        assert!(refresh_error.to_string().contains("Test error"));
        // The error format doesn't include retry_attempts in the display, so we'll test the struct fields directly
        if let BandwidthError::RefreshFailed { retry_attempts, .. } = &refresh_error {
            assert_eq!(*retry_attempts, 3);
        }

        let counter_reset_error = BandwidthError::CounterReset {
            interface: "eth0".to_string(),
            current: 100,
            previous: 200,
        };
        assert!(counter_reset_error.to_string().contains("eth0"));
        assert!(counter_reset_error.to_string().contains("100"));
        assert!(counter_reset_error.to_string().contains("200"));

        let time_interval_error = BandwidthError::InvalidTimeInterval {
            interval_ms: 50,
            min_threshold_ms: 100,
        };
        assert!(time_interval_error.to_string().contains("50"));
        assert!(time_interval_error.to_string().contains("100"));
    }

    #[test]
    fn test_speed_formatting() {
        // Test format_speed function
        assert_eq!(format_speed(0.0), "0.00 B/s");
        assert_eq!(format_speed(512.0), "512.00 B/s");
        assert_eq!(format_speed(1024.0), "1.00 KB/s");
        assert_eq!(format_speed(1536.0), "1.50 KB/s");
        assert_eq!(format_speed(1048576.0), "1.00 MB/s");
        assert_eq!(format_speed(1073741824.0), "1.00 GB/s");

        // Test format_bytes function
        assert_eq!(format_bytes(0.0), "0 B");
        assert_eq!(format_bytes(512.0), "512 B");
        assert_eq!(format_bytes(1024.0), "1.00 KB");
        assert_eq!(format_bytes(1536.0), "1.50 KB");
        assert_eq!(format_bytes(1048576.0), "1.00 MB");
        assert_eq!(format_bytes(1073741824.0), "1.00 GB");
        assert_eq!(format_bytes(1099511627776.0), "1.00 TB");
    }

    #[test]
    fn test_bandwidth_stats_utility_methods() {
        let stats = BandwidthStats {
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
        };

        // Test formatting methods
        assert_eq!(BandwidthStats::format_speed(1024.0), "1.00 KB/s");
        assert_eq!(BandwidthStats::format_bytes(1048576.0), "1.00 MB");

        // Test that stats can be serialized/deserialized
        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: BandwidthStats = serde_json::from_str(&serialized).unwrap();
        assert_eq!(stats.interface_name, deserialized.interface_name);
        assert_eq!(stats.bytes_received, deserialized.bytes_received);
    }

    #[test]
    fn test_interface_type_and_state_enums() {
        // Test InterfaceType serialization
        let ethernet = InterfaceType::Ethernet;
        let serialized = serde_json::to_string(&ethernet).unwrap();
        let deserialized: InterfaceType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(ethernet, deserialized);

        // Test InterfaceState serialization
        let up_state = InterfaceState::Up;
        let serialized = serde_json::to_string(&up_state).unwrap();
        let deserialized: InterfaceState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(up_state, deserialized);

        // Test CalculationConfidence serialization
        let high_confidence = CalculationConfidence::High;
        let serialized = serde_json::to_string(&high_confidence).unwrap();
        let deserialized: CalculationConfidence = serde_json::from_str(&serialized).unwrap();
        assert_eq!(high_confidence, deserialized);
    }
}
