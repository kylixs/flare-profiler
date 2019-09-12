
use std::collections::*;
//use call_tree::{TreeArena, NodeId};
use std::collections::hash_map::Entry;
use time::Duration;
use std::net::{TcpStream, ToSocketAddrs};
use resp::Value;
use std::io::{Write, Read, BufReader, Error, ErrorKind};
use std::str::from_utf8;
use std::io;
use chrono::Local;
use flare_utils::timeseries::*;
use flare_utils::tuple_indexed::*;
use flare_utils::tuple_indexed::{TupleIndexedFile, TupleValue};
use flare_utils::timeseries::{TimeSeries, TSValue, TimeSeriesFileWriter, TimeSeriesFileReader};
use flare_utils::{ValueType, file_utils};
use std::path::PathBuf;
use client_utils;
use client_utils::*;
use std::hash::Hash;
use std::sync::{Mutex, Arc};
use std::cmp::min;
use serde::{Deserialize, Serialize};
use serde_json::json;
use flare_utils::file_utils::open_file;

type JavaLong = i64;
type JavaMethod = i64;

#[derive(Clone, Serialize, Deserialize)]
pub struct ThreadData {
    pub id: JavaLong,
    pub name: String,
    pub priority: u32,
    pub daemon: bool,
    pub state: String,
    pub cpu_time: i64,
    pub cpu_time_delta: i64,
    pub sample_time: i64,
    pub stacktrace: Vec<i64>
}

#[derive(Clone)]
pub struct MethodData {
    pub method_id: i64,
    pub full_name: String,
    pub hits_count: u32
//    pub source_file: String,
//    pub line_num: u16
}

#[derive(Serialize, Deserialize)]
pub struct DashboardInfo {
    sample_info: SampleInfo,
    threads: Vec<ThreadData>
    //jvm_info: JvmInfo,
}

#[derive(Serialize, Deserialize)]
pub struct SampleInfo {
    pub sample_interval: i64,
    pub sample_start_time: i64,
    pub record_start_time: i64,
    pub last_record_time: i64,
    pub agent_addr: String,
    pub sample_data_dir: String,
}

#[derive(Serialize, Deserialize)]
pub struct SummaryInfo {
    sample_info: SampleInfo,
    threads: Vec<ThreadData>
}

pub struct SamplerClient {
    //self ref
    this_ref: Option<Arc<Mutex<SamplerClient>>>,
    connected: bool,
    agent_addr: String,
    agent_stream: Option<TcpStream>,
    readonly: bool,

    //sample option
    sample_interval: i64,
    sample_start_time: i64,
    sample_type: String,

    //client
    record_start_time: i64,
    last_record_time: i64,
    //last save summary time
    last_save_time: i64,

    //sample data processor
    threads : HashMap<JavaLong, ThreadData>,
    sample_data_dir: String,
    sample_cpu_ts_map: HashMap<JavaLong, Option<Box<TimeSeries+Send>>>,
    sample_stacktrace_map: HashMap<JavaLong, Option<TupleIndexedFile>>,
    sample_method_idx_file: Option<TupleIndexedFile>,
//    method_cache: HashMap<MethodId, MethodInfo>,
//    tree_arena: TreeArena
}

impl SamplerClient {


    pub fn new(addr: &str) -> io::Result<Arc<Mutex<SamplerClient>>> {
        let mut client = SamplerClient::new_instance();
        client.lock().unwrap().agent_addr = addr.to_string();
        Ok(client)
    }

    pub fn open(sample_dir: &str) -> io::Result<Arc<Mutex<SamplerClient>>> {
        println!("load sample data from dir: {}", sample_dir);
        let mut client = SamplerClient::new_instance();
        client.lock().unwrap().load_sample(sample_dir)?;

        Ok(client)
    }

