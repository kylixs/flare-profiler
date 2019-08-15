use super::native::jvmti_native::*;
use super::runtime::*;
use super::thread::Thread;

pub type FnMethodEntry = fn(event: MethodInvocationEvent) -> ();
pub type FnMethodExit = fn(event: MethodInvocationEvent) -> ();
pub type FnVMInit = fn() -> ();
pub type FnVMDeath = fn() -> ();
pub type FnVMStart = fn() -> ();
pub type FnVMObjectAlloc = fn(event: ObjectAllocationEvent) -> ();
pub type FnVMObjectFree = fn() -> ();
pub type FnThreadStart = fn(thread: Thread) -> ();
pub type FnThreadEnd = fn(thread: Thread) -> ();
pub type FnException = fn() -> ();
pub type FnExceptionCatch = fn() -> ();
pub type FnMonitorWait = fn(thread: Thread) -> ();
pub type FnMonitorWaited = fn(thread: Thread) -> ();
pub type FnMonitorContendedEnter = fn(thread: Thread) -> ();
pub type FnMonitorContendedEntered = fn(thread: Thread) -> ();
pub type FnFieldAccess = fn() -> ();
pub type FnFieldModification = fn() -> ();
pub type FnGarbageCollectionStart = fn() -> ();
pub type FnGarbageCollectionFinish = fn() -> ();
pub type FnClassFileLoad = fn(event: ClassFileLoadEvent) -> Option<Vec<u8>>;
pub type FnClassLoad = fn() -> ();
pub type FnClassPrepare = fn() -> ();
pub type FnSingleStep = fn() -> ();
pub type FnFramePop = fn() -> ();
pub type FnBreakpoint = fn() -> ();
pub type FnNativeMethodBind = fn() -> ();
pub type FnCompiledMethodLoad = fn() -> ();
pub type FnCompiledMethodUnload = fn() -> ();
pub type FnDynamicCodeGenerated = fn() -> ();
pub type FnResourceExhausted = fn() -> ();
pub type FnDataDumpRequest = fn() -> ();

///
/// `VMEvent` represents events that can occur in JVM applications. These events can be handled
/// using event handlers. For each event a corresponding handler will be called.
///
#[allow(dead_code)]
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum VMEvent {
    VMInit = JVMTI_EVENT_VM_INIT as isize,
    VMDeath = JVMTI_EVENT_VM_DEATH as isize,
    VMObjectAlloc = JVMTI_EVENT_VM_OBJECT_ALLOC as isize,
    VMObjectFree = JVMTI_EVENT_OBJECT_FREE as isize,
    VMStart = JVMTI_EVENT_VM_START as isize,
    MethodEntry = JVMTI_EVENT_METHOD_ENTRY as isize,
    MethodExit = JVMTI_EVENT_METHOD_EXIT as isize,
    ThreadStart = JVMTI_EVENT_THREAD_START as isize,
    ThreadEnd = JVMTI_EVENT_THREAD_END as isize,
    Exception = JVMTI_EVENT_EXCEPTION as isize,
    ExceptionCatch = JVMTI_EVENT_EXCEPTION_CATCH as isize,
    MonitorWait = JVMTI_EVENT_MONITOR_WAIT as isize,
    MonitorWaited = JVMTI_EVENT_MONITOR_WAITED as isize,
    MonitorContendedEnter = JVMTI_EVENT_MONITOR_CONTENDED_ENTER as isize,
    MonitorContendedEntered = JVMTI_EVENT_MONITOR_CONTENDED_ENTERED as isize,
    FieldAccess = JVMTI_EVENT_FIELD_ACCESS as isize,
    FieldModification = JVMTI_EVENT_FIELD_MODIFICATION as isize,
    GarbageCollectionStart = JVMTI_EVENT_GARBAGE_COLLECTION_START as isize,
    GarbageCollectionFinish = JVMTI_EVENT_GARBAGE_COLLECTION_FINISH as isize,
    ClassFileLoadHook = JVMTI_EVENT_CLASS_FILE_LOAD_HOOK as isize,
    ClassLoad = JVMTI_EVENT_CLASS_LOAD as isize,
    ClassPrepare = JVMTI_EVENT_CLASS_PREPARE as isize,
    SingleStep = JVMTI_EVENT_SINGLE_STEP as isize,
    FramePop = JVMTI_EVENT_FRAME_POP as isize,
    Breakpoint = JVMTI_EVENT_BREAKPOINT as isize,
    NativeMethodBind = JVMTI_EVENT_NATIVE_METHOD_BIND as isize,
    CompiledMethodLoad = JVMTI_EVENT_COMPILED_METHOD_LOAD as isize,
    CompiledMethodUnload = JVMTI_EVENT_COMPILED_METHOD_UNLOAD as isize,
    DynamicCodeGenerated = JVMTI_EVENT_DYNAMIC_CODE_GENERATED as isize,
    DataDumpRequest = JVMTI_EVENT_DATA_DUMP_REQUEST as isize,
    ResourceExhausted = JVMTI_EVENT_RESOURCE_EXHAUSTED as isize
}

///
/// The `EventCallbacks` structure is used to define a set of event handlers that the JVM will call
/// when an event fires.
///
#[derive(Default, Clone)]
pub struct EventCallbacks {
    pub vm_init: Option<FnVMInit>,
    pub vm_death: Option<FnVMDeath>,
    pub vm_object_alloc: Option<FnVMObjectAlloc>,
    pub vm_object_free: Option<FnVMObjectFree>,
    pub vm_start: Option<FnVMStart>,
    pub method_entry: Option<FnMethodEntry>,
    pub method_exit: Option<FnMethodExit>,
    pub thread_start: Option<FnThreadStart>,
    pub thread_end: Option<FnThreadEnd>,
    pub exception: Option<FnException>,
    pub exception_catch: Option<FnExceptionCatch>,
    pub monitor_wait: Option<FnMonitorWait>,
    pub monitor_waited: Option<FnMonitorWaited>,
    pub monitor_contended_enter: Option<FnMonitorContendedEnter>,
    pub monitor_contended_entered: Option<FnMonitorContendedEntered>,
    pub field_access: Option<FnFieldAccess>,
    pub field_modification: Option<FnFieldModification>,
    pub garbage_collection_start: Option<FnGarbageCollectionStart>,
    pub garbage_collection_finish: Option<FnGarbageCollectionFinish>,
    pub class_file_load_hook: Option<FnClassFileLoad>,
    pub class_load: Option<FnClassLoad>,
    pub class_prepare: Option<FnClassPrepare>,
    pub single_step: Option<FnSingleStep>,
    pub frame_pop: Option<FnFramePop>,
    pub breakpoint: Option<FnBreakpoint>,
    pub native_method_bind: Option<FnNativeMethodBind>,
    pub compiled_method_load: Option<FnCompiledMethodLoad>,
    pub compiled_method_unload: Option<FnCompiledMethodUnload>,
    pub dynamic_code_generated: Option<FnDynamicCodeGenerated>,
    pub data_dump_request: Option<FnDataDumpRequest>,
    pub resource_exhausted: Option<FnResourceExhausted>
}

impl EventCallbacks {

    pub fn new() -> EventCallbacks {
        EventCallbacks { ..Default::default() }
    }
}
