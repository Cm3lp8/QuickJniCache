use std::fmt::Debug;

/***** Java Method cache module ******/
use jni::objects::{GlobalRef, JObject};
use jni::signature::ReturnType;
use parking_lot::Once;

pub mod executor;
pub mod methods_cache;
pub mod thread_pool;
pub use methods_cache::NullObject;

use methods_cache::{
    java_method_cache_utils::{JavaArgs, ReturnedValue},
    JavaMethods,
};

use self::executor::JvmCaller;
use self::methods_cache::JVMResponse;

pub static INIT: Once = Once::new();
pub static mut JAVAVM: Option<jni::JavaVM> = None;
pub static mut JNIENV: Option<jni::JNIEnv> = None;
pub static mut ACTIVITY: Option<GlobalRef> = None;
pub static mut JAVAMETHODCACHE: JavaMethods = JavaMethods::None;
pub static mut JVMCALLER: Option<JvmCaller> = None;

pub fn build_jni_methods_cache() {
    unsafe { JAVAMETHODCACHE.build_cache() };
}

pub fn call_java_static_method<T: 'static + JVMResponse + Debug>(
    class_name: &str,
    method_name: &str,
    sig: &str,
    args: JavaArgs,
    return_type: ReturnType,
    returned_object_id: Option<String>,
) -> std::result::Result<Option<T>, ()> {
    unsafe {
        let res = JVMCALLER
            .as_ref()
            .expect("no JVMCALLER")
            .call_static_method::<T>(
                class_name,
                method_name,
                sig,
                args,
                return_type,
                returned_object_id,
            );

        res
    }
}

pub fn call_java_method(
    class_name: &str,
    method_name: &str,
    sig: &str,
    args: JavaArgs,
    return_type: ReturnType,
    returned_object_id: Option<String>,
) -> std::result::Result<ReturnedValue, String> {
    unsafe {
        JAVAMETHODCACHE.call_method(
            class_name,
            method_name,
            sig,
            args,
            return_type,
            returned_object_id,
        )
    }
}
