use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, info, warn, error};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Sparkline},
    Frame, Terminal,
};
use std::{
    collections::VecDeque,
    io,
    time::{Duration, Instant},
};

use crate::collectors::{
    bandwidth_collector::{format_bytes, format_speed, BandwidthStats, CalculationConfidence, BandwidthError},
    BandwidthCollector,
};

/// Real-time terminal dashboard for network monitoring
/// Displays live bandwidth statistics using ratatui with enhanced error handling and confidence indicators
pub struct Dashboard {
    /// Collector for gathering network statistics
    bandwidth_collector: BandwidthCollector,
    /// How often to refresh the display
    update_interval: Duration,
    /// Optional filter to show only specific interface
    interface_filter: Option<String>,
    /// Historical data for sparkline graphs (actual speed values)
    download_history: VecDeque<f64>,
    upload_history: VecDeque<f64>,
    /// Current bandwidth statistics (cached for UI rendering)
    current_stats: Vec<BandwidthStats>,
    /// Error message to display in UI
    error_message: Option<String>,
    /// Initialization state tracking
    is_initialized: bool,
    /// Number of successful collections for baseline establishment
    successful_collections: u32,
    /// Last successful collection time
    last_successful_collection: Option<Instant>,
    /// Show only important interfaces
    important_only: bool,
    /// Show all interfaces including virtual
    show_all: bool,
}

impl Dashboard {
    /// Creates a new dashboard instance with enhanced initialization
    pub fn new(update_interval: u64, interface_filter: Option<String>, important_only: bool, show_all: bool) -> Self {
        Self {
            bandwidth_collector: BandwidthCollector::new(),
            update_interval: Duration::from_secs(update_interval),
            interface_filter,
            download_history: VecDeque::with_capacity(50),
            upload_history: VecDeque::with_capacity(50),
            current_stats: Vec::new(),
            error_message: None,
            is_initialized: false,
            successful_collections: 0,
            last_successful_collection: None,
            important_only,
            show_all,
        }
    }

    /// Collects bandwidth data using the appropriate filtering method
    /// Uses the filtering mode specified when creating the dashboard
    fn collect_bandwidth_data(&mut self) -> Result<Vec<BandwidthStats>> {
        if self.show_all {
            // Show all interfaces including virtual and system interfaces
            self.bandwidth_collector.collect()
        } else if self.important_only {
            // Show only important interfaces (physical ethernet, wifi, VPN)
            self.bandwidth_collector.collect_important()
        } else {
            // Use default filtering for dashboard to provide a clean view
            // This excludes most virtual interfaces while keeping important ones like VPN
            self.bandwidth_collector.collect_default()
        }
    }

    /// Main entry point for the dashboard
    /// Sets up terminal, runs the UI loop, and cleans up on exit
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting live dashboard with update interval: {}s", self.update_interval.as_secs());
        if let Some(ref filter) = self.interface_filter {
            info!("Dashboard filtering to interface: '{}'", filter);
        }

        // Setup terminal for full-screen UI
        debug!("Setting up terminal for full-screen UI");
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run the main application loop
        let res = self.run_app(&mut terminal).await;

