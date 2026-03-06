use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};
use crate::ui::chart_utils::{chart_areas, trim_to_width, usage_color};

pub fn draw_chart(frame: &mut Frame, area: Rect, cpu: &crate::metrics::cpu::CpuMetrics) {
    use crate::metrics::SystemMetrics;

    let (title_area, chart_area) = chart_areas(area);

    let usage = cpu.usage_percent();
    let cpu_name = cpu.name.as_deref().unwrap_or("Unknown");
    let load = SystemMetrics::load_average();

    let temp_str = cpu.temperature()
        .map(|t| format!("  {t:.0}°C"))
        .unwrap_or_default();
    let title = format!(
        "🧠 CPU - {} ({:.0}%){temp_str}  Load: {:.2} {:.2} {:.2}",
        cpu_name, usage, load.one, load.five, load.fifteen,
    );

    frame.render_widget(
        Paragraph::new(ratatui::text::Span::styled(title, Style::default().fg(Color::White).bold())),
        title_area,
    );

    let width = chart_area.width as usize;
    let trimmed = trim_to_width(cpu.usage_history(), width);

    let chart = Chart::new(vec![Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(usage_color(usage)))
        .graph_type(GraphType::Line)
        .data(&trimmed)])
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
