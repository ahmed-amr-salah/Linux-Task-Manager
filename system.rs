use crate::process::Process;
use sysinfo::ProcessExt;
use sysinfo::SystemExt;
use sysinfo::System as Sys;
use sysinfo::{Pid, /*PidExt*/CpuExt};
//use sysinfo::Uid;
//use sysinfo::User;
//use sysinfo::UserExt;
pub struct System {
    sysinfo: Sys,
    pub cpu_current_usage: f32,
    pub cpu_num_cores: usize,
    pub mem_total: u64,
    pub mem_free: u64,
    pub disk_used:u64,
    pub disk_total:u64,
    pub mem_used: u64,
    pub cpu_core_usages: Vec<f32>,
    pub processes: Vec<Process>,
    // pub user_list: Vec<User>,
}

impl System {
    pub fn new() -> System {
        let sysinfo = Sys::new_all();
        let cpu_num_cores: usize = sysinfo.cpus().len() - 1;
        let mem_total = sysinfo.total_memory();
        //let disk_total = sysinfo.get_total_space();
        System {
            sysinfo,
            cpu_current_usage: 0.0,
            cpu_num_cores: 0,
            mem_total,
            mem_free: 0,
            disk_used:0,
            disk_total:0,
            mem_used:0,
            cpu_core_usages: vec![],
            processes: vec![],
            // user_list: vec![],
        }
    }

    pub fn update(&mut self) -> System {
        self.sysinfo.refresh_all();

        // Overall CPU usage
        self.cpu_current_usage = self.sysinfo.global_cpu_info().cpu_usage();

        // Memory usage
        self.mem_used = self.sysinfo.used_memory();
        self.mem_free = self.sysinfo.free_memory();
        

        // CPU core usage
        self.cpu_core_usages = self.sysinfo.cpus()
            .iter()
            .skip(1)
            .map(|p| p.cpu_usage())
            .collect();

        // Processes
        self.processes = self.sysinfo.processes()
            .iter()
            .map(|(_, process)|
                Process::new(process)
            )
            .collect();
        // self.user_list = self.sysinfo.users().iter();
        System {
            sysinfo: Sys::new(),
            cpu_core_usages: self.cpu_core_usages.clone(),
            processes: self.processes.clone(),
            // user_list: self.user_list.clone(),
            ..*self
        }
    }

    pub fn kill_process(&mut self, pid: Pid) {
        if let Some(process) = self.sysinfo.process(pid) {
            process.kill();
        }
    }

}