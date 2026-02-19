---
id: shared-state-mutex
phase: 13
phase_title: "Concurrency the Rust Way"
sequence: 3
title: Shared State with Arc and Mutex
hints:
  - Rc is a single-threaded reference counter -- it does not implement Send
  - Arc is the atomic (thread-safe) version of Rc
  - Mutex provides interior mutability with locking -- but it needs to be shared safely first
  - The pattern is always Arc<Mutex<T>> for shared mutable state across threads
---

## Description

When multiple threads need to read and write the same data, Rust requires two layers of protection: `Mutex<T>` for mutual exclusion (only one thread can access the data at a time), and `Arc<T>` for thread-safe shared ownership (multiple threads can hold a handle to the same mutex). Using `Rc<T>` instead of `Arc<T>` will fail because `Rc` is not safe to share across threads -- its reference count is not atomic.

## Broken Code

```rust
use std::rc::Rc;
use std::sync::Mutex;
use std::thread;

fn main() {
    let counter = Rc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..5 {
        let counter = Rc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", *counter.lock().unwrap());
}
```

## Correct Code

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..5 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", *counter.lock().unwrap());
}
```

## Explanation

The only difference between the broken and correct versions is `Rc` versus `Arc`. This single-character change (`Rc` to `Arc`) is the difference between a program that compiles and one that does not -- and it reflects a deep design choice in Rust.

**Why Rc fails:** `Rc<T>` (Reference Counted) uses a non-atomic counter to track how many owners exist. Non-atomic operations are fast but not thread-safe: if two threads increment the reference count simultaneously, the count could become corrupted. Rust prevents this at compile time by not implementing the `Send` trait for `Rc<T>`. If a type is not `Send`, it cannot be moved into another thread.

**Why Arc works:** `Arc<T>` (Atomically Reference Counted) uses atomic operations for its reference count. Atomic operations are slightly slower but are guaranteed to be correct even when multiple threads access them simultaneously. `Arc<T>` implements `Send` (when `T` is `Send`), so it can cross thread boundaries.

**The Mutex layer:** `Mutex<T>` ensures that only one thread can access the inner value at a time. When you call `.lock()`, the current thread blocks until it acquires the lock. The returned `MutexGuard` automatically releases the lock when it goes out of scope (RAII pattern). If a thread panics while holding the lock, the mutex becomes "poisoned," and subsequent `.lock()` calls return an `Err` -- the `.unwrap()` will then panic, propagating the failure.

**The full pattern:** `Arc<Mutex<T>>` gives you:
- `Arc` -- multiple owners across threads (shared ownership)
- `Mutex` -- exclusive access to the inner data (mutual exclusion)

This pattern is Rust's equivalent of a "shared mutable variable" in other languages, but with compile-time guarantees that you cannot forget the locking or sharing mechanisms.

The invariant violated in the broken code: **`Rc<T>` is not `Send` because its reference count is not atomic; use `Arc<T>` for thread-safe shared ownership.**

## Compiler Error Interpretation

```
error[E0277]: `Rc<Mutex<i32>>` cannot be sent between threads safely
   --> src/main.rs:11:36
    |
11  |         let handle = thread::spawn(move || {
    |                      ------------- ^------
    |                      |             |
    |                      |             `Rc<Mutex<i32>>` cannot be sent between threads safely
    |                      required by a bound introduced by this call
    |
    = help: the trait `Send` is not implemented for `Rc<Mutex<i32>>`
    = note: required because it appears within the type `[closure@src/main.rs:11:36: 11:43]`
note: required by a bound in `spawn`
```

The compiler error reveals the trait system at work:

1. **"`Rc<Mutex<i32>>` cannot be sent between threads safely"** -- the type `Rc<Mutex<i32>>` does not implement `Send`. This is a deliberate design decision, not an oversight.
2. **"the trait `Send` is not implemented for `Rc<Mutex<i32>>`"** -- `Send` is a marker trait that types must implement to be moved across thread boundaries. `Rc` deliberately does not implement it.
3. **"required by a bound in `spawn`"** -- `thread::spawn` requires its closure (and everything the closure captures) to be `Send`. This is how Rust enforces thread safety at compile time.

The fix is mechanical: replace `Rc` with `Arc`. But understanding *why* is what matters -- `Rc` and `Arc` have identical APIs but different safety guarantees, and the type system enforces the distinction.
