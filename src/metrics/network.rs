//! Network metrics collection

use sysinfo::Networks;
use crate::core::error::AppError;
use crate::metrics::historical_metric::HistoricalMetric;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Network metrics
pub struct NetworkMetrics {
    networks: Networks,
    interface_stats: HashMap<String, (HistoricalMetric<f64>, HistoricalMetric<f64>)>,
    last_update: Instant,
}

impl NetworkMetrics {
    /// Create a new network metrics collector
    pub fn new() -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
            interface_stats: HashMap::new(),
            last_update: Instant::now(),
        }
    }

    /// Update network metrics
    pub fn update(&mut self) -> Result<(), AppError> {
        self.networks.refresh(true);

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);

        if elapsed < Duration::from_millis(100) {
            return Ok(());
        }

        let elapsed_secs = elapsed.as_secs_f64();
        if elapsed_secs == 0.0 {
            return Ok(());
        }

        for (name, data) in self.networks.iter() {
            let rx = data.received();
            let tx = data.transmitted();

            let entry = self.interface_stats
                .entry(name.to_string())
                .or_insert_with(|| (HistoricalMetric::new(0.0), HistoricalMetric::new(0.0)));

            let rx_mbps = (rx as f64 * 8.0) / (1_000_000.0 * elapsed_secs);
            let tx_mbps = (tx as f64 * 8.0) / (1_000_000.0 * elapsed_secs);

            entry.0.update(rx_mbps);
            entry.1.update(tx_mbps);
        }

        self.last_update = now;
        Ok(())
    }

    /// Get names of all interfaces
    pub fn interface_names(&self) -> Vec<String> {
        self.interface_stats.keys().cloned().collect()
    }

    /// Get current Mbps for a specific interface
    pub fn get_interface_stats(&self, name: &str) -> Option<(&HistoricalMetric<f64>, &HistoricalMetric<f64>)> {
        self.interface_stats.get(name).map(|(rx, tx)| (rx, tx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_metrics() {
        let mut network = NetworkMetrics::new();
        std::thread::sleep(std::time::Duration::from_millis(150));
        assert!(network.update().is_ok());

        let interfaces = network.interface_names();
        assert!(!interfaces.is_empty());

        for iface in interfaces {
            let (rx_hist, tx_hist) = network.get_interface_stats(&iface).unwrap();
            assert!(rx_hist.current() >= &0.0);
            assert!(tx_hist.current() >= &0.0);
            assert!(!rx_hist.history().is_empty());
            assert!(!tx_hist.history().is_empty());
        }
    }
}