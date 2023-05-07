mod system;
mod process;
mod dial;
use std::path::Path;
use std::fs;
use std::io::prelude::*;
use std::fs::File;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use crate::dial::MyDial;
use fltk::output::Output;
use crate::system::System;
use fltk::{app, /*frame::Frame as FltkFrame*/prelude::*, window::Window as FltkWindow, group::Group as FltkGroup};
use fltk::{enums::*, /*prelude::**/};
use fltk::group::Flex as GroupFlex;
use fltk_extras::dial::*;
use fltk::misc::Chart as FltkChart;
use fltk::enums::Color as FltkColor;
use fltk::misc::ChartType as FltkChartType;
use std::str::FromStr;
use fltk::group::{Group, Pack, Tabs};
use fltk::frame::Frame;
use fltk::{
    prelude::*,
    tree::{Tree, TreeSelect},
    window::Window,
};
//use fltk::menu::Choice as FltkChoice;

#[derive(Clone,Debug)]
struct ProcessRecord {
 name: String,
 pid: i32,
 ppid: i32,
}

#[derive(Clone,Debug)]
struct ProcessTreeNode {
 record: ProcessRecord, // the node owns the associated record
 children: Vec<ProcessTreeNode>, // nodes own their children
}

#[derive(Clone,Debug)]
struct ProcessTree {
 root: ProcessTreeNode, // tree owns ref to root node
}

impl ProcessTreeNode {
 // constructor
 fn new(record : &ProcessRecord) -> ProcessTreeNode {
 ProcessTreeNode { record: (*record).clone(), children: Vec::new() }
 }
}


// Given a status file path, return a hashmap with the following form:
// pid -> ProcessRecord
fn get_process_record(status_path: &Path) -> Option<ProcessRecord> {
 let mut pid : Option<i32> = None;
 let mut ppid : Option<i32> = None;
 let mut name : Option<String> = None;

 let mut reader = std::io::BufReader::new(File::open(status_path).unwrap());
 loop {
 let mut linebuf = String::new();
 match reader.read_line(&mut linebuf) {
 Ok(_) => {
 if linebuf.is_empty() {
 break;
 }
 let parts : Vec<&str> = linebuf[..].splitn(2, ':').collect();
 if parts.len() == 2 {
 let key = parts[0].trim();
 let value = parts[1].trim();
 match key {
 "Name" => name = Some(value.to_string()),
 "Pid" => pid = value.parse().ok(),
 "PPid" => ppid = value.parse().ok(),
 _ => (),
 }
 }
 },
 Err(_) => break,
 }
 }
 return if pid.is_some() && ppid.is_some() && name.is_some() {
 Some(ProcessRecord { name: name.unwrap(), pid: pid.unwrap(), ppid: ppid.unwrap() })
 } else {
 None
 }
}


// build a simple struct (ProcessRecord) for each process
fn get_process_records() -> Vec<ProcessRecord> {
 let proc_directory = Path::new("/proc");

 // find potential process directories under /proc
 let proc_directory_contents = fs::read_dir(&proc_directory).unwrap();
 proc_directory_contents.filter_map(|entry| {
 let entry_path = entry.unwrap().path();
 if fs::metadata(entry_path.as_path()).unwrap().is_dir() {
 let status_path = entry_path.join("status");
 if let Ok(metadata) = fs::metadata(status_path.as_path()) {
 if metadata.is_file() {
 return get_process_record(status_path.as_path());
 }
 }
 }
 None
 }).collect()
}

fn populate_node_helper(node: &mut ProcessTreeNode, pid_map: &HashMap<i32, &ProcessRecord>, ppid_map: &HashMap<i32, Vec<i32>>) {
 let pid = node.record.pid; // avoid binding node as immutable in closure
 let child_nodes = &mut node.children;
 match ppid_map.get(&pid) {
 Some(children) => {
 child_nodes.extend(children.iter().map(|child_pid| {
 let record = pid_map[child_pid];
 let mut child = ProcessTreeNode::new(record);
 populate_node_helper(&mut child, pid_map, ppid_map);
 child
 }));
 },
 None => {},
 }
}

fn populate_node(node : &mut ProcessTreeNode, records: &Vec<ProcessRecord>) {
 // O(n): build a mapping of pids to vectors of children. That is, each
 // key is a pid and its value is a vector of the whose parent pid is the key
 let mut ppid_map : HashMap<i32, Vec<i32>> = HashMap::new();
 let mut pid_map : HashMap<i32, &ProcessRecord> = HashMap::new();
 for record in records.iter() {
 // entry returns either a vacant or occupied entry. If vacant,
 // we insert a new vector with this records pid. If occupied,
 // we push this record's pid onto the vec
 pid_map.insert(record.pid, record);
 match ppid_map.entry(record.ppid) {
 Vacant(entry) => { entry.insert(vec![record.pid]); },
 Occupied(mut entry) => { entry.get_mut().push(record.pid); },
 };
 }

 // With the data structures built, it is off to the races
 populate_node_helper(node, &pid_map, &ppid_map);
}

