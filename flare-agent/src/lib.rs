//#![feature(ptr_cast)]

extern crate libc;
#[macro_use]
extern crate lazy_static;
extern crate time;
extern crate toml;
#[macro_use]
extern crate serde_derive;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate serde_json;
extern crate serde;
//extern crate jni;
//extern crate jvmti_sys;
extern crate resp;
extern crate timer;
extern crate chrono;


pub mod agent;
pub mod bytecode;
pub mod capabilities;
pub mod class;
pub mod config;
pub mod context;
//pub mod emulator;
pub mod environment;
pub mod error;
pub mod event;
pub mod event_handler;
pub mod instrumentation;
pub mod mem;
pub mod method;
pub mod native;
pub mod options;
pub mod runtime;
pub mod thread;
pub mod util;
pub mod version;
mod profile;

use agent::Agent;
use bytecode::printer::ClassfilePrinter;
use bytecode::classfile::Constant;
use bytecode::io::ClassWriter;
use config::Config;
use context::static_context;
use instrumentation::asm::transformer::Transformer;
use native::{JavaVMPtr, MutString, VoidPtr, ReturnValue, JavaLong};
use options::Options;
use runtime::*;
use std::io::{Cursor, Write};
use thread::Thread;
use util::stringify;
use std::time::*;
use chrono::Local;
use std::sync::{Mutex,Arc,RwLock};
use time::{Duration,Tm};
use environment::jvm::{JVMF, JVMAgent};
use environment::jvmti::{JVMTI, JVMTIEnvironment, JavaStackTrace, ThreadInfo};
use profile::sample::*;
use environment::Environment;
use environment::jni::JNIEnvironment;
use std::path::Path;
use error::NativeError;
use std::collections::HashMap;
use std::cmp::max;



/*
 * TODO The functions below are essentially parts of an actual client implementation. Because this
 * implementation is highly experimental and incomplete they shall remain here for a while but
 * they will have to find a new home, eventually
 */

lazy_static! {
    //static ref TREE_ARENA: Mutex<TreeArena> = Mutex::new(TreeArena::new());
    //static ref TRACE_ENABLE: Mutex<bool> = Mutex::new(false);
    static ref SAMPLER: Mutex<Sampler> = Mutex::new(Sampler::new());
}

static mut TRACE_RUNNING: bool = false;


fn is_trace_running() -> bool {
    //avoid dead lock in gc event callback function
    //SAMPLER.lock().unwrap().is_running()
    unsafe { TRACE_RUNNING }
}

fn start_trace(sample_interval: u64) {
    unsafe {
        TRACE_RUNNING = true;
    }
    static_context().set_trace_enable(true);
    SAMPLER.lock().unwrap().set_options(sample_interval);
    SAMPLER.lock().unwrap().start();
}

fn stop_trace() {
    unsafe  {
        TRACE_RUNNING = false;
    }
    static_context().set_trace_enable(false);
    SAMPLER.lock().unwrap().stop();
}

fn nowTime() -> String {
    let date = Local::now();
    return date.format("%Y-%m-%d %H:%M:%S.%6f").to_string();
    //println!("{:?} {}", date, date.format("[%Y-%m-%d %H:%M:%S.%3f]"));
}

fn on_method_entry(event: MethodInvocationEvent) {
    if !is_trace_running() {
        return;
    }
    let shall_record = match static_context().config.read() {
        Ok(cfg) => (*cfg).entry_points.iter().any(|item| *item == format!("{}.{}", event.class_sig.name, event.method_sig.name) ), //event.class_name.as_str() == item),
        _ => false
    };

    if !shall_record {
        //TREE_ARENA.lock().unwrap().begin_call(&event.thread, &event.class_sig.name, &event.method_sig.name);
        debug!("[{}] [{}] method_entry [{}.{}]", nowTime(), event.thread.name, event.class_sig.name, event.method_sig.name);
    }

//    static_context().method_enter(&event.thread.id);
}

