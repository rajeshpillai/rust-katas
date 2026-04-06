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

## ⚠️ Caution

- **`mut` on bindings vs `mut` on references are different things.** `let mut x` means you can reassign `x`. `&mut x` means you have a mutable reference. Do not confuse the two — they operate at different levels.
- **Immutable does not mean constant.** `let x = 5` is immutable but not a compile-time constant. Use `const` for values that must be known at compile time and `static` for global lifetimes.

## 💡 Tips

- Start every variable as immutable. Only add `mut` when the compiler tells you it is needed. This keeps your code easier to reason about.
- If you find yourself adding `mut` to many variables, consider whether you can restructure with shadowing (`let x = ...` again) or functional patterns instead.
- Use `rustc --explain E0384` in your terminal to read the full explanation of any error code.

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

---

|  | [Next: Shadowing vs Mutation](#/katas/shadowing-vs-mutation) |
