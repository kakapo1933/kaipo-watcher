// Application modules
mod cli;        // Command-line interface definitions
mod collectors; // Network data collection modules
mod models;     // Data models and types
mod analyzers;  // Protocol analysis modules
mod storage;    // Data persistence layer
mod dashboard;  // Terminal UI dashboard

use anyhow::Result;
use clap::Parser;
use cli::{commands::Commands, Cli, PacketCommandHandler};
use storage::PacketStorage;
use std::sync::Arc;
use dashboard::Dashboard;

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
        Commands::Live { interface, packets: _, interval } => {
            let mut dashboard = Dashboard::new(interval, interface);
            dashboard.run().await?;
        }
        // Display current network status (one-time snapshot)
        Commands::Status { detailed } => {
            let mut collector = collectors::BandwidthCollector::new();
            let stats = collector.collect()?;
            
            println!("Internet Usage Status");
            println!("====================");
            
            // Display stats for each network interface
            for stat in stats {
                println!("
Interface: {}", stat.interface_name);
                println!("  Download: {}", collectors::bandwidth_collector::format_speed(stat.download_speed_bps));
                println!("  Upload: {}", collectors::bandwidth_collector::format_speed(stat.upload_speed_bps));
                
                // Show additional details if requested
                if detailed {
                    println!("  Total Received: {}", collectors::bandwidth_collector::format_bytes(stat.bytes_received as f64));
                    println!("  Total Sent: {}", collectors::bandwidth_collector::format_bytes(stat.bytes_sent as f64));
                    println!("  Packets Received: {}", stat.packets_received);
                    println!("  Packets Sent: {}", stat.packets_sent);
                }
            }
        }
        // Future feature: Generate usage reports
        Commands::Report { period, app_breakdown: _ } => {
            println!("Report generation for period '{}' is not yet implemented.", period);
        }
        // Future feature: Show historical usage data
        Commands::History { days } => {
            println!("History display for {:?} days is not yet implemented.", days);
        }
        // Future feature: Export data to various formats
        Commands::Export { format, output } => {
            println!("Export to format '{}' (output: {:?}) is not yet implemented.", format, output);
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
    }

    Ok(())
}
