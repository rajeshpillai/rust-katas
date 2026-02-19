---
id: message-passing
phase: 13
phase_title: "Concurrency the Rust Way"
sequence: 2
title: Message Passing with Channels
hints:
  - A Sender is moved into the first thread -- it cannot be used again
  - To have multiple producers, you must clone the Sender before moving it
  - Each clone is an independent handle to the same channel
---

## Description

Rust's `std::sync::mpsc` module provides multi-producer, single-consumer channels for message passing between threads. The name `mpsc` stands for "multiple producer, single consumer." A channel has two halves: a `Sender` (or `tx`) and a `Receiver` (or `rx`). Senders can be cloned to allow multiple threads to send messages to the same receiver. However, because Rust enforces ownership, you must think carefully about who owns each sender.

## Broken Code

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    // First thread takes ownership of tx
    thread::spawn(move || {
        tx.send("hello from thread 1").unwrap();
    });

    // Second thread tries to use the same tx -- but it was already moved!
    thread::spawn(move || {
        tx.send("hello from thread 2").unwrap();
    });

    for received in rx {
        println!("Got: {}", received);
    }
}
```

## Correct Code

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    // Clone the sender BEFORE moving it into the first thread
    let tx2 = tx.clone();

    thread::spawn(move || {
        tx.send("hello from thread 1").unwrap();
    });

    thread::spawn(move || {
        tx2.send("hello from thread 2").unwrap();
    });

    // The receiver will get both messages.
    // The for loop ends when all senders are dropped.
    for received in rx {
        println!("Got: {}", received);
    }
}
```

## Explanation

The broken version moves `tx` into the first thread's closure with `move`. After this move, `tx` no longer exists in the main scope. When the second `thread::spawn` tries to capture `tx` again, the compiler rejects it because the value has already been moved.

This is Rust's ownership system at work: a value can only have one owner at a time. Moving `tx` into the first closure transfers ownership. The second closure cannot use a value that no longer exists.

The fix is to clone the sender before the first move. `mpsc::Sender` implements `Clone`, and each clone is an independent handle to the same channel. This is precisely the "multi-producer" part of `mpsc` -- you create multiple senders by cloning.

There is an important subtlety with the `for received in rx` loop: it blocks waiting for messages and only terminates when **all** senders have been dropped. Each sender (the original and all clones) is dropped when its owning thread finishes. Once every sender is gone, the channel is closed and the loop ends.

The invariant violated in the broken code: **a moved value cannot be used again; to share a Sender across multiple threads, clone it before the move.**

## Compiler Error Interpretation

```
error[E0382]: use of moved value: `tx`
  --> src/main.rs:13:19
   |
4  |     let (tx, rx) = mpsc::channel();
   |          -- move occurs because `tx` has type `Sender<&str>`,
   |             which does not implement the `Copy` trait
...
8  |     thread::spawn(move || {
   |                   ------- value moved into closure here
9  |         tx.send("hello from thread 1").unwrap();
   |         -- variable moved due to use in closure
...
13 |     thread::spawn(move || {
   |                   ^^^^^^^ value used here after move
14 |         tx.send("hello from thread 2").unwrap();
   |         -- use occurs due to use in closure
```

The compiler traces the lifetime of `tx` step by step:

1. **"move occurs because `tx` has type `Sender<&str>`, which does not implement the `Copy` trait"** -- `Sender` is not `Copy`, so passing it to a closure with `move` is a true move, not a copy.
2. **"value moved into closure here"** -- the first `thread::spawn(move || { ... })` takes ownership of `tx`.
3. **"value used here after move"** -- the second `thread::spawn` tries to use `tx`, but it is gone.

The fix is always the same pattern: clone before you move. This applies to any non-`Copy` type you need in multiple threads.
