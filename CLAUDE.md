# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the **Internet Monitor CLI Tool** (kaipo-watcher) project - a command-line tool for monitoring internet usage, bandwidth, and network packets. Currently, this is a greenfield project with only specifications defined.

## Project Status

⚠️ **New Project**: No implementation exists yet. Only the specification document is available in `.claude/OVERVIEW.md`.

## Architecture Plan

Based on the specification, the project will have the following structure:

```
src/
├── collectors/       # Data gathering modules (bandwidth, usage, packets, protocols)
├── storage/         # Data persistence (database, cache, backup)
├── analyzers/       # Usage analysis (trends, alerts, costs, anomalies)
├── exporters/       # Output formats (JSON, CSV, HTML, CLI)
├── config/          # Configuration management
└── cli/            # Command-line interface
```

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

The project will use **Ratatui** for the terminal UI, particularly for the `monitor live` command dashboard. Key considerations:

1. **Dashboard Layout**: Use Ratatui's layout system to create the bordered sections shown in BLUEPRINT.md
2. **Real-time Updates**: Combine with tokio for async updates without blocking the UI
3. **Widgets to Use**:
   - `Block` with borders for the main frame
   - `Gauge` for the progress bar (monthly usage)
   - `List` for top applications
   - `Paragraph` for stats display
4. **Color Scheme**: Use Ratatui's styling for status indicators (green for good, yellow for warning, red for alerts)

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

