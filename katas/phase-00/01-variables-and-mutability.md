---
id: variables-and-mutability
phase: 0
phase_title: Rust as a Language
sequence: 1
title: Variables and Mutability
hints:
  - Look at the variable declaration — is it mutable?
  - Rust defaults to immutability. What keyword enables mutation?
---

## Description

Rust variables are immutable by default. You must opt into mutability with `mut`. This is not a limitation — it is a deliberate design choice that prevents an entire class of bugs.

## Broken Code

```rust
fn main() {
    let x = 5;
    x = 10;
    println!("x = {}", x);
}
```

## Correct Code

```rust
fn main() {
    let mut x = 5;
    x = 10;
    println!("x = {}", x);
}
```

## Explanation

In Rust, variables are immutable by default. The binding `let x = 5` creates a value that cannot be reassigned. This is not a limitation — it is a deliberate design choice. Immutability by default means the compiler can guarantee that values do not change unexpectedly, which prevents an entire class of bugs.

To allow reassignment, you must explicitly declare the variable as mutable with `let mut x = 5`. This forces you to acknowledge that this value will change, making your intent clear to both the compiler and future readers.

## Compiler Error Interpretation

```
error[E0384]: cannot assign twice to immutable variable `x`
 --> main.rs:3:5
  |
2 |     let x = 5;
  |         - first assignment to `x`
3 |     x = 10;
  |     ^^^^^^ cannot assign twice to immutable variable
```

The compiler tells you exactly what is wrong: you declared `x` without `mut`, so it is immutable. The error code E0384 is a stable identifier — you can look it up with `rustc --explain E0384` for a detailed explanation.
