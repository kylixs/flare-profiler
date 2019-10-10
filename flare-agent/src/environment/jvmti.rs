use super::super::capabilities::Capabilities;
use super::super::class::{ClassId, ClassSignature, JavaType};
use super::super::error::{wrap_error, NativeError};
use super::super::event::{EventCallbacks, VMEvent};
use super::super::event_handler::*;
use super::super::mem::MemoryAllocation;
use super::super::method::{MethodId, MethodSignature};
use super::super::thread::{ThreadId, Thread};
use super::super::util::stringify;
use super::super::version::VersionNumber;
use super::super::native::{MutString, MutByteArray, JavaClass, JavaObject, JavaInstance, JavaLong, JavaThread, JVMTIEnvPtr, JavaInt};
use super::super::native::jvmti_native::{Struct__jvmtiThreadInfo, jvmtiCapabilities, jint, jvmtiStackInfo, jthread, jvmtiFrameInfo, jlong, jvmtiTimerInfo};
use std::ptr;
use native::jvmti_native::*;
use std::os::raw::{c_char, c_uchar};
use native::{JavaMethod, JNIEnvPtr};
use environment::jni::JNI;
use environment::Environment;


///
/// JVMTI interface
/// https://docs.oracle.com/javase/8/docs/platform/jvmti/jvmti.html
/// https://docs.oracle.com/en/java/javase/12/docs/specs/jvmti.html
///
pub trait JVMTI {

    ///
    /// Return the JVM TI version number, which includes major, minor and micro version numbers.
    ///
    fn get_version_number(&self) -> VersionNumber;
    /// Set new capabilities by adding the capabilities whose values are set to true in new_caps.
    /// All previous capabilities are retained.
    /// Some virtual machines may allow a limited set of capabilities to be added in the live phase.
    fn add_capabilities(&mut self, new_capabilities: &Capabilities) -> Result<Capabilities, NativeError>;
    fn get_capabilities(&self) -> Capabilities;
    fn get_potential_capabilities(&self) -> Capabilities;
    /// Set the functions to be called for each event. The callbacks are specified by supplying a
    /// replacement function table. The function table is copied--changes to the local copy of the
    /// table have no effect. This is an atomic action, all callbacks are set at once. No events
    /// are sent before this function is called. When an entry is None no event is sent.
    /// An event must be enabled and have a callback in order to be sent--the order in which this
    /// function and set_event_notification_mode are called does not affect the result.
    fn set_event_callbacks(&mut self, callbacks: EventCallbacks) -> Option<NativeError>;
    fn set_event_notification_mode(&mut self, event: VMEvent, mode: bool) -> Option<NativeError>;
    fn get_thread_info(&self, jni: &Box<JNI>, thread_id: &JavaThread) -> Result<Thread, NativeError>;
    fn get_method_declaring_class(&self, method_id: &MethodId) -> Result<ClassId, NativeError>;
    fn get_method_name(&self, method_id: &MethodId) -> Result<MethodSignature, NativeError>;
    fn get_class_signature(&self, class_id: &ClassId) -> Result<ClassSignature, NativeError>;
    fn allocate(&self, len: usize) -> Result<MemoryAllocation, NativeError>;
    fn deallocate(&self, ptr: *mut i8);

    fn get_all_stacktraces(&self, jvmenv: &Environment) -> Result<Vec<JavaStackTrace>, NativeError>;
    fn get_all_threads(&self) -> Result<Vec<ThreadId>, NativeError>;
    fn get_thread_cpu_time(&self, thread_id: &JavaThread) -> Result<JavaLong, NativeError>;
    fn get_thread_cpu_timer_info(&self) -> Result<jvmtiTimerInfo, NativeError>;
    fn get_stack_trace(&self, thread_id: &JavaThread) -> Result<Vec<JavaStackFrame>, NativeError>;

}

pub struct JVMTIEnvironment {
    jvmti: JVMTIEnvPtr
}

impl JVMTIEnvironment {
    pub fn new(env_ptr: JVMTIEnvPtr) -> JVMTIEnvironment {
        JVMTIEnvironment { jvmti: env_ptr }
    }
}

impl JVMTI for JVMTIEnvironment {

    fn get_version_number(&self) -> VersionNumber {
        unsafe {
            let mut version: i32 = 0;
            let version_ptr = &mut version;
            (**self.jvmti).GetVersionNumber.unwrap()(self.jvmti, version_ptr);
            let uversion = *version_ptr as u32;
            VersionNumber::from_u32(&uversion)
        }
    }

