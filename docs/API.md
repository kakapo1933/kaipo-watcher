# API Documentation

## Core Modules

### `collectors::BandwidthCollector`

The main component for collecting network bandwidth statistics.

#### Struct Definition

```rust
pub struct BandwidthCollector {
    networks: Networks,
    previous_stats: HashMap<String, (u64, u64, DateTime<Utc>)>,
}
```

#### Methods

##### `new() -> Self`
Creates a new BandwidthCollector instance with refreshed network list.

##### `collect(&mut self) -> Result<Vec<BandwidthStats>>`
Collects current network statistics for all interfaces.

**Returns**: Vector of `BandwidthStats` for each network interface

**Example**:
```rust
let mut collector = BandwidthCollector::new();
let stats = collector.collect()?;
for stat in stats {
    println!("{}: {} down, {} up", 
        stat.interface_name,
        format_speed(stat.download_speed_bps),
        format_speed(stat.upload_speed_bps)
    );
}
```

##### `get_total_bandwidth(&self) -> (f64, f64)`
Returns total bandwidth usage across all interfaces.

**Returns**: Tuple of (total_download_bytes, total_upload_bytes)

### `collectors::BandwidthStats`

Statistics for a single network interface.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthStats {
    pub timestamp: DateTime<Utc>,
    pub interface_name: String,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub packets_received: u64,
    pub packets_sent: u64,
    pub download_speed_bps: f64,
    pub upload_speed_bps: f64,
}
```

### Utility Functions

#### `format_bytes(bytes: f64) -> String`
Formats byte count into human-readable string.

**Example**: 
- `1024` → `"1.00 KB"`
- `1048576` → `"1.00 MB"`

#### `format_speed(bytes_per_second: f64) -> String`
Formats bandwidth speed into human-readable string.

**Example**:
- `1024` → `"1.00 KB/s"`
- `1048576` → `"1.00 MB/s"`

## CLI Module

### `cli::Cli`

Main CLI structure parsed by clap.

```rust
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
```

### `cli::Commands`

Available CLI commands.

```rust
pub enum Commands {
    Live {
        interface: Option<String>,
        packets: bool,
        interval: u64,
    },
    Status {
        detailed: bool,
    },
    Packets {
        interface: Option<String>,
        protocol: Option<String>,
        capture: Option<String>,
        detailed: bool,
        max_connections: usize,
    },
    Analyze {
        period: String,
        interface: Option<String>,
        security: bool,
        protocols: bool,
    },
    Report {
        period: String,
        app_breakdown: bool,
    },
    History {
        days: Option<u32>,
    },
    Export {
        format: String,
        output: Option<String>,
    },
}
```

### `dashboard::Dashboard`

Terminal UI dashboard for live monitoring (located in the `dashboard` module, see [DASHBOARD_MODULE.md](./DASHBOARD_MODULE.md) for detailed documentation).

#### Methods

##### `new(update_interval: u64, interface_filter: Option<String>) -> Self`
Creates new dashboard instance.

**Parameters**:
- `update_interval`: Seconds between updates
- `interface_filter`: Optional interface name filter

##### `run(&mut self) -> Result<()>`
Starts the interactive dashboard.

**Controls**:
- `q` or `ESC`: Quit dashboard

## Usage Examples

### Basic Usage

```rust
use kaipo_watcher::{collectors::BandwidthCollector, cli::{Cli, commands::Commands}, dashboard::Dashboard};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Status { detailed } => {
            let mut collector = BandwidthCollector::new();
            let stats = collector.collect()?;
            // Display stats...
        }
        Commands::Live { interface, interval, .. } => {
            let mut dashboard = Dashboard::new(interval, interface);
            dashboard.run().await?;
        }
        _ => {}
    }
    
    Ok(())
}
```

### Custom Integration

```rust
use kaipo_watcher::collectors::{BandwidthCollector, format_speed};

// Create collector
let mut collector = BandwidthCollector::new();

