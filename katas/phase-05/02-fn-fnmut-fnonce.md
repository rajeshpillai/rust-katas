---
id: fn-fnmut-fnonce
phase: 5
phase_title: "Closures & the Fn Traits"
sequence: 2
title: "Fn, FnMut, and FnOnce"
hints:
  - "A closure that consumes (moves out of) a captured variable can only be called once — it implements `FnOnce`."
  - "If a function parameter is `Fn`, the closure must be callable multiple times without consuming anything."
  - "Either stop consuming the captured value inside the closure, or change the bound from `Fn` to `FnOnce`."
---

## Description

Rust classifies closures by the traits they implement, based on how they interact with their captured variables:

- **`Fn`**: The closure only reads captured variables (borrows immutably). It can be called any number of times.
- **`FnMut`**: The closure mutates captured variables (borrows mutably). It can be called multiple times, but requires mutable access.
- **`FnOnce`**: The closure consumes (moves out of) captured variables. It can only be called **once**, because after that the consumed value no longer exists.

These traits form a hierarchy: every `Fn` is also `FnMut`, and every `FnMut` is also `FnOnce`. But the reverse is not true.

## Broken Code

```rust
fn apply_twice<F: Fn() -> String>(f: F) -> String {
    let first = f();
    let second = f();
    format!("{} | {}", first, second)
}

fn main() {
    let name = String::from("Alice");

    // This closure moves `name` out of its capture (via the return).
    // After the first call, `name` is gone. It can only be called once.
    // But apply_twice requires Fn (callable multiple times).
    let greet = || {
        let greeting = format!("Hello, {}!", name);
        drop(name); // explicitly consumes `name`
        greeting
    };

    let result = apply_twice(greet);
    println!("{}", result);
}
```

## Correct Code

```rust
fn apply_twice<F: Fn() -> String>(f: F) -> String {
    let first = f();
    let second = f();
    format!("{} | {}", first, second)
}

fn main() {
    let name = String::from("Alice");

    // This closure only reads `name` — it borrows it immutably.
    // It never consumes it, so it can be called multiple times.
    // It implements Fn.
    let greet = || {
        format!("Hello, {}!", name)
    };

    let result = apply_twice(greet);
    println!("{}", result);
}
```

## Explanation

The broken version creates a closure that calls `drop(name)`, which takes ownership of `name` and destroys it. After the first call, `name` no longer exists inside the closure's captured state. A second call would try to use a value that has already been consumed — a use-after-move error.

Because this closure consumes a captured variable, it can only implement `FnOnce`. But `apply_twice` requires `Fn`, which guarantees the closure can be called any number of times. The types are incompatible.

The fix is simple: stop consuming `name`. The `format!` macro only needs a reference to `name` (it calls `Display::fmt` which takes `&self`). By removing the `drop(name)`, the closure merely borrows `name` immutably and can be called as many times as needed.

**The trait hierarchy in detail:**

```
FnOnce   (most permissive — can do anything, but may only be called once)
  ^
  |
FnMut    (can mutate captured state, callable multiple times)
  ^
  |
Fn       (only reads captured state, callable multiple times)
```

When you write a function that accepts a closure, choose the **least restrictive** bound you need:

| Your function needs to... | Use this bound |
|---|---|
| Call the closure once | `FnOnce` |
| Call the closure multiple times, closure may mutate state | `FnMut` |
| Call the closure multiple times, closure is pure | `Fn` |

Conversely, when you write a closure:

| Your closure does... | It implements... |
|---|---|
| Only reads captures | `Fn` + `FnMut` + `FnOnce` |
| Mutates captures | `FnMut` + `FnOnce` (not `Fn`) |
| Consumes captures | `FnOnce` only |

**Alternative fix:** If you truly need the closure to consume `name`, change the bound:

```rust
fn apply_once<F: FnOnce() -> String>(f: F) -> String {
    f() // can only call once
}
```

## Compiler Error Interpretation

```
error[E0525]: expected a closure that implements the `Fn` trait, but this closure only implements `FnOnce`
  --> src/main.rs:11:17
   |
11 |     let greet = || {
   |                 ^^ this closure implements `FnOnce`, not `Fn`
12 |         let greeting = format!("Hello, {}!", name);
13 |         drop(name);
   |              ---- closure is `FnOnce` because it moves the variable `name` out of its environment
   |
note: the requirement to implement `Fn` derives from here
  --> src/main.rs:1:20
   |
1  | fn apply_twice<F: Fn() -> String>(f: F) -> String {
   |                   ^^^^^^^^^^^^^^
```

This error tells you exactly what went wrong:

- **"expected a closure that implements the `Fn` trait, but this closure only implements `FnOnce`"** — The mismatch between what the function requires and what the closure provides.
- **"closure is `FnOnce` because it moves the variable `name` out of its environment"** — The compiler identifies the exact operation that makes the closure `FnOnce` instead of `Fn`. It even points to the specific line (`drop(name)`).
- **"the requirement to implement `Fn` derives from here"** — It shows you where the `Fn` bound is declared, so you can decide whether to change the bound or change the closure.

The compiler is guiding you toward a decision: either make the closure less consuming (fix the closure) or make the function less demanding (relax the bound).
