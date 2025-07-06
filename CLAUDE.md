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
- `monitor live` - Real-time dashboard
- `monitor status` - Quick current status
- `monitor report` - Usage reports
- `monitor history` - Historical data
- `monitor packets` - Packet analysis
- `monitor config` - Configuration management

## Getting Started

Since this is a new Rust project, the first steps would be:
1. Initialize with `cargo new kaipo-watcher --bin`
2. Set up the project structure following the architecture plan
3. Add key dependencies to Cargo.toml:
   - `clap` for CLI argument parsing
   - `tokio` for async runtime
   - `ratatui` for terminal UI
   - `crossterm` for terminal manipulation
   - `serde` for serialization
   - `chrono` for time handling
   - Platform-specific network libraries
4. Implement basic bandwidth monitoring
5. Gradually add features per the roadmap in OVERVIEW.md

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
```

## Technical Requirements

- **OS Support**: Linux, macOS, Windows
- **Memory**: < 50MB during operation
- **Storage**: Efficient database compression
- **Update Intervals**: Configurable (1s-60s)