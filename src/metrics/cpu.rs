//! CPU metrics collection

use sysinfo::System;
use crate::core::error::AppError;
use crate::metrics::historical_metric::HistoricalMetric;
use std::collections::VecDeque;

/// CPU metrics
pub struct CpuMetrics {
    pub name: Option<String>,
    usage_percent: HistoricalMetric<f64>,
}

impl CpuMetrics {
    /// Create a new CPU metrics collector
    pub fn new(system: &System) -> Self {
        let system = system;
        let initial_usage = system.global_cpu_usage() as f64;
        Self {
            name: get_cpu_name(&system),
            usage_percent: HistoricalMetric::new(initial_usage),
        }
    }

    /// Update CPU metrics
    pub fn update(&mut self, system: &mut System) -> Result<(), AppError> {
        system.refresh_cpu_all();
        let new_usage = system.global_cpu_usage() as f64;
        self.usage_percent.update(new_usage);
        Ok(())
    }

    /// Get latest CPU usage (%)
    pub fn usage_percent(&self) -> f64 {
        *self.usage_percent.current()
    }

    /// Get historical CPU usage (%)
    pub fn usage_history(&self) -> &VecDeque<f64> {
        self.usage_percent.history()
    }
}

/// Get CPU name using sysinfo
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