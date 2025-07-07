# Code Organization Guide

## Directory Structure

```
kaipo-watcher/
├── Cargo.toml                 # Project manifest with dependencies
├── Cargo.lock                # Dependency lock file
├── README.md                  # Project overview and quick start
├── CLAUDE.md                 # AI assistant instructions
├── LICENSE                   # MIT or Apache-2.0 license
│
├── src/                      # Main source code directory
│   ├── main.rs              # CLI entry point and app initialization
│   ├── lib.rs               # Library root for shared functionality
│   │
│   ├── cli/                 # Command-line interface layer
│   │   ├── mod.rs          # CLI module exports
│   │   ├── commands.rs     # Command definitions and handlers
│   │   ├── args.rs         # Clap argument structures
│   │   └── output.rs       # Output formatting helpers
│   │
│   ├── collectors/          # Data collection modules
│   │   ├── mod.rs          # Collector trait and common types
│   │   ├── bandwidth.rs    # Bandwidth measurement collector
│   │   ├── packets.rs      # Packet capture implementation
│   │   ├── applications.rs # Application identification
│   │   └── platform/       # Platform-specific implementations
│   │       ├── mod.rs      # Platform detection and routing
│   │       ├── linux.rs    # Linux-specific collectors
│   │       ├── macos.rs    # macOS-specific collectors
│   │       └── windows.rs  # Windows-specific collectors
│   │
│   ├── storage/            # Data persistence layer
│   │   ├── mod.rs          # Storage traits and types
│   │   ├── database.rs     # SQLite database implementation
│   │   ├── schema.rs       # Database schema definitions
│   │   ├── migrations.rs   # Database migration logic
│   │   ├── cache.rs        # In-memory caching layer
│   │   └── compression.rs  # Data compression utilities
│   │
│   ├── analyzers/          # Data analysis and processing
│   │   ├── mod.rs          # Analyzer traits and registry
│   │   ├── statistics.rs   # Statistical calculations
│   │   ├── trends.rs       # Trend analysis algorithms
│   │   ├── anomalies.rs    # Anomaly detection logic
│   │   ├── costs.rs        # Cost calculation engine
│   │   └── predictions.rs  # Usage prediction models
│   │
│   ├── exporters/          # Data export functionality
│   │   ├── mod.rs          # Exporter trait and registry
│   │   ├── json.rs         # JSON export implementation
│   │   ├── csv.rs          # CSV export implementation
│   │   ├── html.rs         # HTML report generation
│   │   └── templates/      # HTML/Report templates
│   │
│   ├── ui/                 # User interface components
│   │   ├── mod.rs          # UI module exports
│   │   ├── dashboard.rs    # Ratatui dashboard implementation
│   │   ├── widgets/        # Custom Ratatui widgets
│   │   │   ├── mod.rs      # Widget exports
│   │   │   ├── bandwidth_gauge.rs
│   │   │   ├── usage_chart.rs
│   │   │   └── app_list.rs
│   │   ├── themes.rs       # UI color schemes and styles
│   │   └── events.rs       # Terminal event handling
│   │
│   ├── models/             # Domain models and types
│   │   ├── mod.rs          # Model exports
│   │   ├── network.rs      # Network-related types
│   │   ├── usage.rs        # Usage data structures
│   │   ├── alerts.rs       # Alert definitions
│   │   └── config.rs       # Configuration structures
│   │
│   ├── services/           # Business logic services
│   │   ├── mod.rs          # Service exports
│   │   ├── monitoring.rs   # Core monitoring service
│   │   ├── alerting.rs     # Alert management service
│   │   ├── reporting.rs    # Report generation service
│   │   └── scheduling.rs   # Task scheduling service
│   │
│   ├── utils/              # Utility functions and helpers
│   │   ├── mod.rs          # Utility exports
│   │   ├── formatting.rs   # Data formatting utilities
│   │   ├── validation.rs   # Input validation helpers
│   │   ├── errors.rs       # Error types and handling
│   │   └── constants.rs    # Application constants
│   │
│   └── config/             # Configuration management
│       ├── mod.rs          # Config module exports
│       ├── loader.rs       # Configuration file loading
│       ├── defaults.rs     # Default configuration values
│       └── validator.rs    # Config validation logic
│
├── tests/                   # Integration tests
│   ├── common/             # Shared test utilities
│   │   ├── mod.rs
│   │   └── fixtures.rs     # Test data fixtures
│   ├── cli_tests.rs        # CLI command tests
│   ├── collector_tests.rs  # Collector integration tests
│   └── storage_tests.rs    # Storage layer tests
│
├── benches/                # Performance benchmarks
│   ├── packet_processing.rs
│   └── statistics.rs
│
├── examples/               # Example usage scripts
│   ├── basic_monitoring.rs
│   └── custom_analyzer.rs
│
├── docs/                   # Documentation
│   ├── ARCHITECTURE.md     # System architecture
│   ├── DOMAIN_MODEL.md     # Domain concepts
│   ├── CODE_ORGANIZATION.md # This file
│   └── API.md             # Internal API documentation
│
├── scripts/               # Build and utility scripts
│   ├── install.sh        # Installation script
│   ├── build-release.sh  # Release build script
│   └── generate-caps.sh  # Linux capabilities setup
│
└── .github/              # GitHub specific files
    └── workflows/        # CI/CD workflows
        ├── ci.yml       # Continuous integration
        └── release.yml  # Release automation
```

