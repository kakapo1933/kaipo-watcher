use clap::{Parser, Subcommand};

/// Main CLI structure for the kaipo-watcher application
/// Uses clap's derive macros for automatic CLI generation
#[derive(Parser)]
#[command(author = "Kaipo Chen")]
#[command(version)] // Automatically uses version from Cargo.toml
#[command(about = "Internet Monitor CLI Tool - Monitor bandwidth, usage, and network packets with accurate real-time speed calculations")]
#[command(long_about = "Kaipo Watcher provides comprehensive network monitoring with accurate bandwidth measurements, \
real-time dashboard, packet analysis, and professional graph generation. Features robust error handling, \
cross-platform support, and intelligent interface filtering.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands for the kaipo-watcher application
/// Each variant represents a different mode of operation
#[derive(Subcommand)]
pub enum Commands {
    /// Real-time monitoring with terminal dashboard and sparkline graphs
    /// Displays live bandwidth stats with historical trend visualization
    #[command(about = "Monitor network in real-time with interactive dashboard")]
    #[command(long_about = "Launches an interactive terminal dashboard with real-time bandwidth monitoring, \
sparkline graphs showing historical trends, and comprehensive interface statistics. \
Press 'q' or ESC to exit the dashboard.\n\n\
Examples:\n  \
kw live                               # Monitor all relevant interfaces\n  \
kw live --interface en0               # Monitor specific interface\n  \
kw live --important-only              # Clean view without virtual interfaces\n  \
kw live --interval 2                  # Update every 2 seconds")]
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

        /// Show only important interfaces (physical ethernet, wifi, VPN)
        /// Excludes virtual, container, and system interfaces for cleaner dashboard
        #[arg(
            long,
            help = "Show only important interfaces: physical ethernet, wifi, VPN (excludes virtual/container interfaces)"
        )]
        important_only: bool,

        /// Show all interfaces including virtual and system interfaces
        /// Displays every interface in the dashboard, including Docker, VPN, loopback, etc.
        #[arg(
            long,
            help = "Show all interfaces including virtual, container, and system interfaces"
        )]
        show_all: bool,
    },

    /// One-time snapshot of current network status with accurate speed measurements
    /// Takes two readings separated by measurement duration to calculate precise speeds
    #[command(about = "Show current network status with accurate bandwidth measurements")]
    #[command(long_about = "Displays current network interface statistics with accurate speed calculations. \
Takes an initial baseline reading, waits for the specified measurement duration, then takes a second reading \
to calculate precise download/upload speeds. Supports various filtering options to show only relevant interfaces.\n\n\
Examples:\n  \
kw status --measurement-duration 5    # 5-second measurement for accuracy\n  \
kw status --active-only               # Show only interfaces with traffic\n  \
kw status --important-only            # Show only physical interfaces\n  \
kw status --interface en0             # Monitor specific interface")]
    Status {
        /// Include additional details like total bytes and packet counts
        #[arg(short, long, help = "Show detailed information")]
        detailed: bool,

        /// Duration in seconds to measure bandwidth (minimum 1, maximum 60)
        /// Longer durations provide more accurate speed measurements
        #[arg(
            short = 'm',
            long,
            default_value = "2",
            help = "Measurement duration in seconds for accurate speed calculation (1-60s, longer = more accurate)"
        )]
        measurement_duration: u64,

        /// Filter to show only interfaces with active traffic during measurement
        #[arg(
            short = 'a',
            long,
            help = "Show only active interfaces with measurable traffic during the measurement period"
        )]
        active_only: bool,

        /// Filter to monitor only a specific network interface
        #[arg(short = 'I', long, help = "Monitor specific network interface")]
        interface: Option<String>,

        /// Show only important interfaces (physical ethernet, wifi, VPN)
        /// Excludes virtual, container, and system interfaces for cleaner output
        #[arg(
            long,
            help = "Show only important interfaces: physical ethernet, wifi, VPN (excludes virtual/container interfaces)"
        )]
        important_only: bool,

        /// Show all interfaces including virtual and system interfaces
        /// Displays every interface found by the system, including Docker, VPN, loopback, etc.
        #[arg(
            long,
            help = "Show all interfaces including virtual, container, and system interfaces"
        )]
        show_all: bool,

        /// Export interface analysis report with detailed platform-specific information
        #[arg(
            long,
            help = "Export detailed interface analysis report with platform-specific information"
        )]
        interface_analysis: bool,
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

    /// Generate network monitoring graphs
    #[command(about = "Generate network monitoring graphs")]
    Graph {
        /// Type of graph to generate
        #[command(subcommand)]
        graph_type: GraphType,
    },
}

/// Types of graphs that can be generated
#[derive(Subcommand)]
pub enum GraphType {
    /// Generate bandwidth usage graphs
    #[command(about = "Generate bandwidth usage graphs")]
    Bandwidth {
        /// Time period for the graph
        #[arg(
            short,
            long,
            default_value = "1h",
            help = "Time period (e.g., 30m, 1h, 24h)"
        )]
        period: String,

        /// Network interface to graph
        #[arg(short = 'I', long, help = "Graph specific network interface")]
        interface: Option<String>,

        /// Output file path
        #[arg(short, long, help = "Output file path")]
        output: Option<String>,

        /// Graph format
        #[arg(
            short,
            long,
            default_value = "png",
            help = "Output format: png, svg, json, csv"
        )]
        format: String,

        /// Graph type
        #[arg(
            short,
            long,
            default_value = "speed",
            help = "Graph type: speed, total, both"
        )]
        graph_type: String,
    },

    /// Generate protocol distribution graphs
    #[command(about = "Generate protocol distribution graphs")]
    Protocols {
        /// Time period for the graph
        #[arg(
            short,
            long,
            default_value = "1h",
            help = "Time period (e.g., 30m, 1h, 24h)"
        )]
        period: String,

        /// Network interface to graph
        #[arg(short = 'I', long, help = "Graph specific network interface")]
        interface: Option<String>,

        /// Output file path
        #[arg(short, long, help = "Output file path")]
        output: Option<String>,

        /// Graph format
        #[arg(
            short,
            long,
            default_value = "png",
            help = "Output format: png, svg, json, csv"
        )]
        format: String,

        /// Chart type
        #[arg(
            short,
            long,
            default_value = "bar",
            help = "Chart type: bar, pie, timeline"
        )]
        chart_type: String,
    },

    /// Generate connection pattern graphs
    #[command(about = "Generate connection pattern graphs")]
    Connections {
        /// Time period for the graph
        #[arg(
            short,
            long,
            default_value = "1h",
            help = "Time period (e.g., 30m, 1h, 24h)"
        )]
        period: String,

        /// Network interface to graph
        #[arg(short = 'I', long, help = "Graph specific network interface")]
        interface: Option<String>,

        /// Output file path
        #[arg(short, long, help = "Output file path")]
        output: Option<String>,

        /// Graph format
        #[arg(
            short,
            long,
            default_value = "png",
            help = "Output format: png, svg, json, csv"
        )]
        format: String,

        /// Chart type
        #[arg(
            short,
            long,
            default_value = "timeline",
            help = "Chart type: timeline, ports, traffic"
        )]
        chart_type: String,
    },
}
