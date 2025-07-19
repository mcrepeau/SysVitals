use crate::core::config::Config;
use crate::core::error::AppError;
use crate::metrics::{SystemMetrics, UnixSystemMetrics};
use crate::ui::{Ui, UiMode};
use crossterm::event::{Event, KeyCode};
use ratatui::Frame;
use std::time::{Duration, Instant};

pub struct App {
    config: Config,
    system: SystemMetrics,
    unix_metrics: Option<UnixSystemMetrics>,
    ui: Ui,
    last_update: Instant,
    update_interval: Duration,
    should_quit: bool,
    stats_refreshed: bool,
    last_key: Option<KeyCode>,
    last_key_time: Instant,
}

const DEBOUNCE_DELAY: Duration = Duration::from_millis(200);

impl App {
    /// Create a new application instance
    pub fn new() -> Result<Self, AppError> {
        Self::new_with_metrics(false)
    }

    /// Create a new application instance with optional Unix metrics
    pub fn new_with_metrics(use_unix_metrics: bool) -> Result<Self, AppError> {
        let config = Config::load().unwrap_or_default();
        let system = SystemMetrics::new();

        let mut ui = Ui::new();

        ui.show_cpu = config.show_cpu;
        ui.show_memory = config.show_memory;
        ui.show_gpu = config.show_gpu;
        ui.show_network = config.show_network;

        // Map refresh_rate ms to index in your update_interval_presets
        let presets = vec![
            Duration::from_millis(500),
            Duration::from_secs(1),
            Duration::from_secs(2),
            Duration::from_secs(5),
        ];
        let idx = presets
            .iter()
            .position(|d| d.as_millis() as u64 == config.refresh_rate)
            .unwrap_or(1);
        ui.selected_update_interval_idx = idx;

        // Find network interface index
        if let Some(ref iface) = config.selected_network_interface {
            let interfaces = system.network().interface_names();
            ui.selected_interface = interfaces.iter().position(|n| n == iface).unwrap_or(0);
        }

        let unix_metrics = if use_unix_metrics {
            Some(UnixSystemMetrics::new())
        } else {
            None
        };

        // Enable NPU and RGA if Unix metrics are available and they exist
        if let Some(ref unix_metrics) = unix_metrics {
            if unix_metrics.has_npu() {
                ui.show_npu = true;
            }
            if unix_metrics.has_rga() {
                ui.show_rga = true;
            }
        }

        Ok(Self {
            config,
            system,
            unix_metrics,
            ui,
            last_update: Instant::now(),
            update_interval: presets[idx],
            should_quit: false,
            stats_refreshed: false,
            last_key: None,
            last_key_time: Instant::now(),
        })
    }

