---
id: hashset-and-btreemap
phase: 6
phase_title: "Collections & the Owned/Borrowed Duality"
sequence: 6
title: "HashSet, BTreeMap, and Ordered Collections"
hints:
  - "`HashSet<T>` stores unique values with O(1) lookup. Elements must implement `Hash + Eq`."
  - "`BTreeMap<K, V>` keeps keys sorted. Use it when you need ordered iteration or range queries."
  - "Set operations like union, intersection, and difference are built-in methods on `HashSet`."
---

## Description

Beyond `HashMap`, Rust provides several collection types:
- **`HashSet<T>`** — An unordered set of unique values (O(1) lookup)
- **`BTreeMap<K, V>`** — A sorted map (keys in order, O(log n) operations)
- **`BTreeSet<T>`** — A sorted set of unique values

Each has different performance characteristics and use cases. `HashSet` is fastest for membership checks; `BTreeMap`/`BTreeSet` maintain sorted order.

## Broken Code

```rust
use std::collections::HashSet;

fn main() {
    // Bug: f64 does not implement Hash or Eq — can't use in HashSet
    let mut scores: HashSet<f64> = HashSet::new();
    scores.insert(9.5);
    scores.insert(8.0);
    scores.insert(9.5); // Duplicate

    println!("Unique scores: {:?}", scores);
}
```

## Correct Code

```rust
use std::collections::{HashSet, BTreeMap, BTreeSet};

fn main() {
    // --- HashSet: unique values, unordered ---
    let mut languages: HashSet<&str> = HashSet::new();
    languages.insert("Rust");
    languages.insert("Go");
    languages.insert("Python");
    languages.insert("Rust"); // Duplicate — ignored

    println!("Languages: {:?}", languages);
    println!("Count: {}", languages.len());
    println!("Contains Rust? {}", languages.contains("Rust"));
    println!("Contains Java? {}\n", languages.contains("Java"));

    // Build from an iterator
    let words = vec!["hello", "world", "hello", "rust", "world"];
    let unique: HashSet<&str> = words.iter().copied().collect();
    println!("Words: {:?}", words);
    println!("Unique: {:?} ({} unique)\n", unique, unique.len());

    // --- Set operations ---
    let frontend: HashSet<&str> = ["JavaScript", "TypeScript", "CSS", "Rust"].into();
    let backend: HashSet<&str> = ["Rust", "Go", "Python", "Java"].into();

    // Union: all elements from both sets
    let all: HashSet<&&str> = frontend.union(&backend).collect();
    println!("Frontend: {:?}", frontend);
    println!("Backend:  {:?}", backend);
    println!("Union:        {:?}", all);

    // Intersection: elements in both
    let both: Vec<&&str> = frontend.intersection(&backend).collect();
    println!("Intersection: {:?}", both);

    // Difference: in frontend but not backend
    let only_frontend: Vec<&&str> = frontend.difference(&backend).collect();
    println!("Frontend only: {:?}", only_frontend);

    // Symmetric difference: in one but not both
    let exclusive: Vec<&&str> = frontend.symmetric_difference(&backend).collect();
    println!("Exclusive: {:?}\n", exclusive);

    // --- BTreeMap: sorted by key ---
    let mut scores: BTreeMap<&str, i32> = BTreeMap::new();
    scores.insert("Charlie", 85);
    scores.insert("Alice", 92);
    scores.insert("Eve", 78);
    scores.insert("Bob", 88);
    scores.insert("Diana", 95);

    // Iteration is in sorted key order!
    println!("Scores (sorted by name):");
    for (name, score) in &scores {
        println!("  {}: {}", name, score);
    }

    // Range queries — only BTreeMap supports this
    println!("\nNames C..=D:");
    for (name, score) in scores.range("C"..="D") {
        println!("  {}: {}", name, score);
    }

    // First and last (sorted)
    println!("\nFirst: {:?}", scores.iter().next());
    println!("Last:  {:?}", scores.iter().next_back());

    // --- BTreeSet: sorted unique values ---
    let mut numbers: BTreeSet<i32> = BTreeSet::new();
    numbers.insert(5);
    numbers.insert(2);
    numbers.insert(8);
    numbers.insert(1);
    numbers.insert(5); // Duplicate — ignored

    println!("\nSorted set: {:?}", numbers);
    // Range: elements >= 3 and <= 7
    let range: Vec<&i32> = numbers.range(3..=7).collect();
    println!("Range 3..=7: {:?}", range);

    // --- Choosing the right collection ---
    println!("\n--- Collection guide ---");
    println!("HashSet:  O(1) lookup, unordered, needs Hash+Eq");
    println!("BTreeSet: O(log n) lookup, sorted, needs Ord");
    println!("HashMap:  O(1) lookup by key, unordered, needs Hash+Eq");
    println!("BTreeMap: O(log n) by key, sorted, needs Ord, has range()");

    // --- Practical: find duplicates ---
    let data = vec![1, 3, 5, 3, 7, 1, 9, 5, 3];
    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();
    for &x in &data {
        if !seen.insert(x) { // insert returns false if already present
            duplicates.push(x);
        }
    }
    duplicates.sort();
    duplicates.dedup();
    println!("\nData: {:?}", data);
    println!("Duplicates: {:?}", duplicates);
}
```

