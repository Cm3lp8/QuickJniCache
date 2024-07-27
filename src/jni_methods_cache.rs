/***** Java Method cache module ******/
use jni::objects::{
    JClass, JDoubleArray, JMethodID, JObject, JStaticMethodID, JString, JValue, JValueOwned,
};
use jni::signature::ReturnType;
use jni::sys::{jobject, JNIEnv};
use jni::JavaVM;
use parking_lot::Once;
use std::fmt::Result;
use std::mem;
use winit::platform::android::activity::AndroidApp;

mod executor;
pub mod methods_cache;

use methods_cache::{
    java_method_cache_utils::{JavaArgs, ReturnedValue},
    JavaMethodCache, JavaMethodCacheBuilder, JavaMethods,
};

use self::executor::{run, ExecutorChannel, JvmCaller};

static INIT: Once = Once::new();
static mut JAVAVM: Option<jni::JavaVM> = None;
static mut JNIENV: Option<jni::JNIEnv> = None;
static mut ACTIVITY: Option<JObject> = None;
pub static mut JAVAMETHODCACHE: JavaMethods = JavaMethods::None;
static mut JVMCALLER: Option<JvmCaller> = None;

pub fn build_jni_methods_cache() {
    unsafe { JAVAMETHODCACHE.build_cache() };
}

pub fn call_java_static_method(
    class_name: &str,
    method_name: &str,
    sig: &str,
    args: JavaArgs,
    return_type: ReturnType,
    returned_object_id: Option<&str>,
) -> std::result::Result<ReturnedValue, String> {
    println!("calling in !");
    unsafe {
        JVMCALLER
            .as_ref()
            .expect("no JVMCALLER")
            .call_static_method::<String>(
                class_name,
                method_name,
                sig,
                args,
                return_type,
                returned_object_id,
            );

        Ok(ReturnedValue::String(format!(
            "call done for [{:?}]!",
            class_name
        )))
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

impl<'a: 'static> JavaMethodCache<'a> {
    pub fn init(
        android_app: &AndroidApp,
        build_cb: impl FnOnce(&mut JavaMethodCacheBuilder<'a>) + std::marker::Send + 'static,
    ) {
        let executor_channel = ExecutorChannel::new();
        let executor_receiver = executor_channel.get_receiver();
        let android_app = android_app.clone();
        println!("start ext");
        std::thread::spawn(move || {
            println!("start in");
            methods_cache::init_once(&android_app, build_cb);

            executor::run(executor_receiver);
        });

        unsafe {
            if let None = JVMCALLER {
                JVMCALLER = Some(executor_channel.get_jvm_caller())
            };
        }
    }
}
