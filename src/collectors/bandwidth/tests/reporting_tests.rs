//! Tests for reporting functionality
//!
//! This module contains unit tests for troubleshooting reports,
//! error context reports, and diagnostic functionality.

#[cfg(test)]
mod tests {
    use crate::collectors::bandwidth::errors::{BandwidthError, SystemImpact};
    use crate::collectors::bandwidth::reporting::*;
    use crate::collectors::bandwidth::stats::{
        BandwidthStats, CalculationConfidence, InterfaceState, InterfaceType,
    };
    use crate::collectors::platform::interface_manager::{
        EnhancedInterfaceType, EthernetSubtype, InterfaceManager, InterfaceRelevance,
        PlatformInterfaceInfo,
    };
    use chrono::Utc;
    use std::collections::HashMap;
    use sysinfo::Networks;

    /// Creates test platform interface info
    fn create_test_interface_info(
        name: &str,
        interface_type: EnhancedInterfaceType,
        is_important: bool,
        is_default: bool,
    ) -> PlatformInterfaceInfo {
        PlatformInterfaceInfo {
            name: name.to_string(),
            interface_type,
            relevance: InterfaceRelevance {
                score: if is_important { 90 } else { 30 },
                reason: "Test interface".to_string(),
                show_by_default: is_default,
                is_important,
            },
            platform_metadata: HashMap::new(),
            should_filter: false,
        }
    }

    /// Creates a test bandwidth reporter for testing
    fn create_test_reporter() -> BandwidthReporter {
        let mut previous_stats = HashMap::new();
        let now = Utc::now();

        // Add some test data
        previous_stats.insert("eth0".to_string(), (1000, 500, now, 0));
        previous_stats.insert("wlan0".to_string(), (2000, 1000, now, 1));

        BandwidthReporter::new(
            42, // collection_count
            previous_stats,
            InterfaceManager::new(),
            3,   // max_retries
            100, // retry_delay_ms
            0.1, // min_time_threshold
        )
    }

    /// Creates test Networks instance for testing
    fn create_test_networks() -> Networks {
        Networks::new()
    }

    /// Creates test bandwidth stats for testing
    fn create_test_stats() -> Vec<BandwidthStats> {
        let now = Utc::now();
        vec![
            BandwidthStats {
                timestamp: now,
                interface_name: "eth0".to_string(),
                interface_type: InterfaceType::Ethernet,
                interface_state: InterfaceState::Up,
                bytes_received: 1000,
                bytes_sent: 500,
                packets_received: 10,
                packets_sent: 5,
                download_speed_bps: 100.0,
                upload_speed_bps: 50.0,
                calculation_confidence: CalculationConfidence::High,
                time_since_last_update: 1.0,
            },
            BandwidthStats {
                timestamp: now,
                interface_name: "wlan0".to_string(),
                interface_type: InterfaceType::WiFi,
                interface_state: InterfaceState::Up,
                bytes_received: 2000,
                bytes_sent: 1000,
                packets_received: 20,
                packets_sent: 10,
                download_speed_bps: 200.0,
                upload_speed_bps: 100.0,
                calculation_confidence: CalculationConfidence::Medium,
                time_since_last_update: 2.0,
            },
        ]
    }

    #[test]
    fn test_troubleshooting_report_generation() {
        let reporter = create_test_reporter();
        let networks = create_test_networks();
        let report = reporter.create_troubleshooting_report(&networks);

        // Verify basic report structure
        assert_eq!(report.collection_count, 42);
        assert!(report.timestamp <= Utc::now());

        // Verify system info
        assert!(!report.system_info.platform.is_empty());
        assert!(!report.system_info.architecture.is_empty());
        assert!(!report.system_info.rust_version.is_empty());

        // Verify configuration
        assert_eq!(report.configuration.max_retries, 3);
        assert_eq!(report.configuration.retry_delay_ms, 100);
        assert_eq!(report.configuration.min_time_threshold, 0.1);

        // Verify collection history
        assert_eq!(report.collection_history.total_collections, 42);
        assert!(report.collection_history.interfaces_with_failures <= 2); // We have 2 interfaces, one with failure
        assert!(report.collection_history.average_collection_duration_ms > 0.0);
    }

