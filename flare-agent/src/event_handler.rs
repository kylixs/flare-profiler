use super::environment::Environment;
use super::environment::jni::{JNI, JNIEnvironment};
use super::environment::jvmti::{JVMTI, JVMTIEnvironment};
use super::error::{translate_error, NativeError};
use super::event::*;
use super::method::MethodId;
use super::native::*;
use super::native::jvmti_native::*;
use super::runtime::*;
use libc::{c_char, c_uchar, c_void};
use std::mem::size_of;
use std::ptr;
use super::util::stringify;
use super::bytecode::*;
use std::io::{ Cursor };

pub static mut CALLBACK_TABLE: EventCallbacks = EventCallbacks {
    vm_init: None,
    vm_death: None,
    vm_object_alloc: None,
    vm_object_free: None,
    vm_start: None,
    method_entry: None,
    method_exit: None,
    exception: None,
    exception_catch: None,
    monitor_wait: None,
    monitor_waited: None,
    monitor_contended_enter: None,
    monitor_contended_entered: None,
    thread_start: None,
    thread_end: None,
    field_access: None,
    field_modification: None,
    garbage_collection_start: None,
    garbage_collection_finish: None,
    class_file_load_hook: None,
    class_load: None,
    class_prepare: None,
    single_step: None,
    frame_pop: None,
    breakpoint: None,
    native_method_bind: None,
    compiled_method_load: None,
    compiled_method_unload: None,
    dynamic_code_generated: None,
    data_dump_request: None,
    resource_exhausted: None
};

pub fn register_vm_init_callback(callback: Option<FnVMInit>) {
    unsafe { CALLBACK_TABLE.vm_init = callback; }
}

pub fn register_vm_death_callback(callback: Option<FnVMDeath>) {
    unsafe { CALLBACK_TABLE.vm_death = callback; }
}

pub fn register_vm_object_alloc_callback(callback: Option<FnVMObjectAlloc>) {
    unsafe { CALLBACK_TABLE.vm_object_alloc = callback; }
}

pub fn register_vm_object_free_callback(callback: Option<FnVMObjectFree>) {
    unsafe { CALLBACK_TABLE.vm_object_free = callback; }
}

pub fn register_vm_start_callback(callback: Option<FnVMStart>) {
    unsafe { CALLBACK_TABLE.vm_start = callback; }
}

pub fn register_method_entry_callback(callback: Option<FnMethodEntry>) {
    unsafe { CALLBACK_TABLE.method_entry = callback; }
}

pub fn register_method_exit_callback(callback: Option<FnMethodExit>) {
    unsafe { CALLBACK_TABLE.method_exit = callback; }
}

pub fn register_exception_callback(callback: Option<FnException>) {
    unsafe { CALLBACK_TABLE.exception = callback; }
}

pub fn register_exception_catch_callback(callback: Option<FnExceptionCatch>) {
    unsafe { CALLBACK_TABLE.exception_catch = callback; }
}

pub fn register_monitor_wait_callback(callback: Option<FnMonitorWait>) {
    unsafe { CALLBACK_TABLE.monitor_wait = callback; }
}

pub fn register_monitor_waited_callback(callback: Option<FnMonitorWaited>) {
    unsafe { CALLBACK_TABLE.monitor_waited = callback; }
}

pub fn register_monitor_contended_enter_callback(callback: Option<FnMonitorContendedEnter>) {
    unsafe { CALLBACK_TABLE.monitor_contended_enter = callback; }
}

pub fn register_monitor_contended_endered_callback(callback: Option<FnMonitorContendedEntered>) {
    unsafe { CALLBACK_TABLE.monitor_contended_entered = callback; }
}

pub fn register_thread_start_callback(callback: Option<FnThreadStart>) {
    unsafe { CALLBACK_TABLE.thread_start = callback; }
}

pub fn register_thread_end_callback(callback: Option<FnThreadEnd>) {
    unsafe { CALLBACK_TABLE.thread_end = callback; }
}

pub fn register_field_access_callback(callback: Option<FnFieldAccess>) {
    unsafe { CALLBACK_TABLE.field_access = callback; }
}

pub fn register_field_modification_callback(callback: Option<FnFieldModification>) {
    unsafe { CALLBACK_TABLE.field_modification = callback; }
}

pub fn register_garbage_collection_start(callback: Option<FnGarbageCollectionStart>) {
    unsafe { CALLBACK_TABLE.garbage_collection_start = callback;}
}

