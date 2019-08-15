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

use agent::Agent;
use bytecode::printer::ClassfilePrinter;
use bytecode::classfile::Constant;
use bytecode::io::ClassWriter;
use config::Config;
use context::static_context;
use instrumentation::asm::transformer::Transformer;
use native::{JavaVMPtr, MutString, VoidPtr, ReturnValue};
use options::Options;
use runtime::*;
use std::io::{Cursor, Write};
use thread::Thread;
use util::stringify;
use std::time::*;
extern crate chrono;
use chrono::Local;
use std::sync::{Mutex,Arc,RwLock};
use time::{Duration,Tm};
use environment::jvm::{JVMF, JVMAgent};
use environment::jvmti::{JVMTI, JVMTIEnvironment};
use profile::sample::*;
use environment::Environment;
use environment::jni::JNIEnvironment;
use std::path::Path;

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

fn is_trace_enable() -> bool {
//    *TRACE_ENABLE.lock().unwrap()
    SAMPLER.lock().unwrap().is_enable()
}

fn set_trace_enable(enable:bool) {
    static_context().set_trace_enable(enable);
//    let mut trace_enable = TRACE_ENABLE.lock().unwrap();
//    *trace_enable =  enable;
    SAMPLER.lock().unwrap().set_enable(enable);
}

fn nowTime() -> String {
    let date = Local::now();
    return date.format("%Y-%m-%d %H:%M:%S.%6f").to_string();
    //println!("{:?} {}", date, date.format("[%Y-%m-%d %H:%M:%S.%3f]"));
}

