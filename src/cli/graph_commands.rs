use crate::cli::commands::GraphType;
use crate::graphs::bandwidth_graphs::BandwidthGraph;
use crate::graphs::protocol_graphs::ProtocolGraph;
use crate::graphs::connection_graphs::ConnectionGraph;
use crate::graphs::export::{ExportConfig, ExportFormat, ExportManager};
use crate::graphs::GraphConfig;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;

/// Simple database wrapper for graph operations
pub struct DatabaseManager {
    pub connection: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
}

impl DatabaseManager {
    pub async fn new(path: &str) -> Result<Self> {
        let conn = rusqlite::Connection::open(path)?;
        Ok(Self {
            connection: Arc::new(std::sync::Mutex::new(conn)),
        })
    }
}

pub struct GraphCommandHandler {
    db: Arc<DatabaseManager>,
}

impl GraphCommandHandler {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    pub async fn handle_graph_command(&self, graph_type: GraphType) -> Result<()> {
        match graph_type {
            GraphType::Bandwidth { period, interface, output, format, graph_type } => {
                self.handle_bandwidth_graph(period, interface, output, format, graph_type).await
            }
            GraphType::Protocols { period, interface, output, format, chart_type } => {
                self.handle_protocol_graph(period, interface, output, format, chart_type).await
            }
            GraphType::Connections { period, interface, output, format, chart_type } => {
                self.handle_connection_graph(period, interface, output, format, chart_type).await
            }
        }
    }

