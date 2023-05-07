mod process;
mod system;
extern crate cursive;
extern crate cursive_table_view;
use std::cmp::Ordering;
use std::process::Command;
use sysinfo::{ProcessExt,Pid, PidExt,SystemExt,CpuExt};
use sysinfo::System as SysInfoSystem;
use sysinfo::Process as SysProcess;
use std::thread::sleep;
use std::time::{Duration, Instant};
use prettytable::{Table,Row,Cell};
use crate::system::System as LocalSystem;
use crate::process::Process;
use cursive::views::{Dialog, DummyView, EditView, LinearLayout, ScrollView, TextView, TextArea, Button, FixedLayout};
use cursive::traits::*;
use cursive::Cursive;
use cursive_table_view::{TableView, TableViewItem};
use cursive_core::Rect;
use fltk::button::Button as FltkButton;
use fltk::{app, frame::Frame as FltkFrame, prelude::*, window::Window as FltkWindow, group::Group as FltkGroup,input::Input};
// use fltk::enums::*;
// use std::cell::RefCell;
// use std::rc::Rc;
use fltk::widget_extends;
// use fltk::draw;
// use fltk::group::Flex as GroupFlex;
// use fltk_extras::dial::*;
// use crossterm::terminal;
// use crossterm::terminal::Clear;
// use crossterm::terminal::ClearType;
use cursive::theme::{BaseColor::*, Color::*, PaletteColor::*};
use std::io::{self, Write, stdout};
use std::str::FromStr;
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    Pid,
    Name,
    Cpu,
    Mem,
    Status,
    Cmd,
    Ppid,
    User,
    Priority,
}

impl BasicColumn {
    fn as_str(&self) -> &str {
        match *self {
            BasicColumn::Pid => "PID",
            BasicColumn::Name => "Name",
            BasicColumn::Cpu => "CPU%",
            BasicColumn::Mem => "MEM%",
            BasicColumn::Status => "S",
            BasicColumn::Cmd => "CMD",
            BasicColumn::Ppid => "PPID",
            BasicColumn::User => "User",
            BasicColumn::Priority => "Priority",
        }
    }
}

impl TableViewItem<BasicColumn> for Process {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Name => self.name.to_string(),
            BasicColumn::Pid => format!("{}", self.pid),
            BasicColumn::Cpu => format!("{}%", self.cpu),
            BasicColumn::Cmd => format!("{}", self.command),
            BasicColumn::Mem => format!("{}%", self.mem_percentage),
            BasicColumn::Status => format!("{}", self.state),
            BasicColumn::Ppid => format!("{}", self.ppid.map_or(String::new(), |p| p.to_string())),
            BasicColumn::User => format!("{}", self.user),
            BasicColumn::Priority => format!("{}", self.process_priority.to_string()),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            BasicColumn::Name => self.name.cmp(&other.name),
            BasicColumn::Pid => self.pid.cmp(&other.pid),
            BasicColumn::Cpu => self.cpu.cmp(&other.cpu),
            BasicColumn::Mem => self.mem_percentage.cmp(&other.mem_percentage),
            BasicColumn::Status => self.state.cmp(&other.state),
            BasicColumn::Cmd => self.command.cmp(&other.command),
            BasicColumn::Ppid => self.ppid.cmp(&other.ppid),
            BasicColumn::User => self.user.cmp(&other.user),
            BasicColumn::Priority => (self.process_priority*-1).cmp(&(other.process_priority*-1)),
        }
    }
}
fn getmoreinfo(s:&usize,t:&mut TableView::<Process, BasicColumn>) -> String
{
    let process_info = t.borrow_item(*s).unwrap();
    let pid = process_info.pid;
    return pid.to_string();
}

enum search{
    PiD,
    User,
    PUid,
    PpPid,
    Name,
}

enum filter{
    Cpu,
    Mem,
    Threads,
    FdCount,  
}

enum options{
    Greater,
    Less,
    Range,
}

fn kill_process(s: &mut Cursive,pid: Pid) {
    let system = SysInfoSystem::new_all();
    if let Some(process) = system.process(pid) {
        if process.kill() {
            s.add_layer(Dialog::around(
                LinearLayout::vertical()
                    .child(TextView::new("Process killed successfully."),
                ))
                .button("Close", move |s| {
                    s.pop_layer();
                }),
        );
        }
        else{
            s.add_layer(Dialog::around(
                LinearLayout::vertical()
                    .child(TextView::new("Process couldn't be killed or Process not found! Please enter correct pid."),
                ))
                .button("Close", move |s| {
                    s.pop_layer();
                }),
        );
    }
}
    else{
        s.add_layer(Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Process not found! Please enetr correct pid."),
            ))
            .button("Close", move |s| {
                s.pop_layer();
            }),
    );
    }
}