// Collect stats every second
loop {
    let stats = collector.collect()?;
    
    for stat in stats {
        if stat.interface_name == "en0" {
            println!("WiFi Speed: ↓{} ↑{}", 
                format_speed(stat.download_speed_bps),
                format_speed(stat.upload_speed_bps)
            );
        }
    }
    
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

## Error Handling

All methods return `Result<T>` using the `anyhow` error type for flexibility.

The implementation uses robust error handling including:
- Saturating arithmetic for counter overflows/resets
- Graceful handling of network interface changes
- Safe division with time difference validation

Common error scenarios:
- Network interface not available
- Insufficient permissions (future packet capture)
- System resource limitations

### `models::NetworkPacket`

Represents a captured network packet with comprehensive metadata.

```rust
pub struct NetworkPacket {
    pub timestamp: DateTime<Local>,
    pub interface: String,
    pub size_bytes: u64,
    pub protocol: PacketProtocol,
    pub transport_protocol: TransportProtocol,
    pub source_addr: Option<IpAddr>,
    pub dest_addr: Option<IpAddr>,
    pub source_port: Option<u16>,
    pub dest_port: Option<u16>,
    pub direction: PacketDirection,
}
```

### `collectors::PacketCollector`

Handles low-level packet capture with platform-specific implementations.

#### Methods

##### `new(interface_name: String) -> Result<Self>`
Creates a new packet collector for the specified interface.

##### `start(&mut self) -> Result<()>`
Starts packet capture. Requires elevated privileges.

##### `receive_packet(&mut self) -> Option<NetworkPacket>`
Receives the next captured packet from the queue.

**Example**:
```rust
let mut collector = PacketCollector::new("eth0".to_string())?;
collector.start().await?;

while let Some(packet) = collector.receive_packet().await {
    println!("Captured {} bytes from {}", 
        packet.size_bytes, 
        packet.source_addr.unwrap_or("unknown".parse().unwrap())
    );
}
```

### `analyzers::ProtocolAnalyzer`

Analyzes packets for protocol identification and security patterns.

#### Methods

##### `new() -> Self`
Creates a new protocol analyzer instance.

##### `analyze_packet(&mut self, packet: &NetworkPacket) -> Result<AnalysisResult>`
Analyzes a packet and returns detailed information.

**Returns**: `AnalysisResult` containing:
- Application protocol identification
- Encryption detection
- Traffic type classification
- Security flags
- Flow direction

### `storage::PacketStorage`

Manages persistent storage of packet data and analysis results.

#### Methods

##### `new<P: AsRef<Path>>(db_path: P, batch_size: usize) -> Result<Self>`
Creates new storage instance with SQLite backend.

##### `analyze_packet_for_storage(&self, packet: &NetworkPacket, analysis: &AnalysisResult) -> Result<()>`
Stores packet analysis results in the database.

##### `get_traffic_summary(&self, interface: &str, since: DateTime<Local>) -> Result<TrafficSummary>`
Retrieves aggregated traffic statistics for the specified period.

### `cli::PacketCommandHandler`

Handles packet monitoring CLI commands.

#### Methods

##### `handle_packets_command(&self, interface: Option<String>, protocol_filter: Option<String>, capture_duration: Option<String>, detailed: bool, max_connections: usize) -> Result<()>`
Executes real-time packet monitoring with optional filters.

##### `handle_analyze_command(&self, period: String, interface: Option<String>, security: bool, protocols: bool) -> Result<()>`
Analyzes historical packet data and displays results.

## Packet Monitoring Examples

### Basic Packet Capture

```rust
use kaipo_watcher::collectors::PacketCollector;
use kaipo_watcher::analyzers::ProtocolAnalyzer;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Note: Requires elevated privileges
    let mut collector = PacketCollector::new("eth0".to_string())?;
    let mut analyzer = ProtocolAnalyzer::new();
    
    collector.start().await?;
    
    // Capture for 60 seconds
    let start = tokio::time::Instant::now();
    while start.elapsed() < Duration::from_secs(60) {
        if let Some(packet) = collector.receive_packet().await {
            let analysis = analyzer.analyze_packet(&packet)?;
            
            if let Some(protocol) = analysis.application_protocol {
                println!("Detected {} traffic: {} bytes", 
                    protocol, packet.size_bytes);
            }
        }
    }
    
    Ok(())
}
```

### Traffic Analysis

```rust
use kaipo_watcher::storage::PacketStorage;
use chrono::{Local, Duration};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let storage = Arc::new(PacketStorage::new("./data/packets.db", 100)?);
    
    // Analyze last hour of traffic
    let since = Local::now() - Duration::hours(1);
    let summary = storage.get_traffic_summary("eth0", since)?;
    
    println!("Traffic Summary:");
    println!("Total Packets: {}", summary.total_packets);
    println!("Total Bytes: {}", summary.total_bytes);
    
    for (protocol, stats) in summary.protocols {
        println!("{}: {} packets, {} bytes", 
            protocol, stats.packets, stats.bytes);
    }
    
    Ok(())
}
```

## Security Features

The packet analysis system includes security monitoring:

- **Suspicious Port Detection**: Identifies traffic on uncommon ports
- **Unencrypted Sensitive Data**: Detects potentially sensitive data without encryption
- **High Frequency Patterns**: Identifies unusual traffic frequency
- **Unknown Protocols**: Flags unrecognized or unusual protocols
- **Large Payload Detection**: Identifies unusually large packet payloads

## Performance Considerations

- Packet capture uses efficient channel-based buffering
- Database operations are batched for optimal performance
- Memory usage is controlled through configurable batch sizes
- Platform-specific optimizations for packet capture
- Async/await design prevents blocking operations

## Future API Additions

### Planned for Phase 3
- `ApplicationMonitor` - Per-application bandwidth tracking
- `AlertManager` - Threshold monitoring and notifications
- `DataExporter` - JSON/CSV/HTML export functionality
- `GeolocationAnalyzer` - IP geolocation services
- `MLAnomalyDetector` - Machine learning-based anomaly detection