---
id: ffi-basics
phase: 16
phase_title: "Advanced Systems Patterns"
sequence: 1
title: FFI Basics — Calling C Functions from Rust
hints:
  - Functions declared with extern "C" are foreign function interface declarations
  - The compiler cannot verify the safety of code written in another language
  - All calls to extern functions must be wrapped in unsafe blocks
  - You are responsible for ensuring the C function's preconditions are met
---

## Description

Rust can call functions written in C (and other languages) through the Foreign Function Interface (FFI). To do this, you declare the foreign function's signature using `extern "C"` blocks. However, the Rust compiler cannot verify the correctness of foreign code -- it cannot check that the C function handles null pointers correctly, respects memory boundaries, or avoids undefined behavior. For this reason, calling any `extern "C"` function requires an `unsafe` block. You are asserting that the foreign function's contract is upheld.

## Broken Code

```rust
// Declare a C standard library function
extern "C" {
    /// Computes the absolute value of an integer.
    /// C signature: int abs(int n);
    fn abs(n: i32) -> i32;
}

fn main() {
    let x: i32 = -42;

    // BUG: Calling an extern function without an unsafe block.
    // The compiler cannot verify the safety of foreign code.
    let result = abs(x);

    println!("abs({}) = {}", x, result);
}
```

## Correct Code

```rust
// Declare a C standard library function
extern "C" {
    /// Computes the absolute value of an integer.
    /// C signature: int abs(int n);
    fn abs(n: i32) -> i32;
}

fn main() {
    let x: i32 = -42;

    // Correct: wrap the call in an unsafe block.
    // Safety: `abs` is a well-defined C standard library function.
    // It accepts any i32 value and returns a non-negative i32.
    // Note: abs(i32::MIN) is undefined behavior in C due to signed
    // integer overflow, but for typical values this is safe.
    let result = unsafe { abs(x) };

    println!("abs({}) = {}", x, result);
}
```

## Explanation

The broken version calls `abs(x)` directly, without an `unsafe` block. Since `abs` is declared in an `extern "C"` block, Rust treats it as an unsafe function. All extern functions are implicitly `unsafe` because:

1. **The compiler cannot verify foreign code.** Rust's safety guarantees come from its type system and borrow checker. C code exists outside this system. The C function could read uninitialized memory, overflow a buffer, or invoke undefined behavior -- and Rust would have no way to know.

2. **The ABI contract is not enforced.** When you write `fn abs(n: i32) -> i32`, you are telling Rust the C function takes one `i32` and returns one `i32`. If the actual C function has a different signature, the call will silently corrupt the stack or registers. The compiler trusts your declaration.

3. **Side effects are invisible.** A C function might modify global state, write to files, or allocate memory. Rust's ownership system cannot track these effects.

**Building a safe wrapper:**

In practice, you should wrap unsafe extern calls in safe Rust functions that validate inputs and document the contract:

```rust
extern "C" {
    fn abs(n: i32) -> i32;
}

/// Returns the absolute value of `n`.
///
/// # Panics
///
/// Panics if `n` is `i32::MIN`, because `abs(i32::MIN)` is
/// undefined behavior in C (signed integer overflow).
fn safe_abs(n: i32) -> i32 {
    assert!(n != i32::MIN, "abs(i32::MIN) is undefined behavior");
    // Safety: `n` is not i32::MIN, so abs(n) is well-defined.
    unsafe { abs(n) }
}
```

This wrapper enforces the C function's precondition (no `i32::MIN`) at the Rust level, creating a safe boundary around unsafe code.

The invariant violated in the broken code: **calling an `extern "C"` function requires `unsafe` because the compiler cannot verify the correctness of foreign code.**

## Compiler Error Interpretation

```
error[E0133]: call to unsafe function `abs` is unsafe and requires
              unsafe function or block
  --> src/main.rs:12:18
   |
12 |     let result = abs(x);
   |                  ^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on
           how to avoid undefined behavior
```

The compiler error tells you:

1. **"call to unsafe function `abs` is unsafe"** -- all `extern "C"` functions are implicitly unsafe. You cannot call them in safe code.
2. **"requires unsafe function or block"** -- you must either wrap the call in `unsafe { }` or mark the calling function as `unsafe fn`.
3. **"consult the function's documentation"** -- the compiler reminds you that unsafe operations have preconditions. For FFI calls, you should read the C function's documentation (in this case, the C standard library documentation for `abs`) to understand what inputs are valid and what behavior to expect.

The fix is syntactically simple (add `unsafe { }`), but the deeper responsibility is understanding the foreign function's contract and ensuring your Rust code upholds it.
