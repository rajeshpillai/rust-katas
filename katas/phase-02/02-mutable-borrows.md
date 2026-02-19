---
id: mutable-borrows
phase: 2
phase_title: Borrowing
sequence: 2
title: Mutable Borrows
hints:
  - Only one mutable reference to a value can exist at a time
  - This rule prevents data races at compile time
  - Two mutable references can exist if their usage does not overlap
---

## Description

Rust enforces a strict rule: you can have either one mutable reference or any number of immutable references to a value, but never both at the same time. This single rule prevents data races, iterator invalidation, and a whole class of aliasing bugs — all at compile time.

## Broken Code

```rust
fn main() {
    let mut scores = vec![10, 20, 30];

    let first = &mut scores;
    let second = &mut scores;

    first.push(40);
    second.push(50);

    println!("{:?}", scores);
}
```

## Correct Code

```rust
fn main() {
    let mut scores = vec![10, 20, 30];

    {
        let first = &mut scores;
        first.push(40);
    }

    {
        let second = &mut scores;
        second.push(50);
    }

    println!("{:?}", scores);
}
```

## Explanation

The broken code creates two mutable references to `scores` simultaneously. This is exactly what Rust forbids. If two mutable references existed at the same time, both could modify the data independently, leading to unpredictable behavior — one reference might be reading while the other is writing, or both might be writing to the same memory.

The rule is deceptively simple: **at most one `&mut T` at a time.** This is not a limitation of the borrow checker — it is a fundamental invariant that eliminates data races.

The correct version uses separate scopes so the two mutable borrows never overlap. When `first` goes out of scope at the closing `}`, the mutable borrow ends. Only then can `second` take a new mutable borrow. The borrows are sequential, not simultaneous.

You might wonder: "But this is all single-threaded code. Why does it matter?" The answer is that the same aliasing bugs that cause data races in concurrent code also cause bugs in single-threaded code. Consider a simpler example: if you hold a mutable reference to a `Vec` and also an index into that `Vec`, pushing a new element might reallocate the underlying buffer, invalidating the index. Rust's rule prevents this entire category of bugs, regardless of threading.

## Compiler Error Interpretation

```
error[E0499]: cannot borrow `scores` as mutable more than once at a time
 --> main.rs:5:18
  |
4 |     let first = &mut scores;
  |                 ----------- first mutable borrow occurs here
5 |     let second = &mut scores;
  |                  ^^^^^^^^^^^ second mutable borrow occurs here
6 |
7 |     first.push(40);
  |     ----- first borrow later used here
```

Error E0499 is the canonical "two mutable borrows" error. The compiler shows three things: where the first borrow was created, where the second (conflicting) borrow was created, and where the first borrow is still in use. That last part is crucial — if `first` were never used after `second` was created, Rust's non-lexical lifetimes would allow the code. The conflict exists because `first` is used on line 7, which means its borrow is still active when `second` is created on line 5.