    fn new_instance() -> Arc<Mutex<SamplerClient>> {
        let mut client = Arc::new(Mutex::new(SamplerClient {
            this_ref: None,
            readonly: false,
            sample_type: "".to_string(),
            sample_interval: 20,
            sample_start_time: 0,
            record_start_time: 0,
            last_record_time: 0,
            last_save_time: 0,
            threads: HashMap::new(),
            sample_data_dir: "".to_string(),
            sample_cpu_ts_map: HashMap::new(),
            sample_stacktrace_map: HashMap::new(),
            sample_method_idx_file: None,
            connected: false,
            agent_addr: "".to_string(),
            agent_stream: None,
//            method_cache: HashMap::new(),
//            tree_arena: TreeArena::new()
        }));
        //self ref for threads
        client.lock().unwrap().this_ref = Some(client.clone());
        client
    }

    //加载取样数据
    fn load_sample(&mut self, sample_data_dir: &str) -> io::Result<()> {
        self.readonly = true;
        self.sample_type = "file".to_string();
        self.sample_data_dir = sample_data_dir.to_string();
        let dir_meta = std::fs::metadata(sample_data_dir);
        if dir_meta.is_err() {
            return new_error(ErrorKind::NotFound, "sample data dir not found");
        }
        if !dir_meta.unwrap().is_dir() {
            return new_error(ErrorKind::NotFound, "sample data dir is not a directory");
        }

        self.sample_data_dir = sample_data_dir.to_string();

        //summary info
        let path = format!("{}/summary_info.json", sample_data_dir);
        let json = std::fs::read_to_string(path)?;

        let summary = serde_json::from_str::<SummaryInfo>(&json);
        if summary.is_err() {
            println!("load summary info json failed: {}", summary.err().unwrap());
            return new_error(ErrorKind::InvalidData, "load summary info json failed");
        }
        let summary = summary.unwrap();

        let sample_info = &summary.sample_info;
        self.sample_start_time = sample_info.sample_start_time;
        self.sample_interval = sample_info.sample_interval;
        self.agent_addr = sample_info.agent_addr.clone();
        self.record_start_time = sample_info.record_start_time;
        self.last_record_time = sample_info.last_record_time;

        //threads
        for thread in &summary.threads {
            self.threads.insert(thread.id, thread.clone());

            //load cpu time ts
            let thread_cpu_ts_file = format!("{}/thread_{}_cpu_time", sample_data_dir, thread.id);
            match TimeSeriesFileReader::new(&thread_cpu_ts_file) {
                Ok(ts) => {
                    self.sample_cpu_ts_map.insert(thread.id, Some(Box::new(ts)));
                },
                Err(e) => {
                    println!("load thread cpu time file failed: {}, err: {}", thread_cpu_ts_file, e);
                }
            }

            //load thread stacktrace
            let thread_stack_file = format!("{}/thread_{}_stack", sample_data_dir, thread.id);
            match TupleIndexedFile::new_reader(&thread_stack_file) {
                Ok(file) => {
                    self.sample_stacktrace_map.insert(thread.id, Some(file));
                },
                Err(e) => {
                    println!("load thread stacktrace file failed: {}, err: {}", thread_stack_file, e);
                }
            }
        }

        //method info idx file
        let method_idx_path = format!("{}/method_info", sample_data_dir);
        let mut method_idx_file = TupleIndexedFile::new_writer(&method_idx_path, ValueType::INT64)?;
        self.sample_method_idx_file = Some(method_idx_file);

        //load threads
//        let paths = std::fs::read_dir("sample_data_dir")?;
//        for path in paths {
//            let file_name =  path.unwrap().file_name().into_string().unwrap();
//            if file_name.starts_with("thread_") {
//                //TODO
//            }
//        }

        println!("load sample dir is done: {}", sample_data_dir);
        Ok(())
    }

