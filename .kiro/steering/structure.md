# Project Structure & Organization

## Directory Layout

```
kaipo-watcher/
├── src/
│   ├── main.rs              # Application entry point with command dispatch
│   ├── lib.rs               # Library root with module exports
│   │
│   ├── cli/                 # Command-line interface layer
│   │   ├── mod.rs          # CLI module exports
│   │   ├── commands.rs     # Clap command definitions and structures
│   │   ├── dashboard.rs    # Dashboard command handler
│   │   ├── graph_commands.rs # Graph generation command handlers
│   │   └── packet_commands.rs # Packet monitoring command handlers
│   │
│   ├── collectors/          # Data collection modules
│   │   ├── mod.rs          # Collector exports and traits
│   │   ├── bandwidth_collector.rs # Network speed measurement
│   │   ├── packet_collector.rs # Packet capture implementation
│   │   └── platform/       # Platform-specific implementations
│   │       ├── mod.rs      # Platform detection and routing
│   │       ├── linux.rs    # Linux-specific collectors
│   │       ├── macos.rs    # macOS-specific collectors
│   │       └── windows.rs  # Windows-specific collectors
│   │
│   ├── models/             # Data models and types
│   │   ├── mod.rs          # Model exports
│   │   └── packet.rs       # Network packet structures and enums
│   │
│   ├── storage/            # Data persistence layer
│   │   ├── mod.rs          # Storage module exports
│   │   ├── packet_storage.rs # SQLite database operations
│   │   └── schema.rs       # Database schema definitions
│   │
│   ├── analyzers/          # Data analysis modules
│   │   ├── mod.rs          # Analyzer exports
│   │   └── protocol_analyzer.rs # Protocol detection and analysis
│   │
│   ├── dashboard/          # Terminal UI components
│   │   ├── mod.rs          # Dashboard module exports
│   │   └── live_dashboard.rs # Real-time dashboard with sparklines
│   │
│   ├── graphs/             # Chart generation and visualization
│   │   ├── mod.rs          # Graph module exports
│   │   ├── bandwidth_graphs.rs # Bandwidth trend charts
│   │   ├── protocol_graphs.rs # Protocol distribution charts
│   │   ├── connection_graphs.rs # Connection pattern graphs
│   │   └── export.rs       # Multi-format export functionality
│   │
│   ├── config/             # Configuration management
│   │   └── mod.rs          # Configuration structures and loading
│   │
│   └── exporters/          # Data export functionality
│       └── mod.rs          # Export trait definitions
│
├── data/                   # Runtime data storage (created automatically)
│   ├── packets.db          # SQLite database for packet data
│   ├── packets.db-shm      # SQLite shared memory
│   └── packets.db-wal      # SQLite write-ahead log
│
├── docs/                   # Comprehensive documentation
│   ├── ARCHITECTURE.md     # System architecture overview
│   ├── CODE_ORGANIZATION.md # Code organization guide
│   ├── DOMAIN_MODEL.md     # Domain model documentation
│   ├── API.md             # Internal API reference
│   ├── DASHBOARD_MODULE.md # Dashboard module documentation
│   └── KNOWN_ISSUES.md    # Known bugs and workarounds
│
├── target/                 # Cargo build artifacts (gitignored)
├── Cargo.toml             # Project manifest and dependencies
├── Cargo.lock             # Dependency lock file
├── README.md              # Project overview and usage guide
├── BLUEPRINT.md           # Complete project specification
├── CLAUDE.md              # AI assistant instructions
└── LICENSE                # MIT license
```

## Module Organization Principles

### Naming Conventions
- **Files**: Snake case (`bandwidth_collector.rs`)
- **Modules**: Snake case (`packet_collector`)
- **Types**: Pascal case (`NetworkPacket`, `BandwidthCollector`)
- **Functions**: Snake case (`collect_bandwidth`, `format_speed`)
- **Constants**: Screaming snake case (`MAX_PACKET_SIZE`)

### Module Boundaries
- **CLI Layer**: Handles user interaction and command parsing
- **Collectors**: Gather raw data from system interfaces
- **Models**: Define data structures and domain types
- **Storage**: Persist and retrieve data efficiently
- **Analyzers**: Process raw data into insights
- **Dashboard**: Terminal UI rendering and interaction
- **Graphs**: Chart generation and visualization
- **Exporters**: Output formatting and file generation

### Import Organization
```rust
// Standard library imports first
use std::collections::HashMap;
use std::sync::Arc;

// External crate imports next
use chrono::{DateTime, Local};
use tokio::sync::Mutex;

// Internal crate imports last
use crate::models::NetworkPacket;
use crate::storage::PacketStorage;
```

### Error Handling Pattern
- Use `anyhow::Result<T>` for application errors
- Use `thiserror` for custom error types
- Propagate errors with `?` operator
- Add context with `.context()` when helpful

### Async Patterns
- Use `#[tokio::main]` for main function
- Use `async fn` for I/O operations
- Use `Arc<T>` for shared state across async tasks
- Use channels for communication between async tasks

### Documentation Standards
- Module-level documentation with `//!`
- Function documentation with `///`
- Include examples in documentation
- Document error conditions and panics