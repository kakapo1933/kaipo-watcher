use crate::graphs::{GraphConfig, GraphRenderer};
use crate::cli::graph_commands::DatabaseManager;
use chrono::Timelike;
use anyhow::Result;
use chrono::{DateTime, Utc};
use plotters::prelude::*;
use std::collections::HashMap;
use std::path::Path;

pub struct ConnectionGraph {
    pub config: GraphConfig,
    pub data: Vec<ConnectionDataPoint>,
    pub interface: Option<String>,
}

#[derive(Clone)]
pub struct ConnectionDataPoint {
    pub timestamp: DateTime<Utc>,
    pub source_ip: String,
    pub dest_ip: String,
    pub source_port: u16,
    pub dest_port: u16,
    pub protocol: String,
    #[allow(dead_code)]
    pub state: String,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

impl ConnectionGraph {
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
            "SELECT first_seen, source_ip, dest_ip, source_port, dest_port, protocol, 'active', 
                    packet_count, packet_count, byte_count, byte_count
             FROM connections 
             WHERE first_seen BETWEEN ? AND ?
             ORDER BY first_seen"
        } else {
            "SELECT first_seen, source_ip, dest_ip, source_port, dest_port, protocol, 'active',
                    packet_count, packet_count, byte_count, byte_count
             FROM connections 
             WHERE first_seen BETWEEN ? AND ?
             ORDER BY first_seen"
        };

        let conn = db.connection.lock().unwrap();
        let mut stmt = conn.prepare(query)?;
        
