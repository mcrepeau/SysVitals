use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};

pub fn draw_chart(frame: &mut Frame, area: Rect, memory: &crate::metrics::memory::MemoryMetrics) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let used_gb = memory.used_bytes() as f64 / 1024.0 / 1024.0 / 1024.0;
    let total_gb = memory.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
    let usage = memory.used_percent();

    let title = ratatui::text::Span::styled(
        format!("🗃️ Memory ({:.1} GB / {:.1} GB)", used_gb, total_gb),
        Style::default().fg(Color::White).bold(),
    );
    frame.render_widget(Paragraph::new(title), chunks[0]);
    frame.render_widget(Paragraph::new(""), chunks[1]);

    let chart_area = chunks[2];
    let width = chart_area.width as usize;
    let history = memory.used_percent_history();

    let trimmed: Vec<(f64, f64)> = history
        .iter().rev().take(width)
        .collect::<Vec<_>>().into_iter().rev()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();

    let dataset = Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(usage_color(usage)))
        .graph_type(GraphType::Line)
        .data(&trimmed);

    let chart = Chart::new(vec![dataset])
        .block(Block::default().title("Usage (%)").borders(Borders::ALL))
        .x_axis(Axis::default()
            .bounds([0.0, trimmed.len().max(1) as f64])
            .style(Style::default().fg(Color::Gray)))
        .y_axis(Axis::default()
            .bounds([0.0, 100.0])
            .style(Style::default().fg(Color::Gray))
            .labels(["0%", "50%", "100%"]));

    frame.render_widget(chart, chart_area);
    frame.render_widget(Paragraph::new(""), chunks[3]);
}

fn usage_color(percent: f64) -> Color {
    if percent >= 90.0 { Color::Red }
    else if percent >= 70.0 { Color::Yellow }
    else { Color::Green }
}