    //按小时滚动更换数据保存目录
    fn check_and_roll_data_dir(&mut self, sample_time: i64) -> io::Result<bool> {
        if self.record_start_time==0 || sample_time - self.record_start_time > 3600_000 {
            //create sample data dir
            let now = Local::now();
            let now_time = now.format("%Y%m%dT%H%M%S").to_string();
            let sample_data_dir = format!("flare-samples/{}-{}", self.agent_addr.replace(":","_"), now_time);
            std::fs::create_dir_all(sample_data_dir.clone())?;
            println!("save sample data to dir: {}", sample_data_dir);

            //method info idx file
            let method_idx_path = format!("{}/method_info", sample_data_dir);
            if self.sample_method_idx_file.is_some() {
                //copy old method info file to new dir
                let old_method_file = format!("{}/method_info.fidx", self.sample_data_dir);
                let new_method_file = format!("{}/method_info.fidx", sample_data_dir);
                std::fs::copy(old_method_file, new_method_file)?;

                let old_method_file = format!("{}/method_info.fdata", self.sample_data_dir);
                let new_method_file = format!("{}/method_info.fdata", sample_data_dir);
                std::fs::copy(old_method_file, new_method_file)?;
            }
            let mut method_idx_file = TupleIndexedFile::new_writer(&method_idx_path, ValueType::INT64)?;

            self.record_start_time = sample_time;
            self.sample_data_dir = sample_data_dir;
            self.sample_method_idx_file = Some(method_idx_file);
            self.sample_cpu_ts_map.clear();
            self.sample_stacktrace_map.clear();
            Ok(true)
        }else {
            Ok(false)
        }
    }

    fn save_summary_info(&mut self) -> io::Result<()> {
        let now = Local::now().timestamp_millis();
        if now - self.last_save_time < 1000 {
            //ignore write file frequently
            return Ok(());
        }

        let mut info = SummaryInfo {
            sample_info: self.get_sample_info(),
            threads: vec![]
        };
        for thread in self.threads.values() {
            info.threads.push(thread.clone());
        }

        let path = format!("{}/summary_info.json", self.sample_data_dir);
        match file_utils::open_file(&path, true) {
            Ok(mut file) => {
                let json = serde_json::to_string_pretty(&info).unwrap();
                file.write_all(json.as_bytes());
                file.set_len(json.as_bytes().len() as u64);
                self.last_save_time = Local::now().timestamp_millis();
                Ok(())
            }
            Err(e) => {
                println!("save summary info failed: {}", e);
                Err(e)
            }
        }
    }

    pub fn connect_agent(&mut self) -> io::Result<TcpStream> {
        self.sample_type = "attach".to_string();
        match TcpStream::connect(&self.agent_addr) {
            Ok(mut stream) => {
                println!("Successfully connected to flare agent at: {:?}", self.agent_addr);
                Ok(stream)
            }
            Err(e) => {
                println!("Failed to connect to flare agent: {:?}, error: {:?}", self.agent_addr, e);
                Err(e)
            }
        }
    }