## Explanation

The broken code tries to put `f64` values in a `HashSet`. This fails because `f64` does not implement `Hash` or `Eq` — requirements for any `HashSet` element. The reason: floating-point numbers have `NaN`, and `NaN != NaN`, which breaks the `Eq` contract (equality must be reflexive).

**Collection requirements:**

| Collection | Element/Key must implement |
|---|---|
| `HashSet<T>` | `Hash + Eq` |
| `HashMap<K, V>` | `Hash + Eq` (for K) |
| `BTreeSet<T>` | `Ord` |
| `BTreeMap<K, V>` | `Ord` (for K) |

**When to use which:**

| Need | Use |
|---|---|
| Fast membership check | `HashSet` |
| Unique values in sorted order | `BTreeSet` |
| Key-value with fast lookup | `HashMap` |
| Key-value with sorted keys | `BTreeMap` |
| Range queries (`range(a..b)`) | `BTreeMap` or `BTreeSet` |

**Set operations** (`union`, `intersection`, `difference`, `symmetric_difference`) are methods on `HashSet` that return iterators. They are efficient and compose well with Rust's iterator system.

**`BTreeMap::range`** is unique to sorted maps — it returns an iterator over a key range. This is impossible with `HashMap` because keys are unordered.

**`HashSet::insert` returns `bool`** — `true` if the value was newly inserted, `false` if it was already present. This is a convenient way to detect duplicates in a single pass.

## ⚠️ Caution

- `f64` and `f32` cannot be used as `HashSet` elements or `HashMap` keys because they don't implement `Hash` or `Eq`. Use integer types or a wrapper.
- `HashSet` iteration order is not deterministic — it may differ between runs. Use `BTreeSet` if you need consistent ordering.
- Set operations return iterators of references. Collect into a new set or vec as needed.

## 💡 Tips

- `[value1, value2, ...].into()` can create a `HashSet` from an array literal (using `From` trait).
- `HashSet::insert` returning `bool` is the fastest way to detect duplicates — no separate `contains` check needed.
- `BTreeMap` is ideal for implementing sorted indexes, leaderboards, and time-series data.
- For custom types in sets/maps, derive the required traits: `#[derive(Hash, Eq, PartialEq)]` for hash collections, `#[derive(Ord, PartialOrd, Eq, PartialEq)]` for BTree collections.

## Compiler Error Interpretation

```
error[E0277]: the trait bound `f64: Eq` is not satisfied
 --> main.rs:4:38
  |
4 |     let mut scores: HashSet<f64> = HashSet::new();
  |                                    ^^^^^^^^^^^ the trait `Eq` is not implemented for `f64`
  |
  = help: the following types implement `Eq`: i32, u32, String, ...
  = note: required by a bound in `HashSet::<T>::new`
```

The compiler says `f64` does not implement `Eq`, which is required by `HashSet`. This is because `NaN != NaN` in IEEE 754, violating the reflexivity requirement of `Eq`. Use integer types, strings, or a custom wrapper with defined NaN handling.

---

| [Prev: Multidimensional Collections — Grids and Matrices](#/katas/multidimensional-collections) | [Next: The Owned/Borrowed Duality: PathBuf/Path, OsString/OsStr](#/katas/owned-borrowed-duality) |
