# Kaipo Watcher - Internet Monitor CLI Tool

A command-line tool for monitoring internet usage, bandwidth, and network packets built with Rust.

## Features

- **Real-time Bandwidth Monitoring**: Track upload/download speeds across all network interfaces
- **Live Dashboard**: Interactive terminal UI with real-time updates using Ratatui
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

## Usage

### Basic Commands

```bash
# Show current network status
kaipo-watcher status

# Show detailed network status with packet information
kaipo-watcher status --detailed

# Launch live monitoring dashboard
kaipo-watcher live

# Monitor specific interface  
kaipo-watcher live --interface en0
# Or use short flags
kaipo-watcher live -I en0 -i 2

# Set custom update interval (in seconds)
kaipo-watcher live --interval 2
```

### Available Commands

- `status` - Display current network statistics
  - `--detailed` - Include packet counts and total data transferred
- `live` - Launch real-time monitoring dashboard
  - `--interface <name>` or `-I <name>` - Monitor specific network interface
  - `--interval <seconds>` or `-i <seconds>` - Set update interval (default: 1s)
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
│   │   └── bandwidth_collector.rs
│   ├── cli/                  # Command-line interface
│   │   ├── mod.rs
│   │   ├── commands.rs       # CLI command definitions
│   │   └── dashboard.rs      # Live dashboard implementation
│   └── main.rs              # Application entry point
├── docs/                    # Documentation
│   ├── API.md              # API reference
│   ├── DEVELOPMENT.md      # Development process
│   └── KNOWN_ISSUES.md     # Known bugs and workarounds
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
- **chrono** - Date and time handling
- **serde** - Serialization framework

### Architecture

The project follows a modular architecture:

1. **Collectors**: Responsible for gathering network statistics
   - `BandwidthCollector` tracks network interface speeds and data usage

2. **CLI Module**: Handles command-line interface
   - `commands.rs` defines available commands and arguments
   - `dashboard.rs` implements the live monitoring UI

3. **Main Application**: Coordinates between modules and executes commands

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

### Phase 2: Enhanced Analysis (Planned)
- [ ] Per-application monitoring
- [ ] Alert system for data limits
- [ ] Export capabilities (JSON, CSV, HTML)
- [ ] Local data storage with SQLite

### Phase 3: Packet Intelligence (Planned)
- [ ] Packet capture and analysis
- [ ] Protocol-level monitoring
- [ ] Security features

### Phase 4: Advanced Features (Planned)
- [ ] Usage prediction with ML
- [ ] Web interface
- [ ] Cloud sync capabilities

## Known Issues

No current known issues. See [docs/KNOWN_ISSUES.md](docs/KNOWN_ISSUES.md) for information about recently resolved issues.

## Known Limitations

- Initial speed readings show 0.00 B/s (requires previous data point for calculation)
- Packet capture features require elevated privileges (not yet implemented)
- Per-application monitoring not yet available
- Historical data storage not yet implemented

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Kaipo Chen