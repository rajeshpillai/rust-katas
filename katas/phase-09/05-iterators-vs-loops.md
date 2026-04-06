---
id: iterators-vs-loops
phase: 9
phase_title: "Iterators & Zero-Cost Abstractions"
sequence: 5
title: "Why Iterators Beat Loops"
hints:
  - "Iterator chains are lazy — no intermediate allocations between steps."
  - "The compiler fuses iterator chains into a single loop (loop fusion), often producing better code than hand-written loops."
  - "Iterators express intent (what you want) rather than mechanism (how to get it). This makes bugs harder to write."
---

## Description

Rust iterators are not just syntactic sugar — they are a **zero-cost abstraction** that often produces code as fast as or faster than hand-written loops. The compiler fuses chained iterator operations into a single pass, eliminates bounds checks where possible, and enables auto-vectorization.

Beyond performance, iterators reduce an entire class of bugs: off-by-one errors, index out of bounds, forgetting to increment, and accidental mutation of the loop variable.

## Broken Code

```rust
fn main() {
    let temperatures_f = vec![32.0, 72.0, 98.6, 212.0, -40.0, 451.0];

    // Manual loop: convert to Celsius, filter comfortable, find average.
    let mut sum = 0.0;
    let mut count = 0;

    // Bug 1: off-by-one — should be `< temperatures_f.len()`, not `<=`
    let mut i = 0;
    while i <= temperatures_f.len() {
        let celsius = (temperatures_f[i] - 32.0) * 5.0 / 9.0;
        if celsius >= 18.0 && celsius <= 28.0 {
            sum += celsius;
            count += 1;
        }
        i += 1;
    }

    // Bug 2: division by zero if no comfortable temperatures exist
    let average = sum / count as f64;
    println!("Average comfortable temp: {:.1}°C", average);

    // Bug 3: manual index tracking for parallel iteration
    let names = vec!["Alice", "Bob", "Carol"];
    let scores = vec![95, 87, 92];
    let mut j = 0;
    while j < names.len() {
        println!("{}: {}", names[j], scores[j]); // panics if scores is shorter
        j += 1;
    }
}
```

## Correct Code

```rust
fn main() {
    let temperatures_f = vec![32.0_f64, 72.0, 98.6, 212.0, -40.0, 451.0];

    // Iterator chain: convert, filter, average — one pass, no indexing.
    let comfortable: Vec<f64> = temperatures_f
        .iter()
        .map(|&f| (f - 32.0) * 5.0 / 9.0)   // Convert to Celsius
        .filter(|&c| (18.0..=28.0).contains(&c))  // Keep comfortable range
        .collect();

    // No off-by-one. No index out of bounds. No manual counter.
    let average = if comfortable.is_empty() {
        None
    } else {
        Some(comfortable.iter().sum::<f64>() / comfortable.len() as f64)
    };

    match average {
        Some(avg) => println!("Average comfortable temp: {:.1}°C", avg),
        None => println!("No comfortable temperatures found"),
    }

    // --- Parallel iteration: zip stops at the shorter iterator ---
    let names = vec!["Alice", "Bob", "Carol"];
    let scores = vec![95, 87, 92];

    // zip() safely pairs elements — no index, no out-of-bounds
    for (name, score) in names.iter().zip(scores.iter()) {
        println!("{}: {}", name, score);
    }

    // --- Fold: reduce a collection to a single value ---
    let numbers = vec![1, 2, 3, 4, 5];

    // Manual sum with loop:
    let mut manual_sum = 0;
    for &n in &numbers {
        manual_sum += n;
    }

    // Iterator fold — same result, intent is clearer:
    let fold_sum = numbers.iter().fold(0, |acc, &n| acc + n);

    // Even simpler — .sum() is a specialized fold:
    let iter_sum: i32 = numbers.iter().sum();

    println!("\nSums: manual={}, fold={}, sum={}", manual_sum, fold_sum, iter_sum);

    // --- Chaining: find the first prime above 100 ---
    let first_prime = (101..).find(|&n| is_prime(n));
    println!("First prime above 100: {:?}", first_prime);

    // --- enumerate: index + value without manual counter ---
    let fruits = vec!["apple", "banana", "cherry"];
    for (i, fruit) in fruits.iter().enumerate() {
        println!("  {}. {}", i + 1, fruit);
    }
}

fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    if n == 2 { return true; }
    if n % 2 == 0 { return false; }
    let mut i = 3;
    while i * i <= n {
        if n % i == 0 { return false; }
        i += 2;
    }
    true
}
```

