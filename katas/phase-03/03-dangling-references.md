---
id: dangling-references
phase: 3
phase_title: Lifetimes
sequence: 3
title: Dangling References
hints:
  - A reference must not outlive the data it points to
  - Returning a reference to a local variable creates a dangling reference
  - Return an owned value instead of a reference to a local
---

## Description

A dangling reference is a pointer to memory that has been freed. In C and C++, this is a common source of crashes and security vulnerabilities. Rust prevents dangling references at compile time — you simply cannot create one. If a function tries to return a reference to a local variable, the compiler will stop you.

## Broken Code

```rust
fn create_greeting(name: &str) -> &str {
    let greeting = format!("Hello, {}!", name);
    &greeting
}

fn main() {
    let message = create_greeting("Rust");
    println!("{}", message);
}
```

## Correct Code

```rust
fn create_greeting(name: &str) -> String {
    let greeting = format!("Hello, {}!", name);
    greeting
}

fn main() {
    let message = create_greeting("Rust");
    println!("{}", message);
}
```

## Explanation

In the broken code, `create_greeting` creates a local `String` called `greeting` using `format!`. It then tries to return `&greeting` — a reference to that local variable. But when the function returns, `greeting` goes out of scope and its memory is freed. The returned reference would point to deallocated memory — a dangling reference.

Rust prevents this at compile time. The compiler analyzes the lifetimes and determines that the reference would outlive the data it points to. No dangling reference is ever created.

The correct version returns an owned `String` instead of a reference. When you return `greeting` (without the `&`), ownership of the `String` is transferred to the caller. The data is moved, not freed. The caller — in this case `main` — becomes the new owner.

This is a fundamental rule: **a reference can never outlive the data it refers to.** If you need data to escape a function, you have two options:

1. **Return an owned value** (like `String` instead of `&str`) — the data moves to the caller.
2. **Return a reference to data that was passed in** — the data already lives outside the function.

You cannot return a reference to something created inside the function, because that something will be destroyed when the function returns.

This rule eliminates an entire category of bugs: use-after-free, dangling pointers, and accessing deallocated memory. In C, the equivalent code would compile and run — sometimes crashing, sometimes returning garbage data, sometimes appearing to work. Rust makes this impossible.

## Compiler Error Interpretation

```
error[E0515]: cannot return reference to local variable `greeting`
 --> main.rs:3:5
  |
3 |     &greeting
  |     ^^^^^^^^^ returns a reference to data owned by the current function
```

Error E0515 is one of the clearest error messages in Rust. It says exactly what is wrong: "returns a reference to data owned by the current function." The function owns `greeting`, and when the function ends, that data will be dropped. Returning a reference to it would create a dangling pointer. The fix is to return the owned value directly — change the return type from `&str` to `String` and remove the `&`.
