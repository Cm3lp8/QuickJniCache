use crate::jni_methods_cache::methods_cache::{JVMResponse, JVMResult, JVMResultSender};
pub use channel::ExecutorChannel;
pub use event_handler::run;
pub use executor_receiver::ExecutorReceiver;
use jvm_call_event::*;
pub use jvm_caller::JvmCaller;

mod jvm_call_event {
    use std::time::Instant;

    use super::*;
    use crate::{JavaArgs, ReturnType};

    #[derive(Debug)]
    pub enum JvmCallEvent {
        CallStaticMethod {
            response_channel: JVMResultSender,
            class_name: String,
            method_name: String,
            sig: String,
            args: JavaArgs,
            return_type: ReturnType,
            returned_object_id: Option<String>,
            instant: Instant,
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
        jvm_result: JVMResult,
    }
    impl JvmCaller {
        pub fn new(sender: crossbeam_channel::Sender<JvmCallEvent>) -> Self {
            JvmCaller {
                event_channel: sender,
                jvm_result: JVMResult::new(),
            }
        }
        pub fn call_static_method<T: 'static + JVMResponse>(
            &self,
            class_name: &str,
            method_name: &str,
            sig: &str,
            args: JavaArgs,
            return_type: ReturnType,
            returned_object_id: Option<String>,
        ) -> Result<T, ()> {
            let instant = std::time::Instant::now();
            let msg = JvmCallEvent::CallStaticMethod {
                response_channel: self.jvm_result.get_sender(),
                class_name: class_name.to_owned(),
                method_name: method_name.to_owned(),
                sig: sig.to_owned(),
                args,
                return_type,
                returned_object_id,
                instant,
            };

            if let Err(e) = self.event_channel.send(msg) {
                println!("error in calling jvm")
            }

            if let Ok(res) = self.jvm_result.wait_for_result() {
                if let Ok(r) = res.to_value() {
                    Ok(*r)
                } else {
                    Err(())
                }
            } else {
                Err(())
            }
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
            match event {
                JvmCallEvent::CallStaticMethod {
                    response_channel,
                    class_name,
                    method_name,
                    sig,
                    args,
                    return_type,
                    returned_object_id,
                    instant,
                } => {
                    println!(
                        "\n      time receiver event call jni [{:?}]   class [{:?}]",
                        instant.elapsed(),
                        class_name
                    );
                    if let Ok(res) = call_java_static_method_internal(
                        class_name.as_str(),
                        method_name.as_str(),
                        sig.as_str(),
                        args,
                        return_type,
                        returned_object_id,
                    ) {
                        if let Ok(_) = response_channel.send(res) {}
                    }
                }
                _ => {}
            }
        }
    }
}

mod jvm_method_caller {
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
        let res = unsafe {
            JAVAMETHODCACHE.call_static_method(
                class_name,
                method_name,
                sig,
                args,
                return_type,
                returned_object_id,
            )
        };
        res
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
