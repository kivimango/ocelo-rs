use super::SystemOverviewInfo;
use serde::{Deserialize, Serialize};

/// Some details describing the host system.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub host_name: String,
    pub kernel_version: String,
    pub uptime: u64,
    pub load_one_minute: f64,
    pub load_five_minutes: f64,
    pub load_fifteen_minutes: f64,
}

impl SystemOverviewInfo {
    /// Creates the JSON representation of `self ̇.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}
