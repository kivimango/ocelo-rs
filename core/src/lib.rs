pub mod model;

pub use self::model::{CpuInfo, SystemInfo};
use sysinfo::System;

pub fn get_cpu_info() -> CpuInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpus = sys.cpus();
    let name = cpus
        .get(0)
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let core_count = cpus.len();
    let average_frequency = if core_count > 0 {
        cpus.iter().map(|c| c.frequency()).sum::<u64>() / core_count as u64
    } else {
        0
    };

    //let temperature = cpus.get(0).and_then(|c| c.temperature()); // Option<f32>

    CpuInfo {
        name,
        frequency: average_frequency,
        core_count,
        temperature: None,
    }
}

pub fn get_system_info() -> SystemInfo {
    let host_name = System::host_name().unwrap_or_else(|| "N/A".to_string());
    let uptime = System::uptime();
    let kernel_version = System::kernel_long_version();
    let load_avg = System::load_average();

    SystemInfo {
        host_name,
        kernel_version,
        uptime,
        load_one_minute: load_avg.one,
        load_five_minutes: load_avg.five,
        load_fifteen_minutes: load_avg.fifteen,
    }
}
