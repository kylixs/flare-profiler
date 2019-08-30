
use std::collections::*;
//use call_tree::{TreeArena, NodeId};
use std::collections::hash_map::Entry;
use time::Duration;
use std::net::{TcpStream, ToSocketAddrs};
use resp::Value;
use std::io::{Write, Read, BufReader, Error};
use std::str::from_utf8;
use std::io;
use chrono::Local;
use flare_utils::timeseries::*;
use flare_utils::tuple_indexed::*;
use flare_utils::tuple_indexed::TupleIndexedFile;
use flare_utils::timeseries::TimeSeriesFileWriter;
use flare_utils::{ValueType};
use std::path::PathBuf;
use client_utils;
use client_utils::{get_resp_property, parse_resp_properties};
use std::hash::Hash;


type JavaLong = i64;
type JavaMethod = i64;

#[derive(Clone)]
pub struct ThreadData {
    pub id: JavaLong,
    pub name: String,
    pub priority: u32,
    pub daemon: bool,
    pub state: String,
    pub cpu_time: i64,
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


pub struct SamplerClient {
    connected: bool,
    agent_addr: String,
    agent_stream: Option<TcpStream>,

    //sample option
    sample_interval: i64,
    sample_start_time: i64,

    //sample data processor
    threads : HashMap<JavaLong, ThreadData>,
    sample_data_dir: String,
    sample_cpu_ts_map: HashMap<JavaLong, TimeSeriesFileWriter>,
    sample_stacktrace_map: HashMap<JavaLong, TupleIndexedFile>,
    sample_method_idx_file: TupleIndexedFile,
//    method_cache: HashMap<MethodId, MethodInfo>,
//    tree_arena: TreeArena
}

impl SamplerClient {

    pub fn new(addr: &str) -> io::Result<SamplerClient> {

        //create sample data dir
        let now = Local::now();
        let now_time = now.format("%Y%m%dT%H%M%S").to_string();
        let sample_data_dir = format!("flare-samples/{}-{}", addr.replace(":","_"), now_time);
        std::fs::create_dir_all(sample_data_dir.clone())?;
        println!("save sample data to dir: {}", sample_data_dir);

        //method info idx file
        let mut method_idx_path = format!("{}/method_info", sample_data_dir);
        let mut sample_method_idx_file = TupleIndexedFile::new_writer(&method_idx_path, ValueType::INT64)?;

        Ok(SamplerClient {
            sample_interval: 20,
            sample_start_time: 0,
            threads: HashMap::new(),
            sample_data_dir,
            sample_cpu_ts_map: HashMap::new(),
            sample_stacktrace_map: HashMap::new(),
            sample_method_idx_file,
            connected: false,
            agent_addr: addr.to_string(),
            agent_stream: None,
//            method_cache: HashMap::new(),
//            tree_arena: TreeArena::new()
        })
    }

    fn connect_agent(&mut self) -> io::Result<TcpStream> {
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

        let mut decoder = resp::Decoder::with_buf_bulk(BufReader::new(stream));
        while match decoder.decode() {
            Ok(data) => {
                self.on_sample_data(data);
                true
            },
            Err(e) => {
                println!("Failed to receive data: {}", e);
                false
            }
        }{}
        println!("subscribe events is stopped.");
        Ok(true)
    }

    fn on_sample_data(&mut self, sample_data: resp::Value) {
        println!("events: \n{}", sample_data.to_string_pretty());
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
    }

    fn on_sample_info_data(&mut self, data_vec: &Vec<Value>) {
        let mut start_time= 0;
        if let Some(Value::Integer(x)) = get_resp_property(data_vec, "start_time", 1) {
            start_time = *x;
        }
        let mut sample_interval= 0;
        if let Some(Value::Integer(x)) = get_resp_property(data_vec, "sample_interval", 1) {
            sample_interval = *x;
        }
        self.sample_start_time = start_time;
        self.sample_interval = sample_interval;
        println!("on sample info: start_time:{}, sample_interval:{}", start_time, sample_interval);
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
        //let mut thread_id = get_resp_int_value(map, "id");
        let mut time= 0;
        if let Some(Value::Integer(x)) = get_resp_property(data_vec, "time", 1) {
            time = *x;
        }
        let mut thread_id = 0;
        if let Some(Value::Integer(x)) = get_resp_property(data_vec, "id", 1) {
            thread_id = *x;
        }
        let mut cpu_time = 0;
        if let Some(Value::Integer(x)) = get_resp_property(data_vec, "cpu_time", 1) {
            cpu_time = *x;
        }
        let mut cpu_time_delta = 0;
        if let Some(Value::Integer(x)) = get_resp_property(data_vec, "cpu_time_delta", 1) {
            cpu_time_delta = *x;
        }
        let mut name = "";
        if let Some(Value::String(x)) = get_resp_property(data_vec, "name", 1) {
            name = x;
        }
        let mut state = "";
        if let Some(Value::String(x)) = get_resp_property(data_vec, "state", 1) {
            state = x;
        }
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
                sample_time: time,
                stacktrace: vec![]
            }
        });

        //save thread cpu time
//        let mut cpu_ts = self.sample_cpu_ts_map.entry(thread_id).or_insert_with(||{
//            let mut path = format!("{}/thread_cpu_time_{}", self.sample_data_dir, thread_id);
//            match TimeSeriesFileWriter::new(ValueType::INT32, self.sample_interval as i32, &path) {
//                Ok(ts) => ts,
//                Err(e) => {
//                    println!("create thread cpu ts file failed: thread_id: {}, err: {}", thread_id, e);
//                    return;
//                }
//            }
//        });
//        cpu_ts.add_value(time, TSValue::int32(cpu_time_delta as i32));

        //TODO self.save_thread_stack(*thread_id, )

    }

    fn save_method_info(&mut self, method_id: i64, method_name: &String) {
        self.sample_method_idx_file.add_value(TupleValue::int64(method_id), method_name.as_bytes());
    }

    fn create_thread_cpu_ts_file(&mut self, thread_id: i64) {

        let ts_path = "thread_cpu_time_ts_";
        let unit_time = 100 as i64;
        let mut thread_cpu_ts = TimeSeriesFileWriter::new(ValueType::INT16, unit_time as i32, ts_path).unwrap();

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
