---
id: async-ownership
phase: 14
phase_title: "Async Rust"
sequence: 2
title: Ownership and References Across Await Points
hints:
  - An .await point is where the future suspends and may be resumed later
  - Any data referenced across an .await must live long enough
  - The simplest fix is to own the data instead of borrowing it
  - If you must borrow, ensure the borrow does not span an .await point
---

## Description

When an async function hits an `.await` point, it suspends execution. The runtime may resume it later -- possibly on a different thread. Any references held across that suspension must remain valid. This creates strict constraints on borrowing inside async functions. A reference to a local variable in a calling function cannot safely be held across an `.await`, because the caller might drop the data while the async function is suspended. The solution is usually to own the data or restructure the code so that borrows do not span `.await` points.

## Broken Code

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

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

async fn process(data: &str) {
    println!("Processing: {}", data);
    // Simulate async work with an await point
    std::future::ready(()).await;
    println!("Done processing: {}", data);
}

async fn run() {
    let result = {
        let temp = String::from("temporary data");
        // BUG: We create a future that borrows `temp`,
        // but `temp` is dropped at the end of this block.
        process(&temp)
    };
    // `temp` is dropped here, but the future still holds a reference to it.

    // When we await, the future tries to use the dangling reference.
    result.await;
}

fn main() {
    block_on(run());
}
```

## Correct Code

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

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

async fn process(data: String) {
    println!("Processing: {}", data);
    std::future::ready(()).await;
    println!("Done processing: {}", data);
}

async fn run() {
    let temp = String::from("temporary data");
    // Correct: transfer ownership of the String into the async function.
    // The future now owns the data it needs.
    process(temp).await;
}

fn main() {
    block_on(run());
}
```

## Explanation

The broken version creates a future by calling `process(&temp)` inside a block. The future captures a reference to `temp`. But `temp` is dropped at the end of the block, before the future is ever awaited. When `result.await` tries to resume the future, it would access freed memory.

Rust's borrow checker catches this at compile time. It sees that the future returned by `process(&temp)` borrows `temp`, and that `temp` does not live long enough for the future to be awaited.

**Why async makes borrowing harder:**

In synchronous code, a function call `process(&temp)` borrows `temp` for the duration of the call. The borrow ends when the function returns. But in async code, `process(&temp)` does not execute immediately -- it returns a future. The borrow must last until the future is awaited and completes. This extends the borrow's lifetime far beyond what you might expect.

Think of it this way: an async function's body is a state machine. Any references captured at call time are stored in the state machine struct. If those references outlive the data they point to, you get a dangling reference. Rust prevents this.

**Two strategies to fix this:**

1. **Own the data** (used in the correct version): Change the parameter from `&str` to `String`. The future owns the data and cannot have a dangling reference.
2. **Keep the borrow alive**: Ensure the referenced data lives at least as long as the future. For example:

```rust
async fn run() {
    let temp = String::from("temporary data");
    process(&temp).await; // temp lives until end of run()
}
```

This works because `temp` is not dropped until after the `.await` completes.

The invariant violated in the broken code: **a future that borrows data must not outlive that data; either own the data or ensure the borrow does not span the future's lifetime.**

## Compiler Error Interpretation

```
error[E0597]: `temp` does not live long enough
  --> main.rs:36:18
   |
34 |     let result = {
   |         ------ borrow later used here
35 |         let temp = String::from("temporary data");
   |             ---- binding `temp` declared here
36 |         process(&temp)
   |                 ^^^^^ borrowed value does not live long enough
37 |     };
   |     - `temp` dropped here while still borrowed
```

The compiler identifies:

1. **"`temp` does not live long enough"** -- the core problem. `temp` is destroyed at the end of the block, but something still needs it.
2. **"borrow later used here"** -- the future stored in `result` holds a borrow of `temp`, and it will be used when `result.await` is called.
3. **"`temp` dropped here while still borrowed"** -- the exact point where `temp` is destroyed, while the future still holds a reference to it.

The compiler traces the lifetime of the borrow from creation (the `&temp` argument) through the future (stored in `result`) to the point where `temp` is dropped. It proves the borrow would be dangling, and rejects the code. This is the borrow checker protecting you from use-after-free in concurrent code.