fn show_popup(s: &mut Cursive, name: &str) {
    let mut command: &str = " ";
    let mut first: &str = " ";
    let mut second: &str = " ";
    let mut third: &str = " ";
    let mut fourth: &str = " ";

    if name.is_empty() {
        s.add_layer(Dialog::info("Please enter a command! For help, enter show -help"));
    } else {
        let args: Vec<&str> = name.split_whitespace().collect();
        if args.len() == 3 {
        command = args[0];
        first = args[1];
        second = args[2];
        third = " ";
        fourth = " ";
        }
        else if args.len() == 2 {
            command = args[0];
            first = args[1];
            second = " ";
        }
        else if args.len() == 4
        {
            command = args[0];
            first = args[1];
            second = args[2];
            third = args[3];
            fourth = " ";
        }
        else if args.len() == 5
        {
            command = args[0];
            first = args[1];
            second = args[2];
            third = args[3];
            fourth = args[4];
        }
        else {
            let help = "Incorrect command. Please enter one of the following available commands:".to_owned() +  &"\n".to_owned()
            + &"To show full table, enter: show full table".to_owned()
            + &"To kill a process, enter: kill -pid [process pid]".to_owned()
            + &"\n".to_owned() + &"Options to search for a process: ".to_owned() + &"\n".to_owned()
            + &"1- search -pid [process pid]".to_owned() + &"\n".to_owned() + &"2- search -uid [process uid]".to_owned()
            + &"\n".to_owned() + &"3- search -user [user]".to_owned() + &"\n".to_owned() + &"4- search -name [process name]".to_owned() + &"\n".to_owned() + &"5- search -ppid [process parent id]".to_owned()
            + &"\n".to_owned() + &"Options to filter processes: ".to_owned() + &"\n".to_owned()
            + &"1- filter -cpu -[greater, less] [value]".to_owned() + &"\n".to_owned() + &"2- filter -cpu -range [minimum value] [maximum value]".to_owned()
            + &"\n".to_owned() + &"3- filter -mem -[greater, less] [value]".to_owned() + &"\n".to_owned() + &"4- filter -mem -range [minimum value] [maximum value]".to_owned()
            + &"\n".to_owned() + &"5- filter -threads -[greater, less] [value]".to_owned() + &"\n".to_owned() + &"6- filter -threads -range [minimum value] [maximum value]".to_owned()
            + &"\n".to_owned() + &"7- filter -fd -[greater, less] [value]".to_owned() + &"\n".to_owned() + &"8- filter -fd -range [minimum value] [maximum value]".to_owned();
            s.add_layer(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(TextView::new(&help),
                    ))
                    .button("Close", move |s| {
                        s.pop_layer();
                    }),
            );
            command = " ";
            first = " ";
            second = " ";
        }
    }
        if name == "show full table"{
            show_complete_view();
        }
        else if command == "search" && first == "-user" {
            show_search_view(search::User, (&second).to_string());
        }
        else if command == "search" && first == "-uid" {
            show_search_view(search::PUid, (&second).to_string());
        }
        else if command == "search" && first == "-name" {
            show_search_view(search::Name, (&second).to_string());
        }
        else if command == "search" && first == "-pid" {
            show_search_view(search::PiD, (&second).to_string());
        }
        else if command == "search" && first == "-ppid" {
            show_search_view(search::PpPid, (&second).to_string());
        }
        else if command == "kill" && first == "-pid" {
            let mut system = SysInfoSystem::new_all();
            system.refresh_all();
            let processes_list: Vec<Process> = system.processes().iter()
            .map(|(_, process)|
                Process::new(process,&system)
            )
            .collect();
            for prc in processes_list{
                if prc.pid.to_string() == (&second).to_string() {
                    kill_process(s,prc.piddd);
                }
            }
        }
        else if command == "filter" && first == "-greater" && second == "-cpu" {
            show_filtered_view(filter::Cpu,options::Greater,(&third).to_string(),(&fourth).to_string());
        }
        else if command == "filter" && first == "-less" && second == "-cpu" {
            show_filtered_view(filter::Cpu,options::Less,(&third).to_string(),(&fourth).to_string());
        }
        else if command == "filter" && first == "-range" && second == "-cpu" {
            show_filtered_view(filter::Cpu,options::Range,(&third).to_string(),(&fourth).to_string());
        }
        else if command == "filter" && first == "-greater" && second == "-mem" {
            show_filtered_view(filter::Mem,options::Greater,(&third).to_string(),(&fourth).to_string());
        }
        else if command == "filter" && first == "-less" && second == "-mem" {
            show_filtered_view(filter::Mem,options::Less,(&third).to_string(),(&fourth).to_string());
        }
        else if command == "filter" && first == "-range" && second == "-mem" {
            show_filtered_view(filter::Mem,options::Range,(&third).to_string(),(&fourth).to_string());
        }
        else if command == "filter" && first == "-greater" && second == "-threads" {
            show_filtered_view(filter::Threads,options::Greater,(&third).to_string(),(&fourth).to_string());
        }
        else if command == "filter" && first == "-less" && second == "-threads" {
            show_filtered_view(filter::Threads,options::Less,(&third).to_string(),(&fourth).to_string());
        }
        else if command == "filter" && first == "-range" && second == "-threads" {
            show_filtered_view(filter::Threads,options::Range,(&third).to_string(),(&fourth).to_string());
        }
        else if command == "filter" && first == "-greater" && second == "-fd" {
            show_filtered_view(filter::FdCount,options::Greater,(&third).to_string(),(&fourth).to_string());
        }
        else if command == "filter" && first == "-less" && second == "-fd" {
            show_filtered_view(filter::FdCount,options::Less,(&third).to_string(),(&fourth).to_string());
        }
        else if command == "filter" && first == "-range" && second == "-fd" {
            show_filtered_view(filter::FdCount,options::Range,(&third).to_string(),(&fourth).to_string());
        }
        else if command == "show" && first == "-help" {
            let help = "Incorrect command. Please enter one of the following available commands:".to_owned() +  &"\n".to_owned()
            + &"To show full table, enter: show full table".to_owned()
            + &"To kill a process, enter: kill -pid [process pid]".to_owned()
            + &"\n".to_owned() + &"Options to search for a process: ".to_owned() + &"\n".to_owned()
            + &"1- search -pid [process pid]".to_owned() + &"\n".to_owned() + &"2- search -uid [process uid]".to_owned()
            + &"\n".to_owned() + &"3- search -user [user]".to_owned() + &"\n".to_owned() + &"4- search -name [process name]".to_owned() + &"\n".to_owned() + &"5- search -ppid [process parent id]".to_owned()
            + &"\n".to_owned() + &"Options to filter processes: ".to_owned() + &"\n".to_owned()
            + &"1- filter -cpu -[greater, less] [value]".to_owned() + &"\n".to_owned() + &"2- filter -cpu -range [minimum value] [maximum value]".to_owned()
            + &"\n".to_owned() + &"3- filter -mem -[greater, less] [value]".to_owned() + &"\n".to_owned() + &"4- filter -mem -range [minimum value] [maximum value]".to_owned()
            + &"\n".to_owned() + &"5- filter -threads -[greater, less] [value]".to_owned() + &"\n".to_owned() + &"6- filter -threads -range [minimum value] [maximum value]".to_owned()
            + &"\n".to_owned() + &"7- filter -fd -[greater, less] [value]".to_owned() + &"\n".to_owned() + &"8- filter -fd -range [minimum value] [maximum value]".to_owned();

            s.add_layer(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(TextView::new(&help),
                    ))
                    .button("Close", move |s| {
                        s.pop_layer();
                    }),
            );
        }
        else {
            let help = "Incorrect command. Please enter one of the following available commands:".to_owned() +  &"\n".to_owned()
            + &"To show full table, enter: show full table".to_owned()
            + &"To kill a process, enter: kill -pid [process pid]".to_owned()
            + &"\n".to_owned() + &"Options to search for a process: ".to_owned() + &"\n".to_owned()
            + &"1- search -pid [process pid]".to_owned() + &"\n".to_owned() + &"2- search -uid [process uid]".to_owned()
            + &"\n".to_owned() + &"3- search -user [user]".to_owned() + &"\n".to_owned() + &"4- search -name [process name]".to_owned() + &"\n".to_owned() + &"5- search -ppid [process parent id]".to_owned()
            + &"\n".to_owned() + &"Options to filter processes: ".to_owned() + &"\n".to_owned()
            + &"1- filter -cpu -[greater, less] [value]".to_owned() + &"\n".to_owned() + &"2- filter -cpu -range [minimum value] [maximum value]".to_owned()
            + &"\n".to_owned() + &"3- filter -mem -[greater, less] [value]".to_owned() + &"\n".to_owned() + &"4- filter -mem -range [minimum value] [maximum value]".to_owned()
            + &"\n".to_owned() + &"5- filter -threads -[greater, less] [value]".to_owned() + &"\n".to_owned() + &"6- filter -threads -range [minimum value] [maximum value]".to_owned()
            + &"\n".to_owned() + &"7- filter -fd -[greater, less] [value]".to_owned() + &"\n".to_owned() + &"8- filter -fd -range [minimum value] [maximum value]".to_owned();
            s.add_layer(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(TextView::new(&help),
                    ))
                    .button("Close", move |s| {
                        s.pop_layer();
                    }),
            );
        }
}

