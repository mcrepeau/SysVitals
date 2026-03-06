use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};

pub fn draw_chart(frame: &mut Frame, area: Rect, gpu: &crate::metrics::gpu::GpuMetrics) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let gpu_name = gpu.name.clone().unwrap_or_else(|| "Unknown".to_string());
    let title = ratatui::text::Span::styled(
        format!("🖼️ Graphics - {}", gpu_name),
        Style::default().fg(Color::White).bold(),
    );
    frame.render_widget(Paragraph::new(title), chunks[0]);
    frame.render_widget(Paragraph::new(""), chunks[1]);

    let usage = gpu.usage_percent();
    let memory_usage = gpu.memory_percent();
    let usage_label = format!("GPU Usage ({:.0}%)", usage);
    let memory_label = format!("Memory Usage ({:.0}%)", memory_usage);

    let chart_area = chunks[2];
    let width = chart_area.width as usize;

    let usage_trimmed: Vec<(f64, f64)> = gpu.usage_history()
        .iter().rev().take(width)
        .collect::<Vec<_>>().into_iter().rev()
        .enumerate().map(|(i, v)| (i as f64, *v)).collect();

    let memory_trimmed: Vec<(f64, f64)> = gpu.memory_history()
        .iter().rev().take(width)
        .collect::<Vec<_>>().into_iter().rev()
        .enumerate().map(|(i, v)| (i as f64, *v)).collect();

    let usage_dataset = Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(usage_color(usage)))
        .graph_type(GraphType::Line)
        .data(&usage_trimmed);

    let memory_dataset = Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(usage_color(memory_usage)))
        .graph_type(GraphType::Line)
        .data(&memory_trimmed);

    let usage_chart = Chart::new(vec![usage_dataset])
        .block(Block::default().title(usage_label).borders(Borders::ALL))
        .x_axis(Axis::default()
            .bounds([0.0, usage_trimmed.len().max(1) as f64])
            .style(Style::default().fg(Color::Gray)))
        .y_axis(Axis::default()
            .bounds([0.0, 100.0])
            .style(Style::default().fg(Color::Gray))
            .labels(["0%", "50%", "100%"]));

    let memory_chart = Chart::new(vec![memory_dataset])
        .block(Block::default().title(memory_label).borders(Borders::ALL))
        .x_axis(Axis::default()
            .bounds([0.0, memory_trimmed.len().max(1) as f64])
            .style(Style::default().fg(Color::Gray)))
        .y_axis(Axis::default()
            .bounds([0.0, 100.0])
            .style(Style::default().fg(Color::Gray))
            .labels(["0%", "50%", "100%"]));

    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chart_area);

    frame.render_widget(usage_chart, halves[0]);
    frame.render_widget(memory_chart, halves[1]);
    frame.render_widget(Paragraph::new(""), chunks[3]);
}

fn usage_color(percent: f64) -> Color {
    if percent >= 90.0 { Color::Red }
    else if percent >= 70.0 { Color::Yellow }
    else { Color::Green }
}
