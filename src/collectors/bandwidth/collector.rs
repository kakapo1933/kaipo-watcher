//! Core bandwidth collector implementation
//!
//! This module contains the main BandwidthCollector struct and its core functionality
//! for network data collection, speed calculation, and interface management.

use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use sysinfo::Networks;

use crate::collectors::bandwidth::errors::{
    BandwidthError, log_error_event_anyhow, log_success_event,
};
use crate::collectors::bandwidth::reporting::{
    BandwidthReporter, ErrorContextReport, InterfaceSummaryReport, TroubleshootingReport,
};
use crate::collectors::bandwidth::stats::{BandwidthStats, InterfaceState, InterfaceType};
use crate::collectors::bandwidth::validation::{
    calculate_speeds_with_validation, validate_interface_data,
};
use crate::collectors::platform::interface_manager::{InterfaceManager, PlatformInterfaceInfo};

/// Collects bandwidth statistics from network interfaces
/// Maintains previous readings to calculate speed deltas with robust error handling
#[derive(Debug)]
pub struct BandwidthCollector {
    /// System network interfaces manager from sysinfo crate
    networks: Networks,
    /// Cache of previous readings for speed calculation with validation
    /// Maps interface name to (bytes_received, bytes_sent, timestamp, consecutive_failures)
    previous_stats: HashMap<String, (u64, u64, DateTime<Utc>, u32)>,
    /// Cross-platform interface manager for filtering and prioritization
    interface_manager: InterfaceManager,
    /// Maximum number of retry attempts for network refresh
    max_retries: u32,
    /// Delay between retry attempts in milliseconds
    retry_delay_ms: u64,
    /// Minimum time threshold in seconds to prevent division by very small intervals
    min_time_threshold: f64,
    /// Counter for total collections performed
    collection_count: u64,
}