    pub fn subscribe_events(&mut self) -> Result<bool, Error> {
        let mut stream = self.connect_agent()?;
        let cmdValue = resp::Value::Array(vec![Value::String("subscribe-events".to_string())]);
        let cmd = cmdValue.encode();
        let size = stream.write(cmd.as_slice()).unwrap();
        println!("start subscribe events, awaiting reply: {}", cmdValue.to_encoded_string()?);


        if let Some(this_ref) = &self.this_ref {
            let this = this_ref.clone();
            std::thread::spawn(move ||{
                let mut decoder = resp::Decoder::with_buf_bulk(BufReader::new(stream));
                while match decoder.decode() {
                    Ok(data) => {
                        this.lock().unwrap().on_sample_data(data);
                        true
                    },
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                        false
                    }
                }{}
                println!("subscribe events is stopped.");
            });
        }
        Ok(true)
    }

    fn on_sample_data(&mut self, sample_data: resp::Value) {
        //println!("events: \n{}", sample_data.to_string_pretty());
        if let resp::Value::Array(data_vec) = sample_data {
            if let Value::String(cmd) = &data_vec[0] {
                if cmd == "method" {
                    self.on_method_data(&data_vec);
                } else if cmd == "thread" {
                    self.on_thread_data(&data_vec);
                } else if cmd == "sample_info" {
                    self.on_sample_info_data(&data_vec);
                }
            }
        }

        self.save_summary_info();
    }

    fn on_sample_info_data(&mut self, data_vec: &Vec<Value>) {
        let start_time= get_resp_property_as_int(data_vec, "start_time", 1, 0);
        let sample_interval= get_resp_property_as_int(data_vec, "sample_interval", 1, 0);
        let last_sample_time= get_resp_property_as_int(data_vec, "last_sample_time", 1, 0);
        self.sample_start_time = start_time;
        self.sample_interval = sample_interval;
        println!("on sample info: start_time:{}, sample_interval:{}", start_time, sample_interval);

        self.check_and_roll_data_dir(last_sample_time);
    }

    fn on_method_data(&mut self, data_vec: &Vec<Value>) {
        if let Some(Value::Integer(method_id)) = get_resp_property(data_vec, "id", 1) {
            if let Some(Value::String(method_name)) = get_resp_property(data_vec, "name", 1) {
                self.save_method_info(*method_id, method_name);
            }else {
                println!("parse method name failed")
            }
        }else {
            println!("parse method id failed")
        }
    }

    fn on_thread_data(&mut self, data_vec: &Vec<Value>) {
        //let map = parse_resp_properties(data_vec, 1);
        let sample_time= get_resp_property_as_int(data_vec, "time", 1, 0);
        let thread_id= get_resp_property_as_int(data_vec, "id", 1, 0);
        let cpu_time= get_resp_property_as_int(data_vec, "cpu_time", 1, 0);
        let cpu_time_delta= get_resp_property_as_int(data_vec, "cpu_time_delta", 1, 0);
        let name= get_resp_property_as_str(data_vec, "name", 1, "");
        let state= get_resp_property_as_str(data_vec, "state", 1, "");
        let mut stacktrace = &vec![];
        if let Some(Value::Array(x)) = get_resp_property(data_vec, "stacktrace", 1) {
            stacktrace = x;
        }

        //create thread cpu ts
        let mut is_new = false;
        let thread_data = self.threads.entry(thread_id).or_insert_with(||{
            is_new = true;
            ThreadData {
                id: thread_id,
                name: name.to_string(),
                priority: 0,
                daemon: false,
                state: state.to_string(),
                cpu_time: cpu_time,
                cpu_time_delta: cpu_time_delta,
                sample_time: sample_time,
                stacktrace: vec![]
            }
        });
        thread_data.sample_time = sample_time;
        thread_data.cpu_time = cpu_time;
        thread_data.cpu_time_delta = cpu_time_delta;
        thread_data.state = state.to_string();
        thread_data.name = name.to_string();

        //prepare data dir
        self.check_and_roll_data_dir(sample_time);
        self.last_record_time = sample_time;
        if is_new {
            self.save_summary_info();
        }

        //save thread cpu time
        let sample_interval = self.sample_interval as i32;
        let sample_data_dir = &self.sample_data_dir;
        let cpu_ts = self.sample_cpu_ts_map.entry(thread_id).or_insert_with(||{
            let path = format!("{}/thread_{}_cpu_time", sample_data_dir, thread_id);
            match TimeSeriesFileWriter::new(ValueType::INT32, sample_interval , sample_time, &path) {
                Ok(ts) => Some(Box::new(ts)),
                Err(e) => {
                    println!("create thread cpu ts file failed: thread_id: {}, err: {}", thread_id, e);
                    None
                }
            }
        });
        let mut ts_steps = 0u32;
        if let Some(ts) = cpu_ts {
            if let Ok(steps) = ts.add_value(sample_time, TSValue::int32((cpu_time_delta/1000 as i64) as i32)) {
                ts_steps = steps;
            }
        }

        //save thread stack data
        let thread_stack_idx = self.sample_stacktrace_map.entry(thread_id).or_insert_with(||{
            let path = format!("{}/thread_{}_stack", sample_data_dir, thread_id);
            match TupleIndexedFile::new_writer(&path, ValueType::UINT32) {
                Ok(idx_file) => Some(idx_file),
                Err(e) => {
                    println!("create thread cpu ts file failed: thread_id: {}, err: {}", thread_id, e);
                    None
                }
            }
        });
        if let Some(idx_file) = thread_stack_idx {
            let stack_data = Value::Array(stacktrace.clone());
            idx_file.add_value(TupleValue::uint32(ts_steps), stack_data.encode().as_slice());
        }
    }

    fn save_method_info(&mut self, method_id: i64, method_name: &String) {
        if let Some(idx) = self.sample_method_idx_file.as_mut() {
            idx.add_value(TupleValue::int64(method_id), method_name.as_bytes());
        }
    }

    pub fn get_dashboard(&mut self) -> DashboardInfo {

        let mut info = DashboardInfo {
            sample_info: self.get_sample_info(),
            threads: vec![]
        };

        //println!("{:8} {:48} {:8} {:8} {:8} {:8} {:8} {:8}", "ID", "NAME", "GROUP", "PRIORITY", "STATE", "%CPU", "TIME", "DAEMON");
        for thread in self.threads.values_mut() {
            //计算CPU占用率
            let cpu_util = 1;
//            let cpu_time = thread.cpu_time / 1000_000;
            //println!("{:<8} {:<48} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8}", thread.id,  &thread.name[0..min(48, thread.name.len())], "main", thread.priority, thread.state, cpu_util, cpu_time, thread.daemon );

            info.threads.push(thread.clone())
        }

        info
    }

    pub fn get_sample_info(&self) -> SampleInfo {
        SampleInfo {
            sample_start_time: self.sample_start_time,
            record_start_time: self.record_start_time,
            last_record_time: self.last_record_time,
            sample_interval: self.sample_interval,
            agent_addr: self.agent_addr.clone(),
            sample_data_dir: self.sample_data_dir.clone()
        }
    }

    pub fn get_sample_type(&self) -> String {
        self.sample_type.clone()
    }

    pub fn get_thread_cpu_time(&self, thread_id: &i64, start_time: i64, end_time: i64, unit_time_ms: i64) -> Option<TSResult> {
//        match self.sample_cpu_ts_map.get(thread_id) {
//            Some(ts) => {
//                match ts {
//                    Some(ts) => {
//                        Some(ts.get_range_value(start_time, end_time, unit_time_ms as i32))
//                    },
//                    None => None,
//                }
//            },
//            None => None
//        }

        self.sample_cpu_ts_map.get(thread_id).map_or(None,|ts|{
            ts.as_ref().map( |ts| {
                ts.get_range_value(start_time, end_time, unit_time_ms as i32)
            })
        })
    }

