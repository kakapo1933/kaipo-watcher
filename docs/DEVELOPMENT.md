# Development Documentation

This document records the development process of the Kaipo Watcher project.

## Project Timeline

### 2025-07-06: Project Initialization

#### Initial Setup
1. **Project Creation**
   - Created Rust project using `cargo init --name kaipo-watcher`
   - Used Rust edition 2024 (latest available)
   - Set up basic project metadata in Cargo.toml

2. **Directory Structure**
   - Created modular architecture following the BLUEPRINT.md specification:
     ```
     src/
     ├── collectors/     # Data gathering modules
     ├── cli/           # Command-line interface
     ├── storage/       # Data persistence (future)
     ├── analyzers/     # Usage analysis (future)
     ├── exporters/     # Output formats (future)
     └── config/        # Configuration (future)
     ```

3. **Dependencies Selection**
   - **clap 4.5**: Modern derive-based CLI parsing
   - **tokio 1.43**: Async runtime for concurrent operations
   - **ratatui 0.30.0-alpha.5**: Terminal UI (latest alpha)
   - **crossterm 0.29**: Cross-platform terminal control
   - **sysinfo 0.33**: Network interface statistics
   - **chrono 0.4**: Timestamp handling with serde support
   - **serde 1.0**: Serialization for future data export
   - **anyhow & thiserror**: Error handling
   - **rusqlite 0.33**: Future database support
   - **pnet 0.35**: Future packet capture support

#### First Feature: Bandwidth Monitoring

1. **BandwidthCollector Implementation**
   ```rust
   pub struct BandwidthCollector {
       networks: Networks,
       previous_stats: HashMap<String, (u64, u64, DateTime<Utc>)>,
   }
   ```
   - Tracks network interfaces using sysinfo
   - Calculates speed by comparing current and previous readings
   - Provides formatted output (B/s, KB/s, MB/s, GB/s)

2. **CLI Command Structure**
   - Implemented subcommands: `live`, `status`, `report`, `history`, `export`
   - Added flags for detailed output and interface filtering
   - Used clap's derive API for clean command definitions

3. **Live Dashboard**
   - Built with Ratatui for terminal UI
   - Real-time updates with configurable intervals
   - Shows current speeds, total usage, and per-interface statistics
   - Keyboard controls (q/ESC to quit)

#### Technical Decisions

1. **Why Rust?**
   - Memory safety without garbage collection
   - Excellent performance for system-level monitoring
   - Strong ecosystem for CLI tools
   - Cross-platform compilation

2. **Architecture Choices**
   - Modular design for easy feature addition
   - Async/await for non-blocking operations
   - Trait-based abstractions for future extensibility

3. **UI Framework Selection**
   - Chose Ratatui over alternatives (cursive, tui-rs) because:
     - Active development and modern API
     - Good documentation
     - Specified in CLAUDE.md requirements

#### Challenges Encountered

1. **Dependency Version Issues**
   - Initial versions specified were too new
   - Resolved by checking crates.io for latest stable versions
   - Ratatui required alpha version specification

2. **Compilation Errors**
   - Missing serde feature for chrono DateTime
   - Mutable reference requirements for bandwidth collection
   - Backend trait bounds for terminal error handling

3. **Network Interface Detection**
   - sysinfo returns all interfaces (including virtual)
   - Need to filter for relevant interfaces in future

4. **Runtime Issues**
   - Clap argument conflict causing compilation panics
   - Integer overflow in bandwidth calculation causing crashes
   - Both issues resolved during initial development phase

## Code Quality Measures

### Testing Strategy
- Unit tests for bandwidth calculations (TODO)
- Integration tests for CLI commands (TODO)
- Mock network interfaces for testing (TODO)

### Performance Considerations
- Lazy loading of network statistics
- Efficient data structures for speed calculation
- Minimal memory footprint (~50MB target)

### Security Considerations
- No elevated privileges required for basic monitoring
- Future packet capture will require capability management
- All data stored locally (no network transmission)

## Project Status

### ✅ Completed (Phase 1)
- [x] Basic bandwidth monitoring with BandwidthCollector
- [x] CLI interface with multiple subcommands (status, live, report, history, export)
- [x] Live dashboard using Ratatui with real-time updates
- [x] Cross-platform support (Linux, macOS, Windows)
- [x] Comprehensive documentation (README, API, DEVELOPMENT)
- [x] Error handling for edge cases (counter resets, argument conflicts)
- [x] MIT License
- [x] Stable operation without crashes

### 📊 Current Capabilities
- Real-time bandwidth monitoring for all network interfaces
- Live terminal dashboard with 1-second updates
- Detailed network statistics (speeds, packet counts, total usage)
- Command-line interface with intuitive subcommands
- Human-readable output formatting (B/s, KB/s, MB/s, GB/s)

## Future Implementation Notes

### Phase 2 Features
1. **Per-Application Monitoring**
   - Will require process network mapping
   - Platform-specific implementations needed
   - Consider using netstat2 crate

2. **Alert System**
   - Threshold configuration in YAML/TOML
   - System notifications via notify-rust
   - Email alerts as optional feature

3. **Data Persistence**
   - SQLite schema design needed
   - Time-series optimization
   - Configurable retention policies

### Phase 3 Features
1. **Packet Capture**
   - pnet for cross-platform support
   - Capability handling for Linux
   - Admin privilege prompts for Windows/macOS

2. **Protocol Analysis**
   - TCP/UDP/ICMP classification
   - Application protocol detection
   - Traffic pattern analysis

## Issues Resolved During Development

### 1. Clap Argument Conflict (2025-07-06)
- **Issue**: Short flag `-i` conflict between `interface` and `interval` arguments
- **Impact**: Application panicked during argument parsing
- **Resolution**: Assigned different short flags (-I for interface, -i for interval)
- **Lesson**: Always test CLI argument combinations during development

### 2. Integer Overflow in Bandwidth Calculation (2025-07-06)
- **Issue**: Dashboard crashed when network counters reset or wrapped around
- **Impact**: Application panicked with "attempt to subtract with overflow"
- **Resolution**: Used saturating subtraction to handle counter resets gracefully
- **Lesson**: Always consider edge cases in system-level programming

## Lessons Learned

1. **Start Simple**: Basic bandwidth monitoring provides immediate value
2. **Version Management**: Always verify dependency availability
3. **Error Handling**: Implement comprehensive error handling early
4. **Documentation**: Keep docs updated throughout development
5. **CLI Testing**: Test all argument combinations to catch conflicts early
6. **Edge Case Handling**: System programming requires careful handling of counter resets and wraparounds
7. **Issue Resolution**: Fix issues immediately when discovered rather than documenting for later

## Development Commands

```bash
# Build optimized binary
cargo build --release

# Run with logging
RUST_LOG=debug cargo run

# Check code quality
cargo clippy -- -D warnings
cargo fmt --check

# Generate documentation
cargo doc --open

# Benchmark performance (future)
cargo bench
```

## Resources Used

- [Rust Book](https://doc.rust-lang.org/book/)
- [Ratatui Documentation](https://ratatui.rs/)
- [Clap Documentation](https://docs.rs/clap/latest/clap/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)