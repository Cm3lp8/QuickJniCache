pub use channel::ExecutorChannel;
pub use event_handler::run;
pub use executor_receiver::ExecutorReceiver;
use jvm_call_event::*;
pub use jvm_caller::JvmCaller;

mod jvm_call_event {
    use super::*;
    use crate::{JavaArgs, ReturnType};

    #[derive(Debug)]
    pub enum JvmCallEvent {
        CallStaticMethod {
            class_name: String,
            method_name: String,
            sig: String,
            args: JavaArgs,
            return_type: ReturnType,
            returned_object_id: Option<String>,
        },
        CallMethod,
    }
}
mod channel {
    use super::*;

    pub struct ExecutorChannel {
        channel: (
            crossbeam_channel::Sender<JvmCallEvent>,
            crossbeam_channel::Receiver<JvmCallEvent>,
        ),
    }

    impl ExecutorChannel {
        pub fn new() -> Self {
            ExecutorChannel {
                channel: crossbeam_channel::unbounded(),
            }
        }

        pub fn get_receiver(&self) -> ExecutorReceiver {
            ExecutorReceiver::new(self.channel.1.clone())
        }

        pub fn get_jvm_caller(&self) -> JvmCaller {
            JvmCaller::new(self.channel.0.clone())
        }
    }
}

mod jvm_caller {
    use super::*;
    use crate::{JavaArgs, ReturnType};

    pub struct JvmCaller {
        event_channel: crossbeam_channel::Sender<JvmCallEvent>,
    }
    impl JvmCaller {
        pub fn new(sender: crossbeam_channel::Sender<JvmCallEvent>) -> Self {
            JvmCaller {
                event_channel: sender,
            }
        }
        pub fn call_static_method<T>(
            &self,
            class_name: &str,
            method_name: &str,
            sig: &str,
            args: JavaArgs,
            return_type: ReturnType,
            returned_object_id: Option<&str>,
        ) -> Result<T, ()> {
            let msg = JvmCallEvent::CallStaticMethod {
                class_name: class_name.to_owned(),
                method_name: method_name.to_owned(),
                sig: sig.to_owned(),
                args,
                return_type,
                returned_object_id: if let Some(v) = returned_object_id {
                    Some(v.to_owned())
                } else {
                    None
                },
            };

            if let Err(e) = self.event_channel.send(msg) {
                println!("error in calling jvm")
            } else {
                println!("jvm call done !!")
            }

            Err(())
        }
    }
}

mod executor_receiver {
    use super::*;

    pub struct ExecutorReceiver {
        event_receiver: crossbeam_channel::Receiver<JvmCallEvent>,
    }
    impl ExecutorReceiver {
        pub fn new(receiver: crossbeam_channel::Receiver<JvmCallEvent>) -> Self {
            ExecutorReceiver {
                event_receiver: receiver,
            }
        }

        pub fn receive(&self) -> Result<JvmCallEvent, crossbeam_channel::RecvError> {
            self.event_receiver.recv()
        }
    }
}

mod event_handler {
    use jvm_method_caller::call_java_static_method_internal;

    use super::*;

    pub fn run(event_receiver: ExecutorReceiver) {
        while let Ok(event) = event_receiver.receive() {
            println!("receiving call jvm in executor[{:?}] ", event);
            match event {
                JvmCallEvent::CallStaticMethod {
                    class_name,
                    method_name,
                    sig,
                    args,
                    return_type,
                    returned_object_id,
                } => {
                    println!("new jvm call processed! ");
                    call_java_static_method_internal(
                        class_name.as_str(),
                        method_name.as_str(),
                        sig.as_str(),
                        args,
                        return_type,
                        returned_object_id,
                    );
                }
                _ => {}
            }
        }
    }
}

mod jvm_method_caller {
    use super::*;
    use crate::jni_methods_cache::JAVAMETHODCACHE;
    use crate::jni_methods_cache::{JavaArgs, ReturnType, ReturnedValue};

    pub fn call_java_static_method_internal(
        class_name: &str,
        method_name: &str,
        sig: &str,
        args: JavaArgs,
        return_type: ReturnType,
        returned_object_id: Option<String>,
    ) -> std::result::Result<ReturnedValue, String> {
        unsafe {
            JAVAMETHODCACHE.call_static_method(
                class_name,
                method_name,
                sig,
                args,
                return_type,
                returned_object_id,
            )
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
}
