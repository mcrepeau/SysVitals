use crate::metrics::{SystemMetrics, UnixSystemMetrics};
use crate::ui::{cpu, memory, network, gpu, unix_cpu, unix_gpu, unix_npu, unix_rga};
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
    pub show_npu: bool,
    pub show_rga: bool,
    pub selected_option: usize, // for navigating the menu
    pub selected_interface: usize, // index of selected network interface
    pub update_interval_presets: Vec<Duration>,
    pub selected_update_interval_idx: usize,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            mode: UiMode::Normal,
            show_cpu: true,
            show_memory: true,
            show_gpu: true,
            show_network: true,
            show_npu: false,
            show_rga: false,
            selected_option: 0,
            selected_interface: 0,
            update_interval_presets: vec![
                Duration::from_millis(500),
                Duration::from_secs(1),
                Duration::from_secs(2),
                Duration::from_secs(5),
            ],
            selected_update_interval_idx: 1,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, system: &SystemMetrics, unix_metrics: Option<&UnixSystemMetrics>, stats_refreshed: bool) {
        let area = frame.size();

        let instructions = match self.mode {
            UiMode::Normal => "<q>/<Esc>: Quit | <o>: Options".bold(),
            UiMode::OptionsMenu => "<o>/<Esc>: Close Options | <↑↓>: Navigate | <Enter>: Toggle | <Tab>: Cycle Interface".yellow().bold(),
        };

        let block = Block::bordered()
            .title(" System Monitor ".bold())
            .title_bottom(instructions)
            .border_set(ratatui::symbols::border::THICK)
            .border_type(BorderType::Rounded);

        frame.render_widget(block, area);

        match self.mode {
            UiMode::Normal => self.draw_main_ui(frame, area, system, unix_metrics, stats_refreshed),
            UiMode::OptionsMenu => self.draw_options_menu(frame, area, system, unix_metrics),
        }
    }

    fn draw_main_ui(&self, frame: &mut Frame, area: Rect, system: &SystemMetrics, unix_metrics: Option<&UnixSystemMetrics>, stats_refreshed: bool) {
        let inner_area = Rect {
            x: area.x + 2,
            y: area.y + 2,
            width: area.width - 4,
            height: area.height - 4,
        };

        let mut enabled_metrics: Vec<(&str, Box<dyn FnOnce(&mut Frame, Rect)>)> = vec![];

        // Use Unix metrics if available, otherwise fall back to standard metrics
        if let Some(unix_metrics) = unix_metrics {
            // Unix CPU metrics
            if self.show_cpu {
                if let Some(cpu_data) = unix_metrics.cpu() {
                    enabled_metrics.push(("cpu", Box::new(move |f, r| unix_cpu::draw_chart(f, r, cpu_data))));
                } else {
                    // Fallback to standard CPU metrics
                    let cpu_data = system.cpu();
                    enabled_metrics.push(("cpu", Box::new(move |f, r| cpu::draw_chart(f, r, cpu_data))));
                }
            }

            // Unix GPU metrics
            if self.show_gpu {
                if let Some(gpu_data) = unix_metrics.gpu() {
                    enabled_metrics.push(("gpu", Box::new(move |f, r| unix_gpu::draw_chart(f, r, gpu_data))));
                } else if let Some(gpu_data) = system.gpu() {
                    // Fallback to standard GPU metrics
                    enabled_metrics.push(("gpu", Box::new(move |f, r| gpu::draw_chart(f, r, gpu_data))));
                }
            }

            // Unix NPU metrics
            if self.show_npu {
                if let Some(npu_data) = unix_metrics.npu() {
                    enabled_metrics.push(("npu", Box::new(move |f, r| unix_npu::draw_chart(f, r, npu_data))));
                }
            }

            // Unix RGA metrics
            if self.show_rga {
                if let Some(rga_data) = unix_metrics.rga() {
                    enabled_metrics.push(("rga", Box::new(move |f, r| unix_rga::draw_chart(f, r, rga_data))));
                }
            }
        } else {
            // Standard metrics only
            if self.show_cpu {
                let cpu_data = system.cpu();
                enabled_metrics.push(("cpu", Box::new(move |f, r| cpu::draw_chart(f, r, cpu_data))));
            }

            if self.show_gpu {
                if let Some(gpu_data) = system.gpu() {
                    enabled_metrics.push(("gpu", Box::new(move |f, r| gpu::draw_chart(f, r, gpu_data))));
                }
            }
        }

        // Memory and network are always from standard metrics
        if self.show_memory {
            let memory_data = system.memory();
            enabled_metrics.push(("memory", Box::new(move |f, r| memory::draw_chart(f, r, memory_data))));
        }

        if self.show_network {
            let network_data = system.network();
            let interfaces = network_data.interface_names();
            let selected = self.selected_interface.min(interfaces.len().saturating_sub(1));
            let selected_iface = interfaces.get(selected).cloned();
            enabled_metrics.push((
                "network",
                Box::new(move |f, r| network::draw_chart(f, r, network_data, selected_iface.as_deref())),
            ));
        }

        let constraints = vec![Constraint::Length(12); enabled_metrics.len()];
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner_area)
            .to_vec();

        for ((_, render_fn), chunk) in enabled_metrics.into_iter().zip(chunks) {
            render_fn(frame, chunk);
        }

        let blink_style = if stats_refreshed {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Black)
        };
        let blink_dot = Paragraph::new("•")
            .style(blink_style)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(blink_dot, Rect {
            x: area.x + area.width - 3,
            y: area.y,
            width: 1,
            height: 1,
        });
    }

    fn draw_options_menu(&self, frame: &mut Frame, area: Rect, system: &SystemMetrics, unix_metrics: Option<&UnixSystemMetrics>) {
        let interface_names = system.network().interface_names();

        let mut lines: Vec<String> = vec![];

        // Update Interval at the top
        let update_interval_idx = 0;
        let cursor = if self.selected_option == update_interval_idx { ">" } else { " " };
        let current_interval = self.update_interval_presets[self.selected_update_interval_idx];
        let interval_label = if current_interval.as_millis() < 1000 {
            format!("{} ms", current_interval.as_millis())
        } else {
            format!("{} s", current_interval.as_secs())
        };

        lines.push(format!(" {} Update Interval: {}", cursor, interval_label));
        lines.push(String::new());

        // Metrics header
        lines.push(" Metrics:".bold().to_string());
        lines.push(String::new());

        let mut options = vec![
            ("CPU", self.show_cpu),
            ("Memory", self.show_memory),
            ("GPU", self.show_gpu),
            ("Network", self.show_network),
        ];

        // Add Unix-specific options if Unix metrics are available
        if unix_metrics.is_some() {
            options.push(("NPU", self.show_npu));
            options.push(("RGA", self.show_rga));
        }

        // Metric toggles, index shifted by 1 because update interval is now at 0
        for (i, (label, enabled)) in options.iter().enumerate() {
            let cursor = if self.selected_option == i + 1 { ">" } else { " " };
            let status = if *enabled { "[x]" } else { "[ ]" };
            lines.push(format!(" {} {} {}", cursor, status, label));
        }

        // Interfaces
        if self.show_network && !interface_names.is_empty() {
            for (i, name) in interface_names.iter().enumerate() {
                let cursor = if i == self.selected_interface { ">" } else { " " };
                lines.push(format!("     {} {}", cursor, name));
            }
        }

        let paragraph = Paragraph::new(lines.into_iter().map(|l| l.into()).collect::<Vec<_>>())
            .block(Block::default().title("Options").borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow));

        let rect = Rect {
            x: area.width / 4,
            y: area.height / 4,
            width: area.width / 2,
            height: area.height / 2,
        };
        frame.render_widget(paragraph, rect);
    }

}
