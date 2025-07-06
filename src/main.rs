// Application modules
mod cli;        // Command-line interface definitions
mod collectors; // Network data collection modules

use anyhow::Result;
use clap::Parser;
use cli::{commands::Commands, Cli};

// Include dashboard module directly from cli directory
// This pattern allows us to keep the dashboard code separate while
// maintaining access to the main application's modules
mod dashboard {
    include!("cli/dashboard.rs");
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
        Commands::Live { interface, packets: _, interval } => {
            let mut dashboard = dashboard::Dashboard::new(interval, interface);
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
    }

    Ok(())
}
