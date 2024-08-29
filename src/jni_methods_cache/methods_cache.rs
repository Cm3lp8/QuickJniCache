use jni::objects::{
    JClass, JDoubleArray, JMethodID, JObject, JStaticMethodID, JString, JValue, JValueOwned,
};
use jni::signature::ReturnType;
use jni::sys::jobject;
use jni::JavaVM;
use std::mem;

use crate::jni_methods_cache::executor::ExecutorChannel;
pub use java_method_build_tools::*;
pub use java_vm_response::JVMResponse;
pub use java_vm_response::JVMResult;
pub use java_vm_response::JVMResultSender;

pub mod java_method_build_tools {
    use self::j_object_ref::{JavaMethodsList, JavaMethodsListRefs, JavaStaticMethodsList};
    use self::j_object_store::JObjectStore;
    use self::java_method_cache_utils::{JavaArgs, MethodType, ReturnedValue};
    use self::native_class_finder::NativeClassFinder;
    use self::standard_class_finder::{StandardClassCache, StandardClassPreList};
    use super::*;

    #[derive(Default)]
    pub enum JavaMethods<'a> {
        Cache {
            cache: JavaMethodCache<'a>,
        },
        #[default]
        None,
    }

    impl<'a: 'static> JavaMethods<'a> {
        pub fn build_cache(&'a mut self) {
            match self {
                JavaMethods::Cache { cache } => {
                    if !cache.cache_builded {
                        cache.standard_class_cache.build_standard_class_list(
                            &mut cache.env_2,
                            &cache.standard_class_pre_list,
                        );
                        cache.static_method_list.build_list_with_ref(
                            &mut cache.env_2,
                            &mut cache.java_methods_list_ref,
                            &mut cache.native_class_finder,
                        );
                        cache.method_list.build_list_with_ref(
                            &mut cache.env_2,
                            &mut cache.java_methods_list_ref,
                            &mut cache.native_class_finder_2,
                        );
                        cache.cache_builded = true;
                        println!("JavaMethodCache is correctly initialized !");
                    }
                }
                _ => {
                    println!("Attemps to build cache but no cache instance is instanciated");
                }
            }
        }
        pub fn call_method(
            &'a mut self,
            class: &str,
            name: &str,
            sig: &str,
            args: JavaArgs,
            return_type: ReturnType,
            returned_object_id: Option<String>, //in case of JObject as returned value -> id is a string
                                                //to get the object stored
        ) -> std::result::Result<ReturnedValue, String> {
            match self {
                JavaMethods::Cache { cache } => {
                    if let Ok(r) =
                        cache.call_method(class, name, sig, args, return_type, returned_object_id)
                    {
                        Ok(r)
                    } else {
                        Err(
                            "error while receiving response from java static method call "
                                .to_string(),
                        )
                    }
                }
                _ => Err("No cache initialized in java_method_cache !".to_string()),
            }
        }
        pub fn call_static_method(
            &'a mut self,
            class: &str,
            name: &str,
            sig: &str,
            args: JavaArgs,
            return_type: ReturnType,
            returned_object_id: Option<String>, //in case of JObject as returned value -> id is a string
                                                //to get the object stored
        ) -> std::result::Result<ReturnedValue, String> {
            match self {
                JavaMethods::Cache { cache } => {
                    let time = std::time::Instant::now();
                    if let Ok(r) = cache.call_static_method(
                        class,
                        name,
                        sig,
                        args,
                        return_type,
                        returned_object_id,
                    ) {
                        Ok(r)
                    } else {
                        Err(
                            "error while receiving response from java static method call "
                                .to_string(),
                        )
                    }
                }
                _ => Err("No cache initialized in java_method_cache !".to_string()),
            }
        }
    }

    pub struct JavaMethodCache<'a> {
        env: jni::JNIEnv<'a>,
        env_2: jni::JNIEnv<'a>,
        activity: &'a mut JObject<'a>,
        java_vm: &'a JavaVM,
        instanciate_jobjects: JObjectStore<'a>,
        native_class_finder: NativeClassFinder<'a>,
        native_class_finder_2: NativeClassFinder<'a>,
        standard_class_pre_list: StandardClassPreList,
        standard_class_cache: StandardClassCache<'a>,
        java_methods_list_ref: JavaMethodsListRefs<'a>,
        static_method_list: JavaStaticMethodsList<'a>,
        method_list: JavaMethodsList<'a>,
        cache_builded: bool,
        executor_channel: ExecutorChannel,
    }

    pub struct JavaMethodCacheBuilder<'a> {
        pub env: Option<jni::JNIEnv<'a>>,
        pub activity: Option<&'a mut JObject<'a>>,
        pub java_vm: Option<&'a JavaVM>,
        pub standard_class_pre_list: StandardClassPreList,
        pub java_methods_list_ref: JavaMethodsListRefs<'a>,
        pub cache_builded: bool,
    }

    impl<'a> JavaMethodCacheBuilder<'a> {
        pub fn build(&mut self, build_cb: impl FnOnce(JavaMethodCache<'a>)) {
            let executor_channel = ExecutorChannel::new();
            let mut env = mem::replace(&mut self.env, None);
            let mut activity = mem::replace(&mut self.activity, None);
            let mut java_vm = mem::replace(&mut self.java_vm, None);
            let mut env_2 = java_vm.as_mut().unwrap().get_env().unwrap();

            let mut java_methods_list = JavaStaticMethodsList::new();
            let mut env_3 = java_vm.as_mut().unwrap().get_env().unwrap();
            let mut env_4 = java_vm.as_mut().unwrap().get_env().unwrap();

            let mut native_class_finder = native_class_loader_construction(
                env_3,
                &mut activity.as_mut().expect("No activity attached, build fn"),
            );
            let mut native_class_finder_2 = native_class_loader_construction(
                env_4,
                &mut activity.as_mut().expect("No activity attached, build fn"),
            );

            let mut j_object_store = JObjectStore::new();

            let activity_raw_ptr: jobject = activity
                .as_ref()
                .expect("No activity for raw pointer")
                .as_raw();

            j_object_store.add_object_with_id("native_activity", unsafe {
                JObject::from_raw(activity_raw_ptr)
            });

            let mut java_method_cache = JavaMethodCache {
                env: env.expect("no env attached to JavaMethodCache"),
                env_2,
                activity: activity
                    .expect("no activity instance object ref attached to javaMethodCache"),
                java_vm: java_vm.expect("no java_vm reference attached to javaMethodCache"),
                native_class_finder,
                native_class_finder_2,
                standard_class_pre_list: std::mem::replace(
                    &mut self.standard_class_pre_list,
                    StandardClassPreList::new(),
                ),
                standard_class_cache: StandardClassCache::new(),
                instanciate_jobjects: j_object_store,
                java_methods_list_ref: std::mem::replace(
                    &mut self.java_methods_list_ref,
                    JavaMethodsListRefs::new(),
                ),
                static_method_list: JavaStaticMethodsList::new(),
                method_list: JavaMethodsList::new(),
                cache_builded: false,
                executor_channel,
            };

            build_cb(java_method_cache);
        }

        pub fn add_standard_class_name(&mut self, class_name: &str) -> &mut Self {
            self.standard_class_pre_list
                .add_standard_class_name(class_name);
            self
        }

        pub fn add_java_method(
            &mut self,
            method_type: MethodType,
            class: &str,
            method_name: &str,
            signature: &str,
        ) -> &mut Self {
            self.java_methods_list_ref.add(
                &self
                    .env
                    .as_ref()
                    .expect("A JNIEnv has to be attached first"),
                method_type,
                class,
                method_name,
                signature,
            );
            self
        }
    }
    impl<'a: 'static> JavaMethodCache<'a> {
        pub fn print_method_list(&self) {
            println!("List of the cached java methods :");
            for method in self.method_list.methods_list() {
                println!(
                    "- {:?}- [{:?}]- [{:?}]",
                    method.method_name(),
                    method.method_signature(),
                    method.method_class()
                );
            }
        }

        pub fn set_to_java_methods(self) -> JavaMethods<'a> {
            JavaMethods::Cache { cache: self }
        }
        pub fn call_static_method(
            &'a mut self,
            class: &str,
            method_name: &str,
            sig: &str,
            args: JavaArgs,
            return_type: ReturnType,
            object_id: Option<String>,
        ) -> std::result::Result<ReturnedValue, ()> {
            let now = std::time::Instant::now();
            let find_method = self.static_method_list.list_as_mut().iter_mut().find(|m| {
                m.method_class() == class
                    && m.method_name() == method_name
                    && m.method_signature() == sig
            });

            let mut res: Option<JValueOwned> = None;

            if let Some(find_method) = find_method {
                if let Some(args) = args.to_jvalue(&self.instanciate_jobjects) {
                    let call_time = std::time::Instant::now();
                    unsafe {
                        let result: JValueOwned = self
                            .env
                            .call_static_method_unchecked(
                                JClass::from_raw(find_method.instance_ref().as_raw()),
                                find_method.method_id(),
                                return_type,
                                &args[..],
                            )
                            .unwrap_or_else(|_| {
                                panic!("Error in calling static method [{}]", method_name);
                            });

                        println!("l [{:?}]", result);

                        res = Some(result);
                    }
                } else {
                    let now = std::time::Instant::now();
                    unsafe {
                        let result: JValueOwned = self
                            .env
                            .call_static_method_unchecked(
                                JClass::from_raw(find_method.instance_ref().as_raw()),
                                find_method.method_id(),
                                return_type,
                                &[],
                            )
                            .unwrap_or_else(|e| {
                                panic!(
                                    "Error in calling static method [{}]\n [{:?}]",
                                    method_name, e
                                );
                            });
                        res = Some(result);
                    }
                };
            };

            if let Some(result) = res {
                let result = Ok(ReturnedValue::get_result_type(
                    &mut self.env,
                    result,
                    &self.standard_class_cache,
                    object_id,
                    &mut self.instanciate_jobjects,
                ));
                result
            } else {
                Err(())
            }
        }
        pub fn call_method(
            &'a mut self,
            class: &str,
            method_name: &str,
            sig: &str,
            args: JavaArgs,
            return_type: ReturnType,
            object_id: Option<String>,
        ) -> std::result::Result<ReturnedValue, ()> {
            let find_method = self.method_list.list_as_mut().iter_mut().find(|m| {
                m.method_class() == class
                    && m.method_name() == method_name
                    && m.method_signature() == sig
            });

            let mut res: Option<JValueOwned> = None;

            if let Some(find_method) = find_method {
                if let Some(args) = args.to_jvalue(&self.instanciate_jobjects) {
                    unsafe {
                        let result: JValueOwned = self
                            .env
                            .call_method_unchecked(
                                JObject::from_raw(find_method.instance_ref().as_raw()),
                                find_method.method_id(),
                                return_type,
                                &args[..],
                            )
                            .unwrap_or_else(|_| {
                                panic!("Error in calling static method [{}]", method_name);
                            });

                        res = Some(result);
                    }
                } else {
                    unsafe {
                        let result: JValueOwned = self
                            .env
                            .call_method_unchecked(
                                JObject::from_raw(find_method.instance_ref().as_raw()),
                                find_method.method_id(),
                                return_type,
                                &[],
                            )
                            .unwrap_or_else(|_| {
                                panic!("Error in calling static method [{}]", method_name);
                            });
                        res = Some(result);
                    }
                };
            };

            if let Some(result) = res {
                Ok(ReturnedValue::get_result_type(
                    &mut self.env,
                    result,
                    &self.standard_class_cache,
                    object_id,
                    &mut self.instanciate_jobjects,
                ))
            } else {
                Err(())
            }
        }
    }

    fn native_class_loader_construction<'a>(
        mut env: jni::JNIEnv<'a>,
        activity: &mut JObject,
    ) -> NativeClassFinder<'a> {
        let class_loader_name = "java/lang/ClassLoader";
        let na_activity_class_name = "android/app/NativeActivity";

        let na_class: jni::objects::JClass = env
            .find_class(na_activity_class_name)
            .unwrap_or_else(|_| panic!("No NativeActivity class name found !"));

        let class_loader_method: jni::objects::JMethodID = env
            .get_method_id(na_class, "getClassLoader", "()Ljava/lang/ClassLoader;")
            .unwrap_or_else(|_| panic!("didn't find getClassLoader Method !"));

        let class_loader_instance: jni::objects::JObject = unsafe {
            env.call_method_unchecked(
                &activity,
                class_loader_method,
                jni::signature::ReturnType::Object,
                &[],
            )
            .unwrap_or_else(|_| panic!("cannot call the method class loader on activity !"))
            .l()
            .unwrap()
        };

        let class_loader_class = env
            .find_class(class_loader_name)
            .unwrap_or_else(|_| panic!("No classloader class name found !"));

        let find_class: jni::objects::JMethodID = env
            .get_method_id(
                class_loader_class,
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
            )
            .unwrap_or_else(|_| panic!("didn't find loadClass Method in class_loader class !"));

        NativeClassFinder::new(env, class_loader_instance, find_class)
    }

    mod native_class_finder {

        use super::*;

        pub struct NativeClassFinder<'a> {
            pub env: jni::JNIEnv<'a>,
            pub class_loader: jni::objects::JObject<'a>,
            pub find_class_method: JMethodID,
        }

        impl<'a> NativeClassFinder<'a> {
            pub fn new(
                env: jni::JNIEnv<'a>,
                class_loader: jni::objects::JObject<'a>,
                find_class_method: JMethodID,
            ) -> Self {
                Self {
                    env,
                    class_loader,
                    find_class_method,
                }
            }
        }
    }

    pub mod j_object_ref {

        use jni::JNIEnv;

        use super::*;

        pub struct JavaMethod<'a> {
            instance_ref: JObject<'a>,
            method_class: String,
            method_name: String,
            method_signature: String,
            method_id: JMethodID,
        }
        impl<'a> JavaMethod<'a> {
            pub fn instance_ref(&'a self) -> &'a JObject<'a> {
                &self.instance_ref
            }

            pub fn method_id(&self) -> &JMethodID {
                &self.method_id
            }
            pub fn method_class(&self) -> &str {
                self.method_class.as_str()
            }
            pub fn method_name(&self) -> &str {
                self.method_name.as_str()
            }
            pub fn method_signature(&self) -> &str {
                self.method_signature.as_str()
            }
            pub fn new(
                class_instance: JObject<'a>,
                method_class: &str,
                method_name: &str,
                signature: &str,
                method_id: JMethodID,
            ) -> JavaMethod<'a> {
                Self {
                    instance_ref: class_instance,
                    method_class: method_class.to_owned(),
                    method_name: method_name.to_owned(),
                    method_signature: signature.to_owned(),
                    method_id,
                }
            }
        }

        pub struct JavaMethodsList<'a> {
            methods_list: Vec<JavaMethod<'a>>,
        }

        impl<'a> JavaMethodsList<'a> {
            pub fn new() -> Self {
                Self {
                    methods_list: vec![],
                }
            }
            pub fn methods_list(&self) -> &Vec<JavaMethod<'a>> {
                &self.methods_list
            }
            pub fn list_as_mut(&mut self) -> &mut Vec<JavaMethod<'a>> {
                &mut self.methods_list
            }
            pub fn build_list_with_ref(
                &mut self,
                env: &mut JNIEnv,
                list_refs: &mut JavaMethodsListRefs,
                native_class_finder: &'a mut NativeClassFinder,
            ) {
                let env = &mut native_class_finder.env;

                for method_ref in &mut list_refs.methods_list.iter().filter(|item| {
                    if let MethodType::NonStatic = item.method_type {
                        true
                    } else {
                        false
                    }
                }) {
                    let instance_ref: JObject = unsafe {
                        env.call_method_unchecked(
                            &native_class_finder.class_loader,
                            native_class_finder.find_class_method,
                            ReturnType::Object,
                            &[JValue::Object(&method_ref.class).as_jni()],
                        )
                        .unwrap_or_else(|_| {
                            panic!("can't instanciate [{:?}]", method_ref.method_name)
                        })
                        .l()
                        .unwrap()
                    };

                    let instance_ref_2: JObject = unsafe {
                        env.call_method_unchecked(
                            &native_class_finder.class_loader,
                            native_class_finder.find_class_method,
                            ReturnType::Object,
                            &[JValue::Object(&method_ref.class).as_jni()],
                        )
                        .unwrap_or_else(|_| {
                            panic!("can't instanciate [{:?}]", method_ref.method_name)
                        })
                        .l()
                        .unwrap()
                    };
                    let method_id: JMethodID = env
                        .get_method_id(
                            JClass::from(instance_ref),
                            method_ref.method_name().as_str(),
                            method_ref.method_signature().as_str(),
                        )
                        .unwrap();
                    let new_method = JavaMethod::new(
                        instance_ref_2,
                        method_ref.class_name.as_str(),
                        method_ref.method_name().as_str(),
                        method_ref.method_signature().as_str(),
                        method_id,
                    );

                    self.methods_list.push(new_method);
                }
            }
        }
        #[derive(Debug)]
        pub struct JavaStaticMethod<'a> {
            instance_ref: JObject<'a>,
            method_class: String,
            method_name: String,
            method_signature: String,
            method_id: JStaticMethodID,
        }

        impl<'a> JavaStaticMethod<'a> {
            pub fn instance_ref(&'a self) -> &'a JObject<'a> {
                &self.instance_ref
            }

            pub fn method_id(&self) -> &JStaticMethodID {
                &self.method_id
            }
            pub fn method_class(&self) -> &str {
                self.method_class.as_str()
            }
            pub fn method_name(&self) -> &str {
                self.method_name.as_str()
            }
            pub fn method_signature(&self) -> &str {
                self.method_signature.as_str()
            }
            pub fn new(
                class_instance: JObject<'a>,
                method_class: &str,
                method_name: &str,
                signature: &str,
                method_id: JStaticMethodID,
            ) -> JavaStaticMethod<'a> {
                Self {
                    instance_ref: class_instance,
                    method_class: method_class.to_owned(),
                    method_name: method_name.to_owned(),
                    method_signature: signature.to_owned(),
                    method_id,
                }
            }
        }

        pub struct JavaStaticMethodsList<'a> {
            methods_list: Vec<JavaStaticMethod<'a>>,
        }

        impl<'a> JavaStaticMethodsList<'a> {
            pub fn new() -> Self {
                Self {
                    methods_list: vec![],
                }
            }
            pub fn methods_list(&self) -> &Vec<JavaStaticMethod<'a>> {
                &self.methods_list
            }
            pub fn list_as_mut(&mut self) -> &mut Vec<JavaStaticMethod<'a>> {
                &mut self.methods_list
            }
            pub fn build_list_with_ref(
                &mut self,
                env: &mut JNIEnv,
                list_refs: &mut JavaMethodsListRefs,
                native_class_finder: &'a mut NativeClassFinder,
            ) {
                let env = &mut native_class_finder.env;

                for method_ref in &mut list_refs.methods_list.iter().filter(|item| {
                    if let MethodType::Static = item.method_type {
                        true
                    } else {
                        println!("non static found");
                        false
                    }
                }) {
                    let instance_ref: JObject = unsafe {
                        env.call_method_unchecked(
                            &native_class_finder.class_loader,
                            native_class_finder.find_class_method,
                            ReturnType::Object,
                            &[JValue::Object(&method_ref.class).as_jni()],
                        )
                        .unwrap_or_else(|_| {
                            panic!("can't instanciate [{:?}]", method_ref.method_name)
                        })
                        .l()
                        .unwrap()
                    };

                    let instance_ref_2: JObject = unsafe {
                        env.call_method_unchecked(
                            &native_class_finder.class_loader,
                            native_class_finder.find_class_method,
                            ReturnType::Object,
                            &[JValue::Object(&method_ref.class).as_jni()],
                        )
                        .unwrap_or_else(|_| {
                            panic!("can't instanciate [{:?}]", method_ref.method_name)
                        })
                        .l()
                        .unwrap()
                    };
                    let method_id: JStaticMethodID = env
                        .get_static_method_id(
                            JClass::from(instance_ref),
                            method_ref.method_name().as_str(),
                            method_ref.method_signature().as_str(),
                        )
                        .unwrap();
                    let new_method = JavaStaticMethod::new(
                        instance_ref_2,
                        method_ref.class_name.as_str(),
                        method_ref.method_name().as_str(),
                        method_ref.method_signature().as_str(),
                        method_id,
                    );

                    self.methods_list.push(new_method);
                }
            }
        }

        #[derive(Debug)]
        pub struct JavaMethodsListRefs<'a> {
            methods_list: Vec<MethodItemRef<'a>>,
        }

        impl<'a> JavaMethodsListRefs<'a> {
            pub fn new() -> Self {
                Self {
                    methods_list: vec![],
                }
            }

            pub fn add(
                &mut self,
                env: &JNIEnv<'a>,
                method_type: MethodType,
                class_name: &str,
                method_name: &str,
                signature: &str,
            ) {
                let class: jni::objects::JObject = env.new_string(class_name).unwrap().into();
                let method_name: String = method_name.to_string();
                let signature: String = signature.to_string();

                let new_method = MethodItemRef::new(
                    method_type,
                    class,
                    class_name.to_owned(),
                    method_name,
                    signature,
                );

                self.methods_list.push(new_method);
            }
        }

        #[derive(Debug)]
        pub struct MethodItemRef<'a> {
            class: jni::objects::JObject<'a>,
            method_type: MethodType,
            class_name: String,
            method_name: String,
            method_signature: String,
        }

        impl<'a> MethodItemRef<'a> {
            pub fn new(
                method_type: MethodType,
                class: jni::objects::JObject<'a>,
                class_name: String,
                method_name: String,
                method_signature: String,
            ) -> Self {
                Self {
                    method_type,
                    class,
                    class_name,
                    method_name,
                    method_signature,
                }
            }
            pub fn method_name(&self) -> String {
                self.method_name.to_string()
            }
            pub fn method_signature(&self) -> String {
                self.method_signature.to_string()
            }
        }
    }

    pub mod java_method_cache_utils {
        use std::fmt::Debug;

        use jni::objects::{JIntArray, JValueGen};

        use super::*;

        #[derive(Debug)]
        pub enum MethodType {
            Static,
            NonStatic,
        }

        #[derive(Debug)]
        pub enum JavaArgs {
            JObject(String),
            I32(i32),
            F32,
            None,
            Array(Vec<JavaArgs>),
        }

        impl JavaArgs {
            pub fn to_jvalue(
                &self,
                instanciated_j_objects: &JObjectStore,
            ) -> Option<Vec<jni::sys::jvalue>> {
                match self {
                    JavaArgs::JObject(o_id) => {
                        if let Some(found_object) = instanciated_j_objects.find(o_id) {
                            Some(vec![JValueGen::Object(found_object).as_jni()])
                        } else {
                            None
                        }
                    }
                    JavaArgs::Array(arr) => {
                        let mut args_arr: Vec<jni::sys::jvalue> = vec![];

                        for item in arr.iter() {
                            match item {
                                JavaArgs::JObject(o_id) => {
                                    if let Some(found_object) = instanciated_j_objects.find(o_id) {
                                        args_arr.push(JValueGen::Object(found_object).as_jni());
                                    }
                                }
                                JavaArgs::I32(v) => {
                                    args_arr.push(JValue::from(*v).as_jni());
                                }
                                _ => {}
                            };
                        }

                        if args_arr.len() > 0 {
                            Some(args_arr)
                        } else {
                            None
                        }
                    }
                    JavaArgs::None => None,
                    _ => None,
                }
            }
        }

        #[derive(Clone)]
        pub enum ReturnedValue {
            I32(i32),
            JObject(String),
            Void,
            Long(i64),
            ArrayFloat,
            String(String),
            VecDouble(Vec<f64>),
            VecUsize(Vec<usize>),
        }
        enum Extractible<'a> {
            Yes(&'a str, JObject<'a>, &'a mut jni::JNIEnv<'a>),
            No,
        }
        impl ReturnedValue {
            pub fn get_result_type<'a>(
                env: &'a mut jni::JNIEnv<'a>,
                result: JValueOwned<'a>,
                standard_class_list: &'a StandardClassCache<'a>,
                object_store_id: Option<String>,
                j_object_method_store: &mut JObjectStore<'a>,
            ) -> ReturnedValue {
                match result {
                    JValueGen::Object(o) => {
                        match check_if_extractible_classes(env, standard_class_list, o.as_raw()) {
                            Extractible::Yes(class_name, obj, env_passed) => {
                                extract_value(env_passed, class_name, obj).unwrap()
                            }
                            Extractible::No => {
                                j_object_method_store.add_object_with_id(
                                    &object_store_id.clone().expect("no object identifier given"),
                                    o,
                                );
                                ReturnedValue::JObject(
                                    object_store_id.expect("no object identifer given"),
                                )
                            }
                        }
                    }
                    JValueGen::Long(long) => ReturnedValue::Long(long),
                    _ => ReturnedValue::Void,
                }
            }
        }

        fn extract_value<'a>(
            env: &mut jni::JNIEnv<'a>,
            class_name: &str,
            object: JObject<'a>,
        ) -> Option<ReturnedValue> {
            match class_name {
                "java/lang/String" => {
                    let j_string: JString = JString::from(object);
                    let rust_string: String =
                        unsafe { env.get_string_unchecked(&j_string).unwrap().into() };
                    Some(ReturnedValue::String(rust_string))
                }
                "[D" => {
                    let j_double_array: JDoubleArray = JDoubleArray::from(object);

                    let length: usize = env.get_array_length(&j_double_array).unwrap() as usize;
                    let mut buffer: Vec<jni::sys::jdouble> = vec![0.0; length];

                    let _double_array_region = env
                        .get_double_array_region(j_double_array, 0, &mut buffer)
                        .unwrap();

                    Some(ReturnedValue::VecDouble(
                        buffer.into_iter().map(|i| i as f64).collect(),
                    ))
                }
                "[I" => {
                    let j_int_array: JIntArray = JIntArray::from(object);

                    let length: usize = env.get_array_length(&j_int_array).unwrap() as usize;
                    let mut buffer: Vec<jni::sys::jint> = vec![0; length];
                    let _int_array_region = env
                        .get_int_array_region(j_int_array, 0, &mut buffer)
                        .unwrap();
                    Some(ReturnedValue::VecUsize(
                        buffer.into_iter().map(|i| i as usize).collect(),
                    ))
                }
                _ => None,
            }
        }

        fn check_if_extractible_classes<'a>(
            env: &'a mut jni::JNIEnv<'a>,
            standard_class_list: &'a StandardClassCache<'a>,
            o: *mut jni::sys::_jobject,
        ) -> Extractible<'a> {
            let mut extractible: Extractible = Extractible::No;

            let object_class: JClass = env
                .get_object_class(unsafe { JObject::from_raw(o) })
                .unwrap();
            for class in standard_class_list.inner.iter() {
                let (class_name_ref, class) = class.get_class_ref();

                let is_std_class: bool = env
                    .is_instance_of(unsafe { JObject::from_raw(o) }, class)
                    .unwrap();

                if is_std_class {
                    extractible =
                        Extractible::Yes(class_name_ref, unsafe { JObject::from_raw(o) }, env);
                    break;
                }
            }
            extractible
        }
    }

    mod j_object_store {

        use super::*;

        pub struct JObjectStore<'a> {
            inner: Vec<JObjectEntry<'a>>,
        }

        impl<'a> JObjectStore<'a> {
            pub fn new() -> Self {
                Self { inner: vec![] }
            }

            pub fn find(&self, j_object_id: &str) -> Option<&JObject<'a>> {
                if let Some(entry) = self
                    .inner
                    .iter()
                    .find(|item| item.id.as_str() == j_object_id)
                {
                    Some(&entry.object)
                } else {
                    None
                }
            }

            pub fn add_object_with_id(&mut self, id: &str, j_object: JObject<'a>) -> &mut Self {
                let new_object = JObjectEntry::new(id, j_object);
                self.inner.push(new_object);
                self
            }
        }

        struct JObjectEntry<'a> {
            id: String,
            object: JObject<'a>,
        }

        impl<'a> JObjectEntry<'a> {
            fn new(id: &str, object: JObject<'a>) -> JObjectEntry<'a> {
                Self {
                    id: id.to_string(),
                    object,
                }
            }
        }
    }

    pub mod standard_class_finder {
        use super::*;

        pub struct StandardClassPreList {
            pub list: Vec<String>,
        }

        impl StandardClassPreList {
            pub fn new() -> Self {
                Self { list: vec![] }
            }

            pub fn add_standard_class_name(&mut self, class_name: &str) {
                self.list.push(class_name.to_string());
            }
        }

        pub struct StandardClassCache<'a> {
            pub inner: Vec<StandardClass<'a>>,
            index: usize,
        }

        impl<'a> StandardClassCache<'a> {
            pub fn new() -> Self {
                Self {
                    inner: vec![],
                    index: 0,
                }
            }

            pub fn build_standard_class_list(
                &mut self,
                env: &mut jni::JNIEnv<'a>,
                standard_class_pre_list: &StandardClassPreList,
            ) {
                for class_name in standard_class_pre_list.list.iter() {
                    match env.find_class(class_name) {
                        Ok(class) => {
                            let new_std_class = StandardClass::new(class_name, class);
                            self.inner.push(new_std_class);
                        }
                        _ => {
                            println!("---Class [{:?}] not found in JNIEnv", class_name)
                        }
                    }
                }
            }
        }

        pub struct StandardClass<'a> {
            class_name: String,
            class: JClass<'a>,
        }
        impl<'a> StandardClass<'a> {
            pub fn new(class_name: &str, class: JClass<'a>) -> Self {
                Self {
                    class_name: class_name.to_string(),
                    class,
                }
            }
            pub fn class(&self) -> &JClass {
                &self.class
            }
            pub fn class_name(&self) -> &str {
                self.class_name.as_str()
            }

            pub fn get_class_ref(&self) -> (&str, &JClass) {
                (self.class_name.as_str(), &self.class)
            }
        }
    }
}

