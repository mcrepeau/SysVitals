//! System metrics collection and management

use crate::core::error::AppError;
use crate::metrics::{cpu, disk, gpu, memory, network};
use sysinfo::System;
use std::time::Duration;

const HISTORY_WINDOW: Duration = Duration::from_secs(120);

/// System metrics collector
pub struct SystemMetrics {
    system: System,
    cpu: cpu::CpuMetrics,
    memory: memory::MemoryMetrics,
    network: network::NetworkMetrics,
    disk: disk::DiskMetrics,
    gpu: Option<gpu::GpuMetrics>,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMetrics {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let cpu = cpu::CpuMetrics::new(&system);
        let memory = memory::MemoryMetrics::new(&system);
        let network = network::NetworkMetrics::new();
        let disk = disk::DiskMetrics::new();
        let gpu = gpu::GpuMetrics::new().ok();
        Self { system, cpu, memory, network, disk, gpu }
    }

    /// Update all metrics
    pub fn update(&mut self) -> Result<(), AppError> {
        self.cpu.update(&mut self.system)?;
        self.memory.update(&mut self.system)?;
        self.network.update()?;
        self.disk.update()?;
        if let Some(gpu) = &mut self.gpu {
            gpu.update()?;
        }
        Ok(())
    }

    /// Resize all history buffers to hold `sample_interval`-spaced samples covering
    /// a fixed 2-minute window.
    pub fn resize_history(&mut self, sample_interval: Duration) {
        let len = (HISTORY_WINDOW.as_secs_f64() / sample_interval.as_secs_f64()).ceil() as usize;
        self.cpu.resize_history(len);
        self.memory.resize_history(len);
        self.network.resize_history(len);
        self.disk.resize_history(len);
        if let Some(gpu) = &mut self.gpu {
            gpu.resize_history(len);
        }
    }

    pub fn cpu(&self) -> &cpu::CpuMetrics { &self.cpu }
    pub fn memory(&self) -> &memory::MemoryMetrics { &self.memory }
    pub fn network(&self) -> &network::NetworkMetrics { &self.network }
    pub fn disk(&self) -> &disk::DiskMetrics { &self.disk }
    pub fn gpu(&self) -> Option<&gpu::GpuMetrics> { self.gpu.as_ref() }
}
