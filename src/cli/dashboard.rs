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

pub struct Dashboard {
    bandwidth_collector: BandwidthCollector,
    update_interval: Duration,
    interface_filter: Option<String>,
}

impl Dashboard {
    pub fn new(update_interval: u64, interface_filter: Option<String>) -> Self {
        Self {
            bandwidth_collector: BandwidthCollector::new(),
            update_interval: Duration::from_secs(update_interval),
            interface_filter,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.run_app(&mut terminal).await;

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

    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> 
    where
        B::Error: Send + Sync + 'static,
    {
        let mut last_update = Instant::now();

        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        _ => {}
                    }
                }
            }

            if last_update.elapsed() >= self.update_interval {
                self.bandwidth_collector.collect()?;
                last_update = Instant::now();
            }
        }
    }

    fn ui(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(5),
                    Constraint::Min(10),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(frame.area());

        self.render_header(frame, chunks[0]);
        self.render_current_speed(frame, chunks[1]);
        self.render_interface_list(frame, chunks[2]);
        self.render_footer(frame, chunks[3]);
    }

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

    fn render_current_speed(&mut self, frame: &mut Frame, area: Rect) {
        let stats = match self.bandwidth_collector.collect() {
            Ok(stats) => stats,
            Err(_) => vec![],
        };

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

    fn render_interface_list(&mut self, frame: &mut Frame, area: Rect) {
        let stats = match self.bandwidth_collector.collect() {
            Ok(stats) => stats,
            Err(_) => vec![],
        };

        let items: Vec<ListItem> = stats
            .iter()
            .filter(|s| {
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

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer = Paragraph::new("Press 'q' or ESC to quit")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));

        frame.render_widget(footer, area);
    }
}