        // Cleanup terminal state before exiting
        debug!("Cleaning up terminal state");
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            error!("Dashboard error: {err:?}");
            eprintln!("Error: {err:?}");
        } else {
            info!("Dashboard exited normally");
        }

        Ok(())
    }

    /// Main application loop with enhanced error handling and initialization
    /// Handles UI rendering, keyboard input, and periodic data updates
    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> 
    where
        B::Error: Send + Sync + 'static,
    {
        let mut last_update = Instant::now();
        
        // Perform initial baseline collection to establish proper speed calculation
        self.perform_initialization().await;

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
                self.update_bandwidth_data();
                last_update = Instant::now();
            }
        }
    }

    /// Performs proper initialization sequence to establish baseline readings
    /// This is critical for accurate speed calculations from the start
    async fn perform_initialization(&mut self) {
        info!("Starting dashboard initialization sequence");
        let init_start = Instant::now();

        // Take initial baseline reading
        debug!("Taking initial baseline reading");
        match self.collect_bandwidth_data() {
            Ok(initial_stats) => {
                info!("Initial baseline reading successful: {} interfaces found", initial_stats.len());
                self.successful_collections = 1;
                self.error_message = Some("Initializing... Taking baseline reading".to_string());
            }
            Err(e) => {
                error!("Dashboard initialization failed on first reading: {}", e);
                
                // Create comprehensive error context for better user guidance
                let error_message = if let Some(bandwidth_error) = e.downcast_ref::<BandwidthError>() {
                    let error_context = self.bandwidth_collector.create_error_context_report(bandwidth_error);
                    
                    // Log detailed error context for debugging
                    debug!("Dashboard initialization error context: {:#?}", error_context);
                    
                    // Create user-friendly message with context-aware guidance
                    match error_context.system_impact {
                        crate::collectors::bandwidth_collector::SystemImpact::Critical => {
                            format!("🚨 Critical Error:\n{}\n\nImmediate actions:\n{}", 
                                error_context.user_friendly_message,
                                error_context.suggested_actions.iter()
                                    .take(3)
                                    .map(|a| format!("• {}", a))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            )
                        }
                        _ => {
                            format!("⚠️  Initialization Error:\n{}\n\nTry:\n• Waiting a moment and restarting\n• Running with administrator privileges", 
                                error_context.user_friendly_message)
                        }
                    }
                } else {
                    format!("Failed to initialize network monitoring: {}\n\nTry:\n• Checking your network connections\n• Running with administrator privileges\n• Waiting a moment and restarting", e)
                };
                
                self.error_message = Some(error_message);
                return;
            }
        }

        // Wait a short period to allow for proper speed calculation
        debug!("Waiting 1 second for baseline establishment");
        tokio::time::sleep(Duration::from_millis(1000)).await;

        // Take second reading to enable speed calculation
        debug!("Taking second reading for speed calculation");
        match self.collect_bandwidth_data() {
            Ok(stats) => {
                let init_duration = init_start.elapsed();
                info!("Dashboard initialization completed successfully in {:.3}ms: {} interfaces ready", 
                      init_duration.as_secs_f64() * 1000.0, stats.len());
                
                self.current_stats = stats;
                self.successful_collections = 2;
                self.is_initialized = true;
                self.error_message = None;
                self.last_successful_collection = Some(Instant::now());
                
                // Log interface summary for debugging
                if log::log_enabled!(log::Level::Debug) {
                    for stat in &self.current_stats {
                        debug!("Interface '{}': type={:?}, state={:?}, confidence={:?}", 
                               stat.interface_name, stat.interface_type, stat.interface_state, stat.calculation_confidence);
                    }
                }
            }
            Err(e) => {
                warn!("Dashboard initialization: second reading failed: {}", e);
                
                // Create context-aware error message for second reading failure
                let error_message = if let Some(bandwidth_error) = e.downcast_ref::<BandwidthError>() {
                    let error_context = self.bandwidth_collector.create_error_context_report(bandwidth_error);
                    debug!("Dashboard second reading error context: {:#?}", error_context);
                    
                    format!("⚠️  Initialization Warning:\n{}\n\nThe dashboard will continue trying to collect data.", 
                        error_context.user_friendly_message)
                } else {
                    format!("Failed to complete initialization: {}\n\nThe dashboard will continue trying to collect data.", e)
                };
                
                self.error_message = Some(error_message);
                // Don't return here - allow dashboard to continue and retry
            }
        }
    }

    /// Updates bandwidth data with graceful error handling
    /// Does not crash the dashboard on collection errors
    fn update_bandwidth_data(&mut self) {
        match self.collect_bandwidth_data() {
            Ok(stats) => {
                // Successful collection - update data and clear any error
                self.current_stats = stats;
                self.error_message = None;
                self.successful_collections += 1;
                self.last_successful_collection = Some(Instant::now());
                
                // Update historical data for sparklines with actual speed values
                let total_download: f64 = self.current_stats.iter().map(|s| s.download_speed_bps).sum();
                let total_upload: f64 = self.current_stats.iter().map(|s| s.upload_speed_bps).sum();
                
                self.download_history.push_back(total_download);
                self.upload_history.push_back(total_upload);
                
                // Keep only last 50 data points
                if self.download_history.len() > 50 {
                    self.download_history.pop_front();
                }
                if self.upload_history.len() > 50 {
                    self.upload_history.pop_front();
                }
            }
            Err(e) => {
                // Collection failed - set error message but don't crash
                self.error_message = Some(format!("Collection error: {}", e));
                // Keep using previous stats if available
            }
        }
    }

    /// Main UI layout function with enhanced error display
    /// Divides the terminal into sections and renders each component
    fn ui(&mut self, frame: &mut Frame) {
        // Create a 6-section vertical layout to include error/status section
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),   // Header section
                    Constraint::Length(3),   // Status/Error section
                    Constraint::Length(5),   // Current speed section
                    Constraint::Length(5),   // Sparkline graphs section
                    Constraint::Min(10),     // Interface list (takes remaining space)
                    Constraint::Length(3),   // Footer section
                ]
                .as_ref(),
            )
            .split(frame.area());

        // Render each section
        self.render_header(frame, chunks[0]);
        self.render_status(frame, chunks[1]);
        self.render_current_speed(frame, chunks[2]);
        self.render_sparklines(frame, chunks[3]);
        self.render_interface_list(frame, chunks[4]);
        self.render_footer(frame, chunks[5]);
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

    /// Renders status/error information section
    fn render_status(&self, frame: &mut Frame, area: Rect) {
        let status_text = if let Some(error) = &self.error_message {
            vec![Line::from(vec![
                Span::styled("⚠ ", Style::default().fg(Color::Yellow)),
                Span::styled(error.clone(), Style::default().fg(Color::Yellow)),
            ])]
        } else if !self.is_initialized {
            vec![Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(Color::Blue)),
                Span::raw("Initializing bandwidth monitoring..."),
            ])]
        } else {
            let collections_text = format!("Collections: {} | Last update: {}", 
                self.successful_collections,
                self.last_successful_collection
                    .map(|t| format!("{:.1}s ago", t.elapsed().as_secs_f32()))
                    .unwrap_or_else(|| "Never".to_string())
            );
            vec![Line::from(vec![
                Span::styled("✓ ", Style::default().fg(Color::Green)),
                Span::raw("Monitoring active | "),
                Span::styled(collections_text, Style::default().fg(Color::DarkGray)),
            ])]
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Status")
            .style(Style::default().fg(Color::White));

        let paragraph = Paragraph::new(status_text).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Renders current network speed and total usage statistics with confidence indicators
    fn render_current_speed(&mut self, frame: &mut Frame, area: Rect) {
        // Use cached stats instead of calling collect() again
        let stats = &self.current_stats;

        // Calculate total speeds across all interfaces
        let total_download: f64 = stats.iter().map(|s| s.download_speed_bps).sum();
        let total_upload: f64 = stats.iter().map(|s| s.upload_speed_bps).sum();

        // Calculate overall confidence level
        let overall_confidence = self.calculate_overall_confidence(stats);
        let confidence_indicator = self.get_confidence_indicator(&overall_confidence);
        let confidence_color = self.get_confidence_color(&overall_confidence);

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
                Span::raw("  "),
                Span::styled(
                    confidence_indicator,
                    Style::default().fg(confidence_color),
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

    /// Renders the list of network interfaces with their statistics and confidence indicators
    /// Applies interface filter if specified
    fn render_interface_list(&mut self, frame: &mut Frame, area: Rect) {
        // Use cached stats instead of calling collect() again
        let stats = &self.current_stats;

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
                let confidence_indicator = self.get_confidence_indicator(&stat.calculation_confidence);
                let confidence_color = self.get_confidence_color(&stat.calculation_confidence);
                
                let content = vec![Line::from(vec![
                    Span::styled(
                        format!("{:<15}", stat.interface_name),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(format!(
                        " ↓ {:<12} ↑ {:<12}",
                        format_speed(stat.download_speed_bps),
                        format_speed(stat.upload_speed_bps),
                    )),
                    Span::raw(" "),
                    Span::styled(
                        confidence_indicator,
                        Style::default().fg(confidence_color),
                    ),
                    Span::raw(format!(
                        " | Packets: ↓ {} ↑ {}",
                        stat.packets_received,
                        stat.packets_sent
                    )),
                ])];
                ListItem::new(content)
            })
            .collect();

        let title = if items.is_empty() {
            "Network Interfaces (No data available)"
        } else {
            "Network Interfaces"
        };

        let interfaces = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(interfaces, area);
    }

    /// Renders sparkline graphs for bandwidth trends using actual speed values
    fn render_sparklines(&self, frame: &mut Frame, area: Rect) {
        // Split the area into two columns for download and upload sparklines
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        // Convert f64 speed values to u64 for sparkline (ratatui requirement)
        // Scale values to make them more visible in sparkline
        let download_data: Vec<u64> = self.download_history
            .iter()
            .map(|&speed| (speed / 1024.0) as u64) // Convert to KB/s for better scaling
            .collect();
            
        let upload_data: Vec<u64> = self.upload_history
            .iter()
            .map(|&speed| (speed / 1024.0) as u64) // Convert to KB/s for better scaling
            .collect();

        // Calculate max values for better scaling
        let max_download = download_data.iter().max().copied().unwrap_or(1);
        let max_upload = upload_data.iter().max().copied().unwrap_or(1);

        // Download sparkline with current value display
        let download_title = if let Some(&current) = self.download_history.back() {
            format!("Download Trend (Current: {})", format_speed(current))
        } else {
            "Download Trend (No data)".to_string()
        };

        let download_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(download_title)
                    .title_style(Style::default().fg(Color::Green))
            )
            .data(&download_data)
            .max(max_download.max(1)) // Ensure max is at least 1
            .style(Style::default().fg(Color::Green));

        // Upload sparkline with current value display
        let upload_title = if let Some(&current) = self.upload_history.back() {
            format!("Upload Trend (Current: {})", format_speed(current))
        } else {
            "Upload Trend (No data)".to_string()
        };

        let upload_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(upload_title)
                    .title_style(Style::default().fg(Color::Blue))
            )
            .data(&upload_data)
            .max(max_upload.max(1)) // Ensure max is at least 1
            .style(Style::default().fg(Color::Blue));

        frame.render_widget(download_sparkline, chunks[0]);
        frame.render_widget(upload_sparkline, chunks[1]);
    }

    /// Renders the footer with keyboard shortcuts
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer = Paragraph::new("Press 'q' or ESC to quit")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));

        frame.render_widget(footer, area);
    }

    /// Calculates overall confidence level across all interfaces
    /// Returns the lowest confidence level found, as overall reliability is limited by the weakest link
    fn calculate_overall_confidence(&self, stats: &[BandwidthStats]) -> CalculationConfidence {
        if stats.is_empty() {
            return CalculationConfidence::None;
        }

        // Find the lowest confidence level among all interfaces
        let mut overall_confidence = CalculationConfidence::High;
        
        for stat in stats {
            match (&overall_confidence, &stat.calculation_confidence) {
                (_, CalculationConfidence::None) => overall_confidence = CalculationConfidence::None,
                (CalculationConfidence::High, CalculationConfidence::Low) => overall_confidence = CalculationConfidence::Low,
                (CalculationConfidence::High, CalculationConfidence::Medium) => overall_confidence = CalculationConfidence::Medium,
                (CalculationConfidence::Medium, CalculationConfidence::Low) => overall_confidence = CalculationConfidence::Low,
                _ => {} // Keep current confidence level
            }
        }

        overall_confidence
    }

    /// Returns a visual indicator for confidence level
    fn get_confidence_indicator(&self, confidence: &CalculationConfidence) -> String {
        match confidence {
            CalculationConfidence::High => "●●●".to_string(),    // High confidence - 3 dots
            CalculationConfidence::Medium => "●●○".to_string(),  // Medium confidence - 2 dots
            CalculationConfidence::Low => "●○○".to_string(),     // Low confidence - 1 dot
            CalculationConfidence::None => "○○○".to_string(),    // No confidence - empty dots
        }
    }

    /// Returns appropriate color for confidence level
    fn get_confidence_color(&self, confidence: &CalculationConfidence) -> Color {
        match confidence {
            CalculationConfidence::High => Color::Green,
            CalculationConfidence::Medium => Color::Yellow,
            CalculationConfidence::Low => Color::Red,
            CalculationConfidence::None => Color::DarkGray,
        }
    }
}