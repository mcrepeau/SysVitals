//! Unified Unix-based metrics collection system

use crate::core::error::AppError;
use crate::metrics::{
    unix_cpu::UnixCpuMetrics,
    unix_gpu::UnixGpuMetrics,
    unix_npu::UnixNpuMetrics,
    unix_rga::UnixRgaMetrics,
};

/// Unified Unix-based system metrics collector
pub struct UnixSystemMetrics {
    cpu: Option<UnixCpuMetrics>,
    gpu: Option<UnixGpuMetrics>,
    npu: Option<UnixNpuMetrics>,
    rga: Option<UnixRgaMetrics>,
}

impl UnixSystemMetrics {
    /// Create a new Unix-based metrics collector
    pub fn new() -> Self {
        let cpu = UnixCpuMetrics::new().ok();
        let gpu = UnixGpuMetrics::new().ok();
        let npu = UnixNpuMetrics::new().ok();
        let rga = Some(UnixRgaMetrics::new());
        
        Self {
            cpu,
            gpu,
            npu,
            rga,
        }
    }

    /// Create a new Unix-based metrics collector with custom paths
    pub fn with_paths(
        gpu_path: Option<String>,
        npu_path: Option<String>,
    ) -> Self {
        let cpu = UnixCpuMetrics::new().ok();
        let gpu = gpu_path.map(|path| UnixGpuMetrics::with_path(path));
        let npu = npu_path.map(|path| UnixNpuMetrics::with_path(path));
        let rga = Some(UnixRgaMetrics::new());
        
        Self {
            cpu,
            gpu,
            npu,
            rga,
        }
    }

    /// Update all metrics
    pub fn update(&mut self) -> Result<(), AppError> {
        if let Some(cpu) = &mut self.cpu {
            cpu.update()?;
        }
        
        if let Some(gpu) = &mut self.gpu {
            gpu.update()?;
        }
        
        if let Some(npu) = &mut self.npu {
            npu.update()?;
        }
        
        if let Some(rga) = &mut self.rga {
            rga.update()?;
        }
        
        Ok(())
    }

    /// Get CPU metrics
    pub fn cpu(&self) -> Option<&UnixCpuMetrics> {
        self.cpu.as_ref()
    }

    /// Get GPU metrics
    pub fn gpu(&self) -> Option<&UnixGpuMetrics> {
        self.gpu.as_ref()
    }

    /// Get NPU metrics
    pub fn npu(&self) -> Option<&UnixNpuMetrics> {
        self.npu.as_ref()
    }

    /// Get RGA metrics
    pub fn rga(&self) -> Option<&UnixRgaMetrics> {
        self.rga.as_ref()
    }

    /// Check if CPU metrics are available
    pub fn has_cpu(&self) -> bool {
        self.cpu.is_some()
    }

    /// Check if GPU metrics are available
    pub fn has_gpu(&self) -> bool {
        self.gpu.is_some()
    }

    /// Check if NPU metrics are available
    pub fn has_npu(&self) -> bool {
        self.npu.is_some()
    }

    /// Check if RGA metrics are available
    pub fn has_rga(&self) -> bool {
        self.rga.is_some()
    }

    /// Get a summary of available metrics
    pub fn available_metrics(&self) -> Vec<&'static str> {
        let mut metrics = Vec::new();
        
        if self.has_cpu() {
            metrics.push("CPU");
        }
        if self.has_gpu() {
            metrics.push("GPU");
        }
        if self.has_npu() {
            metrics.push("NPU");
        }
        if self.has_rga() {
            metrics.push("RGA");
        }
        
        metrics
    }
}

impl Default for UnixSystemMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_metrics_creation() {
        let metrics = UnixSystemMetrics::new();
        // At least CPU should be available on Unix systems
        assert!(metrics.has_cpu());
    }

    #[test]
    fn test_unix_metrics_with_custom_paths() {
        let metrics = UnixSystemMetrics::with_paths(
            Some("/sys/class/devfreq/test.gpu".to_string()),
            Some("/sys/class/devfreq/test.npu".to_string()),
        );
        
        assert!(metrics.has_cpu());
        assert!(metrics.has_gpu());
        assert!(metrics.has_npu());
        assert!(metrics.has_rga());
    }

    #[test]
    fn test_available_metrics() {
        let metrics = UnixSystemMetrics::new();
        let available = metrics.available_metrics();
        assert!(!available.is_empty());
        assert!(available.contains(&"CPU"));
    }
} 