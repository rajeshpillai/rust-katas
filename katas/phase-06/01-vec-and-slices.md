---
id: vec-and-slices
phase: 6
phase_title: "Collections & the Owned/Borrowed Duality"
sequence: 1
title: "Vec<T> and Slices: Owned vs Borrowed Collections"
hints:
  - "Iterating over a `Vec` borrows it immutably. You cannot mutate a `Vec` while an immutable borrow exists."
  - "Pushing to a `Vec` requires a mutable borrow — this conflicts with the iterator's immutable borrow."
  - "Collect the changes you need to make first, then apply them after the iteration is complete."
---

## Description

`Vec<T>` is Rust's growable, heap-allocated array. It **owns** its data. A slice `&[T]` is a borrowed view into a contiguous sequence — it could be a view into a `Vec`, an array, or any contiguous memory.

The ownership and borrowing rules apply to collections just as they apply to individual values. When you iterate over a `Vec`, the iterator borrows the `Vec`. While that borrow is active, you cannot mutate the `Vec`. This prevents a common class of bugs: modifying a collection while iterating over it (which in many languages leads to iterator invalidation, crashes, or corrupted state).

## Broken Code

```rust
fn remove_negatives(numbers: &mut Vec<i32>) {
    for (i, &num) in numbers.iter().enumerate() {
        if num < 0 {
            // BUG: `numbers.iter()` borrows `numbers` immutably.
            // `numbers.remove(i)` borrows `numbers` mutably.
            // Both borrows are active simultaneously — forbidden.
            numbers.remove(i);
        }
    }
}

fn main() {
    let mut values = vec![3, -1, 4, -5, 9, -2, 6];
    remove_negatives(&mut values);
    println!("{:?}", values);
}
```

## Correct Code

```rust
fn remove_negatives(numbers: &mut Vec<i32>) {
    // `retain` is the idiomatic way to filter a Vec in place.
    // It iterates over the elements and keeps only those for
    // which the closure returns true. No borrow conflict.
    numbers.retain(|&num| num >= 0);
}

fn main() {
    let mut values = vec![3, -1, 4, -5, 9, -2, 6];
    remove_negatives(&mut values);
    println!("{:?}", values); // prints: [3, 4, 9, 6]
}
```

## Explanation

The broken version attempts to mutate a `Vec` while iterating over it. Here is why this is forbidden:

1. `numbers.iter()` creates an iterator that holds an **immutable borrow** of `numbers`. The iterator needs the `Vec` to remain stable — its internal pointer walks through the `Vec`'s buffer.

2. `numbers.remove(i)` requires a **mutable borrow** of `numbers`. It shifts all elements after index `i` to the left and decrements the length. This changes the `Vec`'s internal state.

3. Rust's borrowing rule: you cannot have a mutable borrow while any immutable borrows exist. The iterator's immutable borrow and `remove`'s mutable borrow are alive at the same time.

Beyond the borrow checker issue, this code would also be **logically wrong** even in a language that allowed it: after removing an element at index `i`, all subsequent elements shift left, so the index-based iteration would skip elements.

**The correct solution uses `retain`**, which is designed for exactly this purpose. Internally, `retain` handles the iteration and removal in a single pass without creating conflicting borrows.

**Other patterns for modifying collections during iteration:**

```rust
// Pattern 1: Collect indices first, then remove (in reverse order)
let indices_to_remove: Vec<usize> = numbers
    .iter()
    .enumerate()
    .filter(|(_, &num)| num < 0)
    .map(|(i, _)| i)
    .collect();

for i in indices_to_remove.into_iter().rev() {
    numbers.remove(i);
}

// Pattern 2: Build a new Vec (functional style)
let positives: Vec<i32> = numbers.iter().copied().filter(|&n| n >= 0).collect();
*numbers = positives;

// Pattern 3: Use drain_filter (nightly) or retain (stable) — preferred
numbers.retain(|&num| num >= 0);
```

**The owned/borrowed duality:**

| Type | Ownership | Growable | Use when |
|---|---|---|---|
| `Vec<T>` | Owned | Yes | You own the data, may need to resize |
| `&[T]` | Borrowed | No | You just need to read a sequence |
| `&mut [T]` | Mutably borrowed | No | You need to modify elements (but not add/remove) |

Functions that only read elements should accept `&[T]`, not `&Vec<T>`. This is more flexible because it accepts arrays, slices, and vectors alike.

## Compiler Error Interpretation

```
error[E0502]: cannot borrow `*numbers` as mutable because it is also borrowed as immutable
  --> src/main.rs:5:13
   |
2  |     for (i, &num) in numbers.iter().enumerate() {
   |                       --------------------------
   |                       |
   |                       immutable borrow occurs here
   |                       immutable borrow later used here
...
5  |             numbers.remove(i);
   |             ^^^^^^^^^^^^^^^^^ mutable borrow occurs here
```

This error captures the essence of Rust's aliasing rules:

- **"cannot borrow `*numbers` as mutable because it is also borrowed as immutable"** — Two borrows of the same data with conflicting mutability. Rust says no.
- **"immutable borrow occurs here"** on `numbers.iter().enumerate()` — The iterator holds the immutable borrow for the duration of the loop.
- **"mutable borrow occurs here"** on `numbers.remove(i)` — The `remove` call needs exclusive access.
- **"immutable borrow later used here"** — The iterator is still in use (the loop continues after the `remove` call), so the immutable borrow has not ended.

The compiler is preventing iterator invalidation at compile time. In C++, modifying a `std::vector` while iterating with iterators is undefined behavior. In Java, it throws `ConcurrentModificationException` at runtime. In Rust, it simply does not compile.
