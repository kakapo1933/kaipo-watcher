//! Tests for validation logic
//!
//! This module contains unit tests for data validation functions
//! including interface data validation and speed calculation validation.

#[cfg(test)]
mod tests {
    use crate::collectors::bandwidth::stats::CalculationConfidence;
    use crate::collectors::bandwidth::validation::*;
    use chrono::{Duration as ChronoDuration, Utc};
    use std::collections::HashMap;

    // Helper function to create test previous stats
    fn create_test_previous_stats(
        interface_name: &str,
        rx_bytes: u64,
        tx_bytes: u64,
        timestamp: chrono::DateTime<chrono::Utc>,
        failures: u32,
    ) -> HashMap<String, (u64, u64, chrono::DateTime<chrono::Utc>, u32)> {
        let mut stats = HashMap::new();
        stats.insert(
            interface_name.to_string(),
            (rx_bytes, tx_bytes, timestamp, failures),
        );
        stats
    }

    #[test]
    fn test_validate_interface_data_valid_cases() {
        // Test case 1: Normal valid data
        let result = validate_interface_data("eth0", 1000, 500, 10, 5, 1);
        assert!(result.is_ok(), "Valid data should pass validation");

        // Test case 2: Zero bytes and packets (interface with no traffic)
        let result = validate_interface_data("eth0", 0, 0, 0, 0, 1);
        assert!(result.is_ok(), "Zero traffic should be valid");

        // Test case 3: Only received traffic
        let result = validate_interface_data("eth0", 1000, 0, 10, 0, 1);
        assert!(result.is_ok(), "Only RX traffic should be valid");

        // Test case 4: Only sent traffic
        let result = validate_interface_data("eth0", 0, 1000, 0, 10, 1);
        assert!(result.is_ok(), "Only TX traffic should be valid");

        // Test case 5: Large but reasonable packet sizes (1500 bytes average)
        let result = validate_interface_data("eth0", 15000, 15000, 10, 10, 1);
        assert!(result.is_ok(), "Reasonable packet sizes should be valid");

        // Test case 6: Maximum valid interface name length
        let long_name = "a".repeat(64);
        let result = validate_interface_data(&long_name, 1000, 500, 10, 5, 1);
        assert!(
            result.is_ok(),
            "64-character interface name should be valid"
        );
    }

    #[test]
    fn test_validate_interface_data_invalid_packet_byte_ratios() {
        // Test case 1: Packets received but no bytes
        let result = validate_interface_data("eth0", 0, 500, 10, 5, 1);
        assert!(result.is_err(), "Packets without bytes should fail");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("packets received but 0 bytes"),
            "Error should mention packets without bytes: {}",
            error_msg
        );