    async fn handle_bandwidth_graph(
        &self,
        period: String,
        interface: Option<String>,
        output: Option<String>,
        format: String,
        graph_type: String,
    ) -> Result<()> {
        let (start_time, end_time) = self.parse_period(&period)?;
        
        let output_path = output.unwrap_or_else(|| {
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
            match interface.as_ref() {
                Some(iface) => format!("bandwidth_{iface}_{timestamp}.{format}"),
                None => format!("bandwidth_all_{timestamp}.{format}"),
            }
        });

        let config = GraphConfig {
            width: 1200,
            height: 800,
            title: match interface.as_ref() {
                Some(iface) => format!("Bandwidth Usage - {iface}"),
                None => "Total Bandwidth Usage".to_string(),
            },
            x_label: "Time".to_string(),
            y_label: "Speed (bytes/s)".to_string(),
        };

        let mut graph = BandwidthGraph::new(config);
        graph.load_data(&self.db, start_time, end_time, interface.clone()).await?;

        if graph.data.is_empty() {
            println!("No bandwidth data found for the specified period.");
            return Ok(());
        }

        let export_format = self.parse_export_format(&format)?;
        let export_config = ExportConfig {
            format: export_format,
            output_path: output_path.clone(),
            include_raw_data: true,
            compress: false,
        };

        let export_manager = ExportManager::new(export_config);

        match graph_type.as_str() {
            "speed" => {
                if format == "png" {
                    graph.render_speed_chart(std::path::Path::new(&output_path))?;
                } else {
                    export_manager.export_bandwidth_data(&graph)?;
                }
            }
            "total" => {
                if format == "png" {
                    graph.render_total_usage_chart(std::path::Path::new(&output_path))?;
                } else {
                    export_manager.export_bandwidth_data(&graph)?;
                }
            }
            "both" => {
                if format == "png" {
                    let speed_path = output_path.replace(".png", "_speed.png");
                    let total_path = output_path.replace(".png", "_total.png");
                    graph.render_speed_chart(std::path::Path::new(&speed_path))?;
                    graph.render_total_usage_chart(std::path::Path::new(&total_path))?;
                    println!("Generated speed chart: {speed_path}");
                    println!("Generated total usage chart: {total_path}");
                } else {
                    export_manager.export_bandwidth_data(&graph)?;
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid graph type: {}", graph_type));
            }
        }

        if format == "png" && graph_type != "both" {
            println!("Bandwidth graph saved to: {output_path}");
        } else if format != "png" {
            println!("Bandwidth data exported to: {output_path}");
        }

        Ok(())
    }

    async fn handle_protocol_graph(
        &self,
        period: String,
        interface: Option<String>,
        output: Option<String>,
        format: String,
        chart_type: String,
    ) -> Result<()> {
        let (start_time, end_time) = self.parse_period(&period)?;
        
        let output_path = output.unwrap_or_else(|| {
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
            match interface.as_ref() {
                Some(iface) => format!("protocols_{iface}_{timestamp}.{format}"),
                None => format!("protocols_all_{timestamp}.{format}"),
            }
        });

        let config = GraphConfig {
            width: 1200,
            height: 800,
            title: match interface.as_ref() {
                Some(iface) => format!("Protocol Distribution - {iface}"),
                None => "Protocol Distribution".to_string(),
            },
            x_label: "Protocol".to_string(),
            y_label: "Packet Count".to_string(),
        };

        let mut graph = ProtocolGraph::new(config);
        graph.load_data(&self.db, start_time, end_time, interface.clone()).await?;

        if graph.data.is_empty() {
            println!("No protocol data found for the specified period.");
            return Ok(());
        }

        let export_format = self.parse_export_format(&format)?;
        let export_config = ExportConfig {
            format: export_format,
            output_path: output_path.clone(),
            include_raw_data: true,
            compress: false,
        };

        let export_manager = ExportManager::new(export_config);

        match chart_type.as_str() {
            "bar" => {
                if format == "png" {
                    graph.render_bar_chart(std::path::Path::new(&output_path))?;
                } else {
                    export_manager.export_protocol_data(&graph)?;
                }
            }
            "pie" => {
                if format == "png" {
                    graph.render_pie_chart(std::path::Path::new(&output_path))?;
                } else {
                    export_manager.export_protocol_data(&graph)?;
                }
            }
            "timeline" => {
                if format == "png" {
                    graph.render_timeline_chart(std::path::Path::new(&output_path))?;
                } else {
                    export_manager.export_protocol_data(&graph)?;
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid chart type: {}", chart_type));
            }
        }

        if format == "png" {
            println!("Protocol {chart_type} chart saved to: {output_path}");
        } else {
            println!("Protocol data exported to: {output_path}");
        }

        Ok(())
    }

    async fn handle_connection_graph(
        &self,
        period: String,
        interface: Option<String>,
        output: Option<String>,
        format: String,
        chart_type: String,
    ) -> Result<()> {
        let (start_time, end_time) = self.parse_period(&period)?;
        
        let output_path = output.unwrap_or_else(|| {
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
            match interface.as_ref() {
                Some(iface) => format!("connections_{iface}_{timestamp}.{format}"),
                None => format!("connections_all_{timestamp}.{format}"),
            }
        });

        let config = GraphConfig {
            width: 1200,
            height: 800,
            title: match interface.as_ref() {
                Some(iface) => format!("Connection Patterns - {iface}"),
                None => "Connection Patterns".to_string(),
            },
            x_label: "Time".to_string(),
            y_label: "Connections".to_string(),
        };

        let mut graph = ConnectionGraph::new(config);
        graph.load_data(&self.db, start_time, end_time, interface.clone()).await?;

        if graph.data.is_empty() {
            println!("No connection data found for the specified period.");
            return Ok(());
        }

        let export_format = self.parse_export_format(&format)?;
        let export_config = ExportConfig {
            format: export_format,
            output_path: output_path.clone(),
            include_raw_data: true,
            compress: false,
        };

        let export_manager = ExportManager::new(export_config);

        match chart_type.as_str() {
            "timeline" => {
                if format == "png" {
                    graph.render_connection_timeline(std::path::Path::new(&output_path))?;
                } else {
                    export_manager.export_connection_data(&graph)?;
                }
            }
            "ports" => {
                if format == "png" {
                    graph.render_port_distribution(std::path::Path::new(&output_path))?;
                } else {
                    export_manager.export_connection_data(&graph)?;
                }
            }
            "traffic" => {
                if format == "png" {
                    graph.render_traffic_flow(std::path::Path::new(&output_path))?;
                } else {
                    export_manager.export_connection_data(&graph)?;
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid chart type: {}", chart_type));
            }
        }

        if format == "png" {
            println!("Connection {chart_type} chart saved to: {output_path}");
        } else {
            println!("Connection data exported to: {output_path}");
        }

        Ok(())
    }

    fn parse_period(&self, period: &str) -> Result<(DateTime<Utc>, DateTime<Utc>)> {
        let now = Utc::now();
        let duration = self.parse_duration(period)?;
        let start_time = now - duration;
        Ok((start_time, now))
    }

    fn parse_duration(&self, period: &str) -> Result<Duration> {
        let period = period.to_lowercase();
        
        if period.ends_with('s') {
            let seconds: i64 = period[..period.len()-1].parse()?;
            Ok(Duration::seconds(seconds))
        } else if period.ends_with('m') {
            let minutes: i64 = period[..period.len()-1].parse()?;
            Ok(Duration::minutes(minutes))
        } else if period.ends_with('h') {
            let hours: i64 = period[..period.len()-1].parse()?;
            Ok(Duration::hours(hours))
        } else if period.ends_with('d') {
            let days: i64 = period[..period.len()-1].parse()?;
            Ok(Duration::days(days))
        } else {
            // Default to hours if no unit specified
            let hours: i64 = period.parse()?;
            Ok(Duration::hours(hours))
        }
    }

    fn parse_export_format(&self, format: &str) -> Result<ExportFormat> {
        match format.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            "html" => Ok(ExportFormat::Html),
            "png" => Ok(ExportFormat::Png),
            "svg" => Ok(ExportFormat::Svg),
            _ => Err(anyhow::anyhow!("Unsupported export format: {}", format)),
        }
    }
}