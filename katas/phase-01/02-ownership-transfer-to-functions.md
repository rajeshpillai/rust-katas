---
id: ownership-transfer-to-functions
phase: 1
phase_title: Ownership
sequence: 2
title: Ownership Transfer to Functions
hints:
  - Passing a value to a function is like assigning it to a new variable — it moves
  - After calling a function with a String argument, the caller no longer owns it
  - You can return the value from the function to give ownership back
---

## Description

Passing a value to a function transfers ownership, just like assignment does. Once you pass a `String` to a function, you cannot use it afterward — the function now owns it. This applies to all types that do not implement `Copy`. Understanding this is essential: function calls are ownership boundaries.

## Broken Code

```rust
fn print_length(s: String) {
    println!("'{}' has {} characters", s, s.len());
}

fn main() {
    let message = String::from("ownership");

    print_length(message);
    println!("The message was: {}", message);
}
```

## Correct Code

```rust
fn print_length(s: String) -> String {
    println!("'{}' has {} characters", s, s.len());
    s
}

fn main() {
    let message = String::from("ownership");

    let message = print_length(message);
    println!("The message was: {}", message);
}
```

## Explanation

In the broken code, `print_length(message)` moves `message` into the function. Inside `print_length`, the parameter `s` owns the String. When `print_length` returns, `s` goes out of scope and the String is dropped (its memory is freed). Back in `main`, `message` is no longer valid — it was consumed by the function call.

This is the same rule as move semantics, applied to function parameters. A function that takes `String` (not `&String` or `&str`) is declaring: "I will take ownership of this value." The caller must accept that the value is gone after the call.

The correct version fixes this by returning `s` from the function. This transfers ownership back to the caller, where it is bound to a new `message` via shadowing. The value survives because ownership was explicitly returned.

This pattern — take ownership, do something, return ownership — works but is cumbersome. In Phase 2, you will learn about borrowing, which solves this much more elegantly. For now, the key lesson is: **passing a value to a function is a transfer of ownership, and the caller loses access.**

Note: We are deliberately not introducing borrowing (`&`) yet. Understanding moves first makes borrowing intuitive later.

## Compiler Error Interpretation

```
error[E0382]: borrow of moved value: `message`
 --> main.rs:9:40
  |
6 |     let message = String::from("ownership");
  |         ------- move occurs because `message` has type `String`, which does not implement the `Copy` trait
7 |
8 |     print_length(message);
  |                  ------- value moved here
9 |     println!("The message was: {}", message);
  |                                     ^^^^^^^ value borrowed here after move
```

The same E0382 error as in the move semantics kata, but now the move happens at a function call boundary. The compiler shows that `message` was moved into `print_length` on line 8, and then used on line 9. The error message is consistent — Rust always tells you where the move happened and where the invalid use occurred.
