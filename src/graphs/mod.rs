pub mod bandwidth_graphs;
pub mod protocol_graphs;
pub mod connection_graphs;
pub mod export;

use anyhow::Result;
use chrono::{DateTime, Utc};
use plotters::prelude::*;
use std::path::Path;

pub trait GraphRenderer {
    fn render(&self, output_path: &Path) -> Result<()>;
}

pub struct GraphConfig {
    pub width: u32,
    pub height: u32,
    #[allow(dead_code)]
    pub title: String,
    #[allow(dead_code)]
    pub x_label: String,
    #[allow(dead_code)]
    pub y_label: String,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            title: "Network Monitor".to_string(),
            x_label: "Time".to_string(),
            y_label: "Value".to_string(),
        }
    }
}

#[allow(dead_code)]
pub struct TimeSeriesData {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub label: String,
}

#[allow(dead_code)]
pub fn create_time_series_chart(
    data: Vec<TimeSeriesData>,
    config: GraphConfig,
    output_path: &Path,
) -> Result<()> {
    let root = BitMapBackend::new(output_path, (config.width, config.height)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(&config.title, ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(
            data.first().map(|d| d.timestamp).unwrap_or_else(Utc::now)
                ..data.last().map(|d| d.timestamp).unwrap_or_else(Utc::now),
            0f64..data.iter().map(|d| d.value).fold(0.0, f64::max),
        )?;

    chart
        .configure_mesh()
        .x_desc(&config.x_label)
        .y_desc(&config.y_label)
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            data.iter().map(|d| (d.timestamp, d.value)),
            &RED,
        ))?
        .label("Value")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], RED));

    chart.configure_series_labels().draw()?;
    root.present()?;

    Ok(())
}