## Module Boundaries

### Core Modules

Each module has clear responsibilities and interfaces:

#### CLI Module (`src/cli/`)
- **Purpose**: Handle all user interaction through the command line
- **Dependencies**: Can depend on services and exporters
- **Exports**: Command handlers, argument parsers
- **Example Usage**:
```rust
use crate::cli::{Commands, run_command};

let args = Commands::parse();
run_command(args).await?;
```

#### Collectors Module (`src/collectors/`)
- **Purpose**: Gather raw data from system interfaces
- **Dependencies**: Platform modules, models
- **Exports**: Collector trait, specific implementations
- **Key Trait**:
```rust
#[async_trait]
pub trait Collector: Send + Sync {
    type Data;
    async fn collect(&mut self) -> Result<Self::Data>;
    fn name(&self) -> &str;
}
```

#### Storage Module (`src/storage/`)
- **Purpose**: Persist and retrieve data efficiently
- **Dependencies**: Models only
- **Exports**: Storage trait, database implementation
- **Design**: Repository pattern for data access

#### Analyzers Module (`src/analyzers/`)
- **Purpose**: Process raw data into insights
- **Dependencies**: Models, storage (read-only)
- **Exports**: Analyzer trait, analysis implementations
- **Pattern**: Pure functions where possible

#### UI Module (`src/ui/`)
- **Purpose**: Terminal user interface rendering
- **Dependencies**: Models, services (read-only)
- **Exports**: Dashboard, widgets
- **Framework**: Ratatui with custom widgets

## Naming Conventions

