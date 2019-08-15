use super::capabilities::Capabilities;
use super::class::{ClassId, ClassSignature};
use super::error::NativeError;
use super::environment::jvm::JVMF;
use super::environment::jvmti::{JVMTI};
use super::event::{EventCallbacks, VMEvent};
use super::mem::MemoryAllocation;
use super::method::{MethodId, MethodSignature};
use super::native::JavaThread;
use super::runtime::*;
use super::thread::Thread;
use super::version::VersionNumber;
use std::collections::HashMap;

/// Allows testing of JVM and JVMTI-related functions by emulating (mocking) a JVM agent.
pub struct JVMEmulator {
    pub capabilities: Capabilities,
    pub callbacks: EventCallbacks,
    pub events: HashMap<VMEvent, bool>
}

impl JVMEmulator {
    pub fn new() -> JVMEmulator {
        JVMEmulator {
            capabilities: Capabilities::new(),
            callbacks: EventCallbacks::new(),
            events: HashMap::new()
        }
    }

    pub fn emit_method_entry(&self, event: MethodInvocationEvent) {
        match self.callbacks.method_entry {
            Some(handler) => {
                handler(event);
            },
            _ => ()
        }
    }
}

impl JVMF for JVMEmulator {
    fn get_environment(&self) -> Result<Box<JVMTI>, NativeError> {
        Ok(Box::new(JVMEmulator::new()))
    }

    fn destroy(&self) -> Result<(), NativeError> {
        Ok(())
    }
}

impl JVMTI for JVMEmulator {

    fn get_version_number(&self) -> VersionNumber {
        VersionNumber::unknown()
    }

    fn add_capabilities(&mut self, new_capabilities: &Capabilities) -> Result<Capabilities, NativeError> {
        let merged = self.capabilities.merge(&new_capabilities);
        self.capabilities = merged;
        Ok(self.capabilities.clone())
    }

    fn get_capabilities(&self) -> Capabilities {
        self.capabilities.clone()
    }

    fn get_potential_capabilities(&self) -> Capabilities {
        self.capabilities.clone()
    }

    fn set_event_callbacks(&mut self, callbacks: EventCallbacks) -> Option<NativeError> {
        self.callbacks = callbacks;

        None
    }

    fn set_event_notification_mode(&mut self, event: VMEvent, mode: bool) -> Option<NativeError> {
        self.events.insert(event, mode);
        None
    }

    fn get_thread_info(&self, thread_id: &JavaThread) -> Result<Thread, NativeError> {
        match *thread_id as u64 {
            _ => Err(NativeError::NotImplemented)
        }
    }

    fn get_method_declaring_class(&self, method_id: &MethodId) -> Result<ClassId, NativeError> {
        match method_id.native_id as u64 {
            _ => Err(NativeError::NotImplemented)
        }
    }

    fn get_method_name(&self, method_id: &MethodId) -> Result<MethodSignature, NativeError> {
        match method_id.native_id as u64 {
            0x01 => Ok(MethodSignature::new("".to_string(), "".to_string(), "".to_string())),
            _ => Err(NativeError::NotImplemented)
        }
    }

    fn get_class_signature(&self, class_id: &ClassId) -> Result<ClassSignature, NativeError> {
        match class_id.native_id as u64 {
            _ => Err(NativeError::NotImplemented)
        }
    }

    fn allocate(&self, len: usize) -> Result<MemoryAllocation, NativeError> {
        Ok(MemoryAllocation { ptr: ::std::ptr::null_mut(), len: len })
    }

    fn deallocate(&self, prt: *mut i8) {

    }
}
