// Application modules
mod cli;        // Command-line interface definitions
mod collectors; // Network data collection modules
mod models;     // Data models and types
mod analyzers;  // Protocol analysis modules
mod storage;    // Data persistence layer
mod dashboard;  // Terminal UI dashboard
mod graphs;     // Graph generation and visualization

use anyhow::Result;
use clap::Parser;
use cli::{commands::Commands, Cli, PacketCommandHandler, GraphCommandHandler};
use storage::PacketStorage;
use cli::graph_commands::DatabaseManager;
use std::sync::Arc;
use std::time::Duration;
use dashboard::Dashboard;
use collectors::bandwidth_collector::CalculationConfidence;

/// Handles the status command with persistent collector instance for accurate speed measurement
/// Creates a collector, takes initial reading, waits for specified duration, then takes second reading
async fn handle_status_command(
    detailed: bool,
    measurement_duration: u64,
    active_only: bool,
    interface_filter: Option<String>,
    important_only: bool,
    show_all: bool,
    interface_analysis: bool,
) -> Result<()> {
    // Validate measurement duration
    let duration_secs = measurement_duration.clamp(1, 60);
    if duration_secs != measurement_duration {
        println!("Warning: Measurement duration clamped to {} seconds (valid range: 1-60)", duration_secs);
    }

    println!("Internet Usage Status");
    println!("====================");
    println!("Measuring bandwidth for {} seconds...\n", duration_secs);

    // Create persistent collector instance
    let mut collector = collectors::BandwidthCollector::new();

    // Take initial baseline reading
    let initial_stats = match collector.collect() {
        Ok(stats) => {
            log::info!("Initial bandwidth reading successful: {} interfaces found", stats.len());
            // Log success event for monitoring
            collector.log_success_event(&stats, 0.0);
            stats
        }
        Err(e) => {
            log::error!("Failed to collect initial network statistics: {}", e);
            
            // Enhanced error handling with comprehensive error context reporting
            if let Some(bandwidth_error) = e.downcast_ref::<collectors::bandwidth_collector::BandwidthError>() {
                // Log structured error event for monitoring
                collector.log_error_event(&e, "initial_collection");
                
                // Create comprehensive error context report
                let error_context = collector.create_error_context_report(bandwidth_error);
                
                // Provide user-friendly error message
                eprintln!("Error: {}", error_context.user_friendly_message);
                
                // Log detailed error context for debugging
                log::debug!("Error context report: {:#?}", error_context);
                
                // Show suggested actions based on system impact
                match error_context.system_impact {
                    collectors::bandwidth_collector::SystemImpact::Critical => {
                        eprintln!("\n🚨 Critical Issue - Bandwidth monitoring unavailable");
                        eprintln!("Immediate actions required:");
                        for (i, action) in error_context.suggested_actions.iter().enumerate() {
                            eprintln!("  {}. {}", i + 1, action);
                        }
                        
                        // Offer to generate support report for critical issues
                        eprintln!("\n📋 For technical support, a detailed report has been logged.");
                        if let Ok(support_report) = collector.export_support_report(Some(bandwidth_error)) {
                            log::info!("Support report generated:\n{}", support_report);
                            eprintln!("   Run with RUST_LOG=info to see the full support report.");
                        }
                    }
                    collectors::bandwidth_collector::SystemImpact::High => {
                        eprintln!("\n⚠️  High Impact - Significant monitoring degradation");
                        eprintln!("Recommended actions:");
                        for action in error_context.suggested_actions.iter().take(3) {
                            eprintln!("  • {}", action);
                        }
                    }
                    _ => {
                        eprintln!("\nFor troubleshooting help, run with RUST_LOG=debug");
                    }
                }
            } else {
                eprintln!("Error: Failed to collect initial network statistics: {}", e);
                eprintln!("This might be due to:");
                eprintln!("  - Insufficient system permissions");
                eprintln!("  - No network interfaces available");
                eprintln!("  - System network subsystem issues");
                eprintln!("  - Application compatibility issues");
            }
            return Err(e.context("Failed to initialize bandwidth collection"));
        }
    };

    println!("Initial reading taken, waiting {} seconds for measurement...", duration_secs);

    // Wait for the specified measurement duration
    tokio::time::sleep(Duration::from_secs(duration_secs)).await;

    // Handle interface analysis export if requested
    if interface_analysis {
        println!("Generating interface analysis report...\n");
        match collector.export_interface_analysis() {
            Ok(report) => {
                println!("{}", report);
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error generating interface analysis: {}", e);
                return Err(e.context("Failed to generate interface analysis report"));
            }
        }
    }

    // Take second reading for speed calculation using appropriate collection method
    let measurement_start = std::time::Instant::now();
    let final_stats = if show_all {
        // Collect all interfaces including virtual and system interfaces
        match collector.collect() {
            Ok(stats) => {
                let measurement_duration = measurement_start.elapsed().as_secs_f64() * 1000.0;
                log::info!("Final bandwidth reading successful (all interfaces): {} interfaces processed", stats.len());
                
                // Log success event with measurement duration
                collector.log_success_event(&stats, measurement_duration);
                stats
            }
            Err(e) => {
                log::error!("Failed to collect all network statistics: {}", e);
                return handle_collection_error(&mut collector, e, "final_collection_all");
            }
        }
    } else if important_only {
        // Collect only important interfaces (physical ethernet, wifi, VPN)
        match collector.collect_important() {
            Ok(stats) => {
                let measurement_duration = measurement_start.elapsed().as_secs_f64() * 1000.0;
                log::info!("Final bandwidth reading successful (important interfaces): {} interfaces processed", stats.len());
                
                // Log success event with measurement duration
                collector.log_success_event(&stats, measurement_duration);
                stats
            }
            Err(e) => {
                log::error!("Failed to collect important network statistics: {}", e);
                return handle_collection_error(&mut collector, e, "final_collection_important");
            }
        }
    } else {
        // Collect default interfaces (filtered but not as restrictive as important)
        match collector.collect_default() {
            Ok(stats) => {
                let measurement_duration = measurement_start.elapsed().as_secs_f64() * 1000.0;
                log::info!("Final bandwidth reading successful (default interfaces): {} interfaces processed", stats.len());
                
                // Log success event with measurement duration
                collector.log_success_event(&stats, measurement_duration);
                stats
            }
            Err(e) => {
                log::error!("Failed to collect default network statistics: {}", e);
                return handle_collection_error(&mut collector, e, "final_collection_default");
            }
        }
    };

    // Filter interfaces based on user preferences
    let filtered_stats = filter_interfaces(final_stats, active_only, interface_filter.as_deref())?;

    if filtered_stats.is_empty() {
        if let Some(interface_name) = interface_filter {
            println!("No data available for interface '{}'", interface_name);
            println!("Available interfaces from initial reading:");
            for stat in initial_stats {
                println!("  - {}", stat.interface_name);
            }
        } else if active_only {
            println!("No active interfaces found with traffic during the measurement period.");
            println!("Try running without --active-only to see all interfaces.");
        } else {
            println!("No network interfaces found.");
        }
        return Ok(());
    }

    // Determine filtering information for display
    let filtering_info = if show_all {
        Some("All interfaces (including virtual and system interfaces)")
    } else if important_only {
        Some("Important interfaces only (physical ethernet, wifi, VPN)")
    } else if active_only {
        Some("Active interfaces only")
    } else if interface_filter.is_some() {
        Some("Specific interface filter applied")
    } else {
        Some("Default interface filtering (excludes most virtual interfaces)")
    };

    // Display results with enhanced error reporting
    display_bandwidth_results(&filtered_stats, detailed, duration_secs, filtering_info)?;

    Ok(())
}

