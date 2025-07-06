mod cli;
mod collectors;

use anyhow::Result;
use clap::Parser;
use cli::{commands::Commands, Cli};

mod dashboard {
    include!("cli/dashboard.rs");
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();

    match cli.command {
        Commands::Live { interface, packets: _, interval } => {
            let mut dashboard = dashboard::Dashboard::new(interval, interface);
            dashboard.run().await?;
        }
        Commands::Status { detailed } => {
            let mut collector = collectors::BandwidthCollector::new();
            let stats = collector.collect()?;
            
            println!("Internet Usage Status");
            println!("====================");
            
            for stat in stats {
                println!("\nInterface: {}", stat.interface_name);
                println!("  Download: {}", collectors::bandwidth_collector::format_speed(stat.download_speed_bps));
                println!("  Upload: {}", collectors::bandwidth_collector::format_speed(stat.upload_speed_bps));
                
                if detailed {
                    println!("  Total Received: {}", collectors::bandwidth_collector::format_bytes(stat.bytes_received as f64));
                    println!("  Total Sent: {}", collectors::bandwidth_collector::format_bytes(stat.bytes_sent as f64));
                    println!("  Packets Received: {}", stat.packets_received);
                    println!("  Packets Sent: {}", stat.packets_sent);
                }
            }
        }
        Commands::Report { period, app_breakdown: _ } => {
            println!("Report generation for period '{}' is not yet implemented.", period);
        }
        Commands::History { days } => {
            println!("History display for {:?} days is not yet implemented.", days);
        }
        Commands::Export { format, output } => {
            println!("Export to format '{}' (output: {:?}) is not yet implemented.", format, output);
        }
    }

    Ok(())
}
