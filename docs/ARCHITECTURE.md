# Architecture Overview

## System Design

The kaipo-watcher CLI tool follows a modular, event-driven architecture designed for high performance and extensibility. The system is structured to minimize resource usage while providing real-time network monitoring capabilities.

## Core Components

### 1. Data Collection Layer

```
┌─────────────────────────────────────────────┐
│           Packet Capture Engine             │
│         (Root/Admin Privileges)             │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│       Network Interface Abstraction         │
│    (Platform-specific implementations)      │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│            Data Collectors                  │
│  ┌─────────┐ ┌─────────┐ ┌─────────────┐    │
│  │Bandwidth│ │ Packets │ │ Application │    │
│  │Collector│ │Collector│ │  Collector  │    │
│  └─────────┘ └─────────┘ └─────────────┘    │
└─────────────────────────────────────────────┘
```

The data collection layer operates at the lowest level, interfacing directly with the operating system's network stack. Key design decisions:

- **Async/Non-blocking**: Uses Tokio runtime for efficient I/O operations
- **Zero-copy buffers**: Minimizes memory allocations during packet capture
- **Platform abstraction**: Isolates OS-specific code behind trait interfaces

### 2. Processing Pipeline

```
┌─────────────────────────────────────────────┐
│             Event Bus (MPSC)                │
│         Async Message Passing               │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│           Data Processors                   │
│  ┌────────┐ ┌──────────┐ ┌─────────────┐    │
│  │Protocol│ │  Stats   │ │   Anomaly   │    │
│  │Analyzer│ │Aggregator│ │  Detector   │    │
│  └────────┘ └──────────┘ └─────────────┘    │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│           Storage Manager                   │
│    (Write-optimized Time-series DB)         │
└─────────────────────────────────────────────┘
```

The processing pipeline uses an event-driven architecture to handle high-throughput data streams:

- **Backpressure handling**: Prevents memory overflow during traffic spikes
- **Batch processing**: Groups data for efficient storage operations
- **Hot path optimization**: Critical paths use lock-free data structures

### 3. User Interface Layer

```
┌─────────────────────────────────────────────┐
│            CLI Command Parser               │
│              (Clap Framework)               │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│           Command Handlers                  │
│  ┌─────────┐ ┌─────────┐ ┌─────────────┐    │
│  │  Live   │ │ Report  │ │   Config    │    │
│  │ Monitor │ │Generator│ │  Manager    │    │
│  └─────────┘ └─────────┘ └─────────────┘    │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│         Terminal UI (Ratatui)               │
│      Real-time Dashboard Rendering          │
└─────────────────────────────────────────────┘
```

The UI layer provides multiple interaction modes:

- **Live Dashboard**: Real-time updates using Ratatui's event loop
- **CLI Commands**: Quick status checks and report generation
- **Export System**: Multiple output formats (JSON, CSV, HTML)

## Data Flow Patterns

### Real-time Monitoring Flow

```
Network Packet → Capture → Parse → Analyze → Aggregate → Display
                    ↓                           ↓
                  Store                      Alert Check
```

1. **Packet Capture**: Raw packets captured from network interface
2. **Protocol Parsing**: Extract metadata (source, destination, protocol, size)
3. **Application Mapping**: Associate traffic with specific applications
4. **Statistical Aggregation**: Calculate bandwidth, usage totals
5. **UI Update**: Push updates to dashboard via event channels

### Historical Analysis Flow

```
Query → Fetch from DB → Aggregate → Transform → Present
           ↓
      Cache Result
```

1. **Time-range Query**: User requests historical data
2. **Efficient Retrieval**: Time-series optimized queries
3. **On-demand Aggregation**: Calculate trends, averages
4. **Result Caching**: Cache frequently accessed computations

## Key Architectural Decisions

### 1. Rust Language Choice

- **Memory Safety**: Prevents common security vulnerabilities
- **Performance**: Zero-cost abstractions for system programming
- **Concurrency**: Fearless concurrency with ownership model
- **Cross-platform**: Single codebase for all platforms