fn build_process_tree() -> ProcessTree {
 let records = get_process_records();
 let mut tree = ProcessTree {
 root : ProcessTreeNode::new(
 &ProcessRecord {
 name: "/".to_string(),
 pid: 0,
 ppid: -1
 })
 };

 // recursively populate all nodes in the tree starting from root (pid 0)
 {
 let root = &mut tree.root;
 populate_node(root, &records);
 }
 tree
}

#[derive(Debug, Copy, Clone)]
pub enum Message {
    Increment(),
    Decrement(),
}

pub fn inc_frame(d: &mut HalfDial, value: & f32) {
    d.set_value(*value as i32);
}

pub fn update_sys_info(d: &mut MyDial, value: & f32) {
    d.set_value(*value as i32);
}

pub fn update_sys_info2(d: &mut MyDial, value: & f64) {
    d.set_value(*value as i32);
}
fn main(){
    let mut curr_system = System::new();
    let duration = std::time::Duration::from_millis(500);
    curr_system.update();
    let a: app::App = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut wind = FltkWindow::default().with_size(1000, 1000);
    let tab: Tabs = Tabs::new(10, 10, 1000 - 20, 950 - 20, "");
    let grp1: Group = Group::new(10, 35, 950 - 20, 500, "CPU Usage\t\t");
    let CpuDialog: Group = Group::new(15, 55, 950 - 20,200, "CPU Usage\t");
    let mut frame = Frame::new(15, 70, 960, 150, None);
    frame.set_color(Color::Black);
    frame.set_frame(FrameType::BorderFrame);
    let mut pack = Pack::new(20, 75, 950, 200, "");
    pack.set_spacing(10);
    let col = GroupFlex::default()
        .row()
        .with_size(950, 130)
        .with_pos(25,75);
    let mut dial_list: Vec<HalfDial>;
    dial_list = vec![];
    let mut count = 0;
    for cpu in &curr_system.cpu_core_usages {
        let text = "CPU ";
        count = count + 1;
        let result = text.to_owned() + " " + &count.to_string() + " %";						
        let mut dial = HalfDial::default();
        dial_list.push(dial);
    }
    let mut val = 0;
    col.end();
    pack.end();
    CpuDialog.end();
    let graph: Group = Group::new(500, 400, 950, 600, "CPU Usage\t\t").below_of(&CpuDialog,10);
    let mut chart = FltkChart::default().with_size(950, 570).center_of_parent();
    chart.set_type(FltkChartType::Fill);
    chart.set_bounds(0.0, 200.0);
    chart.set_text_size(18);
    graph.end();
    grp1.end();
    /// /////////////////////////Tree View///////////////////////////////////
    let grp2: Group = Group::new(10, 35, 1000 - 20, 950 - 45, "Tree\t\t");
    let ptree = build_process_tree();
    let mut tree2 = Tree::default().with_size(900,900).center_of_parent();
    tree2.set_select_mode(TreeSelect::Multi);

    
    tree2.add(&(ptree.root.record.name));

    let size = ptree.root.children.len();
    
    
    let mut name1;
    let mut name2;

    let mut name_1;
    let mut name_2;
    let added_str = "/" ;

    for i in 0..size
    {

        name1 = (ptree.root.children[i].record.name).clone();

        name_1 = name1.replace("/","");

        
        tree2.add(&(name_1)); 
     
        
        let sub_size = ptree.root.children[i].children.len();
        

        for j in 0..sub_size
        {
            name2 = (&(ptree.root.children[i].children[j].record.name).clone()).to_string();
            
            
            name_2 = name2.replace("/","");

            

            name_1.push_str(added_str);
            name_1.push_str(&name_2);


            
            tree2.add(&(name_1));

            

            if(ptree.root.children[i].children[j].children.len()) != 0
            {
                let extra_size = ptree.root.children[i].children[j].children.len();

                for k in 0..extra_size
                {
                    name2 = (&(ptree.root.children[i].children[j].children[k].record.name).clone()).to_string();
                    
                    
                    name_2 = name2.replace("/", "");


                    name_1.push_str(added_str);
                    name_1.push_str(&name_2);

                    tree2.add(&(name_1));

                    name_1 = (&(ptree.root.children[i].children[j].record.name).clone()).to_string();
                }
            }
            

            name_1 = (ptree.root.children[i].record.name).clone();

        }
    }
    tree2.set_trigger(fltk::enums::CallbackTrigger::ReleaseAlways);
    grp2.end();
    ///////////////////////////System Overview/////////////////////////////////////
    let grp4: Group = Group::new(10, 35, 1000 - 20, 950 - 45, "System Overview\t\t");
    let grp5: Group = Group::new(15, 60, 960, 250, "System Information \t\t");
    let mut frame2 = Frame::new(15, 70, 960, 200, None);
    frame2.set_color(Color::Black);
    frame2.set_frame(FrameType::BorderFrame);
    //first dial 
    let mut dial1 = MyDial::new(50,110, 200, 200, "CPU Load %");
    dial1.set_label_size(22);
    dial1.set_label_color(Color::from_u32(0x797979));
    //second dial 
    let mut dial2 = MyDial::new(375, 110, 200, 200, "Total Memory %");
    dial2.set_label_size(22);
    dial2.set_label_color(Color::from_u32(0x797979));
    //thrid dial
    let mut dial3 = MyDial::new(700, 110, 200, 200, "Total Disk%");
    dial3.set_label_size(22);
    dial3.set_label_color(Color::from_u32(0x797979));
    //Memory Graph
    let Memorygraph: Group = Group::new(500, 500, 950, 450, "Memory Consumption\t\t").below_of(&grp5,10);
    let mut chart2 = FltkChart::default().with_size(950, 450).center_of_parent();
    chart2.set_type(FltkChartType::Pie);
    chart2.set_bounds(0.0, 100.0);
    chart2.set_text_size(18);
    Memorygraph.end();
    let mut msg: Output = Output::new(500,950,950,50,"").below_of(&Memorygraph,5);
    msg.set_color(FltkColor::from_u32(0xB2BEB5));
    msg.set_value(" Light Blue --> Memory Used                            Blue--> Memory Remaning                         Yellow: Memory available   ");
    grp5.end();
    grp4.end();
    ///////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////Tab 4////////////////////////////////////
    let Processes: Group = Group::new(10, 35, 1000 - 20, 950 - 45, "Processes Info\t\t");
    let grp10: Group = Group::new(15, 60, 950, 900, "Running Processes \t\t");
    let mut chart4 = FltkChart::default().with_size(950,850).center_of_parent();
    chart4.set_type(FltkChartType::Pie);
    chart4.set_bounds(0.0, 100.0);
    chart4.set_text_size(18);
    grp10.end();
    Processes.end();

    tab.end();
    ///////////
    wind.make_resizable(true);
    wind.end();
    wind.show();
    dial3.set_value(40);
    let system = System::new();
    let (s, r) = app::channel::<Message>();
    std::thread::spawn(move || loop {
        app::sleep(0.5);
        s.send(Message::Increment());
    });
    while app::wait() {
        if let Some(Message::Increment()) = r.recv() {
            curr_system.update();
            let mut count2 = 0;
            let mut change_color = 0;
            for mut cpu in &curr_system.cpu_core_usages{
            inc_frame(&mut dial_list[count2 as usize], &cpu);
            count2 = count2 + 1;
            }
            let mut MemoryPercentage= (curr_system.mem_free as f64/curr_system.mem_total as f64)*100.00;
            let mut MemoryUsed= (curr_system.mem_used as f64/curr_system.mem_total as f64)*100.00;
            let mut remaning = 100.00-MemoryUsed-MemoryPercentage;
            update_sys_info(&mut dial1, &curr_system.cpu_current_usage);
            update_sys_info2(&mut dial2, & mut MemoryPercentage);
            // std::thread::sleep(duration);
            // chart.clear();
            // for prc in &curr_system.processes
            // {
            //     let n: f64 = FromStr::from_str(&prc.cpu).unwrap();
            //     chart.add(n, &prc.pid.to_string(), FltkColor::from_u32(0xcc9c59+(change_color*1000)));
            //     change_color = change_color + 1;
            // }
            std::thread::sleep(duration);
            chart.clear();
            chart2.clear();
            chart4.clear();
            let mut countt = 0;
            for cpu in &curr_system.cpu_core_usages
            {
                let n: f64 = FromStr::from_str(&cpu.to_string()).unwrap(); 
                chart.add(n, &countt.to_string(), FltkColor::from_u32(0xcc9c59+(change_color*5000)));
                //chart 2 to change: 
                
                change_color = change_color + 1;
                countt = countt + 1;
            }
        
                chart2.add(MemoryPercentage,&MemoryPercentage.to_string(), FltkColor::from_u32(0xcc9c59));
                chart2.add(MemoryUsed,&MemoryUsed.to_string(),FltkColor::from_u32(0x0000ff));
                chart2.add(remaning,&remaning.to_string(),FltkColor::from_u32(0x00FFFF));
                for mut i in 0..12{
                    chart2.replace(1,MemoryPercentage,&MemoryPercentage.to_string(),FltkColor::from_u32(0xcc9c59));
                    chart2.replace(2,MemoryUsed,&MemoryUsed.to_string(),FltkColor::from_u32(0x0000ff));
                    chart2.replace(3,remaning,&remaning.to_string(),FltkColor::from_u32(0x00FFFF));
                     if(i==11){
                       i = 0;
                     }
                }
                 for prc in &curr_system.processes
                 {
                 let x: f64 = FromStr::from_str(&prc.cpu).unwrap();
                 chart4.add(x, &prc.pid.to_string(), FltkColor::from_u32(0xcc9c59+(change_color*1000)));
                 change_color = change_color + 1;
               
                }
    }
    }
}
