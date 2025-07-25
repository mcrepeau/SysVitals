use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Marker, Style, Stylize};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};

pub fn draw_chart(frame: &mut Frame, area: Rect, cpu: &crate::metrics::cpu::CpuMetrics) {
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
    let cpu_name = cpu.name.clone().unwrap_or_else(|| "Unknown".to_string());
    let usage = cpu.usage_percent();
    let title = ratatui::text::Span::styled(
        format!("🧠 CPU - {} ({:.0}%)", cpu_name, usage),
        Style::default().fg(Color::White).bold(),
    );
    frame.render_widget(Paragraph::new(title), chunks[0]);

    frame.render_widget(Paragraph::new(""), chunks[1]);

    // Chart data (trimmed to chart width)
    let chart_area = chunks[2];
    let width = chart_area.width as usize;
    let history = cpu.usage_history();

    let trimmed: Vec<(f64, f64)> = history
        .iter()
        .rev()
        .take(width)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();

    let dataset = Dataset::default()
        .name("CPU Usage")
        .marker(Marker::Braille)
        .style(Style::default().fg(Color::Yellow))
        .graph_type(GraphType::Line)
        .data(&trimmed);

    // Chart with X and Y axes
    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .title("Usage (%)")
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, trimmed.len().max(1) as f64])
                .style(Style::default().fg(Color::Gray))
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, 100.0])
                .style(Style::default().fg(Color::Gray))
                .labels(vec!["0%".into(), "50%".into(), "100%".into()]),
        );

    frame.render_widget(chart, chart_area);
    frame.render_widget(Paragraph::new(""), chunks[3]);
}