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
  - You need an async runtime (like tokio) to drive futures to completion
---

## Description

In Rust, `async fn` does not execute its body when called. Instead, it returns a `Future` -- a value that represents a computation that has not happened yet. Futures in Rust are lazy: they do nothing until something polls them. The `.await` keyword is what triggers polling. If you call an async function without `.await`, the function body never runs. The compiler will warn you about this, but it is not a hard error -- making it a subtle source of bugs.

## Broken Code

```rust
// Cargo.toml dependencies:
// tokio = { version = "1", features = ["full"] }

use std::time::Duration;

async fn do_work() {
    println!("Work started!");
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("Work completed!");
}

#[tokio::main]
async fn main() {
    println!("Before work");

    // BUG: Calling an async function without .await
    // This creates a Future but never executes it
    do_work();

    println!("After work");
}
```

## Correct Code

```rust
// Cargo.toml dependencies:
// tokio = { version = "1", features = ["full"] }

use std::time::Duration;

async fn do_work() {
    println!("Work started!");
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("Work completed!");
}

#[tokio::main]
async fn main() {
    println!("Before work");

    // Correct: .await drives the future to completion
    do_work().await;

    println!("After work");
}
```

## Explanation

The broken version calls `do_work()` without `.await`. In Rust, this creates a `Future` object but never polls it. The function body ("Work started!", the sleep, "Work completed!") never executes. The program prints "Before work" and "After work" with nothing in between.

This is fundamentally different from async in JavaScript or Python, where calling an async function immediately begins execution up to the first await point. In Rust, futures are completely inert until polled.

**How async works under the hood:**

1. An `async fn` is syntactic sugar. The compiler transforms the function body into a state machine that implements the `Future` trait.
2. Calling `do_work()` instantiates this state machine but does not advance it.
3. `.await` tells the runtime to poll the future repeatedly until it completes.
4. Each poll advances the state machine to the next suspension point (the next `.await` inside the async function).

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

The invariant violated in the broken code: **futures are lazy; calling an async function only constructs the future -- you must `.await` it to execute it.**

## Compiler Error Interpretation

```
warning: unused implementer of `Future` that must be used
  --> src/main.rs:17:5
   |
17 |     do_work();
   |     ^^^^^^^^^
   |
   = note: futures do nothing unless you `.await` or poll them
   = note: `#[warn(unused_must_use)]` on by default
```

This is a warning, not an error. The program will compile and run -- but the async work will silently never happen. This makes it a particularly insidious bug.

The compiler tells you:

1. **"unused implementer of `Future` that must be used"** -- you created a `Future` but did nothing with it. The `#[must_use]` attribute on `Future` triggers this warning.
2. **"futures do nothing unless you `.await` or poll them"** -- the compiler explains the lazy evaluation model directly.

In production code, you should treat this warning as an error (use `#[deny(unused_must_use)]` or `#![deny(warnings)]`). A forgotten `.await` can cause entire operations to silently not happen -- database writes, network requests, file operations -- with no runtime error to alert you.