### File Names
- **Snake case**: `bandwidth_collector.rs`
- **Descriptive**: Clearly indicate content
- **Singular**: `application.rs` not `applications.rs` (unless it's a collection module)

### Type Names
- **Pascal case**: `NetworkInterface`, `DataUsage`
- **Descriptive**: `BandwidthCollector` not `BWColl`
- **No abbreviations**: `Application` not `App` (except well-known like `Id`)

### Function Names
- **Snake case**: `calculate_bandwidth()`, `get_current_usage()`
- **Verb prefixes**: `is_`, `has_`, `get_`, `set_`, `calculate_`
- **Async suffix**: `fetch_data_async()` for clarity when needed

### Variable Names
- **Snake case**: `total_bytes`, `current_bandwidth`
- **Meaningful**: `bytes_downloaded` not `bd`
- **Units in name**: `timeout_seconds`, `size_bytes`

### Constants
- **Screaming snake case**: `MAX_PACKET_SIZE`, `DEFAULT_TIMEOUT`
- **Module level**: Define in `constants.rs` or module top

### Module Organization
```rust
// Standard module organization
use std::collections::HashMap;  // Std library imports first
use std::sync::Arc;

use chrono::{DateTime, Local}; // External crates next
use tokio::sync::Mutex;

use crate::models::Network;     // Internal imports last
use crate::utils::format_bytes;

// Re-exports at the top
pub use self::bandwidth::BandwidthCollector;
pub use self::packets::PacketCollector;

// Module declarations
mod bandwidth;
mod packets;
mod platform;

// Public items
pub struct CollectorManager {
    // ...
}

// Private items
struct CollectorState {
    // ...
}
```

## Error Handling

### Error Types
```rust
// In src/utils/errors.rs
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Permission denied: {0}")]
    Permission(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Error Propagation
- Use `?` operator for propagation
- Add context with `.context()` from `anyhow`
- Log errors at appropriate levels
- User-friendly messages in CLI layer

## Testing Strategy

### Unit Tests
Located in the same file as the code:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bandwidth_calculation() {
        // Test implementation
    }
}
```

### Integration Tests
Located in `tests/` directory:
- Test multiple modules together
- Use real (but isolated) resources
- Mock external dependencies

### Property-Based Tests
Using `proptest` for invariant testing:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_data_size_conversions(bytes in 0u64..=1_000_000_000) {
        let size = DataSize::from_bytes(bytes);
        assert_eq!(size.as_bytes(), bytes);
    }
}
```

## Documentation Standards

### Module Documentation
```rust
//! # Bandwidth Module
//!
//! This module provides bandwidth measurement and calculation functionality.
//!
//! ## Example
//!
//! ```rust
//! use kaipo_watcher::collectors::BandwidthCollector;
//!
//! let collector = BandwidthCollector::new();
//! let bandwidth = collector.collect().await?;
//! ```
```

### Function Documentation
```rust
/// Calculates the current bandwidth usage.
///
/// # Arguments
///
/// * `interface` - The network interface to measure
/// * `interval` - Measurement interval in seconds
///
/// # Returns
///
/// Returns the bandwidth in bits per second, or an error if measurement fails.
///
/// # Example
///
/// ```rust
/// let bandwidth = calculate_bandwidth("eth0", 1.0)?;
/// println!("Current bandwidth: {} Mbps", bandwidth.as_mbps());
/// ```
pub fn calculate_bandwidth(interface: &str, interval: f64) -> Result<Bandwidth> {
    // Implementation
}
```

## Dependency Management

### Core Dependencies
Specified in `Cargo.toml`:
```toml
[dependencies]
# CLI
clap = { version = "4.0", features = ["derive"] }

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Terminal UI
ratatui = "0.26"
crossterm = "0.27"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database
rusqlite = { version = "0.30", features = ["bundled"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Platform-specific
[target.'cfg(target_os = "linux")'.dependencies]
caps = "0.5"

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.48", features = ["Win32_NetworkManagement"] }
```

### Feature Flags
```toml
[features]
default = ["sqlite", "tui"]
sqlite = ["rusqlite"]
tui = ["ratatui", "crossterm"]
experimental = ["ml-predictions"]
```

## Build Configuration

### Release Profile
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### Platform-Specific Builds
```rust
#[cfg(target_os = "linux")]
mod linux_impl {
    // Linux-specific implementation
}

#[cfg(target_os = "macos")]
mod macos_impl {
    // macOS-specific implementation
}

#[cfg(target_os = "windows")]
mod windows_impl {
    // Windows-specific implementation
}
```

## Code Quality Tools

### Linting
```bash
# Run clippy with strict settings
cargo clippy -- -D warnings

# Common clippy lints to enable
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
```

### Formatting
```bash
# Format all code
cargo fmt

# Check formatting in CI
cargo fmt -- --check
```

### Security Auditing
```bash
# Check for known vulnerabilities
cargo audit

# Check for unsafe code
cargo geiger
```

## Performance Considerations

### Zero-Cost Abstractions
- Use generic types over trait objects where possible
- Inline small functions with `#[inline]`
- Avoid unnecessary allocations

### Profiling Points
Mark critical paths for profiling:
```rust
#[cfg(feature = "profiling")]
let _timer = Timer::new("packet_processing");
```

### Benchmarking
Located in `benches/`:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_packet_parsing(c: &mut Criterion) {
    c.bench_function("parse_packet", |b| {
        b.iter(|| parse_packet(black_box(&packet_data)))
    });
}
```

## Version Control Patterns

### Branch Strategy
- `main`: Stable releases only
- `develop`: Integration branch
- `feature/*`: New features
- `fix/*`: Bug fixes
- `refactor/*`: Code improvements

### Commit Messages
Follow conventional commits:
- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `style:` Code style changes
- `refactor:` Code refactoring
- `perf:` Performance improvements
- `test:` Test additions/changes
- `chore:` Maintenance tasks

### Code Review Checklist
- [ ] Tests pass
- [ ] Documentation updated
- [ ] Error handling complete
- [ ] Performance impact considered
- [ ] Security implications reviewed
- [ ] Platform compatibility verified