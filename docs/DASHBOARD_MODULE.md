# Dashboard Module Documentation

## Overview

The Dashboard module provides the terminal UI for real-time network monitoring in kaipo-watcher. It has been separated from the CLI module to maintain a clean separation of concerns and improve code organization.

## Module Structure

```
src/dashboard/
├── mod.rs              # Module declaration and exports
└── live_dashboard.rs   # Live dashboard implementation
```

## Architecture

### Dashboard Component

The `Dashboard` struct in `live_dashboard.rs` is responsible for:

1. **Terminal Management**: Setting up and tearing down the terminal UI
2. **Data Collection**: Using `BandwidthCollector` to gather network statistics
3. **UI Rendering**: Drawing the dashboard using Ratatui widgets
4. **Event Handling**: Processing keyboard input for user interaction

### Key Features

- **Real-time Updates**: Configurable refresh interval (default: 1 second)
- **Interface Filtering**: Option to monitor specific network interfaces
- **Non-blocking UI**: Uses async/await with Tokio for responsive interface
- **Cross-platform Terminal**: Uses crossterm for compatibility

### UI Layout

The dashboard is divided into 4 sections:

1. **Header (3 lines)**
   - Application title
   - Current timestamp

2. **Current Speed (5 lines)**
   - Total download/upload speeds
   - Total data usage

3. **Interface List (flexible height)**
   - Per-interface statistics
   - Download/upload speeds
   - Packet counts

4. **Footer (3 lines)**
   - Keyboard shortcuts

### Data Flow

```
BandwidthCollector → Dashboard → Ratatui Terminal
        ↑                ↓              ↓
    Network Stats    UI State      Screen Output
```

## Usage

The dashboard is invoked through the `live` command:

```rust
// In main.rs
Commands::Live { interface, packets: _, interval } => {
    let mut dashboard = Dashboard::new(interval, interface);
    dashboard.run().await?;
}
```

## Dependencies

- **ratatui**: Terminal UI framework
- **crossterm**: Cross-platform terminal manipulation
- **tokio**: Async runtime for non-blocking operations
- **chrono**: Time formatting

## Future Enhancements

1. **Packet Statistics**: Integration with packet collector for protocol-level stats
2. **Alerts**: Visual indicators for bandwidth thresholds
3. **Historical Graphs**: Sparklines or charts for trend visualization
4. **Multiple Views**: Tab-based navigation for different monitoring modes
5. **Export Functionality**: Save dashboard snapshots

## Design Decisions

1. **Separation from CLI**: The dashboard is now a standalone module rather than part of the CLI module, improving modularity.

2. **Async Architecture**: Uses Tokio for non-blocking operations, ensuring the UI remains responsive during data collection.

3. **Bounded Update Rate**: The dashboard updates at a configurable interval to balance real-time feedback with CPU usage.

4. **Stateless Rendering**: Each frame is rendered from scratch based on current data, simplifying the rendering logic.