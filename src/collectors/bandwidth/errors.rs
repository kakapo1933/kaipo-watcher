//! Error handling module for bandwidth monitoring operations
//! 
//! This module provides comprehensive error types, system impact assessment,
//! and user-friendly error messaging for bandwidth collection operations.
//! It includes structured error reporting and troubleshooting guidance.


use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Comprehensive error types for bandwidth monitoring operations
/// 
/// Provides specific error categories for different failure modes to enable proper error handling.
/// Each error variant includes detailed context information to help with debugging and user guidance.
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum BandwidthError {
    /// Failed to refresh network statistics from the system
    #[error("Failed to refresh network statistics: {message}")]
    RefreshFailed { 
        /// Detailed error message
        message: String,
        /// Number of retry attempts made
        retry_attempts: u32,
    },
    
    /// Invalid time interval detected during speed calculation
    #[error("Invalid time interval for speed calculation: {interval_ms}ms (minimum: {min_threshold_ms}ms)")]
    InvalidTimeInterval { 
        /// Actual time interval in milliseconds
        interval_ms: i64,
        /// Minimum required threshold in milliseconds
        min_threshold_ms: u64,
    },
    
    /// Network counter reset or wraparound detected
    #[error("Counter reset detected for interface '{interface}': current={current}, previous={previous}")]
    CounterReset { 
        /// Name of the affected interface
        interface: String,
        /// Current counter value
        current: u64,
        /// Previous counter value
        previous: u64,
    },
    
    /// No network interfaces found on the system
    #[error("No network interfaces found on the system")]
    NoInterfacesFound,
    
    /// Specific interface not found or no longer available
    #[error("Interface '{interface}' not found or unavailable")]
    InterfaceNotFound { 
        /// Name of the missing interface
        interface: String,
    },
    
    /// System resource or permission error
    #[error("System resource error: {message}")]
    SystemResourceError {
        /// Error message describing the system issue
        message: String,
    },
    
    /// Data validation failed for interface readings
    #[error("Data validation failed for interface '{interface}': {validation_error}")]
    DataValidationFailed {
        /// Name of the interface with invalid data
        interface: String,
        /// Description of the validation failure
        validation_error: String,
    },
    
    /// Time anomaly detected during measurement (system clock changes, suspend/resume)
    #[error("Time anomaly detected: {description} (current_time={current_time}, previous_time={previous_time})")]
    TimeAnomaly {
        /// Description of the time anomaly
        description: String,
        /// Current timestamp
        current_time: DateTime<Utc>,
        /// Previous timestamp
        previous_time: DateTime<Utc>,
    },
}

/// System impact assessment for different error types
/// 
/// Categorizes errors by their impact on system functionality to help prioritize
/// error handling and user communication strategies.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SystemImpact {
    /// Critical: Bandwidth monitoring completely unavailable
    Critical,
    /// High: Significant degradation in monitoring capabilities
    High,
    /// Medium: Partial functionality affected
    Medium,
    /// Low: Minor issues that don't significantly impact functionality
    Low,
}

/// Comprehensive error context report for detailed debugging
/// 
/// Provides structured error information including technical details,
/// user-friendly messages, troubleshooting guidance, and system impact assessment.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorContextReport {
    /// String representation of the error type
    pub error_type: String,
    /// Technical error message
    pub error_message: String,
    /// User-friendly explanation of the error
    pub user_friendly_message: String,
    /// Detailed troubleshooting information
    pub troubleshooting_report: TroubleshootingReport,
    /// Specific actions the user can take to resolve the issue
    pub suggested_actions: Vec<String>,
    /// Assessment of how this error impacts system functionality
    pub system_impact: SystemImpact,
}

