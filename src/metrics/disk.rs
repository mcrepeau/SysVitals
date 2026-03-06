//! Disk I/O metrics collection

use sysinfo::{DiskRefreshKind, Disks};
use crate::core::error::AppError;
use crate::metrics::historical_metric::HistoricalMetric;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const IO_REFRESH: fn() -> DiskRefreshKind = || DiskRefreshKind::nothing().with_io_usage();

/// Aggregated disk I/O metrics (read/write rates across all disks)
pub struct DiskMetrics {
    disks: Disks,
    read_rate: HistoricalMetric<f64>,
    write_rate: HistoricalMetric<f64>,
    last_update: Instant,
}

impl DiskMetrics {
    pub fn new() -> Self {
        Self {
            disks: Disks::new_with_refreshed_list_specifics(IO_REFRESH()),
            read_rate: HistoricalMetric::new(0.0),
            write_rate: HistoricalMetric::new(0.0),
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self) -> Result<(), AppError> {
        self.disks.refresh_specifics(false, IO_REFRESH());

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);

        if elapsed < Duration::from_millis(100) {
            return Ok(());
        }

        let elapsed_secs = elapsed.as_secs_f64();

        let total_read: u64 = self.disks.list().iter().map(|d| d.usage().read_bytes).sum();
        let total_write: u64 = self.disks.list().iter().map(|d| d.usage().written_bytes).sum();

        self.read_rate.update(total_read as f64 / (1024.0 * 1024.0 * elapsed_secs));
        self.write_rate.update(total_write as f64 / (1024.0 * 1024.0 * elapsed_secs));
        self.last_update = now;

        Ok(())
    }

    pub fn read_rate(&self) -> f64 { *self.read_rate.current() }
    pub fn write_rate(&self) -> f64 { *self.write_rate.current() }
    pub fn read_history(&self) -> &VecDeque<f64> { self.read_rate.history() }
    pub fn write_history(&self) -> &VecDeque<f64> { self.write_rate.history() }

    pub fn resize_history(&mut self, len: usize) {
        self.read_rate.resize(len);
        self.write_rate.resize(len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_metrics() {
        let mut disk = DiskMetrics::new();
        std::thread::sleep(Duration::from_millis(150));
        assert!(disk.update().is_ok());
        assert!(disk.read_rate() >= 0.0);
        assert!(disk.write_rate() >= 0.0);
        assert!(!disk.read_history().is_empty());
        assert!(!disk.write_history().is_empty());
    }
}
