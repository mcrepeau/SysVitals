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
    system: System,
}

impl MemoryMetrics {
    /// Create a new memory metrics collector
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_memory();

        let used = system.used_memory();
        let total = system.total_memory();
        let percent = (used as f64 / total as f64) * 100.0;

        Self {
            used_percent: HistoricalMetric::new(percent),
            used_bytes: HistoricalMetric::new(used),
            total_bytes: total,
            system,
        }
    }

    /// Update memory metrics
    pub fn update(&mut self) -> Result<(), AppError> {
        self.system.refresh_memory();

        let used = self.system.used_memory();
        let total = self.system.total_memory();
        let percent = (used as f64 / total as f64) * 100.0;

        self.used_bytes.update(used);
        self.used_percent.update(percent);
        self.total_bytes = total;

        Ok(())
    }

    /// Get current used memory in bytes
    pub fn used_bytes(&self) -> u64 {
        *self.used_bytes.current()
    }

    /// Get current used memory percent
    pub fn used_percent(&self) -> f64 {
        *self.used_percent.current()
    }

    /// Get historical memory usage percent
    pub fn used_percent_history(&self) -> &VecDeque<f64> {
        self.used_percent.history()
    }

    /// Get historical memory usage in bytes
    pub fn used_bytes_history(&self) -> &VecDeque<u64> {
        self.used_bytes.history()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_metrics() {
        let mut memory = MemoryMetrics::new();
        assert!(memory.update().is_ok());

        let used_percent = memory.used_percent();
        let used_bytes = memory.used_bytes();

        assert!(used_percent >= 0.0 && used_percent <= 100.0);
        assert!(used_bytes <= memory.total_bytes);
        assert!(!memory.used_percent_history().is_empty());
        assert!(!memory.used_bytes_history().is_empty());
    }
}