fn on_method_exit(event: MethodInvocationEvent) {
    if !is_trace_running() {
        return;
    }
    match static_context().method_exit(&event.thread.id) {
        //Some(_) => (),
        Some(duration) => {
            //TREE_ARENA.lock().unwrap().end_call(&event.thread, &event.class_sig.name, &event.method_sig.name, &duration);
            debug!("[{}] [{}] method_exit [{}.{}] after {}", nowTime(), event.thread.name, event.class_sig.name, event.method_sig.name, duration)
        },
        None => {
            //TREE_ARENA.lock().unwrap().end_call(&event.thread, &event.class_sig.name, &event.method_sig.name, &Duration::microseconds(0));
            debug!("[{}] [{}] method_no_start [{}.{}]", nowTime(), event.thread.name, event.class_sig.name, event.method_sig.name)
        }
    }
}

fn on_thread_start(thread: Thread) {
    if !is_trace_running() {
        return;
    }
    println!("[{}] thread start [{}] [{}]", nowTime(), thread.id, thread.name);

    static_context().thread_start(&thread.id);
}

fn on_thread_end(thread: Thread) {
    if !is_trace_running() {
        return;
    }
    println!("[{}] thread end [{}] [{}]", nowTime(), thread.id, thread.name);

    match static_context().thread_end(&thread.id) {
        Some(duration) => {
            println!("[{}] Thread {} lived {}", nowTime(), thread.name, duration);
            //TREE_ARENA.lock().unwrap().print_call_tree(&thread);
        },
        None => println!("[{}] Thread {} has no start", nowTime(), thread.name)
    }
}

fn on_monitor_wait(thread: Thread) {
    if !is_trace_running() {
        return;
    }
    println!("[{}] [W1-{}]", nowTime(), thread.name);
}

fn on_monitor_waited(thread: Thread) {
    if !is_trace_running() {
        return;
    }
    println!("[{}] [W2-{}]", nowTime(), thread.name);
}

fn on_monitor_contended_enter(thread: Thread) {
    if !is_trace_running() {
        return;
    }
    println!("[{}] [C1-{}]", nowTime(), thread.name);

    static_context().monitor_enter(&thread.id);
}

fn on_monitor_contended_entered(thread: Thread) {
    if !is_trace_running() {
        return;
    }
    println!("[{}] [C2-{}]", nowTime(), thread.name);

    match static_context().monitor_entered(&thread.id) {
        Some(duration) => println!("[{}] Thread {} waited {}", nowTime(), thread.name, duration),
        None => println!("[{}] Thread {} has never waited", nowTime(), thread.name)
    }
}

fn on_class_file_load(mut event: ClassFileLoadEvent) -> Option<Vec<u8>> {
    if !is_trace_running() { return None; }
    let shall_transform = match static_context().config.read() {
        Ok(cfg) => (*cfg).entry_points.iter().any(|item| item.starts_with(event.class_name.as_str())), //event.class_name.as_str() == item),
        _ => false
    };

    if shall_transform {
        {
            let mut transformer = Transformer::new(&mut event.class);
            let result = transformer.ensure_constant(Constant::Utf8(String::from("Cde").into_bytes()));

            println!("Result: {:?}", result);
        }
        let _: Vec<()> = ClassfilePrinter::render_lines(&event.class).iter().map(|line| println!("{}", line)).collect();
    }
/*
    let output_class: Vec<u8> = vec![];
    let mut write_cursor = Cursor::new(output_class);

    let mut new_class = event.class;

    new_class.constant_pool.constants = new_class.constant_pool.constants.into_iter().map(|constant| {
        match constant {
            Constant::Utf8(bytes) => String::from_utf8(bytes.clone()).map(|string| match string.as_str() {
                "Hello World" => Constant::Utf8(String::from("Lofasz").into_bytes()),
                _ => Constant::Utf8(string.into_bytes())
            }).unwrap_or(Constant::Utf8(bytes)),
            other @ _ => other
        }
    }).collect();

    let result = {
        let mut writer = ClassWriter::new(&mut write_cursor);
        writer.write_class(&new_class)
    };

    if let Ok(_) = result {
        Some(write_cursor.into_inner())
    } else {
        None
    }
    */
    None
}

fn on_garbage_collection_start() {
    if !is_trace_running() {
        return;
    }
    println!("[{}] GC Start: {:?}", nowTime(), std::time::Instant::now());
}

fn on_garbage_collection_finish() {
    if !is_trace_running() {
        return;
    }
    println!("[{}] GC Finish: {:?}", nowTime(), std::time::Instant::now());
}

