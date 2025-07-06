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

### `cli::Dashboard`

Terminal UI dashboard for live monitoring.

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
use kaipo_watcher::{collectors::BandwidthCollector, cli::Cli};
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

## Future API Additions

### Planned for Phase 2
- `ApplicationMonitor` - Per-application bandwidth tracking
- `AlertManager` - Threshold monitoring and notifications
- `DataExporter` - JSON/CSV/HTML export functionality

### Planned for Phase 3
- `PacketCollector` - Low-level packet capture
- `ProtocolAnalyzer` - Protocol classification
- `SecurityMonitor` - Anomaly detection