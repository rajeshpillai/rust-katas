---
id: when-to-use-macros
phase: 12
phase_title: Macros
sequence: 3
title: "When to Use Macros vs Generics"
hints:
  - If a function can be written as a generic, prefer generics over macros
  - Generics are type-checked, readable, and produce clear error messages
  - Macros are necessary when you need variadic arguments, code generation, or compile-time string manipulation
  - The key question is whether your abstraction operates on types (generics) or on syntax (macros)
---

## Description

Macros are powerful but come with costs: they are harder to read, harder to debug, and produce less clear error messages. Before reaching for `macro_rules!`, ask yourself: "Can this be a generic function?" If the answer is yes, use generics. Macros should be reserved for things that generics genuinely cannot express.

This kata shows an overcomplicated macro that should be a generic function, and then demonstrates a case where a macro IS the right tool.

## Broken Code

```rust
// An overcomplicated macro that wraps values and prints them.
// This could -- and should -- be a generic function.
macro_rules! debug_wrap {
    ($val:expr, i32) => {
        {
            let v: i32 = $val;
            println!("[debug i32] value = {}", v);
            v
        }
    };
    ($val:expr, f64) => {
        {
            let v: f64 = $val;
            println!("[debug f64] value = {}", v);
            v
        }
    };
    ($val:expr, &str) => {
        {
            let v: &str = $val;
            println!("[debug &str] value = {}", v);
            v
        }
    };
    // Every new type requires a new arm! This does not scale.
}

fn main() {
    let a = debug_wrap!(42, i32);
    let b = debug_wrap!(3.14, f64);
    let c = debug_wrap!("hello", &str);

    // This fails: no arm for String
    let d = debug_wrap!(String::from("world"), String);

    println!("a={}, b={}, c={}, d={}", a, b, c, d);
}
```

## Correct Code

```rust
use std::fmt;

// BETTER: A generic function replaces the entire macro
fn debug_wrap<T: fmt::Display>(val: T) -> T {
    println!("[debug {}] value = {}", std::any::type_name::<T>(), val);
    val
}

// WHEN MACROS ARE ACTUALLY NEEDED: variadic logging with file/line info
macro_rules! log_values {
    // This cannot be a function because:
    // 1. It accepts a variable number of arguments (variadic)
    // 2. It captures file!() and line!() at the CALL SITE, not the definition site
    // 3. It stringifies the expression for display ($key is shown as source code)
    ( $( $key:expr ),+ ) => {
        $(
            println!(
                "[{}:{}] {} = {:?}",
                file!(),
                line!(),
                stringify!($key),  // Turns the expression into a string at compile time
                $key
            );
        )+
    };
}

fn main() {
    // The generic function handles any type that implements Display
    let a = debug_wrap(42);
    let b = debug_wrap(3.14);
    let c = debug_wrap("hello");
    let d = debug_wrap(String::from("world"));

    println!("a={}, b={}, c={}, d={}", a, b, c, d);

    println!("\n--- Macro-powered logging (cannot be a function) ---");

    let x = 10;
    let y = vec![1, 2, 3];
    let z = x * 2 + 5;

    // The macro shows the EXPRESSION as source code, plus file and line
    log_values!(x, y, z, x + 100);
    // Output includes things like:
    //   [src/main.rs:38] x = 10
    //   [src/main.rs:38] y = [1, 2, 3]
    //   [src/main.rs:38] z = 25
    //   [src/main.rs:38] x + 100 = 110
}
```

## Explanation

The broken version uses a macro to do something that a generic function does better. The `debug_wrap!` macro has separate arms for `i32`, `f64`, and `&str`, each doing the exact same thing. Adding support for a new type (like `String`) requires adding another arm. This violates the DRY principle and does not scale.

**Why the macro approach is wrong here:**

1. **Repetition.** Every arm has identical logic -- only the type annotation differs.
2. **Not extensible.** Adding a new type means editing the macro.
3. **Poor error messages.** If you pass an unsupported type, the error says "no rules expected the token `String`" rather than explaining a trait bound.
4. **Harder to read.** Macro syntax is more complex than function syntax.

**Why a generic function is right:**

The function `fn debug_wrap<T: fmt::Display>(val: T) -> T` works with *any* type that implements `Display`. No arms to add. No repetition. Clear error messages. The compiler monomorphizes it for each concrete type, producing the same efficient code as the macro would.

**When IS a macro the right tool?**

The `log_values!` macro in the correct version demonstrates three capabilities that functions cannot provide:

1. **Variadic arguments.** Functions take a fixed number of parameters. Macros can accept any number using `$( ... ),*` repetition. `log_values!(x, y, z)` and `log_values!(a)` both work.

2. **Compile-time source information.** The macros `file!()` and `line!()` expand to the file name and line number of the *call site*. If these were inside a function, they would always report the function's location, not the caller's. Macros expand at the call site, so `file!()` and `line!()` capture where the macro was invoked.

3. **Expression stringification.** `stringify!($key)` converts the expression to a string at compile time. When you write `log_values!(x + 100)`, the output includes the literal text `"x + 100"`. A function receives the *value* 110; it has no access to the source code expression that produced it.

**Decision framework:**

| Need | Use |
|------|-----|
| Works with multiple types | Generic function with trait bounds |
| Fixed number of arguments | Function |
| Variable number of arguments | Macro |
| Need call-site file/line | Macro |
| Need to stringify expressions | Macro |
| Need to generate struct/enum definitions | Macro (or proc macro) |
| Need conditional compilation | Macro with `cfg!` |

Rule of thumb: if your macro's arms all do the same thing with different types, you want a generic function. If your macro needs to manipulate syntax, generate code, or capture call-site information, it is the right tool.

## Compiler Error Interpretation

```
error: no rules expected the token `String`
  --> src/main.rs:31:49
   |
2  | macro_rules! debug_wrap {
   | ------------------------ when calling this macro
...
31 |     let d = debug_wrap!(String::from("world"), String);
   |                                                 ^^^^^^ no rules expected this token in macro call
```

The macro has no arm matching the `String` type. This is a symptom of the underlying design problem: the macro should not be enumerating types at all. If you find yourself adding arms to a macro for each new type, step back and consider a generic function. Generics handle new types automatically through trait bounds, and the compiler provides clear, helpful error messages when a type does not satisfy the bounds.

Compare with the generic function approach -- if you tried to pass a type that does not implement `Display`:

```
error[E0277]: `MyStruct` doesn't implement `std::fmt::Display`
```

This error tells you exactly what trait to implement, which is far more actionable than "no rules expected this token."