fn show_complete_view(){
    let mut palette = cursive::theme::Palette::default();
    palette[Background] =Dark(Black);
    palette[Shadow] = Dark(Black); 
    palette[View] = Dark(Black);
    palette[Primary] = Light(Green);
    palette[Secondary] = Light(Green);
    palette[Tertiary] = Light(Green);
    palette[TitlePrimary] = Light(Green);
    palette[TitleSecondary] = Light(Green);
    palette[Highlight] = Dark(White);
    palette[HighlightInactive] = Dark(White);
    palette[HighlightText] = Dark(Green);   
    let theme = cursive::theme::Theme{
        shadow: true,
        borders: cursive::theme::BorderStyle::None,
        palette: palette,
    };
        let mut system = SysInfoSystem::new_all();
        system.refresh_all();
        let mut fill_siv = cursive::default();
        let mut table = TableView::<Process, BasicColumn>::new()
            .column(BasicColumn::Name, "Name", |c| c.width_percent(10))
            .column(BasicColumn::Pid, "PID", |c| c.width_percent(4))
            .column(BasicColumn::Cpu, "CPU%", |c| c.width_percent(7))
            .column(BasicColumn::Cmd, "CMD", |c| c.width_percent(20))
            .column(BasicColumn::Mem, "MEM%", |c| c.width_percent(10))
            .column(BasicColumn::Status, "S", |c| c.width_percent(10))
            .column(BasicColumn::Ppid, "PPID", |c| c.width_percent(7))
            // .column(BasicColumn::Uid, "UID", |c| c.width_percent(5))
            .column(BasicColumn::User, "User", |c| c.width_percent(10))
            .column(BasicColumn::Priority, "Prioirty", |c| c.width_percent(23));
            // .column(BasicColumn::FdCount, "FdCount", |c| c.width_percent(5))
            // .column(BasicColumn::Threads, "Threads", |c| c.width_percent(5));
        let mut items = Vec::new();
        items = system.processes()
        .iter()
        .map(|(_, process)|
            Process::new(process,&system)
        )
        .collect();
        table.set_on_sort(|fill_siv: &mut Cursive, column: BasicColumn, order: Ordering| {
            fill_siv.add_layer(
                Dialog::around(TextView::new(format!("{} / {:?}", column.as_str(), order)))
                    .title("Sorted by")
                    .button("Close", |s| {
                        s.pop_layer();
                    }),
            );
        });
    
        table.set_on_submit(|siv: &mut Cursive, row: usize, index: usize| {
            let mut processes_info = String::new();
            let value = siv
                .call_on_name("table",|table: &mut TableView<Process, BasicColumn>| {
                    table.borrow_item(index);
                    let pid_value = &getmoreinfo(&index,table);
                    let mut system_extra = SysInfoSystem::new_all();
                    system_extra.refresh_all();
                    let mut processes_list: Vec<Process>;
                    processes_list = system_extra.processes()
                    .iter()
                    .map(|(_, process)|
                        Process::new(process,&system_extra)
                    )
                    .collect();

                    for prc in &processes_list{
                        if prc.pid.to_string() == pid_value.to_string() {
                            processes_info = "Pid:".to_owned()+&prc.pid.to_string()+&"    Uid:".to_owned()+&prc.p_u_id.to_string()+&"    User: ".to_owned()+&prc.user.to_string() + &"    Num of. Threads: ".to_owned()+&prc.threads.to_string() + &"\n".to_owned()
                            + &"Num of. open files (fd): ".to_owned()+&prc.fd_count.to_string()+ &"     Priority: ".to_owned()+&prc.process_priority.to_string() + &"\n".to_owned()
                            + &"Disk Writes (Bytes): ".to_owned()+&prc.disk_written.to_string() + &"    Disk Reads (Bytes): ".to_owned()+&prc.disk_read.to_string() +&"     Memory (RAM) used (Bytes): ".to_owned()+&prc.mem.to_string() 
                        }
                    }
                })
                .unwrap();
            
            siv.add_layer(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(TextView::new(&processes_info),
                    ))
                    .button("Close", move |s| {
                        s.pop_layer();
                    }),
            );
        });

        fill_siv.add_layer(
            Dialog::around(
                LinearLayout::vertical()
                    .child(TextView::new(" ").with_name("System Information"))
                    .child(DummyView.fixed_height(2))
                    .child(table.with_name("table").min_size((200, 30)))
                    .child(DummyView.fixed_height(2))
                    .child(EditView::new()
                    .on_submit_mut(show_popup)
                    .with_name("name")
                    .fixed_width(200)),
            ));
            let duration = std::time::Duration::from_millis(1000);
            let cb_sink = fill_siv.cb_sink().clone();
            let handle = std::thread::spawn(move || {
            let mut updated_sys = SysInfoSystem::new_all();
            let vector_process = updated_sys.processes();
            let mut system_info = LocalSystem::new();
            let mut sys_info_str = String::new();
                loop{
                        // system.update();
                        // let mut items2 = system.processes.clone();
                        system_info.update();
                        updated_sys.refresh_all();
                        let vector_process = updated_sys.processes()
                        .iter()
                        .map(|(_, process)|
                            Process::new(process,&updated_sys)
                        )
                        .collect();
                        sys_info_str = "Processes: ".to_owned()+&system_info.processes.len().to_string()+ &"    Total Memory:".to_owned()+&system_info.mem_total.to_string()+&"    Used Memory:".to_owned()+&system_info.mem_used.to_string()+&"    Available Memory:".to_owned()+&system_info.mem_available.to_string() + &"\n".to_owned()
                        + &"Processors: ".to_owned()+&system_info.cpu_num_cores.to_string() + &"    Disks: ".to_owned()+&system_info.disks_num.to_string() +&"    Total Swap: ".to_owned()+&system_info.total_swap.to_string() + &"   Free Swap: ".to_owned()+&system_info.free_swap.to_string() + &"     Used Swap: ".to_owned()+&system_info.used_swap.to_string()
                        + &"\n".to_owned() + &"Load Average : ".to_owned()+&system_info.load_avg.to_string();

                        std::thread::sleep(duration);
                        cb_sink
                            .send(Box::new(move |s| {
                                s.call_on_name("table", |v: &mut TableView::<Process, BasicColumn>| {
                                    v.set_items(vector_process)
                                });
                            }))
                            .unwrap();
                        cb_sink
                        .send(Box::new(move |s| {
                            s.call_on_name("System Information", |v: &mut TextView| {
                                v.set_content(sys_info_str)
                            });
                        }))
                        .unwrap();
                        }
                });
                fill_siv.set_theme(theme);
                fill_siv.run();
}