/// Comprehensive troubleshooting report for debugging bandwidth collection issues
/// 
/// Contains detailed system information, interface diagnostics, and collection history
/// to help identify the root cause of bandwidth monitoring problems.
#[derive(Debug, Serialize, Deserialize)]
pub struct TroubleshootingReport {
    /// Timestamp when the report was generated
    pub timestamp: DateTime<Utc>,
    /// Number of collections performed by the collector
    pub collection_count: u64,
    /// System information relevant to bandwidth monitoring
    pub system_info: SystemInfo,
    /// Diagnostic information about network interfaces
    pub interface_diagnostics: InterfaceDiagnostics,
    /// Historical information about recent collections
    pub collection_history: CollectionHistory,
}

/// System information relevant to bandwidth monitoring
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system platform
    pub platform: String,
    /// System architecture
    pub architecture: String,
    /// Number of available network interfaces
    pub available_interfaces: usize,
    /// Current system uptime information
    pub uptime_info: String,
}

/// Diagnostic information about network interfaces
#[derive(Debug, Serialize, Deserialize)]
pub struct InterfaceDiagnostics {
    /// Total number of interfaces detected
    pub total_interfaces: usize,
    /// Number of active interfaces
    pub active_interfaces: usize,
    /// Number of interfaces with recent failures
    pub failed_interfaces: usize,
    /// Interface names and their current status
    pub interface_status: Vec<(String, String)>,
}

/// Historical information about recent collections
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionHistory {
    /// Number of successful collections in recent history
    pub successful_collections: u32,
    /// Number of failed collections in recent history
    pub failed_collections: u32,
    /// Average collection duration in milliseconds
    pub average_duration_ms: f64,
    /// Most recent collection results
    pub recent_results: Vec<String>,
}

