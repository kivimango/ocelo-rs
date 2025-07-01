pub mod model;

pub use self::model::{CpuInfo, SystemInfo};
use model::{
    CpuCore, CpuMemoryUpdate, DiskInfo, MemoryInfo, NetworkInfo, Storage, SystemOverviewInfo,
};
use std::sync::{Arc, Mutex};
use sysinfo::{
    Components, CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind,
    System,
};

#[derive(Default, Clone, Copy)]
pub enum SystemInfoPollingContext {
    #[default]
    Overview,
    CpuAndMemory,
    Processes,
    Disks,
    Network,
}

pub type SharedSystemInfoPoller = Arc<Mutex<SystemInfoPoller>>;

pub enum SystemInfoUpdate {
    OverView(SystemOverviewInfo),
    CpuAndMemory(CpuMemoryUpdate),
    Process,
    Disk,
    Network,
}

impl From<(&SystemInfoPollingContext, &mut SystemInfoPoller)> for SystemInfoUpdate {
    fn from(value: (&SystemInfoPollingContext, &mut SystemInfoPoller)) -> Self {
        let (ctx, sysinfo) = value;
        match ctx {
            SystemInfoPollingContext::CpuAndMemory => {
                Self::CpuAndMemory(sysinfo.get_cpu_amd_memory_info())
            }
            SystemInfoPollingContext::Disks => Self::Disk,
            SystemInfoPollingContext::Network => Self::Network,
            SystemInfoPollingContext::Overview => Self::OverView(sysinfo.get_system_overview()),
            SystemInfoPollingContext::Processes => Self::Process,
        }
    }
}

pub struct SystemInfoPoller {
    polling_context: SystemInfoPollingContext,
    inner: System,
    disks: Disks,
    networks: Networks,
}

impl Default for SystemInfoPoller {
    /// Creates a new instance of `SystemInfoPoller`.
    /// After creating, it must be initialized to fetch the first chunk of system information
    /// by calling `self.init()`.
    fn default() -> Self {
        SystemInfoPoller {
            polling_context: SystemInfoPollingContext::Overview,
            inner: System::new(),
            disks: Disks::new(),
            networks: Networks::new(),
        }
    }
}

impl SystemInfoPoller {
    /// Initalizes the backing system info fetcher by refreshing all ystem information.
    pub fn init(&mut self) {
        self.inner.refresh_all();
    }

    pub fn get_cpu_info(&mut self) -> CpuInfo {
        self.inner.refresh_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::nothing().with_cpu_usage().with_frequency()),
        );

        let cpus = self.inner.cpus();
        let name = cpus
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let core_count = cpus.len();
        let average_frequency = if core_count > 0 {
            cpus.iter().map(|c| c.frequency()).sum::<u64>() / core_count as u64
        } else {
            0
        };
        let usage = self.inner.global_cpu_usage();

        CpuInfo {
            name,
            frequency: average_frequency,
            core_count,
            temperature: None,
            usage,
        }
    }

    fn get_cpu_amd_memory_info(&mut self) -> CpuMemoryUpdate {
        self.inner.refresh_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::nothing().with_cpu_usage().with_frequency())
                .with_memory(MemoryRefreshKind::everything()),
        );

        let sum_freq = self
            .inner
            .cpus()
            .iter()
            .map(|core| core.frequency())
            .sum::<u64>() as usize;
        let avg_freq = sum_freq / self.inner.cpus().len();

        let _ = Components::new_with_refreshed_list()
            .iter()
            .map(|f| print!("{}", f.temperature().unwrap()));

        CpuMemoryUpdate {
            usage: self.inner.global_cpu_usage(),
            frequency: avg_freq,
            temperature: 0,
            cores: self
                .inner
                .cpus()
                .iter()
                .map(|cpu| CpuCore {
                    frequency: cpu.frequency(),
                    temperature: 0,
                    usage: cpu.cpu_usage() as u64,
                })
                .collect(),
            memory_stats: self.get_memory_info(),
        }
    }

    fn get_disk_info(&mut self) -> DiskInfo {
        self.disks.refresh_specifics(
            true,
            DiskRefreshKind::nothing().with_io_usage().with_storage(),
        );

        let mut disks = self
            .disks
            .iter()
            .map(Storage::from)
            .collect::<Vec<Storage>>();
        disks.sort_by_key(|d| d.used_space);
        disks.reverse();

        DiskInfo { disks }
    }

    fn get_memory_info(&mut self) -> MemoryInfo {
        self.inner.refresh_memory();

        let total = self.inner.total_memory();
        let used = self.inner.used_memory();
        let available = self.inner.available_memory();
        let swap_total = self.inner.total_swap();
        let swap_used = self.inner.used_swap();
        let swap_available = self.inner.free_swap();

        MemoryInfo {
            total,
            used,
            available,
            swap_total,
            swap_used,
            swap_available,
        }
    }

    fn get_network_info(&mut self) -> NetworkInfo {
        self.networks.refresh(true);
        NetworkInfo::from(&self.networks)
    }

    fn get_system_info(&mut self) -> SystemInfo {
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

    pub fn get_system_overview(&mut self) -> SystemOverviewInfo {
        SystemOverviewInfo {
            cpu: self.get_cpu_info(),
            overview: self.get_system_info(),
            memory: self.get_memory_info(),
            disks: self.get_disk_info(),
            network: self.get_network_info(),
        }
    }

    pub fn polling_context(&self) -> SystemInfoPollingContext {
        self.polling_context
    }

    pub fn set_polling_context(&mut self, new_ctx: SystemInfoPollingContext) {
        self.polling_context = new_ctx;
    }
}
