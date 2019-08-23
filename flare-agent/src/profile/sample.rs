
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
use std::sync::Mutex;
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
    pub id: JavaLong,
    pub name: String,
    pub priority: u32,
    pub daemon: bool,
    pub state: String,
    pub cpu_time: i64,
    pub sample_time: i64,
    pub stacktrace: Vec<i64>
}

impl ThreadData {
    pub fn new(id: JavaLong, name: String) -> ThreadData {
        ThreadData {
            id: id,
            name: name,
            priority: 0,
            daemon: false,
            state: "".to_string(),
            cpu_time: 0,
            sample_time: 0,
            stacktrace: vec![]
        }
    }
}

impl SampleData for ThreadData {

    fn encode(&self) -> Vec<u8> {
        resp_encode_thread_data(self)
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
        resp_encode_method_data(self)
    }

    fn get_type(&self) -> String {
        "method".to_string()
    }
}


pub struct Sampler {
    method_cache: HashMap<usize, MethodData>,
    running: bool,
    threads_map: HashMap<JavaLong, ThreadData>
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
            threads_map: HashMap::new()
        }
    }

    pub fn start(&mut self) {
        if(!self.running) {
            self.running = true;
            //running server in new thread
            std::thread::spawn( move|| {
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

    pub fn on_thread_start(&mut self, thread: ThreadId) {
        //self.threads.push(thread);
    }

    pub fn on_thread_end(&mut self, thread: &ThreadId) {
        //self.threads.remove_item(thread);
//        if let Some(pos) = self.threads.iter().position(|x| *x == *thread) {
//            self.threads.remove(pos);
//        }
    }

    pub fn add_stack_traces(&mut self, jvm_env: &Box<Environment>, stack_traces: &Vec<JavaStackTrace>) {
        //merge to call stack tree
        let now_time = Local::now().timestamp_millis();
        for (i, stack_info) in stack_traces.iter().enumerate() {
            if let Ok(thread_info) = jvm_env.get_thread_info(&stack_info.thread) {
                let mut cpu_time: i64 = 0_i64;
                //if std::time::Instant::now()
                if let Ok(t) = jvm_env.get_thread_cpu_time(&stack_info.thread) {
                    cpu_time = t;
                    //ignore inactive thread call
                    if cpu_time == 0_i64 {
                        continue;
                    }
                } else {
                    println!("get_thread_cpu_time error");
                }

                //TODO check and add stacktrace to sending queue
                let mut is_new = false;
                let mut thread_data = self.threads_map.entry(thread_info.thread_id).or_insert_with(||{
                    is_new = true;
                    let mut thd = ThreadData::new(thread_info.thread_id, thread_info.name);
                    thd.priority = thread_info.priority;
                    thd.daemon = thread_info.is_daemon;
                    thd.cpu_time = cpu_time;
                    thd
                });

                //ignore inactive thread
                if !is_new && thread_data.cpu_time == cpu_time {
                    continue;
                }
                //update thread cpu time
                thread_data.cpu_time = cpu_time;
                thread_data.sample_time = now_time;

                let mut thread_data = thread_data.clone();
                let mut sample_data_vec :Vec<Box<SampleData+Send>> = vec![];
                //translate method call
                for stack_frame in &stack_info.frame_buffer {
                    let method_info = self.get_method_info(jvm_env, stack_frame.method);
                    if method_info.hits_count == 1 {
                        sample_data_vec.push(Box::new(method_info.clone()));
                    }
                    thread_data.stacktrace.push(method_info.method_id);
                }

                sample_data_vec.push(Box::new(thread_data));
                //batch add into sending queue
                add_sample_data_batch(sample_data_vec);

            }else {
                //warn!("Thread UNKNOWN [{:?}]: (cpu_time = {})", stack_info.thread, cpu_time);
            }
        }
    }

    pub fn format_stack_traces(&mut self, jvm_env: &Box<Environment>, stack_traces: &Vec<JavaStackTrace>) -> String {
        let mut result  = String::new();
        for (i, stack_info) in stack_traces.iter().enumerate() {
            result.push_str(&format!("\nstack_info: {}, thread: {:?}, state: {:?}\n", (i+1), stack_info.thread, stack_info.state));

            let mut cpu_time = -1f64;
            match jvm_env.get_thread_cpu_time(&stack_info.thread) {
                Ok(t) => { cpu_time = t as f64 / 1000_000.0 },
                Err(err) => {
                    result.push_str(&format!("get_thread_cpu_time error: {:?}\n", err))
                }
            }

            if let Ok(thread_info) = jvm_env.get_thread_info(&stack_info.thread) {
                result.push_str(&format!("Thread {}: (id = {}, priority = {}, daemon = {}, state = {:?}, cpu_time = {}) \n",
                                         thread_info.name, thread_info.thread_id,  thread_info.priority, thread_info.is_daemon, stack_info.state, cpu_time ));
            } else {
                result.push_str(&format!("Thread UNKNOWN [{:?}]: (cpu_time = {}) \n", stack_info.thread, cpu_time));
            }

            for stack_frame in &stack_info.frame_buffer {
                let method_info = self.get_method_info(jvm_env, stack_frame.method);
                result.push_str(&format!("{}\n", &method_info.full_name));
            }
        }
        result
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
    pub queue: VecDeque<Box<dyn SampleData + Send>>
}

impl SampleQueue {
    pub fn new() -> SampleQueue {
        SampleQueue {
            queue: VecDeque::with_capacity(512)
        }
    }
}