mod jni_methods_cache;
mod platform;

pub use crate::jni_methods_cache::call_java_method;
pub use crate::jni_methods_cache::call_java_static_method;
pub use jni::signature;
pub use jni_methods_cache::methods_cache::{
    java_method_cache_utils::{JavaArgs, MethodType, ReturnedValue},
    JavaMethodCache, JavaMethods,
};
pub use signature::Primitive as JniPrimitive;
pub use signature::ReturnType;

pub mod prelude {
    use super::*;

    pub use jni_methods_cache::methods_cache::{
        java_method_cache_utils::{JavaArgs, MethodType, ReturnedValue},
        JavaMethodCache, JavaMethods,
    };
    pub use signature::Primitive as JniPrimitive;
    pub use signature::ReturnType;
}