fn show_search_view(criteria: search, value: String)
{
    let mut palette = cursive::theme::Palette::default();
    palette[Background] =Dark(Black);
    palette[Shadow] = Dark(Black); 
    palette[View] = Dark(Black);
    palette[Primary] = Light(Green);
    palette[Secondary] = Light(Green);
    palette[Tertiary] = Light(Green);
    palette[TitlePrimary] = Light(Green);
    palette[TitleSecondary] = Light(Green);
    palette[Highlight] = Dark(White);
    palette[HighlightInactive] = Dark(White);
    palette[HighlightText] = Dark(Green);   
    let theme = cursive::theme::Theme{
        shadow: true,
        borders: cursive::theme::BorderStyle::None,
        palette: palette,
    };
        let mut system = SysInfoSystem::new_all();
        system.refresh_all();
        let mut search_siv = cursive::default();
        let mut table = TableView::<Process, BasicColumn>::new()
            .column(BasicColumn::Name, "Name", |c| c.width_percent(10))
            .column(BasicColumn::Pid, "PID", |c| c.width_percent(4))
            .column(BasicColumn::Cpu, "CPU%", |c| c.width_percent(7))
            .column(BasicColumn::Cmd, "CMD", |c| c.width_percent(20))
            .column(BasicColumn::Mem, "MEM%", |c| c.width_percent(10))
            .column(BasicColumn::Status, "S", |c| c.width_percent(10))
            .column(BasicColumn::Ppid, "PPID", |c| c.width_percent(7))
            .column(BasicColumn::User, "User", |c| c.width_percent(10))
            .column(BasicColumn::Priority, "Prioirty", |c| c.width_percent(23));
        let mut items = Vec::new();
        items = system.processes()
        .iter()
        .map(|(_, process)|
            Process::new(process,&system)
        )
        .collect();

        table.set_on_sort(|search_siv: &mut Cursive, column: BasicColumn, order: Ordering| {
            search_siv.add_layer(
                Dialog::around(TextView::new(format!("{} / {:?}", column.as_str(), order)))
                    .title("Sorted by")
                    .button("Close", |s| {
                        s.pop_layer();
                    }),
            );
        });
    
        table.set_on_submit(|siv: &mut Cursive, row: usize, index: usize| {
            let mut processes_info = String::new();
            let value = siv
                .call_on_name("table",|table: &mut TableView<Process, BasicColumn>| {
                    table.borrow_item(index);
                    let pid_value = &getmoreinfo(&index,table);
                    let mut system_extra = SysInfoSystem::new_all();
                    system_extra.refresh_all();
                    let processes_list: Vec<Process>;
                    processes_list = system_extra.processes()
                    .iter()
                    .map(|(_, process)|
                        Process::new(process,&system_extra)
                    )
                    .collect();

                    for prc in &processes_list{
                        if prc.pid.to_string() == pid_value.to_string() {
                            processes_info = "Pid:".to_owned()+&prc.pid.to_string()+&"    Uid:".to_owned()+&prc.p_u_id.to_string()+&"    User: ".to_owned()+&prc.user.to_string() + &"    Num of. Threads: ".to_owned()+&prc.threads.to_string() + &"\n".to_owned()
                            + &"Num of. open files (fd): ".to_owned()+&prc.fd_count.to_string()+ &"     Priority: ".to_owned()+&prc.process_priority.to_string() + &"\n".to_owned()
                            + &"Disk Writes (Bytes): ".to_owned()+&prc.disk_written.to_string() + &"    Disk Reads (Bytes): ".to_owned()+&prc.disk_read.to_string() +&"     Memory (RAM) used (Bytes): ".to_owned()+&prc.mem.to_string() 
                        }
                    }
                })
                .unwrap();
            
            siv.add_layer(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(TextView::new(&processes_info),
                    ))
                    .button("Close", move |s| {
                        s.pop_layer();
                    }),
            );
        });
    
        search_siv.add_layer(
            Dialog::around(
                LinearLayout::vertical()
                    .child(TextView::new(" ").with_name("System Information"))
                    .child(DummyView.fixed_height(2))
                    .child(table.with_name("table").min_size((200, 30)))
                    .child(DummyView.fixed_height(2))
                    .child(EditView::new()
                    .on_submit_mut(show_popup)
                    .with_name("name")
                    .fixed_width(200)),
            ));
            let duration = std::time::Duration::from_millis(1000);
            let cb_sink = search_siv.cb_sink().clone();
            let value2 = value;
            let handle = std::thread::spawn(move || {
            let mut updated_sys2 = SysInfoSystem::new_all();
            let mut vector_process2:Vec<Process>;
            let mut system_info = LocalSystem::new();
            let mut sys_info_str = String::new();
                loop{
                        updated_sys2.refresh_all();
                        system_info.update();
                        vector_process2 = updated_sys2.processes()
                        .iter()
                        .map(|(_, process)|
                            Process::new(process,&updated_sys2)
                        )
                        .collect();
                        sys_info_str = "Processes: ".to_owned()+&system_info.processes.len().to_string()+ &"    Total Memory:".to_owned()+&system_info.mem_total.to_string()+&"    Used Memory:".to_owned()+&system_info.mem_used.to_string()+&"    Available Memory:".to_owned()+&system_info.mem_available.to_string() + &"\n".to_owned()
                        + &"Processors: ".to_owned()+&system_info.cpu_num_cores.to_string() + &"    Disks: ".to_owned()+&system_info.disks_num.to_string() +&"    Total Swap: ".to_owned()+&system_info.total_swap.to_string() + &"   Free Swap: ".to_owned()+&system_info.free_swap.to_string() + &"     Used Swap: ".to_owned()+&system_info.used_swap.to_string()
                        + &"\n".to_owned() + &"Load Average : ".to_owned()+&system_info.load_avg.to_string();                        match criteria {
                            search::PiD => {
                                let n: u32 = FromStr::from_str(&value2).unwrap();
                                vector_process2 = vector_process2
                                .into_iter()
                                .filter(|r| r.pid == n)
                                .collect();
                            },
                            search::PUid => {
                                vector_process2 = vector_process2
                                .into_iter()
                                .filter(|r| r.p_u_id == value2.to_string())
                                .collect();
                            },
                            search::PpPid => {
                                vector_process2 = vector_process2
                                .into_iter()
                                .filter(|r| r.ppid.unwrap().to_string() == value2.to_string())
                                .collect();
                            },
                            search::User => {
                                vector_process2 = vector_process2
                                .into_iter()
                                .filter(|r| r.user.contains(&value2.to_string()))
                                .collect();
                            },
                            search::Name => {
                                vector_process2 = vector_process2
                                .into_iter()
                                .filter(|r| r.name.contains(&value2.to_string()))
                                .collect();
                            },
                        }
                        std::thread::sleep(duration);
                        cb_sink
                            .send(Box::new(move |s| {
                                s.call_on_name("table", |v: &mut TableView::<Process, BasicColumn>| {
                                    v.set_items(vector_process2)
                                });
                            }))
                            .unwrap();
                            cb_sink
                        .send(Box::new(move |s| {
                            s.call_on_name("System Information", |v: &mut TextView| {
                                v.set_content(sys_info_str)
                            });
                        }))
                        .unwrap();
                        }
                });
            search_siv.set_theme(theme);
            search_siv.run();
}