/// Filters network interfaces based on user criteria
fn filter_interfaces(
    stats: Vec<collectors::bandwidth_collector::BandwidthStats>,
    active_only: bool,
    interface_filter: Option<&str>,
) -> Result<Vec<collectors::bandwidth_collector::BandwidthStats>> {
    let mut filtered = stats;

    // Filter by specific interface if requested
    if let Some(interface_name) = interface_filter {
        filtered.retain(|stat| stat.interface_name == interface_name);
    }

    // Filter to show only active interfaces if requested
    if active_only {
        filtered.retain(|stat| {
            // Consider interface active if it has measurable speed or recent traffic
            stat.download_speed_bps > 0.0 || 
            stat.upload_speed_bps > 0.0 ||
            (stat.bytes_received > 0 && stat.bytes_sent > 0)
        });
    }

    Ok(filtered)
}

/// Displays bandwidth measurement results with detailed error reporting and confidence indicators
fn display_bandwidth_results(
    stats: &[collectors::bandwidth_collector::BandwidthStats],
    detailed: bool,
    measurement_duration: u64,
    filtering_info: Option<&str>,
) -> Result<()> {
    println!("Bandwidth Measurement Results ({}s measurement period):", measurement_duration);
    if let Some(info) = filtering_info {
        println!("Interface Filtering: {}", info);
    }
    println!("{}", "=".repeat(60));

    for stat in stats {
        println!("\nInterface: {}", stat.interface_name);
        
        // Display speeds with confidence indicators
        let confidence_indicator = match stat.calculation_confidence {
            CalculationConfidence::High => "✓",
            CalculationConfidence::Medium => "~",
            CalculationConfidence::Low => "!",
            CalculationConfidence::None => "?",
        };

        println!("  Download: {} {}", 
            collectors::bandwidth_collector::format_speed(stat.download_speed_bps),
            confidence_indicator
        );
        println!("  Upload:   {} {}", 
            collectors::bandwidth_collector::format_speed(stat.upload_speed_bps),
            confidence_indicator
        );

        // Show confidence explanation
        match stat.calculation_confidence {
            CalculationConfidence::High => {
                if detailed {
                    println!("  Confidence: High ✓ (reliable measurement)");
                }
            },
            CalculationConfidence::Medium => {
                println!("  Confidence: Medium ~ (measurement may be affected by short intervals or interface changes)");
            },
            CalculationConfidence::Low => {
                println!("  Confidence: Low ! (counter resets or time anomalies detected)");
            },
            CalculationConfidence::None => {
                println!("  Confidence: None ? (insufficient data for calculation)");
            },
        }

        // Show interface type and state if detailed
        if detailed {
            println!("  Interface Type: {:?}", stat.interface_type);
            println!("  Interface State: {:?}", stat.interface_state);
            println!("  Time Since Last Update: {:.2}s", stat.time_since_last_update);
            println!("  Total Received: {}", collectors::bandwidth_collector::format_bytes(stat.bytes_received as f64));
            println!("  Total Sent: {}", collectors::bandwidth_collector::format_bytes(stat.bytes_sent as f64));
            println!("  Packets Received: {}", stat.packets_received);
            println!("  Packets Sent: {}", stat.packets_sent);
            println!("  Timestamp: {}", stat.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
        }
    }

    // Display legend for confidence indicators
    println!("\nConfidence Indicators:");
    println!("  ✓ High confidence    ~ Medium confidence    ! Low confidence    ? No data");
    
    if !detailed {
        println!("\nUse --detailed for more information about each interface.");
    }

    Ok(())
}

/// Helper function to handle collection errors with comprehensive error reporting
fn handle_collection_error(
    collector: &mut collectors::BandwidthCollector,
    e: anyhow::Error,
    context: &str,
) -> Result<()> {
    // Enhanced error handling for collection with comprehensive context
    if let Some(bandwidth_error) = e.downcast_ref::<collectors::bandwidth_collector::BandwidthError>() {
        // Log structured error event
        collector.log_error_event(&e, context);
        
        // Create comprehensive error context report
        let error_context = collector.create_error_context_report(bandwidth_error);
        
        // Provide user-friendly error message
        eprintln!("Error: {}", error_context.user_friendly_message);
        
        // Log detailed error context for debugging
        log::debug!("Collection error context report: {:#?}", error_context);
        
        // Show context-aware guidance based on error type and system impact
        match bandwidth_error {
            collectors::bandwidth_collector::BandwidthError::RefreshFailed { retry_attempts, .. } => {
                eprintln!("\n🔄 The system attempted {} retries but could not refresh network data.", retry_attempts);
                eprintln!("This suggests a persistent system issue that may require manual intervention.");
                eprintln!("\nNext steps:");
                for action in error_context.suggested_actions.iter().take(3) {
                    eprintln!("  • {}", action);
                }
            }
            collectors::bandwidth_collector::BandwidthError::TimeAnomaly { .. } => {
                eprintln!("\n⏰ A time anomaly was detected during measurement.");
                eprintln!("This can happen if the system clock changed or the system was suspended.");
                eprintln!("Try running the measurement again.");
            }
            _ => {
                if error_context.system_impact == collectors::bandwidth_collector::SystemImpact::Critical {
                    eprintln!("\n🚨 Critical system issue detected");
                    for action in error_context.suggested_actions.iter().take(2) {
                        eprintln!("  • {}", action);
                    }
                }
                eprintln!("\nFor detailed troubleshooting information, run with RUST_LOG=debug");
            }
        }
    } else {
        eprintln!("Error: Failed to collect network statistics: {}", e);
        eprintln!("This might be due to:");
        eprintln!("  - Network interface state changes during measurement");
        eprintln!("  - System network subsystem issues");
        eprintln!("  - Temporary network connectivity problems");
        eprintln!("  - System resource constraints");
    }
    Err(e.context("Failed to complete bandwidth measurement"))
}

/// Main application entry point
/// Handles command-line parsing and dispatches to appropriate handlers
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging based on RUST_LOG environment variable
    env_logger::init();
    
    // Parse command-line arguments using clap
    let cli = Cli::parse();

    // Match on the parsed command and execute appropriate handler
    match cli.command {
        // Live monitoring with real-time dashboard
        Commands::Live { interface, packets: _, interval, important_only, show_all } => {
            let mut dashboard = Dashboard::new(interval, interface, important_only, show_all);
            dashboard.run().await?;
        }
        // Display current network status (one-time snapshot)
        Commands::Status { detailed, measurement_duration, active_only, interface, important_only, show_all, interface_analysis } => {
            handle_status_command(detailed, measurement_duration, active_only, interface, important_only, show_all, interface_analysis).await?;
        }
        // Future feature: Generate usage reports
        Commands::Report { period, app_breakdown: _ } => {
            println!("Report generation for period '{period}' is not yet implemented.");
        }
        // Future feature: Show historical usage data
        Commands::History { days } => {
            println!("History display for {days:?} days is not yet implemented.");
        }
        // Future feature: Export data to various formats
        Commands::Export { format, output } => {
            println!("Export to format '{format}' (output: {output:?}) is not yet implemented.");
        }
        // Real-time packet monitoring
        Commands::Packets { interface, protocol, capture, detailed, max_connections } => {
            // Initialize packet storage
            let storage = Arc::new(PacketStorage::new("./data/packets.db", 100)?);
            let handler = PacketCommandHandler::new(storage);
            
            handler.handle_packets_command(
                interface,
                protocol,
                capture,
                detailed,
                max_connections,
            ).await?;
        }
        // Traffic pattern analysis
        Commands::Analyze { period, interface, security, protocols } => {
            // Initialize packet storage
            let storage = Arc::new(PacketStorage::new("./data/packets.db", 100)?);
            let handler = PacketCommandHandler::new(storage);
            
            handler.handle_analyze_command(
                period,
                interface,
                security,
                protocols,
            ).await?;
        }
        // Graph generation
        Commands::Graph { graph_type } => {
            // Initialize database manager
            let db = Arc::new(DatabaseManager::new("./data/packets.db").await?);
            let handler = GraphCommandHandler::new(db);
            
            handler.handle_graph_command(graph_type).await?;
        }
    }

    Ok(())
}
