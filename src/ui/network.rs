use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};
use crate::metrics::network::NetworkMetrics;

pub fn draw_chart(frame: &mut Frame, area: Rect, network: &NetworkMetrics, selected: Option<&str>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let (rx_mbps, tx_mbps, rx_history, tx_history) = if let Some(iface) = selected {
        if let Some((rx_hist, tx_hist)) = network.get_interface_stats(iface) {
            let rx_history = rx_hist.history().iter().copied().collect::<Vec<_>>();
            let tx_history = tx_hist.history().iter().copied().collect::<Vec<_>>();
            (*rx_hist.current(), *tx_hist.current(), rx_history, tx_history)
        } else {
            (0.0, 0.0, vec![], vec![])
        }
    } else {
        (0.0, 0.0, vec![], vec![])
    };

    let label = selected.unwrap_or("");
    let title = ratatui::text::Span::styled(
        format!("📡 Network – {label}"),
        Style::default().fg(Color::White).bold(),
    );
    frame.render_widget(Paragraph::new(title), chunks[0]);
    frame.render_widget(Paragraph::new(""), chunks[1]);

    let chart_area = chunks[2];
    let width = chart_area.width as usize;

    let rx_trimmed = trim_to_width(&rx_history, width);
    let tx_trimmed = trim_to_width(&tx_history, width);

    let rx_bound = dynamic_bound(&rx_history);
    let tx_bound = dynamic_bound(&tx_history);

    let rx_label = format!("↓ RX ({:.2} Mb/s)", rx_mbps);
    let tx_label = format!("↑ TX ({:.2} Mb/s)", tx_mbps);

    let rx_dataset = Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(Color::Green))
        .graph_type(GraphType::Line)
        .data(&rx_trimmed);

    let tx_dataset = Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(Color::Red))
        .graph_type(GraphType::Line)
        .data(&tx_trimmed);

    let rx_chart = Chart::new(vec![rx_dataset])
        .block(Block::default().title(rx_label).borders(Borders::ALL))
        .x_axis(Axis::default()
            .bounds([0.0, rx_trimmed.len().max(1) as f64])
            .style(Style::default().fg(Color::Gray)))
        .y_axis(Axis::default()
            .bounds([0.0, rx_bound])
            .style(Style::default().fg(Color::Gray))
            .labels(axis_labels(rx_bound)));

    let tx_chart = Chart::new(vec![tx_dataset])
        .block(Block::default().title(tx_label).borders(Borders::ALL))
        .x_axis(Axis::default()
            .bounds([0.0, tx_trimmed.len().max(1) as f64])
            .style(Style::default().fg(Color::Gray)))
        .y_axis(Axis::default()
            .bounds([0.0, tx_bound])
            .style(Style::default().fg(Color::Gray))
            .labels(axis_labels(tx_bound)));

    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chart_area);

    frame.render_widget(rx_chart, halves[0]);
    frame.render_widget(tx_chart, halves[1]);
    frame.render_widget(Paragraph::new(""), chunks[3]);
}

fn trim_to_width(history: &[f64], width: usize) -> Vec<(f64, f64)> {
    history.iter().rev().take(width)
        .collect::<Vec<_>>().into_iter().rev()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect()
}

/// Scale Y-axis to 120% of the max visible value, with a floor of 1.0 Mb/s.
fn dynamic_bound(history: &[f64]) -> f64 {
    let max = history.iter().cloned().fold(0.0f64, f64::max);
    (max * 1.2).max(1.0)
}

fn format_mbps(val: f64) -> String {
    if val >= 100.0 { format!("{:.0}", val) }
    else if val >= 10.0 { format!("{:.1}", val) }
    else { format!("{:.2}", val) }
}

fn axis_labels(bound: f64) -> [String; 3] {
    [format_mbps(0.0), format_mbps(bound / 2.0), format_mbps(bound)]
}
