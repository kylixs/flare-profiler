
use super::super::environment::jvmti::*;
use method::{MethodId, MethodSignature};
use std::collections::*;
use native::{JavaMethod, JavaLong};
use class::ClassSignature;
use thread::{ThreadId, Thread};
use environment::Environment;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use profile::tree::{TreeArena, NodeId};
use std::collections::hash_map::Entry;
use time::Duration;
use super::server::*;
use chrono::Local;
use profile::encoder::*;
use std::sync::{Mutex, mpsc};
//use std::sync::mpsc::{Sender, Receiver};

#[derive(Serialize, Deserialize)]
pub struct SampleResult {
    sample_time: i64, //ms
    cpu_time: f64, //ms
    thread_count: i32,
}

pub trait SampleData {
    fn encode(&self) -> Vec<u8>;
    fn get_type(&self) -> String;
}

#[derive(Clone)]
pub struct ThreadData {
    pub id: i64,
    pub name: String,
    pub priority: u32,
    pub daemon: bool,
    pub state: String,
    pub cpu_time: i64,
    pub sample_time: i64,
    pub stacktrace: Vec<i64>,
    pub last_stack_frame: i64,
    pub last_stack_len: usize
}

impl ThreadData {
    pub fn new(id: JavaLong, name: String) -> ThreadData {
        ThreadData {
            id: id as i64,
            name: name,
            priority: 0,
            daemon: false,
            state: "".to_string(),
            cpu_time: 0,
            sample_time: 0,
            stacktrace: vec![],
            last_stack_frame: 0,
            last_stack_len:0
        }
    }
}

impl SampleData for ThreadData {

    fn encode(&self) -> Vec<u8> {
        resp_encode_thread_data(self).encode()
    }

    fn get_type(&self) -> String {
        "thread".to_string()
    }
}

#[derive(Clone)]
pub struct MethodData {
    pub method_id: i64,
    pub full_name: String,
    pub hits_count: u32
//    pub source_file: String,
//    pub line_num: u16
}

impl SampleData for MethodData {
    fn encode(&self) -> Vec<u8> {
        resp_encode_method_data(self).encode()
    }

    fn get_type(&self) -> String {
        "method".to_string()
    }
}

//#[derive(Clone)]
pub struct ResponseData {
    cmd: String,
    data: resp::Value,
}

impl ResponseData {
    pub fn new(cmd: String, data: resp::Value) -> ResponseData {
        ResponseData {
            cmd,
            data
        }
    }
}

impl SampleData for ResponseData {
    fn encode(&self) -> Vec<u8> {
        self.data.encode()
    }

    fn get_type(&self) -> String {
        self.cmd.clone()
    }
}

pub struct Sampler {
    method_cache: HashMap<usize, MethodData>,
    running: bool,
    sample_interval: u64,
    start_time: i64,
    threads_map: HashMap<JavaLong, ThreadData>,
    sender: Option<mpsc::Sender<resp::Value>>,
    receiver: Option<mpsc::Receiver<resp::Value>>,
}

//pub struct MethodInfo {
//    method_id: MethodId,
//    method: MethodSignature,
//    class: ClassSignature,
//    full_name: String
//}

impl Sampler {
    pub fn new() -> Sampler {
        Sampler {
            method_cache: HashMap::new(),
            running: false,
            sample_interval: 0,
            start_time:0,
            sender: None,
            receiver: None,
            threads_map: HashMap::new()
        }
    }

    pub fn start(&mut self) {
        if(!self.running) {
            self.running = true;
            self.start_time = Local::now().timestamp_millis();

            // 创建一个通道
            let (tx0, rx0): (mpsc::Sender<resp::Value>, mpsc::Receiver<resp::Value>) = mpsc::channel();
            let (tx1, rx1): (mpsc::Sender<resp::Value>, mpsc::Receiver<resp::Value>) = mpsc::channel();
            self.receiver = Some(rx0);
            self.sender = Some(tx1);

            get_server().lock().unwrap().set_options(tx0, rx1, self.start_time, self.sample_interval);
            //running server in new thread
            std::thread::spawn( move || {
                start_server();
            });
        }
    }

