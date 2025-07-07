# Kaipo Watcher - Internet Monitor CLI Tool

A command-line tool for monitoring internet usage, bandwidth, and network packets built with Rust.

## Features

- **Real-time Bandwidth Monitoring**: Track upload/download speeds across all network interfaces
- **Live Dashboard**: Interactive terminal UI with real-time updates using Ratatui
- **Packet Monitoring**: Capture and analyze network packets with protocol detection
- **Traffic Analysis**: Detailed protocol distribution and connection tracking
- **Security Analysis**: Detect suspicious patterns and security events
- **Detailed Network Statistics**: View packet counts, total data transferred, and per-interface metrics
- **Cross-Platform Support**: Works on Linux, macOS, and Windows

## Installation

### Prerequisites

- Rust 1.88.0 or higher
- Cargo (comes with Rust)

### Building from Source

```bash
git clone https://github.com/yourusername/kaipo-watcher.git
cd kaipo-watcher
cargo build --release
```

The compiled binary will be available at `target/release/kaipo-watcher`.

You can also use the shorter command alias `kw` instead of `kaipo-watcher`.

## Usage

### Basic Commands

```bash
# Show current network status
kaipo-watcher status
# Or use the short alias
kw status

# Show detailed network status with packet information
kw status --detailed

# Launch live monitoring dashboard
kw live

# Monitor specific interface  
kw live --interface en0
# Or use short flags
kw live -I en0 -i 2

# Set custom update interval (in seconds)
kw live --interval 2
```

### Available Commands

- `status` - Display current network statistics
  - `--detailed` - Include packet counts and total data transferred
- `live` - Launch real-time monitoring dashboard
  - `--interface <name>` or `-I <name>` - Monitor specific network interface
  - `--interval <seconds>` or `-i <seconds>` - Set update interval (default: 1s)
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
- `report` - Generate usage reports (not yet implemented)
- `history` - View historical data (not yet implemented)
- `export` - Export data to various formats (not yet implemented)

### Live Dashboard Controls

- Press `q` or `ESC` to quit the dashboard

## Project Structure

```
kaipo-watcher/
├── src/
│   ├── collectors/           # Data gathering modules
│   │   ├── mod.rs
│   │   ├── bandwidth_collector.rs
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
│   │   └── dashboard.rs     # Live dashboard implementation
│   └── main.rs             # Application entry point
├── docs/                   # Documentation
│   ├── ARCHITECTURE.md     # System architecture
│   ├── DOMAIN_MODEL.md     # Domain model documentation
│   ├── CODE_ORGANIZATION.md # Code organization guide
│   ├── API.md             # API reference
│   └── KNOWN_ISSUES.md    # Known bugs and workarounds
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
- **ratatui** - Terminal UI framework
- **crossterm** - Cross-platform terminal manipulation
- **sysinfo** - System and network information gathering
- **pnet** - Network packet capture and manipulation
- **rusqlite** - SQLite database for local storage
- **chrono** - Date and time handling
- **serde** - Serialization framework
- **anyhow** - Error handling
- **log** - Logging framework
- **env_logger** - Environment-based logging configuration

### Architecture

The project follows a modular architecture:

1. **Collectors**: Responsible for gathering network statistics
   - `BandwidthCollector` tracks network interface speeds and data usage
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
   - `dashboard.rs` implements the live monitoring UI

6. **Main Application**: Coordinates between modules and executes commands

## Development

### Running in Development

```bash
# Run with debug output
RUST_LOG=debug cargo run -- status

# Run with specific command
cargo run -- live --interface en0
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

### Phase 3: Enhanced Analysis (Planned)
- [ ] Per-application monitoring
- [ ] Alert system for data limits
- [ ] Export capabilities (JSON, CSV, HTML)
- [ ] Geolocation tracking
- [ ] Advanced security analysis

### Phase 4: Advanced Features (Planned)
- [ ] Usage prediction with ML
- [ ] Web interface
- [ ] Cloud sync capabilities

## Known Issues

No current known issues. See [docs/KNOWN_ISSUES.md](docs/KNOWN_ISSUES.md) for information about recently resolved issues.

## Known Limitations

- Initial speed readings show 0.00 B/s (requires previous data point for calculation)
- Packet capture features require elevated privileges (sudo/administrator)
- Per-application monitoring not yet available
- Some advanced security analysis features in development

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Kaipo Chen