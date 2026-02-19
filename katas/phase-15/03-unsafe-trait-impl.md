---
id: unsafe-trait-impl
phase: 15
phase_title: "Unsafe Rust"
sequence: 3
title: Unsafe Traits — Send and Sync Contracts
hints:
  - Send and Sync are unsafe traits because implementing them incorrectly causes undefined behavior
  - A type containing a raw pointer is not automatically Send or Sync
  - Incorrectly implementing Send for a type with interior shared state can cause data races
  - Only implement Send/Sync when you can truly guarantee thread safety
---

## Description

`Send` and `Sync` are "unsafe traits" in Rust. This means implementing them requires an `unsafe impl` block, because the compiler cannot verify the guarantees you are asserting. `Send` says "this type can be safely moved to another thread." `Sync` says "this type can be safely shared (via `&T`) between threads." The compiler automatically implements these traits for most types, but types containing raw pointers opt out by default. If you manually implement `Send` or `Sync` for a type that is not actually thread-safe, you introduce data races -- undefined behavior that Rust normally prevents.

## Broken Code

```rust
use std::thread;

struct SharedCounter {
    // Using a raw pointer to shared mutable state.
    // This is NOT thread-safe.
    ptr: *mut i32,
}

// BUG: We claim SharedCounter is Send, but it contains
// a raw pointer to mutable state. If two threads hold
// SharedCounters pointing to the same i32, they can
// race on reads and writes with no synchronization.
unsafe impl Send for SharedCounter {}

impl SharedCounter {
    fn new(ptr: *mut i32) -> Self {
        SharedCounter { ptr }
    }

    fn increment(&self) {
        // No synchronization! Two threads calling this
        // on pointers to the same i32 is a data race.
        unsafe {
            *self.ptr += 1;
        }
    }

    fn get(&self) -> i32 {
        unsafe { *self.ptr }
    }
}

fn main() {
    let mut value: i32 = 0;
    let ptr = &mut value as *mut i32;

    let counter1 = SharedCounter::new(ptr);
    let counter2 = SharedCounter::new(ptr);

    // Both threads mutate the same memory without synchronization.
    // This is a DATA RACE -- undefined behavior.
    let h1 = thread::spawn(move || {
        for _ in 0..1000 {
            counter1.increment();
        }
    });

    let h2 = thread::spawn(move || {
        for _ in 0..1000 {
            counter2.increment();
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();

    // The result is unpredictable: lost updates, garbage values, or worse.
    println!("Final value: {}", unsafe { *ptr });
}
```

## Correct Code

```rust
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;

struct SharedCounter {
    value: Arc<AtomicI32>,
}

// No need for `unsafe impl Send` -- Arc<AtomicI32> is already Send.
// The compiler derives Send automatically because all fields are Send.

impl SharedCounter {
    fn new() -> Self {
        SharedCounter {
            value: Arc::new(AtomicI32::new(0)),
        }
    }

    fn increment(&self) {
        // Atomic operations are thread-safe by definition.
        self.value.fetch_add(1, Ordering::SeqCst);
    }

    fn get(&self) -> i32 {
        self.value.load(Ordering::SeqCst)
    }
}

impl Clone for SharedCounter {
    fn clone(&self) -> Self {
        SharedCounter {
            value: Arc::clone(&self.value),
        }
    }
}

fn main() {
    let counter = SharedCounter::new();

    let counter1 = counter.clone();
    let counter2 = counter.clone();

    let h1 = thread::spawn(move || {
        for _ in 0..1000 {
            counter1.increment();
        }
    });

    let h2 = thread::spawn(move || {
        for _ in 0..1000 {
            counter2.increment();
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();

    // Guaranteed correct: 2000
    println!("Final value: {}", counter.get());
}
```

## Explanation

The broken version is a textbook example of why `Send` and `Sync` are unsafe traits. The programmer asserts (via `unsafe impl Send`) that `SharedCounter` is safe to send to another thread. But the type contains a raw pointer to shared mutable state with no synchronization. Two threads increment the same `i32` via raw pointer dereference, which is a data race -- undefined behavior in Rust (and C/C++).

**What makes this unsafe impl wrong:**

The contract of `Send` says: "If I move this value to another thread, no undefined behavior will result." For `SharedCounter`, this is false. Two `SharedCounter` instances can point to the same `i32`, and both can mutate it without synchronization. The `unsafe impl Send` lies to the compiler, and the compiler trusts you. This breaks Rust's fundamental guarantee: safe code cannot cause undefined behavior.

**Why the compiler does not catch this:**

Raw pointers (`*mut T`, `*const T`) are not `Send` or `Sync` by default. This is a conservative default: the compiler does not know whether the pointer is shared, whether the memory is valid, or whether access is synchronized. By writing `unsafe impl Send`, you override this default and take full responsibility.

The compiler cannot verify the correctness of an `unsafe impl` -- that is precisely why the trait is marked `unsafe`. If you implement it wrong, the compiler cannot save you. Data races in Rust are just as undefined as in C++.

**The correct approach:**

The correct version eliminates raw pointers entirely. It uses `Arc<AtomicI32>`:
- `Arc` provides thread-safe shared ownership (atomic reference counting).
- `AtomicI32` provides thread-safe mutation (atomic CPU instructions).

No `unsafe` is needed. The compiler automatically derives `Send` for `SharedCounter` because `Arc<AtomicI32>` is `Send`. The data race is impossible: atomic operations are guaranteed to be sequentially consistent (with `Ordering::SeqCst`).

**When is `unsafe impl Send` appropriate?**

Only when you have genuinely verified thread safety through other means that the compiler cannot see. For example, a type might contain a raw pointer that is only ever accessed through a `Mutex`. The `Mutex` provides synchronization, but the compiler does not know this because it only sees the raw pointer. In such cases, `unsafe impl Send` is correct -- but it must come with a detailed safety comment explaining why.

The invariant violated in the broken code: **`unsafe impl Send` asserts thread safety to the compiler; if the assertion is false, you introduce data races that Rust is designed to prevent.**

## Compiler Error Interpretation

The broken version compiles without errors -- because the `unsafe impl Send` silences the compiler's safety checks. If you remove the `unsafe impl Send` line, you get:

```
error[E0277]: `*mut i32` cannot be sent between threads safely
   --> src/main.rs:37:14
    |
37  |     let h1 = thread::spawn(move || {
    |              ------------- ^------
    |              |             |
    |              |             `*mut i32` cannot be sent between threads safely
    |              required by a bound introduced by this call
    |
    = help: within `SharedCounter`, the trait `Send` is not implemented
            for `*mut i32`
note: required because it appears within the type `SharedCounter`
   --> src/main.rs:3:8
    |
3   |  struct SharedCounter {
    |         ^^^^^^^^^^^^^
note: required by a bound in `spawn`
```

This is the error you **should** see. The compiler correctly identifies that `*mut i32` is not `Send`, and since `SharedCounter` contains a `*mut i32`, it is not `Send` either. The `unsafe impl Send` overrides this check, telling the compiler "trust me, it is fine." When the assertion is wrong, you get undefined behavior at runtime instead of a compile error. This is why `unsafe impl` for marker traits is one of the most dangerous things you can do in Rust -- and why it requires the `unsafe` keyword.
