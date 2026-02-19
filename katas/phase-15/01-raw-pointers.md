---
id: raw-pointers
phase: 15
phase_title: "Unsafe Rust"
sequence: 1
title: Raw Pointers and Unsafe Dereferencing
hints:
  - Creating a raw pointer is safe -- dereferencing it is not
  - "*const T" is a raw immutable pointer, "*mut T" is a raw mutable pointer
  - You must use an unsafe block to dereference a raw pointer
  - When you write unsafe, YOU are the borrow checker -- you promise the pointer is valid
---

## Description

Rust has two kinds of raw pointers: `*const T` (immutable) and `*mut T` (mutable). Unlike references (`&T` and `&mut T`), raw pointers have no safety guarantees. They can be null, dangling, or misaligned. They bypass the borrow checker entirely. Creating a raw pointer is safe -- it is just a number representing a memory address. But dereferencing a raw pointer (reading or writing the memory it points to) is unsafe, because Rust cannot verify the pointer is valid. You must wrap the dereference in an `unsafe` block, which is your assertion to the compiler: "I have manually verified this is correct."

## Broken Code

```rust
fn main() {
    let value: i32 = 42;

    // Creating a raw pointer is safe
    let ptr: *const i32 = &value as *const i32;

    // BUG: Dereferencing a raw pointer outside an unsafe block
    let retrieved = *ptr;

    println!("Value: {}", retrieved);
}
```

## Correct Code

```rust
fn main() {
    let value: i32 = 42;

    // Creating a raw pointer is safe
    let ptr: *const i32 = &value as *const i32;

    // Correct: dereference inside an unsafe block.
    // Safety: `ptr` was created from a valid reference to `value`,
    // which is still alive and in scope. The pointer is valid,
    // aligned, and points to an initialized i32.
    let retrieved = unsafe { *ptr };

    println!("Value: {}", retrieved);
}
```

## Explanation

The broken version dereferences `*ptr` in safe code. The compiler rejects this because dereferencing a raw pointer is one of the operations that require `unsafe`.

**Why raw pointers exist:**

Raw pointers serve several purposes in Rust:
- **FFI**: When calling C code, you work with C pointers, which are raw pointers in Rust.
- **Data structures**: Some data structures (like linked lists, trees, or graphs) are difficult to express with Rust's ownership model and may use raw pointers internally.
- **Performance**: In rare cases, bypassing the borrow checker allows optimizations the compiler cannot verify automatically.

**What unsafe means:**

An `unsafe` block does not disable all safety checks. It enables exactly five additional capabilities:
1. Dereference a raw pointer
2. Call an unsafe function
3. Access or modify a mutable static variable
4. Implement an unsafe trait
5. Access fields of a union

Everything else -- type checking, borrow checking on references, bounds checking on slices -- still works inside `unsafe`. The `unsafe` keyword is not a declaration that the code is dangerous; it is a contract saying "I have verified the invariants the compiler cannot check."

**The safety comment:**

In the correct version, the comment starting with `// Safety:` explains why the dereference is valid. This is a strong convention in Rust. Every `unsafe` block should have a safety comment explaining:
- Why the pointer is valid (not null, not dangling)
- Why the pointer is properly aligned
- Why the pointed-to memory is initialized
- Why no aliasing rules are violated

This is not enforced by the compiler, but it is essential for code review and maintenance.

The invariant violated in the broken code: **dereferencing a raw pointer requires an `unsafe` block because the compiler cannot verify the pointer is valid.**

## Compiler Error Interpretation

```
error[E0133]: dereference of raw pointer is unsafe and requires
              unsafe function or block
 --> src/main.rs:8:21
  |
8 |     let retrieved = *ptr;
  |                     ^^^^ dereference of raw pointer
  |
  = note: raw pointers may be null, dangling or unaligned;
          they can violate aliasing rules and cause data races:
          all of these are undefined behavior
```

The compiler error is direct and educational:

1. **"dereference of raw pointer is unsafe and requires unsafe function or block"** -- the operation you are attempting requires explicit opt-in via `unsafe`.
2. **"raw pointers may be null, dangling or unaligned"** -- the compiler explains what could go wrong with raw pointers.
3. **"they can violate aliasing rules and cause data races: all of these are undefined behavior"** -- the compiler lists the full category of risks. Unlike references, raw pointers have no guarantees about aliasing (multiple pointers to the same memory) or thread safety.

The fix is simple: wrap the dereference in `unsafe { }`. But the deeper lesson is that you must understand and verify the invariants yourself. The `unsafe` block is not magic -- it transfers responsibility from the compiler to you.
