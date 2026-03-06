use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};
use crate::metrics::network::NetworkMetrics;
use crate::ui::chart_utils::{chart_areas, dynamic_bound, rate_axis_labels, split_horizontal, trim_to_width};

pub fn draw_chart(frame: &mut Frame, area: Rect, network: &NetworkMetrics, selected: Option<&str>) {
    let (title_area, chart_area) = chart_areas(area);

    frame.render_widget(
        Paragraph::new(ratatui::text::Span::styled(
            format!("📡 Network – {}", selected.unwrap_or("")),
            Style::default().fg(Color::White).bold(),
        )),
        title_area,
    );

    // Early-return after rendering the title if there is no valid interface.
    let Some(iface) = selected else { return };
    let Some((rx_hist, tx_hist)) = network.get_interface_stats(iface) else { return };

    let width = chart_area.width as usize;
    let rx_history = rx_hist.history();
    let tx_history = tx_hist.history();

    let rx_trimmed = trim_to_width(rx_history, width);
    let tx_trimmed = trim_to_width(tx_history, width);

    let rx_chart = Chart::new(vec![Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(Color::Green))
        .graph_type(GraphType::Line)
        .data(&rx_trimmed)])
    .block(Block::default()
        .title(format!("↓ RX ({:.2} Mb/s)", rx_hist.current()))
        .borders(Borders::ALL))
    .x_axis(Axis::default()
        .bounds([0.0, rx_trimmed.len().max(1) as f64])
        .style(Style::default().fg(Color::Gray)))
    .y_axis(Axis::default()
        .bounds([0.0, dynamic_bound(rx_history)])
        .style(Style::default().fg(Color::Gray))
        .labels(rate_axis_labels(dynamic_bound(rx_history))));

    let tx_chart = Chart::new(vec![Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(Color::Red))
        .graph_type(GraphType::Line)
        .data(&tx_trimmed)])
    .block(Block::default()
        .title(format!("↑ TX ({:.2} Mb/s)", tx_hist.current()))
        .borders(Borders::ALL))
    .x_axis(Axis::default()
        .bounds([0.0, tx_trimmed.len().max(1) as f64])
        .style(Style::default().fg(Color::Gray)))
    .y_axis(Axis::default()
        .bounds([0.0, dynamic_bound(tx_history)])
        .style(Style::default().fg(Color::Gray))
        .labels(rate_axis_labels(dynamic_bound(tx_history))));

    let (left, right) = split_horizontal(chart_area);
    frame.render_widget(rx_chart, left);
    frame.render_widget(tx_chart, right);
}
