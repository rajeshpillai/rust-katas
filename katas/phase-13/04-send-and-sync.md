---
id: send-and-sync
phase: 13
phase_title: "Concurrency the Rust Way"
sequence: 4
title: Send and Sync — Why Data Races Do Not Compile
hints:
  - Rc uses non-atomic reference counting, making it unsafe to share across threads
  - Send means a type can be transferred to another thread
  - Sync means a type can be referenced from multiple threads simultaneously
  - Arc is the thread-safe replacement for Rc
---

## Description

Rust's concurrency safety rests on two marker traits: `Send` and `Sync`. A type is `Send` if it can be safely transferred (moved) to another thread. A type is `Sync` if it can be safely shared (referenced) from multiple threads simultaneously. These traits are automatically derived by the compiler for most types -- but certain types deliberately opt out. `Rc<T>` is the canonical example: it is neither `Send` nor `Sync`, because its internal reference count uses non-atomic operations. This kata demonstrates what happens when you try to send a non-`Send` type to another thread.

## Broken Code

```rust
use std::rc::Rc;
use std::thread;

fn main() {
    let data = Rc::new(vec![1, 2, 3, 4, 5]);

    let handle = thread::spawn(move || {
        let sum: i32 = data.iter().sum();
        println!("Sum: {}", sum);
    });

    handle.join().unwrap();
}
```

## Correct Code

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);

    let handle = thread::spawn(move || {
        let sum: i32 = data.iter().sum();
        println!("Sum: {}", sum);
    });

    handle.join().unwrap();
}
```

## Explanation

The broken code attempts to move an `Rc<Vec<i32>>` into a spawned thread. Even though only one thread will use the data (we are not sharing it), the compiler still rejects this because `Rc<T>` does not implement `Send`.

**Why Rc is not Send:**

`Rc<T>` maintains a reference count using ordinary (non-atomic) integers. If an `Rc` were moved to another thread, and the original thread still held a clone, both threads could try to increment or decrement the reference count simultaneously. This is a data race on the reference count itself -- not on the inner data, but on the bookkeeping that controls when the data is freed. A corrupted reference count could lead to use-after-free or double-free bugs.

Rust prevents this class of bug entirely by making `Rc<T>` not `Send`. The compiler enforces this at every thread boundary.

**The Send and Sync traits:**

- `Send`: A type `T` is `Send` if values of type `T` can be safely moved to another thread. Most types are `Send`. Notable exceptions: `Rc<T>`, raw pointers, and types containing them.
- `Sync`: A type `T` is `Sync` if `&T` (a shared reference) can be safely sent to another thread. In other words, `T` is `Sync` if multiple threads can read from it simultaneously. A type is `Sync` if and only if `&T` is `Send`.

These traits are automatically implemented by the compiler. You rarely need to implement them manually -- and doing so requires `unsafe`, because you are asserting a guarantee the compiler cannot verify.

**Why this matters:**

In languages like C++, Java, or Go, you can freely pass any object to any thread. Thread safety bugs are discovered at runtime (or not at all). In Rust, the type system prevents entire categories of concurrency bugs at compile time. You cannot accidentally share non-thread-safe data across threads. The compiler is your concurrency auditor.

The invariant violated in the broken code: **`Rc<T>` is not `Send` because its reference count is not atomic; use `Arc<T>` for data that must cross thread boundaries.**

## Compiler Error Interpretation

```
error[E0277]: `Rc<Vec<i32>>` cannot be sent between threads safely
   --> src/main.rs:7:36
    |
7   |     let handle = thread::spawn(move || {
    |                  ------------- ^------
    |                  |             |
    |                  |             `Rc<Vec<i32>>` cannot be sent between threads safely
    |                  required by a bound introduced by this call
    |
    = help: the trait `Send` is not implemented for `Rc<Vec<i32>>`
    = note: required because it appears within the type `[closure@src/main.rs:7:36: 7:43]`
note: required by a bound in `spawn`
   --> /rustc/.../library/std/src/thread/mod.rs
    |
    = note: required by this bound in `spawn`
```

Reading this error:

1. **"`Rc<Vec<i32>>` cannot be sent between threads safely"** -- the compiler identifies the exact type that violates the thread safety requirement.
2. **"the trait `Send` is not implemented for `Rc<Vec<i32>>`"** -- this is the core issue. `Send` is a marker trait, and `Rc<T>` deliberately does not have it.
3. **"required by a bound in `spawn`"** -- the signature of `thread::spawn` requires `F: Send + 'static`. The closure must be `Send`, which means everything it captures must also be `Send`. Since the closure captures an `Rc`, and `Rc` is not `Send`, the closure itself is not `Send`.

The compiler traces the `Send` requirement from `thread::spawn` through the closure to the captured `Rc`, and shows you exactly why it fails. The fix is to replace `Rc` with `Arc`, which provides the same API with atomic reference counting.
