use serde::{Deserialize, Serialize};

mod cpu;
mod disk;
mod system;

pub use cpu::*;
pub use disk::*;
pub use system::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Stores memory-related statistics.
pub struct MemoryInfo {
    /// Total available memory for this machine in bytes
    pub total: u64,
    /// Used memory from in bytes
    pub used: u64,
    /// Reamining available memory from `self.total` in bytes
    pub available: u64,
    /// Swap page/file information
    pub swap_total: u64,
    pub swap_used: u64,
    pub swap_available: u64,
}

/// Collection of system information to be displayed in the Overview component.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SystemOverviewInfo {
    pub cpu: CpuInfo,
    pub overview: SystemInfo,
    pub memory: MemoryInfo,
    pub disks: DiskInfo,
}

impl SystemOverviewInfo {
    /// Creates `self` from a JSON reprentation
    pub fn from_json(value: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(value)
    }
}
