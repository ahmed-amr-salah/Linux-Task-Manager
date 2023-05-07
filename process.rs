use sysinfo::Process as SysProcess;
use sysinfo::{ProcessExt,Pid, PidExt,SystemExt,System};
use procfs::process::Process as ProcfsProcess;
use psutil::process::Process as PsutilProcess;
use sysinfo::Uid;
use sysinfo::User;
use sysinfo::UserExt;
use users::get_current_uid;
use users::get_user_by_uid;
use sysinfo::DiskUsage;
#[derive(PartialEq, Clone, Debug)]
pub struct Process {
    pub pid: u32,
    pub name: String,
    pub cpu: String,
    pub mem: u64,
    pub state: String,
    pub command: String,
    pub ppid: Option<Pid>,
    pub user: String,
    pub piddd: Pid,
    pub fd_count: usize,
    pub threads: u64,
    pub process_priority: i64,
    pub p_u_id: String,
    pub virtual_mem: u64,
    pub disk_written: u64,
    pub disk_read: u64,
    pub mem_percentage: String,
}

impl Process {
    pub fn new(process: &SysProcess, system_try: &System) -> Process {
        //let mut sys = System::new_all();
        let mut p = Process{
                    pid: process.pid().as_u32(),
                    name: process.name().to_string(),
                    cpu: process.cpu_usage().to_string(),
                    mem: process.memory(),
                    command: String::new(),
                    ppid: process.parent(),
                    piddd: process.pid(),
                    p_u_id: String::new(),
                    user: system_try.get_user_by_id(process.user_id().unwrap()).unwrap().name().to_string(),
                    state: String::new(),
                    fd_count: 0,
                    threads: 0,
                    process_priority: 0,
                    virtual_mem: 0,
                    disk_read: 0,
                    disk_written: 0,
                    mem_percentage: (((process.memory() as f32 / system_try.total_memory() as f32) * 100.0)).to_string(),
        };
        //users::get_user_by_uid(users::get_current_uid()).unwrap().name().to_str().unwrap().to_string() == String::from(:root)
        //let mut user_copy = String::new();
        for prc in procfs::process::all_processes().unwrap()
        {
            if prc.as_ref().expect("Error!").pid().to_string() == process.pid().as_u32().to_string()
            {
                let mut process_uid = process.user_id().unwrap().to_string();
                let mut open_files = 0;
                if users::get_user_by_uid(users::get_current_uid()).unwrap().name().to_str().unwrap().to_string()== system_try.get_user_by_id(&*process.user_id().unwrap()).unwrap().name().to_string() || users::get_user_by_uid(users::get_current_uid()).unwrap().name().to_str().unwrap().to_string() == String::from("root")
                {
                    open_files = ProcfsProcess::new(process.pid().as_u32() as i32).as_ref().expect("Error!").fd_count().unwrap();
                }
                p = Process
                {
                    pid: process.pid().as_u32(),
                    name: process.name().to_string(),
                    cpu: process.cpu_usage().to_string(),
                    mem: process.memory(),
                    command:ProcfsProcess::new(process.pid().as_u32() as i32).as_ref().expect("Error!").stat().unwrap().comm,
                    ppid: process.parent(),
                    piddd: process.pid(),
                    state: ProcfsProcess::new(process.pid().as_u32() as i32).as_ref().expect("Error!").status().unwrap().state,
                    fd_count: open_files,
                    threads: ProcfsProcess::new(process.pid().as_u32() as i32).as_ref().expect("Error!").status().unwrap().threads,
                    process_priority: ProcfsProcess::new(process.pid().as_u32() as i32).as_ref().expect("Error!").stat().unwrap().priority,
                    p_u_id: process_uid,
                    user: system_try.get_user_by_id(&*process.user_id().unwrap()).unwrap().name().to_string(),
                    virtual_mem: process.virtual_memory(),
                    disk_read: process.disk_usage().total_read_bytes,
                    disk_written: process.disk_usage().total_written_bytes,
                    mem_percentage: (((process.memory() as f32 / system_try.total_memory() as f32) * 100.0)).to_string(),
                };
            }
        }
        return p;
    }
    
    // pub fn format(&self) -> Vec<String> {
    //     let ppid_str = self.ppid.map_or(String::new(), |p| p.to_string());
    //     vec![
    //         self.pid.to_string(),
    //         self.name.clone(),
    //         self.cpu.clone(),
    //         pretty_bytes::converter::convert((self.mem as f64) * 1000.0),
    //         self.state.clone(),
    //         self.command.clone(),
    //         ppid_str,
    //         self.user.clone(),
    //         self.piddd.to_string(),
    //         self.state.clone(),
    //         self.fd_count.to_string(),
    //         self.threads.to_string(),
    //         self.p_u_id.to_string(),
    //         self.process_priority.to_string(),
    //     ]
    // }
}