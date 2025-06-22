# QuickJniCache

Rust crate to facilitate custom Java class usage in the JNI environment, in addition with the Winit crate for android platform.

This crate is currently under development!

## Overview

QuickJniCache is designed to simplify the interaction between Rust and custom Java classes through the Java Native Interface (JNI). This crate is aimed for Android applications that require efficient JNI calls and need to cache method references to improve performance and reliability.


## Purpose

In an Android application using rust, I encountered an issue where the Java environment couldn't find custom classes. To solve this, I created a mechanism to cache methods via the Activity object, ensuring that custom Java classes and methods are readily available for JNI calls.

## Example

First, you need to register methods like this :

```rust 
JavaMethodCache::init(&app, |builder| {
        builder
            .add_standard_class_name("java/lang/String")
            .add_standard_class_name("[D")
            .add_standard_class_name("[I")
            .add_java_method(
                MethodType::Static,
                "com/example/new_test/VibraTools",
                "InitVibrator",
                "(Landroid/app/NativeActivity;)V",
            )
            .add_java_method(
                MethodType::Static,
                "com/example/new_test/GpsAuth",
                "requestLocationPermission",
                "(Landroid/app/NativeActivity;)V",
            )
            .add_java_method(
                MethodType::Static,
                "com/example/new_test/VibraTools",
                "vibrate",
                "(Landroid/app/NativeActivity;)V",
            )
            .add_java_method(
                MethodType::Static,
                "com/example/new_test/OpenCamera",
                "setCameraBackOnOpen",
                "(Landroid/app/NativeActivity;)V",
            )
            .add_java_method(
                MethodType::Static,
                "com/example/new_test/OpenCamera",
                "setCameraFrontOnOpen",
                "(Landroid/app/NativeActivity;)V",
            )
            .add_java_method(
                MethodType::Static,
                "com/example/new_test/OpenCamera",
                "closeCameraDevice",
                "()V",
            )
            .add_java_method(
                MethodType::Static,
                "com/example/new_test/AudioAuth",
                "audioPermission",
                "(Landroid/app/NativeActivity;)V",
            )
            .add_java_method(
                MethodType::Static,
                "com/example/new_test/NanoTime",
                "getRealNanoTimeSinceBoot",
                "()J",
            )
            .add_java_method(
                MethodType::Static,
                "com/example/new_test/LocalFilePath",
                "getLocalFileDir",
                "(Landroid/app/NativeActivity;)Ljava/lang/String;",
            )
            .add_java_method(
                MethodType::Static,
                "com/example/new_test/OpenCamera",
                "deviceResolution",
                "(Landroid/app/NativeActivity;)[I",
            );
    });

```
Then you can call the cache registered methods like this 

```rust
use jni_methods_cache::{call_java_static_method, JavaArgs, ReturnedValue};

pub fn start_front_capture() {
    let _ = call_java_static_method::<()>(
        "com/example/new_test/OpenCamera",
        "setCameraFrontOnOpen",
        "(Landroid/app/NativeActivity;)V",
        JavaArgs::JObject("native_activity".to_string()),
        ReturnType::Primitive(jni::signature::Primitive::Void),
        None,
    );
}
pub fn close_capture() {
    let _ = call_java_static_method::<()>(
        "com/example/new_test/OpenCamera",
        "closeCameraDevice",
        "()V",
        JavaArgs::None,
        ReturnType::Primitive(jni::signature::Primitive::Void),
        None,
    );
}
pub fn start_back_capture() {
    let _ = call_java_static_method::<()>(
        "com/example/new_test/OpenCamera",
        "setCameraBackOnOpen",
        "(Landroid/app/NativeActivity;)V",
        JavaArgs::JObject("native_activity".to_string()),
        ReturnType::Primitive(jni::signature::Primitive::Void),
        None,
    );
}
pub fn close_back_capture() {
    let _ = call_java_static_method::<()>(
        "com/example/new_test/OpenCamera",
        "closeBackCameraDevice",
        "()V",
        JavaArgs::None,
        ReturnType::Primitive(jni::signature::Primitive::Void),
        None,
    );
}
pub fn init_vibrator() {
    call_java_static_method::<()>(
        "com/example/new_test/VibraTools",
        "InitVibrator",
        "(Landroid/app/NativeActivity;)V",
        JavaArgs::JObject("native_activity".to_string()),
        ReturnType::Primitive(jni::signature::Primitive::Void),
        None,
    );
}
pub fn vibrate() {
    call_java_static_method::<()>(
        "com/example/new_test/VibraTools",
        "vibrate",
        "(Landroid/app/NativeActivity;)V",
        JavaArgs::JObject("native_activity".to_string()),
        ReturnType::Primitive(jni::signature::Primitive::Void),
        None,
    );
}

pub fn audio_permission() {
    let _ = call_java_static_method::<()>(
        "com/example/new_test/AudioAuth",
        "audioPermission",
        "(Landroid/app/NativeActivity;)V",
        JavaArgs::JObject("native_activity".to_string()),
        ReturnType::Primitive(jni::signature::Primitive::Void),
        None,
    );
}

pub fn get_real_time() -> Option<i64> {
    if let Ok(res) = call_java_static_method::<i64>(
        "com/example/new_test/NanoTime",
        "getRealNanoTimeSinceBoot",
        "()J",
        JavaArgs::None,
        ReturnType::Primitive(jni::signature::Primitive::Long),
        None,
    ) {
        Some(res)
    } else {
        None
    }
}
```
## Features

 - Efficient Caching: Cache frequently used Java classes and methods to avoid repetitive lookups.
 - Easy Integration: Seamlessly integrate Rust code with existing Android Java codebases.
 - Improved Performance: Reduce the overhead of JNI calls by caching method references.

## Use Cases

 - Calling custom Java methods from Rust in an Android application.
 - Ensuring custom Java classes are accessible in the JNI environment.
 - Enhancing the performance of Rust-Java interactions in Android apps


## Development Status

This crate is under active development. Contributions, suggestions, and feedback are welcome!
