---
id: shadowing-vs-mutation
phase: 0
phase_title: Rust as a Language
sequence: 2
title: Shadowing vs Mutation
hints:
  - Shadowing creates a new binding with the same name
  - Mutation changes the value of an existing binding
  - Can you change the type of a mutable variable?
---

## Description

Rust allows you to shadow a variable by declaring a new `let` binding with the same name. This is different from mutation — shadowing creates a new variable entirely, which can even have a different type.

## Broken Code

```rust
fn main() {
    let mut x = "5";
    x = x.len();
    println!("length = {}", x);
}
```

## Correct Code

```rust
fn main() {
    let x = "5";
    let x = x.len();
    println!("length = {}", x);
}
```

## Explanation

Mutation (`let mut x`) allows you to change the *value* of a variable, but not its *type*. If `x` is a `&str`, you cannot assign a `usize` to it — the types do not match.

Shadowing (`let x = ...` again) creates an entirely new variable that happens to have the same name. The old `x` is no longer accessible. Because it is a new binding, it can have a completely different type.

This distinction matters: shadowing is about *rebinding a name*, mutation is about *changing a value*. Rust keeps these concepts separate because conflating them is a source of bugs in other languages.

## Compiler Error Interpretation

```
error[E0308]: mismatched types
 --> main.rs:3:9
  |
2 |     let mut x = "5";
  |                 --- expected due to this value
3 |     x = x.len();
  |         ^^^^^^^ expected `&str`, found `usize`
```

The compiler sees that `x` was declared as `&str` (inferred from `"5"`). When you try to assign `x.len()` (which returns `usize`), the types do not match. Rust does not do implicit type conversion. The fix is to use shadowing instead of mutation.
