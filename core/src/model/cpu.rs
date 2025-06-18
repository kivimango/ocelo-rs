use serde::{Deserialize, Serialize};

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

pub struct CpuCore {
    pub usage: u64,
    pub frequency: u64,
    pub temperature: u32,
}
