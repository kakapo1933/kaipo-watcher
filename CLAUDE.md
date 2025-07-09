# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the **Internet Monitor CLI Tool** (kaipo-watcher) project - a command-line tool for monitoring internet usage, bandwidth, and network packets. Currently, this is a greenfield project with only specifications defined.

## Project Status

✅ **Phase 2 Complete**: Core functionality implemented including real-time monitoring, packet capture, and traffic analysis. The project is now actively developed with a modular architecture.

## Architecture Plan

The project follows a modular architecture:

```
src/
├── collectors/       # Data gathering modules (bandwidth, usage, packets, protocols)
├── storage/         # Data persistence (database, cache, backup)
├── analyzers/       # Usage analysis (trends, alerts, costs, anomalies)
├── cli/            # Command-line interface and commands
├── dashboard/      # Terminal UI dashboard (separated from CLI)
├── models/         # Data models and types
└── main.rs         # Application entry point
```

### Recent Architectural Changes

- **Dashboard Module Separation**: The dashboard functionality has been moved from `cli/dashboard.rs` to a dedicated `dashboard/` module for better organization and separation of concerns.
- **Modular Design**: Each module has clear responsibilities and minimal coupling with other modules.

## Key Features to Implement

1. **Real-time Monitoring**: Bandwidth speeds, data usage, packet analysis
2. **Per-Application Tracking**: Monitor network usage by application
3. **Alert System**: Data limit warnings and anomaly detection
4. **Export Capabilities**: Multiple output formats for reports
5. **Cross-Platform Support**: Linux, macOS, Windows compatibility

## Development Guidelines

### Language: Rust

This project will be implemented in **Rust** for:

- High performance and memory safety
- Excellent cross-platform support
- Strong ecosystem for CLI tools (clap, tokio)
- Zero-cost abstractions for system programming
- Built-in package management with Cargo

### Key Implementation Considerations

1. **Privileges**: Packet capture requires root/admin privileges
   - Use `caps` crate for Linux capabilities
   - Consider `pcap` or `pnet` for packet capture
2. **Performance**: Must maintain < 5% CPU usage during monitoring
   - Use async/await with tokio for efficient I/O
   - Implement efficient data structures for statistics
3. **Privacy**: All data stays local, optional encryption
   - Use `ring` or `rustcrypto` for encryption needs
4. **Cross-Platform**: Use abstraction layers for OS-specific features
   - `sysinfo` for system information
   - Conditional compilation with `#[cfg(target_os = "...")]`

### Rust-Specific Libraries to Consider

- **Network Monitoring**: `pnet`, `pcap`, `netstat2`
- **CLI Framework**: `clap` with derive macros
- **Terminal UI**: `ratatui` for the live dashboard (confirmed)
- **Database**: `rusqlite` or `sled` for local storage
- **Serialization**: `serde` with JSON/CSV support
- **Error Handling**: `anyhow` or `thiserror`

### Ratatui Implementation Notes

The project uses **Ratatui** for the terminal UI in the `dashboard` module. Implementation details:

1. **Dashboard Module**: Located in `src/dashboard/live_dashboard.rs` (separated from CLI module)
2. **Dashboard Layout**: Uses Ratatui's layout system with 4 main sections:
   - Header: Title and timestamp
   - Current Speed: Real-time bandwidth statistics
   - Interface List: Per-interface metrics with packet counts
   - Footer: Keyboard shortcuts
3. **Real-time Updates**: Combined with tokio for async updates without blocking the UI
4. **Widgets Used**:
   - `Block` with borders for section frames
   - `List` for network interfaces
   - `Paragraph` for stats display
   - `Line` and `Span` for styled text
5. **Color Scheme**: 
   - Green for download speeds
   - Blue for upload speeds
   - Yellow for timestamps
   - Cyan for interface names

### CLI Command Structure

Primary commands as per specification:

- `live` - Real-time dashboard
- `status` - Quick current status
- `packets` - Real-time packet monitoring
- `analyze` - Traffic pattern analysis
- `report` - Usage reports (planned)
- `history` - Historical data (planned)
- `export` - Data export (planned)

## Getting Started

The project is now fully functional with core features implemented:

**Current Dependencies:**
- `clap` for CLI argument parsing with derive macros
- `tokio` for async runtime and non-blocking I/O
- `ratatui` for terminal UI and live dashboard
- `crossterm` for cross-platform terminal manipulation
- `pnet` for low-level packet capture
- `rusqlite` for SQLite database storage
- `serde` for serialization (JSON/CSV)
- `chrono` for time handling
- `anyhow` for error handling
- `log` and `env_logger` for logging

**Key Implementation Highlights:**
1. ✅ Async packet capture with platform-specific privilege handling
2. ✅ Protocol analysis with security pattern detection
3. ✅ Time-series database optimization for packet storage
4. ✅ Real-time monitoring with bounded channel buffering
5. ✅ Cross-platform support (Linux, macOS, Windows)

## Common Rust Commands

```bash
# Build and run
cargo build
cargo run
cargo build --release

# Testing
cargo test
cargo test -- --nocapture

# Linting and formatting
cargo clippy -- -D warnings
cargo fmt

# Documentation
cargo doc --open

# Run specific commands (examples)
cargo run -- status --detailed
cargo run -- packets --interface eth0 --protocol tcp
cargo run -- analyze --period 1h --security --protocols
```

## Technical Requirements

- **OS Support**: Linux, macOS, Windows
- **Memory**: < 50MB during operation
- **Storage**: Efficient database compression
- **Update Intervals**: Configurable (1s-60s)

