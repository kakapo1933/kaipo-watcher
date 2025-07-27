use crate::graphs::{GraphConfig, GraphRenderer};
use crate::cli::graph_commands::DatabaseManager;
use anyhow::Result;
use chrono::{DateTime, Utc};
use plotters::prelude::*;
use std::collections::HashMap;
use std::path::Path;

pub struct ProtocolGraph {
    pub config: GraphConfig,
    pub data: Vec<ProtocolDataPoint>,
    pub interface: Option<String>,
}

#[derive(Clone)]
pub struct ProtocolDataPoint {
    pub timestamp: DateTime<Utc>,
    pub protocol: String,
    pub packet_count: u64,
    pub byte_count: u64,
}

impl ProtocolGraph {
    pub fn new(config: GraphConfig) -> Self {
        Self {
            config,
            data: Vec::new(),
            interface: None,
        }
    }

    pub async fn load_data(
        &mut self,
        db: &DatabaseManager,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        interface: Option<String>,
    ) -> Result<()> {
        self.interface = interface.clone();
        
        let query = if let Some(ref _iface) = interface {
            "SELECT timestamp, protocol_name, packet_count, byte_count 
             FROM protocol_distribution 
             WHERE interface_name = ? AND timestamp BETWEEN ? AND ?
             ORDER BY timestamp, protocol_name"
        } else {
            "SELECT timestamp, protocol_name, SUM(packet_count) as packet_count, SUM(byte_count) as byte_count
             FROM protocol_distribution 
             WHERE timestamp BETWEEN ? AND ?
             GROUP BY timestamp, protocol_name
             ORDER BY timestamp, protocol_name"
        };

        let conn = db.connection.lock().unwrap();
        let mut stmt = conn.prepare(query)?;
        
        let mapper = |row: &rusqlite::Row| {
            Ok(ProtocolDataPoint {
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(0)?)
                    .unwrap()
                    .with_timezone(&Utc),
                protocol: row.get(1)?,
                packet_count: row.get(2)?,
                byte_count: row.get(3)?,
            })
        };
        
        let rows = if let Some(ref iface) = interface {
            stmt.query_map(
                [iface.as_str(), &start_time.to_rfc3339(), &end_time.to_rfc3339()],
                mapper,
            )?
        } else {
            stmt.query_map(
                [&start_time.to_rfc3339(), &end_time.to_rfc3339()],
                mapper,
            )?
        };

        self.data = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    pub fn render_pie_chart(&self, output_path: &Path) -> Result<()> {
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let title = if let Some(ref iface) = self.interface {
            format!("Protocol Distribution - {iface}")
        } else {
            "Protocol Distribution".to_string()
        };

        let mut protocol_totals: HashMap<String, u64> = HashMap::new();
        for data_point in &self.data {
            *protocol_totals.entry(data_point.protocol.clone()).or_insert(0) += data_point.packet_count;
        }

        let mut sorted_protocols: Vec<_> = protocol_totals.into_iter().collect();
        sorted_protocols.sort_by(|a, b| b.1.cmp(&a.1));

        let total_packets: u64 = sorted_protocols.iter().map(|(_, count)| count).sum();
        
        let colors = [&RED, &BLUE, &GREEN, &MAGENTA, &CYAN, &BLACK];
        let mut chart = ChartBuilder::on(&root)
            .caption(&title, ("sans-serif", 50).into_font())
            .margin(10)
            .build_cartesian_2d(-1f32..1f32, -1f32..1f32)?;

        let mut current_angle = 0.0;
        for (i, (_protocol, count)) in sorted_protocols.iter().enumerate() {
            let percentage = *count as f64 / total_packets as f64;
            let end_angle = current_angle + percentage * 360.0;
            
            let color = colors[i % colors.len()];
            
            chart.draw_series(std::iter::once(Circle::new(
                (0.0, 0.0),
                0.8,
                color.filled(),
            )))?;
            
            current_angle = end_angle;
        }

        Ok(())
    }

