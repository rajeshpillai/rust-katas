---
id: arrays-and-slices
phase: 0
phase_title: "Rust as a Language"
sequence: 10
title: Arrays and Slices
hints:
  - "An array's size is part of its type: `[i32; 3]` and `[i32; 5]` are different types."
  - "A function that accepts `[i32; 3]` cannot accept `[i32; 5]`. Use a slice `&[i32]` to accept any length."
  - "A slice `&[T]` is a reference to a contiguous sequence — it works with arrays of any size and with Vec."
---

## Description

An **array** in Rust is a fixed-size, stack-allocated collection where every element has the same type. The size is part of the type — `[i32; 3]` is a different type from `[i32; 5]`.

A **slice** (`&[T]`) is a reference to a contiguous sequence of elements. Slices let you write functions that work with arrays of any size, or with portions of arrays and vectors.

## Broken Code

```rust
fn sum(numbers: [i32; 3]) -> i32 {
    let mut total = 0;
    for n in &numbers {
        total += n;
    }
    total
}

fn main() {
    let a = [1, 2, 3];
    let b = [10, 20, 30, 40, 50];

    println!("sum(a) = {}", sum(a));
    println!("sum(b) = {}", sum(b)); // Bug: b is [i32; 5], not [i32; 3]!
}
```

## Correct Code

```rust
// Accept a slice — works with any array size
fn sum(numbers: &[i32]) -> i32 {
    let mut total = 0;
    for &n in numbers {
        total += n;
    }
    total
}

fn main() {
    // --- Arrays: fixed size, stack allocated ---
    let a: [i32; 5] = [1, 2, 3, 4, 5];
    println!("Array: {:?}", a);
    println!("Length: {}", a.len());
    println!("First: {}, Last: {}", a[0], a[a.len() - 1]);

    // Initialize all elements to the same value
    let zeros = [0i32; 10]; // Ten zeros
    println!("Zeros: {:?}", zeros);

    // --- Slices: references to contiguous data ---
    // A slice of the whole array
    let whole: &[i32] = &a;
    println!("Whole slice: {:?}", whole);

    // A slice of part of the array (index 1 to 3, exclusive)
    let middle: &[i32] = &a[1..4];
    println!("Middle [1..4]: {:?}", middle);

    // Inclusive range
    let inclusive: &[i32] = &a[1..=3];
    println!("Inclusive [1..=3]: {:?}", inclusive);

    // From start or to end
    let first_three = &a[..3];
    let last_two = &a[3..];
    println!("First 3: {:?}, Last 2: {:?}", first_three, last_two);

    // --- Functions with slices work for any size ---
    let small = [10, 20];
    let large = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    println!("\nsum([10, 20]) = {}", sum(&small));
    println!("sum([1..10]) = {}", sum(&large));
    println!("sum(first 3 of a) = {}", sum(&a[..3]));

    // --- Common array/slice methods ---
    let mut data = [5, 2, 8, 1, 9, 3];
    println!("\nUnsorted: {:?}", data);

    data.sort();
    println!("Sorted: {:?}", data);

    println!("Contains 8? {}", data.contains(&8));
    println!("Min: {:?}", data.iter().min());
    println!("Max: {:?}", data.iter().max());

    // Iterate with index
    for (i, &val) in data.iter().enumerate() {
        println!("  data[{}] = {}", i, val);
    }

    // --- Arrays are Copy if elements are Copy ---
    let original = [1, 2, 3];
    let copy = original; // Copies, not moves!
    println!("\nOriginal: {:?}", original); // Still accessible
    println!("Copy: {:?}", copy);
}
```

## Explanation

The broken code defines `sum` to accept `[i32; 3]` — an array of exactly 3 elements. When called with `b` (which is `[i32; 5]`), the compiler rejects it because `[i32; 3]` and `[i32; 5]` are **different types**.

This is unlike C, where array parameters decay to pointers. In Rust, the size is part of the type, enforced at compile time.

**The fix: use a slice `&[i32]`.** A slice is a "fat pointer" — it stores a pointer to the data and the length. It accepts any contiguous sequence of `i32`, regardless of the underlying array size.

```rust
fn sum(numbers: &[i32]) -> i32  // Works with &[i32; 3], &[i32; 5], &Vec<i32>, etc.
```

**Array vs Slice vs Vec:**

| Type | Size | Location | Growable? |
|---|---|---|---|
| `[T; N]` (array) | Fixed at compile time | Stack | No |
| `&[T]` (slice) | Known at runtime | Reference | No |
| `Vec<T>` (vector) | Dynamic | Heap | Yes |

**Key insight:** `&[T]` is the universal "sequence of T" parameter type. It works with:
- `&array` — a reference to a fixed-size array
- `&vec` — a reference to a vector
- `&array[1..4]` — a sub-slice of an array
- `&vec[..]` — a slice of the entire vector

This is the **owned/borrowed duality** preview: `Vec<T>` is owned, `&[T]` is borrowed.

## ⚠️ Caution

- Array index out of bounds panics at runtime: `a[10]` on a 5-element array crashes. Use `.get(10)` to get `Option<&T>` instead.
- Array size must be a compile-time constant. You cannot write `let a = [0; n]` where `n` is a variable — use `Vec` for runtime-sized collections.
- Slices cannot outlive the data they reference. This is enforced by lifetimes (Phase 3).

## 💡 Tips

- Use `[value; count]` to initialize: `[0u8; 1024]` creates a 1KB zero-filled buffer.
- `.iter()` on arrays/slices returns references. Use `for &x in slice` to get values directly.
- `.windows(n)` and `.chunks(n)` are powerful slice methods for processing data in groups.
- Prefer `&[T]` over `&Vec<T>` in function parameters — it's more general and avoids unnecessary coupling to `Vec`.

## Compiler Error Interpretation

```
error[E0308]: mismatched types
 --> main.rs:13:29
  |
13|     println!("sum(b) = {}", sum(b));
  |                             --- ^ expected `[i32; 3]`, found `[i32; 5]`
  |                             |
  |                             arguments to this function are incorrect
  |
  = note: expected array `[i32; 3]`
             found array `[i32; 5]`
```

The compiler treats `[i32; 3]` and `[i32; 5]` as completely different types. The fix is to generalize the parameter from a specific array type to a slice (`&[i32]`), which accepts any length.
