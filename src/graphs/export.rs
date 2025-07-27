use crate::graphs::bandwidth_graphs::BandwidthGraph;
use crate::graphs::protocol_graphs::ProtocolGraph;
use crate::graphs::connection_graphs::ConnectionGraph;
use crate::graphs::GraphRenderer;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub output_path: String,
    pub include_raw_data: bool,
    pub compress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Html,
    Png,
    Svg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub timestamp: DateTime<Utc>,
    pub export_type: String,
    pub interface: Option<String>,
    pub bandwidth_data: Option<BandwidthExportData>,
    pub protocol_data: Option<ProtocolExportData>,
    pub connection_data: Option<ConnectionExportData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthExportData {
    pub summary: BandwidthSummary,
    pub time_series: Vec<BandwidthTimePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthSummary {
    pub total_download: u64,
    pub total_upload: u64,
    pub avg_download_speed: f64,
    pub avg_upload_speed: f64,
    pub peak_download_speed: f64,
    pub peak_upload_speed: f64,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthTimePoint {
    pub timestamp: DateTime<Utc>,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub total_rx: u64,
    pub total_tx: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolExportData {
    pub summary: Vec<ProtocolSummaryData>,
    pub time_series: Vec<ProtocolTimePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolSummaryData {
    pub protocol: String,
    pub total_packets: u64,
    pub total_bytes: u64,
    pub percentage_of_total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolTimePoint {
    pub timestamp: DateTime<Utc>,
    pub protocol: String,
    pub packet_count: u64,
    pub byte_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionExportData {
    pub summary: ConnectionSummaryData,
    pub top_connections: Vec<ConnectionDetail>,
    pub time_series: Vec<ConnectionTimePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSummaryData {
    pub total_connections: u64,
    pub unique_ips: u64,
    pub unique_ports: u64,
    pub most_active_protocol: String,
    pub total_traffic: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDetail {
    pub source_ip: String,
    pub dest_ip: String,
    pub source_port: u16,
    pub dest_port: u16,
    pub protocol: String,
    pub total_packets: u64,
    pub total_bytes: u64,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTimePoint {
    pub timestamp: DateTime<Utc>,
    pub active_connections: u64,
    pub total_traffic: u64,
}

pub struct ExportManager {
    config: ExportConfig,
}

impl ExportManager {
    pub fn new(config: ExportConfig) -> Self {
        Self { config }
    }

    pub fn export_bandwidth_data(&self, graph: &BandwidthGraph) -> Result<()> {
        let export_data = self.prepare_bandwidth_export(graph)?;
        
        match self.config.format {
            ExportFormat::Json => self.export_json(&export_data)?,
            ExportFormat::Csv => self.export_bandwidth_csv(graph)?,
            ExportFormat::Html => self.export_bandwidth_html(&export_data)?,
            ExportFormat::Png => graph.render(Path::new(&self.config.output_path))?,
            ExportFormat::Svg => {
                // SVG export would require different backend
                return Err(anyhow::anyhow!("SVG export not yet implemented"));
            }
        }
        
        Ok(())
    }

    pub fn export_protocol_data(&self, graph: &ProtocolGraph) -> Result<()> {
        let export_data = self.prepare_protocol_export(graph)?;
        
        match self.config.format {
            ExportFormat::Json => self.export_json(&export_data)?,
            ExportFormat::Csv => self.export_protocol_csv(graph)?,
            ExportFormat::Html => self.export_protocol_html(&export_data)?,
            ExportFormat::Png => graph.render(Path::new(&self.config.output_path))?,
            ExportFormat::Svg => {
                return Err(anyhow::anyhow!("SVG export not yet implemented"));
            }
        }
        
        Ok(())
    }

    pub fn export_connection_data(&self, graph: &ConnectionGraph) -> Result<()> {
        let export_data = self.prepare_connection_export(graph)?;
        
        match self.config.format {
            ExportFormat::Json => self.export_json(&export_data)?,
            ExportFormat::Csv => self.export_connection_csv(graph)?,
            ExportFormat::Html => self.export_connection_html(&export_data)?,
            ExportFormat::Png => graph.render(Path::new(&self.config.output_path))?,
            ExportFormat::Svg => {
                return Err(anyhow::anyhow!("SVG export not yet implemented"));
            }
        }
        
        Ok(())
    }

    fn prepare_bandwidth_export(&self, graph: &BandwidthGraph) -> Result<ExportData> {
        let summary = self.calculate_bandwidth_summary(graph);
        let time_series = graph.data.iter().map(|d| BandwidthTimePoint {
            timestamp: d.timestamp,
            download_speed: d.download_speed,
            upload_speed: d.upload_speed,
            total_rx: d.total_rx,
            total_tx: d.total_tx,
        }).collect();

        Ok(ExportData {
            timestamp: Utc::now(),
            export_type: "bandwidth".to_string(),
            interface: graph.interface.clone(),
            bandwidth_data: Some(BandwidthExportData {
                summary,
                time_series,
            }),
            protocol_data: None,
            connection_data: None,
        })
    }

    fn prepare_protocol_export(&self, graph: &ProtocolGraph) -> Result<ExportData> {
        let summaries = graph.get_protocol_summary();
        let total_packets: u64 = summaries.values().map(|s| s.total_packets).sum();
        
        let summary = summaries.into_iter().map(|(protocol, summary)| {
            ProtocolSummaryData {
                protocol: protocol.clone(),
                total_packets: summary.total_packets,
                total_bytes: summary.total_bytes,
                percentage_of_total: (summary.total_packets as f64 / total_packets as f64) * 100.0,
            }
        }).collect();

        let time_series = graph.data.iter().map(|d| ProtocolTimePoint {
            timestamp: d.timestamp,
            protocol: d.protocol.clone(),
            packet_count: d.packet_count,
            byte_count: d.byte_count,
        }).collect();

        Ok(ExportData {
            timestamp: Utc::now(),
            export_type: "protocol".to_string(),
            interface: graph.interface.clone(),
            bandwidth_data: None,
            protocol_data: Some(ProtocolExportData {
                summary,
                time_series,
            }),
            connection_data: None,
        })
    }

    fn prepare_connection_export(&self, graph: &ConnectionGraph) -> Result<ExportData> {
        let top_connections = graph.get_top_connections(20);
        let connection_details = top_connections.iter().map(|c| ConnectionDetail {
            source_ip: c.source_ip.clone(),
            dest_ip: c.dest_ip.clone(),
            source_port: c.source_port,
            dest_port: c.dest_port,
            protocol: c.protocol.clone(),
            total_packets: c.total_packets,
            total_bytes: c.total_bytes,
            duration_seconds: (c.last_seen - c.first_seen).num_seconds(),
        }).collect();

        let unique_ips = graph.data.iter()
            .map(|d| d.source_ip.clone())
            .collect::<std::collections::HashSet<_>>()
            .len() as u64;

        let unique_ports = graph.data.iter()
            .map(|d| d.dest_port)
            .collect::<std::collections::HashSet<_>>()
            .len() as u64;

        let summary = ConnectionSummaryData {
            total_connections: graph.data.len() as u64,
            unique_ips,
            unique_ports,
            most_active_protocol: "Tcp".to_string(), // Could be calculated
            total_traffic: graph.data.iter().map(|d| d.bytes_sent + d.bytes_received).sum(),
        };

        Ok(ExportData {
            timestamp: Utc::now(),
            export_type: "connection".to_string(),
            interface: graph.interface.clone(),
            bandwidth_data: None,
            protocol_data: None,
            connection_data: Some(ConnectionExportData {
                summary,
                top_connections: connection_details,
                time_series: vec![], // Could be implemented
            }),
        })
    }

    fn calculate_bandwidth_summary(&self, graph: &BandwidthGraph) -> BandwidthSummary {
        if graph.data.is_empty() {
            return BandwidthSummary {
                total_download: 0,
                total_upload: 0,
                avg_download_speed: 0.0,
                avg_upload_speed: 0.0,
                peak_download_speed: 0.0,
                peak_upload_speed: 0.0,
                duration_seconds: 0,
            };
        }

        let total_download = graph.data.last().map(|d| d.total_rx).unwrap_or(0);
        let total_upload = graph.data.last().map(|d| d.total_tx).unwrap_or(0);
        
        let avg_download_speed = graph.data.iter().map(|d| d.download_speed).sum::<f64>() / graph.data.len() as f64;
        let avg_upload_speed = graph.data.iter().map(|d| d.upload_speed).sum::<f64>() / graph.data.len() as f64;
        
        let peak_download_speed = graph.data.iter().map(|d| d.download_speed).fold(0.0, f64::max);
        let peak_upload_speed = graph.data.iter().map(|d| d.upload_speed).fold(0.0, f64::max);
        
        let duration_seconds = if let (Some(first), Some(last)) = (graph.data.first(), graph.data.last()) {
            (last.timestamp - first.timestamp).num_seconds()
        } else {
            0
        };

        BandwidthSummary {
            total_download,
            total_upload,
            avg_download_speed,
            avg_upload_speed,
            peak_download_speed,
            peak_upload_speed,
            duration_seconds,
        }
    }

    fn export_json<T: Serialize>(&self, data: &T) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        fs::write(&self.config.output_path, json)?;
        Ok(())
    }

    fn export_bandwidth_csv(&self, graph: &BandwidthGraph) -> Result<()> {
        let mut csv_content = "timestamp,download_speed,upload_speed,total_rx,total_tx\n".to_string();
        
        for data_point in &graph.data {
            csv_content.push_str(&format!(
                "{},{},{},{},{}\n",
                data_point.timestamp.to_rfc3339(),
                data_point.download_speed,
                data_point.upload_speed,
                data_point.total_rx,
                data_point.total_tx
            ));
        }
        
        fs::write(&self.config.output_path, csv_content)?;
        Ok(())
    }

    fn export_protocol_csv(&self, graph: &ProtocolGraph) -> Result<()> {
        let mut csv_content = "timestamp,protocol,packet_count,byte_count\n".to_string();
        
        for data_point in &graph.data {
            csv_content.push_str(&format!(
                "{},{},{},{}\n",
                data_point.timestamp.to_rfc3339(),
                data_point.protocol,
                data_point.packet_count,
                data_point.byte_count
            ));
        }
        
        fs::write(&self.config.output_path, csv_content)?;
        Ok(())
    }

    fn export_connection_csv(&self, graph: &ConnectionGraph) -> Result<()> {
        let mut csv_content = "timestamp,source_ip,dest_ip,source_port,dest_port,protocol,packets_sent,packets_received,bytes_sent,bytes_received\n".to_string();
        
        for data_point in &graph.data {
            csv_content.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                data_point.timestamp.to_rfc3339(),
                data_point.source_ip,
                data_point.dest_ip,
                data_point.source_port,
                data_point.dest_port,
                data_point.protocol,
                data_point.packets_sent,
                data_point.packets_received,
                data_point.bytes_sent,
                data_point.bytes_received
            ));
        }
        
        fs::write(&self.config.output_path, csv_content)?;
        Ok(())
    }

    fn export_bandwidth_html(&self, _data: &ExportData) -> Result<()> {
        // HTML export implementation would go here
        Err(anyhow::anyhow!("HTML export not yet implemented"))
    }

    fn export_protocol_html(&self, _data: &ExportData) -> Result<()> {
        // HTML export implementation would go here
        Err(anyhow::anyhow!("HTML export not yet implemented"))
    }

    fn export_connection_html(&self, _data: &ExportData) -> Result<()> {
        // HTML export implementation would go here
        Err(anyhow::anyhow!("HTML export not yet implemented"))
    }
}