    pub fn render_bar_chart(&self, output_path: &Path) -> Result<()> {
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let title = if let Some(ref iface) = self.interface {
            format!("Protocol Usage - {iface}")
        } else {
            "Protocol Usage".to_string()
        };

        let mut protocol_totals: HashMap<String, u64> = HashMap::new();
        for data_point in &self.data {
            *protocol_totals.entry(data_point.protocol.clone()).or_insert(0) += data_point.packet_count;
        }

        let mut sorted_protocols: Vec<_> = protocol_totals.into_iter().collect();
        sorted_protocols.sort_by(|a, b| b.1.cmp(&a.1));
        sorted_protocols.truncate(10);

        let max_count = sorted_protocols.first().map(|(_, count)| *count).unwrap_or(0);

        let mut chart = ChartBuilder::on(&root)
            .caption(&title, ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(60)
            .y_label_area_size(60)
            .build_cartesian_2d(
                0f64..sorted_protocols.len() as f64,
                0u64..max_count,
            )?;

        chart
            .configure_mesh()
            .x_desc("Protocol")
            .y_desc("Packet Count")
            .x_label_formatter(&|x| {
                if *x as usize >= sorted_protocols.len() {
                    String::new()
                } else {
                    sorted_protocols[*x as usize].0.clone()
                }
            })
            .draw()?;

        chart.draw_series(
            sorted_protocols.iter().enumerate().map(|(i, (_protocol, count))| {
                Rectangle::new([(i as f64, 0), (i as f64 + 0.8, *count)], BLUE.filled())
            })
        )?;

        Ok(())
    }

    pub fn render_timeline_chart(&self, output_path: &Path) -> Result<()> {
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let title = if let Some(ref iface) = self.interface {
            format!("Protocol Timeline - {iface}")
        } else {
            "Protocol Timeline".to_string()
        };

        let protocols: Vec<String> = self.data.iter()
            .map(|d| d.protocol.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let max_count = self.data.iter().map(|d| d.packet_count).max().unwrap_or(0);

        let mut chart = ChartBuilder::on(&root)
            .caption(&title, ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(
                self.data.first().map(|d| d.timestamp).unwrap_or_else(Utc::now)
                    ..self.data.last().map(|d| d.timestamp).unwrap_or_else(Utc::now),
                0u64..max_count,
            )?;

        chart
            .configure_mesh()
            .x_desc("Time")
            .y_desc("Packet Count")
            .draw()?;

        let colors = [&RED, &BLUE, &GREEN, &MAGENTA, &CYAN, &BLACK];
        
        for (i, protocol) in protocols.iter().enumerate() {
            let protocol_data: Vec<_> = self.data.iter()
                .filter(|d| d.protocol == *protocol)
                .collect();
            
            let color = colors[i % colors.len()];
            
            chart
                .draw_series(LineSeries::new(
                    protocol_data.iter().map(|d| (d.timestamp, d.packet_count)),
                    color,
                ))?
                .label(protocol)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], *color));
        }

        chart.configure_series_labels().draw()?;

        Ok(())
    }

    pub fn get_protocol_summary(&self) -> HashMap<String, ProtocolSummary> {
        let mut summaries: HashMap<String, ProtocolSummary> = HashMap::new();
        
        for data_point in &self.data {
            let summary = summaries.entry(data_point.protocol.clone()).or_insert(ProtocolSummary {
                protocol: data_point.protocol.clone(),
                total_packets: 0,
                total_bytes: 0,
                first_seen: data_point.timestamp,
                last_seen: data_point.timestamp,
            });
            
            summary.total_packets += data_point.packet_count;
            summary.total_bytes += data_point.byte_count;
            summary.first_seen = summary.first_seen.min(data_point.timestamp);
            summary.last_seen = summary.last_seen.max(data_point.timestamp);
        }
        
        summaries
    }
}

#[derive(Debug, Clone)]
pub struct ProtocolSummary {
    #[allow(dead_code)]
    pub protocol: String,
    pub total_packets: u64,
    pub total_bytes: u64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl GraphRenderer for ProtocolGraph {
    fn render(&self, output_path: &Path) -> Result<()> {
        self.render_bar_chart(output_path)
    }
}

#[allow(dead_code)]
pub fn create_protocol_sparkline(data: &[ProtocolDataPoint], protocol: &str) -> String {
    let protocol_data: Vec<_> = data.iter()
        .filter(|d| d.protocol == protocol)
        .collect();
    
    if protocol_data.is_empty() {
        return "No data".to_string();
    }
    
    let counts: Vec<u64> = protocol_data.iter().map(|d| d.packet_count).collect();
    let max_count = counts.iter().max().unwrap_or(&0);
    
    if *max_count == 0 {
        return "▁".repeat(protocol_data.len());
    }
    
    let sparkline_chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    counts.iter()
        .map(|&count| {
            let normalized = (count as f64 / *max_count as f64 * 7.0) as usize;
            sparkline_chars[normalized.min(7)]
        })
        .collect()
}