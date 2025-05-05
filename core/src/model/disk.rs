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
    pub bytes_read: u64,
    pub bytes_written: u64,
}

impl From<&Disk> for Storage {
    fn from(disk: &Disk) -> Self {
        Storage {
            total_space: disk.total_space(),
            used_space: disk.total_space() - disk.available_space(),
            available_space: disk.available_space(),
            file_system: disk.file_system().to_string_lossy().into_owned(),
            mount: disk.mount_point().to_string_lossy().to_string(),
            bytes_read: disk.usage().read_bytes,
            bytes_written: disk.usage().written_bytes,
        }
    }
}

/// Information collected about the mass storage on the host machine.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DiskInfo {
    pub disks: Vec<Storage>,
}