### 2. Event-Driven Architecture

- **Scalability**: Handles varying network loads efficiently
- **Decoupling**: Components communicate via message passing
- **Testability**: Easy to test components in isolation
- **Extensibility**: New analyzers can be added without core changes

### 3. Time-Series Storage

- **Optimized for Writes**: High-frequency data insertion
- **Compression**: Efficient storage of repetitive data
- **Retention Policies**: Automatic data lifecycle management
- **Query Performance**: Fast time-range queries

### 4. Plugin Architecture

```rust
pub trait Analyzer: Send + Sync {
    fn analyze(&self, packet: &Packet) -> AnalysisResult;
    fn name(&self) -> &str;
}

pub trait Exporter: Send + Sync {
    fn export(&self, data: &UsageData) -> Result<Vec<u8>>;
    fn format(&self) -> ExportFormat;
}
```

- **Extensibility**: Easy to add new analyzers or exporters
- **Isolation**: Plugins run in isolated contexts
- **Hot-reloading**: Future support for dynamic plugin loading

## Performance Considerations

### Memory Management

- **Bounded Channels**: Prevent unbounded memory growth
- **Object Pools**: Reuse allocations for packet processing
- **Lazy Loading**: Load historical data on-demand
- **Memory Mapping**: Efficient file I/O for large datasets

### CPU Optimization

- **SIMD Operations**: Vectorized operations for statistics
- **Work Stealing**: Tokio's scheduler for CPU utilization
- **Batch Processing**: Amortize overhead across multiple items
- **Profile-Guided**: Optimization based on real usage patterns

### Network I/O

- **Zero-Copy**: Direct memory access for packet capture
- **Ring Buffers**: Efficient producer-consumer patterns
- **Kernel Bypass**: Optional high-performance packet capture
- **Selective Capture**: BPF filters to reduce processing

## Security Architecture

### Privilege Management

```
┌─────────────────────────┐
│   Privileged Process    │
│  (Packet Capture Only)  │
└───────────┬─────────────┘
            │ Unix Socket
            ▼
┌─────────────────────────┐
│  Unprivileged Process   │
│   (Main Application)    │
└─────────────────────────┘
```

- **Privilege Separation**: Minimal code runs with elevated privileges
- **Capability-based**: Linux capabilities instead of full root
- **Sandboxing**: Optional seccomp filters for hardening

### Data Protection

- **Local Storage Only**: No cloud connectivity by default
- **Encryption at Rest**: Optional AES-256 for sensitive data
- **Memory Protection**: Secure erasure of sensitive buffers
- **Access Control**: File permissions for database files

## Scalability Patterns

### Horizontal Scaling

- **Multi-Interface**: Monitor multiple network interfaces
- **Sharding**: Partition data by time or interface
- **Read Replicas**: Separate read and write paths

### Vertical Scaling

- **Thread Pool Sizing**: Adaptive based on CPU cores
- **Buffer Tuning**: Configurable based on available memory
- **Batch Sizes**: Dynamic adjustment based on load

## Integration Points

### External Systems

- **Syslog Export**: Integration with logging infrastructure
- **Prometheus Metrics**: Export statistics for monitoring
- **Webhook Alerts**: HTTP callbacks for notifications
- **API Server**: Optional REST API for remote access

### Platform Integration

- **systemd**: Service management on Linux
- **launchd**: Background service on macOS
- **Windows Service**: Native Windows service support
- **Container Support**: Docker/Podman compatibility

## Future Architecture Evolution

### Planned Enhancements

1. **Distributed Monitoring**: Multi-node deployment support
2. **Machine Learning**: Anomaly detection improvements
3. **Cloud Sync**: Optional encrypted cloud backup
4. **Mobile Companion**: Remote monitoring capabilities

### Extension Points

- **Custom Protocols**: User-defined protocol analyzers
- **Alert Plugins**: Custom notification channels
- **UI Themes**: Customizable dashboard layouts
- **Data Sources**: Additional input beyond packet capture

