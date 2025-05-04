use serde::{Deserialize, Serialize};
use sysinfo::Disk;

/// Information collected about a storage device.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Storage {
    pub total_space: u64,
    pub used_space: u64,
    pub available_space: u64,
    pub file_system: String,
    pub mount: String,
}

impl From<&Disk> for Storage {
    fn from(disk: &Disk) -> Self {
        Storage {
            total_space: disk.total_space(),
            used_space: disk.total_space() - disk.available_space(),
            available_space: disk.available_space(),
            file_system: disk.file_system().to_string_lossy().into_owned(),
            mount: disk.mount_point().to_string_lossy().to_string(),
        }
    }
}

/// Information collected about the mass storage on the host machine.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DiskInfo {
    pub disks: Vec<Storage>,
}