        // Test case 2: Packets sent but no bytes
        let result = validate_interface_data("eth0", 1000, 0, 10, 5, 1);
        assert!(result.is_err(), "Packets without bytes should fail");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("packets sent but 0 bytes"),
            "Error should mention packets without bytes: {}",
            error_msg
        );

        // Test case 3: Both RX and TX packets without bytes
        let result = validate_interface_data("eth0", 0, 0, 10, 5, 1);
        assert!(result.is_err(), "Packets without bytes should fail");
    }

    #[test]
    fn test_validate_interface_data_unreasonable_packet_sizes() {
        // Test case 1: Unreasonably large RX packet sizes (100KB per packet)
        let result = validate_interface_data("eth0", 1000000, 500, 10, 5, 1);
        assert!(result.is_err(), "Large RX packet sizes should fail");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Average RX packet size too large"),
            "Error should mention large RX packet size: {}",
            error_msg
        );

        // Test case 2: Unreasonably large TX packet sizes (200KB per packet)
        let result = validate_interface_data("eth0", 1000, 1000000, 10, 5, 1);
        assert!(result.is_err(), "Large TX packet sizes should fail");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Average TX packet size too large"),
            "Error should mention large TX packet size: {}",
            error_msg
        );

        // Test case 3: Both RX and TX with large packet sizes
        let result = validate_interface_data("eth0", 1000000, 1000000, 10, 10, 1);
        assert!(result.is_err(), "Large packet sizes should fail");

        // Test case 4: Exactly at the 64KB threshold should pass
        let result = validate_interface_data("eth0", 65536, 65536, 1, 1, 1);
        assert!(result.is_ok(), "Exactly 64KB packets should be valid");

        // Test case 5: Just over the 64KB threshold should fail
        let result = validate_interface_data("eth0", 65537, 65536, 1, 1, 1);
        assert!(result.is_err(), "Just over 64KB packets should fail");
    }

    #[test]
    fn test_validate_interface_data_invalid_interface_names() {
        // Test case 1: Empty interface name
        let result = validate_interface_data("", 1000, 500, 10, 5, 1);
        assert!(result.is_err(), "Empty interface name should fail");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Invalid interface name: empty string"),
            "Error should mention empty interface name: {}",
            error_msg
        );

        // Test case 2: Interface name too long (65 characters)
        let long_name = "a".repeat(65);
        let result = validate_interface_data(&long_name, 1000, 500, 10, 5, 1);
        assert!(result.is_err(), "Too long interface name should fail");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("too long"),
            "Error should mention interface name too long: {}",
            error_msg
        );

        // Test case 3: Interface name with special characters (should be valid)
        let result = validate_interface_data("eth0:1", 1000, 500, 10, 5, 1);
        assert!(
            result.is_ok(),
            "Interface names with special characters should be valid"
        );

        // Test case 4: Interface name with spaces (should be valid)
        let result = validate_interface_data("Local Area Connection", 1000, 500, 10, 5, 1);
        assert!(
            result.is_ok(),
            "Interface names with spaces should be valid"
        );
    }

    #[test]
    fn test_calculate_speeds_with_validation_no_previous_data() {
        let previous_stats = HashMap::new();
        let now = Utc::now();

        let (download_speed, upload_speed, confidence) =
            calculate_speeds_with_validation("eth0", 1000, 500, now, &previous_stats, 0.1, 1);

        assert_eq!(
            download_speed, 0.0,
            "Download speed should be 0 with no previous data"
        );
        assert_eq!(
            upload_speed, 0.0,
            "Upload speed should be 0 with no previous data"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::None,
            "Confidence should be None with no previous data"
        );
    }

    #[test]
    fn test_calculate_speeds_with_validation_normal_operation() {
        let base_time = Utc::now();
        let current_time = base_time + ChronoDuration::seconds(1);
        let previous_stats = create_test_previous_stats("eth0", 1000, 500, base_time, 0);

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            2000,
            1000,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(download_speed, 1000.0, "Download speed should be 1000 B/s");
        assert_eq!(upload_speed, 500.0, "Upload speed should be 500 B/s");
        assert_eq!(
            confidence,
            CalculationConfidence::High,
            "Confidence should be High for normal operation"
        );
    }

    #[test]
    fn test_calculate_speeds_with_validation_different_time_intervals() {
        let base_time = Utc::now();

        // Test case 1: 2-second interval (optimal)
        let current_time = base_time + ChronoDuration::seconds(2);
        let previous_stats = create_test_previous_stats("eth0", 1000, 500, base_time, 0);

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            3000,
            1500,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(
            download_speed, 1000.0,
            "Download speed should be 1000 B/s over 2 seconds"
        );
        assert_eq!(
            upload_speed, 500.0,
            "Upload speed should be 500 B/s over 2 seconds"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::High,
            "Confidence should be High for 2-second interval"
        );

        // Test case 2: 5-second interval (still optimal)
        let current_time = base_time + ChronoDuration::seconds(5);
        let previous_stats = create_test_previous_stats("eth0", 1000, 500, base_time, 0);

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            6000,
            3000,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(
            download_speed, 1000.0,
            "Download speed should be 1000 B/s over 5 seconds"
        );
        assert_eq!(
            upload_speed, 500.0,
            "Upload speed should be 500 B/s over 5 seconds"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::High,
            "Confidence should be High for 5-second interval"
        );

        // Test case 3: 0.7-second interval (medium confidence)
        let current_time = base_time + ChronoDuration::milliseconds(700);
        let previous_stats = create_test_previous_stats("eth0", 1000, 500, base_time, 0);

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            1700,
            850,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert!(
            (download_speed - 1000.0).abs() < 0.01,
            "Download speed should be approximately 1000 B/s over 0.7 seconds, got {}",
            download_speed
        );
        assert!(
            (upload_speed - 500.0).abs() < 0.01,
            "Upload speed should be approximately 500 B/s over 0.7 seconds, got {}",
            upload_speed
        );
        assert_eq!(
            confidence,
            CalculationConfidence::Medium,
            "Confidence should be Medium for 0.7-second interval"
        );
    }

    #[test]
    fn test_calculate_speeds_with_validation_time_anomalies() {
        let base_time = Utc::now();
        let previous_stats = create_test_previous_stats("eth0", 1000, 500, base_time, 0);

        // Test case 1: Time went backwards (negative interval)
        let current_time = base_time - ChronoDuration::seconds(1);

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            2000,
            1000,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(
            download_speed, 0.0,
            "Download speed should be 0 for negative time interval"
        );
        assert_eq!(
            upload_speed, 0.0,
            "Upload speed should be 0 for negative time interval"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::None,
            "Confidence should be None for negative time interval"
        );

        // Test case 2: Same timestamp (zero interval)
        let current_time = base_time;

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            2000,
            1000,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(
            download_speed, 0.0,
            "Download speed should be 0 for zero time interval"
        );
        assert_eq!(
            upload_speed, 0.0,
            "Upload speed should be 0 for zero time interval"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::None,
            "Confidence should be None for zero time interval"
        );
    }

    #[test]
    fn test_calculate_speeds_with_validation_minimum_time_threshold() {
        let base_time = Utc::now();
        let previous_stats = create_test_previous_stats("eth0", 1000, 500, base_time, 0);

        // Test case 1: Below minimum threshold (50ms < 100ms)
        let current_time = base_time + ChronoDuration::milliseconds(50);

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            1100,
            550,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(
            download_speed, 0.0,
            "Download speed should be 0 for interval below threshold"
        );
        assert_eq!(
            upload_speed, 0.0,
            "Upload speed should be 0 for interval below threshold"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::Low,
            "Confidence should be Low for interval below threshold"
        );

        // Test case 2: Exactly at minimum threshold (100ms)
        let current_time = base_time + ChronoDuration::milliseconds(100);

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            1100,
            550,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(
            download_speed, 1000.0,
            "Download speed should be calculated at threshold"
        );
        assert_eq!(
            upload_speed, 500.0,
            "Upload speed should be calculated at threshold"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::Medium,
            "Confidence should be Medium at threshold"
        );

        // Test case 3: Above minimum threshold (200ms > 100ms)
        let current_time = base_time + ChronoDuration::milliseconds(200);

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            1200,
            600,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(
            download_speed, 1000.0,
            "Download speed should be calculated above threshold"
        );
        assert_eq!(
            upload_speed, 500.0,
            "Upload speed should be calculated above threshold"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::Medium,
            "Confidence should be Medium above threshold"
        );
    }

    #[test]
    fn test_calculate_speeds_with_validation_counter_resets() {
        let base_time = Utc::now();
        let current_time = base_time + ChronoDuration::seconds(1);

        // Test case 1: RX counter reset (current < previous)
        let previous_stats = create_test_previous_stats("eth0", 5000, 2000, base_time, 0);

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            1000,
            3000,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(
            download_speed, 0.0,
            "Download speed should be 0 for RX counter reset"
        );
        assert_eq!(
            upload_speed, 0.0,
            "Upload speed should be 0 for RX counter reset"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::Low,
            "Confidence should be Low for counter reset"
        );

        // Test case 2: TX counter reset (current < previous)
        let previous_stats = create_test_previous_stats("eth0", 1000, 5000, base_time, 0);

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            2000,
            1000,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(
            download_speed, 0.0,
            "Download speed should be 0 for TX counter reset"
        );
        assert_eq!(
            upload_speed, 0.0,
            "Upload speed should be 0 for TX counter reset"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::Low,
            "Confidence should be Low for counter reset"
        );

        // Test case 3: Both counters reset
        let previous_stats = create_test_previous_stats("eth0", 5000, 3000, base_time, 0);

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            1000,
            500,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(
            download_speed, 0.0,
            "Download speed should be 0 for both counter reset"
        );
        assert_eq!(
            upload_speed, 0.0,
            "Upload speed should be 0 for both counter reset"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::Low,
            "Confidence should be Low for counter reset"
        );

        // Test case 4: No counter reset (normal operation)
        let previous_stats = create_test_previous_stats("eth0", 1000, 500, base_time, 0);

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            2000,
            1000,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(
            download_speed, 1000.0,
            "Download speed should be calculated normally"
        );
        assert_eq!(
            upload_speed, 500.0,
            "Upload speed should be calculated normally"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::High,
            "Confidence should be High for normal operation"
        );
    }

    #[test]
    fn test_detect_counter_reset() {
        // Test case 1: Normal operation - no reset
        assert!(
            !detect_counter_reset(2000, 1000, 1000, 500),
            "Normal counter progression should not be detected as reset"
        );

        // Test case 2: RX counter reset
        assert!(
            detect_counter_reset(500, 1000, 1000, 500),
            "RX counter decrease should be detected as reset"
        );

        // Test case 3: TX counter reset
        assert!(
            detect_counter_reset(2000, 200, 1000, 500),
            "TX counter decrease should be detected as reset"
        );

        // Test case 4: Both counters reset
        assert!(
            detect_counter_reset(500, 200, 1000, 500),
            "Both counter decreases should be detected as reset"
        );

        // Test case 5: Counters unchanged (edge case)
        assert!(
            !detect_counter_reset(1000, 500, 1000, 500),
            "Unchanged counters should not be detected as reset"
        );

        // Test case 6: Large counter values (test for overflow handling)
        assert!(
            !detect_counter_reset(u64::MAX, u64::MAX - 1000, u64::MAX - 1000, u64::MAX - 2000),
            "Large counter progression should not be detected as reset"
        );

        // Test case 7: Counter reset from maximum value (wraparound)
        assert!(
            detect_counter_reset(1000, 500, u64::MAX, u64::MAX - 1000),
            "Counter wraparound should be detected as reset"
        );
    }

    #[test]
    fn test_detect_time_anomaly() {
        let base_time = Utc::now();
        let min_threshold_ms = 100;

        // Test case 1: Normal time progression
        let current_time = base_time + ChronoDuration::seconds(1);
        let result = detect_time_anomaly(current_time, base_time, min_threshold_ms);
        assert!(result.is_ok(), "Normal time progression should be valid");
        assert_eq!(result.unwrap(), 1000, "1 second should equal 1000ms");

        // Test case 2: Negative time interval (clock went backwards)
        let current_time = base_time - ChronoDuration::seconds(1);
        let result = detect_time_anomaly(current_time, base_time, min_threshold_ms);
        assert!(result.is_err(), "Negative time interval should be detected");
        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("Negative time interval"),
            "Error should mention negative time interval: {}",
            error_msg
        );

        // Test case 3: Zero time interval (identical timestamps)
        let result = detect_time_anomaly(base_time, base_time, min_threshold_ms);
        assert!(result.is_err(), "Zero time interval should be detected");
        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("Zero time interval"),
            "Error should mention zero time interval: {}",
            error_msg
        );

        // Test case 4: Time interval too small
        let current_time = base_time + ChronoDuration::milliseconds(50);
        let result = detect_time_anomaly(current_time, base_time, min_threshold_ms);
        assert!(result.is_err(), "Small time interval should be detected");
        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("Time interval too small"),
            "Error should mention small time interval: {}",
            error_msg
        );

        // Test case 5: Time interval exactly at threshold
        let current_time = base_time + ChronoDuration::milliseconds(min_threshold_ms);
        let result = detect_time_anomaly(current_time, base_time, min_threshold_ms);
        assert!(result.is_ok(), "Time interval at threshold should be valid");
        assert_eq!(
            result.unwrap(),
            min_threshold_ms,
            "Should return exact threshold value"
        );

        // Test case 6: Suspiciously large time interval (possible suspend/resume)
        let current_time = base_time + ChronoDuration::minutes(10);
        let result = detect_time_anomaly(current_time, base_time, min_threshold_ms);
        assert!(result.is_err(), "Large time interval should be detected");
        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("Suspiciously large time interval"),
            "Error should mention large time interval: {}",
            error_msg
        );

        // Test case 7: Time interval just under the large threshold (should be valid)
        let current_time = base_time + ChronoDuration::minutes(4);
        let result = detect_time_anomaly(current_time, base_time, min_threshold_ms);
        assert!(result.is_ok(), "4-minute interval should be valid");
        assert_eq!(result.unwrap(), 240000, "4 minutes should equal 240000ms");
    }

    #[test]
    fn test_validate_packet_byte_consistency() {
        // Test case 1: Valid consistency - normal packet sizes
        let result = validate_packet_byte_consistency(1000, 10, "received");
        assert!(result.is_ok(), "Normal packet sizes should be valid");

        // Test case 2: Valid consistency - large packets
        let result = validate_packet_byte_consistency(65536, 1, "sent");
        assert!(
            result.is_ok(),
            "Large but valid packet sizes should be valid"
        );

        // Test case 3: Packets without bytes
        let result = validate_packet_byte_consistency(0, 10, "received");
        assert!(result.is_err(), "Packets without bytes should fail");
        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("packets received but 0 bytes"),
            "Error should mention packets without bytes: {}",
            error_msg
        );

        // Test case 4: Suspiciously small packets (10 bytes per packet)
        let result = validate_packet_byte_consistency(100, 10, "sent");
        assert!(result.is_err(), "Small packets should fail");
        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("Suspiciously small average packet size"),
            "Error should mention small packet size: {}",
            error_msg
        );

        // Test case 5: Unreasonably large packets (100KB per packet)
        let result = validate_packet_byte_consistency(1000000, 10, "received");
        assert!(result.is_err(), "Large packets should fail");
        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("Average packet size too large"),
            "Error should mention large packet size: {}",
            error_msg
        );

        // Test case 6: Edge case - exactly 20 bytes per packet (minimum valid)
        let result = validate_packet_byte_consistency(200, 10, "sent");
        assert!(result.is_ok(), "20-byte packets should be valid");

        // Test case 7: Edge case - just under 20 bytes per packet
        let result = validate_packet_byte_consistency(199, 10, "received");
        assert!(result.is_err(), "Sub-20-byte packets should fail");

        // Test case 8: Edge case - exactly 64KB per packet (maximum valid)
        let result = validate_packet_byte_consistency(65536, 1, "sent");
        assert!(result.is_ok(), "64KB packets should be valid");

        // Test case 9: Edge case - just over 64KB per packet
        let result = validate_packet_byte_consistency(65537, 1, "received");
        assert!(result.is_err(), "Over-64KB packets should fail");

        // Test case 10: Zero bytes and packets (valid edge case)
        let result = validate_packet_byte_consistency(0, 0, "sent");
        assert!(result.is_ok(), "Zero bytes and packets should be valid");
    }

    #[test]
    fn test_assess_calculation_confidence() {
        // Test case 1: High confidence - optimal conditions
        let confidence = assess_calculation_confidence(2.0, false, true, 0);
        assert_eq!(
            confidence,
            CalculationConfidence::High,
            "Optimal conditions should result in High confidence"
        );

        // Test case 2: High confidence - good time interval
        let confidence = assess_calculation_confidence(1.5, false, true, 0);
        assert_eq!(
            confidence,
            CalculationConfidence::High,
            "Good time interval should result in High confidence"
        );

        // Test case 3: Medium confidence - acceptable time interval
        let confidence = assess_calculation_confidence(0.7, false, true, 0);
        assert_eq!(
            confidence,
            CalculationConfidence::Medium,
            "Acceptable time interval should result in Medium confidence"
        );

        // Test case 4: Low confidence - short time interval
        let confidence = assess_calculation_confidence(0.3, false, true, 0);
        assert_eq!(
            confidence,
            CalculationConfidence::Low,
            "Short time interval should result in Low confidence"
        );

        // Test case 5: Low confidence - counter reset detected
        let confidence = assess_calculation_confidence(2.0, true, true, 0);
        assert_eq!(
            confidence,
            CalculationConfidence::Low,
            "Counter reset should result in Low confidence"
        );

        // Test case 6: None confidence - data validation failed
        let confidence = assess_calculation_confidence(2.0, false, false, 0);
        assert_eq!(
            confidence,
            CalculationConfidence::None,
            "Failed data validation should result in None confidence"
        );

        // Test case 7: None confidence - too many consecutive failures
        let confidence = assess_calculation_confidence(2.0, false, true, 5);
        assert_eq!(
            confidence,
            CalculationConfidence::None,
            "Many consecutive failures should result in None confidence"
        );

        // Test case 8: Low confidence - some consecutive failures
        let confidence = assess_calculation_confidence(2.0, false, true, 2);
        assert_eq!(
            confidence,
            CalculationConfidence::Low,
            "Some consecutive failures should result in Low confidence"
        );

        // Test case 9: Edge case - exactly 3 consecutive failures
        let confidence = assess_calculation_confidence(2.0, false, true, 3);
        assert_eq!(
            confidence,
            CalculationConfidence::Low,
            "Exactly 3 consecutive failures should result in Low confidence"
        );

        // Test case 10: Edge case - exactly 4 consecutive failures
        let confidence = assess_calculation_confidence(2.0, false, true, 4);
        assert_eq!(
            confidence,
            CalculationConfidence::None,
            "Exactly 4 consecutive failures should result in None confidence"
        );

        // Test case 11: Multiple issues - counter reset and validation failure
        let confidence = assess_calculation_confidence(2.0, true, false, 0);
        assert_eq!(
            confidence,
            CalculationConfidence::None,
            "Multiple issues should result in None confidence"
        );

        // Test case 12: Edge case - exactly 1 second interval
        let confidence = assess_calculation_confidence(1.0, false, true, 0);
        assert_eq!(
            confidence,
            CalculationConfidence::High,
            "1-second interval should result in High confidence"
        );

        // Test case 13: Edge case - exactly 0.5 second interval
        let confidence = assess_calculation_confidence(0.5, false, true, 0);
        assert_eq!(
            confidence,
            CalculationConfidence::Medium,
            "0.5-second interval should result in Medium confidence"
        );
    }

    #[test]
    fn test_validation_edge_cases_and_boundary_conditions() {
        // Test case 1: Values that create unreasonable packet sizes
        let result = validate_interface_data("eth0", u64::MAX, u64::MAX, 1, 1, 1);
        // This should fail due to unreasonable packet sizes (u64::MAX bytes per packet)
        assert!(
            result.is_err(),
            "Maximum bytes with 1 packet should fail due to packet size validation"
        );

        // Test case 2: Single packet with maximum reasonable size
        let result = validate_interface_data("eth0", 65536, 65536, 1, 1, 1);
        assert!(result.is_ok(), "Single 64KB packet should be valid");

        // Test case 3: Many small packets
        let result = validate_interface_data("eth0", 20000, 20000, 1000, 1000, 1);
        assert!(result.is_ok(), "Many small packets should be valid");

        // Test case 4: Interface name with Unicode characters
        let result = validate_interface_data("eth0-测试", 1000, 500, 10, 5, 1);
        assert!(result.is_ok(), "Unicode interface names should be valid");

        // Test case 5: Very long but valid interface name
        let name_63_chars = "a".repeat(63);
        let result = validate_interface_data(&name_63_chars, 1000, 500, 10, 5, 1);
        assert!(
            result.is_ok(),
            "63-character interface name should be valid"
        );
    }

    #[test]
    fn test_validation_with_real_world_scenarios() {
        // Test case 1: Ethernet interface with typical traffic
        let result = validate_interface_data("eth0", 1048576, 524288, 1000, 500, 1);
        assert!(result.is_ok(), "Typical Ethernet traffic should be valid");

        // Test case 2: WiFi interface with variable packet sizes
        let result = validate_interface_data("wlan0", 2097152, 1048576, 1500, 800, 1);
        assert!(result.is_ok(), "Typical WiFi traffic should be valid");

        // Test case 3: Loopback interface with large packets
        let result = validate_interface_data("lo", 65536000, 65536000, 1000, 1000, 1);
        assert!(
            result.is_ok(),
            "Loopback traffic with large packets should be valid"
        );

        // Test case 4: VPN interface with encrypted overhead
        let result = validate_interface_data("tun0", 1100000, 1000000, 1000, 900, 1);
        assert!(result.is_ok(), "VPN traffic with overhead should be valid");

        // Test case 5: Mobile interface with variable conditions
        let result = validate_interface_data("ppp0", 500000, 100000, 800, 200, 1);
        assert!(result.is_ok(), "Mobile interface traffic should be valid");
    }

    #[test]
    fn test_speed_calculation_with_various_data_patterns() {
        let base_time = Utc::now();
        let current_time = base_time + ChronoDuration::seconds(1);

        // Test case 1: High-speed interface (1 Gbps simulation)
        let previous_stats = create_test_previous_stats("eth0", 0, 0, base_time, 0);
        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            125000000,
            125000000,
            current_time,
            &previous_stats,
            0.1,
            1,
        );
        assert_eq!(
            download_speed, 125000000.0,
            "High-speed download should be calculated correctly"
        );
        assert_eq!(
            upload_speed, 125000000.0,
            "High-speed upload should be calculated correctly"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::High,
            "High-speed calculation should have high confidence"
        );

        // Test case 2: Low-speed interface (56k modem simulation)
        let previous_stats = create_test_previous_stats("ppp0", 0, 0, base_time, 0);
        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "ppp0",
            7000,
            3500,
            current_time,
            &previous_stats,
            0.1,
            1,
        );
        assert_eq!(
            download_speed, 7000.0,
            "Low-speed download should be calculated correctly"
        );
        assert_eq!(
            upload_speed, 3500.0,
            "Low-speed upload should be calculated correctly"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::High,
            "Low-speed calculation should have high confidence"
        );

        // Test case 3: Asymmetric interface (typical broadband)
        let previous_stats = create_test_previous_stats("eth0", 1000000, 100000, base_time, 0);
        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            13000000,
            1100000,
            current_time,
            &previous_stats,
            0.1,
            1,
        );
        assert_eq!(
            download_speed, 12000000.0,
            "Asymmetric download should be calculated correctly"
        );
        assert_eq!(
            upload_speed, 1000000.0,
            "Asymmetric upload should be calculated correctly"
        );
        assert_eq!(
            confidence,
            CalculationConfidence::High,
            "Asymmetric calculation should have high confidence"
        );
    }
}
