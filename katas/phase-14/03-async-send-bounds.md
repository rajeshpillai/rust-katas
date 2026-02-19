---
id: async-send-bounds
phase: 14
phase_title: "Async Rust"
sequence: 3
title: Async Tasks Must Be Send
hints:
  - Spawning a future on a thread requires the future to be Send
  - Rc is not Send -- holding it across an .await makes the entire future not Send
  - Replace Rc with Arc when the future must be Send
  - Alternatively, ensure non-Send values are dropped before any .await point
---

## Description

When you spawn a future onto another thread (via `std::thread::spawn` or a runtime like `tokio::spawn`), the future must implement `Send` -- it must be safe to move between threads. A future is `Send` if all values it holds across `.await` points are `Send`. If you hold a non-`Send` type (like `Rc<T>`) across an `.await` point inside a future that gets sent to another thread, the compiler will reject it. This is Rust extending its thread-safety guarantees into the async world.

## Broken Code

```rust
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
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

/// Spawn a future on a new thread and wait for it to complete.
/// Requires the future to be Send (safe to move between threads).
fn spawn_blocking<F>(future: F) -> F::Output
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let handle = std::thread::spawn(move || block_on(future));
    handle.join().expect("spawned thread panicked")
}

async fn do_work() {
    // Rc is not Send. Holding it across an .await point
    // makes this future not Send.
    let data = Rc::new(vec![1, 2, 3]);

    println!("Data before: {:?}", data);

    // The .await point: the future suspends here.
    // Because `data` (an Rc) is still alive across this point,
    // the future's state machine holds a non-Send type.
    std::future::ready(()).await;

    println!("Data after: {:?}", data);
}

fn main() {
    // BUG: spawn_blocking requires the future to be Send.
    // do_work() is not Send because it holds Rc across an .await.
    spawn_blocking(do_work());
}
```

## Correct Code

```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
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

/// Spawn a future on a new thread and wait for it to complete.
/// Requires the future to be Send (safe to move between threads).
fn spawn_blocking<F>(future: F) -> F::Output
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let handle = std::thread::spawn(move || block_on(future));
    handle.join().expect("spawned thread panicked")
}

async fn do_work() {
    // Arc is Send (when T is Send + Sync), so holding it across
    // an .await point is safe.
    let data = Arc::new(vec![1, 2, 3]);

    println!("Data before: {:?}", data);

    std::future::ready(()).await;

    println!("Data after: {:?}", data);
}

fn main() {
    // Now do_work() returns a Send future, so spawn_blocking accepts it.
    spawn_blocking(do_work());
}
```

## Explanation

The broken version holds an `Rc<Vec<i32>>` across an `.await` point inside an async function whose future is sent to another thread via `spawn_blocking`. Here is why that fails:

**How async futures become state machines:**

The compiler transforms an async function into a state machine. Each `.await` point becomes a state transition. The state machine struct must store all local variables that are live across `.await` points. In the broken version, `data` (an `Rc`) is alive both before and after the `.await`, so it becomes a field of the state machine struct.

**Why Send matters for spawned tasks:**

Our `spawn_blocking` function (and runtime functions like `tokio::spawn`) move the future to another thread. For this to be safe, the future must be `Send` -- every value stored in its state machine must be safe to move between threads.

**Why Rc breaks this:**

`Rc<T>` uses non-atomic reference counting. If an `Rc` were moved to a different thread while another clone existed on the original thread, both threads could modify the reference count simultaneously, causing a data race. Rust prevents this by making `Rc<T>` not `Send`.

Since the future's state machine holds an `Rc` across the `.await` point, the state machine itself is not `Send`, and `spawn_blocking` rejects it.

**Alternative fix -- drop before await:**

If you only need the `Rc` before the `.await` point, you can drop it before suspending:

```rust
async fn do_work() {
    {
        let data = Rc::new(vec![1, 2, 3]);
        println!("Data: {:?}", data);
    } // `data` is dropped here, before the .await

    std::future::ready(()).await;

    println!("Work done!");
}
```

This works because the `Rc` is no longer part of the state machine's state at the `.await` point. The future is `Send` because no non-`Send` values are held across suspension points.

The invariant violated in the broken code: **a future sent to another thread must be `Send`; holding a non-`Send` type across an `.await` point makes the entire future non-`Send`.**

## Compiler Error Interpretation

```
error[E0277]: `Rc<Vec<i32>>` cannot be sent between threads safely
   --> main.rs:55:20
    |
55  |     spawn_blocking(do_work());
    |     -------------- ^^^^^^^^^ `Rc<Vec<i32>>` cannot be sent between threads safely
    |     |
    |     required by a bound introduced by this call
    |
    = help: within `impl Future<Output = ()>`, the trait `Send` is not
            implemented for `Rc<Vec<i32>>`
note: future is not `Send` as this value is used across an await
   --> main.rs:47:5
    |
43  |     let data = Rc::new(vec![1, 2, 3]);
    |         ---- has type `Rc<Vec<i32>>` which is not `Send`
...
47  |     std::future::ready(()).await;
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^ await occurs here,
    |                               with `data` maybe used afterwards
...
49  |     println!("Data after: {:?}", data);
    |                                  ---- `data` is later used here
```

This error message is remarkably detailed:

1. **"`Rc<Vec<i32>>` cannot be sent between threads safely"** -- identifies the exact type causing the problem.
2. **"the trait `Send` is not implemented for `Rc<Vec<i32>>`"** -- explains why: `Rc` is not thread-safe.
3. **"future is not `Send` as this value is used across an await"** -- the compiler explains the mechanism. The `Rc` is held across an `.await`, which means it must be stored in the future's state machine, making the state machine not `Send`.
4. **"has type `Rc<Vec<i32>>` which is not `Send`"** -- points to where the non-`Send` value is created.
5. **"await occurs here, with `data` maybe used afterwards"** -- shows the `.await` point and notes that `data` is used after it, confirming it must be stored across the suspension.

The compiler gives you everything you need: which type is the problem, why it is a problem, where the `.await` happens, and what requires `Send`. You can then choose your fix: use `Arc` instead, or restructure to drop the `Rc` before the `.await`.
