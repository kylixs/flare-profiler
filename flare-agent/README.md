rust-jvmti
==========

An extensible, safe native JVM agent implemented in pure Rust.

## A word of warning

This project is far from being complete or usable to say the least and contains
a healthy dose of proof-of-concept code that is guaranteed to either work or not.

## Abstract

Rust JVMTI (yeah, it deserves a better name) is intended to become a slim JVM
application performance management (APM) tool leveraging both safe access to native
JVM functionality via Rust and byte code instrumentation using Java code.  

## Already implemented (probably poorly)

* Ability to connect to a JVM as a native agent library
* Read and parse loaded class files
* Generate byte code from loaded or created class files
* Gathering and displaying statistics about method class, class loading and synchronization times
* Read basic command line configuration
* Basic JVM emulator for implementing unit tests without the need for an actual JVM

## Planned features

_Short(ish) term plans_

* Clean the initial code design up a bit
* Make data collection and JVM callbacks work in parallel so that it doesn't block Java calls
* Higher level API for accessing JVMTI/JNI functionality
* JVM byte code instrumentation/transformation
* Dynamic tracing/profiling
* Execution path tracing
* Proper logging using the slog crate instead of dumping everything to stdout

_Medium term ideas_

* Some ugly but usable GUI to display the gathered statistics (I'm so bad at designing GUIs)
* Separate store for collected data
* DSL to generate custom JVM byte code in an efficient and type-safe manner
* Java source parsing and AST support

_Long term ideas (wishes, cough)_

* Ability to trace other JVM languages (Scala, Clojure, Frege, Kotlin, etc)
* Ability to process creepy stuff like AspectJ and other BCI stuff
* Naive Java-to-native compilation (similarly to scala-native)

_(to be continued...)_

## Usage

*Just don't yet.* Although the code is supposed to be safe and correct, it's highly experimental
and immature in general. Loading this agent may corrupt your soul as well as your JVM but
at least in a type-safe way.

If you really insist on trying this library then build it with the usual `cargo build --release` and
fire up a Java VM with a command something like

```java -agentpath:./target/release/libjvmti.so MyClass```

The only supported configuration directive is `agentid` at the moment. This allows identifying
and a specific instance more easily. Every other configuration will be passed to `custom_args`.
