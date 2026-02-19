---
id: hashmap-entry-api
phase: 6
phase_title: "Collections & the Owned/Borrowed Duality"
sequence: 3
title: HashMap and the Entry API
hints:
  - "`get` borrows the HashMap immutably. `insert` borrows it mutably. You cannot do both at the same time."
  - "The Entry API (`entry().or_insert()`) performs the lookup and insert in a single operation, avoiding the double borrow."
  - "This is not just a convenience — it is the correct way to express 'insert if absent' without violating borrowing rules."
---

## Description

`HashMap<K, V>` is Rust's hash map implementation. A common pattern is "check if a key exists, and if not, insert a value." In many languages, you would write `if !map.contains(key) { map.insert(key, value); }`. In Rust, the naive approach creates conflicting borrows.

The **Entry API** solves this elegantly. `map.entry(key)` returns an `Entry` enum that represents either a vacant or occupied slot. You can then operate on it without creating conflicting borrows, because the lookup and mutation happen in a single, well-scoped borrow.

## Broken Code

```rust
use std::collections::HashMap;

fn count_words(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    for word in text.split_whitespace() {
        // First, we borrow `counts` immutably to look up the word.
        let current = counts.get(word);

        // Then we try to borrow `counts` mutably to insert/update.
        // But the immutable borrow from `get` is still alive
        // (because `current` holds a reference into the map).
        match current {
            Some(&count) => {
                counts.insert(word.to_string(), count + 1);
            }
            None => {
                counts.insert(word.to_string(), 1);
            }
        }
    }

    counts
}

fn main() {
    let text = "the cat sat on the mat the cat";
    let counts = count_words(text);
    println!("{:?}", counts);
}
```

## Correct Code

```rust
use std::collections::HashMap;

fn count_words(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    for word in text.split_whitespace() {
        // The Entry API performs lookup and insert in one operation.
        // `entry()` takes a mutable borrow of `counts` and returns
        // an Entry that we can operate on.
        // `or_insert(0)` inserts 0 if the key is absent, then
        // returns a mutable reference to the value.
        let count = counts.entry(word.to_string()).or_insert(0);
        *count += 1;
    }

    counts
}

fn main() {
    let text = "the cat sat on the mat the cat";
    let counts = count_words(text);
    println!("{:?}", counts);
    // Output: {"the": 3, "cat": 2, "sat": 1, "on": 1, "mat": 1}
}
```

## Explanation

The broken version has a classic borrow conflict:

1. `counts.get(word)` borrows `counts` immutably and returns `Option<&usize>` — a reference into the map's internal storage.
2. The `match` keeps this reference alive (via `current` and the `&count` binding).
3. `counts.insert(...)` borrows `counts` mutably.
4. Both borrows are alive at the same time: the immutable borrow from `get` and the mutable borrow from `insert`.

This is not just a pedantic compiler rule. If `insert` causes the HashMap to reallocate its internal storage (when the load factor is exceeded), the reference from `get` would point to freed memory. Rust prevents this at compile time.

**The Entry API resolves this by combining the lookup and mutation into a single operation:**

```rust
counts.entry(key)     // Takes &mut counts, does the hash lookup
    .or_insert(0)     // If vacant, inserts 0. Returns &mut V.
```

The `entry()` method returns an `Entry<K, V>` enum:

```rust
enum Entry<'a, K, V> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}
```

This enum holds a mutable borrow to the map's internals. You then call methods on it to operate on the entry:

| Method | Behavior |
|---|---|
| `.or_insert(default)` | Insert `default` if vacant, return `&mut V` |
| `.or_insert_with(f)` | Insert `f()` if vacant (lazy), return `&mut V` |
| `.or_default()` | Insert `V::default()` if vacant, return `&mut V` |
| `.and_modify(f)` | Apply `f` to the value if occupied |

**Advanced pattern — `and_modify` with `or_insert`:**

```rust
counts
    .entry(word.to_string())
    .and_modify(|count| *count += 1)  // If exists, increment
    .or_insert(1);                     // If absent, start at 1
```

**The Entry API is not just syntactic sugar.** It is a fundamental pattern for working with mutable collections in Rust. It encapsulates the "check-then-act" pattern into a single borrow scope, making it both safe and efficient (the hash is computed only once).

## Compiler Error Interpretation

```
error[E0502]: cannot borrow `counts` as mutable because it is also borrowed as immutable
  --> src/main.rs:12:17
   |
8  |         let current = counts.get(word);
   |                       ------ immutable borrow occurs here
...
12 |                 counts.insert(word.to_string(), count + 1);
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
...
14 |             None => {
   |             ---- immutable borrow later used here
```

The compiler error reveals the exact conflict:

- **"cannot borrow `counts` as mutable because it is also borrowed as immutable"** — The fundamental violation: aliasing + mutation.
- **"immutable borrow occurs here"** on `counts.get(word)` — The `get` call borrows the map to look up the key and return a reference to the value.
- **"mutable borrow occurs here"** on `counts.insert(...)` — The `insert` call needs exclusive access to modify the map.
- **"immutable borrow later used here"** on the `None` branch — The `match` expression keeps the result of `get` alive through all branches, extending the immutable borrow.

This error teaches a deeper lesson: in Rust, you must design your data access patterns so that reads and writes do not overlap. The Entry API is Rust's answer to the "read-then-write" pattern for hash maps. It is a design-level solution, not a workaround.
