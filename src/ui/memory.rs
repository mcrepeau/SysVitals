use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};
use crate::ui::chart_utils::{chart_areas, trim_to_width, usage_color};

pub fn draw_chart(frame: &mut Frame, area: Rect, memory: &crate::metrics::memory::MemoryMetrics) {
    let (title_area, chart_area) = chart_areas(area);

    let used_gb  = memory.used_bytes() as f64 / 1024.0f64.powi(3);
    let total_gb = memory.total_bytes as f64 / 1024.0f64.powi(3);
    let usage    = memory.used_percent();

    let title = if memory.total_swap > 0 {
        let swap_used_gb  = memory.swap_used_bytes() as f64 / 1024.0f64.powi(3);
        let swap_total_gb = memory.total_swap as f64 / 1024.0f64.powi(3);
        format!(
            "🗃️ Memory ({:.1} / {:.1} GB) | Swap ({:.1} / {:.1} GB)",
            used_gb, total_gb, swap_used_gb, swap_total_gb,
        )
    } else {
        format!("🗃️ Memory ({:.1} / {:.1} GB)", used_gb, total_gb)
    };

    frame.render_widget(
        Paragraph::new(ratatui::text::Span::styled(title, Style::default().fg(Color::White).bold())),
        title_area,
    );

    let width = chart_area.width as usize;
    let trimmed = trim_to_width(memory.used_percent_history(), width);

    let mut datasets = vec![
        Dataset::default()
            .name("RAM")
            .marker(Marker::Braille)
            .style(Style::default().fg(usage_color(usage)))
            .graph_type(GraphType::Line)
            .data(&trimmed),
    ];

    // Build swap dataset only when swap is configured on this machine.
    let swap_trimmed;
    if memory.total_swap > 0 {
        let swap_pct_history: std::collections::VecDeque<f64> = memory
            .swap_history()
            .iter()
            .map(|&b| (b as f64 / memory.total_swap as f64) * 100.0)
            .collect();
        swap_trimmed = trim_to_width(&swap_pct_history, width);
        datasets.push(
            Dataset::default()
                .name("Swap")
                .marker(Marker::Braille)
                .style(Style::default().fg(Color::Magenta))
                .graph_type(GraphType::Line)
                .data(&swap_trimmed),
        );
    }

    let chart = Chart::new(datasets)
        .block(Block::default().title("Usage (%)").borders(Borders::ALL))
        .x_axis(Axis::default()
            .bounds([0.0, trimmed.len().max(1) as f64])
            .style(Style::default().fg(Color::Gray)))
        .y_axis(Axis::default()
            .bounds([0.0, 100.0])
            .style(Style::default().fg(Color::Gray))
            .labels(["0%", "50%", "100%"]));

    frame.render_widget(chart, chart_area);
}