    /// Handle input events
    pub fn handle_event(&mut self, event: Event) -> Result<(), AppError> {
        if let Event::Key(key_event) = event {
            let now = Instant::now();
            let key_code = key_event.code;

            // Debounce key repeat
            if Some(key_code) == self.last_key && now.duration_since(self.last_key_time) < DEBOUNCE_DELAY {
                return Ok(()); // ignore this event
            }

            self.last_key = Some(key_code);
            self.last_key_time = now;

            let mut config_changed = false;

            match self.ui.mode {
                UiMode::Normal => match key_code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        self.should_quit = true;
                    }
                    KeyCode::Char('o') | KeyCode::Char('O') => {
                        self.ui.mode = UiMode::OptionsMenu;
                    }
                    _ => {}
                },
                UiMode::OptionsMenu => match key_code {
                    KeyCode::Char('o') | KeyCode::Esc => {
                        self.ui.mode = UiMode::Normal;
                    }
                    KeyCode::Up => {
                        if self.ui.selected_option > 0 {
                            self.ui.selected_option -= 1;
                        }
                    }
                    KeyCode::Down => {
                        let max_option = if self.unix_metrics.is_some() { 6 } else { 4 };
                        if self.ui.selected_option < max_option {
                            self.ui.selected_option += 1;
                        }
                    }
                    KeyCode::Enter | KeyCode::Left | KeyCode::Right => {
                        if self.ui.selected_option == 0 {
                            // Cycle update interval presets
                            let presets_len = self.ui.update_interval_presets.len();
                            if key_code == KeyCode::Enter || key_code == KeyCode::Right {
                                self.ui.selected_update_interval_idx = (self.ui.selected_update_interval_idx + 1) % presets_len;
                            } else if key_code == KeyCode::Left {
                                if self.ui.selected_update_interval_idx == 0 {
                                    self.ui.selected_update_interval_idx = presets_len - 1;
                                } else {
                                    self.ui.selected_update_interval_idx -= 1;
                                }
                            }
                            config_changed = true;
                        } else {
                            // Toggle metrics options (offset by 1)
                            match self.ui.selected_option {
                                1 => self.ui.show_cpu = !self.ui.show_cpu,
                                2 => self.ui.show_memory = !self.ui.show_memory,
                                3 => self.ui.show_gpu = !self.ui.show_gpu,
                                4 => self.ui.show_network = !self.ui.show_network,
                                5 => {
                                    if self.unix_metrics.is_some() {
                                        self.ui.show_npu = !self.ui.show_npu;
                                    }
                                }
                                6 => {
                                    if self.unix_metrics.is_some() {
                                        self.ui.show_rga = !self.ui.show_rga;
                                    }
                                }
                                _ => {}
                            }
                            config_changed = true;
                        }
                    }
                    KeyCode::Tab => {
                        if self.ui.show_network {
                            let interface_count = self.system.network().interface_names().len();
                            if interface_count > 0 {
                                self.ui.selected_interface = (self.ui.selected_interface + 1) % interface_count;
                                config_changed = true;
                            }
                        }
                    }
                    _ => {}
                }
            }

            if config_changed {
                // Update config fields from UI state
                self.config.refresh_rate = self.ui.update_interval_presets[self.ui.selected_update_interval_idx].as_millis() as u64;
                self.config.show_cpu = self.ui.show_cpu;
                self.config.show_memory = self.ui.show_memory;
                self.config.show_gpu = self.ui.show_gpu;
                self.config.show_network = self.ui.show_network;
                // Note: NPU and RGA settings are not saved to config as they're Unix-specific

                let interfaces = self.system.network().interface_names();
                if !interfaces.is_empty() && self.ui.selected_interface < interfaces.len() {
                    self.config.selected_network_interface = Some(interfaces[self.ui.selected_interface].clone());
                } else {
                    self.config.selected_network_interface = None;
                }

                self.config.save().map_err(|e| AppError::Config(format!("Failed to save config: {e}")))?;
            }
        }
        Ok(())
    }


    /// Update application state
    pub fn update(&mut self) -> Result<(), AppError> {
        self.update_interval = self.ui.update_interval_presets[self.ui.selected_update_interval_idx];

        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.update_interval {
            self.system.update()?;
            
            // Update Unix metrics if available
            if let Some(unix_metrics) = &mut self.unix_metrics {
                unix_metrics.update()?;
            }
            
            self.last_update = now;
            self.stats_refreshed = true;
        }
        Ok(())
    }

    /// Render the UI
    pub fn draw(&mut self, frame: &mut Frame) {
        self.ui.draw(frame, &self.system, self.unix_metrics.as_ref(), self.stats_refreshed);
        self.stats_refreshed = false;
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Get available metrics types
    pub fn available_metrics(&self) -> Vec<&'static str> {
        if let Some(unix_metrics) = &self.unix_metrics {
            unix_metrics.available_metrics()
        } else {
            vec!["CPU", "Memory", "Network", "GPU"]
        }
    }

    /// Check if using Unix metrics
    pub fn is_using_unix_metrics(&self) -> bool {
        self.unix_metrics.is_some()
    }
}