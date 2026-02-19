---
id: async-send-bounds
phase: 14
phase_title: "Async Rust"
sequence: 3
title: Async Tasks Must Be Send
hints:
  - tokio::spawn requires the future to be Send because it may run on any thread
  - Rc is not Send -- holding it across an .await makes the entire future not Send
  - Replace Rc with Arc when working in async spawned tasks
  - Alternatively, ensure non-Send values are dropped before any .await point
---

## Description

When you use `tokio::spawn` (or similar runtime spawn functions), the future may be executed on any thread in the runtime's thread pool. This means the future itself must implement `Send` -- it must be safe to move between threads. A future is `Send` if all values it holds across `.await` points are `Send`. If you hold a non-`Send` type (like `Rc<T>`) across an `.await` point inside a spawned task, the compiler will reject it. This is Rust extending its thread-safety guarantees into the async world.

## Broken Code

```rust
// Cargo.toml dependencies:
// tokio = { version = "1", features = ["full"] }

use std::rc::Rc;
use std::time::Duration;

async fn do_work() {
    // Rc is not Send. Holding it across an .await point
    // makes this future not Send.
    let data = Rc::new(vec![1, 2, 3]);

    println!("Data before: {:?}", data);

    // The .await point: the future suspends here.
    // Because `data` (an Rc) is still alive across this point,
    // the future's state machine holds a non-Send type.
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("Data after: {:?}", data);
}

#[tokio::main]
async fn main() {
    // tokio::spawn requires the future to be Send.
    // This fails because do_work() is not Send.
    tokio::spawn(do_work()).await.unwrap();
}
```

## Correct Code

```rust
// Cargo.toml dependencies:
// tokio = { version = "1", features = ["full"] }

use std::sync::Arc;
use std::time::Duration;

async fn do_work() {
    // Arc is Send (when T is Send), so holding it across
    // an .await point is safe.
    let data = Arc::new(vec![1, 2, 3]);

    println!("Data before: {:?}", data);

    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("Data after: {:?}", data);
}

#[tokio::main]
async fn main() {
    // Now do_work() returns a Send future.
    tokio::spawn(do_work()).await.unwrap();
}
```

## Explanation

The broken version holds an `Rc<Vec<i32>>` across an `.await` point inside an async function that is spawned with `tokio::spawn`. Here is why that fails:

**How async futures become state machines:**

The compiler transforms an async function into a state machine. Each `.await` point becomes a state transition. The state machine struct must store all local variables that are live across `.await` points. In the broken version, `data` (an `Rc`) is alive both before and after the `sleep().await`, so it becomes a field of the state machine struct.

**Why Send matters for spawned tasks:**

`tokio::spawn` schedules a future onto the tokio runtime's thread pool. The runtime may start the future on one thread, suspend it at an `.await` point, and resume it on a different thread. For this to be safe, the future must be `Send` -- every value stored in its state machine must be safe to move between threads.

**Why Rc breaks this:**

`Rc<T>` uses non-atomic reference counting. If an `Rc` were moved to a different thread while another clone existed on the original thread, both threads could modify the reference count simultaneously, causing a data race. Rust prevents this by making `Rc<T>` not `Send`.

Since the future's state machine holds an `Rc` across the `.await` point, the state machine itself is not `Send`, and `tokio::spawn` rejects it.

**Alternative fix -- drop before await:**

If you only need the `Rc` before the `.await` point, you can drop it before suspending:

```rust
async fn do_work() {
    {
        let data = Rc::new(vec![1, 2, 3]);
        println!("Data: {:?}", data);
    } // `data` is dropped here, before the .await

    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("Work done!");
}
```

This works because the `Rc` is no longer part of the state machine's state at the `.await` point. The future is `Send` because no non-`Send` values are held across suspension points.

The invariant violated in the broken code: **a future spawned with `tokio::spawn` must be `Send`; holding a non-`Send` type across an `.await` point makes the entire future non-`Send`.**

## Compiler Error Interpretation

```
error: future cannot be sent between threads safely
   --> src/main.rs:23:18
    |
23  |     tokio::spawn(do_work()).await.unwrap();
    |                  ^^^^^^^^^ future returned by `do_work` is not `Send`
    |
    = help: within `impl Future<Output = ()>`, the trait `Send` is not
            implemented for `Rc<Vec<i32>>`
note: future is not `Send` as this value is used across an await
   --> src/main.rs:17:5
    |
10  |     let data = Rc::new(vec![1, 2, 3]);
    |         ---- has type `Rc<Vec<i32>>` which is not `Send`
...
17  |     tokio::time::sleep(Duration::from_millis(100)).await;
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ await occurs here,
    |                                     with `data` maybe used afterwards
...
19  |     println!("Data after: {:?}", data);
    |                                  ---- `data` is later used here
note: required by a bound in `tokio::spawn`
```

This error message is remarkably detailed:

1. **"future returned by `do_work` is not `Send`"** -- the top-level problem. The future cannot be sent to another thread.
2. **"the trait `Send` is not implemented for `Rc<Vec<i32>>`"** -- identifies the exact type causing the problem.
3. **"future is not `Send` as this value is used across an await"** -- the compiler explains the mechanism. The `Rc` is held across an `.await`, which means it must be stored in the future's state machine, making the state machine not `Send`.
4. **"has type `Rc<Vec<i32>>` which is not `Send`"** -- points to where the non-`Send` value is created.
5. **"await occurs here, with `data` maybe used afterwards"** -- shows the `.await` point and notes that `data` is used after it, confirming it must be stored across the suspension.
6. **"required by a bound in `tokio::spawn`"** -- traces the `Send` requirement back to `tokio::spawn`.

The compiler gives you everything you need: which type is the problem, why it is a problem, where the `.await` happens, and what requires `Send`. You can then choose your fix: use `Arc` instead, or restructure to drop the `Rc` before the `.await`.
