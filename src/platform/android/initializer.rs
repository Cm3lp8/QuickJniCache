use crate::jni_methods_cache::executor;
use crate::jni_methods_cache::executor::ExecutorChannel;
use crate::jni_methods_cache::methods_cache::JavaMethodCache;
use crate::jni_methods_cache::methods_cache::JavaMethodCacheBuilder;
use crate::jni_methods_cache::ACTIVITY;
use crate::jni_methods_cache::INIT;
use crate::jni_methods_cache::JAVAMETHODCACHE;
use crate::jni_methods_cache::JAVAVM;
use crate::jni_methods_cache::JNIENV;
use crate::jni_methods_cache::JVMCALLER;
use jni::objects::JObject;

use winit::platform::android::activity::AndroidApp;
impl<'a: 'static> JavaMethodCache<'a> {
    pub fn init(
        android_app: &AndroidApp,
        build_cb: impl FnOnce(&mut JavaMethodCacheBuilder<'a>) + std::marker::Send + 'static,
    ) {
        unsafe {
            if ACTIVITY.is_none() {
                let jv_vm_ptr = android_app.vm_as_ptr() as *mut jni::sys::JavaVM;
                let java_vm = std::mem::ManuallyDrop::new(jni::JavaVM::from_raw(jv_vm_ptr).unwrap());
                let mut env = java_vm.get_env().expect("No env on init thread");
                let activity_raw = JObject::from_raw(android_app.activity_as_ptr() as *mut jni::sys::_jobject);
                let activity_global = env
                    .new_global_ref(&activity_raw)
                    .expect("failed to create NativeActivity global ref on init thread");
                std::mem::forget(activity_raw);
                ACTIVITY = Some(activity_global);
            }
        }

        let executor_channel = ExecutorChannel::new();
        let executor_receiver = executor_channel.get_receiver();
        let android_app = android_app.clone();
        std::thread::spawn(move || {
            initializer_internal::init_once(&android_app, build_cb);

            executor::run(executor_receiver);
        });

        unsafe {
            if let None = JVMCALLER {
                JVMCALLER = Some(executor_channel.get_jvm_caller())
            };
        }
    }
}
mod initializer_internal {
    use super::*;

    use crate::jni_methods_cache::methods_cache::java_method_build_tools::j_object_ref::JavaMethodsListRefs;
    use crate::jni_methods_cache::methods_cache::java_method_build_tools::standard_class_finder::StandardClassPreList;
    use jni::objects::JObject;

    pub fn init_once<'a: 'static>(
        android_app: &AndroidApp,
        build_cb: impl FnOnce(&mut JavaMethodCacheBuilder<'a>),
    ) {
        INIT.call_once(|| unsafe {
            let jv_vm_ptr = unsafe { android_app.vm_as_ptr() as *mut jni::sys::JavaVM };
            let java_vm = unsafe { jni::JavaVM::from_raw(jv_vm_ptr).unwrap() };
            unsafe {
                JAVAVM = Some(java_vm);
            };
            let res_env = JAVAVM
                .as_ref()
                .expect("no java vm attached")
                .attach_current_thread_permanently();
            let env: jni::JNIEnv =
                unsafe { JAVAVM.as_ref().expect("no jvm").get_env().expect("No env") };

            unsafe {
                JNIENV = None;
            }

            let activity_local = env
                .new_local_ref(unsafe { ACTIVITY.as_ref().expect("no native activity").as_obj() })
                .expect("failed to create NativeActivity local ref");
            let activity_local: &'static mut JObject<'static> =
                Box::leak(Box::new(activity_local));

            let mut java_method_builder = JavaMethodCacheBuilder {
                env: Some(env),
                activity: Some(activity_local),
                java_vm: unsafe { Some(&JAVAVM.as_ref().expect("no jvm")) },
                standard_class_pre_list: StandardClassPreList::new(),
                java_methods_list_ref: JavaMethodsListRefs::new(),
                cache_builded: false,
            };

            build_cb(&mut java_method_builder);

            java_method_builder.build(|cache| {
                unsafe {
                    JAVAMETHODCACHE = cache.set_to_java_methods();
                    JAVAMETHODCACHE.build_cache()
                };
            });
        });
    }
}