fn on_method_entry(event: MethodInvocationEvent) {
    if !is_trace_enable() {
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
    if !is_trace_enable() {
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
    if !is_trace_enable() {
        return;
    }
    println!("[{}] thread start [{}] [{}]", nowTime(), thread.id, thread.name);

    static_context().thread_start(&thread.id);
}

fn on_thread_end(thread: Thread) {
    if !is_trace_enable() {
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
    if !is_trace_enable() {
        return;
    }
    println!("[{}] [W1-{}]", nowTime(), thread.name);
}

fn on_monitor_waited(thread: Thread) {
    if !is_trace_enable() {
        return;
    }
    println!("[{}] [W2-{}]", nowTime(), thread.name);
}

fn on_monitor_contended_enter(thread: Thread) {
    if !is_trace_enable() {
        return;
    }
    println!("[{}] [C1-{}]", nowTime(), thread.name);

    static_context().monitor_enter(&thread.id);
}

fn on_monitor_contended_entered(thread: Thread) {
    if !is_trace_enable() {
        return;
    }
    println!("[{}] [C2-{}]", nowTime(), thread.name);

    match static_context().monitor_entered(&thread.id) {
        Some(duration) => println!("[{}] Thread {} waited {}", nowTime(), thread.name, duration),
        None => println!("[{}] Thread {} has never waited", nowTime(), thread.name)
    }
}

fn on_class_file_load(mut event: ClassFileLoadEvent) -> Option<Vec<u8>> {
    if !is_trace_enable() { return None; }
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
    if !is_trace_enable() {
        return;
    }
    println!("[{}] GC Start: {:?}", nowTime(), std::time::Instant::now());
}

fn on_garbage_collection_finish() {
    if !is_trace_enable() {
        return;
    }
    println!("[{}] GC Finish: {:?}", nowTime(), std::time::Instant::now());
}

fn on_object_alloc(event: ObjectAllocationEvent) {
    if !is_trace_enable() {
        return;
    }
    println!("[{}] [{}] Object allocation: (size: {})", nowTime(), event.thread.name, event.size);
}

fn on_object_free() {
    if !is_trace_enable() {
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

    set_trace_enable(false);

    let mut agent = Agent::new(vm);
    init_agent(&mut agent);

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



    if let Some(val) = options.custom_args.get("trace") {
        match val.as_ref() {
            "on" => {
                println!("Starting JVMTI agent ..");
                if(is_trace_enable()){
                    println!("Trace agent already running, do nothing.");
                    return 0;
                }

//                let mut agent = Agent::new(vm);
//                init_agent(&mut agent);
//                let jvmti = &agent.environment;
//                let caps = jvmti.get_capabilities();
//                println!("caps: {}", caps);
//                jvmti.get_all_stacktraces();

                let vm_ptr = vm as usize;
                //TODO how to pass vm or agent to thread safely?
                let handle = std::thread::spawn( move||{
                    println!("Trace agent is running ...");
                    let vm = vm_ptr as JavaVMPtr;
                    println!("create agent ..");
                    let mut agent = Agent::new_attach(vm, "Flare-Profiler");
                    println!("init_agent ..");
                    init_agent(&mut agent);
                    let jvmti = &agent.jvm_env;

                    set_trace_enable(true);
                    let mut samples=0;
                    while is_trace_enable() {
                        samples += 1;
//                        println!("[{}] get sample: {}", nowTime(), samples);
                        let t0 = time::now();
                        match jvmti.get_all_stacktraces() {
                            Ok(stack_traces) => {
                                let t1 = time::now();
//                                let output = SAMPLER.lock().unwrap().format_stack_traces(jvmti, &stack_traces);
                                SAMPLER.lock().unwrap().add_stack_traces(jvmti, &stack_traces);
                                let t2 = time::now();

//                                println!("jvmti get all stack traces, size: {}, cost: {}ms", stack_traces.len(),  (t1-t0).num_microseconds().unwrap() as f64 / 1000.0);
//                                println!("process all stack traces, cost: {}ms", (t2-t1).num_microseconds().unwrap() as f64 / 1000.0);
//                                println!("---------------------------------------");
                            },
                            Err(e) => {
                                println!("get all stack traces failed, error: {:?}", e);
                            }
                        }

                        if samples % 250 == 0 {
                            let t4 = time::now();
                            let file_path = Path::new("flare-data.txt");
                            println!("[{}] writing to file: {}", nowTime(), file_path.display());
                            let mut file = std::fs::File::create(file_path).expect("create failed");
                            //file.write_all(&output.as_bytes()).expect("write failed");
                            SAMPLER.lock().unwrap().write_all_call_trees(&mut file, true);
                            let t5 = time::now();
                            println!("[{}] print all stack traces, cost: {}ms", nowTime(), (t5-t4).num_microseconds().unwrap() as f64 / 1000.0);
                        }

                        std::thread::sleep(std::time::Duration::from_millis(20));
                    }
                    set_trace_enable(false);
                    println!("Trace agent is stopped.");
                });
            },
            _ => {
                println!("Shutting down JVMTI agent ..");
                set_trace_enable(false);

                //TREE_ARENA.lock().unwrap().print_all();
                //TREE_ARENA.lock().unwrap().clear();
            }
        }

        println!("Attach thread is exited.");
    }

    return 0;
}

fn init_agent(agent: &mut Agent) {
    agent.capabilities.can_get_thread_cpu_time = true;
    agent.capabilities.can_get_current_thread_cpu_time = true;
    agent.capabilities.can_access_local_variables = true;
    agent.capabilities.can_get_line_numbers = true;
    agent.capabilities.can_get_source_file_name = true;
    agent.capabilities.can_generate_all_class_hook_events = true;
    agent.capabilities.can_get_bytecodes = true;

    agent.on_garbage_collection_start(Some(on_garbage_collection_start));
    agent.on_garbage_collection_finish(Some(on_garbage_collection_finish));
    //agent.on_vm_object_alloc(Some(on_object_alloc));
    //agent.on_vm_object_free(Some(on_object_free));
    //agent.on_class_file_load(Some(on_class_file_load));
//    agent.on_method_entry(Some(on_method_entry));
//    agent.on_method_exit(Some(on_method_exit));
    agent.on_thread_start(Some(on_thread_start));
    agent.on_thread_end(Some(on_thread_end));
    agent.on_monitor_wait(Some(on_monitor_wait));
    agent.on_monitor_waited(Some(on_monitor_waited));
    agent.on_monitor_contended_enter(Some(on_monitor_contended_enter));
    agent.on_monitor_contended_entered(Some(on_monitor_contended_entered));
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
