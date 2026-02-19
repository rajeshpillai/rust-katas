---
id: question-mark-operator
phase: 8
phase_title: "Error Handling as Design"
sequence: 2
title: The ? Operator for Error Propagation
hints:
  - "The `?` operator can only be used in functions that return `Result` or `Option`."
  - "If your function returns `()` (unit), the `?` has nowhere to propagate the error to."
  - "Change the return type to `Result<(), E>` so `?` can propagate errors to the caller."
---

## Description

The `?` operator is Rust's ergonomic tool for error propagation. When you write `expression?`, it does the following:

1. Evaluate the expression (which must return `Result<T, E>` or `Option<T>`).
2. If it is `Ok(value)` / `Some(value)`, unwrap and continue.
3. If it is `Err(error)` / `None`, **return early** from the current function with the error.

The key requirement: the `?` operator performs a **return**, so the enclosing function must have a return type compatible with the error being propagated.

## Broken Code

```rust
use std::fs;

fn print_file_contents(path: &str) {
    // This function returns () — it has no return type.
    // The ? operator needs to return an error, but there is
    // nowhere for the error to go.
    let content = fs::read_to_string(path)?;
    println!("File contents:\n{}", content);
}

fn main() {
    print_file_contents("example.txt");
}
```

## Correct Code

```rust
use std::fs;
use std::io;

fn print_file_contents(path: &str) -> Result<(), io::Error> {
    // Now ? can propagate the io::Error to the caller.
    let content = fs::read_to_string(path)?;
    println!("File contents:\n{}", content);
    Ok(()) // Explicit success return
}

fn main() {
    // The caller decides how to handle the error.
    match print_file_contents("example.txt") {
        Ok(()) => {}
        Err(e) => eprintln!("Failed to read file: {}", e),
    }
}
```

## Explanation

The `?` operator desugars to roughly this:

```rust
// This:
let content = fs::read_to_string(path)?;

// Is equivalent to:
let content = match fs::read_to_string(path) {
    Ok(val) => val,
    Err(err) => return Err(err.into()),
};
```

Notice the `return Err(...)`. This is a **return statement**. It exits the current function with an error value. But if the function's return type is `()`, where does `Err(...)` go? The types do not match: you cannot return `Err(io::Error)` from a function that returns nothing.

The fix is to change the return type to `Result<(), io::Error>`. The `()` is the success type (the function produces no meaningful value on success), and `io::Error` is the error type.

**Why Rust makes you do this:**

In languages with exceptions, errors propagate implicitly — a function can throw without declaring it, and callers may not know. In Rust, error propagation is **explicit in the type signature**. If a function can fail, its return type says so. This means:

1. **The caller always knows a function can fail** by looking at its signature.
2. **The caller must handle the error** — they cannot accidentally ignore it.
3. **Error paths are visible in the code** — every `?` is a potential early return.

**Using `?` in `main()`:**

The `main` function can also return `Result`:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string("example.txt")?;
    println!("{}", content);
    Ok(())
}
```

When `main` returns `Err`, Rust prints the error using its `Debug` representation and exits with a non-zero status code. `Box<dyn std::error::Error>` is a common choice for the error type in `main` because it can hold any error type.

**The `?` operator with `Option`:**

The `?` operator also works with `Option<T>`:

```rust
fn first_even(numbers: &[i32]) -> Option<i32> {
    let first = numbers.first()?; // Returns None if empty
    if first % 2 == 0 {
        Some(*first)
    } else {
        None
    }
}
```

When used on `Option`, `?` returns `None` on `None`. The enclosing function must return `Option<T>`.

**You cannot mix `Result` and `Option` with `?` in the same function** without converting between them (using `.ok_or()` to convert `Option` to `Result`, or `.ok()` to convert `Result` to `Option`).

## Compiler Error Interpretation

```
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option` (or another type that implements `FromResidual`)
 --> src/main.rs:5:48
  |
3 | fn print_file_contents(path: &str) {
  | ----------------------------------- this function should return `Result` or `Option` to accept `?`
4 |     let content = fs::read_to_string(path)?;
  |                                            ^ cannot use the `?` operator in a function that returns `()`
  |
  = help: the trait `FromResidual<Result<Infallible, std::io::Error>>` is not implemented for `()`
help: consider adding return type
  |
3 | fn print_file_contents(path: &str) -> Result<(), std::io::Error> {
  |                                     ++++++++++++++++++++++++++++
```

This is another example of the Rust compiler being an excellent teacher:

- **"the `?` operator can only be used in a function that returns `Result` or `Option`"** — Clear statement of the rule. The `?` needs a compatible return type because it performs early return.
- **"this function should return `Result` or `Option` to accept `?`"** — Points to the function signature as the source of the problem.
- **"cannot use the `?` operator in a function that returns `()`"** — Connects the dots: your function returns nothing, but `?` needs to return an error.
- **"consider adding return type"** — The compiler suggests the exact fix, including the full type signature `-> Result<(), std::io::Error>`.

The `FromResidual` trait mentioned in the error is the underlying mechanism that powers `?`. For practical purposes, you can ignore it — just follow the compiler's suggestion to add a `Result` return type.
