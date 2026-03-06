//! Compact bars view — live values only, no history.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Style, Stylize};
use ratatui::widgets::{LineGauge, Paragraph};
use crate::metrics::SystemMetrics;
use crate::ui::chart_utils::{dynamic_bound, format_rate, usage_color};

struct Row {
    label: &'static str,
    ratio: f64,
    color: Color,
    value: String,
}

impl Row {
    fn new(label: &'static str, ratio: f64, color: Color, value: String) -> Self {
        Self { label, ratio: ratio.clamp(0.0, 1.0), color, value }
    }
}

pub fn draw_bars(
    frame: &mut Frame,
    area: Rect,
    system: &SystemMetrics,
    show_cpu: bool,
    show_memory: bool,
    show_gpu: bool,
    show_network: bool,
    show_disk: bool,
    selected_interface: usize,
) {
    let mut rows: Vec<Row> = vec![];

    if show_cpu {
        let pct = system.cpu().usage_percent();
        rows.push(Row::new("CPU", pct / 100.0, usage_color(pct), format!("{pct:.1}%")));
    }

    if show_memory {
        let mem = system.memory();
        let pct = mem.used_percent();
        let used_gb = mem.used_bytes() as f64 / 1024.0f64.powi(3);
        let total_gb = mem.total_bytes as f64 / 1024.0f64.powi(3);
        rows.push(Row::new(
            "RAM",
            pct / 100.0,
            usage_color(pct),
            format!("{used_gb:.1} / {total_gb:.1} GB"),
        ));
    }

    if show_gpu {
        if let Some(gpu) = system.gpu() {
            let pct = gpu.usage_percent();
            rows.push(Row::new("GPU", pct / 100.0, usage_color(pct), format!("{pct:.1}%")));
            let vram_pct = gpu.memory_percent();
            rows.push(Row::new("VRAM", vram_pct / 100.0, usage_color(vram_pct), format!("{vram_pct:.1}%")));
        }
    }

    if show_network {
        let net = system.network();
        let interfaces = net.interface_names();
        let selected = selected_interface.min(interfaces.len().saturating_sub(1));
        if let Some(iface) = interfaces.get(selected).cloned() {
            if let Some((rx_hist, tx_hist)) = net.get_interface_stats(&iface) {
                let rx = *rx_hist.current();
                let tx = *tx_hist.current();
                let rx_bound = dynamic_bound(rx_hist.history());
                let tx_bound = dynamic_bound(tx_hist.history());
                rows.push(Row::new("↓ NET", rx / rx_bound, Color::Green, format!("{} Mb/s", format_rate(rx))));
                rows.push(Row::new("↑ NET", tx / tx_bound, Color::Red,   format!("{} Mb/s", format_rate(tx))));
            }
        }
    }

    if show_disk {
        let disk = system.disk();
        let read  = disk.read_rate();
        let write = disk.write_rate();
        let read_bound  = dynamic_bound(disk.read_history());
        let write_bound = dynamic_bound(disk.write_history());
        rows.push(Row::new("↓ DSK", read  / read_bound,  Color::Cyan, format!("{} MB/s", format_rate(read))));
        rows.push(Row::new("↑ DSK", write / write_bound, Color::Blue, format!("{} MB/s", format_rate(write))));
    }

    if rows.is_empty() {
        return;
    }

    let row_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(1); rows.len()])
        .split(area);

    const LABEL_W: u16 = 8;
    const VALUE_W: u16 = 18;

    for (row, &row_area) in rows.into_iter().zip(row_areas.iter()) {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(LABEL_W),
                Constraint::Min(0),
                Constraint::Length(VALUE_W),
            ])
            .split(row_area);

        frame.render_widget(
            Paragraph::new(row.label).style(Style::default().fg(Color::White).bold()),
            cols[0],
        );
        frame.render_widget(
            LineGauge::default()
                .ratio(row.ratio)
                .filled_style(Style::default().fg(row.color)),
            cols[1],
        );
        frame.render_widget(
            Paragraph::new(row.value)
                .alignment(Alignment::Right)
                .style(Style::default().fg(row.color)),
            cols[2],
        );
    }
}
