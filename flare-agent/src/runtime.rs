use super::bytecode::Classfile;
use super::class::{ClassId, ClassSignature};
use super::method::{MethodId, MethodSignature};
use super::thread::Thread;
use super::thread::ThreadId;
use super::native::{JavaMethod, JavaLong};

pub trait RuntimeEvent {
}

pub struct ObjectAllocationEvent {
    pub class_id: ClassId,
    pub thread: Thread,
    pub size: i64
}

pub struct ObjectFreeEvent {

}

pub struct MethodInvocationEvent {
    pub method_id: MethodId,
    pub thread_id: ThreadId,
    // pub method_sig: MethodSignature,
    // pub class_sig: ClassSignature,
    // pub thread: Thread,
}

impl RuntimeEvent for ObjectAllocationEvent {}
impl RuntimeEvent for MethodInvocationEvent {}

pub struct ClassFileLoadEvent {
    pub class_name: String,
    pub class: Classfile
}

impl RuntimeEvent for ClassFileLoadEvent {}
