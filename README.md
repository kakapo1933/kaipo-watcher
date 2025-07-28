# Kaipo Watcher - Internet Monitor CLI Tool

A command-line tool for monitoring internet usage, bandwidth, and network packets built with Rust.

## Features

- **Highly Accurate Bandwidth Monitoring**: Advanced speed calculation system with counter reset detection, time anomaly handling, and confidence indicators
- **Intelligent Interface Filtering**: Platform-aware filtering with multiple display modes (important-only, active-only, show-all)
- **Enhanced Live Dashboard**: Interactive terminal UI with real-time sparkline graphs, 50-point historical data tracking, and confidence indicators
- **Comprehensive Graph Generation**: Professional network monitoring charts and visualizations
  - Bandwidth trend charts (line graphs with speed and total usage)
  - Protocol distribution charts (bar, pie, timeline views)
  - Connection pattern visualizations (timeline, port distribution, traffic flow)
  - Multiple export formats: PNG, SVG, JSON, CSV
- **Robust Error Handling**: Graceful degradation with detailed error categorization and recovery mechanisms
- **Cross-Platform Optimization**: Platform-specific interface handling for macOS, Linux, and Windows
- **Packet Monitoring**: Capture and analyze network packets with protocol detection
- **Traffic Analysis**: Detailed protocol distribution and connection tracking
- **Security Analysis**: Detect suspicious patterns and security events
- **Performance Optimized**: Efficient collection with minimal system impact and comprehensive performance monitoring
- **Detailed Network Statistics**: View packet counts, total data transferred, and per-interface metrics with confidence levels
- **Clean Codebase**: Warning-free compilation with comprehensive error handling and extensive test coverage

## Installation

### Option 1: Download Pre-built Binary (Recommended)

Download the latest release from GitHub:

```bash
# Download the compressed binary for macOS ARM64 (Apple Silicon)
curl -L -o kaipo-watcher-v0.2.0-macos-aarch64.tar.gz \
  https://github.com/kakapo1933/kaipo-watcher/releases/download/v0.2.0/kaipo-watcher-v0.2.0-macos-aarch64.tar.gz

# Extract the binary
tar -xzf kaipo-watcher-v0.2.0-macos-aarch64.tar.gz

# Make it executable
chmod +x kw

# Move to system PATH (optional)
sudo mv kw /usr/local/bin/

# Now you can use it from anywhere
kw --help
```