mod java_vm_response {
    use std::{fmt::Debug, ops::Deref, time::Instant};

    use crate::ReturnedValue;

    #[derive(Debug)]
    pub struct JVMResult {
        channel: (
            kanal::Sender<(ReturnedValue, Instant)>,
            kanal::Receiver<(ReturnedValue, Instant)>,
        ),
    }
    impl JVMResult {
        pub fn new() -> Self {
            Self {
                channel: kanal::bounded(1),
            }
        }
        pub fn get_sender(&self) -> JVMResultSender {
            JVMResultSender {
                sender: self.channel.0.clone(),
            }
        }

        pub fn wait_for_result(&self) -> std::result::Result<JVMResponseWrapper, ()> {
            if let Ok((res, now)) = self.channel.1.recv() {
                let result: JVMResponseWrapper = match res {
                    ReturnedValue::I32(i) => JVMResponseWrapper::new(i),
                    ReturnedValue::Long(v) => JVMResponseWrapper::new(v),
                    ReturnedValue::String(s) => JVMResponseWrapper::new(s),
                    ReturnedValue::VecUsize(u) => JVMResponseWrapper::new(u),
                    _ => JVMResponseWrapper::new(()),
                };

                Ok(result)
            } else {
                Err(())
            }
        }
    }

    #[derive(Debug)]
    pub struct JVMResultSender {
        sender: kanal::Sender<(ReturnedValue, Instant)>,
    }

