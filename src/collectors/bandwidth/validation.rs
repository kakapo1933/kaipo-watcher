//! Data validation for bandwidth collection
//!
//! This module contains validation logic for interface readings, speed calculations,
//! counter reset detection, and time anomaly detection.

use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{debug, trace, warn};
use std::collections::HashMap;

use crate::collectors::bandwidth::stats::CalculationConfidence;

/// Validates interface data before processing to catch common data integrity issues
///
/// This function performs comprehensive validation of network interface data to ensure
/// data integrity and catch common issues that could lead to incorrect calculations.
///
/// # Arguments
///
/// * `interface_name` - Name of the network interface being validated
/// * `bytes_received` - Total bytes received by the interface
/// * `bytes_sent` - Total bytes sent by the interface  
/// * `packets_received` - Total packets received by the interface
/// * `packets_sent` - Total packets sent by the interface
/// * `collection_count` - Current collection number for logging purposes
///
/// # Returns
///
/// * `Ok(())` if validation passes
/// * `Err(anyhow::Error)` if validation fails with detailed error message
///
/// # Validation Rules
///
/// 1. **Packet-to-byte consistency**: If packets exist, bytes must also exist
/// 2. **Reasonable packet sizes**: Average packet size should not exceed 64KB
/// 3. **Interface name validity**: Name must be non-empty and reasonable length
pub fn validate_interface_data(
    interface_name: &str,
    bytes_received: u64,
    bytes_sent: u64,
    packets_received: u64,
    packets_sent: u64,
    collection_count: u64,
) -> Result<()> {
    // Check for obviously invalid data patterns

    // Validation 1: Check for impossible packet-to-byte ratios
    if packets_received > 0 && bytes_received == 0 {
        return Err(anyhow::anyhow!(
            "Invalid data: {} packets received but 0 bytes",
            packets_received
        ));
    }

    if packets_sent > 0 && bytes_sent == 0 {
        return Err(anyhow::anyhow!(
            "Invalid data: {} packets sent but 0 bytes",
            packets_sent
        ));
    }

    // Validation 2: Check for unreasonably large packet sizes (over 64KB average)
    if bytes_received > 0 && packets_received > 0 {
        let avg_rx_packet_size = bytes_received as f64 / packets_received as f64;
        if avg_rx_packet_size > 65536.0 {
            return Err(anyhow::anyhow!(
                "Invalid data: Average RX packet size too large: {:.2} bytes",
                avg_rx_packet_size
            ));
        }
    }

    if bytes_sent > 0 && packets_sent > 0 {
        let avg_tx_packet_size = bytes_sent as f64 / packets_sent as f64;
        if avg_tx_packet_size > 65536.0 {
            return Err(anyhow::anyhow!(
                "Invalid data: Average TX packet size too large: {:.2} bytes",
                avg_tx_packet_size
            ));
        }
    }

    // Validation 3: Check for interface name validity
    if interface_name.is_empty() {
        return Err(anyhow::anyhow!("Invalid interface name: empty string"));
    }

    if interface_name.len() > 64 {
        return Err(anyhow::anyhow!(
            "Invalid interface name: too long ({} chars)",
            interface_name.len()
        ));
    }

    trace!(
        "Interface '{}' data validation passed for collection #{}: rx_bytes={}, tx_bytes={}, rx_packets={}, tx_packets={}",
        interface_name,
        collection_count,
        bytes_received,
        bytes_sent,
        packets_received,
        packets_sent
    );

    Ok(())
}

