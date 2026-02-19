---
id: borrow-scope-nll
phase: 2
phase_title: Borrowing
sequence: 4
title: Borrow Scopes and Non-Lexical Lifetimes
hints:
  - Borrows end at their last use, not at the end of the enclosing block
  - This feature is called Non-Lexical Lifetimes (NLL)
  - If a reference is not used after a certain point, the borrow ends there
---

## Description

Rust's borrow checker uses Non-Lexical Lifetimes (NLL) — a borrow ends at the point of its last use, not at the end of the lexical scope. This means code that looks like it has overlapping borrows might actually be fine, because the compiler is smart enough to see when a borrow is no longer needed. Understanding NLL helps you write natural code without unnecessary workarounds.

## Broken Code

```rust
fn main() {
    let mut data = String::from("hello");

    let r1 = &data;
    let r2 = &data;

    println!("{} and {}", r1, r2);

    let r3 = &mut data;
    r3.push_str(" world");

    println!("{} and {}", r1, r2);
    println!("{}", r3);
}
```

## Correct Code

```rust
fn main() {
    let mut data = String::from("hello");

    let r1 = &data;
    let r2 = &data;

    println!("{} and {}", r1, r2);
    // r1 and r2 are no longer used after this point.
    // Their borrows end here thanks to NLL.

    let r3 = &mut data;
    r3.push_str(" world");

    println!("{}", r3);
}
```

## Explanation

The broken code creates two immutable references (`r1` and `r2`), then a mutable reference (`r3`), and then tries to use the immutable references again after the mutable borrow. This fails because the immutable borrows are still alive (they are used on the last `println!`), so they overlap with the mutable borrow.

The correct code removes the second use of `r1` and `r2` (the last `println!`). With NLL, the immutable borrows end at their last use point — the first `println!` on line 7. By the time `r3` is created on line 11, the immutable borrows are already finished.

This is what Non-Lexical Lifetimes means in practice:

**Before NLL (Rust 2015 edition):** A borrow lasted until the end of the enclosing `{}` block. Even if you never used a reference again, its borrow occupied the entire scope. This forced programmers to use extra blocks `{ }` to manually shorten borrows.

**After NLL (Rust 2018+ edition):** A borrow lasts only until the last point where the reference is used. The compiler computes the minimal lifetime for each borrow, allowing more code to compile naturally.

In the correct code, the timeline looks like this:
- Lines 4-7: `r1` and `r2` are alive (immutable borrows active)
- Line 7: `r1` and `r2` are used for the last time (immutable borrows end)
- Lines 11-14: `r3` is alive (mutable borrow active)

There is no overlap, so the code compiles. The key insight: **borrows are about time, not syntax.** The compiler reasons about when references are actually used, not where they are declared.

## Compiler Error Interpretation

```
error[E0502]: cannot borrow `data` as mutable because it is also borrowed as immutable
  --> main.rs:9:14
   |
4  |     let r1 = &data;
   |              ----- immutable borrow occurs here
...
9  |     let r3 = &mut data;
   |              ^^^^^^^^^ mutable borrow occurs here
...
12 |     println!("{} and {}", r1, r2);
   |                           -- immutable borrow later used here
```

The compiler highlights line 12 as the critical point — the immutable borrow is "later used here." If line 12 did not exist, the immutable borrows would end at line 7, and the code would compile. This is the NLL system at work: the compiler calculates the actual last-use point of each reference and checks for conflicts. The error message tells you exactly which use is keeping the borrow alive, so you know what to move or remove to resolve the conflict.