    fn add_capabilities(&mut self, new_capabilities: &Capabilities) -> Result<Capabilities, NativeError> {
        let native_caps = new_capabilities.to_native();
        let caps_ptr:*const jvmtiCapabilities = &native_caps;

        unsafe {
            match wrap_error((**self.jvmti).AddCapabilities.unwrap()(self.jvmti, caps_ptr)) {
                NativeError::NoError => Ok(self.get_capabilities()),
                err @ _ => Err(err)
            }
        }
    }

    fn get_capabilities(&self) -> Capabilities {
        unsafe {
            let caps = Capabilities::new();
            let mut native_caps = caps.to_native();
            {
                let cap_ptr = &mut native_caps;
                (**self.jvmti).GetCapabilities.unwrap()(self.jvmti, cap_ptr);
            }
            Capabilities::from_native(&native_caps)
        }
    }

    fn get_potential_capabilities(&self) -> Capabilities {
        unsafe {
            let caps = Capabilities::new();
            let mut native_caps = caps.to_native();
            {
                let cap_ptr = &mut native_caps;
                (**self.jvmti).GetPotentialCapabilities.unwrap()(self.jvmti, cap_ptr);
            }
            Capabilities::from_native(&native_caps)
        }
    }

    fn set_event_callbacks(&mut self, callbacks: EventCallbacks) -> Option<NativeError> {
        register_vm_init_callback(callbacks.vm_init);
        register_vm_start_callback(callbacks.vm_start);
        register_vm_death_callback(callbacks.vm_death);
        register_vm_object_alloc_callback(callbacks.vm_object_alloc);
        register_method_entry_callback(callbacks.method_entry);
        register_method_exit_callback(callbacks.method_exit);
        register_thread_start_callback(callbacks.thread_start);
        register_thread_end_callback(callbacks.thread_end);
        register_exception_callback(callbacks.exception);
        register_exception_catch_callback(callbacks.exception_catch);
        register_monitor_wait_callback(callbacks.monitor_wait);
        register_monitor_waited_callback(callbacks.monitor_waited);
        register_monitor_contended_enter_callback(callbacks.monitor_contended_enter);
        register_monitor_contended_endered_callback(callbacks.monitor_contended_entered);
        register_field_access_callback(callbacks.field_access);
        register_field_modification_callback(callbacks.field_modification);
        register_garbage_collection_start(callbacks.garbage_collection_start);
        register_garbage_collection_finish(callbacks.garbage_collection_finish);
        register_class_file_load_hook(callbacks.class_file_load_hook);

        let (native_callbacks, callbacks_size) = registered_callbacks();

        unsafe {
            match wrap_error((**self.jvmti).SetEventCallbacks.unwrap()(self.jvmti, &native_callbacks, callbacks_size)) {
                NativeError::NoError => None,
                err @ _ => Some(err)
            }
        }
    }

    fn set_event_notification_mode(&mut self, event: VMEvent, mode: bool) -> Option<NativeError> {
        unsafe {
            let mode_i = match mode { true => 1, false => 0 };
            let sptr: JavaObject = ptr::null_mut();

            let event1 = event.clone();
            match wrap_error((**self.jvmti).SetEventNotificationMode.unwrap()(self.jvmti, mode_i, event as u32, sptr)) {
                NativeError::NoError => None,
                err @ _ => {
                    println!("set_event_notification_mode failed, event: {:?}, mode: {}, error: {:?}", event1, mode, err);
                    Some(err)
                }
            }
        }
    }

    fn get_thread_info(&self, jni: &Box<JNI>, thread_id: &JavaThread) -> Result<Thread, NativeError> {
        let mut info = Struct__jvmtiThreadInfo { name: ptr::null_mut(), priority: 0, is_daemon: 0, thread_group: ptr::null_mut(), context_class_loader: ptr::null_mut()};
        let mut info_ptr = &mut info;

        unsafe {
            match (**self.jvmti).GetThreadInfo {
                Some(func) => {
                    match wrap_error(func(self.jvmti, *thread_id, info_ptr)) {
                        NativeError::NoError => {
                            let thread = Thread {
                                id: ThreadId {native_id: *thread_id},
                                thread_id: 0,
                                name: stringify((*info_ptr).name),
                                priority: (*info_ptr).priority as u32,
                                is_daemon: if (*info_ptr).is_daemon > 0 { true } else { false },
                                thread_group: info.thread_group,
                                context_class_loader: info.context_class_loader,
                            };
                            //jni.delete_local_ref(info.thread_group);
                            //jni.delete_local_ref(info.context_class_loader);
                            self.deallocate(info.name);
                            Ok(thread)
                        },
                        err@_ => Err(err)
                    }
                },
                None => Err(NativeError::NoError)
            }
        }
    }

