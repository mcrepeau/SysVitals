use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};
use crate::ui::chart_utils::{chart_areas, split_horizontal, trim_to_width, usage_color};

pub fn draw_chart(frame: &mut Frame, area: Rect, gpu: &crate::metrics::gpu::GpuMetrics) {
    let (title_area, chart_area) = chart_areas(area);

    let gpu_name = gpu.name.clone().unwrap_or_else(|| "Unknown".to_string());
    frame.render_widget(
        Paragraph::new(ratatui::text::Span::styled(
            format!("🖼️ Graphics - {}", gpu_name),
            Style::default().fg(Color::White).bold(),
        )),
        title_area,
    );

    let usage = gpu.usage_percent();
    let memory_usage = gpu.memory_percent();
    let width = chart_area.width as usize;

    let usage_trimmed = trim_to_width(gpu.usage_history(), width);
    let memory_trimmed = trim_to_width(gpu.memory_history(), width);

    let usage_chart = Chart::new(vec![Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(usage_color(usage)))
        .graph_type(GraphType::Line)
        .data(&usage_trimmed)])
    .block(Block::default().title(format!("GPU Usage ({:.0}%)", usage)).borders(Borders::ALL))
    .x_axis(Axis::default()
        .bounds([0.0, usage_trimmed.len().max(1) as f64])
        .style(Style::default().fg(Color::Gray)))
    .y_axis(Axis::default()
        .bounds([0.0, 100.0])
        .style(Style::default().fg(Color::Gray))
        .labels(["0%", "50%", "100%"]));

    let memory_chart = Chart::new(vec![Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(usage_color(memory_usage)))
        .graph_type(GraphType::Line)
        .data(&memory_trimmed)])
    .block(Block::default().title(format!("Memory Usage ({:.0}%)", memory_usage)).borders(Borders::ALL))
    .x_axis(Axis::default()
        .bounds([0.0, memory_trimmed.len().max(1) as f64])
        .style(Style::default().fg(Color::Gray)))
    .y_axis(Axis::default()
        .bounds([0.0, 100.0])
        .style(Style::default().fg(Color::Gray))
        .labels(["0%", "50%", "100%"]));

    let (left, right) = split_horizontal(chart_area);
    frame.render_widget(usage_chart, left);
    frame.render_widget(memory_chart, right);
}
