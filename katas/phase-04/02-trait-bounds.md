---
id: trait-bounds
phase: 4
phase_title: "Traits & Generics"
sequence: 2
title: Trait Bounds on Generic Functions
hints:
  - "When a generic function uses a value in a specific way (like formatting it), the compiler needs proof that the type supports that operation."
  - "The `{}` format placeholder requires the `std::fmt::Display` trait."
  - "Add a trait bound after the generic parameter: `T: Display`."
---

## Description

Generic functions in Rust let you write code that works with many types. But "many types" does not mean "all types unconditionally." If your function uses a capability of the type — like printing it, comparing it, or cloning it — the compiler demands that you declare that requirement as a **trait bound**.

A trait bound says: "This function works for any type `T`, **as long as** `T` implements this trait." Without the bound, the compiler cannot verify that the operations inside the function are valid for every possible `T`.

## Broken Code

```rust
fn print_labeled<T>(label: &str, value: T) {
    // The {} format requires Display, but we haven't told the compiler
    // that T implements Display
    println!("{}: {}", label, value);
}

fn main() {
    print_labeled("name", "Alice");
    print_labeled("age", 30);
    print_labeled("score", 99.5);
}
```

## Correct Code

```rust
use std::fmt::Display;

fn print_labeled<T: Display>(label: &str, value: T) {
    println!("{}: {}", label, value);
}

fn main() {
    print_labeled("name", "Alice");
    print_labeled("age", 30);
    print_labeled("score", 99.5);
}
```

## Explanation

In the broken version, the function `print_labeled` is generic over `T` — it accepts any type. But inside the function body, we use `{}` in the `println!` macro, which expands to a call to `Display::fmt`. The compiler does not know that every possible `T` implements `Display`. What if someone passed a type that has no display representation? The compiler would generate broken code.

Rust prevents this by requiring you to **declare your assumptions**. The bound `T: Display` is a contract that says: "I will only accept types that can be formatted with `{}`." Now the compiler can:

1. Verify that every call site passes a type that implements `Display`.
2. Generate the correct `Display::fmt` call inside the function.

You can also write the bound using `where` syntax for readability, especially when there are multiple bounds:

```rust
fn print_labeled<T>(label: &str, value: T)
where
    T: Display,
{
    println!("{}: {}", label, value);
}
```

Both forms are equivalent. The `where` clause is preferred when bounds get complex, such as `T: Display + Debug + Clone`.

Think of trait bounds as the type-level equivalent of function preconditions. They make the requirements explicit and machine-checkable.

## Compiler Error Interpretation

```
error[E0277]: `T` doesn't implement `std::fmt::Display`
 --> src/main.rs:3:34
  |
3 |     println!("{}: {}", label, value);
  |                                ^^^^^ `T` cannot be formatted with the default formatter
  |
  = help: the trait `std::fmt::Display` is not implemented for `T`
  = note: in format strings you may be able to use `{:?}` (or {:#?} for pretty-print) instead
help: consider restricting type parameter `T`
  |
1 | fn print_labeled<T: std::fmt::Display>(label: &str, value: T) {
  |                   +++++++++++++++++++
```

This error is rich with guidance:

- **"`T` doesn't implement `std::fmt::Display`"** — The compiler knows exactly which trait is missing and on which type parameter.
- **"`T` cannot be formatted with the default formatter"** — It connects the missing trait to the specific operation you tried to perform.
- **"consider restricting type parameter `T`"** — It suggests the exact fix: add `: std::fmt::Display` to the generic parameter.
- **"you may be able to use `{:?}`"** — It even offers an alternative: `Debug` formatting, which is a different trait. This is useful but not always what you want — `Display` is for user-facing output, `Debug` is for developer-facing output.
