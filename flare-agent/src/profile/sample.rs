
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
//use std::sync::mpsc::{Sender, Receiver};

#[derive(Serialize, Deserialize)]
pub struct SampleResult {
    sample_time: i64, //ms
    cpu_time: f64, //ms
    thread_count: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ThreadData {
    pub id: JavaLong,
    pub name: String,
    pub priority: u32,
    pub daemon: bool,
    pub state: String,
    pub cpu_time: i64,
    pub sample_time: i64,
    pub stacktrace: Vec<String>
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

pub struct Sampler {
    method_cache: HashMap<usize, MethodInfo>,
    running: bool,
    threads_map: HashMap<JavaLong, ThreadData>
}

pub struct MethodInfo {
    method_id: MethodId,
    method: MethodSignature,
    class: ClassSignature,
    full_name: String
}

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
                //translate method call
                for stack_frame in &stack_info.frame_buffer {
                    let method_info = self.get_method_info(jvm_env, stack_frame.method);
                    thread_data.stacktrace.push(method_info.full_name.clone());
                }

                //add into sending queue
                add_thread_data(thread_data);

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
                result.push_str(&format!("{}.{}()\n", &method_info.class.name, &method_info.method.name));
            }
        }
        result
    }

    fn get_method_info(&mut self, jvm_env: &Box<Environment>, method: JavaMethod) -> &MethodInfo {
        self.method_cache.entry(method as usize).or_insert_with(|| {
            let method_id = MethodId { native_id: method };
            let method = jvm_env.get_method_name(&method_id).unwrap();
            let class_id = jvm_env.get_method_declaring_class(&method_id).unwrap();
            let class = jvm_env.get_class_signature(&class_id).unwrap();
            let full_name =  format!("{}.{}()", class.name, method.name);
            MethodInfo {
                method_id: method_id,
                method,
                class,
                full_name: full_name
            }
        })
        //self.method_cache.get(&method_id).unwrap()
    }
}