/// Calculates speeds with enhanced validation and error handling
///
/// This function performs speed calculations with comprehensive validation including
/// time anomaly detection, counter reset detection, and confidence assessment.
///
/// # Arguments
///
/// * `interface_name` - Name of the network interface
/// * `current_rx` - Current bytes received counter
/// * `current_tx` - Current bytes sent counter
/// * `now` - Current timestamp
/// * `previous_stats` - Map of previous interface statistics
/// * `min_time_threshold` - Minimum time threshold for reliable calculations (seconds)
/// * `collection_count` - Current collection number for logging purposes
///
/// # Returns
///
/// A tuple containing:
/// * `download_speed_bps` - Download speed in bytes per second
/// * `upload_speed_bps` - Upload speed in bytes per second  
/// * `calculation_confidence` - Confidence level of the calculation
///
/// # Validation and Error Handling
///
/// - **No previous data**: Returns (0.0, 0.0, None) for first reading
/// - **Time anomalies**: Detects negative time intervals (clock changes)
/// - **Counter resets**: Detects when counters decrease (interface restart)
/// - **Minimum time threshold**: Ensures sufficient time for reliable calculation
/// - **Confidence assessment**: Evaluates calculation reliability based on conditions
pub fn calculate_speeds_with_validation(
    interface_name: &str,
    current_rx: u64,
    current_tx: u64,
    now: DateTime<Utc>,
    previous_stats: &HashMap<String, (u64, u64, DateTime<Utc>, u32)>,
    min_time_threshold: f64,
    collection_count: u64,
) -> (f64, f64, CalculationConfidence) {
    // Check if we have previous data for this interface
    let Some((prev_rx, prev_tx, prev_time, _)) = previous_stats.get(interface_name) else {
        trace!(
            "Interface '{}' for collection #{}: No previous data available, establishing baseline",
            interface_name, collection_count
        );
        return (0.0, 0.0, CalculationConfidence::None);
    };

    // Calculate time difference with enhanced validation
    let time_diff_ms = (now - prev_time).num_milliseconds();

    // Handle time anomalies (negative time, system clock changes)
    if time_diff_ms <= 0 {
        warn!(
            "Interface '{}' for collection #{}: Time anomaly detected - time_diff={}ms (current={}, previous={}) - possible system clock issue",
            interface_name,
            collection_count,
            time_diff_ms,
            now.format("%H:%M:%S%.3f"),
            prev_time.format("%H:%M:%S%.3f")
        );
        return (0.0, 0.0, CalculationConfidence::None);
    }

    let time_diff = time_diff_ms as f64 / 1000.0;

    // Apply minimum time threshold validation
    if time_diff < min_time_threshold {
        trace!(
            "Interface '{}' for collection #{}: Time interval too small for reliable calculation: {:.3}s < {:.3}s",
            interface_name, collection_count, time_diff, min_time_threshold
        );
        return (0.0, 0.0, CalculationConfidence::Low);
    }

    // Calculate byte deltas with counter reset detection
    let (rx_delta, tx_delta, counter_reset_detected) = if current_rx >= *prev_rx
        && current_tx >= *prev_tx
    {
        (current_rx - prev_rx, current_tx - prev_tx, false)
    } else {
        // Potential counter reset
        debug!(
            "Interface '{}' for collection #{}: Counter reset detected (rx: {} -> {}, tx: {} -> {}) - establishing new baseline",
            interface_name, collection_count, prev_rx, current_rx, prev_tx, current_tx
        );
        return (0.0, 0.0, CalculationConfidence::Low);
    };

    // Calculate speeds
    let download_speed = rx_delta as f64 / time_diff;
    let upload_speed = tx_delta as f64 / time_diff;

    // Determine calculation confidence based on various factors
    let confidence = if counter_reset_detected {
        CalculationConfidence::Low
    } else if time_diff < 1.0 {
        CalculationConfidence::Medium
    } else if time_diff >= 1.0 && !counter_reset_detected {
        CalculationConfidence::High
    } else {
        CalculationConfidence::Medium
    };

    trace!(
        "Interface '{}' for collection #{}: Speed calculation - down: {:.2} B/s, up: {:.2} B/s (rx_delta={}, tx_delta={}, time_diff={:.3}s, confidence={:?})",
        interface_name,
        collection_count,
        download_speed,
        upload_speed,
        rx_delta,
        tx_delta,
        time_diff,
        confidence
    );

    (download_speed, upload_speed, confidence)
}

