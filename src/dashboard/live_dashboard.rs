use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

use crate::collectors::{bandwidth_collector::*, BandwidthCollector};

/// Real-time terminal dashboard for network monitoring
/// Displays live bandwidth statistics using ratatui
pub struct Dashboard {
    /// Collector for gathering network statistics
    bandwidth_collector: BandwidthCollector,
    /// How often to refresh the display
    update_interval: Duration,
    /// Optional filter to show only specific interface
    interface_filter: Option<String>,
}

impl Dashboard {
    /// Creates a new dashboard instance
    pub fn new(update_interval: u64, interface_filter: Option<String>) -> Self {
        Self {
            bandwidth_collector: BandwidthCollector::new(),
            update_interval: Duration::from_secs(update_interval),
            interface_filter,
        }
    }

    /// Main entry point for the dashboard
    /// Sets up terminal, runs the UI loop, and cleans up on exit
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal for full-screen UI
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run the main application loop
        let res = self.run_app(&mut terminal).await;

        // Cleanup terminal state before exiting
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            eprintln!("Error: {err:?}");
        }

        Ok(())
    }

    /// Main application loop
    /// Handles UI rendering, keyboard input, and periodic data updates
    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> 
    where
        B::Error: Send + Sync + 'static,
    {
        let mut last_update = Instant::now();

        loop {
            // Render the current UI state
            terminal.draw(|f| self.ui(f))?;

            // Check for keyboard input (non-blocking with 100ms timeout)
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        // Exit on 'q' or Escape key
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        _ => {}
                    }
                }
            }

            // Update network data at the specified interval
            if last_update.elapsed() >= self.update_interval {
                self.bandwidth_collector.collect()?;
                last_update = Instant::now();
            }
        }
    }

    /// Main UI layout function
    /// Divides the terminal into sections and renders each component
    fn ui(&mut self, frame: &mut Frame) {
        // Create a 4-section vertical layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),   // Header section
                    Constraint::Length(5),   // Current speed section
                    Constraint::Min(10),     // Interface list (takes remaining space)
                    Constraint::Length(3),   // Footer section
                ]
                .as_ref(),
            )
            .split(frame.area());

        // Render each section
        self.render_header(frame, chunks[0]);
        self.render_current_speed(frame, chunks[1]);
        self.render_interface_list(frame, chunks[2]);
        self.render_footer(frame, chunks[3]);
    }

    /// Renders the header section with title and current timestamp
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let header = vec![Line::from(vec![
            Span::raw("Internet Monitor - Live Dashboard"),
            Span::raw("    "),
            Span::styled(
                Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ])];

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));
        
        let paragraph = Paragraph::new(header).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Renders current network speed and total usage statistics
    fn render_current_speed(&mut self, frame: &mut Frame, area: Rect) {
        // Get latest network statistics
        let stats = match self.bandwidth_collector.collect() {
            Ok(stats) => stats,
            Err(_) => vec![],
        };

        // Calculate total speeds across all interfaces
        let total_download: f64 = stats.iter().map(|s| s.download_speed_bps).sum();
        let total_upload: f64 = stats.iter().map(|s| s.upload_speed_bps).sum();

        let speed_text = vec![
            Line::from(vec![
                Span::raw("Current Speed: "),
                Span::styled(
                    format!("↓ {}", format_speed(total_download)),
                    Style::default().fg(Color::Green),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("↑ {}", format_speed(total_upload)),
                    Style::default().fg(Color::Blue),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Total Usage: "),
                Span::raw(format!(
                    "↓ {} ↑ {}",
                    format_bytes(self.bandwidth_collector.get_total_bandwidth().0),
                    format_bytes(self.bandwidth_collector.get_total_bandwidth().1)
                )),
            ]),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Network Statistics")
            .style(Style::default().fg(Color::White));

        let paragraph = Paragraph::new(speed_text).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Renders the list of network interfaces with their statistics
    /// Applies interface filter if specified
    fn render_interface_list(&mut self, frame: &mut Frame, area: Rect) {
        // Get latest network statistics
        let stats = match self.bandwidth_collector.collect() {
            Ok(stats) => stats,
            Err(_) => vec![],
        };

        // Create list items for each interface (filtered if needed)
        let items: Vec<ListItem> = stats
            .iter()
            .filter(|s| {
                // Apply interface filter if specified
                self.interface_filter.as_ref()
                    .map(|f| s.interface_name.contains(f))
                    .unwrap_or(true)
            })
            .map(|stat| {
                let content = vec![Line::from(vec![
                    Span::styled(
                        format!("{:<15}", stat.interface_name),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(format!(
                        " ↓ {:<12} ↑ {:<12} Packets: ↓ {} ↑ {}",
                        format_speed(stat.download_speed_bps),
                        format_speed(stat.upload_speed_bps),
                        stat.packets_received,
                        stat.packets_sent
                    )),
                ])];
                ListItem::new(content)
            })
            .collect();

        let interfaces = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Network Interfaces"),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(interfaces, area);
    }

    /// Renders the footer with keyboard shortcuts
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer = Paragraph::new("Press 'q' or ESC to quit")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));

        frame.render_widget(footer, area);
    }
}