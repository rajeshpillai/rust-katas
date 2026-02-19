---
id: thread-ownership
phase: 13
phase_title: "Concurrency the Rust Way"
sequence: 1
title: Thread Ownership Transfer
hints:
  - A spawned thread might outlive the scope that created it
  - The compiler cannot guarantee the reference will still be valid when the thread runs
  - The `move` keyword transfers ownership of captured variables into the closure
---

## Description

When you spawn a new thread in Rust, the closure you pass to `std::thread::spawn` must own all the data it uses. This is because the new thread might outlive the scope that created it -- Rust has no way to guarantee the reference will remain valid. This is fundamentally different from languages like Java or Go, where the garbage collector keeps referenced data alive. In Rust, ownership must be explicit.

## Broken Code

```rust
use std::thread;

fn main() {
    let message = String::from("hello from the main thread");

    let handle = thread::spawn(|| {
        println!("Thread says: {}", message);
    });

    handle.join().unwrap();
}
```

## Correct Code

```rust
use std::thread;

fn main() {
    let message = String::from("hello from the main thread");

    let handle = thread::spawn(move || {
        println!("Thread says: {}", message);
    });

    // Note: `message` has been moved into the thread.
    // We can no longer use it here.
    // println!("{}", message); // This would fail to compile.

    handle.join().unwrap();
}
```

## Explanation

The broken version captures `message` by reference inside the closure passed to `thread::spawn`. The problem is that `thread::spawn` requires a `'static` lifetime for the closure -- the closure and everything it captures must be able to live for the entire duration of the program. This is because Rust cannot guarantee when the spawned thread will actually run or finish. The main thread could drop `message` before the spawned thread reads it.

The `move` keyword solves this by transferring ownership of `message` into the closure. Once moved, the thread owns the `String` and is responsible for dropping it. The original scope can no longer use `message`.

This is the ownership system applied to concurrency: instead of relying on a garbage collector to keep data alive, Rust forces you to decide who owns the data. A thread that owns its data can never have a dangling reference.

The invariant violated in the broken code: **a spawned thread's closure must own its captured data because the thread's lifetime is independent of the spawning scope.**

## Compiler Error Interpretation

```
error[E0373]: closure may outlive the current function, but it borrows `message`,
              which is owned by the current function
 --> src/main.rs:6:32
  |
6 |     let handle = thread::spawn(|| {
  |                                ^^ may outlive borrowed value `message`
7 |         println!("Thread says: {}", message);
  |                                     ------- `message` is borrowed here
  |
note: function requires argument type to outlive `'static`
 --> src/main.rs:6:18
  |
6 |     let handle = thread::spawn(|| {
  |                  ^^^^^^^^^^^^^
help: to force the closure to take ownership of `message` (and any other
      referenced variables), use the `move` keyword
  |
6 |     let handle = thread::spawn(move || {
  |                                ++++
```

The compiler tells you three things:

1. **"closure may outlive the current function"** -- the thread could still be running after `main` (or the enclosing function) returns.
2. **"`message` is borrowed here"** -- the closure captures `message` by reference, not by value.
3. **"use the `move` keyword"** -- the compiler directly suggests the fix. The `move` keyword forces the closure to take ownership of all captured variables.

This is one of the most common and well-diagnosed errors in Rust. The compiler not only explains the problem but tells you exactly how to fix it.
