//! Reporting and diagnostics module for bandwidth collection
//!
//! This module provides comprehensive troubleshooting reports, error context reports,
//! interface analysis, and diagnostic information gathering for the bandwidth collector.

use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sysinfo::Networks;

use super::errors::{BandwidthError, SystemImpact};
use super::stats::BandwidthStats;
use crate::collectors::platform::interface_manager::{InterfaceManager, PlatformInterfaceInfo};

/// Comprehensive troubleshooting report for debugging bandwidth collection issues
#[derive(Debug, Serialize, Deserialize)]
pub struct TroubleshootingReport {
    /// Timestamp when the report was generated
    pub timestamp: DateTime<Utc>,
    /// Total number of collections performed
    pub collection_count: u64,
    /// System information for troubleshooting
    pub system_info: SystemInfo,
    /// Interface-specific diagnostic information
    pub interface_diagnostics: Vec<InterfaceDiagnostic>,
    /// Collection history and statistics
    pub collection_history: CollectionHistory,
    /// Current collector configuration
    pub configuration: CollectorConfiguration,
}

/// System information for troubleshooting
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system platform
    pub platform: String,
    /// System architecture
    pub architecture: String,
    /// Rust version used to compile
    pub rust_version: String,
}

/// Interface-specific diagnostic information
#[derive(Debug, Serialize, Deserialize)]
pub struct InterfaceDiagnostic {
    /// Interface name
    pub name: String,
    /// Whether the interface appears to be up (has traffic)
    pub is_up: bool,
    /// Total bytes received
    pub bytes_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Whether we have cached data for this interface
    pub has_cached_data: bool,
    /// Number of consecutive failures for this interface
    pub consecutive_failures: u32,
    /// Time since last successful update
    pub time_since_last_update: Option<f64>,
}

/// Collection history for troubleshooting
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionHistory {
    /// Total number of collections performed
    pub total_collections: u64,
    /// Number of interfaces that have experienced failures
    pub interfaces_with_failures: usize,
    /// Average collection duration in milliseconds
    pub average_collection_duration_ms: f64,
}

/// Collector configuration for troubleshooting
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectorConfiguration {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Delay between retry attempts in milliseconds
    pub retry_delay_ms: u64,
    /// Minimum time threshold for calculations
    pub min_time_threshold: f64,
}

/// Comprehensive error context report for detailed debugging
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorContextReport {
    /// Type of error that occurred
    pub error_type: String,
    /// Original error message
    pub error_message: String,
    /// User-friendly error message
    pub user_friendly_message: String,
    /// Comprehensive troubleshooting report
    pub troubleshooting_report: TroubleshootingReport,
    /// Suggested actions to resolve the error
    pub suggested_actions: Vec<String>,
    /// Assessment of system impact
    pub system_impact: SystemImpact,
}

/// Summary report of interface analysis and filtering
#[derive(Debug, Serialize, Deserialize)]
pub struct InterfaceSummaryReport {
    /// Timestamp when the report was generated
    pub timestamp: DateTime<Utc>,
    /// Platform information
    pub platform: String,
    /// Total number of interfaces found
    pub total_interfaces: usize,
    /// Number of default interfaces
    pub default_interfaces: usize,
    /// Number of important interfaces
    pub important_interfaces: usize,
    /// Number of relevant interfaces
    pub relevant_interfaces: usize,
    /// Detailed information about all interfaces
    pub interface_details: Vec<PlatformInterfaceInfo>,
    /// Interface manager cache statistics
    pub cache_size: usize,
}

/// Reporting functionality for bandwidth collector
pub struct BandwidthReporter {
    /// Collection count for tracking
    collection_count: u64,
    /// Previous statistics cache
    previous_stats: HashMap<String, (u64, u64, DateTime<Utc>, u32)>,
    /// Interface manager for analysis
    interface_manager: InterfaceManager,
    /// Network interfaces
    networks: Networks,
    /// Configuration
    max_retries: u32,
    retry_delay_ms: u64,
    min_time_threshold: f64,
}