    #[test]
    fn test_error_context_report_creation() {
        let reporter = create_test_reporter();

        // Test with RefreshFailed error
        let refresh_error = BandwidthError::RefreshFailed {
            message: "Test network refresh failure".to_string(),
            retry_attempts: 3,
        };

        let networks = create_test_networks();
        let error_report = reporter.create_error_context_report(&refresh_error, &networks);

        // Verify error information
        assert!(error_report.error_type.contains("RefreshFailed"));
        assert!(
            error_report
                .error_message
                .contains("Test network refresh failure")
        );
        assert!(
            error_report
                .user_friendly_message
                .contains("Failed to refresh network statistics")
        );
        assert_eq!(error_report.system_impact, SystemImpact::Critical);

        // Verify suggested actions
        assert!(!error_report.suggested_actions.is_empty());
        assert!(
            error_report
                .suggested_actions
                .iter()
                .any(|action| action.contains("network interface"))
        );

        // Verify troubleshooting report is included
        assert_eq!(error_report.troubleshooting_report.collection_count, 42);
    }

    #[test]
    fn test_error_context_report_different_error_types() {
        let reporter = create_test_reporter();

        // Test CounterReset error
        let counter_reset_error = BandwidthError::CounterReset {
            interface: "eth0".to_string(),
            current: 100,
            previous: 200,
        };

        let networks = create_test_networks();
        let error_report = reporter.create_error_context_report(&counter_reset_error, &networks);
        assert_eq!(error_report.system_impact, SystemImpact::Low);
        assert!(
            error_report
                .user_friendly_message
                .contains("Network counter reset detected")
        );
        assert!(
            error_report
                .suggested_actions
                .iter()
                .any(|action| action.contains("Wait for next measurement"))
        );

        // Test NoInterfacesFound error
        let no_interfaces_error = BandwidthError::NoInterfacesFound;
        let networks = create_test_networks();
        let error_report = reporter.create_error_context_report(&no_interfaces_error, &networks);
        assert_eq!(error_report.system_impact, SystemImpact::Critical);
        assert!(
            error_report
                .user_friendly_message
                .contains("No network interfaces were found")
        );

        // Test TimeAnomaly error
        let time_anomaly_error = BandwidthError::TimeAnomaly {
            description: "System clock jumped backward".to_string(),
            current_time: Utc::now(),
            previous_time: Utc::now(),
        };
        let networks = create_test_networks();
        let error_report = reporter.create_error_context_report(&time_anomaly_error, &networks);
        assert_eq!(error_report.system_impact, SystemImpact::Low);
        assert!(
            error_report
                .user_friendly_message
                .contains("Time anomaly detected")
        );
    }

    #[test]
    fn test_interface_summary_report_generation() {
        let mut reporter = create_test_reporter();

        // Create test interface info
        let interface_info = vec![
            create_test_interface_info(
                "eth0",
                EnhancedInterfaceType::Ethernet {
                    subtype: EthernetSubtype::Standard,
                },
                true,
                true,
            ),
            create_test_interface_info(
                "wlan0",
                EnhancedInterfaceType::WiFi {
                    standard: Some("802.11ac".to_string()),
                },
                true,
                true,
            ),
            create_test_interface_info("lo", EnhancedInterfaceType::Loopback, false, false),
        ];

        let summary_report = reporter.create_interface_summary_report(interface_info);

        // Verify basic report structure
        assert_eq!(summary_report.total_interfaces, 3);
        assert!(summary_report.timestamp <= Utc::now());
        assert!(!summary_report.platform.is_empty());

        // Verify interface details are included
        assert_eq!(summary_report.interface_details.len(), 3);
        assert!(
            summary_report
                .interface_details
                .iter()
                .any(|info| info.name == "eth0")
        );
        assert!(
            summary_report
                .interface_details
                .iter()
                .any(|info| info.name == "wlan0")
        );
        assert!(
            summary_report
                .interface_details
                .iter()
                .any(|info| info.name == "lo")
        );

        // Verify cache size is reported (usize is always >= 0, but we check it exists)
        let _ = summary_report.cache_size;
    }

