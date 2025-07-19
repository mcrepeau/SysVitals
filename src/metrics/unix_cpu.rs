//! Unix-based CPU metrics collection using /proc/stat and /sys/devices/system/cpu/

use crate::core::error::AppError;
use crate::metrics::historical_metric::HistoricalMetric;
use std::collections::VecDeque;
use std::fs;
use std::path::Path;

/// Unix-based CPU metrics
pub struct UnixCpuMetrics {
    usage_percent: HistoricalMetric<f64>,
    frequencies: Vec<HistoricalMetric<u64>>,
    cpu_count: usize,
    prev_stats: Option<CpuStats>,
}

#[derive(Debug, Clone)]
struct CpuStats {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
    guest: u64,
    guest_nice: u64,
}

impl UnixCpuMetrics {
    /// Create a new Unix-based CPU metrics collector
    pub fn new() -> Result<Self, AppError> {
        let cpu_count = Self::get_cpu_count()?;
        let frequencies = vec![HistoricalMetric::new(0); cpu_count];
        
        Ok(Self {
            usage_percent: HistoricalMetric::new(0.0),
            frequencies,
            cpu_count,
            prev_stats: None,
        })
    }

    /// Update CPU metrics
    pub fn update(&mut self) -> Result<(), AppError> {
        // Update CPU usage from /proc/stat
        self.update_cpu_usage()?;
        
        // Update CPU frequencies
        self.update_cpu_frequencies()?;
        
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

    /// Get current frequency for a specific CPU core (in MHz)
    pub fn frequency_mhz(&self, core: usize) -> Option<u64> {
        if core < self.frequencies.len() {
            Some(*self.frequencies[core].current() / 1000) // Convert from kHz to MHz
        } else {
            None
        }
    }

    /// Get all CPU frequencies (in MHz)
    pub fn all_frequencies_mhz(&self) -> Vec<u64> {
        self.frequencies.iter()
            .map(|f| *f.current() / 1000)
            .collect()
    }

    /// Get CPU count
    pub fn cpu_count(&self) -> usize {
        self.cpu_count
    }

    fn update_cpu_usage(&mut self) -> Result<(), AppError> {
        let stat_content = fs::read_to_string("/proc/stat")
            .map_err(|e| AppError::System(format!("Failed to read /proc/stat: {}", e)))?;
        
        let lines: Vec<&str> = stat_content.lines().collect();
        let cpu_line = lines.first()
            .ok_or_else(|| AppError::System("No CPU line found in /proc/stat".to_string()))?;
        
        let stats = Self::parse_cpu_line(cpu_line)?;
        
        if let Some(prev_stats) = &self.prev_stats {
            let usage = Self::calculate_cpu_usage(prev_stats, &stats);
            self.usage_percent.update(usage);
        }
        
        self.prev_stats = Some(stats);
        Ok(())
    }

    fn update_cpu_frequencies(&mut self) -> Result<(), AppError> {
        for i in 0..self.cpu_count {
            let freq_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq", i);
            if Path::new(&freq_path).exists() {
                let freq_content = fs::read_to_string(&freq_path)
                    .map_err(|e| AppError::System(format!("Failed to read {}: {}", freq_path, e)))?;
                
                let freq: u64 = freq_content.trim()
                    .parse()
                    .map_err(|e| AppError::System(format!("Failed to parse frequency from {}: {}", freq_path, e)))?;
                
                self.frequencies[i].update(freq);
            }
        }
        Ok(())
    }

    fn get_cpu_count() -> Result<usize, AppError> {
        let stat_content = fs::read_to_string("/proc/stat")
            .map_err(|e| AppError::System(format!("Failed to read /proc/stat: {}", e)))?;
        
        let cpu_lines = stat_content.lines()
            .filter(|line| line.starts_with("cpu") && !line.starts_with("cpu "))
            .count();
        
        Ok(cpu_lines)
    }

    fn parse_cpu_line(line: &str) -> Result<CpuStats, AppError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 11 {
            return Err(AppError::System("Invalid CPU line format in /proc/stat".to_string()));
        }

        Ok(CpuStats {
            user: parts[1].parse().unwrap_or(0),
            nice: parts[2].parse().unwrap_or(0),
            system: parts[3].parse().unwrap_or(0),
            idle: parts[4].parse().unwrap_or(0),
            iowait: parts[5].parse().unwrap_or(0),
            irq: parts[6].parse().unwrap_or(0),
            softirq: parts[7].parse().unwrap_or(0),
            steal: parts[8].parse().unwrap_or(0),
            guest: parts[9].parse().unwrap_or(0),
            guest_nice: parts[10].parse().unwrap_or(0),
        })
    }

    fn calculate_cpu_usage(prev: &CpuStats, curr: &CpuStats) -> f64 {
        let prev_total = prev.user + prev.nice + prev.system + prev.idle + 
                        prev.iowait + prev.irq + prev.softirq + prev.steal;
        let curr_total = curr.user + curr.nice + curr.system + curr.idle + 
                        curr.iowait + curr.irq + curr.softirq + curr.steal;
        
        let prev_idle = prev.idle + prev.iowait;
        let curr_idle = curr.idle + curr.iowait;
        
        let total_diff = curr_total - prev_total;
        let idle_diff = curr_idle - prev_idle;
        
        if total_diff == 0 {
            0.0
        } else {
            ((total_diff - idle_diff) as f64 / total_diff as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu_line() {
        let line = "cpu  123456 789 12345 678901 1234 567 890 123 456 789";
        let stats = UnixCpuMetrics::parse_cpu_line(line).unwrap();
        assert_eq!(stats.user, 123456);
        assert_eq!(stats.nice, 789);
        assert_eq!(stats.system, 12345);
        assert_eq!(stats.idle, 678901);
    }

    #[test]
    fn test_calculate_cpu_usage() {
        let prev = CpuStats {
            user: 100, nice: 10, system: 50, idle: 200, iowait: 20,
            irq: 5, softirq: 15, steal: 0, guest: 0, guest_nice: 0,
        };
        let curr = CpuStats {
            user: 150, nice: 15, system: 75, idle: 250, iowait: 25,
            irq: 8, softirq: 20, steal: 0, guest: 0, guest_nice: 0,
        };
        
        let usage = UnixCpuMetrics::calculate_cpu_usage(&prev, &curr);
        assert!(usage > 0.0 && usage < 100.0);
    }
} 