fn on_object_alloc(event: ObjectAllocationEvent) {
    if !is_trace_running() {
        return;
    }
    println!("[{}] [{}] Object allocation: (size: {})", nowTime(), event.thread.name, event.size);
}

fn on_object_free() {
    if !is_trace_running() {
        return;
    }
    println!("[{}] Object free", nowTime());
}


///
/// `Agent_OnLoad` is the actual entry point of the agent code and it is called by the
/// Java Virtual Machine directly.
///
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern fn Agent_OnLoad(vm: JavaVMPtr, options: MutString, reserved: VoidPtr) -> ReturnValue {

    env_logger::init();

    let options = Options::parse(stringify(options));
    println!("Starting up as {}", options.agent_id);

    if let Some(config) = Config::read_config() {
        println!("Setting configuration");
        static_context().set_config(config);
    }

    let mut interval = 20;
    if let Some(str) = options.custom_args.get("interval") {
        interval = str.parse().unwrap();
    }

    let mut agent = Agent::new(vm);
    init_agent(&mut agent);
    start_trace(interval);

    return 0;
}

struct JavaVMPtrVo {
    vm: JavaVMPtr
}

unsafe impl Send for JavaVMPtrVo {
}

///
/// `Agent_OnAttach` is the actual entry point of the agent code and it is called by the
/// Java Virtual Machine directly.
/// -- Dynamic load java agent by VirtualMachine.loadAgentPath()
///
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern fn Agent_OnAttach(vm: JavaVMPtr, options: MutString, reserved: VoidPtr) -> ReturnValue {

    env_logger::try_init();

    let options = Options::parse(stringify(options));
    println!("Starting up as {}, options: {:?}", options.agent_id, options);

    if let Some(config) = Config::read_config() {
        println!("Setting configuration");
        static_context().set_config(config);
    }

//                let mut agent = Agent::new(vm);
//                init_agent(&mut agent);
//                let jvmti = &agent.environment;
//               let caps = jvmti.get_capabilities();
//                println!("caps: {}", caps);
//                jvmti.get_all_stacktraces();

    if let Some(val) = options.custom_args.get("trace") {
        match val.as_ref() {
            "on" => {
                println!("Starting JVMTI agent ..");
                if(is_trace_running()){
                    println!("Trace agent already running, do nothing.");
                    return 0;
                }

                let mut interval = 20;
                if let Some(str) = options.custom_args.get("interval") {
                    interval = str.parse().unwrap();
                }

                let vm_ptr = vm as usize;
                //TODO how to pass vm or agent to thread safely?
                let handle = std::thread::spawn( move||{
                    println!("Trace agent is running ...");
                    start_trace(interval);
                    let vm = vm_ptr as JavaVMPtr;
                    println!("create agent ..");
                    let mut agent = Agent::new_attach(vm, "Flare-Profiler");
                    println!("init_agent ..");
                    init_agent(&mut agent);
                    let jvmenv = &agent.jvm_env;

                    let mut samples=0i64;
                    let mut thread_info_map: HashMap<JavaLong, ThreadInfo> = HashMap::new();
                    let mut last_get_cpu_time = 0i64;
                    //let get_cpu_time_per_samples = max(1, 50/interval);
                    while is_trace_running() {
                        samples += 1;
                        let t0 = Local::now().timestamp_millis();
                        let update_cpu_time = (t0 - last_get_cpu_time) > 50;
                        if update_cpu_time {
                            last_get_cpu_time = t0;
                        }
                        match jvmenv.get_all_stacktraces() {
//                        match get_stack_traces(jvmenv, &mut thread_info_map, update_cpu_time) {
                            Ok(stack_traces) => {
                                let t1 = time::now();
                                SAMPLER.lock().unwrap().add_stack_traces(jvmenv, &stack_traces);
                                let t2 = time::now();
                            },
                            Err(e) => {
                                println!("get all stack traces failed, error: {:?}", e);
                            }
                        }

                        //process client request
                        SAMPLER.lock().unwrap().handle_request();

                        //sample interval
                        std::thread::sleep(std::time::Duration::from_millis(interval));

                        //TODO auto close after exceed max idle time

                    }
                    stop_trace();
                    println!("Trace agent is stopped.");
                });
            },
            _ => {
                println!("Shutting down JVMTI agent ..");
                stop_trace();
            }
        }

        println!("Attach thread is exited.");
    }

    return 0;
}

