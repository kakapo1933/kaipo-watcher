use crate::graphs::{GraphConfig, GraphRenderer};
use crate::cli::graph_commands::DatabaseManager;
use anyhow::Result;
use chrono::{DateTime, Utc};
use plotters::prelude::*;
use std::path::Path;

pub struct BandwidthGraph {
    pub config: GraphConfig,
    pub data: Vec<BandwidthDataPoint>,
    pub interface: Option<String>,
}

#[derive(Clone)]
pub struct BandwidthDataPoint {
    pub timestamp: DateTime<Utc>,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub total_rx: u64,
    pub total_tx: u64,
}

impl BandwidthGraph {
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
            "SELECT timestamp, bytes_per_second, bytes_per_second, total_bytes, total_bytes 
             FROM packet_stats 
             WHERE interface_name = ? AND timestamp BETWEEN ? AND ?
             ORDER BY timestamp"
        } else {
            "SELECT timestamp, SUM(bytes_per_second) as bytes_per_second, SUM(bytes_per_second) as bytes_per_second, 
                    SUM(total_bytes) as total_bytes, SUM(total_bytes) as total_bytes
             FROM packet_stats 
             WHERE timestamp BETWEEN ? AND ?
             GROUP BY timestamp
             ORDER BY timestamp"
        };

        let conn = db.connection.lock().unwrap();
        let mut stmt = conn.prepare(query)?;
        
        let mapper = |row: &rusqlite::Row| {
            Ok(BandwidthDataPoint {
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(0)?)
                    .unwrap()
                    .with_timezone(&Utc),
                download_speed: row.get(1)?,
                upload_speed: row.get(2)?,
                total_rx: row.get(3)?,
                total_tx: row.get(4)?,
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

    pub fn render_speed_chart(&self, output_path: &Path) -> Result<()> {
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let title = if let Some(ref iface) = self.interface {
            format!("Bandwidth Usage - {iface}")
        } else {
            "Total Bandwidth Usage".to_string()
        };

        let max_speed = self.data.iter()
            .map(|d| d.download_speed.max(d.upload_speed))
            .fold(0.0, f64::max);

        let mut chart = ChartBuilder::on(&root)
            .caption(&title, ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(
                self.data.first().map(|d| d.timestamp).unwrap_or_else(Utc::now)
                    ..self.data.last().map(|d| d.timestamp).unwrap_or_else(Utc::now),
                0f64..max_speed * 1.1,
            )?;

        chart
            .configure_mesh()
            .x_desc("Time")
            .y_desc("Speed (bytes/s)")
            .draw()?;

        chart
            .draw_series(LineSeries::new(
                self.data.iter().map(|d| (d.timestamp, d.download_speed)),
                &BLUE,
            ))?
            .label("Download Speed")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], BLUE));

        chart
            .draw_series(LineSeries::new(
                self.data.iter().map(|d| (d.timestamp, d.upload_speed)),
                &RED,
            ))?
            .label("Upload Speed")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], RED));

        chart.configure_series_labels().draw()?;
        root.present()?;

        Ok(())
    }

    pub fn render_total_usage_chart(&self, output_path: &Path) -> Result<()> {
        let root = BitMapBackend::new(output_path, (self.config.width, self.config.height))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let title = if let Some(ref iface) = self.interface {
            format!("Total Data Usage - {iface}")
        } else {
            "Total Data Usage".to_string()
        };

        let max_bytes = self.data.iter()
            .map(|d| (d.total_rx as f64).max(d.total_tx as f64))
            .fold(0.0, f64::max);

        let mut chart = ChartBuilder::on(&root)
            .caption(&title, ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(
                self.data.first().map(|d| d.timestamp).unwrap_or_else(Utc::now)
                    ..self.data.last().map(|d| d.timestamp).unwrap_or_else(Utc::now),
                0f64..max_bytes * 1.1,
            )?;

        chart
            .configure_mesh()
            .x_desc("Time")
            .y_desc("Total Bytes")
            .draw()?;

        chart
            .draw_series(LineSeries::new(
                self.data.iter().map(|d| (d.timestamp, d.total_rx as f64)),
                &GREEN,
            ))?
            .label("Total Download")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], GREEN));

        chart
            .draw_series(LineSeries::new(
                self.data.iter().map(|d| (d.timestamp, d.total_tx as f64)),
                &MAGENTA,
            ))?
            .label("Total Upload")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], MAGENTA));

        chart.configure_series_labels().draw()?;
        root.present()?;

        Ok(())
    }
}

impl GraphRenderer for BandwidthGraph {
    fn render(&self, output_path: &Path) -> Result<()> {
        self.render_speed_chart(output_path)
    }
}

#[allow(dead_code)]
pub fn format_bytes(bytes: f64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit_index])
}

#[allow(dead_code)]
pub fn create_bandwidth_sparkline(data: &[BandwidthDataPoint]) -> String {
    if data.is_empty() {
        return "No data".to_string();
    }
    
    let speeds: Vec<f64> = data.iter().map(|d| d.download_speed).collect();
    let max_speed = speeds.iter().fold(0.0f64, |a, &b| a.max(b));
    
    if max_speed == 0.0 {
        return "▁".repeat(data.len());
    }
    
    let sparkline_chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    speeds.iter()
        .map(|&speed| {
            let normalized = (speed / max_speed * 7.0) as usize;
            sparkline_chars[normalized.min(7)]
        })
        .collect()
}