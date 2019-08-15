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
use native::{JavaClass, JavaMethod, JavaLong, JNIEnvPtr};
use environment::jvmti::JavaStackTrace;
use thread::ThreadId;
use native::jvmti_native::jvmtiTimerInfo;
use std::cell::Cell;

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

    pub fn new(jvmti: JVMTIEnvironment, jni: JNIEnvironment) -> Environment {
        Environment { jvmti: Box::new(jvmti), jni: Box::new(jni ), thread_get_id_method: Cell::new(None) }
    }

    pub fn new_from(jvmti: Box<JVMTI>, jni: Box<JNI>) -> Environment {
        Environment { jvmti: jvmti, jni: jni, thread_get_id_method: Cell::new(None) }
    }

    fn get_thread_id(&self, thread_id: &JavaThread) -> JavaLong {
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

}

impl JVMTI for Environment {

    fn get_version_number(&self) -> VersionNumber {
        self.jvmti.get_version_number()
    }

    fn add_capabilities(&mut self, new_capabilities: &Capabilities) -> Result<Capabilities, NativeError> {
        self.jvmti.add_capabilities(new_capabilities)
    }

    fn get_capabilities(&self) -> Capabilities {
        self.jvmti.get_capabilities()
    }

    fn get_potential_capabilities(&self) -> Capabilities {
        self.jvmti.get_potential_capabilities()
    }

    fn set_event_callbacks(&mut self, callbacks: EventCallbacks) -> Option<NativeError> {
        self.jvmti.set_event_callbacks(callbacks)
    }

    fn set_event_notification_mode(&mut self, event: VMEvent, mode: bool) -> Option<NativeError> {
        self.jvmti.set_event_notification_mode(event, mode)
    }

    fn get_thread_info(&self, thread_id: &JavaThread) -> Result<Thread, NativeError> {
        let mut thread_info = self.jvmti.get_thread_info(thread_id).unwrap();
        let java_thread_id = self.get_thread_id(&thread_id);
        thread_info.thread_id = java_thread_id;
        Ok(thread_info)
    }

    fn get_method_declaring_class(&self, method_id: &MethodId) -> Result<ClassId, NativeError> {
        self.jvmti.get_method_declaring_class(method_id)
    }

    fn get_method_name(&self, method_id: &MethodId) -> Result<MethodSignature, NativeError> {
        self.jvmti.get_method_name(method_id)
    }

    fn get_class_signature(&self, class_id: &ClassId) -> Result<ClassSignature, NativeError> {
        self.jvmti.get_class_signature(class_id)
    }

    fn allocate(&self, len: usize) -> Result<MemoryAllocation, NativeError> {
        self.jvmti.allocate(len)
    }

    fn deallocate(&self, ptr: *mut i8) {
        self.jvmti.deallocate(ptr)
    }

    fn get_all_stacktraces(&self) -> Result<Vec<JavaStackTrace>, NativeError> {
        self.jvmti.get_all_stacktraces()
    }

    fn get_all_threads(&self) -> Result<Vec<ThreadId>, NativeError> {
        self.jvmti.get_all_threads()
    }

    fn get_thread_cpu_time(&self, thread_id: &JavaThread) -> Result<JavaLong, NativeError> {
        self.jvmti.get_thread_cpu_time(thread_id)
    }

    fn get_thread_cpu_timer_info(&self) -> Result<jvmtiTimerInfo, NativeError> {
        self.jvmti.get_thread_cpu_timer_info()
    }

    fn get_jni_env(&self) -> Result<JNIEnvPtr, NativeError> {
        self.jvmti.get_jni_env()
    }
}

impl JNI for Environment {

    fn get_object_class(&self, object_id: &JavaObject) -> ClassId {
        self.jni.get_object_class(object_id)
    }

    fn find_class(&self, class_name: &str) -> ClassId {
        self.jni.find_class(class_name)
    }

    fn get_method_id(&self, clazz: JavaClass, method_name: &str, method_sig: &str) -> JavaMethod {
        self.jni.get_method_id(clazz, method_name, method_sig)
    }

    fn call_long_method(&self, thread: JavaThread, method_id: JavaMethod) -> JavaLong {
        self.jni.call_long_method(thread, method_id)
    }
}
