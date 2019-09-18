use self::jvmti::{JVMTI, JVMTIEnvironment};
use self::jni::{JNI, JNIEnvironment};
use super::capabilities::Capabilities;
use super::class::{ClassId, ClassSignature};
use super::error::NativeError;
use super::event::{EventCallbacks, VMEvent};
use super::mem::MemoryAllocation;
use super::method::{MethodId, MethodSignature};
use super::native::{JavaObject, JavaThread};
use super::thread::Thread;
use super::version::VersionNumber;
use native::{JavaClass, JavaMethod, JavaLong, JNIEnvPtr, JavaInt};
use thread::ThreadId;
use native::jvmti_native::{jvmtiTimerInfo, jobject, jvmtiStackInfo};
use std::cell::Cell;
use std::ptr;
use environment::jvmti::{ThreadInfo, JavaStackTrace, JavaStackFrame};

pub mod jni;
pub mod jvm;
pub mod jvmti;

/// `Environment` combines the functionality of both `JNI` and `JVMTI` by wrapping an instance of
/// both and delegating the method calls to their corresponding recipients.
pub struct Environment {
    jvmti: Box<JVMTI>,
    jni: Box<JNI>,
    thread_get_id_method: Cell<Option<JavaMethod>>
}

impl Environment {

//    pub fn new(jvmti: JVMTIEnvironment, jni: JNIEnvironment) -> Environment {
//        Environment { jvmti: Box::new(jvmti), jni: Box::new(jni ), thread_get_id_method: Cell::new(None) }
//    }

    pub fn new(jvmti: Box<JVMTI>, jni: Box<JNI>) -> Environment {
        Environment { jvmti: jvmti, jni: jni, thread_get_id_method: Cell::new(None) }
    }

    pub fn get_thread_id(&self, thread_id: &JavaThread) -> JavaLong {
        //get actual java thread id
        match self.thread_get_id_method.get() {
            Some(method_id) => {
                self.call_long_method(thread_id.clone(), method_id)
            },
            None => {
                let thread_class = self.jni.find_class("java/lang/Thread");
                let get_id_method = self.jni.get_method_id(thread_class.native_id, "getId", "()J");
                self.thread_get_id_method.set(Some(get_id_method.clone()));
                self.call_long_method(thread_id.clone(), get_id_method)
            },
        }
    }

    pub fn get_thread_cpu_time_ex(&self, thread_id: JavaLong) -> i64 {
//        let classid_management_factory = self.jni.find_class("java/lang/management/ManagementFactory");
//        let method_getThreadMXBean = self.jni.get_method_id(classid_management_factory.native_id, "getThreadMXBean", "()J");
        unimplemented!()
    }

    //JNI methods
    pub fn get_object_class(&self, object_id: &JavaObject) -> ClassId {
        self.jni.get_object_class(object_id)
    }

    pub fn find_class(&self, class_name: &str) -> ClassId {
        self.jni.find_class(class_name)
    }

    pub fn get_method_id(&self, clazz: JavaClass, method_name: &str, method_sig: &str) -> JavaMethod {
        self.jni.get_method_id(clazz, method_name, method_sig)
    }

    pub fn call_long_method(&self, obj: jobject, method_id: JavaMethod) -> JavaLong {
        self.jni.call_long_method(obj, method_id)
    }

    pub fn delete_local_ref(&self, obj: jobject) {
        self.jni.delete_local_ref(obj);
    }

    pub fn delete_global_ref(&self, obj: jobject) {
        self.jni.delete_global_ref(obj);
    }

    //JVMTI methods
    pub fn get_version_number(&self) -> VersionNumber {
        self.jvmti.get_version_number()
    }

    pub fn add_capabilities(&mut self, new_capabilities: &Capabilities) -> Result<Capabilities, NativeError> {
        self.jvmti.add_capabilities(new_capabilities)
    }

    pub fn get_capabilities(&self) -> Capabilities {
        self.jvmti.get_capabilities()
    }

    pub fn get_potential_capabilities(&self) -> Capabilities {
        self.jvmti.get_potential_capabilities()
    }

    pub fn set_event_callbacks(&mut self, callbacks: EventCallbacks) -> Option<NativeError> {
        self.jvmti.set_event_callbacks(callbacks)
    }

    pub fn set_event_notification_mode(&mut self, event: VMEvent, mode: bool) -> Option<NativeError> {
        self.jvmti.set_event_notification_mode(event, mode)
    }

    pub fn get_thread_info(&self, thread_id: &JavaThread) -> Result<Thread, NativeError> {
        self.jvmti.get_thread_info(&self.jni, thread_id)
    }

    pub fn get_thread_info_ex(&self, thread_id: &JavaThread) -> Result<ThreadInfo, NativeError> {
        let java_thread_id = self.get_thread_id(&thread_id);
        let mut thread = self.jvmti.get_thread_info(&self.jni, thread_id).unwrap();
        let thread_info = ThreadInfo{
            thread_id: java_thread_id,
            name: thread.name.clone(),
            priority: thread.priority,
            is_daemon: thread.is_daemon,
            cpu_time: 0
        };
        //release jni local ref ?
        self.delete_local_ref(thread.thread_group);
        self.delete_local_ref(thread.context_class_loader);
        Ok(thread_info)
    }

    pub fn get_method_declaring_class(&self, method_id: &MethodId) -> Result<ClassId, NativeError> {
        self.jvmti.get_method_declaring_class(method_id)
    }

    pub fn get_method_name(&self, method_id: &MethodId) -> Result<MethodSignature, NativeError> {
        self.jvmti.get_method_name(method_id)
    }

    pub fn get_class_signature(&self, class_id: &ClassId) -> Result<ClassSignature, NativeError> {
        self.jvmti.get_class_signature(class_id)
    }

    pub fn allocate(&self, len: usize) -> Result<MemoryAllocation, NativeError> {
        self.jvmti.allocate(len)
    }

    pub fn deallocate(&self, ptr: *mut i8) {
        self.jvmti.deallocate(ptr)
    }

    pub fn get_all_stacktraces(&self) -> Result<Vec<JavaStackTrace>, NativeError> {
        self.jvmti.get_all_stacktraces(self)
    }

    pub fn get_all_threads(&self) -> Result<Vec<ThreadId>, NativeError> {
        self.jvmti.get_all_threads()
    }

    pub fn get_thread_cpu_time(&self, thread_id: &JavaThread) -> Result<JavaLong, NativeError> {
        self.jvmti.get_thread_cpu_time(thread_id)
    }

    pub fn get_thread_cpu_timer_info(&self) -> Result<jvmtiTimerInfo, NativeError> {
        self.jvmti.get_thread_cpu_timer_info()
    }

    pub fn get_stack_trace(&self, thread_id: &JavaThread) -> Result<Vec<JavaStackFrame>, NativeError> {
        self.jvmti.get_stack_trace(thread_id)
    }

}