fn show_filtered_view(criteria: filter, option_criteria: options, value: String, value2: String)
{
    let mut palette = cursive::theme::Palette::default();
    palette[Background] =Dark(Black);
    palette[Shadow] = Dark(Black); 
    palette[View] = Dark(Black);
    palette[Primary] = Light(Green);
    palette[Secondary] = Light(Green);
    palette[Tertiary] = Light(Green);
    palette[TitlePrimary] = Light(Green);
    palette[TitleSecondary] = Light(Green);
    palette[Highlight] = Dark(White);
    palette[HighlightInactive] = Dark(White);
    palette[HighlightText] = Dark(Green);    
    let theme = cursive::theme::Theme{
        shadow: true,
        borders: cursive::theme::BorderStyle::None,
        palette: palette,
    };
        let mut system = SysInfoSystem::new_all();
        system.refresh_all();
        let mut search_siv = cursive::default();
        let mut table = TableView::<Process, BasicColumn>::new()
            .column(BasicColumn::Name, "Name", |c| c.width_percent(10))
            .column(BasicColumn::Pid, "PID", |c| c.width_percent(4))
            .column(BasicColumn::Cpu, "CPU%", |c| c.width_percent(7))
            .column(BasicColumn::Cmd, "CMD", |c| c.width_percent(20))
            .column(BasicColumn::Mem, "MEM%", |c| c.width_percent(10))
            .column(BasicColumn::Status, "S", |c| c.width_percent(10))
            .column(BasicColumn::Ppid, "PPID", |c| c.width_percent(7))
            // .column(BasicColumn::Uid, "UID", |c| c.width_percent(5))
            .column(BasicColumn::User, "User", |c| c.width_percent(10))
            .column(BasicColumn::Priority, "Prioirty", |c| c.width_percent(23));
            // .column(BasicColumn::FdCount, "FdCount", |c| c.width_percent(5))
            // .column(BasicColumn::Threads, "Threads", |c| c.width_percent(5));
        let mut items = Vec::new();
        items = system.processes()
        .iter()
        .map(|(_, process)|
            Process::new(process,&system)
        )
        .collect();

        table.set_on_sort(|search_siv: &mut Cursive, column: BasicColumn, order: Ordering| {
            search_siv.add_layer(
                Dialog::around(TextView::new(format!("{} / {:?}", column.as_str(), order)))
                    .title("Sorted by")
                    .button("Close", |s| {
                        s.pop_layer();
                    }),
            );
        });
    
        table.set_on_submit(|siv: &mut Cursive, row: usize, index: usize| {
            let mut processes_info = String::new();
            let value = siv
                .call_on_name("table",|table: &mut TableView<Process, BasicColumn>| {
                    table.borrow_item(index);
                    let pid_value = &getmoreinfo(&index,table);
                    let mut system_extra = SysInfoSystem::new_all();
                    system_extra.refresh_all();
                    let processes_list: Vec<Process>;
                    processes_list = system_extra.processes()
                    .iter()
                    .map(|(_, process)|
                        Process::new(process,&system_extra)
                    )
                    .collect();

                    for prc in &processes_list{
                        if prc.pid.to_string() == pid_value.to_string() {
                            processes_info = "Pid:".to_owned()+&prc.pid.to_string()+&"    Uid:".to_owned()+&prc.p_u_id.to_string()+&"    User: ".to_owned()+&prc.user.to_string() + &"    Num of. Threads: ".to_owned()+&prc.threads.to_string() + &"\n".to_owned()
                            + &"Num of. open files (fd): ".to_owned()+&prc.fd_count.to_string()+ &"     Priority: ".to_owned()+&prc.process_priority.to_string() + &"\n".to_owned()
                            + &"Disk Writes (Bytes): ".to_owned()+&prc.disk_written.to_string() + &"    Disk Reads (Bytes): ".to_owned()+&prc.disk_read.to_string() +&"     Memory (RAM) used (Bytes): ".to_owned()+&prc.mem.to_string() 
                        }
                    }
                })
                .unwrap();
            
            siv.add_layer(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(TextView::new(&processes_info),
                    ))
                    .button("Close", move |s| {
                        s.pop_layer();
                    }),
            );
        });
    
        search_siv.add_layer(
            Dialog::around(
                LinearLayout::vertical()
                    .child(TextView::new(" ").with_name("System Information"))
                    .child(DummyView.fixed_height(2))
                    .child(table.with_name("table").min_size((200, 30)))
                    .child(DummyView.fixed_height(2))
                    .child(EditView::new()
                    .on_submit_mut(show_popup)
                    .with_name("name")
                    .fixed_width(200)),
            ));
            let duration = std::time::Duration::from_millis(1000);
            let cb_sink = search_siv.cb_sink().clone();
            let value3 = value;
            let value4 = value2;
            let handle = std::thread::spawn(move || {
            let mut updated_sys2 = SysInfoSystem::new_all();
            let mut vector_process2:Vec<Process>;
            let mut system_info = LocalSystem::new();
            let mut sys_info_str = String::new();
                loop{
                        updated_sys2.refresh_all();
                        system_info.update();
                        vector_process2 = updated_sys2.processes()
                        .iter()
                        .map(|(_, process)|
                            Process::new(process,&updated_sys2)
                        )
                        .collect();
                        sys_info_str = "Processes: ".to_owned()+&system_info.processes.len().to_string()+ &"    Total Memory:".to_owned()+&system_info.mem_total.to_string()+&"    Used Memory:".to_owned()+&system_info.mem_used.to_string()+&"    Available Memory:".to_owned()+&system_info.mem_available.to_string() + &"\n".to_owned()
                        + &"Processors: ".to_owned()+&system_info.cpu_num_cores.to_string() + &"    Disks: ".to_owned()+&system_info.disks_num.to_string() +&"    Total Swap: ".to_owned()+&system_info.total_swap.to_string() + &"   Free Swap: ".to_owned()+&system_info.free_swap.to_string() + &"     Used Swap: ".to_owned()+&system_info.used_swap.to_string()
                        + &"\n".to_owned() + &"Load Average : ".to_owned()+&system_info.load_avg.to_string();
                        match criteria {
                            filter::Cpu => {
                                let n: f32 = FromStr::from_str(&value3).unwrap();
                                let n2: f32;
                                if value4 != " " {
                                    n2 = FromStr::from_str(&value4).unwrap();
                                }
                                else{
                                    n2 = 0.0;
                                }
                                match option_criteria{
                                    options::Greater =>
                                    {
                                        vector_process2 = vector_process2
                                        .into_iter()
                                        .filter(|r| r.cpu.parse::<f32>().unwrap() > n)
                                        .collect();
                                    }
                                    options::Less =>
                                    {
                                        vector_process2 = vector_process2
                                        .into_iter()
                                        .filter(|r| r.cpu.parse::<f32>().unwrap() < n)
                                        .collect();
                                    }
                                    options::Range =>
                                    {
                                        vector_process2 = vector_process2
                                        .into_iter()
                                        .filter(|r| r.cpu.parse::<f32>().unwrap() > n && r.cpu.parse::<f32>().unwrap() < n2)
                                        .collect();
                                    }
                                }
                            },
                            filter::Mem => {
                                let n: f32 = FromStr::from_str(&value3).unwrap();
                                let n2: f32;
                                if value4 != " " {
                                    n2 = FromStr::from_str(&value4).unwrap();
                                }
                                else{
                                    n2 = 0.0;
                                }
                                match option_criteria{
                                    options::Greater =>
                                    {
                                        vector_process2 = vector_process2
                                        .into_iter()
                                        .filter(|r| r.mem_percentage.parse::<f32>().unwrap() > n)
                                        .collect();
                                    }
                                    options::Less =>
                                    {
                                        vector_process2 = vector_process2
                                        .into_iter()
                                        .filter(|r| r.mem_percentage.parse::<f32>().unwrap() < n)
                                        .collect();
                                    }
                                    options::Range =>
                                    {
                                        vector_process2 = vector_process2
                                        .into_iter()
                                        .filter(|r| r.mem_percentage.parse::<f32>().unwrap() > n && r.mem_percentage.parse::<f32>().unwrap() < n2)
                                        .collect();
                                    }
                                }
                            },
                            filter::Threads => {
                                let n: u64 = FromStr::from_str(&value3).unwrap();
                                let n2: u64;
                                if value4 != " " {
                                    n2 = FromStr::from_str(&value4).unwrap();
                                }
                                else{
                                    n2 = 0;
                                }
                                match option_criteria{
                                    options::Greater =>
                                    {
                                        vector_process2 = vector_process2
                                        .into_iter()
                                        .filter(|r| r.threads > n)
                                        .collect();
                                    }
                                    options::Less =>
                                    {
                                        vector_process2 = vector_process2
                                        .into_iter()
                                        .filter(|r| r.threads < n)
                                        .collect();
                                    }
                                    options::Range =>
                                    {
                                        vector_process2 = vector_process2
                                        .into_iter()
                                        .filter(|r| r.threads > n && r.threads < n2)
                                        .collect();
                                    }
                                }
                            },
                            filter::FdCount => {
                                let n: usize = FromStr::from_str(&value3).unwrap();
                                let n2: usize;
                                if value4 != " " {
                                    n2 = FromStr::from_str(&value4).unwrap();
                                }
                                else{
                                    n2 = 0;
                                }
                                match option_criteria{
                                    options::Greater =>
                                    {
                                        vector_process2 = vector_process2
                                        .into_iter()
                                        .filter(|r| r.fd_count > n)
                                        .collect();
                                    }
                                    options::Less =>
                                    {
                                        vector_process2 = vector_process2
                                        .into_iter()
                                        .filter(|r| r.fd_count < n)
                                        .collect();
                                    }
                                    options::Range =>
                                    {
                                        vector_process2 = vector_process2
                                        .into_iter()
                                        .filter(|r| r.fd_count > n && r.fd_count < n2)
                                        .collect();
                                    }
                                }
                            },
                        }
                        std::thread::sleep(duration);
                        cb_sink
                            .send(Box::new(move |s| {
                                s.call_on_name("table", |v: &mut TableView::<Process, BasicColumn>| {
                                    v.set_items(vector_process2)
                                });
                            }))
                            .unwrap();
                            cb_sink
                        .send(Box::new(move |s| {
                            s.call_on_name("System Information", |v: &mut TextView| {
                                v.set_content(sys_info_str)
                            });
                        }))
                        .unwrap();
                        }
                });
            search_siv.set_theme(theme);
            search_siv.run();
}
fn main()
{
    let handle = std::thread::spawn(|| {
    let output = Command::new("./external/process_manager_gui").output();
    });
    show_complete_view();
    handle.join().unwrap();
}



	
