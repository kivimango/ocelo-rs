use serde::{Deserialize, Serialize};

mod cpu;
mod system;

pub use cpu::*;
pub use system::*;

/// Collection of system information to be displayed in the Overview component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOverviewInfo {
    pub cpu: CpuInfo,
    pub overview: SystemInfo,
}
