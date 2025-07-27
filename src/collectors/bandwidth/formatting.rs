//! Formatting utilities for bandwidth data
//!
//! This module provides utility functions for formatting bandwidth speeds and byte values
//! with appropriate units. It ensures consistent formatting across the entire application.

/// Formats speed in bytes per second with appropriate units
///
/// Converts raw bytes per second values into human-readable format with appropriate
/// unit prefixes (B/s, KB/s, MB/s, GB/s).
///
/// # Arguments
///
/// * `speed_bps` - Speed in bytes per second as a floating-point value
///
/// # Returns
///
/// A formatted string with the speed value and appropriate unit
///
/// # Examples
///
/// ```
/// use kaipo_watcher::collectors::bandwidth::formatting::format_speed;
///
/// assert_eq!(format_speed(0.0), "0.00 B/s");
/// assert_eq!(format_speed(512.0), "512.00 B/s");
/// assert_eq!(format_speed(1024.0), "1.00 KB/s");
/// assert_eq!(format_speed(1048576.0), "1.00 MB/s");
/// assert_eq!(format_speed(1073741824.0), "1.00 GB/s");
/// ```
pub fn format_speed(speed_bps: f64) -> String {
    if speed_bps < 1024.0 {
        format!("{:.2} B/s", speed_bps)
    } else if speed_bps < 1024.0 * 1024.0 {
        format!("{:.2} KB/s", speed_bps / 1024.0)
    } else if speed_bps < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} MB/s", speed_bps / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB/s", speed_bps / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Formats byte values with appropriate units
///
/// Converts raw byte values into human-readable format with appropriate
/// unit prefixes (B, KB, MB, GB, TB).
///
/// # Arguments
///
/// * `bytes` - Byte count as a floating-point value
///
/// # Returns
///
/// A formatted string with the byte value and appropriate unit
///
/// # Examples
///
/// ```
/// use kaipo_watcher::collectors::bandwidth::formatting::format_bytes;
///
/// assert_eq!(format_bytes(0.0), "0 B");
/// assert_eq!(format_bytes(512.0), "512 B");
/// assert_eq!(format_bytes(1024.0), "1.00 KB");
/// assert_eq!(format_bytes(1048576.0), "1.00 MB");
/// assert_eq!(format_bytes(1073741824.0), "1.00 GB");
/// assert_eq!(format_bytes(1099511627776.0), "1.00 TB");
/// ```
pub fn format_bytes(bytes: f64) -> String {
    if bytes < 1024.0 {
        format!("{:.0} B", bytes)
    } else if bytes < 1024.0 * 1024.0 {
        format!("{:.2} KB", bytes / 1024.0)
    } else if bytes < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} MB", bytes / (1024.0 * 1024.0))
    } else if bytes < 1024.0 * 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
    } else {
        format!("{:.2} TB", bytes / (1024.0 * 1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_speed() {
        // Test bytes per second formatting
        assert_eq!(format_speed(0.0), "0.00 B/s");
        assert_eq!(format_speed(512.0), "512.00 B/s");
        assert_eq!(format_speed(1023.0), "1023.00 B/s");

        // Test kilobytes per second formatting
        assert_eq!(format_speed(1024.0), "1.00 KB/s");
        assert_eq!(format_speed(1536.0), "1.50 KB/s");
        assert_eq!(format_speed(1048575.0), "1024.00 KB/s");

        // Test megabytes per second formatting
        assert_eq!(format_speed(1048576.0), "1.00 MB/s");
        assert_eq!(format_speed(1572864.0), "1.50 MB/s");
        assert_eq!(format_speed(1073741823.0), "1024.00 MB/s");

        // Test gigabytes per second formatting
        assert_eq!(format_speed(1073741824.0), "1.00 GB/s");
        assert_eq!(format_speed(1610612736.0), "1.50 GB/s");

        // Test edge cases
        assert_eq!(format_speed(0.0), "0.00 B/s");
        assert_eq!(format_speed(f64::INFINITY), "inf GB/s");
        assert_eq!(format_speed(f64::NAN), "NaN GB/s");
    }

    #[test]
    fn test_format_bytes() {
        // Test bytes formatting
        assert_eq!(format_bytes(0.0), "0 B");
        assert_eq!(format_bytes(512.0), "512 B");
        assert_eq!(format_bytes(1023.0), "1023 B");

        // Test kilobytes formatting
        assert_eq!(format_bytes(1024.0), "1.00 KB");
        assert_eq!(format_bytes(1536.0), "1.50 KB");
        assert_eq!(format_bytes(1048575.0), "1024.00 KB");

        // Test megabytes formatting
        assert_eq!(format_bytes(1048576.0), "1.00 MB");
        assert_eq!(format_bytes(1572864.0), "1.50 MB");
        assert_eq!(format_bytes(1073741823.0), "1024.00 MB");

        // Test gigabytes formatting
        assert_eq!(format_bytes(1073741824.0), "1.00 GB");
        assert_eq!(format_bytes(1610612736.0), "1.50 GB");
        assert_eq!(format_bytes(1099511627775.0), "1024.00 GB");

        // Test terabytes formatting
        assert_eq!(format_bytes(1099511627776.0), "1.00 TB");
        assert_eq!(format_bytes(1649267441664.0), "1.50 TB");

        // Test edge cases
        assert_eq!(format_bytes(0.0), "0 B");
        assert_eq!(format_bytes(f64::INFINITY), "inf TB");
        assert_eq!(format_bytes(f64::NAN), "NaN TB");
    }

    #[test]
    fn test_formatting_precision() {
        // Test that formatting maintains consistent precision
        assert_eq!(format_speed(1024.123), "1.00 KB/s");
        assert_eq!(format_speed(1024.999), "1.00 KB/s");
        assert_eq!(format_speed(1025.0), "1.00 KB/s");

        assert_eq!(format_bytes(1024.123), "1.00 KB");
        assert_eq!(format_bytes(1024.999), "1.00 KB");
        assert_eq!(format_bytes(1025.0), "1.00 KB");
    }

    #[test]
    fn test_formatting_consistency() {
        // Test that both functions handle similar values consistently
        let test_values = vec![0.0, 512.0, 1024.0, 1536.0, 1048576.0, 1073741824.0];

        for value in test_values {
            let speed_result = format_speed(value);
            let bytes_result = format_bytes(value);

            // Both should handle the same input values without panicking
            assert!(!speed_result.is_empty());
            assert!(!bytes_result.is_empty());

            // Both should use the same unit prefixes (just different suffixes)
            if value >= 1073741824.0 {
                assert!(speed_result.contains("GB"));
                assert!(bytes_result.contains("GB"));
            } else if value >= 1048576.0 {
                assert!(speed_result.contains("MB"));
                assert!(bytes_result.contains("MB"));
            } else if value >= 1024.0 {
                assert!(speed_result.contains("KB"));
                assert!(bytes_result.contains("KB"));
            } else {
                assert!(speed_result.contains("B/s"));
                assert!(bytes_result.contains("B"));
            }
        }
    }
}
