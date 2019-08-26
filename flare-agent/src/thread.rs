use super::native::JavaThread;
use std::fmt::{Display, Formatter, Error};
use native::JavaLong;
use native::jvmti_native::jobject;

//use jni::sys::*;
//use jvmti_sys::*;

///
/// Represents a link between a JVM thread and the Rust code calling the JVMTI API.
///
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct ThreadId {
    pub native_id: JavaThread,
}

/// Marker trait implementation for `Send`
unsafe impl Send for ThreadId { }

/// Marker trait implementation for `Sync`
unsafe impl Sync for ThreadId { }

impl Display for ThreadId {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{:?}", self.native_id)
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Thread {
    pub id: ThreadId,
    pub thread_id: JavaLong, // actual java thread id
    pub name: String,
    pub priority: u32,
    pub is_daemon: bool,
    pub thread_group: jobject,
    pub context_class_loader: jobject
}
