use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Marker, Style, Stylize};
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
            let rx_mbps = *rx_hist.current();
            let tx_mbps = *tx_hist.current();
            let rx_history = rx_hist.history().iter().copied().collect::<Vec<_>>();
            let tx_history = tx_hist.history().iter().copied().collect::<Vec<_>>();
            (rx_mbps, tx_mbps, rx_history, tx_history)
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

    let rx_trimmed: Vec<(f64, f64)> = rx_history
        .iter()
        .rev()
        .take(width)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();

    let tx_trimmed: Vec<(f64, f64)> = tx_history
        .iter()
        .rev()
        .take(width)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();

    let rx_label = format!("↓ RX ({:.1} Mb/s)", rx_mbps);
    let tx_label = format!("↑ TX ({:.1} Mb/s)", tx_mbps);

    let rx_dataset = Dataset::default()
        .name("RX")
        .marker(Marker::Braille)
        .style(Style::default().fg(Color::Green))
        .graph_type(GraphType::Line)
        .data(&rx_trimmed);

    let tx_dataset = Dataset::default()
        .name("TX")
        .marker(Marker::Braille)
        .style(Style::default().fg(Color::Green))
        .graph_type(GraphType::Line)
        .data(&tx_trimmed);

    let rx_chart = Chart::new(vec![rx_dataset])
        .block(
            Block::default()
                .title(rx_label)
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, rx_trimmed.len().max(1) as f64])
                .style(Style::default().fg(Color::Gray))
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, 1000.0])
                .style(Style::default().fg(Color::Gray))
                .labels(vec!["0".into(), "500".into(), "1000".into()]),
        );

    let tx_chart = Chart::new(vec![tx_dataset])
        .block(
            Block::default()
                .title(tx_label)
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, tx_trimmed.len().max(1) as f64])
                .style(Style::default().fg(Color::Gray))
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, 1000.0])
                .style(Style::default().fg(Color::Gray))
                .labels(vec!["0".into(), "500".into(), "1000".into()]),
        );

    let chart_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chart_area);

    frame.render_widget(rx_chart, chart_chunks[0]);
    frame.render_widget(tx_chart, chart_chunks[1]);
    frame.render_widget(Paragraph::new(""), chunks[3]);
}
