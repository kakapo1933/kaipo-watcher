# Technology Stack & Build System

## Core Technology
- **Language**: Rust (Edition 2024, requires Rust 1.88.0+)
- **Build System**: Cargo
- **Binary Name**: `kw` (alias for `kaipo-watcher`)

## Key Dependencies

### CLI & Terminal UI
- **clap**: Command-line argument parsing with derive macros
- **ratatui**: Terminal UI framework with sparkline support
- **crossterm**: Cross-platform terminal manipulation

### Async & Concurrency
- **tokio**: Async runtime with full feature set
- **async-trait**: Async trait support

### Network & System
- **sysinfo**: System and network information gathering
- **pnet**: Network packet capture and manipulation
- **if-addrs**: Network interface enumeration

### Data & Storage
- **rusqlite**: SQLite database with bundled features
- **serde**: Serialization framework with derive support
- **serde_json**: JSON serialization

### Visualization & Export
- **plotters**: High-quality chart generation
- **textplots**: Terminal-based plotting

### Utilities
- **chrono**: Date and time handling with serde support
- **anyhow**: Error handling
- **thiserror**: Custom error types
- **log**: Logging framework
- **env_logger**: Environment-based logging
- **config**: Configuration management

### Platform-Specific
- **Linux**: `nix` crate for user management
- **macOS/Windows**: `libc` for system calls

## Common Commands

### Development
```bash
# Run with debug logging
RUST_LOG=debug cargo run -- status

# Run specific commands
cargo run -- live --interface en0
cargo run -- graph bandwidth --period 30m --output test.png

# Development testing
cargo run -- status --detailed
```

### Building
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Binary location: target/release/kw
```

### Code Quality
```bash
# Format code
cargo fmt

# Lint with clippy
cargo clippy -- -D warnings

# Run tests
cargo test
```

## Architecture Patterns
- **Modular Design**: Clear separation between collectors, analyzers, storage, CLI, and UI
- **Async/Await**: Non-blocking operations using Tokio runtime
- **Trait-Based**: Extensible design with trait abstractions
- **Error Handling**: Comprehensive error handling with `anyhow` and `thiserror`
- **Platform Abstraction**: OS-specific code isolated in platform modules

## Privileges
- **Packet Capture**: Requires root/administrator privileges for packet monitoring features
- **Network Interfaces**: Standard user privileges sufficient for bandwidth monitoring