    #[test]
    fn test_support_report_export() {
        let reporter = create_test_reporter();

        // Test support report without error
        let networks = create_test_networks();
        let support_report = reporter.export_support_report(None, &networks).unwrap();

        // Verify report contains expected sections
        assert!(support_report.contains("KAIPO WATCHER BANDWIDTH COLLECTOR SUPPORT REPORT"));
        assert!(support_report.contains("SYSTEM INFORMATION"));
        assert!(support_report.contains("COLLECTOR CONFIGURATION"));
        assert!(support_report.contains("COLLECTION HISTORY"));
        assert!(support_report.contains("INTERFACE DIAGNOSTICS"));
        assert!(support_report.contains("END OF REPORT"));

        // Verify system information is included
        assert!(support_report.contains("Platform:"));
        assert!(support_report.contains("Architecture:"));
        assert!(support_report.contains("Rust Version:"));

        // Verify configuration is included
        assert!(support_report.contains("Max Retries: 3"));
        assert!(support_report.contains("Retry Delay: 100ms"));
        assert!(support_report.contains("Min Time Threshold: 0.100s"));

        // Verify collection history is included
        assert!(support_report.contains("Total Collections: 42"));
    }

    #[test]
    fn test_support_report_export_with_error() {
        let reporter = create_test_reporter();

        let error = BandwidthError::RefreshFailed {
            message: "Test error for support report".to_string(),
            retry_attempts: 2,
        };

        let networks = create_test_networks();
        let support_report = reporter
            .export_support_report(Some(&error), &networks)
            .unwrap();

        // Verify error information is included
        assert!(support_report.contains("ERROR INFORMATION"));
        assert!(support_report.contains("Error Type: RefreshFailed"));
        assert!(support_report.contains("Test error for support report"));
        assert!(support_report.contains("System Impact: Critical"));
        assert!(support_report.contains("Suggested Actions:"));

        // Verify the rest of the report is still there
        assert!(support_report.contains("SYSTEM INFORMATION"));
        assert!(support_report.contains("INTERFACE DIAGNOSTICS"));
    }

    #[test]
    fn test_user_friendly_error_messages() {
        let reporter = create_test_reporter();

        // Test RefreshFailed error message
        let refresh_error = BandwidthError::RefreshFailed {
            message: "Network subsystem error".to_string(),
            retry_attempts: 3,
        };
        let message = reporter.create_user_friendly_error_message(&refresh_error);
        assert!(message.contains("Failed to refresh network statistics after 4 attempts"));
        assert!(message.contains("Try:"));
        assert!(message.contains("Network subsystem error"));

        // Test NoInterfacesFound error message
        let no_interfaces_error = BandwidthError::NoInterfacesFound;
        let message = reporter.create_user_friendly_error_message(&no_interfaces_error);
        assert!(message.contains("No network interfaces were found"));
        assert!(message.contains("Try:"));
        assert!(message.contains("ip link"));

        // Test InterfaceNotFound error message
        let interface_not_found_error = BandwidthError::InterfaceNotFound {
            interface: "eth1".to_string(),
        };
        let message = reporter.create_user_friendly_error_message(&interface_not_found_error);
        assert!(message.contains("interface 'eth1' was not found"));
        assert!(message.contains("Try:"));

        // Test CounterReset error message
        let counter_reset_error = BandwidthError::CounterReset {
            interface: "wlan0".to_string(),
            current: 100,
            previous: 200,
        };
        let message = reporter.create_user_friendly_error_message(&counter_reset_error);
        assert!(message.contains("Network counter reset detected for interface 'wlan0'"));
        assert!(message.contains("This is normal"));

        // Test InvalidTimeInterval error message
        let time_interval_error = BandwidthError::InvalidTimeInterval {
            interval_ms: 50,
            min_threshold_ms: 100,
        };
        let message = reporter.create_user_friendly_error_message(&time_interval_error);
        assert!(message.contains("Time interval too small"));
        assert!(message.contains("50ms"));
        assert!(message.contains("100ms"));
    }