        let mapper = |row: &rusqlite::Row| {
            Ok(ConnectionDataPoint {
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(0)?)
                    .unwrap()
                    .with_timezone(&Utc),
                source_ip: row.get(1)?,
                dest_ip: row.get(2)?,
                source_port: row.get(3)?,
                dest_port: row.get(4)?,
                protocol: row.get(5)?,
                state: row.get(6)?,
                packets_sent: row.get(7)?,
                packets_received: row.get(8)?,
                bytes_sent: row.get(9)?,
                bytes_received: row.get(10)?,
            })
        };
        
        let rows = stmt.query_map(
            [&start_time.to_rfc3339(), &end_time.to_rfc3339()],
            mapper,
        )?;

        self.data = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    pub fn render_connection_timeline(&self, output_path: &Path) -> Result<()> {
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let title = if let Some(ref iface) = self.interface {
            format!("Connection Timeline - {iface}")
        } else {
            "Connection Timeline".to_string()
        };

        let connections_per_minute = self.get_connections_per_minute();
        let max_connections = connections_per_minute.values().max().unwrap_or(&0);

        let mut chart = ChartBuilder::on(&root)
            .caption(&title, ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(
                self.data.first().map(|d| d.timestamp).unwrap_or_else(Utc::now)
                    ..self.data.last().map(|d| d.timestamp).unwrap_or_else(Utc::now),
                0u64..*max_connections,
            )?;

        chart
            .configure_mesh()
            .x_desc("Time")
            .y_desc("Active Connections")
            .draw()?;

        let timeline_data: Vec<_> = connections_per_minute.iter()
            .map(|(timestamp, count)| (*timestamp, *count))
            .collect();

        chart
            .draw_series(LineSeries::new(timeline_data, &BLUE))?
            .label("Active Connections")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], BLUE));

        chart.configure_series_labels().draw()?;

        Ok(())
    }

    pub fn render_port_distribution(&self, output_path: &Path) -> Result<()> {
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let title = if let Some(ref iface) = self.interface {
            format!("Port Distribution - {iface}")
        } else {
            "Port Distribution".to_string()
        };

        let mut port_counts: HashMap<u16, u64> = HashMap::new();
        for conn in &self.data {
            *port_counts.entry(conn.dest_port).or_insert(0) += 1;
        }

        let mut sorted_ports: Vec<_> = port_counts.into_iter().collect();
        sorted_ports.sort_by(|a, b| b.1.cmp(&a.1));
        sorted_ports.truncate(20);

        let max_count = sorted_ports.first().map(|(_, count)| *count).unwrap_or(0);

        let mut chart = ChartBuilder::on(&root)
            .caption(&title, ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(60)
            .y_label_area_size(60)
            .build_cartesian_2d(
                0f64..sorted_ports.len() as f64,
                0u64..max_count,
            )?;

        chart
            .configure_mesh()
            .x_desc("Port")
            .y_desc("Connection Count")
            .x_label_formatter(&|x| {
                if *x as usize >= sorted_ports.len() {
                    String::new()
                } else {
                    sorted_ports[*x as usize].0.to_string()
                }
            })
            .draw()?;

        chart.draw_series(
            sorted_ports.iter().enumerate().map(|(i, (_port, count))| {
                Rectangle::new([(i as f64, 0), (i as f64 + 0.8, *count)], GREEN.filled())
            })
        )?;

        Ok(())
    }

    pub fn render_traffic_flow(&self, output_path: &Path) -> Result<()> {
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let title = if let Some(ref iface) = self.interface {
            format!("Traffic Flow - {iface}")
        } else {
            "Traffic Flow".to_string()
        };

        let traffic_data = self.get_traffic_over_time();
        let max_traffic = traffic_data.iter().map(|(_, bytes)| *bytes).max().unwrap_or(0);

        let mut chart = ChartBuilder::on(&root)
            .caption(&title, ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(
                self.data.first().map(|d| d.timestamp).unwrap_or_else(Utc::now)
                    ..self.data.last().map(|d| d.timestamp).unwrap_or_else(Utc::now),
                0u64..max_traffic,
            )?;

        chart
            .configure_mesh()
            .x_desc("Time")
            .y_desc("Bytes Transferred")
            .draw()?;

        chart
            .draw_series(LineSeries::new(traffic_data, &RED))?
            .label("Total Traffic")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], RED));

        chart.configure_series_labels().draw()?;

        Ok(())
    }

    fn get_connections_per_minute(&self) -> HashMap<DateTime<Utc>, u64> {
        let mut connections_per_minute = HashMap::new();
        
        for conn in &self.data {
            let minute = conn.timestamp.with_second(0).unwrap().with_nanosecond(0).unwrap();
            *connections_per_minute.entry(minute).or_insert(0) += 1;
        }
        
        connections_per_minute
    }

    fn get_traffic_over_time(&self) -> Vec<(DateTime<Utc>, u64)> {
        let mut traffic_per_minute = HashMap::new();
        
        for conn in &self.data {
            let minute = conn.timestamp.with_second(0).unwrap().with_nanosecond(0).unwrap();
            let total_bytes = conn.bytes_sent + conn.bytes_received;
            *traffic_per_minute.entry(minute).or_insert(0) += total_bytes;
        }
        
        let mut sorted_traffic: Vec<_> = traffic_per_minute.into_iter().collect();
        sorted_traffic.sort_by(|a, b| a.0.cmp(&b.0));
        sorted_traffic
    }

    pub fn get_top_connections(&self, limit: usize) -> Vec<ConnectionSummary> {
        let mut connection_summaries = HashMap::new();
        
        for conn in &self.data {
            let key = format!("{}:{} -> {}:{}", 
                conn.source_ip, conn.source_port, 
                conn.dest_ip, conn.dest_port);
            
            let summary = connection_summaries.entry(key.clone()).or_insert(ConnectionSummary {
                connection_id: key,
                source_ip: conn.source_ip.clone(),
                dest_ip: conn.dest_ip.clone(),
                source_port: conn.source_port,
                dest_port: conn.dest_port,
                protocol: conn.protocol.clone(),
                total_packets: 0,
                total_bytes: 0,
                first_seen: conn.timestamp,
                last_seen: conn.timestamp,
            });
            
            summary.total_packets += conn.packets_sent + conn.packets_received;
            summary.total_bytes += conn.bytes_sent + conn.bytes_received;
            summary.first_seen = summary.first_seen.min(conn.timestamp);
            summary.last_seen = summary.last_seen.max(conn.timestamp);
        }
        
        let mut summaries: Vec<_> = connection_summaries.into_values().collect();
        summaries.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
        summaries.truncate(limit);
        summaries
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionSummary {
    #[allow(dead_code)]
    pub connection_id: String,
    pub source_ip: String,
    pub dest_ip: String,
    pub source_port: u16,
    pub dest_port: u16,
    pub protocol: String,
    pub total_packets: u64,
    pub total_bytes: u64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl GraphRenderer for ConnectionGraph {
    fn render(&self, output_path: &Path) -> Result<()> {
        self.render_connection_timeline(output_path)
    }
}

#[allow(dead_code)]
pub fn get_well_known_port_name(port: u16) -> &'static str {
    match port {
        80 => "HTTP",
        443 => "HTTPS",
        22 => "SSH",
        21 => "FTP",
        25 => "SMTP",
        53 => "DNS",
        110 => "POP3",
        143 => "IMAP",
        993 => "IMAPS",
        995 => "POP3S",
        3389 => "RDP",
        5432 => "PostgreSQL",
        3306 => "MySQL",
        6379 => "Redis",
        27017 => "MongoDB",
        _ => "Unknown",
    }
}