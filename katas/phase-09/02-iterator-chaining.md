---
id: iterator-chaining
phase: 9
phase_title: "Iterators & Zero-Cost Abstractions"
sequence: 2
title: Iterator Chaining and Lazy Evaluation
hints:
  - Vec does not have a .map() method -- you need to call .iter() first to get an iterator
  - Iterator adaptors like map() and filter() are lazy -- they produce new iterators, not new collections
  - You must call .collect() (or another consumer) to actually drive the iteration and produce a result
---

## Description

Rust iterators are lazy. When you call `.map()` or `.filter()` on an iterator, no work happens immediately. Instead, these methods return a new iterator that will perform the transformation when consumed. This laziness is key to Rust's zero-cost abstraction philosophy: the compiler can fuse multiple iterator stages into a single optimized loop, often producing code identical to what you would write by hand with a `for` loop and manual indexing.

However, this laziness means you must explicitly consume the iterator to get results. The most common consumer is `.collect()`, which gathers values into a collection.

## Broken Code

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // Try to use .map() directly on a Vec
    let doubled = numbers.map(|x| x * 2);

    // Try to filter without collecting
    let evens = numbers.iter().filter(|&&x| x % 2 == 0);

    println!("Doubled: {:?}", doubled);
    println!("Evens: {:?}", evens);
}
```

## Correct Code

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // First call .iter() to get an iterator, then .map(), then .collect()
    let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();

    // Chain filter and collect to materialize the results
    let evens: Vec<&i32> = numbers.iter().filter(|&&x| x % 2 == 0).collect();

    println!("Doubled: {:?}", doubled);
    println!("Evens: {:?}", evens);
}
```

## Explanation

The broken version has two distinct problems:

**Problem 1: Calling `.map()` on a `Vec` directly.** `Vec<T>` does not implement the `map()` method. The `map()` method belongs to the `Iterator` trait. You must first convert the `Vec` into an iterator by calling `.iter()` (borrows each element), `.iter_mut()` (borrows each element mutably), or `.into_iter()` (takes ownership of each element). Only then can you chain iterator adaptors like `map`, `filter`, `take`, etc.

**Problem 2: Not collecting the `filter` result.** The expression `numbers.iter().filter(|&&x| x % 2 == 0)` does not produce a `Vec`. It produces a `Filter<Iter<'_, i32>>` -- a lazy iterator adaptor. No elements have been filtered yet. The `Filter` struct just records the closure and waits for someone to call `next()` on it. To get a `Vec` out of it, you must call `.collect()`.

This laziness is a feature, not a bug. Consider a chain like:

```rust
numbers.iter().map(|x| x * 2).filter(|x| *x > 5).take(3).collect::<Vec<_>>()
```

The compiler does not create three intermediate collections. It fuses all the operations into a single pass: for each element, it doubles, checks the condition, and if it passes, adds it to the result -- stopping after three matches. This is the "zero-cost abstraction" promise: you write high-level, composable code, and the compiler generates the same machine code as a hand-rolled loop.

Note the double reference `&&x` in the filter closure. When you call `.iter()` on a `Vec<i32>`, you get an iterator over `&i32`. The `filter` method then passes a reference to each item, so your closure receives `&&i32`. The pattern `&&x` destructures both layers of reference to give you a plain `i32` to work with.

## Compiler Error Interpretation

```
error[E0599]: no method named `map` found for struct `Vec<{integer}>` in the current scope
 --> src/main.rs:5:30
  |
5 |     let doubled = numbers.map(|x| x * 2);
  |                           --- ^^^ method not found in `Vec<{integer}>`
  |
  = note: the method was found for `Iterator`
```

This error is clear: `Vec` does not have a `map` method. The compiler even hints that `map` exists on `Iterator`. The fix is to call `.iter()` first to obtain an iterator, then call `.map()` on that iterator. When you see "no method named X found for struct Vec," check whether X is an iterator method and insert `.iter()` before the chain.
