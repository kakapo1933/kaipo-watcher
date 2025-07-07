use clap::{Parser, Subcommand};

/// Main CLI structure for the kaipo-watcher application
/// Uses clap's derive macros for automatic CLI generation
#[derive(Parser)]
#[command(author = "Kaipo Chen")]
#[command(version)] // Automatically uses version from Cargo.toml
#[command(about = "Internet Monitor CLI Tool - Monitor bandwidth, usage, and network packets")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands for the kaipo-watcher application
/// Each variant represents a different mode of operation
#[derive(Subcommand)]
pub enum Commands {
    /// Real-time monitoring with terminal dashboard
    /// Displays live bandwidth stats and network activity
    #[command(about = "Monitor network in real-time")]
    Live {
        /// Filter to monitor only a specific network interface
        #[arg(short = 'I', long, help = "Monitor specific network interface")]
        interface: Option<String>,

        /// Include detailed packet-level information (future feature)
        #[arg(short, long, help = "Include packet-level details")]
        packets: bool,

        /// How often to update the display (in seconds)
        #[arg(
            short = 'i',
            long,
            default_value = "1",
            help = "Update interval in seconds"
        )]
        interval: u64,
    },

    /// One-time snapshot of current network status
    /// Shows current speeds and interface statistics
    #[command(about = "Show current network status")]
    Status {
        /// Include additional details like total bytes and packet counts
        #[arg(short, long, help = "Show detailed information")]
        detailed: bool,
    },

    /// Generate usage reports for specified time periods (future feature)
    #[command(about = "Generate usage report")]
    Report {
        /// Time period for the report
        #[arg(
            short,
            long,
            default_value = "month",
            help = "Report period: day, week, month"
        )]
        period: String,

        /// Include per-application network usage breakdown
        #[arg(short, long, help = "Include per-application breakdown")]
        app_breakdown: bool,
    },

    /// Show historical network usage data (future feature)
    #[command(about = "Show historical usage data")]
    History {
        /// Number of days of history to display
        #[arg(short, long, help = "Number of days to show")]
        days: Option<u32>,
    },

    /// Export network data to various formats (future feature)
    #[command(about = "Export data")]
    Export {
        /// Output format for the exported data
        #[arg(
            short,
            long,
            default_value = "json",
            help = "Export format: json, csv, html"
        )]
        format: String,

        /// File path for the exported data
        #[arg(short, long, help = "Output file path")]
        output: Option<String>,
    },

    /// Real-time packet monitoring and analysis
    #[command(about = "Monitor network packets")]
    Packets {
        /// Network interface to monitor
        #[arg(short = 'I', long, help = "Monitor specific network interface")]
        interface: Option<String>,

        /// Filter by protocol (tcp, udp, icmp, http, https)
        #[arg(short, long, help = "Filter by protocol")]
        protocol: Option<String>,

        /// Capture duration in seconds
        #[arg(short, long, help = "Capture duration (e.g., 60s, 5m)")]
        capture: Option<String>,

        /// Show packet details
        #[arg(short, long, help = "Show detailed packet information")]
        detailed: bool,

        /// Maximum number of connections to display
        #[arg(long, default_value = "10", help = "Maximum connections to show")]
        max_connections: usize,
    },

    /// Analyze captured traffic patterns
    #[command(about = "Analyze network traffic patterns")]
    Analyze {
        /// Time period to analyze
        #[arg(
            short,
            long,
            default_value = "1h",
            help = "Analysis period (e.g., 30m, 1h, 24h)"
        )]
        period: String,

        /// Network interface to analyze
        #[arg(short = 'I', long, help = "Analyze specific network interface")]
        interface: Option<String>,

        /// Include security analysis
        #[arg(short, long, help = "Include security analysis")]
        security: bool,

        /// Show protocol distribution
        #[arg(long, help = "Show protocol distribution")]
        protocols: bool,
    },
}