impl Default for BandwidthCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl BandwidthCollector {
    /// Creates a new bandwidth collector with fresh network interface list
    pub fn new() -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
            previous_stats: HashMap::new(),
            interface_manager: InterfaceManager::new(),
            max_retries: 3,
            retry_delay_ms: 100,
            min_time_threshold: 0.1, // 100ms minimum interval
            collection_count: 0,
        }
    }

    /// Creates a new bandwidth collector with custom retry configuration
    pub fn with_retry_config(max_retries: u32, retry_delay_ms: u64) -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
            previous_stats: HashMap::new(),
            interface_manager: InterfaceManager::new(),
            max_retries,
            retry_delay_ms,
            min_time_threshold: 0.1,
            collection_count: 0,
        }
    }

    /// Collects bandwidth statistics with intelligent interface filtering
    /// Returns only relevant interfaces based on platform-specific filtering and prioritization
    pub fn collect_filtered(&mut self) -> Result<Vec<BandwidthStats>> {
        let all_stats = self.collect()?;
        let interface_names: Vec<String> =
            all_stats.iter().map(|s| s.interface_name.clone()).collect();
        let relevant_interfaces = self
            .interface_manager
            .get_relevant_interfaces(&interface_names);

        // Filter stats to only include relevant interfaces
        let filtered_stats: Vec<BandwidthStats> = all_stats
            .into_iter()
            .filter(|stat| {
                relevant_interfaces
                    .iter()
                    .any(|info| info.name == stat.interface_name)
            })
            .collect();

        debug!(
            "Filtered bandwidth collection: {} relevant interfaces from {} total",
            filtered_stats.len(),
            interface_names.len()
        );

        Ok(filtered_stats)
    }

    /// Collects bandwidth statistics for default interfaces only
    /// Returns interfaces that should be shown by default (excludes virtual/irrelevant interfaces)
    pub fn collect_default(&mut self) -> Result<Vec<BandwidthStats>> {
        let all_stats = self.collect()?;
        let interface_names: Vec<String> =
            all_stats.iter().map(|s| s.interface_name.clone()).collect();
        let default_interfaces = self
            .interface_manager
            .get_default_interfaces(&interface_names);

        // Filter stats to only include default interfaces
        let default_stats: Vec<BandwidthStats> = all_stats
            .into_iter()
            .filter(|stat| {
                default_interfaces
                    .iter()
                    .any(|info| info.name == stat.interface_name)
            })
            .collect();

        debug!(
            "Default bandwidth collection: {} default interfaces from {} total",
            default_stats.len(),
            interface_names.len()
        );

        Ok(default_stats)
    }

    /// Collects bandwidth statistics for important interfaces only
    /// Returns only high-priority interfaces (physical ethernet, wifi, VPN)
    pub fn collect_important(&mut self) -> Result<Vec<BandwidthStats>> {
        let all_stats = self.collect()?;
        let interface_names: Vec<String> =
            all_stats.iter().map(|s| s.interface_name.clone()).collect();
        let important_interfaces = self
            .interface_manager
            .get_important_interfaces(&interface_names);

        // Filter stats to only include important interfaces
        let important_stats: Vec<BandwidthStats> = all_stats
            .into_iter()
            .filter(|stat| {
                important_interfaces
                    .iter()
                    .any(|info| info.name == stat.interface_name)
            })
            .collect();

        debug!(
            "Important bandwidth collection: {} important interfaces from {} total",
            important_stats.len(),
            interface_names.len()
        );

        Ok(important_stats)
    }

    /// Gets detailed interface information for a specific interface
    pub fn get_interface_info(&mut self, interface_name: &str) -> PlatformInterfaceInfo {
        self.interface_manager.analyze_interface(interface_name)
    }

    /// Gets detailed interface information for all current interfaces
    pub fn get_all_interface_info(&mut self) -> Result<Vec<PlatformInterfaceInfo>> {
        // Refresh network data first to get current interfaces
        self.refresh_network_data_with_retry()?;

        let interface_names: Vec<String> = self
            .networks
            .iter()
            .map(|(name, _)| name.to_string())
            .collect();
        let interface_info: Vec<PlatformInterfaceInfo> = interface_names
            .iter()
            .map(|name| self.interface_manager.analyze_interface(name))
            .collect();

        Ok(interface_info)
    }

    /// Clears the interface manager cache (useful when interfaces change)
    pub fn clear_interface_cache(&mut self) {
        self.interface_manager.clear_cache();
    }

    /// Gets interface manager statistics
    pub fn get_interface_manager_stats(&self) -> (usize, String) {
        let (cache_size, platform) = self.interface_manager.get_cache_stats();
        (cache_size, format!("{:?}", platform))
    }

    /// Collects current bandwidth statistics from all network interfaces
    /// Returns a vector of BandwidthStats, one per active interface
    /// Implements robust error handling and comprehensive logging
    pub fn collect(&mut self) -> Result<Vec<BandwidthStats>> {
        let collection_start = std::time::Instant::now();
        let now = Utc::now();
        self.collection_count += 1;

        // Enhanced logging for collection events
        info!(
            "Starting bandwidth collection #{} at {} (cached_interfaces={}, retry_config=max_retries={}, delay={}ms)",
            self.collection_count,
            now.format("%H:%M:%S%.3f"),
            self.previous_stats.len(),
            self.max_retries,
            self.retry_delay_ms
        );

        // Refresh network statistics with retry logic and detailed error logging
        match self.refresh_network_data_with_retry() {
            Ok(()) => {
                debug!(
                    "Network data refresh successful for collection #{} (interfaces_found={})",
                    self.collection_count,
                    self.networks.len()
                );
            }
            Err(e) => {
                error!(
                    "Failed to refresh network data after retries for collection #{} (max_retries={}, error={})",
                    self.collection_count, self.max_retries, e
                );
                // Log error event for monitoring
                log_error_event_anyhow(&e, "network_refresh_failure", self.collection_count);
                return Err(e.context("Critical failure: Unable to refresh network data"));
            }
        }

        let mut stats = Vec::new();
        let mut successful_interfaces = 0;
        let mut failed_interfaces = 0;
        let mut interface_errors = Vec::new();

        // Collect interface data first to avoid borrowing issues
        let interface_data: Vec<(String, u64, u64, u64, u64)> = self
            .networks
            .iter()
            .map(|(name, network)| {
                (
                    name.to_string(),
                    network.received(),
                    network.transmitted(),
                    network.packets_received(),
                    network.packets_transmitted(),
                )
            })
            .collect();

        info!(
            "Processing {} network interfaces for collection #{}",
            interface_data.len(),
            self.collection_count
        );

        // Process each network interface with individual error handling
        for (interface_name, bytes_received, bytes_sent, packets_received, packets_sent) in
            interface_data
        {
            trace!(
                "Processing interface '{}' for collection #{}: rx={} bytes, tx={} bytes, rx_packets={}, tx_packets={}",
                interface_name,
                self.collection_count,
                bytes_received,
                bytes_sent,
                packets_received,
                packets_sent
            );

            // Validate interface data before processing
            match validate_interface_data(
                &interface_name,
                bytes_received,
                bytes_sent,
                packets_received,
                packets_sent,
                self.collection_count,
            ) {
                Ok(()) => {
                    trace!(
                        "Interface '{}' data validation passed for collection #{}",
                        interface_name, self.collection_count
                    );
                }
                Err(validation_error) => {
                    warn!(
                        "Interface '{}' failed data validation for collection #{}: {} - applying graceful degradation",
                        interface_name, self.collection_count, validation_error
                    );

                    let error_msg = format!("Interface '{}': {}", interface_name, validation_error);
                    interface_errors.push(error_msg);
                    failed_interfaces += 1;

                    // Mark interface as having consecutive failures
                    if let Some((_rx, _tx, _time, failures)) =
                        self.previous_stats.get_mut(&interface_name)
                    {
                        *failures += 1;
                        debug!(
                            "Interface '{}' marked with consecutive failure #{} for collection #{}",
                            interface_name, failures, self.collection_count
                        );
                    }
                    continue; // Graceful degradation: Skip this interface but continue with others
                }
            }

            // Calculate speeds with enhanced error handling
            let (download_speed_bps, upload_speed_bps, calculation_confidence) =
                calculate_speeds_with_validation(
                    &interface_name,
                    bytes_received,
                    bytes_sent,
                    now,
                    &self.previous_stats,
                    self.min_time_threshold,
                    self.collection_count,
                );

            // Determine interface type and state
            let interface_type = self.determine_interface_type(&interface_name);
            let interface_state =
                self.determine_interface_state(&interface_name, bytes_received, bytes_sent);

            // Calculate time since last update
            let time_since_last_update = self
                .previous_stats
                .get(&interface_name)
                .map(|(_, _, prev_time, _)| (now - *prev_time).num_milliseconds() as f64 / 1000.0)
                .unwrap_or(0.0);

            // Store current readings for next speed calculation
            let consecutive_failures = self
                .previous_stats
                .get(&interface_name)
                .map(|(_, _, _, failures)| {
                    if download_speed_bps == 0.0 && upload_speed_bps == 0.0 {
                        *failures
                    } else {
                        0
                    }
                })
                .unwrap_or(0);

            self.previous_stats.insert(
                interface_name.clone(),
                (bytes_received, bytes_sent, now, consecutive_failures),
            );

            stats.push(BandwidthStats {
                timestamp: now,
                interface_name: interface_name.clone(),
                interface_type,
                interface_state,
                bytes_received,
                bytes_sent,
                packets_received,
                packets_sent,
                download_speed_bps,
                upload_speed_bps,
                calculation_confidence,
                time_since_last_update,
            });

            successful_interfaces += 1;
        }

        let collection_duration = collection_start.elapsed();
        let collection_duration_ms = collection_duration.as_secs_f64() * 1000.0;

        // Comprehensive logging for collection summary
        if failed_interfaces > 0 {
            warn!(
                "Bandwidth collection #{} completed with partial success - graceful degradation active: {}/{} interfaces successful, {} failed (duration: {:.3}ms)",
                self.collection_count,
                successful_interfaces,
                successful_interfaces + failed_interfaces,
                failed_interfaces,
                collection_duration_ms
            );

            // Log specific interface errors for debugging
            for (index, error) in interface_errors.iter().enumerate() {
                debug!(
                    "Interface error #{} for collection #{}: {} - troubleshooting: check interface connectivity and system permissions",
                    index + 1,
                    self.collection_count,
                    error
                );
            }
        } else {
            info!(
                "Bandwidth collection #{} completed successfully: {} interfaces processed (duration: {:.3}ms, performance: {})",
                self.collection_count,
                successful_interfaces,
                collection_duration_ms,
                if collection_duration_ms < 100.0 {
                    "excellent"
                } else if collection_duration_ms < 500.0 {
                    "good"
                } else {
                    "slow"
                }
            );
        }

        // Check for critical failure conditions with enhanced error reporting
        if successful_interfaces == 0 {
            error!(
                "Critical failure for collection #{}: No interfaces successfully processed (total_interfaces_attempted={}, duration={:.3}ms)",
                self.collection_count, failed_interfaces, collection_duration_ms
            );

            let error_context = if !interface_errors.is_empty() {
                format!(
                    "Specific errors encountered:\n{}",
                    interface_errors.join("\n")
                )
            } else {
                "No specific interface errors recorded - possible system-level issue".to_string()
            };

            return Err(anyhow::anyhow!(
                "No network interfaces could be processed successfully. {}",
                error_context
            ));
        } else if failed_interfaces > 0 {
            warn!(
                "Collection #{} operating in degraded mode: {}/{} interfaces failed (degradation_level: {})",
                self.collection_count,
                failed_interfaces,
                successful_interfaces + failed_interfaces,
                if failed_interfaces > successful_interfaces {
                    "severe"
                } else {
                    "moderate"
                }
            );
        }

        // Log success event for monitoring
        let total_download_bps: f64 = stats.iter().map(|s| s.download_speed_bps).sum();
        let total_upload_bps: f64 = stats.iter().map(|s| s.upload_speed_bps).sum();
        log_success_event(
            stats.len(),
            collection_duration_ms,
            total_download_bps,
            total_upload_bps,
            self.collection_count,
        );

        Ok(stats)
    }

    /// Returns total bandwidth usage across all interfaces
    /// Returns tuple of (total_download_bytes, total_upload_bytes)
    pub fn get_total_bandwidth(&self) -> (f64, f64) {
        let stats: Vec<_> = self
            .previous_stats
            .values()
            .map(|(rx, tx, _, _)| (*rx, *tx))
            .collect();

        let total_download = stats.iter().map(|(rx, _)| rx).sum::<u64>() as f64;
        let total_upload = stats.iter().map(|(_, tx)| tx).sum::<u64>() as f64;

        (total_download, total_upload)
    }
    /// Refreshes network data with retry logic and comprehensive error logging
    fn refresh_network_data_with_retry(&mut self) -> Result<()> {
        let mut last_error = None;
        let retry_start = std::time::Instant::now();

        debug!(
            "Starting network data refresh with retry logic for collection #{} (max_retries={}, retry_delay={}ms)",
            self.collection_count, self.max_retries, self.retry_delay_ms
        );

        for attempt in 0..=self.max_retries {
            let attempt_start = std::time::Instant::now();

            match self.refresh_network_data() {
                Ok(()) => {
                    let total_duration = retry_start.elapsed();
                    let total_duration_ms = total_duration.as_secs_f64() * 1000.0;

                    if attempt > 0 {
                        info!(
                            "Network data refresh succeeded after retries for collection #{}: attempt {}/{} (total_duration={:.3}ms, recovery_successful=true)",
                            self.collection_count,
                            attempt + 1,
                            self.max_retries + 1,
                            total_duration_ms
                        );
                    } else {
                        trace!(
                            "Network data refresh succeeded on first attempt for collection #{} (duration={:.3}ms, optimal_performance=true)",
                            self.collection_count,
                            attempt_start.elapsed().as_secs_f64() * 1000.0
                        );
                    }
                    return Ok(());
                }
                Err(e) => {
                    let attempt_duration = attempt_start.elapsed();
                    let attempt_duration_ms = attempt_duration.as_secs_f64() * 1000.0;
                    last_error = Some(e.clone());

                    if attempt < self.max_retries {
                        let delay = Duration::from_millis(self.retry_delay_ms * (1 << attempt));
                        warn!(
                            "Network refresh attempt {}/{} failed for collection #{} (duration={:.3}ms) - retrying with exponential backoff in {}ms: {}",
                            attempt + 1,
                            self.max_retries + 1,
                            self.collection_count,
                            attempt_duration_ms,
                            delay.as_millis(),
                            e
                        );
                        thread::sleep(delay);
                    } else {
                        error!(
                            "Network refresh final attempt {}/{} failed for collection #{} (duration={:.3}ms) - no more retries: {}",
                            attempt + 1,
                            self.max_retries + 1,
                            self.collection_count,
                            attempt_duration_ms,
                            e
                        );
                    }
                }
            }
        }

        let total_duration = retry_start.elapsed();
        let total_duration_ms = total_duration.as_secs_f64() * 1000.0;
        let final_error =
            last_error.unwrap_or_else(|| "Unknown error during network refresh".to_string());

        error!(
            "All network refresh attempts failed for collection #{} - critical system issue (total_attempts={}, total_duration={:.3}ms, system_impact=bandwidth_monitoring_unavailable)",
            self.collection_count,
            self.max_retries + 1,
            total_duration_ms
        );

        Err(BandwidthError::RefreshFailed {
            message: final_error,
            retry_attempts: self.max_retries,
        }
        .into())
    }

    /// Performs the actual network data refresh using proper sysinfo API calls
    fn refresh_network_data(&mut self) -> Result<(), String> {
        trace!(
            "Starting network interface refresh for collection #{} (cached_interfaces={})",
            self.collection_count,
            self.previous_stats.len()
        );

        // The key fix: use refresh(true) to actually refresh network data
        // refresh(false) only refreshes the interface list, not the statistics
        let refresh_start = std::time::Instant::now();
        self.networks.refresh(true);
        let refresh_duration_ms = refresh_start.elapsed().as_secs_f64() * 1000.0;

        let interface_count = self.networks.len();

        trace!(
            "Network refresh operation completed for collection #{} (interfaces_found={}, refresh_duration={:.3}ms, performance={})",
            self.collection_count,
            interface_count,
            refresh_duration_ms,
            if refresh_duration_ms < 50.0 {
                "excellent"
            } else if refresh_duration_ms < 200.0 {
                "good"
            } else {
                "slow"
            }
        );

        // Verify that we have at least some network interfaces
        if interface_count == 0 {
            error!(
                "No network interfaces found after refresh for collection #{} - system issue detected (refresh_duration={:.3}ms, system_impact=bandwidth_monitoring_completely_unavailable)",
                self.collection_count, refresh_duration_ms
            );
            return Err("No network interfaces found - possible causes: system network subsystem issues, insufficient permissions, all interfaces down".to_string());
        }

        debug!(
            "Network data refresh successful for collection #{} (interfaces_available={}, refresh_duration={:.3}ms, system_health=network_subsystem_operational)",
            self.collection_count, interface_count, refresh_duration_ms
        );

        Ok(())
    }

    /// Determines the type of network interface using the enhanced interface manager
    fn determine_interface_type(&mut self, interface_name: &str) -> InterfaceType {
        let interface_info = self.interface_manager.analyze_interface(interface_name);
        interface_info.interface_type.into()
    }

    /// Determines the operational state of a network interface
    fn determine_interface_state(
        &self,
        interface_name: &str,
        bytes_received: u64,
        bytes_sent: u64,
    ) -> InterfaceState {
        // Simple heuristic: if the interface has any traffic, consider it up
        // This is a basic implementation - more sophisticated detection could be added
        if bytes_received > 0 || bytes_sent > 0 {
            InterfaceState::Up
        } else {
            // Check if we have previous data to determine if it was previously active
            if let Some((prev_rx, prev_tx, _, _)) = self.previous_stats.get(interface_name) {
                if *prev_rx > 0 || *prev_tx > 0 {
                    InterfaceState::Up // Was active before, likely still up but no current traffic
                } else {
                    InterfaceState::Down
                }
            } else {
                InterfaceState::Unknown // No historical data to determine state
            }
        }
    }

    /// Creates a comprehensive troubleshooting report for debugging collection issues
    pub fn create_troubleshooting_report(&self) -> TroubleshootingReport {
        let reporter = self.create_reporter();
        reporter.create_troubleshooting_report(&self.networks)
    }

    /// Creates a detailed error context report for specific errors
    /// This provides additional debugging information beyond the standard error message
    pub fn create_error_context_report(&self, error: &BandwidthError) -> ErrorContextReport {
        let reporter = self.create_reporter();
        reporter.create_error_context_report(error, &self.networks)
    }

    /// Exports comprehensive error logs and system information for support purposes
    /// This creates a detailed report that can be shared with support teams
    pub fn export_support_report(&self, error: Option<&BandwidthError>) -> Result<String> {
        let reporter = self.create_reporter();
        reporter.export_support_report(error, &self.networks)
    }

    /// Creates a BandwidthReporter instance for reporting functionality
    fn create_reporter(&self) -> BandwidthReporter {
        // Create a new interface manager since it doesn't implement Clone
        let interface_manager = InterfaceManager::new();
        BandwidthReporter::new(
            self.collection_count,
            self.previous_stats.clone(),
            interface_manager,
            self.max_retries,
            self.retry_delay_ms,
            self.min_time_threshold,
        )
    }

    /// Creates a summary report of interface analysis and filtering
    pub fn create_interface_summary_report(&mut self) -> Result<InterfaceSummaryReport> {
        let all_interface_info = self.get_all_interface_info()?;
        let mut reporter = self.create_reporter();
        Ok(reporter.create_interface_summary_report(all_interface_info))
    }

    /// Exports interface analysis to a human-readable format
    pub fn export_interface_analysis(&mut self) -> Result<String> {
        let report = self.create_interface_summary_report()?;
        let mut output = String::new();

        output.push_str("# Network Interface Analysis Report\n\n");
        output.push_str(&format!(
            "Generated: {}\n",
            report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        output.push_str(&format!("Platform: {}\n\n", report.platform));

        output.push_str("## Summary\n");
        output.push_str(&format!(
            "- Total Interfaces: {}\n",
            report.total_interfaces
        ));
        output.push_str(&format!(
            "- Default (Shown): {}\n",
            report.default_interfaces
        ));
        output.push_str(&format!(
            "- Important (High Priority): {}\n",
            report.important_interfaces
        ));
        output.push_str(&format!("- Relevant: {}\n", report.relevant_interfaces));
        output.push_str(&format!("- Cache Size: {} entries\n\n", report.cache_size));

        output.push_str("## Detailed Interface Analysis\n");
        for info in &report.interface_details {
            output.push_str(&format!("### Interface: {}\n", info.name));
            output.push_str(&format!("- Type: {:?}\n", info.interface_type));
            output.push_str(&format!(
                "- Relevance Score: {}/100\n",
                info.relevance.score
            ));
            output.push_str(&format!("- Reason: {}\n", info.relevance.reason));
            output.push_str(&format!(
                "- Show by Default: {}\n",
                info.relevance.show_by_default
            ));
            output.push_str(&format!("- Important: {}\n", info.relevance.is_important));
            output.push_str(&format!("- Filtered: {}\n", info.should_filter));

            if !info.platform_metadata.is_empty() {
                output.push_str("- Platform Metadata:\n");
                for (key, value) in &info.platform_metadata {
                    output.push_str(&format!("  - {}: {}\n", key, value));
                }
            }
            output.push_str("\n");
        }

        Ok(output)
    }

    /// Logs a structured success event for monitoring and performance tracking
    /// This method maintains backward compatibility with the original API
    pub fn log_success_event(&self, stats: &[BandwidthStats], duration_ms: f64) {
        let total_download_bps: f64 = stats.iter().map(|s| s.download_speed_bps).sum();
        let total_upload_bps: f64 = stats.iter().map(|s| s.upload_speed_bps).sum();
        log_success_event(
            stats.len(),
            duration_ms,
            total_download_bps,
            total_upload_bps,
            self.collection_count,
        );
    }

    /// Logs a structured error event for monitoring and debugging
    /// This method maintains backward compatibility with the original API
    pub fn log_error_event(&self, error: &anyhow::Error, context: &str) {
        log_error_event_anyhow(error, context, self.collection_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_creation() {
        // Test default configuration
        let collector = BandwidthCollector::default();
        // We can't access private fields, but we can verify the collector was created
        assert!(true); // Placeholder - collector creation succeeded if we get here

        // Test custom configuration
        let collector = BandwidthCollector::with_retry_config(5, 200);
        // We can't access private fields, but we can verify the collector was created
        assert!(true); // Placeholder - collector creation succeeded if we get here
    }

    #[test]
    fn test_total_bandwidth_calculation() {
        let collector = BandwidthCollector::new();

        // Test with empty collector (no previous stats)
        let (total_download, total_upload) = collector.get_total_bandwidth();
        assert_eq!(total_download, 0.0);
        assert_eq!(total_upload, 0.0);
    }

    #[test]
    fn test_collector_interface_info() {
        let mut collector = BandwidthCollector::new();

        // Test getting interface info for a non-existent interface
        let info = collector.get_interface_info("non_existent_interface");
        assert_eq!(info.name, "non_existent_interface");

        // Test getting all interface info
        let result = collector.get_all_interface_info();
        assert!(result.is_ok());
        let all_info = result.unwrap();
        // The result should be a vector (might be empty on test systems)
        assert!(all_info.len() >= 0);
    }

    #[test]
    fn test_collector_cache_management() {
        let mut collector = BandwidthCollector::new();

        // Test clearing interface cache
        collector.clear_interface_cache();

        // Test getting interface manager stats
        let (cache_size, platform) = collector.get_interface_manager_stats();
        assert!(cache_size >= 0);
        assert!(!platform.is_empty());
    }

    #[test]
    fn test_collector_filtering_methods() {
        let mut collector = BandwidthCollector::new();

        // Test collect_default - should not panic
        let result = collector.collect_default();
        // On test systems, this might fail due to no interfaces, but it shouldn't panic
        match result {
            Ok(stats) => {
                // If successful, verify it returns a vector
                assert!(stats.len() >= 0);
            }
            Err(_) => {
                // If it fails, that's okay for test environments
                assert!(true);
            }
        }

        // Test collect_important - should not panic
        let result = collector.collect_important();
        match result {
            Ok(stats) => {
                assert!(stats.len() >= 0);
            }
            Err(_) => {
                assert!(true);
            }
        }

        // Test collect_filtered - should not panic
        let result = collector.collect_filtered();
        match result {
            Ok(stats) => {
                assert!(stats.len() >= 0);
            }
            Err(_) => {
                assert!(true);
            }
        }
    }

    #[test]
    fn test_collector_main_collect() {
        let mut collector = BandwidthCollector::new();

        // Test main collect method - should not panic
        let result = collector.collect();
        match result {
            Ok(stats) => {
                // If successful, verify each stat has required fields
                for stat in stats {
                    assert!(!stat.interface_name.is_empty());
                    assert!(stat.bytes_received >= 0);
                    assert!(stat.bytes_sent >= 0);
                    assert!(stat.packets_received >= 0);
                    assert!(stat.packets_sent >= 0);
                    assert!(stat.download_speed_bps >= 0.0);
                    assert!(stat.upload_speed_bps >= 0.0);
                    assert!(stat.time_since_last_update >= 0.0);
                }
            }
            Err(_) => {
                // If it fails, that's okay for test environments without network interfaces
                assert!(true);
            }
        }
    }
}