pub fn register_garbage_collection_finish(callback: Option<FnGarbageCollectionFinish>) {
    unsafe { CALLBACK_TABLE.garbage_collection_finish = callback; }
}

pub fn register_class_file_load_hook(callback: Option<FnClassFileLoad>) {
    unsafe { CALLBACK_TABLE.class_file_load_hook = callback; }
}

pub fn registered_callbacks() -> (jvmtiEventCallbacks, i32) {
    (local_event_callbacks(), size_of::<jvmtiEventCallbacks>() as i32)
}

///
/// Generates a native `jvmtiEventCallbacks` structure holding the local extern even handler methods.
///
pub fn local_event_callbacks() -> jvmtiEventCallbacks {
    jvmtiEventCallbacks {
        VMInit: Some(local_cb_vm_init), //jvmtiEventVMInit,
        VMDeath: Some(local_cb_vm_death), //jvmtiEventVMDeath,
        ThreadStart: Some(local_cb_thread_start), //jvmtiEventThreadStart,
        ThreadEnd: Some(local_cb_thread_end), //jvmtiEventThreadEnd,
        //ClassFileLoadHook: Some(local_cb_class_file_load_hook), //jvmtiEventClassFileLoadHook,
        ClassFileLoadHook: None, //jvmtiEventClassFileLoadHook,
        ClassLoad: Some(local_cb_class_load), //jvmtiEventClassLoad,
        ClassPrepare: Some(local_cb_class_prepare), //jvmtiEventClassPrepare,
        VMStart: Some(local_cb_vm_start), //jvmtiEventVMStart,
        Exception: Some(local_cb_exception), //jvmtiEventException,
        ExceptionCatch: Some(local_cb_exception_catch), //jvmtiEventExceptionCatch,
        SingleStep: Some(local_cb_single_step), //jvmtiEventSingleStep,
        FramePop: Some(local_cb_frame_pop), //jvmtiEventFramePop,
        Breakpoint: Some(local_cb_breakpoint), //jvmtiEventBreakpoint,
        FieldAccess: Some(local_cb_field_access), //jvmtiEventFieldAccess,
        FieldModification: Some(local_cb_field_modification), //jvmtiEventFieldModification,
        MethodEntry: Some(local_cb_method_entry), //jvmtiEventMethodEntry,
        MethodExit: Some(local_cb_method_exit), //jvmtiEventMethodExit,
        NativeMethodBind: Some(local_cb_native_method_bind), //jvmtiEventNativeMethodBind,
        CompiledMethodLoad: Some(local_cb_compiled_method_load), //jvmtiEventCompiledMethodLoad,
        CompiledMethodUnload: Some(local_cb_compiled_method_unload), //jvmtiEventCompiledMethodUnload,
        DynamicCodeGenerated: Some(local_cb_dynamic_code_generated), //jvmtiEventDynamicCodeGenerated,
        DataDumpRequest: Some(local_cb_data_dump_request), //jvmtiEventDataDumpRequest,
        reserved72: None, //jvmtiEventReserved,
        MonitorWait: Some(local_cb_monitor_wait), //jvmtiEventMonitorWait,
        MonitorWaited: Some(local_cb_monitor_waited), //jvmtiEventMonitorWaited,
        MonitorContendedEnter: Some(local_cb_monitor_contended_enter), //jvmtiEventMonitorContendedEnter,
        MonitorContendedEntered: Some(local_cb_monitor_contended_entered), //jvmtiEventMonitorContendedEntered,
        reserved77: None, //jvmtiEventReserved,
        reserved78: None, //jvmtiEventReserved,
        reserved79: None, //jvmtiEventReserved,
        ResourceExhausted: Some(local_cb_resource_exhausted), //jvmtiEventResourceExhausted,
        GarbageCollectionStart: Some(local_cb_garbage_collection_start), //jvmtiEventGarbageCollectionStart,
        GarbageCollectionFinish: Some(local_cb_garbage_collection_finish), //jvmtiEventGarbageCollectionFinish,
        ObjectFree: Some(local_cb_object_free), //jvmtiEventObjectFree,
        VMObjectAlloc: Some(local_cb_vm_object_alloc) //jvmtiEventVMObjectAlloc,
    }
}


