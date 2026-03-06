use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};
use crate::metrics::disk::DiskMetrics;
use crate::ui::chart_utils::{chart_areas, dynamic_bound, rate_axis_labels, split_horizontal, trim_to_width};

pub fn draw_chart(frame: &mut Frame, area: Rect, disk: &DiskMetrics) {
    let (title_area, chart_area) = chart_areas(area);

    frame.render_widget(
        Paragraph::new(ratatui::text::Span::styled(
            "💾 Disk I/O",
            Style::default().fg(Color::White).bold(),
        )),
        title_area,
    );

    let width = chart_area.width as usize;
    let read_history = disk.read_history();
    let write_history = disk.write_history();

    let read_trimmed = trim_to_width(read_history, width);
    let write_trimmed = trim_to_width(write_history, width);

    let read_chart = Chart::new(vec![Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(Color::Green))
        .graph_type(GraphType::Line)
        .data(&read_trimmed)])
    .block(Block::default()
        .title(format!("Read ({:.2} MB/s)", disk.read_rate()))
        .borders(Borders::ALL))
    .x_axis(Axis::default()
        .bounds([0.0, read_trimmed.len().max(1) as f64])
        .style(Style::default().fg(Color::Gray)))
    .y_axis(Axis::default()
        .bounds([0.0, dynamic_bound(read_history)])
        .style(Style::default().fg(Color::Gray))
        .labels(rate_axis_labels(dynamic_bound(read_history))));

    let write_chart = Chart::new(vec![Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(Color::Red))
        .graph_type(GraphType::Line)
        .data(&write_trimmed)])
    .block(Block::default()
        .title(format!("Write ({:.2} MB/s)", disk.write_rate()))
        .borders(Borders::ALL))
    .x_axis(Axis::default()
        .bounds([0.0, write_trimmed.len().max(1) as f64])
        .style(Style::default().fg(Color::Gray)))
    .y_axis(Axis::default()
        .bounds([0.0, dynamic_bound(write_history)])
        .style(Style::default().fg(Color::Gray))
        .labels(rate_axis_labels(dynamic_bound(write_history))));

    let (left, right) = split_horizontal(chart_area);
    frame.render_widget(read_chart, left);
    frame.render_widget(write_chart, right);
}
