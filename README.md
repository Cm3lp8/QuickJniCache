# QuickJniCache

Rust crate to facilitate custom Java class usage in the JNI environment, in addition with the Winit crate for android platform.

This crate is currently under development!

## Overview

QuickJniCache is designed to simplify the interaction between Rust and custom Java classes through the Java Native Interface (JNI). This crate is aimed for Android applications that require efficient JNI calls and need to cache method references to improve performance and reliability.


## Purpose

In an Android application using rust, I encountered an issue where the Java environment couldn't find custom classes. To solve this, I created a mechanism to cache methods via the Activity object, ensuring that custom Java classes and methods are readily available for JNI calls.


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
