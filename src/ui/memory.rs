use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};
use crate::ui::chart_utils::{chart_areas, trim_to_width, usage_color};

pub fn draw_chart(frame: &mut Frame, area: Rect, memory: &crate::metrics::memory::MemoryMetrics) {
    let (title_area, chart_area) = chart_areas(area);

    let used_gb = memory.used_bytes() as f64 / 1024.0f64.powi(3);
    let total_gb = memory.total_bytes as f64 / 1024.0f64.powi(3);
    let usage = memory.used_percent();

    frame.render_widget(
        Paragraph::new(ratatui::text::Span::styled(
            format!("🗃️ Memory ({:.1} GB / {:.1} GB)", used_gb, total_gb),
            Style::default().fg(Color::White).bold(),
        )),
        title_area,
    );

    let width = chart_area.width as usize;
    let trimmed = trim_to_width(memory.used_percent_history(), width);

    let chart = Chart::new(vec![Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(usage_color(usage)))
        .graph_type(GraphType::Line)
        .data(&trimmed)])
    .block(Block::default().title("Usage (%)").borders(Borders::ALL))
    .x_axis(Axis::default()
        .bounds([0.0, trimmed.len().max(1) as f64])
        .style(Style::default().fg(Color::Gray)))
    .y_axis(Axis::default()
        .bounds([0.0, 100.0])
        .style(Style::default().fg(Color::Gray))
        .labels(["0%", "50%", "100%"]));

    frame.render_widget(chart, chart_area);
}
