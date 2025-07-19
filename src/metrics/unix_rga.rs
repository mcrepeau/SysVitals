//! Unix-based RGA metrics collection using /sys/kernel/debug/rkrga/ and /sys/kernel/debug/clk/

use crate::core::error::AppError;
use crate::metrics::historical_metric::HistoricalMetric;
use std::collections::VecDeque;
use std::process::Command;

/// Unix-based RGA metrics
pub struct UnixRgaMetrics {
    usage_percent: HistoricalMetric<f64>,
    frequency_mhz: HistoricalMetric<u64>,
}

impl UnixRgaMetrics {
    /// Create a new Unix-based RGA metrics collector
    pub fn new() -> Self {
        Self {
            usage_percent: HistoricalMetric::new(0.0),
            frequency_mhz: HistoricalMetric::new(0),
        }
    }

    /// Update RGA metrics
    pub fn update(&mut self) -> Result<(), AppError> {
        // Update RGA load
        self.update_rga_load()?;
        
        // Update RGA frequency
        self.update_rga_frequency()?;
        
        Ok(())
    }

    /// Get latest RGA usage (%)
    pub fn usage_percent(&self) -> f64 {
        *self.usage_percent.current()
    }

    /// Get latest RGA frequency (MHz)
    pub fn frequency_mhz(&self) -> u64 {
        *self.frequency_mhz.current()
    }

    /// Get historical RGA usage (%)
    pub fn usage_history(&self) -> &VecDeque<f64> {
        self.usage_percent.history()
    }

    /// Get historical RGA frequency (MHz)
    pub fn frequency_history(&self) -> &VecDeque<u64> {
        self.frequency_mhz.history()
    }

    fn update_rga_load(&mut self) -> Result<(), AppError> {
        // Use sudo to read from /sys/kernel/debug/rkrga/load
        let output = Command::new("sudo")
            .arg("cat")
            .arg("/sys/kernel/debug/rkrga/load")
            .output()
            .map_err(|e| AppError::System(format!("Failed to execute sudo cat /sys/kernel/debug/rkrga/load: {}", e)))?;
        
        if output.status.success() {
            let load_content = String::from_utf8(output.stdout)
                .map_err(|e| AppError::System(format!("Failed to parse output from /sys/kernel/debug/rkrga/load: {}", e)))?;
            
            // Parse RGA load format with multiple schedulers
            let load_str = load_content.trim();
            let load: u64 = if load_str.contains("load =") {
                // Extract the first load percentage value
                load_str.lines()
                    .find(|line| line.contains("load ="))
                    .and_then(|line| line.split("load =").nth(1))
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
        Ok(())
    }

    fn update_rga_frequency(&mut self) -> Result<(), AppError> {
        // Use sudo to read from /sys/kernel/debug/clk/clk_summary and grep for rga
        let output = Command::new("sudo")
            .arg("cat")
            .arg("/sys/kernel/debug/clk/clk_summary")
            .output()
            .map_err(|e| AppError::System(format!("Failed to execute sudo cat /sys/kernel/debug/clk/clk_summary: {}", e)))?;
        
        if output.status.success() {
            let clk_content = String::from_utf8(output.stdout)
                .map_err(|e| AppError::System(format!("Failed to parse output from /sys/kernel/debug/clk/clk_summary: {}", e)))?;
            
            // Parse the frequency from the clk_summary output
            if let Some(freq) = Self::extract_rga_frequency(&clk_content) {
                self.frequency_mhz.update(freq);
            }
        }
        Ok(())
    }

    fn extract_rga_frequency(clk_content: &str) -> Option<u64> {
        // Look for RGA-related lines in the clk_summary
        for line in clk_content.lines() {
            if line.contains("rga") || line.contains("RGA") {
                // Try to extract frequency from the line
                // Format from RK3588: "clk_rga3_0_core 0 1 0 750000000 0 0 50000 N"
                let parts: Vec<&str> = line.split_whitespace().collect();
                for (i, part) in parts.iter().enumerate() {
                    if part.contains("rga") || part.contains("RGA") {
                        // Look for frequency in subsequent parts (usually the 4th field)
                        for j in (i + 1)..parts.len() {
                            if let Ok(freq) = parts[j].parse::<u64>() {
                                // Convert to MHz if needed (assuming input is in Hz)
                                if freq > 1000000 {
                                    return Some(freq / 1000000); // Convert Hz to MHz
                                } else {
                                    return Some(freq); // Already in MHz
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rga_metrics_creation() {
        let rga = UnixRgaMetrics::new();
        assert_eq!(rga.usage_percent(), 0.0);
        assert_eq!(rga.frequency_mhz(), 0);
    }

    #[test]
    fn test_extract_rga_frequency() {
        let test_content = "rga_clk    1000000000\nother_clk  500000000";
        let freq = UnixRgaMetrics::extract_rga_frequency(test_content);
        assert_eq!(freq, Some(1000)); // 1000000000 Hz = 1000 MHz
    }

    #[test]
    fn test_extract_rga_frequency_no_match() {
        let test_content = "cpu_clk    2000000000\nother_clk  500000000";
        let freq = UnixRgaMetrics::extract_rga_frequency(test_content);
        assert_eq!(freq, None);
    }
} 