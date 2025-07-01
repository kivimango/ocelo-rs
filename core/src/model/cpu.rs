use serde::{Deserialize, Serialize};

use super::MemoryInfo;

///  Detailed information collected about the main processor.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// Full name of the processor, like Intel Core i5-6500
    pub name: String,

    /// Current frequency in MHz
    pub frequency: u64,

    /// The physical core count
    pub core_count: usize,

    /// CPU usage in percentage
    pub usage: f32,

    /// The reported temperature of the processor.
    /// On some platforms, this information is not available
    pub temperature: Option<f32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CpuCore {
    pub usage: u64,
    pub frequency: u64,
    pub temperature: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CpuMemoryUpdate {
    pub usage: f32,
    pub frequency: usize,
    pub temperature: usize,
    pub cores: Vec<CpuCore>,
    pub memory_stats: MemoryInfo,
}

impl CpuMemoryUpdate {
    /// Creates `self` from a JSON reprentation.
    pub fn from_json(value: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(value)
    }

    /// Creates tje JSON representation of `self`.
    pub fn to_json(self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}