//    pub fn write_all_call_trees(&self, writer: &mut std::io::Write, compact: bool) {
//        for (thread_id, call_tree) in self.tree_arena.get_all_call_trees() {
//            let tree_name = &call_tree.get_root_node().data.name;
//            writer.write_fmt(format_args!("Thread: {}, {}, {}\n", &call_tree.thread_id, tree_name, call_tree.total_duration as f64/1000_000.0));
//
//            writer.write_all(call_tree.format_call_tree(compact).as_bytes());
//            writer.write_all("\n".as_bytes());
//        }
//    }
//
//    pub fn add_stack_traces(&mut self, jvm_env: &Box<Environment>, stack_traces: &Vec<JavaStackTrace>) {
//        //merge to call stack tree
//        for (i, stack_info) in stack_traces.iter().enumerate() {
//            if let Ok(thread_info) = jvm_env.get_thread_info(&stack_info.thread) {
//                let mut cpu_time: i64 = 0_i64;
//                //if std::time::Instant::now()
//                if let Ok(t) = jvm_env.get_thread_cpu_time(&stack_info.thread) {
//                    cpu_time = t;
//                    //ignore inactive thread call
//                    if cpu_time == 0_i64 {
//                        continue;
//                    }
//                } else {
//                    println!("get_thread_cpu_time error");
//                }
//
//                let call_tree = self.tree_arena.get_call_tree(&thread_info);
//                call_tree.reset_top_call_stack_node();
//                if call_tree.total_duration == cpu_time {
//                    continue;
//                }
//
//                let mut call_methods :Vec<JavaMethod> = vec![];
//                for stack_frame in &stack_info.frame_buffer {
//                    call_methods.push(stack_frame.method);
//                }
//                //save nodes in temp vec, process it after build call tree, avoid second borrow muttable *self
//                let mut naming_nodes: Vec<(NodeId, JavaMethod)> = vec![];
//
//                //reverse call
//                for method_id in call_methods.iter().rev() {
//                    if !call_tree.begin_call(method_id) {
//                        naming_nodes.push((call_tree.get_top_node().data.node_id, method_id.clone()));
//                    }
//                }
//
//                call_tree.end_last_call(cpu_time);
//                //println!("add call stack: {} cpu_time:{}", thread_info.name, cpu_time);
//
//                //get method call_name of node
//                let mut node_methods: Vec<(NodeId, String)> = vec![];
//                for (node_id, method_id) in naming_nodes {
//                    let method_info = self.get_method_info(jvm_env, method_id);
//                    let call_name = format!("{}.{}()", &method_info.class.name, &method_info.method.name);
//                    node_methods.push((node_id, call_name));
//                }
//
//                //set node's call_name
//                let call_tree = self.tree_arena.get_call_tree(&thread_info);
//                for (node_id, call_name) in node_methods {
//                    call_tree.get_mut_node(&node_id).data.name = call_name;
//                }
//            }else {
//                //warn!("Thread UNKNOWN [{:?}]: (cpu_time = {})", stack_info.thread, cpu_time);
//            }
//        }
//    }
//
//    pub fn format_stack_traces(&mut self, jvm_env: &Box<Environment>, stack_traces: &Vec<JavaStackTrace>) -> String {
//        let mut result  = String::new();
//        for (i, stack_info) in stack_traces.iter().enumerate() {
//            result.push_str(&format!("\nstack_info: {}, thread: {:?}, state: {:?}\n", (i+1), stack_info.thread, stack_info.state));
//
//            let mut cpu_time = -1f64;
//            match jvm_env.get_thread_cpu_time(&stack_info.thread) {
//                Ok(t) => { cpu_time = t as f64 / 1000_000.0 },
//                Err(err) => {
//                    result.push_str(&format!("get_thread_cpu_time error: {:?}\n", err))
//                }
//            }
//
//            if let Ok(thread_info) = jvm_env.get_thread_info(&stack_info.thread) {
//                result.push_str(&format!("Thread {}: (id = {}, priority = {}, daemon = {}, state = {:?}, cpu_time = {}) \n",
//                                         thread_info.name, thread_info.thread_id,  thread_info.priority, thread_info.is_daemon, stack_info.state, cpu_time ));
//            } else {
//                result.push_str(&format!("Thread UNKNOWN [{:?}]: (cpu_time = {}) \n", stack_info.thread, cpu_time));
//            }
//
//            for stack_frame in &stack_info.frame_buffer {
//                let method_info = self.get_method_info(jvm_env, stack_frame.method);
//                result.push_str(&format!("{}.{}()\n", &method_info.class.name, &method_info.method.name));
//            }
//        }
//        result
//    }
//
//    fn get_method_info(&mut self, jvm_env: &Box<Environment>, method: JavaMethod) -> &MethodInfo {
//        let method_id = MethodId { native_id: method };
//        self.method_cache.entry(method_id).or_insert_with(|| {
//            let method = jvm_env.get_method_name(&method_id).unwrap();
//            let class_id = jvm_env.get_method_declaring_class(&method_id).unwrap();
//            let class = jvm_env.get_class_signature(&class_id).unwrap();
//            MethodInfo {
//                method_id: method_id,
//                method,
//                class
//            }
//        })
//        //self.method_cache.get(&method_id).unwrap()
//    }
}


impl Drop for SamplerClient {
    fn drop(&mut self) {
        println!("save summary info before destroy sampler client ..");
        self.save_summary_info();
    }
}