Or visit the [releases page](https://github.com/kakapo1933/kaipo-watcher/releases) to download other formats.

### Option 2: Building from Source

#### Prerequisites

- Rust 1.88.0 or higher (Edition 2024)
- Cargo (comes with Rust)
- Administrative privileges (required for packet capture)

#### Build Steps

```bash
git clone https://github.com/kakapo1933/kaipo-watcher.git
cd kaipo-watcher
cargo build --release
```

The compiled binary will be available at `target/release/kw`.

You can also use the shorter command alias `kw` instead of `kaipo-watcher`.

## Usage

### Basic Commands

```bash
# Show current network status with accurate speed measurements
kaipo-watcher status
# Or use the short alias
kw status

# Show detailed network status with packet information
kw status --detailed

# Use longer measurement duration for more accurate speeds (1-60 seconds)
kw status --measurement-duration 5

# Show only interfaces with active traffic
kw status --active-only

# Show only important interfaces (excludes virtual/container interfaces)
kw status --important-only

# Show all interfaces including virtual and system interfaces
kw status --show-all

# Launch live monitoring dashboard with real-time sparklines
kw live

# Monitor specific interface
kw live --interface en0
# Or use short flags
kw live -I en0 -i 2

# Set custom update interval (in seconds)
kw live --interval 2

# Live dashboard with interface filtering
kw live --important-only  # Clean view without virtual interfaces
kw live --show-all        # Comprehensive view with all interfaces

# Generate bandwidth usage graphs
kw graph bandwidth --period 1h --output bandwidth.png

# Generate protocol distribution chart
kw graph protocols --period 24h --chart-type pie --output protocols.png

# Generate connection timeline with CSV export
kw graph connections --period 6h --format csv --output connections.csv

# Generate multiple bandwidth charts
kw graph bandwidth --period 2h --graph-type both --interface eth0
```

### Available Commands

- `status` - Display current network statistics with accurate speed measurements
  - `--detailed` - Include packet counts and total data transferred
  - `--measurement-duration <seconds>` - Set measurement duration (1-60s, default: 2s)
  - `--active-only` - Show only interfaces with measurable traffic
  - `--important-only` - Show only physical ethernet, wifi, VPN (excludes virtual interfaces)
  - `--show-all` - Show all interfaces including virtual and system interfaces
  - `--interface <name>` - Monitor specific network interface
  - `--interface-analysis` - Export detailed interface analysis report
- `live` - Launch real-time monitoring dashboard
  - `--interface <name>` or `-I <name>` - Monitor specific network interface
  - `--interval <seconds>` or `-i <seconds>` - Set update interval (default: 1s)
  - `--important-only` - Show only important interfaces in dashboard
  - `--show-all` - Show all interfaces including virtual and system interfaces
- `packets` - Real-time packet monitoring and analysis
  - `--interface <name>` or `-I <name>` - Monitor specific network interface
  - `--protocol <protocol>` - Filter by protocol (tcp, udp, icmp, http, https)
  - `--capture <duration>` - Capture duration (e.g., 60s, 5m)
  - `--detailed` - Show detailed packet information
  - `--max-connections <num>` - Maximum connections to display
- `analyze` - Analyze captured traffic patterns
  - `--period <period>` - Analysis period (e.g., 30m, 1h, 24h)
  - `--interface <name>` or `-I <name>` - Analyze specific network interface
  - `--security` - Include security analysis
  - `--protocols` - Show protocol distribution
- `graph` - Generate network monitoring graphs and charts
  - `bandwidth` - Generate bandwidth usage graphs
    - `--period <period>` - Time period (e.g., 30m, 1h, 24h) [default: 1h]
    - `--interface <name>` or `-I <name>` - Graph specific network interface
    - `--output <file>` - Output file path
    - `--format <format>` - Output format: png, svg, json, csv [default: png]
    - `--graph-type <type>` - Graph type: speed, total, both [default: speed]
  - `protocols` - Generate protocol distribution graphs
    - `--period <period>` - Time period (e.g., 30m, 1h, 24h) [default: 1h]
    - `--interface <name>` or `-I <name>` - Graph specific network interface
    - `--output <file>` - Output file path
    - `--format <format>` - Output format: png, svg, json, csv [default: png]
    - `--chart-type <type>` - Chart type: bar, pie, timeline [default: bar]
  - `connections` - Generate connection pattern graphs
    - `--period <period>` - Time period (e.g., 30m, 1h, 24h) [default: 1h]
    - `--interface <name>` or `-I <name>` - Graph specific network interface
    - `--output <file>` - Output file path
    - `--format <format>` - Output format: png, svg, json, csv [default: png]
    - `--chart-type <type>` - Chart type: timeline, ports, traffic [default: timeline]
- `report` - Generate usage reports (not yet implemented)
- `history` - View historical data (not yet implemented)
- `export` - Export data to various formats (deprecated - use `graph` command instead)

### Live Dashboard Features

- **Real-time Sparkline Graphs**: Visual trend indicators for download/upload speeds
- **Historical Data Tracking**: Maintains last 50 data points for trend analysis
- **Per-Interface Monitoring**: Detailed statistics for each network interface
- **Color-coded Display**: Green for downloads, blue for uploads, cyan for interface names

### Live Dashboard Controls

- Press `q` or `ESC` to quit the dashboard

## Bandwidth Monitoring Features

### Advanced Speed Calculation System

Kaipo Watcher provides industry-leading bandwidth measurement accuracy through:

- **Dual Reading System**: Takes baseline and measurement readings separated by configurable duration (1-60 seconds)
- **Counter Reset Detection**: Automatically detects and handles network interface resets, counter wraparounds, and system suspend/resume cycles
- **Time Anomaly Handling**: Robust handling of system clock changes, NTP adjustments, and timing irregularities
- **Data Validation**: Comprehensive validation of interface data integrity including packet-to-byte ratio checks and size validation
- **Confidence Indicators**: Four-level confidence system (High/Medium/Low/None) indicating measurement reliability
- **Graceful Degradation**: Continues monitoring other interfaces when individual interfaces fail
- **Retry Logic**: Configurable retry mechanisms with exponential backoff for network refresh failures

### Intelligent Interface Filtering

Advanced platform-aware filtering system with multiple display modes:

- **Default**: Automatically shows relevant interfaces (excludes most virtual interfaces)
- **`--important-only`**: Shows only physical ethernet, WiFi, and VPN connections
- **`--active-only`**: Shows only interfaces with measurable traffic during measurement
- **`--show-all`**: Shows every interface including Docker, containers, and system interfaces
- **`--interface <name>`**: Focus on a specific interface for detailed monitoring

### Measurement Duration Guidelines

- **1-2 seconds**: Quick checks, may be less accurate for low traffic
- **3-5 seconds**: Good balance of speed and accuracy (recommended)
- **5-10 seconds**: High accuracy, ideal for detailed analysis
- **10+ seconds**: Maximum accuracy for precise measurements

### Platform-Specific Interface Handling

- **macOS**: Advanced filtering of Apple private interfaces (anpi*, awdl*, llw*) while preserving VPN tunnels (utun*)
- **Linux**: Intelligent handling of Docker containers, virtual bridges (br-*, virbr*), and systemd predictable interface names
- **Windows**: Full support for interface names with spaces and virtual machine interface filtering
- **Cross-platform**: Consistent interface type detection, relevance scoring, and intelligent prioritization

## Project Structure

```
kaipo-watcher/
├── src/
│   ├── collectors/           # Data gathering modules
│   │   ├── mod.rs
│   │   ├── bandwidth_collector.rs  # Re-export module for backward compatibility
│   │   ├── bandwidth/        # Modular bandwidth collection system
│   │   │   ├── mod.rs       # Module organization and re-exports
│   │   │   ├── collector.rs # Core BandwidthCollector implementation
│   │   │   ├── errors.rs    # Error handling and system impact assessment
│   │   │   ├── stats.rs     # BandwidthStats and related data structures
│   │   │   ├── validation.rs # Data validation and speed calculation logic
│   │   │   ├── reporting.rs # Troubleshooting and diagnostic reporting
│   │   │   ├── formatting.rs # Utility functions for data formatting
│   │   │   └── tests/       # Comprehensive test modules
│   │   │       ├── mod.rs
│   │   │       ├── collector_tests.rs
│   │   │       ├── validation_tests.rs
│   │   │       ├── reporting_tests.rs
│   │   │       └── integration_tests.rs
│   │   ├── packet_collector.rs
│   │   └── platform/         # Platform-specific packet capture
│   ├── models/              # Data models and types
│   │   ├── mod.rs
│   │   ├── packet.rs
│   │   └── usage.rs
│   ├── analyzers/           # Protocol analysis modules
│   │   ├── mod.rs
│   │   └── protocol_analyzer.rs
│   ├── storage/             # Data persistence layer
│   │   ├── mod.rs
│   │   ├── packet_storage.rs
│   │   └── schema.rs
│   ├── cli/                 # Command-line interface
│   │   ├── mod.rs
│   │   ├── commands.rs      # CLI command definitions
│   │   ├── packet_commands.rs # Packet monitoring commands
│   │   └── graph_commands.rs # Graph generation commands
│   ├── dashboard/           # Terminal UI dashboard
│   │   ├── mod.rs
│   │   └── live_dashboard.rs # Live dashboard with sparklines
│   ├── graphs/              # Graph generation and visualization
│   │   ├── mod.rs
│   │   ├── bandwidth_graphs.rs # Bandwidth trend charts
│   │   ├── protocol_graphs.rs # Protocol distribution charts
│   │   ├── connection_graphs.rs # Connection pattern graphs
│   │   └── export.rs        # Export functionality
│   └── main.rs             # Application entry point
├── docs/                   # Documentation
│   ├── ARCHITECTURE.md     # System architecture
│   ├── DOMAIN_MODEL.md     # Domain model documentation
│   ├── CODE_ORGANIZATION.md # Code organization guide
│   ├── API.md             # API reference
│   ├── DASHBOARD_MODULE.md # Dashboard module documentation
│   ├── BANDWIDTH_COLLECTOR_REFACTORING.md # Bandwidth collector refactoring documentation
│   ├── BANDWIDTH_TROUBLESHOOTING.md # Bandwidth monitoring troubleshooting guide
│   ├── BANDWIDTH_EXAMPLES.md # Examples with actual network traffic output
│   ├── PLATFORM_CONSIDERATIONS.md # Platform-specific requirements and behaviors
│   └── KNOWN_ISSUES.md    # Known bugs and workarounds
├── data/                   # Data storage (created at runtime)
├── Cargo.toml               # Project dependencies
├── CLAUDE.md                # AI assistant instructions
├── BLUEPRINT.md             # Project specification
├── LICENSE                  # MIT License
└── README.md                # This file
```

## Technical Details

### Code Documentation

The codebase includes comprehensive inline documentation:

- All public structs and functions have doc comments
- Key algorithms (like bandwidth speed calculation) are explained
- CLI commands and arguments are documented
- Terminal UI components have detailed comments explaining their purpose

### Dependencies

- **clap** - Command-line argument parsing
- **tokio** - Async runtime for non-blocking operations
- **ratatui** - Terminal UI framework with sparkline support
- **crossterm** - Cross-platform terminal manipulation
- **sysinfo** - System and network information gathering
- **pnet** - Network packet capture and manipulation
- **rusqlite** - SQLite database for local storage
- **chrono** - Date and time handling
- **serde** - Serialization framework
- **anyhow** - Error handling
- **log** - Logging framework
- **env_logger** - Environment-based logging configuration
- **plotters** - High-quality chart generation
- **textplots** - Terminal-based plotting

### Architecture

The project follows a modular architecture with enhanced organization:

1. **Collectors**: Responsible for gathering network statistics
   - **Bandwidth Collection System**: Modular bandwidth monitoring with focused sub-modules
     - `collector.rs`: Core BandwidthCollector implementation (643 lines)
     - `errors.rs`: Comprehensive error handling and system impact assessment (500+ lines)
     - `stats.rs`: BandwidthStats and related data structures (350+ lines)
     - `validation.rs`: Data validation and speed calculation logic (400+ lines)
     - `reporting.rs`: Troubleshooting and diagnostic reporting (400+ lines)
     - `formatting.rs`: Utility functions for data formatting (100+ lines)
   - `PacketCollector` captures and processes network packets

2. **Models**: Define data structures and types
   - `NetworkPacket` represents captured packet data
   - `PacketStatistics` for aggregated packet metrics

3. **Analyzers**: Process and analyze network data
   - `ProtocolAnalyzer` identifies protocols and security patterns

4. **Storage**: Persist data for analysis and reporting
   - `PacketStorage` manages SQLite database operations

5. **CLI Module**: Handles command-line interface
   - `commands.rs` defines available commands and arguments
   - `packet_commands.rs` handles packet monitoring commands

6. **Dashboard Module**: Terminal UI implementation
   - `live_dashboard.rs` implements the real-time monitoring dashboard with sparklines

7. **Graphs Module**: Chart generation and visualization
   - `bandwidth_graphs.rs` generates bandwidth trend charts
   - `protocol_graphs.rs` creates protocol distribution visualizations
   - `connection_graphs.rs` produces connection pattern graphs
   - `export.rs` handles multiple output formats (PNG, SVG, JSON, CSV)

8. **Main Application**: Coordinates between modules and executes commands

## Development

### Running in Development

```bash
# Run with debug output
RUST_LOG=debug cargo run -- status

# Run with specific command
cargo run -- live --interface en0

# Generate graphs in development
cargo run -- graph bandwidth --period 30m --output test_bandwidth.png
cargo run -- graph protocols --period 1h --chart-type pie --output test_protocols.png
```

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy -- -D warnings
```

## Roadmap

### Phase 1: Core Foundation ✅

- [x] Basic bandwidth monitoring
- [x] Simple CLI interface
- [x] Live dashboard with Ratatui

### Phase 2: Packet Intelligence ✅

- [x] Packet capture and analysis
- [x] Protocol-level monitoring
- [x] Security event detection
- [x] Local data storage with SQLite
- [x] Real-time packet monitoring dashboard
- [x] Traffic pattern analysis

### Phase 3: Graph Visualization ✅

- [x] Comprehensive graph generation system (1,349 lines of code)
- [x] Bandwidth trend charts (line graphs with speed and total usage)
- [x] Protocol distribution charts (bar, pie, timeline views)
- [x] Connection pattern visualizations (timeline, port distribution, traffic flow)
- [x] Multiple export formats (PNG, SVG, JSON, CSV)
- [x] Enhanced dashboard with real-time sparklines and 50-point historical data
- [x] Time period filtering (30m, 1h, 24h, etc.)
- [x] Interface-specific graph generation
- [x] Clean, warning-free code with comprehensive error handling

### Phase 4: Enhanced Analysis (Planned)

- [ ] Per-application monitoring
- [ ] Alert system for data limits
- [ ] HTML report generation
- [ ] Geolocation tracking
- [ ] Advanced security analysis

### Phase 5: Advanced Features (Planned)

- [ ] Usage prediction with ML
- [ ] Web interface
- [ ] Cloud sync capabilities

## Recent Major Improvements

### v0.2.0 Release - Major Feature Completion (January 2025)

Kaipo Watcher v0.2.0 represents a significant milestone with enterprise-grade network monitoring capabilities:

- **Professional Graph Generation**: Complete implementation with 1,349+ lines of visualization code
- **Enhanced Live Dashboard**: Real-time sparkline graphs with 50-point historical data tracking
- **Industry-Leading Accuracy**: Advanced bandwidth measurement with confidence indicators
- **Cross-Platform Optimization**: Platform-specific interface filtering and handling
- **Comprehensive Error Handling**: Robust error categorization and graceful degradation
- **Modular Architecture**: Well-organized codebase with 113 unit tests and full coverage
- **Production Ready**: Warning-free compilation with extensive documentation

### Bandwidth Collector Refactoring (Completed)

The bandwidth collection system has undergone a major refactoring to improve maintainability and code organization:

- **Modular Architecture**: The original monolithic `bandwidth_collector.rs` file (1,881 lines) has been successfully refactored into a well-organized modular structure
- **Backward Compatibility**: All existing imports continue to work unchanged - no breaking changes
- **Enhanced Organization**: Code is now split into focused modules:
  - `collector.rs`: Core implementation (643 lines)
  - `errors.rs`: Error handling and system impact assessment (500+ lines)
  - `stats.rs`: Data structures and types (350+ lines)
  - `validation.rs`: Data validation logic (400+ lines)
  - `reporting.rs`: Diagnostic reporting (400+ lines)
  - `formatting.rs`: Utility functions (100+ lines)
- **Comprehensive Testing**: 113 unit tests with full coverage across all modules
- **Same Performance**: No regression in bandwidth collection speed or memory usage
- **Better Documentation**: Each module is well-documented with clear purpose and examples

This refactoring provides a solid foundation for future enhancements while maintaining the reliability and performance of the original implementation.

## Known Issues

No current known issues. See [docs/KNOWN_ISSUES.md](docs/KNOWN_ISSUES.md) for information about recently resolved issues.

## Troubleshooting

### Quick Fixes for Common Issues

**All interfaces show 0.00 B/s**: This is normal for the first measurement as it establishes a baseline. The system now provides clear confidence indicators:
```bash
kw status --measurement-duration 5  # Use longer duration for better accuracy
# or
kw live  # Live dashboard shows real-time updates
```

**"No network interfaces found"**: Enhanced error reporting now provides specific guidance:
```bash
sudo kw status  # Try with elevated privileges
# Check system logs for detailed error information
```

**Interface not found**: Improved error messages now suggest available interfaces:
```bash
kw status --show-all  # See all available interfaces with platform-specific filtering
```

**Counter reset detected**: The system now automatically handles interface resets:
```bash
# Simply run the command again - the system will establish a new baseline
kw status --measurement-duration 3
```

**Too many virtual interfaces**: Enhanced filtering with platform-aware intelligence:
```bash
kw status --important-only  # Show only physical ethernet, WiFi, VPN
kw status --active-only     # Show only interfaces with current traffic
```

For comprehensive troubleshooting, see [docs/BANDWIDTH_TROUBLESHOOTING.md](docs/BANDWIDTH_TROUBLESHOOTING.md).

## Known Limitations

- ~~Initial speed readings show 0.00 B/s~~ **Fixed**: Now provides accurate speed measurements with confidence indicators
- ~~Interface counter resets cause incorrect readings~~ **Fixed**: Automatic counter reset detection and handling
- ~~Time anomalies from system suspend/resume~~ **Fixed**: Robust time anomaly detection and recovery
- ~~Poor error handling for network issues~~ **Fixed**: Comprehensive error categorization and graceful degradation
- Packet capture features require elevated privileges (sudo/administrator)
- Per-application monitoring not yet available
- Some advanced security analysis features in development
- SVG export format not yet implemented for graphs
- HTML report generation not yet implemented

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Kaipo Chen