#[allow(unused_variables)]
unsafe extern "C" fn local_cb_vm_object_alloc(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: JavaThread, object: JavaObject, object_klass: JavaClass, size: jlong) -> () {
    match CALLBACK_TABLE.vm_object_alloc {
        Some(function) => {
            let env = Environment::new(JVMTIEnvironment::new(jvmti_env), JNIEnvironment::new(jni_env));
            match env.get_thread_info(&thread) {
                Ok(current_thread) => {
                    let class_id = env.get_object_class(&object);

                    function(ObjectAllocationEvent { class_id: class_id, size: size as i64, thread: current_thread })
                },
                Err(err) => {
                    match err {
                        NativeError::WrongPhase => { /* we're in the wrong phase, just ignore this */ },
                        _ => println!("Couldn't get thread info: {}", translate_error(&err))
                    }
                }
            }
        },
        None => println!("No dynamic callback method was found for VM object allocation")
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_method_entry(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: JavaThread, method: JavaMethod) -> () {
    match CALLBACK_TABLE.method_entry {
        Some(function) => {
            let env = Environment::new(JVMTIEnvironment::new(jvmti_env), JNIEnvironment::new(jni_env));
            match env.get_thread_info(&thread) {
                Ok(current_thread) => {
                    let method_id = MethodId { native_id : method };
                    let class_id = env.get_method_declaring_class(&method_id).ok().unwrap();
                    let class_sig = env.get_class_signature(&class_id).ok().unwrap();
                    let method_sig = env.get_method_name(&method_id).ok().unwrap();

                    function(MethodInvocationEvent { method_id: method_id, method_sig: method_sig, class_sig: class_sig, thread: current_thread })

                },
                Err(err) => {
                    match err {
                        NativeError::WrongPhase => { /* we're in the wrong phase, just ignore this */ },
                        _ => println!("Couldn't get thread info: {}", translate_error(&err))
                    }
                }
            }
        },
        None => println!("No dynamic callback method was found for method entry")
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_method_exit(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, method: jmethodID, was_popped_by_exception: jboolean, return_value: jvalue) -> () {
    match CALLBACK_TABLE.method_exit {
        Some(function) => {
            let env = Environment::new(JVMTIEnvironment::new(jvmti_env), JNIEnvironment::new(jni_env));
            match env.get_thread_info(&thread) {
                Ok(current_thread) => {
                    let method_id = MethodId { native_id : method };
                    let class_id = env.get_method_declaring_class(&method_id).ok().unwrap();
                    let class_sig = env.get_class_signature(&class_id).ok().unwrap();
                    let method_sig = env.get_method_name(&method_id).ok().unwrap();

                    function(MethodInvocationEvent { method_id: method_id, method_sig: method_sig, class_sig: class_sig, thread: current_thread })

                },
                Err(err) => {
                    match err {
                        NativeError::WrongPhase => { /* we're in the wrong phase, just ignore this */ },
                        _ => println!("Couldn't get thread info: {}", translate_error(&err))
                    }
                }
            }
        }
        None => println!("No dynamic callback method was found for method exit")
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_exception(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, method: jmethodID, location: jlocation, exception: JavaObject, catch_method: jmethodID, catch_location: jlocation) -> () {
    match CALLBACK_TABLE.exception {
        Some(function) => {
            function();
            let env = Environment::new(JVMTIEnvironment::new(jvmti_env), JNIEnvironment::new(jni_env));
            let exception_class = env.get_object_class(&exception);

            function()
        },
        None => println!("No dynamic callback method was found for exception")
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_exception_catch(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, method: jmethodID, location: jlocation, exception: jobject) -> () {
    match CALLBACK_TABLE.exception_catch {
        Some(function) => {
            function();
            /*
            let env = Environment::new(JVMTIEnvironment::new(jvmti_env), JNIEnvironment::new(jni_env));
            let current_thread = env.get_thread_info(&thread).ok().unwrap();

            function()
            */
        },
        None => println!("No dynamic callback method was found for exception catch")
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_monitor_wait(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, object: jobject, timeout: jlong) -> () {
    match CALLBACK_TABLE.monitor_wait {
        Some(function) => {
            let env = Environment::new(JVMTIEnvironment::new(jvmti_env), JNIEnvironment::new(jni_env));
            match env.get_thread_info(&thread) {
                Ok(current_thread) => function(current_thread),
                Err(err) => {
                    match err {
                        NativeError::WrongPhase => { /* we're in the wrong phase, just ignore this */ },
                        _ => println!("Couldn't get thread info: {}", translate_error(&err))
                    }
                }
            }
        },
        None => println!("No dynamic callback method was found for monitor wait")
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_monitor_waited(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, object: jobject, timed_out: jboolean) -> () {
    match CALLBACK_TABLE.monitor_waited {
        Some(function) => {
            let env = Environment::new(JVMTIEnvironment::new(jvmti_env), JNIEnvironment::new(jni_env));
            match env.get_thread_info(&thread) {
                Ok(current_thread) => function(current_thread),
                Err(err) => {
                    match err {
                        NativeError::WrongPhase => { /* we're in the wrong phase, just ignore this */ },
                        _ => println!("Couldn't get thread info: {}", translate_error(&err))
                    }
                }
            }
        },
        None => println!("No dynamic callback method was found for monitor entered")
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_monitor_contended_enter(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, object: jobject) -> () {
    match CALLBACK_TABLE.monitor_contended_enter {
        Some(function) => {
            let env = Environment::new(JVMTIEnvironment::new(jvmti_env), JNIEnvironment::new(jni_env));
            match env.get_thread_info(&thread) {
                Ok(current_thread) => function(current_thread),
                Err(err) => {
                    match err {
                        NativeError::WrongPhase => { /* we're in the wrong phase, just ignore this */ },
                        _ => println!("Couldn't get thread info: {}", translate_error(&err))
                    }
                }
            }
        },
        None => println!("No dynamic callback method was found for monitor contended enter")
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_monitor_contended_entered(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, object: jobject) -> () {
    match CALLBACK_TABLE.monitor_contended_entered {
        Some(function) => {
            let env = Environment::new(JVMTIEnvironment::new(jvmti_env), JNIEnvironment::new(jni_env));
            match env.get_thread_info(&thread) {
                Ok(current_thread) => function(current_thread),
                Err(err) => {
                    match err {
                        NativeError::WrongPhase => { /* we're in the wrong phase, just ignore this */ },
                        _ => println!("Couldn't get thread info: {}", translate_error(&err))
                    }
                }
            }
        },
        None => println!("No dynamic callback method was found for monitor contended entered")
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_thread_start(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread) -> () {
    match CALLBACK_TABLE.thread_start {
        Some(function) => {
            let env = Environment::new(JVMTIEnvironment::new(jvmti_env), JNIEnvironment::new(jni_env));
            match env.get_thread_info(&thread) {
                Ok(current_thread) => function(current_thread),
                Err(err) => {
                    match err {
                        NativeError::WrongPhase => { /* we're in the wrong phase, just ignore this */ },
                        _ => println!("Couldn't get thread info: {}", translate_error(&err))
                    }
                }
            }
        },
        None => println!("No dynamic callback method was found for thread start events")
    }

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_thread_end(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread) -> () {
    match CALLBACK_TABLE.thread_end {
        Some(function) => {
            let env = Environment::new(JVMTIEnvironment::new(jvmti_env), JNIEnvironment::new(jni_env));
            match env.get_thread_info(&thread) {
                Ok(current_thread) => function(current_thread),
                Err(err) => {
                    match err {
                        NativeError::WrongPhase => { /* wrong phase, just ignore this */ },
                        _ => println!("Couldn't get thread info: {}", translate_error(&err))
                    }
                }
            }
        },
        None => println!("No dynamic callback method was found for thread end events")
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_garbage_collection_start(jvmti_env: *mut jvmtiEnv) -> () {
    match CALLBACK_TABLE.garbage_collection_start {
        Some(function) => {
            function();

        },
        None => println!("No dynamic callback method was found for garbage collection start events")
    }

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_garbage_collection_finish(jvmti_env: *mut jvmtiEnv) -> () {
    match CALLBACK_TABLE.garbage_collection_finish {
        Some(function) => {
            function();

        },
        None => println!("No dynamic callback method was found for garbage collection finish events")
    }

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_breakpoint(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, method: jmethodID, location: jlocation) -> () {

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_class_file_load_hook(jvmti_env: JVMTIEnvPtr, jni_env: JNIEnvPtr, class_being_redefined: JavaClass, loader: JavaObject,
                                                   name: *const c_char, protection_domain: JavaObject, class_data_len: jint, class_data: *const c_uchar,
                                                   new_class_data_len: *mut jint, new_class_data: *mut *mut c_uchar) -> () {
    match CALLBACK_TABLE.class_file_load_hook {
        Some(function) => {
            let env = Environment::new(JVMTIEnvironment::new(jvmti_env), JNIEnvironment::new(jni_env));

            let mut raw_data: Vec<u8> = Vec::with_capacity(class_data_len as usize);
            let data_ptr = raw_data.as_mut_ptr();

            ptr::copy_nonoverlapping(class_data, data_ptr, class_data_len as usize);
            raw_data.set_len(class_data_len as usize);

            if let Ok(classfile) = parse_class(&raw_data) {
                match function(ClassFileLoadEvent { class_name: stringify(name), class: classfile }) {
                    Some(transformed) => {
                        println!("Transformed class {}", stringify(name));

                        match env.allocate(transformed.len()) {
                            Ok(allocation) => {
                                ptr::copy_nonoverlapping(transformed.as_ptr(), allocation.ptr, allocation.len);
                                *new_class_data_len = allocation.len as i32;
                                *new_class_data = allocation.ptr;
                            },
                            Err(err) => {
                                println!("Failed to allocate memory")
                            }
                        }
                    },
                    None => ()
                }

            } else {
                println!("Coult not parse class file");
            }


            println!("Loading class {} with length {}", stringify(name), class_data_len);
        },
        None => println!("No dynamic callback method was found for class file load events")
    }
}

fn parse_class(data: &Vec<u8>) -> Result<Classfile, ::std::io::Error> {
    let mut cursor = Cursor::new(data);

    //let class_result = ClassReader::read_class(&mut cursor);
    ClassReader::read_class(&mut cursor)

/*
    match class_result {
        Ok(classfile) => {
            let output_class: Vec<u8> = vec![];
            let mut write_cursor = Cursor::new(output_class);
            let mut writer = ClassWriter::new(&mut write_cursor);
            match writer.write_class(&classfile) {
                Ok(len) => (),
                Err(error) => ()
            }
        },
        Err(error) => ()
    }
    */
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_class_load(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, klass: jclass) -> () {

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_class_prepare(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, klass: jclass) -> () {

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_compiled_method_load(jvmti_env: *mut jvmtiEnv, method: jmethodID, code_size: jint, code_addr: *const c_void, map_length: jint,
                                                   map: *const jvmtiAddrLocationMap, compile_info: *const c_void) -> () {

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_compiled_method_unload(jvmti_env: *mut jvmtiEnv, method: jmethodID, code_addr: *const c_void) -> () {

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_data_dump_request(jvmti_env: *mut jvmtiEnv) -> () {

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_dynamic_code_generated(jvmti_env: *mut jvmtiEnv, name: *const c_char, address: *const c_void, length: jint) -> () {

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_field_access(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, method: jmethodID, location: jlocation,
                                                   field_klass: jclass, object: jobject, field: jfieldID) -> () {
    match CALLBACK_TABLE.field_access {
        Some(function) => {
            function();
        },
        None => println!("No dynamic callback method was found for field access events")
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_field_modification(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, method: jmethodID, location: jlocation,
                                                   field_klass: jclass, object: jobject, field: jfieldID, signature_type: c_char, new_value: jvalue) -> () {
    match CALLBACK_TABLE.field_modification {
        Some(function) => {
            function();
        },
        None => println!("No dynamic callback method was found for field modification events")
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_frame_pop(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, method: jmethodID, was_popped_by_exception: jboolean) -> () {

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_native_method_bind(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, method: jmethodID, address: *mut c_void,
                                                   new_address_ptr: *mut *mut c_void) -> () {

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_object_free(jvmti_env: *mut jvmtiEnv, tag: jlong) -> () {

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_resource_exhausted(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, flags: jint, reserved: *const c_void, description: *const c_char) -> () {

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_single_step(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, method: jmethodID, location: jlocation) -> () {

}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_vm_death(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv) -> () {

    match CALLBACK_TABLE.vm_death {
        Some(function) => {
            function();
        },
        None => println!("No dynamic callback method was found for VM death events")
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_vm_init(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread) -> () {

    match CALLBACK_TABLE.vm_init {
        Some(function) => {
            function();
        },
        None => println!("No dynamic callback method was found for VM init events")
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn local_cb_vm_start(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv) -> () {
    match CALLBACK_TABLE.vm_start {
        Some(function) => {
            function();
        },
        None => println!("No dynamic callback method was found for VM start events")
    }
}
