use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};
use crate::metrics::disk::DiskMetrics;

pub fn draw_chart(frame: &mut Frame, area: Rect, disk: &DiskMetrics) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let title = ratatui::text::Span::styled(
        "💾 Disk I/O",
        Style::default().fg(Color::White).bold(),
    );
    frame.render_widget(Paragraph::new(title), chunks[0]);
    frame.render_widget(Paragraph::new(""), chunks[1]);

    let chart_area = chunks[2];
    let width = chart_area.width as usize;

    let read_history: Vec<f64> = disk.read_history().iter().copied().collect();
    let write_history: Vec<f64> = disk.write_history().iter().copied().collect();

    let read_trimmed = trim_to_width(&read_history, width);
    let write_trimmed = trim_to_width(&write_history, width);

    let read_bound = dynamic_bound(&read_history);
    let write_bound = dynamic_bound(&write_history);

    let read_label = format!("Read ({:.2} MB/s)", disk.read_rate());
    let write_label = format!("Write ({:.2} MB/s)", disk.write_rate());

    let read_dataset = Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(Color::Cyan))
        .graph_type(GraphType::Line)
        .data(&read_trimmed);

    let write_dataset = Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(Color::Blue))
        .graph_type(GraphType::Line)
        .data(&write_trimmed);

    let read_chart = Chart::new(vec![read_dataset])
        .block(Block::default().title(read_label).borders(Borders::ALL))
        .x_axis(Axis::default()
            .bounds([0.0, read_trimmed.len().max(1) as f64])
            .style(Style::default().fg(Color::Gray)))
        .y_axis(Axis::default()
            .bounds([0.0, read_bound])
            .style(Style::default().fg(Color::Gray))
            .labels(axis_labels(read_bound)));

    let write_chart = Chart::new(vec![write_dataset])
        .block(Block::default().title(write_label).borders(Borders::ALL))
        .x_axis(Axis::default()
            .bounds([0.0, write_trimmed.len().max(1) as f64])
            .style(Style::default().fg(Color::Gray)))
        .y_axis(Axis::default()
            .bounds([0.0, write_bound])
            .style(Style::default().fg(Color::Gray))
            .labels(axis_labels(write_bound)));

    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chart_area);

    frame.render_widget(read_chart, halves[0]);
    frame.render_widget(write_chart, halves[1]);
    frame.render_widget(Paragraph::new(""), chunks[3]);
}

fn trim_to_width(history: &[f64], width: usize) -> Vec<(f64, f64)> {
    history.iter().rev().take(width)
        .collect::<Vec<_>>().into_iter().rev()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect()
}

/// Scale the Y-axis to 120% of the max visible value, with a floor of 1.0 MB/s.
fn dynamic_bound(history: &[f64]) -> f64 {
    let max = history.iter().cloned().fold(0.0f64, f64::max);
    (max * 1.2).max(1.0)
}

fn format_mb(val: f64) -> String {
    if val >= 100.0 { format!("{:.0}", val) }
    else if val >= 10.0 { format!("{:.1}", val) }
    else { format!("{:.2}", val) }
}

fn axis_labels(bound: f64) -> [String; 3] {
    [format_mb(0.0), format_mb(bound / 2.0), format_mb(bound)]
}