## Explanation

The broken version has three categories of bugs that iterators eliminate:

**1. Off-by-one errors.** The condition `i <= temperatures_f.len()` should be `i < temperatures_f.len()`. With iterators, there is no index variable to get wrong. `.iter()` handles bounds automatically.

**2. Division by zero.** The manual loop computes `sum / count` without checking if `count` is zero. The iterator version uses `Option` to handle the empty case explicitly.

**3. Mismatched lengths.** The manual parallel loop assumes `names` and `scores` have the same length. If `scores` is shorter, it panics with an index out of bounds. `zip()` stops at the shorter iterator — no panic, no check needed.

**Why iterators are zero-cost:**

When you write:

```rust
temperatures_f.iter()
    .map(|&f| (f - 32.0) * 5.0 / 9.0)
    .filter(|&c| (18.0..=28.0).contains(&c))
    .collect::<Vec<f64>>()
```

The compiler does not create intermediate `Vec`s for the `map` and `filter` steps. Instead, it **fuses** the chain into a single loop equivalent to:

```rust
let mut result = Vec::new();
for &f in &temperatures_f {
    let c = (f - 32.0) * 5.0 / 9.0;
    if c >= 18.0 && c <= 28.0 {
        result.push(c);
    }
}
```

This fusion happens because iterators are **lazy** — each step produces values one at a time. `map` does not run until `filter` asks for a value, and `filter` does not run until `collect` asks. The entire chain becomes a single loop with no intermediate allocations.

**When to use which:**

| Pattern | Use iterator | Use loop |
|---|---|---|
| Transform + filter + collect | `iter().map().filter().collect()` | — |
| Accumulate a single value | `.fold()` or `.sum()` | — |
| Parallel iteration | `.zip()` | — |
| Early exit on condition | `.find()` or `.any()` | — |
| Complex state machine | — | `loop { match state { ... } }` |
| Mutable state across iterations | — | `for` or `while` with `&mut` |

Iterators shine for data pipelines. Loops are better when the iteration logic itself is complex or stateful.

## ⚠️ Caution

- Iterator chains are lazy. Without a consumer (`.collect()`, `.for_each()`, `.sum()`, `.count()`), the chain does nothing at all. `vec.iter().map(|x| x + 1);` is a no-op.
- Chaining too many operations can make code hard to read. If a chain exceeds 5-6 steps, consider breaking it into named intermediate steps.

## 💡 Tips

- Use `.enumerate()` instead of a manual index counter.
- Use `.zip()` instead of indexed parallel iteration.
- Use `.find()` and `.any()` for early exit — they short-circuit.
- Infinite ranges (`(0..)`) work with `.take()`, `.find()`, and other short-circuiting adaptors.
- Profile before assuming iterators are slower — they often generate identical or better machine code than hand-written loops due to bounds-check elimination and auto-vectorization.

## Compiler Error Interpretation

```
thread 'main' panicked at 'index out of bounds: the len is 6 but the index is 6'
```

This is a runtime panic, not a compile error — and that is the problem. Index-based loops can only fail at runtime. The compiler cannot prove that `i <= temperatures_f.len()` is wrong because it is a valid boolean expression. Iterator-based code avoids this class of bugs entirely because there is no index to get wrong.

The deeper lesson: **prefer compile-time safety (iterators, strong types) over runtime checking (bounds checks, assertions)**. When the compiler handles the bookkeeping, you eliminate entire bug categories rather than catching them one at a time.

---

| [Prev: collect() and the Turbofish](#/katas/collect-and-turbofish) | [Next: Modules and Visibility Rules](#/katas/module-visibility) |
