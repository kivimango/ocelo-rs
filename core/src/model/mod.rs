use serde::{Deserialize, Serialize};

mod cpu;
mod system;

pub use cpu::*;
pub use system::*;

/// Collection of system information to be displayed in the Overview component.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SystemOverviewInfo {
    pub cpu: CpuInfo,
    pub overview: SystemInfo,
}

impl SystemOverviewInfo {
    /// Creates `self` from a JSON reprentation
    pub fn from_json(value: &String) -> Result<Self, serde_json::Error> {
        serde_json::from_str(&value)
    }
}
