use crate::metrics::SystemMetrics;
use crate::ui::{bars, cpu, disk, gpu, memory, network};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, BorderType};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::Frame;
use std::time::Duration;

pub enum UiMode {
    Normal,
    OptionsMenu,
}

pub struct Ui {
    pub mode: UiMode,
    pub show_cpu: bool,
    pub show_memory: bool,
    pub show_gpu: bool,
    pub show_network: bool,
    pub show_disk: bool,
    pub selected_option: usize,
    pub selected_interface: usize,
    pub update_interval_presets: Vec<Duration>,
    pub selected_update_interval_idx: usize,
    pub compact_view: bool,
}

impl Ui {
    /// Number of toggleable metric panels (one per show_* field).
    /// Increment this when adding a new panel.
    pub const METRIC_COUNT: usize = 5; // CPU, Memory, GPU, Network, Disk

    /// Total navigable items in the options menu: update interval + metrics.
    pub const MENU_OPTION_COUNT: usize = Self::METRIC_COUNT + 1;

    pub fn new() -> Self {
        Self {
            mode: UiMode::Normal,
            show_cpu: true,
            show_memory: true,
            show_gpu: true,
            show_network: true,
            show_disk: true,
            selected_option: 0,
            selected_interface: 0,
            update_interval_presets: vec![
                Duration::from_millis(500),
                Duration::from_secs(1),
                Duration::from_secs(2),
                Duration::from_secs(5),
            ],
            selected_update_interval_idx: 1,
            compact_view: false,
        }
    }

    /// Returns the ordered list of metric toggle options for the options menu.
    /// Each entry is `(label, current_enabled_state)`.
    /// Keep in sync with `METRIC_COUNT`.
    fn metric_options(&self) -> [(&'static str, bool); Self::METRIC_COUNT] {
        [
            ("CPU",     self.show_cpu),
            ("Memory",  self.show_memory),
            ("GPU",     self.show_gpu),
            ("Disk",    self.show_disk),
            ("Network", self.show_network),
        ]
    }

    pub fn draw(&mut self, frame: &mut Frame, system: &SystemMetrics, stats_refreshed: bool) {
        let area = frame.area();

        let instructions = match self.mode {
            UiMode::Normal => "<q>/<Esc>: Quit | <o>: Options | <v>: Toggle view".gray().bold(),
            UiMode::OptionsMenu => "<o>/<Esc>: Close Options | <↑↓>: Navigate | <Enter>: Toggle | <Tab>: Cycle Interface".gray().bold(),
        };

        let block = Block::bordered()
            .title(" System Monitor ".bold())
            .title_bottom(instructions)
            .border_set(ratatui::symbols::border::THICK)
            .border_type(BorderType::Rounded);

        frame.render_widget(block, area);

        match self.mode {
            UiMode::Normal => self.draw_main_ui(frame, area, system, stats_refreshed),
            UiMode::OptionsMenu => self.draw_options_menu(frame, area, system),
        }
    }

    fn draw_main_ui(&self, frame: &mut Frame, area: Rect, system: &SystemMetrics, stats_refreshed: bool) {
        let inner_area = Rect {
            x: area.x + 2,
            y: area.y + 2,
            width: area.width - 4,
            height: area.height - 4,
        };

        if self.compact_view {
            bars::draw_bars(
                frame, inner_area, system,
                self.show_cpu, self.show_memory, self.show_gpu,
                self.show_network, self.show_disk,
                self.selected_interface,
            );
            // Blink dot still shown in compact mode
            let blink_style = if stats_refreshed {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Black)
            };
            frame.render_widget(
                Paragraph::new("•").style(blink_style).block(Block::default().borders(Borders::NONE)),
                Rect { x: area.x + area.width - 3, y: area.y, width: 1, height: 1 },
            );
            return;
        }

        let mut enabled_metrics: Vec<Box<dyn FnOnce(&mut Frame, Rect)>> = vec![];

        if self.show_cpu {
            let cpu_data = system.cpu();
            enabled_metrics.push(Box::new(move |f, r| cpu::draw_chart(f, r, cpu_data)));
        }
        if self.show_memory {
            let memory_data = system.memory();
            enabled_metrics.push(Box::new(move |f, r| memory::draw_chart(f, r, memory_data)));
        }
        if self.show_disk {
            let disk_data = system.disk();
            enabled_metrics.push(Box::new(move |f, r| disk::draw_chart(f, r, disk_data)));
        }
        if self.show_network {
            let network_data = system.network();
            let interfaces = network_data.interface_names();
            let selected = self.selected_interface.min(interfaces.len().saturating_sub(1));
            let selected_iface = interfaces.get(selected).cloned();
            enabled_metrics.push(Box::new(move |f, r| network::draw_chart(f, r, network_data, selected_iface.as_deref())));
        }
        if self.show_gpu {
            if let Some(gpu_data) = system.gpu() {
                enabled_metrics.push(Box::new(move |f, r| gpu::draw_chart(f, r, gpu_data)));
            }
        }

        let constraints = vec![Constraint::Length(12); enabled_metrics.len()];
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner_area)
            .to_vec();

        for (render_fn, chunk) in enabled_metrics.into_iter().zip(chunks) {
            render_fn(frame, chunk);
        }

        // Blink dot: green on data refresh, invisible otherwise.
        let blink_style = if stats_refreshed {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Black)
        };
        frame.render_widget(
            Paragraph::new("•").style(blink_style).block(Block::default().borders(Borders::NONE)),
            Rect { x: area.x + area.width - 3, y: area.y, width: 1, height: 1 },
        );
    }

    fn draw_options_menu(&self, frame: &mut Frame, area: Rect, system: &SystemMetrics) {
        let interface_names = system.network().interface_names();
        let mut lines: Vec<Line> = vec![];

        let cursor = if self.selected_option == 0 { ">" } else { " " };
        let current_interval = self.update_interval_presets[self.selected_update_interval_idx];
        let interval_label = if current_interval.as_millis() < 1000 {
            format!("{} ms", current_interval.as_millis())
        } else {
            format!("{} s", current_interval.as_secs())
        };
        lines.push(Line::raw(format!(" {cursor} Update Interval: {interval_label}")));
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(" Metrics:", Style::default().bold())));
        lines.push(Line::raw(""));

        for (i, (label, enabled)) in self.metric_options().iter().enumerate() {
            let cursor = if self.selected_option == i + 1 { ">" } else { " " };
            let status = if *enabled { "[x]" } else { "[ ]" };
            lines.push(Line::raw(format!(" {cursor} {status} {label}")));
        }

        if self.show_network && !interface_names.is_empty() {
            for (i, name) in interface_names.iter().enumerate() {
                let cursor = if i == self.selected_interface { ">" } else { " " };
                lines.push(Line::raw(format!("     {cursor} {name}")));
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().title("Options").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, Rect {
            x: area.width / 4,
            y: area.height / 4,
            width: area.width / 2,
            height: area.height / 2,
        });
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}
