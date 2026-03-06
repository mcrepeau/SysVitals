//! CPU metrics collection

use sysinfo::{Components, System};
use crate::core::error::AppError;
use crate::metrics::historical_metric::HistoricalMetric;
use std::collections::VecDeque;

/// CPU metrics
pub struct CpuMetrics {
    pub name: Option<String>,
    usage_percent: HistoricalMetric<f64>,
    components: Components,
    temperature: Option<f64>,
}

impl CpuMetrics {
    /// Create a new CPU metrics collector
    pub fn new(system: &System) -> Self {
        let initial_usage = system.global_cpu_usage() as f64;
        let components = Components::new_with_refreshed_list();
        let temperature = find_cpu_temp(&components);
        Self {
            name: get_cpu_name(system),
            usage_percent: HistoricalMetric::new(initial_usage),
            components,
            temperature,
        }
    }

    /// Update CPU metrics
    pub fn update(&mut self, system: &mut System) -> Result<(), AppError> {
        system.refresh_cpu_all();
        self.usage_percent.update(system.global_cpu_usage() as f64);
        self.components.refresh(false);
        self.temperature = find_cpu_temp(&self.components);
        Ok(())
    }

    /// Current CPU usage (%)
    pub fn usage_percent(&self) -> f64 { *self.usage_percent.current() }

    /// Historical CPU usage (%)
    pub fn usage_history(&self) -> &VecDeque<f64> { self.usage_percent.history() }

    /// Current CPU temperature in °C, if available
    pub fn temperature(&self) -> Option<f64> { self.temperature }

    pub fn resize_history(&mut self, len: usize) {
        self.usage_percent.resize(len);
    }
}

/// Return the best-guess CPU temperature from the component list.
/// Tries common label patterns in priority order.
fn find_cpu_temp(components: &Components) -> Option<f64> {
    let priority = ["package id 0", "tctl", "cpu temperature", "cpu"];
    for term in priority {
        if let Some(c) = components.iter().find(|c| c.label().to_lowercase().contains(term)) {
            return c.temperature().map(|t| t as f64);
        }
    }
    // Fallback: first component with a valid reading
    components.iter().find_map(|c| c.temperature().map(|t| t as f64))
}

fn get_cpu_name(system: &System) -> Option<String> {
    system.cpus().first().map(|cpu| cpu.brand().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_metrics() {
        let mut system = System::new_all();
        system.refresh_cpu_all();
        let mut cpu = CpuMetrics::new(&system);
        assert!(cpu.update(&mut system).is_ok());
        let usage = cpu.usage_percent();
        assert!(usage >= 0.0 && usage <= 100.0);
        assert!(!cpu.usage_history().is_empty());
    }
}