/// Error context creation and management functions
impl BandwidthError {
    /// Creates user-friendly error messages for common failure scenarios
    /// 
    /// Converts technical error information into clear, actionable guidance
    /// that helps users understand what went wrong and how to fix it.
    pub fn create_user_friendly_message(&self) -> String {
        match self {
            BandwidthError::RefreshFailed { message, retry_attempts } => {
                format!(
                    "Failed to refresh network statistics after {} attempts.\n\
                     This usually means:\n\
                     - Your system's network subsystem is experiencing issues\n\
                     - Network interfaces are rapidly changing state\n\
                     - Insufficient system resources\n\
                     \n\
                     Try:\n\
                     - Waiting a moment and running the command again\n\
                     - Checking if your network interfaces are stable\n\
                     - Running with elevated privileges if needed\n\
                     \n\
                     Technical details: {}",
                    retry_attempts, message
                )
            }
            BandwidthError::NoInterfacesFound => {
                "No network interfaces found on your system.\n\
                 This usually means:\n\
                 - All network interfaces are disconnected or disabled\n\
                 - You don't have permission to access network statistics\n\
                 - Your system's network subsystem is not functioning\n\
                 \n\
                 Try:\n\
                 - Checking your network connections\n\
                 - Running with administrator/root privileges\n\
                 - Verifying that network interfaces exist with 'ip addr' (Linux) or 'ifconfig' (macOS)".to_string()
            }
            BandwidthError::InterfaceNotFound { interface } => {
                format!(
                    "Network interface '{}' not found or unavailable.\n\
                     This usually means:\n\
                     - The interface name was mistyped\n\
                     - The interface was disconnected or disabled\n\
                     - The interface is virtual and was removed\n\
                     \n\
                     Try:\n\
                     - Running without specifying an interface to see all available interfaces\n\
                     - Checking the correct interface name with 'ip addr' (Linux) or 'ifconfig' (macOS)\n\
                     - Verifying the interface is connected and enabled",
                    interface
                )
            }
            BandwidthError::CounterReset { interface, current, previous } => {
                format!(
                    "Network counter reset detected on interface '{}'.\n\
                     Counter values: {} -> {} (decrease detected)\n\
                     \n\
                     This is normal and can happen when:\n\
                     - The network interface was restarted or reset\n\
                     - The system was suspended/hibernated\n\
                     - Network driver was reloaded\n\
                     \n\
                     This is normal and the next measurement should work correctly.\n\
                     Run the measurement again to get accurate speed readings.",
                    interface, previous, current
                )
            }
            BandwidthError::InvalidTimeInterval { interval_ms, min_threshold_ms } => {
                format!(
                    "Time interval too small for reliable speed calculation.\n\
                     Measured interval: {}ms (minimum required: {}ms)\n\
                     \n\
                     This usually means:\n\
                     - The measurement duration was too short\n\
                     - System timing issues or high CPU load\n\
                     - Rapid successive measurements\n\
                     \n\
                     Try:\n\
                     - Using a longer measurement duration (--measurement-duration)\n\
                     - Waiting a moment between measurements\n\
                     - Checking system load and performance",
                    interval_ms, min_threshold_ms
                )
            }
            BandwidthError::SystemResourceError { message } => {
                format!(
                    "System resource error encountered.\n\
                     \n\
                     Error details: {}\n\
                     \n\
                     This usually means:\n\
                     - Insufficient system permissions\n\
                     - System resource constraints (memory, file descriptors)\n\
                     - Network subsystem overload\n\
                     \n\
                     Try:\n\
                     - Running with elevated privileges (sudo)\n\
                     - Closing other network monitoring tools\n\
                     - Checking system resource usage\n\
                     - Restarting network services if necessary",
                    message
                )
            }
            BandwidthError::DataValidationFailed { interface, validation_error } => {
                format!(
                    "Data validation failed for interface '{}'.\n\
                     \n\
                     Validation error: {}\n\
                     \n\
                     This usually means:\n\
                     - The network interface is providing inconsistent data\n\
                     - Driver or firmware issues with the network adapter\n\
                     - Virtual interface with unusual behavior\n\
                     \n\
                     Try:\n\
                     - Checking the interface status and connectivity\n\
                     - Updating network drivers\n\
                     - Excluding problematic virtual interfaces\n\
                     - Running diagnostics on the network adapter",
                    interface, validation_error
                )
            }
            BandwidthError::TimeAnomaly { description, current_time, previous_time } => {
                format!(
                    "Time anomaly detected during measurement.\n\
                     \n\
                     Anomaly: {}\n\
                     Current time: {}\n\
                     Previous time: {}\n\
                     \n\
                     This usually means:\n\
                     - System clock was adjusted during measurement\n\
                     - System was suspended or hibernated\n\
                     - NTP time synchronization occurred\n\
                     - System performance issues causing timing problems\n\
                     \n\
                     Try:\n\
                     - Running the measurement again\n\
                     - Checking system clock stability\n\
                     - Avoiding system suspend during measurements\n\
                     - Checking for NTP synchronization events",
                    description, 
                    current_time.format("%Y-%m-%d %H:%M:%S%.3f UTC"),
                    previous_time.format("%Y-%m-%d %H:%M:%S%.3f UTC")
                )
            }
        }
    }

