---
id: aliasing-and-mutation
phase: 2
phase_title: Borrowing
sequence: 3
title: Aliasing and Mutation
hints:
  - You cannot hold an immutable reference and a mutable reference at the same time
  - Immutable references assume the data will not change while they exist
  - The conflict arises when borrows overlap — finish reading before writing
---

## Description

Rust forbids having an immutable reference and a mutable reference to the same data at the same time. This is the aliasing-and-mutation rule: you can alias (multiple readers) or you can mutate (one writer), but never both simultaneously. This rule is what makes Rust's safety guarantees possible.

## Broken Code

```rust
fn main() {
    let mut items = vec![1, 2, 3, 4, 5];

    let first = &items[0];

    items.push(6);

    println!("The first item is: {}", first);
}
```

## Correct Code

```rust
fn main() {
    let mut items = vec![1, 2, 3, 4, 5];

    let first = items[0];

    items.push(6);

    println!("The first item is: {}", first);
}
```

## Explanation

This kata demonstrates one of the most important bugs that Rust prevents: **iterator invalidation**.

In the broken code, `&items[0]` creates an immutable reference pointing into the Vec's internal buffer. Then `items.push(6)` might need to reallocate that buffer if there is not enough capacity. If reallocation happens, all existing references into the old buffer become dangling pointers — they point to freed memory. This is a classic memory safety bug in C and C++.

Rust prevents this by recognizing that `&items[0]` is an immutable borrow of `items`, and `items.push(6)` requires a mutable borrow of `items`. Since the immutable borrow (`first`) is used after the mutable borrow (`push`), their lifetimes overlap, and the compiler rejects the code.

The correct version copies the value with `let first = items[0]` instead of borrowing it. Since `i32` implements `Copy`, this creates an independent copy that is not tied to the Vec's internal buffer. Now `push` can reallocate freely without invalidating anything.

Alternative fixes include:
- Do the `push` before taking the reference
- Finish using `first` before calling `push`
- Clone the value if it is not `Copy`

The underlying principle is: **aliasing (reading through a reference) and mutation (writing through a reference) cannot coexist.** Rust encodes time in types — the borrow checker reasons about when references are alive and ensures they never conflict.

## Compiler Error Interpretation

```
error[E0502]: cannot borrow `items` as mutable because it is also borrowed as immutable
 --> main.rs:6:5
  |
4 |     let first = &items[0];
  |                  ----- immutable borrow occurs here
5 |
6 |     items.push(6);
  |     ^^^^^^^^^^^^^ mutable borrow occurs here
7 |
8 |     println!("The first item is: {}", first);
  |                                       ----- immutable borrow later used here
```

Error E0502 captures the aliasing-and-mutation conflict. It shows three locations: where the immutable borrow was created (line 4), where the conflicting mutable borrow occurs (line 6), and where the immutable borrow is still in use (line 8). The compiler is proving that the immutable reference `first` might be invalidated by the `push`. This is not a theoretical concern — this exact bug causes real crashes in C/C++ programs. Rust catches it before the code ever runs.