impl BandwidthReporter {
    /// Creates a new bandwidth reporter
    pub fn new(
        collection_count: u64,
        previous_stats: HashMap<String, (u64, u64, DateTime<Utc>, u32)>,
        interface_manager: InterfaceManager,
        max_retries: u32,
        retry_delay_ms: u64,
        min_time_threshold: f64,
    ) -> Self {
        Self {
            collection_count,
            previous_stats,
            interface_manager,
            networks: Networks::new(), // Create a new empty Networks instance
            max_retries,
            retry_delay_ms,
            min_time_threshold,
        }
    }

    /// Creates a comprehensive troubleshooting report for debugging collection issues
    pub fn create_troubleshooting_report(&self, networks: &Networks) -> TroubleshootingReport {
        debug!(
            "Creating troubleshooting report for collection #{}",
            self.collection_count
        );

        let system_info = self.gather_system_info();
        let interface_diagnostics = self.gather_interface_diagnostics(networks);
        let collection_history = self.gather_collection_history();

        TroubleshootingReport {
            timestamp: Utc::now(),
            collection_count: self.collection_count,
            system_info,
            interface_diagnostics,
            collection_history,
            configuration: CollectorConfiguration {
                max_retries: self.max_retries,
                retry_delay_ms: self.retry_delay_ms,
                min_time_threshold: self.min_time_threshold,
            },
        }
    }

    /// Creates a detailed error context report for specific errors
    /// This provides additional debugging information beyond the standard error message
    pub fn create_error_context_report(
        &self,
        error: &BandwidthError,
        networks: &Networks,
    ) -> ErrorContextReport {
        debug!(
            "Creating error context report for collection #{}: {:?}",
            self.collection_count, error
        );

        let troubleshooting_report = self.create_troubleshooting_report(networks);

        ErrorContextReport {
            error_type: format!("{:?}", error),
            error_message: error.to_string(),
            user_friendly_message: self.create_user_friendly_error_message(error),
            troubleshooting_report,
            suggested_actions: self.get_suggested_actions(error),
            system_impact: self.assess_system_impact(error),
        }
    }

    /// Creates a summary report of interface analysis and filtering
    pub fn create_interface_summary_report(
        &mut self,
        all_interface_info: Vec<PlatformInterfaceInfo>,
    ) -> InterfaceSummaryReport {
        debug!(
            "Creating interface summary report for collection #{}",
            self.collection_count
        );

        let total_interfaces = all_interface_info.len();
        let interface_names: Vec<String> = all_interface_info
            .iter()
            .map(|info| info.name.clone())
            .collect();

        let default_interfaces = self
            .interface_manager
            .get_default_interfaces(&interface_names);
        let important_interfaces = self
            .interface_manager
            .get_important_interfaces(&interface_names);
        let relevant_interfaces = self
            .interface_manager
            .get_relevant_interfaces(&interface_names);

        let (cache_size, platform) = self.interface_manager.get_cache_stats();

        InterfaceSummaryReport {
            timestamp: Utc::now(),
            platform: format!("{:?}", platform),
            total_interfaces,
            default_interfaces: default_interfaces.len(),
            important_interfaces: important_interfaces.len(),
            relevant_interfaces: relevant_interfaces.len(),
            interface_details: all_interface_info,
            cache_size,
        }
    }

