---
id: into-iterator
phase: 9
phase_title: "Iterators & Zero-Cost Abstractions"
sequence: 3
title: "IntoIterator: iter() vs into_iter() vs iter_mut()"
hints:
  - into_iter() on a Vec<T> consumes the Vec and yields owned T values
  - After into_iter() consumes the Vec, the original variable is moved and cannot be used
  - Use iter() to borrow elements without consuming the collection
  - Use iter_mut() to mutably borrow elements in place
---

## Description

Rust provides three ways to iterate over a collection, each with different ownership semantics:

- **`.iter()`** borrows the collection and yields `&T` (shared references).
- **`.iter_mut()`** mutably borrows the collection and yields `&mut T` (mutable references).
- **`.into_iter()`** consumes the collection and yields owned `T` values.

The `for` loop uses `into_iter()` by default. This means `for x in vec` **moves** the vector -- you cannot use it afterward. Understanding which variant to use is essential for writing correct Rust code.

## Broken Code

```rust
fn main() {
    let names = vec![
        String::from("Alice"),
        String::from("Bob"),
        String::from("Charlie"),
    ];

    // into_iter() consumes the Vec, taking ownership of each String
    let uppercased: Vec<String> = names
        .into_iter()
        .map(|name| name.to_uppercase())
        .collect();

    println!("Uppercased: {:?}", uppercased);

    // Trying to use `names` again after it was consumed
    println!("Original names: {:?}", names);
    println!("Total names: {}", names.len());
}
```

## Correct Code

```rust
fn main() {
    let names = vec![
        String::from("Alice"),
        String::from("Bob"),
        String::from("Charlie"),
    ];

    // Use iter() to borrow the names, keeping ownership with the Vec
    let uppercased: Vec<String> = names
        .iter()
        .map(|name| name.to_uppercase())
        .collect();

    println!("Uppercased: {:?}", uppercased);

    // `names` is still valid because we only borrowed from it
    println!("Original names: {:?}", names);
    println!("Total names: {}", names.len());
}
```

## Explanation

The broken version calls `.into_iter()` on `names`. For a `Vec<T>`, `into_iter()` consumes the entire vector and yields each element as an owned `T`. After the `into_iter()` chain completes, the `names` variable has been moved -- its memory has been transferred into the iterator pipeline. Any subsequent use of `names` is a use-after-move error.

The correct version calls `.iter()` instead. This borrows the vector immutably, yielding `&String` references. The `map` closure receives `&String`, and since `to_uppercase()` is defined on `str` (which `&String` derefs to), it works without needing ownership. The original `names` vector remains intact and usable.

Here is a summary of the three iteration methods:

| Method        | Yields    | Collection after? | Use when...                          |
|---------------|-----------|-------------------|--------------------------------------|
| `.iter()`     | `&T`      | Still usable      | You need to read elements            |
| `.iter_mut()` | `&mut T`  | Still usable      | You need to modify elements in place |
| `.into_iter()`| `T`       | Consumed (moved)  | You need owned values or are done with the collection |

An important subtlety: when you write `for name in names`, Rust calls `names.into_iter()` implicitly. This is why a `for` loop over a `Vec` consumes it. If you want to borrow, write `for name in &names` (which calls `names.iter()`) or `for name in &mut names` (which calls `names.iter_mut()`).

Choose the iteration method based on what you need:
- If you need the original collection afterward, use `.iter()`.
- If you need to modify elements in place, use `.iter_mut()`.
- If you are done with the collection and need owned values (e.g., to move them into a new structure), use `.into_iter()`.

## Compiler Error Interpretation

```
error[E0382]: borrow of moved value: `names`
  --> src/main.rs:16:40
   |
2  |     let names = vec![
   |         ----- move occurs because `names` has type `Vec<String>`, which does not implement the `Copy` trait
...
9  |         .into_iter()
   |          ----------- `names` moved due to this method call
...
16 |     println!("Original names: {:?}", names);
   |                                      ^^^^^ value borrowed here after move
```

The compiler identifies exactly where the move happened (`.into_iter()`) and where the invalid use occurs (the `println!`). It also explains *why* the move happens: `Vec<String>` does not implement `Copy`, so `into_iter()` takes ownership by moving. The fix is to switch to `.iter()` if you need the collection afterward, or to restructure your code so that the `into_iter()` call is the last use of the variable.
