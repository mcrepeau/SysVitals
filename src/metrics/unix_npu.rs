//! Unix-based NPU metrics collection using /sys/kernel/debug/rknpu/ and /sys/class/devfreq/

use crate::core::error::AppError;
use crate::metrics::historical_metric::HistoricalMetric;
use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Unix-based NPU metrics
pub struct UnixNpuMetrics {
    usage_percent: HistoricalMetric<f64>,
    frequency_mhz: HistoricalMetric<u64>,
    npu_path: String,
}

impl UnixNpuMetrics {
    /// Create a new Unix-based NPU metrics collector
    pub fn new() -> Result<Self, AppError> {
        let npu_path = Self::find_npu_devfreq_path()?;
        
        Ok(Self {
            usage_percent: HistoricalMetric::new(0.0),
            frequency_mhz: HistoricalMetric::new(0),
            npu_path,
        })
    }

    /// Create a new Unix-based NPU metrics collector with custom NPU path
    pub fn with_path(npu_path: String) -> Self {
        Self {
            usage_percent: HistoricalMetric::new(0.0),
            frequency_mhz: HistoricalMetric::new(0),
            npu_path,
        }
    }

    /// Update NPU metrics
    pub fn update(&mut self) -> Result<(), AppError> {
        // Update NPU load
        self.update_npu_load()?;
        
        // Update NPU frequency
        self.update_npu_frequency()?;
        
        Ok(())
    }

    /// Get latest NPU usage (%)
    pub fn usage_percent(&self) -> f64 {
        *self.usage_percent.current()
    }

    /// Get latest NPU frequency (MHz)
    pub fn frequency_mhz(&self) -> u64 {
        *self.frequency_mhz.current()
    }

    /// Get historical NPU usage (%)
    pub fn usage_history(&self) -> &VecDeque<f64> {
        self.usage_percent.history()
    }

    /// Get historical NPU frequency (MHz)
    pub fn frequency_history(&self) -> &VecDeque<u64> {
        self.frequency_mhz.history()
    }

    /// Get NPU device path
    pub fn npu_path(&self) -> &str {
        &self.npu_path
    }

    fn update_npu_load(&mut self) -> Result<(), AppError> {
        // Try to read from /sys/kernel/debug/rknpu/load using sudo
        let load_path = "/sys/kernel/debug/rknpu/load";
        if Path::new(load_path).exists() {
            // Use sudo to read the debug file
            let output = Command::new("sudo")
                .arg("cat")
                .arg(load_path)
                .output()
                .map_err(|e| AppError::System(format!("Failed to execute sudo cat {}: {}", load_path, e)))?;
            
            if output.status.success() {
                let load_content = String::from_utf8(output.stdout)
                    .map_err(|e| AppError::System(format!("Failed to parse output from {}: {}", load_path, e)))?;
                
                // Parse NPU load format: "NPU load: Core0: 0%, Core1: 0%, Core2: 0%,"
                let load_str = load_content.trim();
                let load: u64 = if load_str.contains("NPU load:") {
                    // Extract the first percentage value
                    load_str.split("Core0:")
                        .nth(1)
                        .and_then(|s| s.split('%').next())
                        .and_then(|s| s.trim().parse::<u64>().ok())
                        .unwrap_or(0)
                } else {
                    // Standard format: just a number
                    load_str.parse().unwrap_or(0)
                };
                
                // Convert load to percentage (assuming load is in the range 0-100)
                let usage = load.min(100) as f64;
                self.usage_percent.update(usage);
            }
        }
        Ok(())
    }

    fn update_npu_frequency(&mut self) -> Result<(), AppError> {
        let freq_path = format!("{}/cur_freq", self.npu_path);
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

    fn find_npu_devfreq_path() -> Result<String, AppError> {
        // Common NPU devfreq paths to try
        let possible_paths = [
            "/sys/class/devfreq/fdab0000.npu",
            "/sys/class/devfreq/10000000.npu",
            "/sys/class/devfreq/npu",
        ];

        for path in &possible_paths {
            if Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        // If no common paths found, try to find any NPU devfreq device
        let devfreq_dir = "/sys/class/devfreq";
        if let Ok(entries) = fs::read_dir(devfreq_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let path_str = path.to_string_lossy();
                        if path_str.contains("npu") || path_str.contains("fdab") {
                            return Ok(path_str.to_string());
                        }
                    }
                }
            }
        }

        Err(AppError::System("No NPU devfreq device found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npu_metrics_creation() {
        // This test will only pass if running on a system with NPU devfreq support
        if let Ok(npu) = UnixNpuMetrics::new() {
            assert!(!npu.npu_path().is_empty());
        }
    }

    #[test]
    fn test_npu_metrics_with_custom_path() {
        let npu = UnixNpuMetrics::with_path("/sys/class/devfreq/test.npu".to_string());
        assert_eq!(npu.npu_path(), "/sys/class/devfreq/test.npu");
    }
} 