    /// Provides specific suggested actions based on error type
    /// 
    /// Returns a list of actionable steps that users can take to resolve
    /// or work around the specific error condition.
    pub fn get_suggested_actions(&self) -> Vec<String> {
        match self {
            BandwidthError::RefreshFailed { retry_attempts, .. } => {
                vec![
                    "Check system network service status".to_string(),
                    "Verify network interfaces are stable".to_string(),
                    "Try running with elevated privileges".to_string(),
                    format!("Consider increasing retry attempts (current: {})", retry_attempts),
                    "Check system resource usage (CPU, memory)".to_string(),
                ]
            }
            BandwidthError::NoInterfacesFound => {
                vec![
                    "Verify network interfaces exist: ip addr (Linux) or ifconfig (macOS)".to_string(),
                    "Check if all interfaces are disabled or disconnected".to_string(),
                    "Run with administrator/root privileges".to_string(),
                    "Restart network services if necessary".to_string(),
                    "Check system network subsystem status".to_string(),
                ]
            }
            BandwidthError::InterfaceNotFound { interface } => {
                vec![
                    format!("Verify interface '{}' exists and is enabled", interface),
                    "Check interface name spelling and case sensitivity".to_string(),
                    "List available interfaces without filtering".to_string(),
                    "Check if interface was recently removed or renamed".to_string(),
                ]
            }
            BandwidthError::CounterReset { .. } => {
                vec![
                    "This is normal behavior - wait for next measurement".to_string(),
                    "Check if interface was recently restarted".to_string(),
                    "Verify system wasn't suspended or hibernated".to_string(),
                    "Consider using longer measurement intervals".to_string(),
                ]
            }
            BandwidthError::InvalidTimeInterval { .. } => {
                vec![
                    "Use longer measurement duration (--measurement-duration)".to_string(),
                    "Wait between successive measurements".to_string(),
                    "Check system clock stability".to_string(),
                    "Reduce system load during measurements".to_string(),
                ]
            }
            BandwidthError::SystemResourceError { .. } => {
                vec![
                    "Run with elevated privileges (sudo/administrator)".to_string(),
                    "Close other network monitoring tools".to_string(),
                    "Check available system resources".to_string(),
                    "Restart network services if necessary".to_string(),
                ]
            }
            BandwidthError::DataValidationFailed { interface, .. } => {
                vec![
                    format!("Check interface '{}' status and connectivity", interface),
                    "Update network drivers".to_string(),
                    "Run network adapter diagnostics".to_string(),
                    "Consider excluding problematic virtual interfaces".to_string(),
                ]
            }
            BandwidthError::TimeAnomaly { .. } => {
                vec![
                    "Run the measurement again".to_string(),
                    "Check system clock stability".to_string(),
                    "Avoid system suspend during measurements".to_string(),
                    "Check for NTP synchronization events".to_string(),
                ]
            }
        }
    }

    /// Assesses the system impact of different error types
    /// 
    /// Categorizes errors by their severity and impact on bandwidth monitoring
    /// functionality to help prioritize error handling and user communication.
    pub fn assess_system_impact(&self) -> SystemImpact {
        match self {
            BandwidthError::RefreshFailed { .. } => SystemImpact::Critical,
            BandwidthError::NoInterfacesFound => SystemImpact::Critical,
            BandwidthError::SystemResourceError { .. } => SystemImpact::High,
            BandwidthError::InterfaceNotFound { .. } => SystemImpact::Medium,
            BandwidthError::DataValidationFailed { .. } => SystemImpact::Medium,
            BandwidthError::TimeAnomaly { .. } => SystemImpact::Low,
            BandwidthError::CounterReset { .. } => SystemImpact::Low,
            BandwidthError::InvalidTimeInterval { .. } => SystemImpact::Low,
        }
    }

    /// Creates a detailed error context report for specific errors
    /// 
    /// Provides comprehensive debugging information beyond the standard error message,
    /// including troubleshooting guidance and system impact assessment.
    pub fn create_error_context_report(&self, troubleshooting_report: TroubleshootingReport) -> ErrorContextReport {
        let error_type = match self {
            BandwidthError::RefreshFailed { .. } => "RefreshFailed",
            BandwidthError::InvalidTimeInterval { .. } => "InvalidTimeInterval",
            BandwidthError::CounterReset { .. } => "CounterReset",
            BandwidthError::NoInterfacesFound => "NoInterfacesFound",
            BandwidthError::InterfaceNotFound { .. } => "InterfaceNotFound",
            BandwidthError::SystemResourceError { .. } => "SystemResourceError",
            BandwidthError::DataValidationFailed { .. } => "DataValidationFailed",
            BandwidthError::TimeAnomaly { .. } => "TimeAnomaly",
        };
        
        ErrorContextReport {
            error_type: error_type.to_string(),
            error_message: self.to_string(),
            user_friendly_message: self.create_user_friendly_message(),
            troubleshooting_report,
            suggested_actions: self.get_suggested_actions(),
            system_impact: self.assess_system_impact(),
        }
    }
}