    fn get_method_declaring_class(&self, method_id: &MethodId) -> Result<ClassId, NativeError> {
        let mut jstruct: JavaInstance = JavaInstance { _hacky_hack_workaround: 0 };
        let mut jclass_instance: JavaClass = &mut jstruct;
        let meta_ptr: *mut JavaClass = &mut jclass_instance;

        unsafe {
            match wrap_error((**self.jvmti).GetMethodDeclaringClass.unwrap()(self.jvmti, method_id.native_id, meta_ptr)) {
                NativeError::NoError => Ok(ClassId { native_id: *meta_ptr }),
                err @ _ => Err(err)
            }
        }
    }

    fn get_method_name(&self, method_id: &MethodId) -> Result<MethodSignature, NativeError> {
        let mut method_name = ptr::null_mut();
        let mut method_ptr = &mut method_name;

        let mut signature: MutString = ptr::null_mut();
        let mut signature_ptr = &mut signature;

        let mut generic_sig: MutString = ptr::null_mut();
        let mut generic_sig_ptr = &mut generic_sig;

        unsafe {
            match wrap_error((**self.jvmti).GetMethodName.unwrap()(self.jvmti, method_id.native_id, method_ptr, signature_ptr, generic_sig_ptr)) {
                NativeError::NoError => {
                    let method_signature = MethodSignature::new(stringify(*method_ptr), stringify(*signature_ptr), stringify(*generic_sig_ptr));
                    self.deallocate(method_name);
                    self.deallocate(signature);
                    self.deallocate(generic_sig);
                    Ok(method_signature)
                },
                err @ _ => Err(err)
            }
        }
    }

    fn get_class_signature(&self, class_id: &ClassId) -> Result<ClassSignature, NativeError> {
        unsafe {
            let mut sig: MutString = ptr::null_mut();
            let mut generic: MutString = ptr::null_mut();
            let p1: *mut MutString = &mut sig;
            let p2: *mut MutString = &mut generic;

            match wrap_error((**self.jvmti).GetClassSignature.unwrap()(self.jvmti, class_id.native_id, p1, p2)) {
                NativeError::NoError => {
                    let class_signature = ClassSignature::new(&JavaType::parse(&stringify(sig)).unwrap(), stringify(generic));
                    self.deallocate(sig);
                    self.deallocate(generic);
                    Ok(class_signature)
                },
                err @ _ => Err(err)
            }
        }
    }

    fn allocate(&self, len: usize) -> Result<MemoryAllocation, NativeError> {
        let size: JavaLong = len as JavaLong;
        let mut ptr: MutByteArray = ptr::null_mut();
        let mem_ptr: *mut MutByteArray = &mut ptr;

        unsafe {
            match wrap_error((**self.jvmti).Allocate.unwrap()(self.jvmti, size, mem_ptr)) {
                NativeError::NoError => Ok(MemoryAllocation { ptr: ptr, len: len }),
                err @ _ => Err(err)
            }
        }
    }

    fn deallocate(&self, ptr: *mut i8) {
        if ptr != ptr::null_mut() {
            unsafe {
                (**self.jvmti).Deallocate.unwrap()(self.jvmti, ptr as *mut c_uchar);
            }
        }
    }

    fn get_all_stacktraces(&self, jvmenv: &Environment) -> Result<Vec<JavaStackTrace>, NativeError> {
        let max_frame_count:jint = 2000;
        let mut thread_count:jint = 0;
        let mut stack_info_ptr: *mut jvmtiStackInfo = ptr::null_mut();
        let mut stack_traces_list: Vec<JavaStackTrace> = vec![];
        unsafe {
            match wrap_error((**self.jvmti).GetAllStackTraces.unwrap()(self.jvmti, max_frame_count, &mut stack_info_ptr, &mut thread_count )){
                NativeError::NoError => {
                    let count: usize = thread_count as usize;
                    let stack_info_array = unsafe { std::slice::from_raw_parts(stack_info_ptr, count ) };
                    //enumerate thread stacks
                    for i in 0..count {
                        let stack_info = stack_info_array[i];

                        let mut cpu_time: i64 = 0_i64;
                        //if std::time::Instant::now()
                        if let Ok(t) = jvmenv.get_thread_cpu_time(&stack_info.thread) {
                            cpu_time = t;
                            //ignore inactive thread call
                            if cpu_time == 0_i64 {
                                jvmenv.delete_local_ref(stack_info.thread);
                                continue;
                            }
                        } else {
                            println!("get_thread_cpu_time error");
                        }

                        //get thread info and release thread local ref
                        let thread_info = jvmenv.get_thread_info_ex(&stack_info.thread).unwrap();
                        jvmenv.delete_local_ref(stack_info.thread);

                        let mut stack_trace = JavaStackTrace{
                            thread: thread_info,
                            state: stack_info.state,
                            frame_buffer: vec![],
                            cpu_time: cpu_time
                        };

                        let stack_frames = unsafe { std::slice::from_raw_parts(stack_info.frame_buffer,stack_info.frame_count as usize) };
                        for n in 0..stack_info.frame_count as usize {
                            let stack_frame = stack_frames[n];
                            stack_trace.frame_buffer.push( JavaStackFrame{ method: stack_frame.method, location: stack_frame.location } );
                        }
                        stack_traces_list.push(stack_trace);

                    }
                    self.deallocate(stack_info_ptr as *mut i8);
                    Ok(stack_traces_list)
                },
                err@ _ => {
                    println!("GetAllStackTraces error: {:?}", err);
                    Err(err)
                }
            }
        }
    }

