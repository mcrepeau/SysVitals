use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Marker, Style, Stylize};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};

pub fn draw_chart(frame: &mut Frame, area: Rect, gpu: &crate::metrics::gpu::GpuMetrics) {
    // Vertical layout: title + chart
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(1), // Spacer line
            Constraint::Min(0),    // Chart area
            Constraint::Length(1), // Spacer line
        ])
        .split(area);

    // Title
    let gpu_name = gpu.name.clone().unwrap_or_else(|| "Unknown".to_string());
    let title = ratatui::text::Span::styled(format!("🖼️ Graphics - {}", gpu_name), Style::default().fg(Color::White).bold());
    frame.render_widget(Paragraph::new(title), chunks[0]);
    frame.render_widget(Paragraph::new(""), chunks[1]);

    // GPU Usage Gauge
    let usage = gpu.usage_percent() as u16;
    let label = format!("GPU Usage ({}%)", usage);

    // GPU Memory Usage Gauge
    let memory_usage = gpu.memory_percent() as u16;
    let memory_label = format!("Memory Usage ({}%)", memory_usage);

    // Chart data (trimmed to chart width)
    let chart_area = chunks[2];
    let width = chart_area.width as usize;
    let usage_history = gpu.usage_history();
    let memory_history = gpu.memory_history();

    let usage_trimmed: Vec<(f64, f64)> = usage_history
        .iter()
        .rev()
        .take(width)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();

    let usage_dataset = Dataset::default()
        .name("Usage")
        .marker(Marker::Braille)
        .style(Style::default().fg(Color::Red))
        .graph_type(GraphType::Line)
        .data(&usage_trimmed);

    // Chart with X and Y axes
    let usage_chart = Chart::new(vec![usage_dataset])
        .block(
            Block::default()
                .title(label)
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, usage_trimmed.len().max(1) as f64])
                .style(Style::default().fg(Color::Gray))
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, 100.0])
                .style(Style::default().fg(Color::Gray))
                .labels(vec!["0%".into(), "50%".into(), "100%".into()]),
        );

    let memory_trimmed: Vec<(f64, f64)> = memory_history
        .iter()
        .rev()
        .take(width)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();

    let memory_dataset = Dataset::default()
        .name("Memory")
        .marker(Marker::Braille)
        .style(Style::default().fg(Color::Magenta))
        .graph_type(GraphType::Line)
        .data(&memory_trimmed);

    // Chart with X and Y axes
    let memory_chart = Chart::new(vec![memory_dataset])
        .block(
            Block::default()
                .title(memory_label)
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, memory_trimmed.len().max(1) as f64])
                .style(Style::default().fg(Color::Gray))
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, 100.0])
                .style(Style::default().fg(Color::Gray))
                .labels(vec!["0%".into(), "50%".into(), "100%".into()]),
        );

    let chart_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),  // Usage Chart
            Constraint::Percentage(50),  // Memory Chart
        ])
        .split(chart_area);

    frame.render_widget(usage_chart, chart_chunks[0]);
    frame.render_widget(memory_chart, chart_chunks[1]);
    frame.render_widget(Paragraph::new(""), chunks[3]);
}