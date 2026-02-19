---
id: closure-capture
phase: 5
phase_title: "Closures & the Fn Traits"
sequence: 1
title: Closure Capture and Ownership
hints:
  - "Closures capture variables from their enclosing scope. By default, they borrow."
  - "If the original variable is moved or dropped before the closure runs, the borrow becomes invalid."
  - "Use the `move` keyword to force the closure to take ownership of captured variables."
---

## Description

Closures in Rust are anonymous functions that can capture variables from their enclosing environment. By default, Rust infers the most lightweight capture mode: it will borrow by shared reference (`&T`) if possible, by mutable reference (`&mut T`) if the closure mutates the value, or by value (move) if the closure consumes the value.

This automatic inference works well in many cases, but problems arise when the closure outlives the data it captures by reference. If the original variable is moved away or goes out of scope before the closure executes, the closure holds a dangling reference. Rust prevents this at compile time.

## Broken Code

```rust
fn make_greeter() -> impl Fn() -> String {
    let name = String::from("Alice");

    // This closure captures `name` by reference (borrowing it).
    // But `name` is a local variable that will be dropped when
    // make_greeter() returns. The closure would hold a dangling reference.
    let greeter = || {
        format!("Hello, {}!", name)
    };

    greeter
    // `name` is dropped here, but `greeter` still references it
}

fn main() {
    let greet = make_greeter();
    println!("{}", greet());
}
```

## Correct Code

```rust
fn make_greeter() -> impl Fn() -> String {
    let name = String::from("Alice");

    // `move` forces the closure to take ownership of `name`.
    // The String is moved into the closure, so it lives as long
    // as the closure does. No dangling reference.
    let greeter = move || {
        format!("Hello, {}!", name)
    };

    greeter
    // `name` has been moved into the closure; nothing is dropped here
}

fn main() {
    let greet = make_greeter();
    println!("{}", greet());
}
```

## Explanation

The broken version fails because of a fundamental rule: **references must not outlive the data they point to**. Here is the timeline:

1. `make_greeter()` is called. `name` is created on the stack.
2. The closure `greeter` is created. By default, it captures `name` by reference (`&name`).
3. `make_greeter()` returns the closure. The closure is moved to the caller.
4. The stack frame of `make_greeter()` is destroyed. `name` is dropped.
5. The caller calls `greet()`. The closure tries to access `name` via its reference — but `name` no longer exists.

Rust catches this at step 3: the closure's lifetime extends beyond the function, but the reference it holds does not.

The `move` keyword changes the capture mode: instead of borrowing `name`, the closure **takes ownership** of it. The `String` is moved into the closure's internal state. Now the closure carries its own copy of the data, and it does not depend on the function's stack frame. The data lives as long as the closure does.

**Key insight:** `move` does not change what the closure *does* — it changes how the closure *captures*. A `move` closure that only reads a value will still implement `Fn`, not `FnOnce`. The `move` keyword is about ownership of the captured variables, not about what the closure does with them.

**When to use `move`:**
- Returning closures from functions (the most common case)
- Sending closures to other threads (`std::thread::spawn` requires `move`)
- Any time the closure must outlive the scope where it was created

## Compiler Error Interpretation

```
error[E0373]: closure may outlive the current function, but it borrows `name`, which is owned by the current function
 --> src/main.rs:4:19
  |
4 |     let greeter = || {
  |                   ^^ may outlive borrowed value `name`
5 |         format!("Hello, {}!", name)
  |                               ---- `name` is borrowed here
  |
note: closure is returned here
 --> src/main.rs:8:5
  |
8 |     greeter
  |     ^^^^^^^
help: to force the closure to take ownership of `name` (and any other referenced variables), use the `move` keyword
  |
4 |     let greeter = move || {
  |                   ++++
```

This error is exceptionally clear:

- **"closure may outlive the current function"** — The compiler detected that the closure escapes the function scope (because it is returned).
- **"but it borrows `name`, which is owned by the current function"** — The variable `name` lives on the stack of this function. When the function returns, `name` is destroyed, but the closure still holds a borrow.
- **"to force the closure to take ownership... use the `move` keyword"** — The compiler tells you the exact fix. Adding `move` transfers ownership of `name` into the closure, so the closure is self-contained.

This is one of those moments where the Rust compiler is genuinely teaching you. The error message explains the problem, identifies the specific variable, and provides the exact solution.
