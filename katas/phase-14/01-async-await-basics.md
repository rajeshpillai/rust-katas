---
id: async-await-basics
phase: 14
phase_title: "Async Rust"
sequence: 1
title: Async Await Basics — Futures Are Lazy
hints:
  - An async function does not execute when called -- it returns a Future
  - A Future does nothing until it is polled, which happens when you .await it
  - Forgetting to .await is a logic bug, not a compile error (though the compiler warns)
  - You need an executor (like block_on) to drive futures to completion
---

## Description

In Rust, `async fn` does not execute its body when called. Instead, it returns a `Future` -- a value that represents a computation that has not happened yet. Futures in Rust are lazy: they do nothing until something polls them. The `.await` keyword is what triggers polling. If you call an async function without `.await`, the function body never runs. The compiler will warn you about this, but it is not a hard error -- making it a subtle source of bugs.

This kata also introduces a minimal executor (`block_on`) that polls a future to completion. In production, you would use a runtime like `tokio` or `async-std`, but understanding how an executor works is essential to understanding async Rust.

## Broken Code

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// A minimal single-threaded executor that polls a future to completion.
fn block_on<F: Future>(future: F) -> F::Output {
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    let mut future = Box::pin(future);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(val) => return val,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

fn noop_waker() -> Waker {
    fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    fn no_op(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no_op, no_op, no_op);
    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
}

async fn do_work() {
    println!("Work started!");
    println!("Work completed!");
}

fn main() {
    println!("Before work");

    // BUG: Calling an async function without .await or an executor.
    // This creates a Future but never executes it.
    do_work();

    println!("After work");
}
```

## Correct Code

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// A minimal single-threaded executor that polls a future to completion.
fn block_on<F: Future>(future: F) -> F::Output {
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    let mut future = Box::pin(future);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(val) => return val,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

fn noop_waker() -> Waker {
    fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    fn no_op(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no_op, no_op, no_op);
    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
}

async fn do_work() {
    println!("Work started!");
    println!("Work completed!");
}

fn main() {
    println!("Before work");

    // Correct: use block_on to drive the future to completion.
    block_on(do_work());

    println!("After work");
}
```

## Explanation

The broken version calls `do_work()` without using an executor or `.await`. In Rust, this creates a `Future` object but never polls it. The function body ("Work started!", "Work completed!") never executes. The program prints "Before work" and "After work" with nothing in between.

This is fundamentally different from async in JavaScript or Python, where calling an async function immediately begins execution up to the first await point. In Rust, futures are completely inert until polled.

**How async works under the hood:**

1. An `async fn` is syntactic sugar. The compiler transforms the function body into a state machine that implements the `Future` trait.
2. Calling `do_work()` instantiates this state machine but does not advance it.
3. An executor (like our `block_on`) calls `poll()` on the future to advance it.
4. Each `poll()` advances the state machine to the next suspension point (the next `.await` inside the async function), or to completion.

**Understanding the executor:**

Our `block_on` function is a minimal executor. It:
1. Creates a no-op `Waker` (since our simple futures never return `Pending`).
2. Pins the future on the heap with `Box::pin` (futures must be pinned to be polled).
3. Polls in a loop until the future returns `Poll::Ready`.

In production, runtimes like `tokio` do much more: they manage thread pools, handle I/O events, and efficiently wake sleeping futures. But at their core, all executors do the same thing -- they poll futures.

**Why Rust chose lazy futures:**

Lazy futures give you full control over when and whether a computation happens. You can create a future, pass it around, store it in a data structure, or decide not to run it at all. This is a zero-cost abstraction: you only pay for the computation you actually perform.

The output of the broken version:
```
Before work
After work
```

The output of the correct version:
```
Before work
Work started!
Work completed!
After work
```

The invariant violated in the broken code: **futures are lazy; calling an async function only constructs the future -- you must poll it (via an executor or `.await`) to execute it.**

## Compiler Error Interpretation

```
warning: unused implementer of `Future` that must be used
  --> main.rs:35:5
   |
35 |     do_work();
   |     ^^^^^^^^^
   |
   = note: futures do nothing unless you `.await` or poll them
   = note: `#[warn(unused_must_use)]` on by default
```

This is a warning, not an error. The program will compile and run -- but the async work will silently never happen. This makes it a particularly insidious bug.

The compiler tells you:

1. **"unused implementer of `Future` that must be used"** -- you created a `Future` but did nothing with it. The `#[must_use]` attribute on `Future` triggers this warning.
2. **"futures do nothing unless you `.await` or poll them"** -- the compiler explains the lazy evaluation model directly.

In production code, you should treat this warning as an error (use `#[deny(unused_must_use)]` or `#![deny(warnings)]`). A forgotten `.await` or missing executor call can cause entire operations to silently not happen -- database writes, network requests, file operations -- with no runtime error to alert you.