    fn get_all_threads(&self) -> Result<Vec<ThreadId>, NativeError> {
        let mut thread_count:jint = 0;
        let mut threads_ptr : *mut jthread = ptr::null_mut();

        unsafe {
            match wrap_error((**self.jvmti).GetAllThreads.unwrap()(self.jvmti, &mut thread_count, &mut threads_ptr)){
                NativeError::NoError => {
                    let mut threads = vec![];

                    let threads_array = unsafe { std::slice::from_raw_parts(threads_ptr, thread_count as usize ) };
                    for thr in threads_array {
                        threads.push(ThreadId{ native_id: *thr })
                    }

                    self.deallocate(threads_ptr as *mut i8);
                    Ok(threads)
                },
                err@ _ => Err(err)
            }
        }
    }

    fn get_thread_cpu_time(&self, thread_id: &JavaThread) -> Result<JavaLong, NativeError> {
        let mut nanos: JavaLong = 0;
        unsafe {
            match wrap_error((**self.jvmti).GetThreadCpuTime.unwrap()(self.jvmti, *thread_id, &mut nanos)){
                NativeError::NoError => Ok(nanos),
                err @ _ => Err(err)
            }
        }
    }

    //jvmtiError GetThreadCpuTimerInfo(jvmtiEnv* env, jvmtiTimerInfo* info_ptr)
    fn get_thread_cpu_timer_info(&self) -> Result<jvmtiTimerInfo, NativeError> {
        let mut timerInfo = jvmtiTimerInfo{
            max_value: 0,
            may_skip_forward: 0,
            may_skip_backward: 0,
            kind: JVMTI_TIMER_TOTAL_CPU,
            reserved1: 0,
            reserved2: 0
        };

        unsafe {
            match wrap_error((**self.jvmti).GetThreadCpuTimerInfo.unwrap()(self.jvmti, &mut timerInfo)){
                NativeError::NoError => Ok(timerInfo),
                err @ _ => Err(err)
            }
        }
    }

    fn get_stack_trace(&self, thread_id: &JavaThread) -> Result<Vec<JavaStackFrame>, NativeError> {
        const max_frame_count:jint = 100;
        let mut frame_infos = [jvmtiFrameInfo{ method: 0 as jmethodID, location: 0};max_frame_count as usize];
        let mut frame_count = 0;
        unsafe {
            match wrap_error((**self.jvmti).GetStackTrace.unwrap()(self.jvmti, *thread_id, 0, max_frame_count, frame_infos.as_mut_ptr(), &mut frame_count)){
                NativeError::NoError => {
                    let mut frames = vec![];
//                    let frame_infos = unsafe { std::slice::from_raw_parts(frame_info_ptr, frame_count as usize ) };
                    for i in 0..frame_count as usize {
                        let frame = frame_infos[i];
                        frames.push(JavaStackFrame{
                            method: frame.method,
                            location: frame.location
                        });
                    }
                    Ok(frames)
                },
                err @ _ => Err(err)
            }
        }
    }

}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct ThreadInfo {
    pub thread_id: JavaLong, // actual java thread id
    pub name: String,
    pub priority: u32,
    pub is_daemon: bool,
    pub cpu_time: i64
}

pub struct JavaStackTrace {
    pub thread: ThreadInfo,
    pub state: JavaInt,
    pub cpu_time: i64,
    pub frame_buffer: Vec<JavaStackFrame>
}

pub struct JavaStackFrame {
    pub method: JavaMethod,
    pub location: JavaLong,
}
