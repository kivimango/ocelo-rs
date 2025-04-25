/// Some details describing the host system.
#[derive(Debug, Default, Clone)]
pub struct SystemInfo {
    pub host_name: String,
    pub kernel_version: String,
    pub uptime: u64,
    pub load_one_minute: f64,
    pub load_five_minutes: f64,
    pub load_fifteen_minutes: f64,
}
