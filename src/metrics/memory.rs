//! Memory metrics collection

use sysinfo::System;
use crate::core::error::AppError;
use crate::metrics::historical_metric::HistoricalMetric;
use std::collections::VecDeque;

/// Memory metrics
pub struct MemoryMetrics {
    used_percent: HistoricalMetric<f64>,
    used_bytes: HistoricalMetric<u64>,
    pub total_bytes: u64,
    swap_used: HistoricalMetric<u64>,
    pub total_swap: u64,
}

impl MemoryMetrics {
    /// Create a new memory metrics collector
    pub fn new(system: &System) -> Self {
        let used = system.used_memory();
        let total = system.total_memory();
        let percent = (used as f64 / total as f64) * 100.0;
        Self {
            used_percent: HistoricalMetric::new(percent),
            used_bytes: HistoricalMetric::new(used),
            total_bytes: total,
            swap_used: HistoricalMetric::new(system.used_swap()),
            total_swap: system.total_swap(),
        }
    }

    /// Update memory metrics
    pub fn update(&mut self, system: &mut System) -> Result<(), AppError> {
        system.refresh_memory();
        let used = system.used_memory();
        let total = system.total_memory();
        let percent = (used as f64 / total as f64) * 100.0;
        self.used_bytes.update(used);
        self.used_percent.update(percent);
        self.total_bytes = total;
        self.swap_used.update(system.used_swap());
        self.total_swap = system.total_swap();
        Ok(())
    }

    pub fn used_bytes(&self) -> u64 { *self.used_bytes.current() }
    pub fn used_percent(&self) -> f64 { *self.used_percent.current() }
    pub fn used_percent_history(&self) -> &VecDeque<f64> { self.used_percent.history() }

    pub fn swap_used_bytes(&self) -> u64 { *self.swap_used.current() }
    pub fn swap_used_percent(&self) -> f64 {
        if self.total_swap == 0 { return 0.0; }
        (*self.swap_used.current() as f64 / self.total_swap as f64) * 100.0
    }
    pub fn swap_history(&self) -> &VecDeque<u64> { self.swap_used.history() }

    pub fn resize_history(&mut self, len: usize) {
        self.used_percent.resize(len);
        self.used_bytes.resize(len);
        self.swap_used.resize(len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_metrics() {
        let mut system = System::new_all();
        system.refresh_memory();
        let mut memory = MemoryMetrics::new(&system);
        assert!(memory.update(&mut system).is_ok());

        let used_percent = memory.used_percent();
        let used_bytes = memory.used_bytes();

        assert!(used_percent >= 0.0 && used_percent <= 100.0);
        assert!(used_bytes <= memory.total_bytes);
        assert!(!memory.used_percent_history().is_empty());
    }
}