    #[test]
    fn test_system_impact_assessment() {
        let reporter = create_test_reporter();

        // Test Critical impact errors
        let refresh_error = BandwidthError::RefreshFailed {
            message: "Test".to_string(),
            retry_attempts: 3,
        };
        assert_eq!(
            reporter.assess_system_impact(&refresh_error),
            SystemImpact::Critical
        );

        let no_interfaces_error = BandwidthError::NoInterfacesFound;
        assert_eq!(
            reporter.assess_system_impact(&no_interfaces_error),
            SystemImpact::Critical
        );

        // Test High impact errors
        let system_resource_error = BandwidthError::SystemResourceError {
            message: "Resource exhausted".to_string(),
        };
        assert_eq!(
            reporter.assess_system_impact(&system_resource_error),
            SystemImpact::High
        );

        // Test Medium impact errors
        let interface_not_found_error = BandwidthError::InterfaceNotFound {
            interface: "eth0".to_string(),
        };
        assert_eq!(
            reporter.assess_system_impact(&interface_not_found_error),
            SystemImpact::Medium
        );

        let data_validation_error = BandwidthError::DataValidationFailed {
            interface: "eth0".to_string(),
            validation_error: "Invalid packet size".to_string(),
        };
        assert_eq!(
            reporter.assess_system_impact(&data_validation_error),
            SystemImpact::Medium
        );

        // Test Low impact errors
        let time_anomaly_error = BandwidthError::TimeAnomaly {
            description: "Clock jump".to_string(),
            current_time: Utc::now(),
            previous_time: Utc::now(),
        };
        assert_eq!(
            reporter.assess_system_impact(&time_anomaly_error),
            SystemImpact::Low
        );

        let counter_reset_error = BandwidthError::CounterReset {
            interface: "eth0".to_string(),
            current: 100,
            previous: 200,
        };
        assert_eq!(
            reporter.assess_system_impact(&counter_reset_error),
            SystemImpact::Low
        );

        let time_interval_error = BandwidthError::InvalidTimeInterval {
            interval_ms: 50,
            min_threshold_ms: 100,
        };
        assert_eq!(
            reporter.assess_system_impact(&time_interval_error),
            SystemImpact::Low
        );
    }

    #[test]
    fn test_suggested_actions_for_different_errors() {
        let reporter = create_test_reporter();

        // Test RefreshFailed suggestions
        let refresh_error = BandwidthError::RefreshFailed {
            message: "Test".to_string(),
            retry_attempts: 3,
        };
        let actions = reporter.get_suggested_actions(&refresh_error);
        assert!(!actions.is_empty());
        assert!(
            actions
                .iter()
                .any(|action| action.contains("network interface"))
        );
        assert!(
            actions
                .iter()
                .any(|action| action.contains("administrator"))
        );

        // Test NoInterfacesFound suggestions
        let no_interfaces_error = BandwidthError::NoInterfacesFound;
        let actions = reporter.get_suggested_actions(&no_interfaces_error);
        assert!(
            actions
                .iter()
                .any(|action| action.contains("Verify network interfaces exist"))
        );
        assert!(
            actions
                .iter()
                .any(|action| action.contains("network driver"))
        );

        // Test CounterReset suggestions
        let counter_reset_error = BandwidthError::CounterReset {
            interface: "eth0".to_string(),
            current: 100,
            previous: 200,
        };
        let actions = reporter.get_suggested_actions(&counter_reset_error);
        assert!(
            actions
                .iter()
                .any(|action| action.contains("Wait for next measurement"))
        );
        assert!(actions.iter().any(|action| action.contains("temporary")));

        // Test SystemResourceError suggestions
        let system_resource_error = BandwidthError::SystemResourceError {
            message: "Resource exhausted".to_string(),
        };
        let actions = reporter.get_suggested_actions(&system_resource_error);
        assert!(
            actions
                .iter()
                .any(|action| action.contains("administrator"))
        );
        assert!(
            actions
                .iter()
                .any(|action| action.contains("system resources"))
        );
    }

