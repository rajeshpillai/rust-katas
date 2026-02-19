---
id: move-semantics
phase: 1
phase_title: Ownership
sequence: 1
title: Move Semantics
hints:
  - When you assign a String to a new variable, the original is invalidated
  - Types that live on the heap (like String) are moved, not copied
  - Simple types like i32, f64, bool are copied automatically
---

## Description

In Rust, every value has exactly one owner. When you assign a heap-allocated value like `String` to a new variable, ownership **moves** — the original variable is invalidated, and you cannot use it anymore. This is fundamentally different from languages where assignment copies a reference or pointer. Rust's move semantics prevent double-free bugs and data races at compile time.

## Broken Code

```rust
fn main() {
    let greeting = String::from("hello");
    let greeting_copy = greeting;

    println!("Original: {}", greeting);
    println!("Copy: {}", greeting_copy);
}
```

## Correct Code

```rust
fn main() {
    let greeting = String::from("hello");
    let greeting_copy = greeting.clone();

    println!("Original: {}", greeting);
    println!("Copy: {}", greeting_copy);
}
```

## Explanation

When you write `let greeting_copy = greeting;`, you might expect both variables to point to the same string data, as they would in Python or JavaScript. But Rust does not work that way. The assignment **moves** ownership of the `String` from `greeting` to `greeting_copy`. After the move, `greeting` is no longer valid — it has been consumed.

Why does Rust do this? A `String` consists of three parts: a pointer to heap memory, a length, and a capacity. If both `greeting` and `greeting_copy` held the same pointer, then when they go out of scope, Rust would try to free the same memory twice (a double-free bug). Instead of allowing this, Rust makes the move explicit: after the move, only one variable owns the data.

Compare this with `i32`: if you write `let x = 5; let y = x;`, both `x` and `y` remain valid because `i32` implements the `Copy` trait. Integers are small, stack-allocated values where copying is cheap. `String` does not implement `Copy` because copying heap data is expensive — you must use `.clone()` to opt into that cost explicitly.

This is the first and most important ownership rule: **each value has exactly one owner, and when ownership moves, the old owner is gone.**

## Compiler Error Interpretation

```
error[E0382]: borrow of moved value: `greeting`
 --> main.rs:5:34
  |
2 |     let greeting = String::from("hello");
  |         -------- move occurs because `greeting` has type `String`, which does not implement the `Copy` trait
3 |     let greeting_copy = greeting;
  |                         -------- value moved here
4 |
5 |     println!("Original: {}", greeting);
  |                               ^^^^^^^^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
3 |     let greeting_copy = greeting.clone();
  |                                 ++++++++
```

Error E0382 is one of the most common Rust errors. The compiler traces the full story: `greeting` was moved on line 3, and then you tried to use it on line 5. It even explains *why* the move happened — `String` does not implement `Copy`. The suggested fix (`.clone()`) is correct but comes with a note about performance cost. Rust wants you to understand that cloning heap data is not free.
