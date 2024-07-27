#[cfg(target_os = "android")]
mod jni_methods_cache;

#[cfg(target_os = "android")]
pub use crate::jni_methods_cache::call_java_method;
#[cfg(target_os = "android")]
pub use crate::jni_methods_cache::call_java_static_method;
#[cfg(target_os = "android")]
pub use jni::signature;
#[cfg(target_os = "android")]
pub use jni_methods_cache::methods_cache::{
    java_method_cache_utils::{JavaArgs, MethodType, ReturnedValue},
    JavaMethodCache, JavaMethods,
};
#[cfg(target_os = "android")]
pub use signature::Primitive as JniPrimitive;
#[cfg(target_os = "android")]
pub use signature::ReturnType;

#[cfg(target_os = "android")]
pub mod prelude {
    use super::*;

    pub use jni_methods_cache::methods_cache::{
        java_method_cache_utils::{JavaArgs, MethodType, ReturnedValue},
        JavaMethodCache, JavaMethods,
    };
    pub use signature::Primitive as JniPrimitive;
    pub use signature::ReturnType;
}