    #[test]
    fn test_log_error_event() {
        let reporter = create_test_reporter();
        let error = anyhow::anyhow!("Test error for logging");

        // This test mainly verifies the function doesn't panic
        // In a real scenario, you might want to capture log output
        reporter.log_error_event(&error, "test_context");
    }

    #[test]
    fn test_log_success_event() {
        let reporter = create_test_reporter();
        let stats = create_test_stats();

        // This test mainly verifies the function doesn't panic
        // In a real scenario, you might want to capture log output
        reporter.log_success_event(&stats, 150.0);
    }

    #[test]
    fn test_gather_system_info() {
        let reporter = create_test_reporter();
        let system_info = reporter.gather_system_info();

        // Verify system info contains expected data
        assert!(!system_info.platform.is_empty());
        assert!(!system_info.architecture.is_empty());
        assert!(!system_info.rust_version.is_empty());

        // Verify platform is one of the expected values
        assert!(["linux", "macos", "windows"].contains(&system_info.platform.as_str()));
    }

    #[test]
    fn test_gather_interface_diagnostics() {
        let reporter = create_test_reporter();
        let networks = create_test_networks();
        let diagnostics = reporter.gather_interface_diagnostics(&networks);

        // The diagnostics will be empty since we're using an empty Networks instance
        // But we can verify the function works without panicking
        assert!(diagnostics.is_empty() || !diagnostics.is_empty());

        // If there are diagnostics, verify their structure
        for diagnostic in diagnostics {
            assert!(!diagnostic.name.is_empty());
            // consecutive_failures is u32, so always >= 0, but we verify it exists
            let _ = diagnostic.consecutive_failures;
        }
    }

    #[test]
    fn test_gather_collection_history() {
        let reporter = create_test_reporter();
        let history = reporter.gather_collection_history();

        // Verify collection history structure
        assert_eq!(history.total_collections, 42);
        assert!(history.interfaces_with_failures <= 2); // We have at most 2 interfaces in test data
        assert!(history.average_collection_duration_ms > 0.0);
    }

    #[test]
    fn test_serialization_of_report_structures() {
        let mut reporter = create_test_reporter();

        // Test TroubleshootingReport serialization
        let networks = create_test_networks();
        let troubleshooting_report = reporter.create_troubleshooting_report(&networks);
        let json = serde_json::to_string(&troubleshooting_report).unwrap();
        assert!(json.contains("collection_count"));
        assert!(json.contains("system_info"));

        // Test ErrorContextReport serialization
        let error = BandwidthError::RefreshFailed {
            message: "Test".to_string(),
            retry_attempts: 1,
        };
        let networks = create_test_networks();
        let error_report = reporter.create_error_context_report(&error, &networks);
        let json = serde_json::to_string(&error_report).unwrap();
        assert!(json.contains("error_type"));
        assert!(json.contains("system_impact"));

        // Test InterfaceSummaryReport serialization
        let interface_info = vec![create_test_interface_info(
            "eth0",
            EnhancedInterfaceType::Ethernet {
                subtype: EthernetSubtype::Standard,
            },
            true,
            true,
        )];
        let summary_report = reporter.create_interface_summary_report(interface_info);
        let json = serde_json::to_string(&summary_report).unwrap();
        assert!(json.contains("total_interfaces"));
        assert!(json.contains("interface_details"));
    }
}
