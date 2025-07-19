//! System metrics collection and management

use crate::core::error::AppError;
use crate::metrics::{cpu, memory, network, gpu};
use sysinfo::System;

/// System metrics collector
pub struct SystemMetrics {
    system: System,
    cpu: cpu::CpuMetrics,
    memory: memory::MemoryMetrics,
    network: network::NetworkMetrics,
    gpu: Option<gpu::GpuMetrics>,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMetrics {
    /// Create a new metrics metrics collector
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let cpu = cpu::CpuMetrics::new(&system);
        let memory = memory::MemoryMetrics::new(&system);
        let network = network::NetworkMetrics::new();
        let gpu = gpu::GpuMetrics::new().ok();
        Self {
            system,
            cpu,
            memory,
            network,
            gpu,
        }
    }

    /// Update all metrics metrics
    pub fn update(&mut self) -> Result<(), AppError> {
        self.system.refresh_all();
        self.cpu.update(&mut self.system)?;
        self.memory.update(&mut self.system)?;
        self.network.update()?;
        if let Some(gpu) = &mut self.gpu {
            gpu.update()?;
        }
        Ok(())
    }

    /// Get a reference to CPU metrics
    pub fn cpu(&self) -> &cpu::CpuMetrics {
        &self.cpu
    }

    /// Get a reference to memory metrics
    pub fn memory(&self) -> &memory::MemoryMetrics {
        &self.memory
    }

    /// Get a reference to network metrics
    pub fn network(&self) -> &network::NetworkMetrics {
        &self.network
    }

    /// Get an optional reference to GPU metrics
    pub fn gpu(&self) -> Option<&gpu::GpuMetrics> {
        self.gpu.as_ref()
    }
}