fn get_stack_traces(jvmenv: &Box<Environment>, thread_info_map: &mut HashMap<JavaLong, ThreadInfo>, update_cpu_time: bool) -> Result<Vec<JavaStackTrace>, NativeError> {
    let mut stack_traces = vec![];
    match jvmenv.get_all_threads() {
        Err(e) => {
            println!("get_all_threads failed: {:?}", e);
            Err(e)
        }
        Ok(threads) => {
            //println!("get_all_threads: {:?}", threads);
            for thread in threads {
                let java_thread_id = jvmenv.get_thread_id(&thread.native_id);
                let mut thread_info = thread_info_map.get_mut(&java_thread_id);
                let mut is_new_thread = false;
                if thread_info.is_none() {
                    is_new_thread = true;
                    let mut new_thread_info;
                    match jvmenv.get_thread_info_ex(&thread.native_id) {
                        Ok(v) => {
                            new_thread_info = v;
                        },
                        Err(e) => {
                            println!("get_thread_info_ex failed: {:?}, java_thread_id: {}", e, java_thread_id);
                            jvmenv.delete_local_ref(thread.native_id);
                            continue;
                        }
                    }
                    thread_info_map.insert(java_thread_id, new_thread_info);
                    thread_info = thread_info_map.get_mut(&java_thread_id);
                }
                let thread_info = thread_info.unwrap();

                //update thread cpu time discontinuous
                let mut cpu_time = thread_info.cpu_time;
                if update_cpu_time || is_new_thread {
                    match jvmenv.get_thread_cpu_time(&thread.native_id) {
                        Ok(t) => { cpu_time = t; },
                        Err(e) => {
                            println!("get_thread_cpu_time failed: {:?}, thread: {:?}", e, thread);
                        },
                    }
                    thread_info.cpu_time = cpu_time;
                }

                let mut frames = vec![];
                match jvmenv.get_stack_trace(&thread.native_id) {
                    Ok(v) => { frames = v; },
                    Err(e) => {
                        println!("get_stack_trace failed: {:?}, thread: {:?}", e, thread);
                    },
                }

                //检查是否为新线程
                jvmenv.delete_local_ref(thread.native_id);

                let mut stack_trace = JavaStackTrace {
                    thread: thread_info.clone(),
                    state: 0,
                    cpu_time,
                    frame_buffer: frames
                };
                stack_traces.push(stack_trace);
            }
            Ok(stack_traces)
        },
    }
}

fn init_agent(agent: &mut Agent) {
    agent.capabilities.can_get_thread_cpu_time = true;
    agent.capabilities.can_get_current_thread_cpu_time = true;
    agent.capabilities.can_access_local_variables = true;
    agent.capabilities.can_get_line_numbers = true;
    agent.capabilities.can_get_source_file_name = true;
    agent.capabilities.can_generate_all_class_hook_events = true;
    agent.capabilities.can_get_bytecodes = true;

//    agent.on_garbage_collection_start(Some(on_garbage_collection_start));
//    agent.on_garbage_collection_finish(Some(on_garbage_collection_finish));
    //agent.on_vm_object_alloc(Some(on_object_alloc));
    //agent.on_vm_object_free(Some(on_object_free));
    //agent.on_class_file_load(Some(on_class_file_load));
//    agent.on_method_entry(Some(on_method_entry));
//    agent.on_method_exit(Some(on_method_exit));
//    agent.on_thread_start(Some(on_thread_start));
//    agent.on_thread_end(Some(on_thread_end));
//    agent.on_monitor_wait(Some(on_monitor_wait));
//    agent.on_monitor_waited(Some(on_monitor_waited));
//    agent.on_monitor_contended_enter(Some(on_monitor_contended_enter));
//    agent.on_monitor_contended_entered(Some(on_monitor_contended_entered));
    agent.update();
}


///
/// `Agent_OnUnload` is the exit point of the agent code. It is called when the JVM has finished
/// running and the virtual machine is unloading the agent from memory before shutting down.
/// Note: this method is also called when the JVM crashes due to an internal error.
///
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern fn Agent_OnUnload(vm: JavaVMPtr) {
    //TREE_ARENA.lock().unwrap().print_all();
}
