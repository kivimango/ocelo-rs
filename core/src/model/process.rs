use serde::{Deserialize, Serialize};
use sysinfo::{Process, Users};

const NOT_FOUND: &str = "N/A";
pub type ProcessList = Vec<ProcessInfo>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// The ID of the process
    pub pid: u32,

    /// Name of the process.
    /// It will be filled by NOT_FOUND if the name of the process cannot be acquired
    pub name: String,

    /// Used physical memory in bytes by the process
    pub memory: u64,

    /// Used virtual memory in bytes by the process
    pub virtual_memory: u64,

    /// Current CPU usage percent by the process
    pub cpu_usage: f32,

    /// Accumulated CPU time
    pub cpu_time: u64,

    /// Name of the user who launched the process.
    /// It will be filled by NOT_FOUND if the owner of the process cannot be acquired
    pub username: String,

    /// Total runtime of the process in seconds
    pub running_time: u64,

    /// The path where the process started from
    pub command: String,
}

impl From<&Process> for ProcessInfo {
    fn from(proc: &Process) -> Self {
        let users = Users::new_with_refreshed_list();

        ProcessInfo {
            pid: proc.pid().as_u32(),
            name: proc
                .name()
                .to_owned()
                .into_string()
                .unwrap_or(NOT_FOUND.to_string()),
            memory: proc.memory(),
            virtual_memory: proc.virtual_memory(),
            cpu_usage: proc.cpu_usage(),
            cpu_time: proc.accumulated_cpu_time(),
            username: proc.user_id().map_or(NOT_FOUND.to_string(), |uid| {
                users
                    .get_user_by_id(uid)
                    .map_or(NOT_FOUND.to_string(), |user| user.name().to_owned())
            }),
            running_time: proc.run_time(),
            command: proc.exe().map_or(NOT_FOUND.to_string(), |path| {
                path.to_string_lossy().to_string()
            }),
        }
    }
}

/// Serializes the process `list` into the JSON representation.
pub fn process_list_to_json(list: ProcessList) -> Result<String, serde_json::Error> {
    serde_json::to_string(&list)
}

/// Deserializes the JSON representation back into `ProcessList`.
pub fn process_list_from_json(json: &str) -> Result<ProcessList, serde_json::Error> {
    serde_json::from_str(json)
}
