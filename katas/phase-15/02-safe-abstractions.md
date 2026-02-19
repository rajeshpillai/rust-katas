---
id: safe-abstractions
phase: 15
phase_title: "Unsafe Rust"
sequence: 2
title: Building Safe Abstractions Over Unsafe Code
hints:
  - A safe function must not allow callers to trigger undefined behavior
  - Returning a raw pointer from a safe function is dangerous because the caller cannot know if it is valid
  - The solution is to encapsulate unsafe operations behind a safe API with proper lifetime constraints
  - Use references with lifetimes to tie the returned value to the source data's lifetime
---

## Description

The golden rule of unsafe Rust is: **unsafe code must create safe boundaries**. This means that while you may use `unsafe` internally, the public API of your function or type must be safe -- callers should never be able to trigger undefined behavior through normal use. A common mistake is writing a "safe" function that actually exposes unsafe behavior to callers, such as returning a pointer that could dangle. The correct approach is to encapsulate the unsafe operation and expose a safe interface using Rust's type system (especially lifetimes) to enforce correctness.

## Broken Code

```rust
/// Returns a pointer to the first element of a slice.
/// BUG: This function is marked safe, but returns a raw pointer
/// that the caller could dereference after the slice is freed.
fn first_element_ptr(data: &[i32]) -> *const i32 {
    data.as_ptr()
}

fn main() {
    let ptr;
    {
        let numbers = vec![10, 20, 30];
        ptr = first_element_ptr(&numbers);
        // `numbers` is dropped here
    }

    // Undefined behavior: dereferencing a pointer to freed memory.
    // The compiler cannot catch this because the safe function
    // returned a raw pointer with no lifetime information.
    unsafe {
        println!("First element: {}", *ptr); // dangling pointer!
    }
}
```

## Correct Code

```rust
/// Returns a reference to the first element of a slice.
/// The returned reference's lifetime is tied to the input slice,
/// so it cannot outlive the data it points to.
fn first_element(data: &[i32]) -> Option<&i32> {
    data.first()
}

fn main() {
    let numbers = vec![10, 20, 30];
    if let Some(first) = first_element(&numbers) {
        println!("First element: {}", first);
    }
    // `numbers` lives long enough. The borrow checker enforces this.
}
```

## Explanation

The broken version exposes a fundamental design flaw: a "safe" function returns a raw pointer that carries no lifetime information. Once the caller has the raw pointer, the compiler cannot track whether the underlying data still exists. The caller can (and does) dereference the pointer after the `Vec` is dropped, causing use-after-free -- classic undefined behavior.

**The problem is the API, not the implementation:**

The issue is not that `as_ptr()` is called internally. The issue is that the function's return type (`*const i32`) strips away all of Rust's safety guarantees. A raw pointer is just a memory address with no lifetime, no validity guarantee, and no aliasing rules. By returning it from a safe function, you create a hole in Rust's safety model.

**The safe abstraction:**

The correct version returns `Option<&i32>` instead of `*const i32`. The lifetime of the returned reference is tied to the input slice's lifetime (via lifetime elision: both share the same anonymous lifetime). Now the borrow checker enforces that the reference cannot outlive the slice. If you tried the same pattern as the broken version:

```rust
let result;
{
    let numbers = vec![10, 20, 30];
    result = first_element(&numbers);
    // numbers dropped here
}
println!("{:?}", result); // COMPILE ERROR: `numbers` does not live long enough
```

The compiler would catch this at compile time. That is the power of a safe abstraction: undefined behavior becomes a compile error.

**When returning raw pointers is acceptable:**

Sometimes you genuinely need to return a raw pointer -- for example, in FFI code or when building custom data structures. In those cases, the function should be marked `unsafe fn`, and the safety requirements should be documented:

```rust
/// Returns a pointer to the first element.
///
/// # Safety
///
/// The caller must ensure the returned pointer is not dereferenced
/// after the input slice's data is freed.
unsafe fn first_element_ptr(data: &[i32]) -> *const i32 {
    data.as_ptr()
}
```

By marking the function `unsafe`, you shift responsibility to the caller and make the danger explicit.

The invariant violated in the broken code: **a safe function must not allow callers to trigger undefined behavior; returning a raw pointer from a safe function breaks this contract because the pointer can outlive the data it points to.**

## Compiler Error Interpretation

The broken version compiles without errors -- and that is exactly the problem. The compiler trusts safe functions to uphold Rust's safety invariants. When a safe function returns a raw pointer, the compiler cannot track the pointer's validity, so the dangling dereference goes undetected.

This is what makes unsafe abstraction design so critical. The compiler gives you no warning here. The bug manifests as undefined behavior at runtime: the program might print garbage, crash, or appear to work correctly (which is the most dangerous outcome, because the bug is hidden).

If the function returned a reference instead, the compiler would produce:

```
error[E0597]: `numbers` does not live long enough
  --> src/main.rs:5:33
   |
4  |         let numbers = vec![10, 20, 30];
   |             ------- binding `numbers` declared here
5  |         ptr = first_element(&numbers);
   |                              ^^^^^^^ borrowed value does not live long enough
6  |     }
   |     - `numbers` dropped here while still borrowed
7  |
8  |     println!("First element: {}", ptr);
   |                                   --- borrow later used here
```

This error shows Rust's borrow checker doing its job: it traces the lifetime of the borrow from creation to use, identifies where the data is dropped, and rejects the code. The safe abstraction converts runtime undefined behavior into a compile-time error. That is the purpose of safe wrappers around unsafe code.
