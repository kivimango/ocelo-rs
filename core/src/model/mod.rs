use serde::{Deserialize, Serialize};

mod cpu;
mod system;

pub use cpu::*;
pub use system::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Stores memory-related statistics.
pub struct MemoryInfo {
    /// Total avaiable memory for this machine in bytes
    pub total: u64,
    /// Used memory from in bytes
    pub used: u64,
    /// Reamining available memory from `self.total` in bytes
    pub available: u64,
}

/// Collection of system information to be displayed in the Overview component.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SystemOverviewInfo {
    pub cpu: CpuInfo,
    pub overview: SystemInfo,
    pub memory: MemoryInfo,
}

impl SystemOverviewInfo {
    /// Creates `self` from a JSON reprentation
    pub fn from_json(value: &String) -> Result<Self, serde_json::Error> {
        serde_json::from_str(&value)
    }
}