/// Detects counter reset conditions for network interface counters
///
/// This function analyzes current and previous counter values to detect
/// counter resets, which can occur due to interface restarts, driver reloads,
/// or system suspend/resume cycles.
///
/// # Arguments
///
/// * `current_rx` - Current bytes received counter
/// * `current_tx` - Current bytes sent counter
/// * `prev_rx` - Previous bytes received counter
/// * `prev_tx` - Previous bytes sent counter
///
/// # Returns
///
/// * `true` if a counter reset is detected (current < previous)
/// * `false` if counters are increasing normally
pub fn detect_counter_reset(current_rx: u64, current_tx: u64, prev_rx: u64, prev_tx: u64) -> bool {
    current_rx < prev_rx || current_tx < prev_tx
}

/// Detects time anomalies in measurement timestamps
///
/// This function identifies various time-related issues that can affect
/// bandwidth calculations, including negative time intervals, system
/// clock changes, and suspend/resume cycles.
///
/// # Arguments
///
/// * `current_time` - Current measurement timestamp
/// * `previous_time` - Previous measurement timestamp
/// * `min_threshold_ms` - Minimum acceptable time interval in milliseconds
///
/// # Returns
///
/// * `Ok(time_diff_ms)` if time interval is valid
/// * `Err(description)` if time anomaly is detected with description
pub fn detect_time_anomaly(
    current_time: DateTime<Utc>,
    previous_time: DateTime<Utc>,
    min_threshold_ms: i64,
) -> Result<i64, String> {
    let time_diff_ms = (current_time - previous_time).num_milliseconds();

    if time_diff_ms < 0 {
        return Err(format!(
            "Negative time interval: {}ms (system clock went backwards)",
            time_diff_ms
        ));
    }

    if time_diff_ms == 0 {
        return Err("Zero time interval (identical timestamps)".to_string());
    }

    if time_diff_ms < min_threshold_ms {
        return Err(format!(
            "Time interval too small: {}ms < {}ms (rapid successive measurements)",
            time_diff_ms, min_threshold_ms
        ));
    }

    // Check for suspiciously large time intervals (possible suspend/resume)
    if time_diff_ms > 300_000 {
        // 5 minutes
        return Err(format!(
            "Suspiciously large time interval: {}ms (possible system suspend/resume)",
            time_diff_ms
        ));
    }

    Ok(time_diff_ms)
}

/// Validates that packet and byte counters are consistent
///
/// This function checks for logical consistency between packet counts
/// and byte counts to detect data corruption or driver issues.
///
/// # Arguments
///
/// * `bytes` - Total bytes transferred
/// * `packets` - Total packets transferred
/// * `direction` - Direction description ("received" or "sent") for error messages
///
/// # Returns
///
/// * `Ok(())` if counters are consistent
/// * `Err(description)` if inconsistency is detected
pub fn validate_packet_byte_consistency(
    bytes: u64,
    packets: u64,
    direction: &str,
) -> Result<(), String> {
    // If we have packets, we should have bytes
    if packets > 0 && bytes == 0 {
        return Err(format!(
            "Invalid data: {} packets {} but 0 bytes",
            packets, direction
        ));
    }

    // If we have bytes, check for reasonable packet sizes
    if bytes > 0 && packets > 0 {
        let avg_packet_size = bytes as f64 / packets as f64;

        // Check for unreasonably small packets (less than 20 bytes is suspicious for most protocols)
        if avg_packet_size < 20.0 {
            return Err(format!(
                "Suspiciously small average packet size: {:.2} bytes (possible data corruption)",
                avg_packet_size
            ));
        }

        // Check for unreasonably large packets (over 64KB is suspicious)
        if avg_packet_size > 65536.0 {
            return Err(format!(
                "Average packet size too large: {:.2} bytes (possible counter overflow)",
                avg_packet_size
            ));
        }
    }

    Ok(())
}

