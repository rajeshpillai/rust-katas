---
id: move-closures
phase: 5
phase_title: "Closures & the Fn Traits"
sequence: 3
title: "move Closures and Ownership Transfer"
hints:
  - "A closure returned from a function must own all the data it uses — it cannot borrow locals."
  - "The `move` keyword transfers ownership of all captured variables into the closure."
  - "Without `move`, the closure borrows, and borrows cannot outlive the scope they borrow from."
---

## Description

When a closure needs to outlive the scope in which it was created — for example, when it is returned from a function or sent to another thread — it must **own** all the data it references. The `move` keyword forces all captured variables to be moved into the closure, rather than borrowed.

This kata focuses on the pattern of returning closures from functions, which is one of the most common situations where `move` is required.

## Broken Code

```rust
fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
    // `x` is a local parameter. Without `move`, the closure
    // captures `x` by reference. But `x` lives on the stack
    // of make_adder, which is about to return.
    |y| x + y
}

fn make_counter() -> impl FnMut() -> i32 {
    let mut count = 0;

    // `count` is a local variable. The closure borrows it mutably.
    // But `count` will be dropped when make_counter returns.
    || {
        count += 1;
        count
    }
}

fn main() {
    let add_five = make_adder(5);
    println!("{}", add_five(3));

    let mut counter = make_counter();
    println!("{}", counter());
    println!("{}", counter());
}
```

## Correct Code

```rust
fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
    // `move` transfers ownership of `x` into the closure.
    // Since i32 is Copy, this actually copies the value in.
    // The closure now owns its own copy of `x`.
    move |y| x + y
}

fn make_counter() -> impl FnMut() -> i32 {
    let mut count = 0;

    // `move` transfers ownership of `count` into the closure.
    // The closure now owns `count` and can mutate it freely.
    // It still implements FnMut (not just FnOnce) because it
    // mutates but does not consume `count`.
    move || {
        count += 1;
        count
    }
}

fn main() {
    let add_five = make_adder(5);
    println!("{}", add_five(3));  // prints: 8
    println!("{}", add_five(10)); // prints: 15

    let mut counter = make_counter();
    println!("{}", counter()); // prints: 1
    println!("{}", counter()); // prints: 2
    println!("{}", counter()); // prints: 3
}
```

## Explanation

Both functions in the broken version share the same fundamental problem: they return a closure that captures local variables by reference, but those variables are destroyed when the function returns.

**For `make_adder`:** The parameter `x` has type `i32`, which implements `Copy`. When we add `move`, the closure gets its own copy of `x`. This is cheap — it is just copying 4 bytes. The closure is self-contained and can outlive the function.

**For `make_counter`:** The variable `count` has type `i32` (also `Copy`). With `move`, the closure takes ownership of its own copy of `count`. Each call to `counter()` mutates the closure's internal `count`. This is why the closure must be declared `mut` at the call site — calling it mutates its internal state.

**Critical insight about `move` and `Fn` traits:**

Adding `move` does **not** automatically make a closure `FnOnce`. The `move` keyword only changes *how* variables are captured (by value instead of by reference). The `Fn`/`FnMut`/`FnOnce` classification still depends on *what the closure does* with those captured values:

| Capture mode | Closure body | Trait |
|---|---|---|
| `move`, only reads captured values | `x + y` | `Fn` |
| `move`, mutates captured values | `count += 1` | `FnMut` |
| `move`, consumes captured values | `drop(name)` | `FnOnce` |

In `make_adder`, the closure only reads `x`, so it implements `Fn` despite using `move`.
In `make_counter`, the closure mutates `count`, so it implements `FnMut`.

**The `move` keyword is about ownership transfer, not about single-use.**

**Pattern summary — returning closures from functions:**

1. The return type is `impl Fn(...)`, `impl FnMut(...)`, or `impl FnOnce(...)`.
2. The closure almost always needs `move` (because it must outlive the function).
3. The specific `Fn` trait depends on what the closure does with its captures.

## Compiler Error Interpretation

```
error[E0373]: closure may outlive the current function, but it borrows `x`, which is owned by the current function
 --> src/main.rs:3:5
  |
3 |     |y| x + y
  |     ^^^ - `x` is borrowed here
  |     |
  |     may outlive borrowed value `x`
  |
note: closure is returned here
 --> src/main.rs:3:5
  |
3 |     |y| x + y
  |     ^^^^^^^^^
help: to force the closure to take ownership of `x` (and any other referenced variables), use the `move` keyword
  |
3 |     move |y| x + y
  |     ++++
```

This error follows the same pattern as the previous kata:

- **"closure may outlive the current function"** — The compiler detects that the closure escapes the function scope.
- **"but it borrows `x`, which is owned by the current function"** — The captured variable `x` is a local (a function parameter), so it will be dropped when the function returns. A borrow of it cannot escape.
- **"use the `move` keyword"** — The fix is explicit and mechanical. The compiler even shows you exactly where to add `move`.

For the `make_counter` function, the error is analogous but refers to `count` instead of `x`. The same fix applies: add `move` to transfer ownership into the closure.

Note that this error (`E0373`) is the same error code for any case where a closure outlives borrowed data — whether returning from a function, spawning a thread, or any other scenario. The pattern and the fix are always the same: use `move` to give the closure ownership of the data it needs.