/// Error logging and monitoring functions
impl BandwidthError {
    /// Logs a structured error event for monitoring and debugging
    /// 
    /// Provides consistent error logging format for monitoring systems
    /// and debugging purposes.
    pub fn log_error_event(&self, context: &str, collection_count: u64) {
        log::error!(
            "Bandwidth collector error event for collection #{} (context={}, error={}, system_impact={:?})", 
            collection_count, 
            context, 
            self,
            self.assess_system_impact()
        );
    }
}

/// Success event logging for monitoring and performance tracking
/// 
/// Logs structured success events for bandwidth collection operations
/// to provide monitoring and performance insights.
pub fn log_success_event(
    interface_count: usize, 
    duration_ms: f64, 
    total_download_bps: f64, 
    total_upload_bps: f64, 
    collection_count: u64
) {
    log::info!(
        "Bandwidth collection success event for collection #{} (interfaces_processed={}, duration={:.3}ms, total_download={:.2}B/s, total_upload={:.2}B/s, performance={}, system_health=operational)", 
        collection_count, 
        interface_count, 
        duration_ms, 
        total_download_bps, 
        total_upload_bps,
        if duration_ms < 100.0 { "excellent" } else if duration_ms < 500.0 { "good" } else { "slow" }
    );
}

/// Logs a structured error event for anyhow::Error types
/// 
/// Provides consistent error logging format for general error types
/// encountered during bandwidth collection operations.
pub fn log_error_event_anyhow(error: &anyhow::Error, context: &str, collection_count: u64) {
    log::error!(
        "Bandwidth collector error event for collection #{} (context={}, error={}, system_impact=bandwidth_monitoring_degraded)", 
        collection_count, 
        context, 
        error
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation_and_display() {
        let refresh_error = BandwidthError::RefreshFailed {
            message: "Network subsystem unavailable".to_string(),
            retry_attempts: 3,
        };

        assert_eq!(
            refresh_error.to_string(),
            "Failed to refresh network statistics: Network subsystem unavailable"
        );

        let counter_reset_error = BandwidthError::CounterReset {
            interface: "eth0".to_string(),
            current: 1000,
            previous: 2000,
        };

        assert_eq!(
            counter_reset_error.to_string(),
            "Counter reset detected for interface 'eth0': current=1000, previous=2000"
        );
    }

    #[test]
    fn test_system_impact_assessment() {
        let refresh_error = BandwidthError::RefreshFailed {
            message: "Test error".to_string(),
            retry_attempts: 3,
        };
        assert_eq!(refresh_error.assess_system_impact(), SystemImpact::Critical);

        let counter_reset_error = BandwidthError::CounterReset {
            interface: "eth0".to_string(),
            current: 1000,
            previous: 2000,
        };
        assert_eq!(counter_reset_error.assess_system_impact(), SystemImpact::Low);

        let time_interval_error = BandwidthError::InvalidTimeInterval {
            interval_ms: 50,
            min_threshold_ms: 100,
        };
        assert_eq!(time_interval_error.assess_system_impact(), SystemImpact::Low);
    }

    #[test]
    fn test_user_friendly_messages() {
        let refresh_error = BandwidthError::RefreshFailed {
            message: "Network subsystem unavailable".to_string(),
            retry_attempts: 3,
        };

        let user_message = refresh_error.create_user_friendly_message();
        assert!(user_message.contains("Failed to refresh network statistics"));
        assert!(user_message.contains("Try:"));

        let counter_reset_error = BandwidthError::CounterReset {
            interface: "eth0".to_string(),
            current: 1000,
            previous: 2000,
        };

        let user_message = counter_reset_error.create_user_friendly_message();
        assert!(user_message.contains("Network counter reset detected"));
        assert!(user_message.contains("This is normal"));
    }

    #[test]
    fn test_suggested_actions() {
        let refresh_error = BandwidthError::RefreshFailed {
            message: "Test error".to_string(),
            retry_attempts: 3,
        };

        let actions = refresh_error.get_suggested_actions();
        assert!(!actions.is_empty());
        assert!(actions.iter().any(|action| action.contains("network service")));

        let interface_error = BandwidthError::InterfaceNotFound {
            interface: "eth0".to_string(),
        };

        let actions = interface_error.get_suggested_actions();
        assert!(actions.iter().any(|action| action.contains("eth0")));
    }

    #[test]
    fn test_error_serialization() {
        let error = BandwidthError::RefreshFailed {
            message: "Test error".to_string(),
            retry_attempts: 3,
        };

        // Test that the error can be serialized and deserialized
        let serialized = serde_json::to_string(&error).expect("Failed to serialize error");
        let deserialized: BandwidthError = serde_json::from_str(&serialized)
            .expect("Failed to deserialize error");

        match deserialized {
            BandwidthError::RefreshFailed { message, retry_attempts } => {
                assert_eq!(message, "Test error");
                assert_eq!(retry_attempts, 3);
            }
            _ => panic!("Deserialized error has wrong variant"),
        }
    }

    #[test]
    fn test_system_impact_serialization() {
        let impact = SystemImpact::Critical;
        let serialized = serde_json::to_string(&impact).expect("Failed to serialize SystemImpact");
        let deserialized: SystemImpact = serde_json::from_str(&serialized)
            .expect("Failed to deserialize SystemImpact");
        
        assert_eq!(deserialized, SystemImpact::Critical);
    }

    #[test]
    fn test_error_context_report_creation() {
        let error = BandwidthError::RefreshFailed {
            message: "Test network failure".to_string(),
            retry_attempts: 2,
        };

        let troubleshooting_report = TroubleshootingReport {
            timestamp: Utc::now(),
            collection_count: 5,
            system_info: SystemInfo {
                platform: "test_platform".to_string(),
                architecture: "test_arch".to_string(),
                available_interfaces: 3,
                uptime_info: "test_uptime".to_string(),
            },
            interface_diagnostics: InterfaceDiagnostics {
                total_interfaces: 3,
                active_interfaces: 2,
                failed_interfaces: 1,
                interface_status: vec![("eth0".to_string(), "active".to_string())],
            },
            collection_history: CollectionHistory {
                successful_collections: 4,
                failed_collections: 1,
                average_duration_ms: 150.0,
                recent_results: vec!["success".to_string(), "failure".to_string()],
            },
        };

        let context_report = error.create_error_context_report(troubleshooting_report);

        assert_eq!(context_report.error_type, "RefreshFailed");
        assert!(context_report.error_message.contains("Failed to refresh network statistics"));
        assert!(context_report.user_friendly_message.contains("Failed to refresh network statistics"));
        assert!(!context_report.suggested_actions.is_empty());
        assert_eq!(context_report.system_impact, SystemImpact::Critical);
    }

    #[test]
    fn test_error_logging_functions() {
        // Test BandwidthError logging
        let error = BandwidthError::RefreshFailed {
            message: "Test error".to_string(),
            retry_attempts: 3,
        };

        // This test verifies the function doesn't panic - actual log output would need integration testing
        error.log_error_event("test_context", 42);

        // Test anyhow error logging
        let anyhow_error = anyhow::anyhow!("Test anyhow error");
        log_error_event_anyhow(&anyhow_error, "test_context", 42);
    }

    #[test]
    fn test_success_event_logging() {
        // Test success event logging with various performance levels
        log_success_event(3, 50.0, 1000.0, 500.0, 10); // excellent performance
        log_success_event(5, 200.0, 2000.0, 1000.0, 11); // good performance
        log_success_event(2, 600.0, 500.0, 250.0, 12); // slow performance
    }

    #[test]
    fn test_all_error_variants_user_friendly_messages() {
        let errors = vec![
            BandwidthError::RefreshFailed {
                message: "Test".to_string(),
                retry_attempts: 3,
            },
            BandwidthError::InvalidTimeInterval {
                interval_ms: 50,
                min_threshold_ms: 100,
            },
            BandwidthError::CounterReset {
                interface: "eth0".to_string(),
                current: 1000,
                previous: 2000,
            },
            BandwidthError::NoInterfacesFound,
            BandwidthError::InterfaceNotFound {
                interface: "eth0".to_string(),
            },
            BandwidthError::SystemResourceError {
                message: "Test resource error".to_string(),
            },
            BandwidthError::DataValidationFailed {
                interface: "eth0".to_string(),
                validation_error: "Test validation error".to_string(),
            },
            BandwidthError::TimeAnomaly {
                description: "Test time anomaly".to_string(),
                current_time: Utc::now(),
                previous_time: Utc::now() - chrono::Duration::seconds(10),
            },
        ];

        for error in errors {
            let user_message = error.create_user_friendly_message();
            assert!(!user_message.is_empty(), "User-friendly message should not be empty for error: {:?}", error);
            
            let actions = error.get_suggested_actions();
            assert!(!actions.is_empty(), "Suggested actions should not be empty for error: {:?}", error);
            
            let impact = error.assess_system_impact();
            assert!(matches!(impact, SystemImpact::Critical | SystemImpact::High | SystemImpact::Medium | SystemImpact::Low), 
                   "System impact should be valid for error: {:?}", error);
        }
    }

    #[test]
    fn test_troubleshooting_report_serialization() {
        let report = TroubleshootingReport {
            timestamp: Utc::now(),
            collection_count: 10,
            system_info: SystemInfo {
                platform: "Linux".to_string(),
                architecture: "x86_64".to_string(),
                available_interfaces: 5,
                uptime_info: "5 days".to_string(),
            },
            interface_diagnostics: InterfaceDiagnostics {
                total_interfaces: 5,
                active_interfaces: 3,
                failed_interfaces: 2,
                interface_status: vec![
                    ("eth0".to_string(), "active".to_string()),
                    ("wlan0".to_string(), "inactive".to_string()),
                ],
            },
            collection_history: CollectionHistory {
                successful_collections: 8,
                failed_collections: 2,
                average_duration_ms: 125.5,
                recent_results: vec!["success".to_string(), "success".to_string(), "failure".to_string()],
            },
        };

        let serialized = serde_json::to_string(&report).expect("Failed to serialize TroubleshootingReport");
        let deserialized: TroubleshootingReport = serde_json::from_str(&serialized)
            .expect("Failed to deserialize TroubleshootingReport");

        assert_eq!(deserialized.collection_count, 10);
        assert_eq!(deserialized.system_info.platform, "Linux");
        assert_eq!(deserialized.interface_diagnostics.total_interfaces, 5);
        assert_eq!(deserialized.collection_history.successful_collections, 8);
    }

    #[test]
    fn test_error_context_report_serialization() {
        let error = BandwidthError::NoInterfacesFound;
        let troubleshooting_report = TroubleshootingReport {
            timestamp: Utc::now(),
            collection_count: 1,
            system_info: SystemInfo {
                platform: "Test".to_string(),
                architecture: "Test".to_string(),
                available_interfaces: 0,
                uptime_info: "Test".to_string(),
            },
            interface_diagnostics: InterfaceDiagnostics {
                total_interfaces: 0,
                active_interfaces: 0,
                failed_interfaces: 0,
                interface_status: vec![],
            },
            collection_history: CollectionHistory {
                successful_collections: 0,
                failed_collections: 1,
                average_duration_ms: 0.0,
                recent_results: vec![],
            },
        };

        let context_report = error.create_error_context_report(troubleshooting_report);
        
        let serialized = serde_json::to_string(&context_report).expect("Failed to serialize ErrorContextReport");
        let deserialized: ErrorContextReport = serde_json::from_str(&serialized)
            .expect("Failed to deserialize ErrorContextReport");

        assert_eq!(deserialized.error_type, "NoInterfacesFound");
        assert_eq!(deserialized.system_impact, SystemImpact::Critical);
    }
}