    impl JVMResultSender {
        pub fn send(&self, value: ReturnedValue) -> std::result::Result<(), kanal::SendError> {
            let now = std::time::Instant::now();
            self.sender.send((value, now))
        }
    }

    pub struct JVMResponseWrapper {
        inner: Box<dyn DynJVMResponse>,
    }

    impl JVMResponseWrapper {
        fn new(value: impl JVMResponse + 'static) -> Self {
            let value = Box::new(value);
            JVMResponseWrapper { inner: value }
        }

        pub fn to_value<T: 'static + JVMResponse>(
            &self,
        ) -> std::result::Result<Box<T>, Box<dyn std::any::Any>> {
            self.inner.get_value().downcast::<T>()
        }
    }
    impl Deref for JVMResponseWrapper {
        type Target = dyn DynJVMResponse;
        fn deref(&self) -> &Self::Target {
            &*self.inner
        }
    }

    pub trait DynJVMResponse {
        fn get_value(&self) -> Box<dyn std::any::Any>;
    }

    impl<T: 'static + JVMResponse> DynJVMResponse for T
    where
        T::Item: 'static,
    {
        fn get_value(&self) -> Box<dyn std::any::Any> {
            Box::new(JVMResponse::get_value(self))
        }
    }
    pub trait JVMResponse {
        type Item;
        fn get_value(&self) -> Self::Item;
    }
    impl JVMResponse for () {
        type Item = ();
        fn get_value(&self) -> Self::Item {
            *self
        }
    }

    impl JVMResponse for i64 {
        type Item = i64;
        fn get_value(&self) -> Self::Item {
            *self
        }
    }
    impl JVMResponse for String {
        type Item = String;
        fn get_value(&self) -> Self::Item {
            self.to_owned()
        }
    }
    impl JVMResponse for i32 {
        type Item = i32;
        fn get_value(&self) -> Self::Item {
            *self
        }
    }
    impl JVMResponse for Vec<usize> {
        type Item = Vec<usize>;
        fn get_value(&self) -> Self::Item {
            self.to_vec()
        }
    }
}
