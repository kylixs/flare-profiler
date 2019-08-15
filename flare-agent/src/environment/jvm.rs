use super::super::native::{JavaVMPtr, JVMTIEnvPtr};
use super::super::native::jvmti_native::JVMTI_VERSION;
use super::super::environment::jvmti::{JVMTI, JVMTIEnvironment};
use super::super::error::{wrap_error, NativeError};
use libc::c_void;
use std::ptr;
use native::jvmti_native::JavaVMAttachArgs;
use std::ffi::CString;
use native::JNIEnvPtr;
use environment::jni::{JNIEnvironment, JNI};

pub trait JVMF {
    fn get_environment(&self) -> Result<Box<JVMTI>, NativeError>;
    fn attach(&self, thread_name: &str) -> Result<Box<JNI>, NativeError> { Result::Err(wrap_error(999999)) }
    fn destroy(&self) -> Result<(), NativeError>;
}
///
/// `JVMAgent` represents a binding to the JVM.
///
pub struct JVMAgent {
    vm: JavaVMPtr
}

impl JVMAgent {

    /// Create a new `JVMAgent` instance
    pub fn new(vm: JavaVMPtr) -> JVMAgent {
        JVMAgent { vm: vm }
    }
}

impl JVMF for JVMAgent {

    /// Return the native JVMTI environment if available (ie. the current thread is attached to it)
    /// otherwise return an error message.
    fn get_environment(&self) -> Result<Box<JVMTI>, NativeError> {
        unsafe {
            let mut void_ptr: *mut c_void = ptr::null_mut() as *mut c_void;
            let penv_ptr: *mut *mut c_void = &mut void_ptr as *mut *mut c_void;
            let result = wrap_error((**self.vm).GetEnv.unwrap()(self.vm, penv_ptr, JVMTI_VERSION) as u32);

            match result {
                NativeError::NoError => {
                    let env_ptr: JVMTIEnvPtr = *penv_ptr as JVMTIEnvPtr;
                    let env = JVMTIEnvironment::new(env_ptr);
                    return Result::Ok(Box::new(env));
                },
                err @ _ => Result::Err(wrap_error(err as u32))
            }
        }
    }

//    #define JNI_VERSION_1_1 0x00010001
//    #define JNI_VERSION_1_2 0x00010002
//    #define JNI_VERSION_1_4 0x00010004
//    #define JNI_VERSION_1_6 0x00010006
//    #define JNI_VERSION_1_8 0x00010008
    fn attach(&self, thread_name: &str) -> Result<Box<JNI>, NativeError> {
        unsafe {
            let mut void_ptr: *mut c_void = ptr::null_mut() as *mut c_void;
            let penv_ptr: *mut *mut c_void = &mut void_ptr as *mut *mut c_void;
            let thread_name = CString::new(thread_name).expect("CString::new failed");
            let thread_name_ptr = thread_name.as_ptr() as *mut i8;
            let mut args = JavaVMAttachArgs {
                version: 0x00010006,
                name: thread_name_ptr,
                group: std::ptr::null_mut()
            };
            let mut args_ptr =  &mut args as *mut JavaVMAttachArgs;
            let result = wrap_error((**self.vm).AttachCurrentThreadAsDaemon.unwrap()(self.vm, penv_ptr, args_ptr as *mut c_void) as u32);
            match result {
                NativeError::NoError => {
                    //Note: use env_ptr from AttachCurrentThreadAsDaemon will get crash on call jvmti method, but call GetEnv one more time will work fine!
                    let env_ptr: JNIEnvPtr = *penv_ptr as JNIEnvPtr;
                    let env = JNIEnvironment::new(env_ptr);
                    return Result::Ok(Box::new(env));
//                    return self.get_environment();
                },
                err @ _ => Result::Err(wrap_error(err as u32))
            }
        }
    }

    fn destroy(&self) -> Result<(), NativeError> {
        unsafe {
            let error = (**self.vm).DestroyJavaVM.unwrap()(self.vm) as u32;

            if error == 0 {
                Ok(())
            } else {
                Err(wrap_error(error))
            }
        }
    }
}
