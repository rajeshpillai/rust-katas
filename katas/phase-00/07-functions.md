---
id: functions
phase: 0
phase_title: "Rust as a Language"
sequence: 7
title: Functions — The Basic Unit of Code
hints:
  - "Every function must declare the types of its parameters and its return type (unless it returns nothing)."
  - "The last expression in a function body is the return value — no semicolon needed."
  - "Adding a semicolon to the last line turns it into a statement, which returns `()` (the unit type) instead of the value."
---

## Description

Functions in Rust are declared with `fn`. Unlike dynamically typed languages, Rust requires you to annotate the types of every parameter and the return type. This makes the function signature a contract — callers know exactly what to provide and what they get back.

The last expression in a function body (without a semicolon) is automatically returned. Adding a semicolon turns it into a statement, which discards the value.

## Broken Code

```rust
fn add(x, y) {
    x + y;
}

fn greet(name: &str) -> String {
    println!("Hello, {}!", name);
}

fn main() {
    let sum = add(3, 5);
    println!("3 + 5 = {}", sum);

    let greeting = greet("Rust");
    println!("{}", greeting);
}
```

## Correct Code

```rust
// Parameters must have type annotations
fn add(x: i32, y: i32) -> i32 {
    x + y  // No semicolon = this is the return value
}

// Multiple return values using a tuple
fn divide(a: f64, b: f64) -> (f64, f64) {
    let quotient = a / b;
    let remainder = a % b;
    (quotient, remainder)  // Return a tuple
}

// Functions that return nothing have implicit return type ()
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

// Explicit early return with `return` keyword
fn classify_age(age: u32) -> &'static str {
    if age < 13 {
        return "child";
    }
    if age < 20 {
        return "teenager";
    }
    "adult"  // Last expression — no return keyword needed
}

fn main() {
    // Basic function calls
    let sum = add(3, 5);
    println!("3 + 5 = {}", sum);

    // Tuple destructuring on return
    let (q, r) = divide(17.0, 5.0);
    println!("17 / 5 = {} remainder {}", q, r);

    // Void function (returns unit type)
    greet("Rust");

    // Early return example
    println!("Age 8: {}", classify_age(8));
    println!("Age 16: {}", classify_age(16));
    println!("Age 30: {}", classify_age(30));
}
```

## Explanation

The broken code has two bugs:

**Bug 1: Missing type annotations on parameters.**

```rust
fn add(x, y) {  // Error: expected type, found `,`
```

Rust requires explicit types on all function parameters. Unlike `let` bindings where the type can be inferred, function signatures are always explicit. This is intentional — function signatures are the API boundary, and ambiguity there would propagate confusion through the entire program.

**Bug 2: Semicolon on the return expression.**

```rust
fn add(x: i32, y: i32) -> i32 {
    x + y;  // Semicolon turns this into a statement — returns () not i32
}
```

In Rust, the last expression in a block is its value. Adding `;` turns an expression into a statement, which evaluates to `()` (the unit type). So `x + y;` returns `()`, but the function signature promises `-> i32`.

**The `greet` function** has a similar issue: it declares `-> String` but the body only calls `println!`, which returns `()`. The function never constructs a `String` to return.

**Key rules:**
- Every parameter needs a type: `fn name(param: Type)`
- Return type follows `->`: `fn name() -> ReturnType`
- No return type means `-> ()` (unit type, like void)
- Last expression (no semicolon) is the return value
- Use `return` keyword for early exit from a function

## ⚠️ Caution

- The semicolon trap is the most common Rust beginner mistake. If your function "should return X but returns ()", check for an accidental semicolon on the last line.
- Functions in Rust do not support default parameter values. Use `Option<T>` or builder patterns instead.
- Rust does not support function overloading (multiple functions with the same name but different parameter types). Use generics or traits instead.

## 💡 Tips

- Return multiple values with tuples: `fn split() -> (i32, i32)`. Destructure at the call site: `let (a, b) = split();`.
- Use `-> !` (the "never" type) for functions that never return (e.g., `fn exit() -> !`).
- Functions defined in Rust can be called before their definition — there is no forward declaration requirement.

## Compiler Error Interpretation

```
error: expected one of `:`, `@`, or `|`, found `,`
 --> main.rs:1:10
  |
1 | fn add(x, y) {
  |          ^ expected one of `:`, `@`, or `|`
```

The compiler expects a type annotation after each parameter name. `x` must be followed by `: Type`, not `,`.

For the semicolon issue:

```
error[E0308]: mismatched types
 --> main.rs:2:5
  |
1 | fn add(x: i32, y: i32) -> i32 {
  |                            --- expected `i32` because of return type
2 |     x + y;
  |          - help: remove this semicolon to return this value
  |     expected `i32`, found `()`
```

The compiler even suggests the fix: "remove this semicolon to return this value." This is one of Rust's most helpful error messages.

---

| [Prev: Type Inference and Annotations](#/katas/type-inference-and-annotations) | [Next: Control Flow — if, loops, and match](#/katas/control-flow) |
