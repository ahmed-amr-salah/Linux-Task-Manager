use crate::process::Process;
use sysinfo::ProcessExt;
use sysinfo::SystemExt;
use sysinfo::System as Sys;
use sysinfo::{Pid, PidExt,CpuExt};
use sysinfo::Uid;
use sysinfo::User;
use sysinfo::UserExt;
pub struct System {
    sysinfo: Sys,
    pub cpu_current_usage: f32,
    pub cpu_num_cores: usize,
    pub mem_total: u64,
    pub mem_free: u64,
    pub mem_used: u64,
    pub mem_available: u64,
    pub total_swap: u64,
    pub free_swap: u64,
    pub used_swap: u64,
    pub disks_num: usize,
    pub load_avg: f64,
    pub cpu_core_usages: Vec<u16>,
    pub processes: Vec<Process>,
}

impl System {
    pub fn new() -> System {
        let sysinfo = Sys::new_all();
        let cpu_num_cores: usize = sysinfo.cpus().len() - 1;
        let mem_total = sysinfo.total_memory();
        System {
            sysinfo,
            cpu_current_usage: 0.0,
            cpu_num_cores,
            mem_total,
            mem_free: 0,
            mem_used: 0,
            mem_available :0,
            total_swap: 0,
            free_swap: 0,
            used_swap: 0,
            disks_num: 0,
            load_avg: 0.0,
            cpu_core_usages: vec![],
            processes: vec![],
        }
    }

    pub fn update(&mut self) -> System {
        self.sysinfo.refresh_all();

        // Overall CPU usage
        self.cpu_current_usage = self.sysinfo.global_cpu_info().cpu_usage();

        // Memory usage
        self.mem_used = self.sysinfo.used_memory();
        self.mem_free = self.sysinfo.free_memory();
        self.mem_available = self.sysinfo.available_memory();
        self.total_swap = self.sysinfo.total_swap();
        self.free_swap = self.sysinfo.free_swap();
        self.used_swap = self.sysinfo.used_swap();
        self.disks_num = self.sysinfo.disks().len();
        self.load_avg = self.sysinfo.load_average().one;
        // CPU core usage
        self.cpu_core_usages = self.sysinfo.cpus()
            .iter()
            .skip(1)
            .map(|p| (p.cpu_usage() * 100.0).round() as u16)
            .collect();

        // Processes
        self.processes = self.sysinfo.processes()
            .iter()
            .map(|(_, process)|
                Process::new(process,&self.sysinfo)
            )
            .collect();
        System {
            sysinfo: Sys::new(),
            cpu_core_usages: self.cpu_core_usages.clone(),
            processes: self.processes.clone(),
            ..*self
        }
    }
}