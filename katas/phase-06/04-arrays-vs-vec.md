---
id: arrays-vs-vec
phase: 6
phase_title: "Collections & the Owned/Borrowed Duality"
sequence: 4
title: "Fixed Arrays vs Vec — Stack vs Heap"
hints:
  - "Arrays `[T; N]` are fixed-size and live on the stack. Vec<T> is growable and lives on the heap."
  - "Use arrays when the size is known at compile time and won't change. Use Vec when you need to grow or shrink."
  - "Both can be borrowed as `&[T]` — write functions that take `&[T]` to accept either."
---

## Description

Rust has two primary sequential collections: fixed-size **arrays** (`[T; N]`) on the stack, and growable **vectors** (`Vec<T>`) on the heap. Choosing between them is a fundamental design decision that affects performance, memory layout, and API flexibility.

Both can be borrowed as a **slice** (`&[T]`), which is the universal read-only view into sequential data.

## Broken Code

```rust
fn main() {
    // Bug: trying to push onto a fixed-size array
    let mut data = [1, 2, 3, 4, 5];
    data.push(6); // Arrays can't grow!

    println!("{:?}", data);
}
```

## Correct Code

```rust
fn average(data: &[f64]) -> f64 {
    if data.is_empty() { return 0.0; }
    let sum: f64 = data.iter().sum();
    sum / data.len() as f64
}

fn main() {
    // --- Array: fixed size, stack allocated ---
    let temperatures: [f64; 5] = [20.1, 22.5, 19.8, 23.2, 21.0];
    println!("Array (stack): {:?}", temperatures);
    println!("Size: {} elements, known at compile time", temperatures.len());
    println!("Average: {:.1}\n", average(&temperatures));

    // Arrays are Copy if elements are Copy
    let copy = temperatures;
    println!("Original still accessible: {:?}\n", temperatures);
    let _ = copy;

    // --- Vec: growable, heap allocated ---
    let mut readings: Vec<f64> = Vec::new();
    readings.push(20.1);
    readings.push(22.5);
    readings.push(19.8);
    readings.extend_from_slice(&[23.2, 21.0, 24.5]);
    println!("Vec (heap): {:?}", readings);
    println!("Size: {}, Capacity: {}", readings.len(), readings.capacity());
    println!("Average: {:.1}\n", average(&readings));

    // Vec operations
    readings.push(25.0);          // Append
    readings.insert(0, 18.5);     // Insert at position
    readings.remove(3);           // Remove at index
    readings.retain(|&x| x > 20.0); // Keep only values > 20
    println!("After modifications: {:?}", readings);

    // --- Both coerce to &[T] ---
    let arr = [1.0, 2.0, 3.0];
    let vec = vec![4.0, 5.0, 6.0];

    println!("\naverage(array): {:.1}", average(&arr));
    println!("average(vec):   {:.1}", average(&vec));
    println!("average(slice): {:.1}", average(&vec[1..3]));

    // --- When to use which ---
    // Array: known size, performance critical, stack allocation
    let identity_matrix: [[f64; 3]; 3] = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    println!("\nIdentity matrix:");
    for row in &identity_matrix {
        println!("  {:?}", row);
    }

    // Vec: dynamic size, building incrementally
    let mut fibonacci: Vec<u64> = vec![0, 1];
    for _ in 2..20 {
        let next = fibonacci[fibonacci.len() - 1] + fibonacci[fibonacci.len() - 2];
        fibonacci.push(next);
    }
    println!("\nFirst 20 Fibonacci: {:?}", fibonacci);

    // --- Conversion ---
    let arr = [10, 20, 30];
    let vec: Vec<i32> = arr.to_vec();      // Array -> Vec (copies)
    let slice: &[i32] = &vec;              // Vec -> slice (borrows)
    println!("\nArray: {:?}", arr);
    println!("Vec:   {:?}", vec);
    println!("Slice: {:?}", slice);

    // --- Performance comparison ---
    println!("\n--- Array vs Vec tradeoffs ---");
    println!("Array [T; N]: stack, no allocation, Copy, fixed size");
    println!("Vec<T>:       heap, allocates, growable, Clone not Copy");
    println!("&[T]:         universal read-only view into both");
}
```

## Explanation

The broken code calls `.push()` on an array. Arrays are fixed-size — `[i32; 5]` is always exactly 5 elements. There is no `.push()` method because the size cannot change. Use `Vec<T>` for growable collections.

**Comparison:**

| | Array `[T; N]` | Vec `<T>` | Slice `&[T]` |
|---|---|---|---|
| Size | Fixed (compile time) | Dynamic (runtime) | Dynamic (runtime) |
| Memory | Stack | Heap | Reference |
| Growable | No | Yes | No |
| Copy | Yes (if T: Copy) | No (must Clone) | N/A (it's a reference) |
| As function param | Rare | `&Vec<T>` or `&[T]` | Preferred |

**The slice `&[T]` is the bridge.** Both arrays and vectors coerce to `&[T]`. Writing functions that take `&[T]` makes them work with all sequential data:

```rust
fn average(data: &[f64]) -> f64  // Works with arrays, vecs, and sub-slices
```

**When to use arrays:**
- Buffer with known compile-time size (`[u8; 1024]`)
- Small, fixed data (RGB colors, matrix rows)
- Performance-critical code (no heap allocation)

**When to use Vec:**
- Building a collection incrementally
- Size depends on runtime input
- Need to grow/shrink

## ⚠️ Caution

- Arrays are `Copy` only if their elements are `Copy` AND the size is small enough. Very large arrays on the stack can cause stack overflow.
- `Vec::push` may reallocate. If you know the final size, use `Vec::with_capacity(n)` to avoid repeated allocations.
- `&Vec<T>` in function signatures is an anti-pattern. Use `&[T]` instead — it's more flexible and idiomatic.

## 💡 Tips

- `vec![value; count]` creates a Vec with `count` copies of `value`. `vec![0; 100]` is 100 zeros.
- `.to_vec()` converts a slice to a Vec (clones the data). `&vec` or `&vec[..]` borrows as a slice.
- `.windows(n)` gives overlapping sub-slices of length `n`. `.chunks(n)` gives non-overlapping chunks.
- For read-only function parameters, always prefer `&[T]` over `&Vec<T>` or `[T; N]`.

## Compiler Error Interpretation

```
error[E0599]: no method named `push` found for array `[{integer}; 5]` in the current scope
 --> main.rs:3:10
  |
3 |     data.push(6);
  |          ^^^^ method not found in `[{integer}; 5]`
  |
  = note: `push` is a method on `Vec`, not arrays. Arrays have a fixed size.
```

The compiler clearly states: `push` exists on `Vec`, not on arrays. Arrays have a fixed size determined at compile time. If you need to add elements, use `Vec<T>` instead.
