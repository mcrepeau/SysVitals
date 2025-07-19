//! Unix-based GPU metrics collection using /sys/class/devfreq/

use crate::core::error::AppError;
use crate::metrics::historical_metric::HistoricalMetric;
use std::collections::VecDeque;
use std::fs;
use std::path::Path;

/// Unix-based GPU metrics
pub struct UnixGpuMetrics {
    usage_percent: HistoricalMetric<f64>,
    frequency_mhz: HistoricalMetric<u64>,
    gpu_path: String,
}

impl UnixGpuMetrics {
    /// Create a new Unix-based GPU metrics collector
    pub fn new() -> Result<Self, AppError> {
        let gpu_path = Self::find_gpu_devfreq_path()?;
        
        Ok(Self {
            usage_percent: HistoricalMetric::new(0.0),
            frequency_mhz: HistoricalMetric::new(0),
            gpu_path,
        })
    }

    /// Create a new Unix-based GPU metrics collector with custom GPU path
    pub fn with_path(gpu_path: String) -> Self {
        Self {
            usage_percent: HistoricalMetric::new(0.0),
            frequency_mhz: HistoricalMetric::new(0),
            gpu_path,
        }
    }

    /// Update GPU metrics
    pub fn update(&mut self) -> Result<(), AppError> {
        // Update GPU load
        self.update_gpu_load()?;
        
        // Update GPU frequency
        self.update_gpu_frequency()?;
        
        Ok(())
    }

    /// Get latest GPU usage (%)
    pub fn usage_percent(&self) -> f64 {
        *self.usage_percent.current()
    }

    /// Get latest GPU frequency (MHz)
    pub fn frequency_mhz(&self) -> u64 {
        *self.frequency_mhz.current()
    }

    /// Get historical GPU usage (%)
    pub fn usage_history(&self) -> &VecDeque<f64> {
        self.usage_percent.history()
    }

    /// Get historical GPU frequency (MHz)
    pub fn frequency_history(&self) -> &VecDeque<u64> {
        self.frequency_mhz.history()
    }

    /// Get GPU device path
    pub fn gpu_path(&self) -> &str {
        &self.gpu_path
    }

    fn update_gpu_load(&mut self) -> Result<(), AppError> {
        let load_path = format!("{}/load", self.gpu_path);
        if Path::new(&load_path).exists() {
            let load_content = fs::read_to_string(&load_path)
                .map_err(|e| AppError::System(format!("Failed to read {}: {}", load_path, e)))?;
            
            // Handle different load formats
            let load_str = load_content.trim();
            let load: u64 = if load_str.contains('@') {
                // Format: "0@300000000Hz" - extract the first number
                load_str.split('@').next()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0)
            } else {
                // Standard format: just a number
                load_str.parse().unwrap_or(0)
            };
            
            // Convert load to percentage (assuming load is in the range 0-100)
            let usage = load.min(100) as f64;
            self.usage_percent.update(usage);
        }
        Ok(())
    }

    fn update_gpu_frequency(&mut self) -> Result<(), AppError> {
        let freq_path = format!("{}/cur_freq", self.gpu_path);
        if Path::new(&freq_path).exists() {
            let freq_content = fs::read_to_string(&freq_path)
                .map_err(|e| AppError::System(format!("Failed to read {}: {}", freq_path, e)))?;
            
            let freq_khz: u64 = freq_content.trim()
                .parse()
                .map_err(|e| AppError::System(format!("Failed to parse frequency from {}: {}", freq_path, e)))?;
            
            // Convert from Hz to MHz
            let freq_mhz = freq_khz / 1000000;
            self.frequency_mhz.update(freq_mhz);
        }
        Ok(())
    }

    fn find_gpu_devfreq_path() -> Result<String, AppError> {
        // Common GPU devfreq paths to try
        let possible_paths = [
            "/sys/class/devfreq/fb000000.gpu",
            "/sys/class/devfreq/10000000.gpu",
            "/sys/class/devfreq/gpu",
        ];

        for path in &possible_paths {
            if Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        // If no common paths found, try to find any GPU devfreq device
        let devfreq_dir = "/sys/class/devfreq";
        if let Ok(entries) = fs::read_dir(devfreq_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let path_str = path.to_string_lossy();
                        if path_str.contains("gpu") || path_str.contains("fb") {
                            return Ok(path_str.to_string());
                        }
                    }
                }
            }
        }

        Err(AppError::System("No GPU devfreq device found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_metrics_creation() {
        // This test will only pass if running on a system with GPU devfreq support
        if let Ok(gpu) = UnixGpuMetrics::new() {
            assert!(!gpu.gpu_path().is_empty());
        }
    }

    #[test]
    fn test_gpu_metrics_with_custom_path() {
        let gpu = UnixGpuMetrics::with_path("/sys/class/devfreq/test.gpu".to_string());
        assert_eq!(gpu.gpu_path(), "/sys/class/devfreq/test.gpu");
    }
} 