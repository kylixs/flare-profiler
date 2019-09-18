use super::super::native::{JavaObject, JNIEnvPtr};
use super::super::class::ClassId;
use native::jvmti_native::{jclass, jmethodID, jobject};
use std::ffi::CString;
use native::{JavaMethod, JavaClass, JavaThread, JavaLong};

///
/// `JNI` defines a set of operatations the JVM offers through it's JNI interface.
///
pub trait JNI {

    /// Return an `ClassId` belonging to the given Java object instance.
    fn get_object_class(&self, object_id: &JavaObject) -> ClassId;

    fn find_class(&self, class_name: &str) -> ClassId;

    fn get_method_id(&self, clazz: JavaClass, method_name: &str, method_sig: &str ) -> JavaMethod;

    fn call_long_method(&self, obj: jobject, method_id: JavaMethod) -> JavaLong;

    fn delete_local_ref(&self, obj: jobject);

    fn delete_global_ref(&self, obj: jobject);
}

///
/// This is the native implementation of the `JNI` trait. Each trait method call is delegated
/// to the represented JNI instance.
pub struct JNIEnvironment {

    jni: JNIEnvPtr
}

impl JNIEnvironment {

    pub fn new(jni: JNIEnvPtr) -> JNIEnvironment {
        JNIEnvironment { jni: jni }
    }
}

impl JNI for JNIEnvironment {

    fn get_object_class(&self, object_id: &JavaObject) -> ClassId {
        unsafe {
            let class_id = (**self.jni).GetObjectClass.unwrap()(self.jni, *object_id);

            ClassId { native_id: class_id }
        }
    }

    fn find_class(&self, class_name: &str) -> ClassId {
        unsafe {
            let class_name = class_name.to_string();
            let class_name = CString::new(class_name).expect("CString::new failed");
            let class_name_ptr = class_name.as_ptr() as *const i8;
            let class_id = (**self.jni).FindClass.unwrap()(self.jni, class_name_ptr);
            ClassId { native_id: class_id }
        }
    }

    fn get_method_id(&self, clazz: JavaClass, method_name: &str, method_sig: &str ) -> JavaMethod {
        unsafe {
            let method_name = CString::new(method_name.to_string()).expect("CString::new failed");
            let method_name_ptr = method_name.as_ptr() as *const i8;
            let method_sig = CString::new(method_sig.to_string()).expect("CString::new failed");
            let method_sig_ptr = method_sig.as_ptr() as *const i8;
            let method_id = (**self.jni).GetMethodID.unwrap()(self.jni, clazz, method_name_ptr, method_sig_ptr );
            method_id
        }
    }

    fn call_long_method(&self, obj: jobject, method_id: JavaMethod) -> JavaLong {
        unsafe {
           let value = (**self.jni).CallLongMethod.unwrap()(self.jni, obj, method_id);
            value
        }
    }

    fn delete_local_ref(&self, obj: jobject) {
        unsafe {
            (**self.jni).DeleteLocalRef.unwrap()(self.jni, obj);
        }
    }

    fn delete_global_ref(&self, obj: jobject) {
        unsafe {
            (**self.jni).DeleteGlobalRef.unwrap()(self.jni, obj);
        }
    }
}
