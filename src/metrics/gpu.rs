//! GPU metrics collection

use nvml_wrapper::Nvml;
use crate::core::error::AppError;
use crate::metrics::historical_metric::HistoricalMetric;
use std::collections::VecDeque;

/// GPU metrics
pub struct GpuMetrics {
    usage_percent: HistoricalMetric<f64>,
    memory_percent: HistoricalMetric<f64>,
    pub name: Option<String>,
    nvml: Nvml,
}

impl GpuMetrics {
    /// Create a new GPU metrics collector
    pub fn new() -> Result<Self, AppError> {
        let nvml = Nvml::init().map_err(|e| AppError::System(e.to_string()))?;
        let device = nvml.device_by_index(0).map_err(|e| AppError::System(e.to_string()))?;
        let name = device.name().map_err(|e| AppError::System(e.to_string()))?;

        Ok(Self {
            usage_percent: HistoricalMetric::new(0.0),
            memory_percent: HistoricalMetric::new(0.0),
            name: Some(name),
            nvml,
        })
    }

    /// Update GPU metrics
    pub fn update(&mut self) -> Result<(), AppError> {
        let device = self.nvml.device_by_index(0).map_err(|e| AppError::System(e.to_string()))?;

        let usage = device
            .utilization_rates()
            .map_err(|e| AppError::System(e.to_string()))?
            .gpu as f64;

        let mem_info = device
            .memory_info()
            .map_err(|e| AppError::System(e.to_string()))?;

        let memory_percent = (mem_info.used as f64 / mem_info.total as f64) * 100.0;

        self.usage_percent.update(usage);
        self.memory_percent.update(memory_percent);

        Ok(())
    }

    /// Current GPU usage (%)
    pub fn usage_percent(&self) -> f64 {
        *self.usage_percent.current()
    }

    /// Current GPU memory usage (%)
    pub fn memory_percent(&self) -> f64 {
        *self.memory_percent.current()
    }

    /// History of GPU usage (%)
    pub fn usage_history(&self) -> &VecDeque<f64> {
        self.usage_percent.history()
    }

    /// History of GPU memory usage (%)
    pub fn memory_history(&self) -> &VecDeque<f64> {
        self.memory_percent.history()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_metrics() {
        if let Ok(mut gpu) = GpuMetrics::new() {
            assert!(gpu.update().is_ok());

            let usage = gpu.usage_percent();
            let mem = gpu.memory_percent();

            assert!(usage >= 0.0 && usage <= 100.0);
            assert!(mem >= 0.0 && mem <= 100.0);
            assert!(!gpu.usage_history().is_empty());
            assert!(!gpu.memory_history().is_empty());
        }
    }
}