    pub fn stop(&mut self) {
        if(self.running){
            self.running = false;
            stop_server();
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn set_options(&mut self, sample_interval: u64) {
        self.sample_interval = sample_interval;
    }

    pub fn get_sample_interval(&self) -> u64 {
        self.sample_interval
    }

    pub fn get_start_time(&self) -> i64 {
        self.start_time
    }

    pub fn on_thread_start(&mut self, thread: ThreadId) {
        //self.threads.push(thread);
    }

    pub fn on_thread_end(&mut self, thread: &ThreadId) {
        //self.threads.remove_item(thread);
//        if let Some(pos) = self.threads.iter().position(|x| *x == *thread) {
//            self.threads.remove(pos);
//        }
    }

    pub fn add_stack_traces(&mut self, jvmenv: &Box<Environment>, stack_traces: &Vec<JavaStackTrace>) {
        //merge to call stack tree
        let now_time = Local::now().timestamp_millis();
        let mut sample_data_vec :Vec<Box<SampleData+Send>> = vec![];
        for (i, stack_info) in stack_traces.iter().enumerate() {
            let thread_info = &stack_info.thread;
            let mut is_new = false;
            let mut thread_data = self.threads_map.entry(thread_info.thread_id).or_insert_with(||{
                is_new = true;
                let mut thd = ThreadData::new(thread_info.thread_id, thread_info.name.clone());
                thd.priority = thread_info.priority;
                thd.daemon = thread_info.is_daemon;
                thd.cpu_time = stack_info.cpu_time;
                thd
            });

            let mut top_stack_frame = 0i64;
            let stack_len = stack_info.frame_buffer.len();
            if !is_new {
                //ignore inactive thread
                if thread_data.cpu_time == stack_info.cpu_time {
                    //check last frame
                    if stack_len > 0 {
                        top_stack_frame = (stack_info.frame_buffer[0].method as i64);
                        if thread_data.last_stack_frame == top_stack_frame && thread_data.last_stack_len == stack_len {
                            continue;
                        }
                    }else {
                        continue;
                    }
                }
            }

            //update sample time
            thread_data.cpu_time = stack_info.cpu_time;
            thread_data.sample_time = now_time;
            //save last frame
            thread_data.last_stack_frame = top_stack_frame;
            thread_data.last_stack_len = stack_len;

            //clone thread_data and push to sample data queue
            let mut thread_data = thread_data.clone();
            //translate method call
            for stack_frame in &stack_info.frame_buffer {
                let method_info = self.get_method_info(jvmenv, stack_frame.method);
                if method_info.hits_count == 1 {
                    sample_data_vec.push(Box::new(method_info.clone()));
                }
                thread_data.stacktrace.push(method_info.method_id);
            }

            sample_data_vec.push(Box::new(thread_data));
        }

        //batch add into sending queue
        add_sample_data_batch(sample_data_vec);
    }

    fn get_method_info(&mut self, jvm_env: &Box<Environment>, method: JavaMethod) -> &MethodData {
        let method_data = self.method_cache.entry(method as usize).or_insert_with(|| {
            let method_id = MethodId { native_id: method };
            let method_sig = jvm_env.get_method_name(&method_id).unwrap();
            let class_id = jvm_env.get_method_declaring_class(&method_id).unwrap();
            let class = jvm_env.get_class_signature(&class_id).unwrap();
            let full_name =  format!("{}.{}()", class.name, method_sig.name);
            MethodData{
                method_id: method as i64,
                full_name,
                hits_count: 0
            }
        });
        method_data.hits_count += 1;
        method_data
    }

    pub fn handle_request(&mut self) {
        if let Some(rx) = &self.receiver {
            if let Ok(resp::Value::Array(vec)) = rx.try_recv() {
                let first = &vec[0];
                match first {
                    resp::Value::String(s) => {
                        let cmd_options = parse_request_options(&vec);
                        self.dispatch_request(s, &cmd_options);
                    },
                    _ => {
                        println!("invalid request array, first element must be String, but get {:?}", first);
                    }
                }
            }
        }
    }

    fn dispatch_request(&mut self, cmd: &String, options: &HashMap<String, resp::Value>) {
        match cmd.as_str() {
            "get_sample_info" => {
                self.send_sample_info();
            }
            "get_method_cache" => {
                self.send_method_cache();
            }
            _ => { println!("unknown request cmd: {}, options: {:?}", cmd, options); }
        }
    }

    fn send_sample_info(&mut self) {
        let response = resp_encode_sample_info(self.start_time, self.sample_interval);
        //add_sample_data(ResponseData::new("sample_info".to_string(),response));
        Sampler::send_response(&self.sender, response);
    }

    fn send_response(sender: &Option<mpsc::Sender<resp::Value>>, response: resp::Value) {
        if let Some(tx) = sender {
            tx.send(response);
        }
    }

    fn send_method_cache(&mut self) {
        let sender = &self.sender;
        self.method_cache.values().for_each(|method_info| {
            //add_sample_data(Box::new(method_info.clone()));
            Sampler::send_response(sender, resp_encode_method_data(method_info));
        });
    }

//    pub fn add_stack_traces_to_call_tree(&mut self, jvm_env: &Box<Environment>, stack_traces: &Vec<JavaStackTrace>) {
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
//    pub fn write_all_call_trees(&self, writer: &mut std::io::Write, compact: bool) {
//        for (thread_id, call_tree) in self.tree_arena.get_all_call_trees() {
//            let tree_name = &call_tree.get_root_node().data.name;
//            writer.write_fmt(format_args!("Thread: {}, {}, {}\n", &call_tree.thread_id, tree_name, call_tree.total_duration as f64/1000_000.0));
//
//            writer.write_all(call_tree.format_call_tree(compact).as_bytes());
//            writer.write_all("\n".as_bytes());
//        }
//    }

}


pub struct SampleQueue {
    pub queue: VecDeque<Box<dyn SampleData + Send>>,
    total_count: usize,
    last_count: usize,
    last_time: i64
}

//pub struct SampleStats {
//
//}

impl SampleQueue {
    pub fn new() -> SampleQueue {
        SampleQueue {
            queue: VecDeque::with_capacity(512),
            total_count: 0,
            last_count: 0,
            last_time: 0
        }
    }

    pub fn push_back(&mut self, data_vec: Vec<Box<SampleData + Send>>) {
        self.total_count += data_vec.len();
        for sample_data in data_vec {
            self.queue.push_back(sample_data);
        }
        while(self.queue.len() > 10000){
            self.queue.pop_front();
        }
    }

    pub fn pop_front(&mut self) -> Option<Box<SampleData + Send>> {
        self.queue.pop_front()
    }

    pub fn stats(&mut self) {
        let now_time = Local::now().timestamp_millis();
        if self.last_time > 0 {
            let delta = self.total_count-self.last_count;
            let rate = delta as f64 * 1000.0 / (now_time-self.last_time) as f64;
            println!("sample queue stats: new samples={}, rate={:.2}/s, total={}, queue={}", delta, rate, self.total_count, self.queue.len());
        }
        self.last_count = self.total_count;
        self.last_time = now_time;
    }

}