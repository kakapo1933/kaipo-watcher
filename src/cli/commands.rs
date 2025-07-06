use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "monitor")]
#[command(author = "Kaipo Chen")]
#[command(version = "0.1.0")]
#[command(about = "Internet Monitor CLI Tool - Monitor bandwidth, usage, and network packets")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Monitor network in real-time")]
    Live {
        #[arg(short = 'I', long, help = "Monitor specific network interface")]
        interface: Option<String>,
        
        #[arg(short, long, help = "Include packet-level details")]
        packets: bool,
        
        #[arg(short = 'i', long, default_value = "1", help = "Update interval in seconds")]
        interval: u64,
    },
    
    #[command(about = "Show current network status")]
    Status {
        #[arg(short, long, help = "Show detailed information")]
        detailed: bool,
    },
    
    #[command(about = "Generate usage report")]
    Report {
        #[arg(short, long, default_value = "month", help = "Report period: day, week, month")]
        period: String,
        
        #[arg(short, long, help = "Include per-application breakdown")]
        app_breakdown: bool,
    },
    
    #[command(about = "Show historical usage data")]
    History {
        #[arg(short, long, help = "Number of days to show")]
        days: Option<u32>,
    },
    
    #[command(about = "Export data")]
    Export {
        #[arg(short, long, default_value = "json", help = "Export format: json, csv, html")]
        format: String,
        
        #[arg(short, long, help = "Output file path")]
        output: Option<String>,
    },
}