/// Assesses the confidence level of speed calculations based on various factors
///
/// This function evaluates multiple factors to determine how reliable
/// the calculated speeds are, helping users understand data quality.
///
/// # Arguments
///
/// * `time_diff` - Time interval between measurements in seconds
/// * `counter_reset_detected` - Whether a counter reset was detected
/// * `data_validation_passed` - Whether data validation checks passed
/// * `consecutive_failures` - Number of consecutive failures for this interface
///
/// # Returns
///
/// Confidence level assessment:
/// * `High` - Optimal conditions, reliable data
/// * `Medium` - Good conditions with minor issues
/// * `Low` - Problematic conditions, data may be unreliable
/// * `None` - No confidence, data should not be trusted
pub fn assess_calculation_confidence(
    time_diff: f64,
    counter_reset_detected: bool,
    data_validation_passed: bool,
    consecutive_failures: u32,
) -> CalculationConfidence {
    // No confidence if data validation failed
    if !data_validation_passed {
        return CalculationConfidence::None;
    }

    // Low confidence if counter reset detected
    if counter_reset_detected {
        return CalculationConfidence::Low;
    }

    // Reduce confidence based on consecutive failures
    if consecutive_failures > 3 {
        return CalculationConfidence::None;
    } else if consecutive_failures > 1 {
        return CalculationConfidence::Low;
    }

    // Assess based on time interval
    if time_diff >= 2.0 {
        // Optimal time interval for stable measurements
        CalculationConfidence::High
    } else if time_diff >= 1.0 {
        // Good time interval
        CalculationConfidence::High
    } else if time_diff >= 0.5 {
        // Acceptable but not optimal
        CalculationConfidence::Medium
    } else {
        // Too short for reliable measurements
        CalculationConfidence::Low
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as ChronoDuration;
    use std::collections::HashMap;

    #[test]
    fn test_validate_interface_data_valid() {
        // Valid data should pass
        let result = validate_interface_data("eth0", 1000, 500, 10, 5, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_interface_data_packets_without_bytes() {
        // Packets without bytes should fail
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
    }

    #[test]
    fn test_validate_interface_data_large_packet_sizes() {
        // Unreasonably large packet sizes should fail
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
    }

    #[test]
    fn test_validate_interface_data_invalid_names() {
        // Empty interface name should fail
        let result = validate_interface_data("", 1000, 500, 10, 5, 1);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid interface name: empty string")
        );

        // Too long interface name should fail
        let long_name = "a".repeat(65);
        let result = validate_interface_data(&long_name, 1000, 500, 10, 5, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[test]
    fn test_calculate_speeds_with_validation_no_previous_data() {
        let previous_stats = HashMap::new();
        let now = Utc::now();

        let (download_speed, upload_speed, confidence) =
            calculate_speeds_with_validation("eth0", 1000, 500, now, &previous_stats, 0.1, 1);

        assert_eq!(download_speed, 0.0);
        assert_eq!(upload_speed, 0.0);
        assert_eq!(confidence, CalculationConfidence::None);
    }

    #[test]
    fn test_calculate_speeds_with_validation_normal_operation() {
        let mut previous_stats = HashMap::new();
        let base_time = Utc::now();
        let current_time = base_time + ChronoDuration::seconds(1);

        // Set up previous stats
        previous_stats.insert("eth0".to_string(), (1000, 500, base_time, 0));

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            2000,
            1000,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(download_speed, 1000.0); // 1000 bytes in 1 second
        assert_eq!(upload_speed, 500.0); // 500 bytes in 1 second
        assert_eq!(confidence, CalculationConfidence::High);
    }

    #[test]
    fn test_calculate_speeds_with_validation_time_anomaly() {
        let mut previous_stats = HashMap::new();
        let base_time = Utc::now();
        let current_time = base_time - ChronoDuration::seconds(1); // Time went backwards

        previous_stats.insert("eth0".to_string(), (1000, 500, base_time, 0));

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            2000,
            1000,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(download_speed, 0.0);
        assert_eq!(upload_speed, 0.0);
        assert_eq!(confidence, CalculationConfidence::None);
    }

    #[test]
    fn test_calculate_speeds_with_validation_counter_reset() {
        let mut previous_stats = HashMap::new();
        let base_time = Utc::now();
        let current_time = base_time + ChronoDuration::seconds(1);

        previous_stats.insert("eth0".to_string(), (5000, 2000, base_time, 0));

        // Counter reset: current < previous
        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            1000,
            500,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(download_speed, 0.0);
        assert_eq!(upload_speed, 0.0);
        assert_eq!(confidence, CalculationConfidence::Low);
    }

    #[test]
    fn test_calculate_speeds_with_validation_short_interval() {
        let mut previous_stats = HashMap::new();
        let base_time = Utc::now();
        let current_time = base_time + ChronoDuration::milliseconds(50); // 50ms < 100ms threshold

        previous_stats.insert("eth0".to_string(), (1000, 500, base_time, 0));

        let (download_speed, upload_speed, confidence) = calculate_speeds_with_validation(
            "eth0",
            1100,
            550,
            current_time,
            &previous_stats,
            0.1,
            1,
        );

        assert_eq!(download_speed, 0.0);
        assert_eq!(upload_speed, 0.0);
        assert_eq!(confidence, CalculationConfidence::Low);
    }

    #[test]
    fn test_detect_counter_reset() {
        // Normal operation - no reset
        assert!(!detect_counter_reset(2000, 1000, 1000, 500));

        // RX counter reset
        assert!(detect_counter_reset(500, 1000, 1000, 500));

        // TX counter reset
        assert!(detect_counter_reset(2000, 200, 1000, 500));

        // Both counters reset
        assert!(detect_counter_reset(500, 200, 1000, 500));
    }

    #[test]
    fn test_detect_time_anomaly() {
        let base_time = Utc::now();

        // Normal time progression
        let current_time = base_time + ChronoDuration::seconds(1);
        let result = detect_time_anomaly(current_time, base_time, 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000); // 1 second = 1000ms

        // Negative time (clock went backwards)
        let current_time = base_time - ChronoDuration::seconds(1);
        let result = detect_time_anomaly(current_time, base_time, 100);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Negative time interval"));

        // Zero time interval
        let result = detect_time_anomaly(base_time, base_time, 100);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Zero time interval"));

        // Too small interval
        let current_time = base_time + ChronoDuration::milliseconds(50);
        let result = detect_time_anomaly(current_time, base_time, 100);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Time interval too small"));

        // Suspiciously large interval (possible suspend/resume)
        let current_time = base_time + ChronoDuration::minutes(10);
        let result = detect_time_anomaly(current_time, base_time, 100);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Suspiciously large time interval")
        );
    }

    #[test]
    fn test_validate_packet_byte_consistency() {
        // Valid consistency
        assert!(validate_packet_byte_consistency(1000, 10, "received").is_ok());

        // Packets without bytes
        let result = validate_packet_byte_consistency(0, 10, "received");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("packets received but 0 bytes"));

        // Suspiciously small packets
        let result = validate_packet_byte_consistency(100, 10, "sent"); // 10 bytes per packet
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Suspiciously small average packet size")
        );

        // Unreasonably large packets
        let result = validate_packet_byte_consistency(1000000, 10, "received"); // 100KB per packet
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Average packet size too large")
        );
    }

    #[test]
    fn test_assess_calculation_confidence() {
        // High confidence - optimal conditions
        let confidence = assess_calculation_confidence(2.0, false, true, 0);
        assert_eq!(confidence, CalculationConfidence::High);

        // Low confidence - counter reset
        let confidence = assess_calculation_confidence(2.0, true, true, 0);
        assert_eq!(confidence, CalculationConfidence::Low);

        // None confidence - data validation failed
        let confidence = assess_calculation_confidence(2.0, false, false, 0);
        assert_eq!(confidence, CalculationConfidence::None);

        // None confidence - too many consecutive failures
        let confidence = assess_calculation_confidence(2.0, false, true, 5);
        assert_eq!(confidence, CalculationConfidence::None);

        // Low confidence - some consecutive failures
        let confidence = assess_calculation_confidence(2.0, false, true, 2);
        assert_eq!(confidence, CalculationConfidence::Low);

        // Medium confidence - short but acceptable interval
        let confidence = assess_calculation_confidence(0.7, false, true, 0);
        assert_eq!(confidence, CalculationConfidence::Medium);

        // Low confidence - very short interval
        let confidence = assess_calculation_confidence(0.3, false, true, 0);
        assert_eq!(confidence, CalculationConfidence::Low);
    }
}