    /// Exports comprehensive error logs and system information for support purposes
    /// This creates a detailed report that can be shared with support teams
    pub fn export_support_report(
        &self,
        error: Option<&BandwidthError>,
        networks: &Networks,
    ) -> Result<String> {
        info!(
            "Exporting support report for collection #{}",
            self.collection_count
        );

        let mut report = String::new();

        // Header
        report.push_str("=== KAIPO WATCHER BANDWIDTH COLLECTOR SUPPORT REPORT ===\n");
        report.push_str(&format!(
            "Generated: {}\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        report.push_str(&format!("Collection Count: {}\n", self.collection_count));
        report.push_str("\n");

        // Error information if provided
        if let Some(error) = error {
            report.push_str("=== ERROR INFORMATION ===\n");
            let error_context = self.create_error_context_report(error, networks);
            report.push_str(&format!("Error Type: {}\n", error_context.error_type));
            report.push_str(&format!("Error Message: {}\n", error_context.error_message));
            report.push_str(&format!(
                "User-Friendly Message: {}\n",
                error_context.user_friendly_message
            ));
            report.push_str(&format!(
                "System Impact: {:?}\n",
                error_context.system_impact
            ));
            report.push_str("\nSuggested Actions:\n");
            for (i, action) in error_context.suggested_actions.iter().enumerate() {
                report.push_str(&format!("  {}. {}\n", i + 1, action));
            }
            report.push_str("\n");
        }

        // System information
        let troubleshooting_report = self.create_troubleshooting_report(networks);
        report.push_str("=== SYSTEM INFORMATION ===\n");
        report.push_str(&format!(
            "Platform: {}\n",
            troubleshooting_report.system_info.platform
        ));
        report.push_str(&format!(
            "Architecture: {}\n",
            troubleshooting_report.system_info.architecture
        ));
        report.push_str(&format!(
            "Rust Version: {}\n",
            troubleshooting_report.system_info.rust_version
        ));
        report.push_str("\n");

        // Configuration
        report.push_str("=== COLLECTOR CONFIGURATION ===\n");
        report.push_str(&format!(
            "Max Retries: {}\n",
            troubleshooting_report.configuration.max_retries
        ));
        report.push_str(&format!(
            "Retry Delay: {}ms\n",
            troubleshooting_report.configuration.retry_delay_ms
        ));
        report.push_str(&format!(
            "Min Time Threshold: {:.3}s\n",
            troubleshooting_report.configuration.min_time_threshold
        ));
        report.push_str("\n");

        // Collection history
        report.push_str("=== COLLECTION HISTORY ===\n");
        report.push_str(&format!(
            "Total Collections: {}\n",
            troubleshooting_report.collection_history.total_collections
        ));
        report.push_str(&format!(
            "Interfaces with Failures: {}\n",
            troubleshooting_report
                .collection_history
                .interfaces_with_failures
        ));
        report.push_str(&format!(
            "Average Duration: {:.3}ms\n",
            troubleshooting_report
                .collection_history
                .average_collection_duration_ms
        ));
        report.push_str("\n");

        // Interface diagnostics
        report.push_str("=== INTERFACE DIAGNOSTICS ===\n");
        for diagnostic in &troubleshooting_report.interface_diagnostics {
            report.push_str(&format!("Interface: {}\n", diagnostic.name));
            report.push_str(&format!(
                "  Status: {}\n",
                if diagnostic.is_up { "UP" } else { "DOWN" }
            ));
            report.push_str(&format!("  RX Bytes: {}\n", diagnostic.bytes_received));
            report.push_str(&format!("  TX Bytes: {}\n", diagnostic.bytes_sent));
            report.push_str(&format!("  Has Cache: {}\n", diagnostic.has_cached_data));
            report.push_str(&format!(
                "  Failures: {}\n",
                diagnostic.consecutive_failures
            ));
            if let Some(time_since_update) = diagnostic.time_since_last_update {
                report.push_str(&format!("  Last Update: {:.3}s ago\n", time_since_update));
            }
            report.push_str("\n");
        }

        report.push_str("=== END OF REPORT ===\n");

        Ok(report)
    }

    /// Logs a structured error event for monitoring and debugging
    pub fn log_error_event(&self, error: &anyhow::Error, context: &str) {
        error!(
            "Bandwidth collector error event for collection #{} (context={}, error={}, system_impact=bandwidth_monitoring_degraded)",
            self.collection_count, context, error
        );
    }

    /// Logs a structured success event for monitoring and performance tracking
    pub fn log_success_event(&self, stats: &[BandwidthStats], duration_ms: f64) {
        let total_download_bps: f64 = stats.iter().map(|s| s.download_speed_bps).sum();
        let total_upload_bps: f64 = stats.iter().map(|s| s.upload_speed_bps).sum();

        info!(
            "Bandwidth collection #{} success event (interfaces={}, duration={:.3}ms, total_download={:.2}bps, total_upload={:.2}bps, performance={})",
            self.collection_count,
            stats.len(),
            duration_ms,
            total_download_bps,
            total_upload_bps,
            if duration_ms < 100.0 {
                "excellent"
            } else if duration_ms < 500.0 {
                "good"
            } else {
                "slow"
            }
        );
    }

    /// Creates user-friendly error messages for common failure scenarios
    pub fn create_user_friendly_error_message(&self, error: &BandwidthError) -> String {
        match error {
            BandwidthError::RefreshFailed { message, retry_attempts } => {
                format!(
                    "Failed to refresh network statistics after {} attempts. This usually indicates a system-level networking issue.\n\nTry:\n• Checking if network interfaces are properly configured\n• Running with administrator/root privileges\n• Restarting the network service\n• Checking system logs for network-related errors\n\nTechnical details: {}",
                    retry_attempts + 1,
                    message
                )
            }
            BandwidthError::NoInterfacesFound => {
                "No network interfaces were found on your system. This is unusual and may indicate a serious system configuration issue.\n\nTry:\n• Checking if network drivers are properly installed\n• Running 'ip link' (Linux) or 'ifconfig' (macOS) to verify interfaces exist\n• Restarting the network service\n• Running with administrator/root privileges".to_string()
            }
            BandwidthError::InterfaceNotFound { interface } => {
                format!(
                    "The network interface '{}' was not found or is no longer available. This can happen when interfaces are disconnected or renamed.\n\nTry:\n• Checking available interfaces with 'ip link' (Linux) or 'ifconfig' (macOS)\n• Reconnecting the network interface\n• Using a different interface name",
                    interface
                )
            }
            BandwidthError::CounterReset { interface, .. } => {
                format!(
                    "Network counter reset detected for interface '{}'. This is normal after interface restarts, system resume, or driver reloads.\n\nThis is typically not a problem - the system will automatically recover on the next measurement.",
                    interface
                )
            }
            BandwidthError::InvalidTimeInterval { interval_ms, min_threshold_ms } => {
                format!(
                    "Time interval too small for reliable speed calculation: {}ms (minimum: {}ms). This can happen with very frequent measurements or system clock issues.\n\nTry:\n• Increasing the time between measurements\n• Checking system clock stability",
                    interval_ms, min_threshold_ms
                )
            }
            BandwidthError::SystemResourceError { message } => {
                format!(
                    "System resource error occurred. This usually indicates insufficient permissions or system resource constraints.\n\nTry:\n• Running with administrator/root privileges\n• Checking system resource usage (CPU, memory)\n• Closing other network monitoring tools\n\nTechnical details: {}",
                    message
                )
            }
            BandwidthError::DataValidationFailed { interface, validation_error } => {
                format!(
                    "Data validation failed for interface '{}'. This indicates potentially corrupted or inconsistent network statistics.\n\nTry:\n• Restarting the network interface\n• Checking for driver issues\n• Running network diagnostics\n\nValidation error: {}",
                    interface, validation_error
                )
            }
            BandwidthError::TimeAnomaly { description, .. } => {
                format!(
                    "Time anomaly detected: {}. This can happen after system suspend/resume, clock changes, or NTP synchronization.\n\nThis is typically temporary - the system will automatically recover on the next measurement.",
                    description
                )
            }
        }
    }

    /// Provides specific suggested actions based on error type
    pub fn get_suggested_actions(&self, error: &BandwidthError) -> Vec<String> {
        match error {
            BandwidthError::RefreshFailed { retry_attempts, .. } => {
                let mut actions = vec![
                    "Check network interface configuration".to_string(),
                    "Verify network drivers are properly installed".to_string(),
                    "Run with administrator/root privileges".to_string(),
                ];
                if *retry_attempts >= 3 {
                    actions.push("Consider restarting the network service".to_string());
                    actions.push("Check system logs for network-related errors".to_string());
                }
                actions
            }
            BandwidthError::NoInterfacesFound => vec![
                "Verify network interfaces exist using system tools".to_string(),
                "Check network driver installation".to_string(),
                "Run with administrator/root privileges".to_string(),
                "Restart network services".to_string(),
            ],
            BandwidthError::InterfaceNotFound { .. } => vec![
                "List available interfaces using system tools".to_string(),
                "Check if interface is connected".to_string(),
                "Verify interface name spelling".to_string(),
                "Try using a different interface".to_string(),
            ],
            BandwidthError::CounterReset { .. } => vec![
                "Wait for next measurement cycle".to_string(),
                "This is typically a temporary condition".to_string(),
            ],
            BandwidthError::InvalidTimeInterval { .. } => vec![
                "Increase time between measurements".to_string(),
                "Check system clock stability".to_string(),
                "Verify NTP synchronization".to_string(),
            ],
            BandwidthError::SystemResourceError { .. } => vec![
                "Run with administrator/root privileges".to_string(),
                "Check available system resources".to_string(),
                "Close other network monitoring tools".to_string(),
                "Restart the application".to_string(),
            ],
            BandwidthError::DataValidationFailed { .. } => vec![
                "Restart the network interface".to_string(),
                "Check for network driver issues".to_string(),
                "Run network diagnostics".to_string(),
                "Update network drivers".to_string(),
            ],
            BandwidthError::TimeAnomaly { .. } => vec![
                "Wait for next measurement cycle".to_string(),
                "Check system clock synchronization".to_string(),
                "This is typically a temporary condition".to_string(),
            ],
        }
    }

    /// Assesses the system impact of different error types
    pub fn assess_system_impact(&self, error: &BandwidthError) -> SystemImpact {
        match error {
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

    /// Gathers system information for troubleshooting
    pub fn gather_system_info(&self) -> SystemInfo {
        SystemInfo {
            platform: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            rust_version: std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()),
        }
    }

    /// Gathers interface-specific diagnostics
    pub fn gather_interface_diagnostics(&self, networks: &Networks) -> Vec<InterfaceDiagnostic> {
        let mut diagnostics = Vec::new();

        for (interface_name, network) in networks.iter() {
            let cached_data = self.previous_stats.get(interface_name);

            diagnostics.push(InterfaceDiagnostic {
                name: interface_name.to_string(),
                is_up: network.received() > 0 || network.transmitted() > 0,
                bytes_received: network.received(),
                bytes_sent: network.transmitted(),
                has_cached_data: cached_data.is_some(),
                consecutive_failures: cached_data
                    .map(|(_, _, _, failures)| *failures)
                    .unwrap_or(0),
                time_since_last_update: cached_data.map(|(_, _, prev_time, _)| {
                    (Utc::now() - *prev_time).num_milliseconds() as f64 / 1000.0
                }),
            });
        }

        diagnostics
    }

    /// Gathers collection history for troubleshooting
    pub fn gather_collection_history(&self) -> CollectionHistory {
        CollectionHistory {
            total_collections: self.collection_count,
            interfaces_with_failures: self
                .previous_stats
                .values()
                .filter(|(_, _, _, failures)| *failures > 0)
                .count(),
            average_collection_duration_ms: 150.0, // Placeholder - would need actual tracking
        }
    }
}
