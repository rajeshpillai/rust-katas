---
id: multiple-trait-bounds
phase: 4
phase_title: "Traits & Generics"
sequence: 7
title: Multiple Trait Bounds and Where Clauses
hints:
  - "Use `+` to require multiple traits: `T: Display + Debug` means T must implement both."
  - "When bounds get long, use a `where` clause after the function signature for readability."
  - "Each type parameter can have independent bounds: `fn f<T: Clone, U: Debug>(a: T, b: U)`."
---

## Description

Real-world generic functions often need types that satisfy multiple constraints. Rust uses `+` to combine trait bounds: `T: Display + Clone` means "T must implement both Display and Clone."

When bounds become complex, **where clauses** provide a cleaner syntax by moving bounds after the function signature.

## Broken Code

```rust
use std::fmt;

fn print_and_clone<T>(item: T) -> T {
    // Bug: T has no bounds — can't print or clone
    println!("Item: {}", item);
    item.clone()
}

fn main() {
    let s = String::from("hello");
    let copy = print_and_clone(s);
    println!("Clone: {}", copy);
}
```

## Correct Code

```rust
use std::fmt;

// Multiple bounds with +
fn print_and_clone<T: fmt::Display + Clone>(item: T) -> T {
    println!("Item: {}", item);
    item.clone()
}

// Where clause — same thing, cleaner for complex bounds
fn summarize<T, U>(a: &T, b: &U) -> String
where
    T: fmt::Display + fmt::Debug,
    U: fmt::Display + Clone,
{
    format!("a: {} (debug: {:?}), b: {}", a, a, b)
}

// Bounds on return type
fn make_pair<T: Default + fmt::Debug>() -> (T, T) {
    (T::default(), T::default())
}

// Complex example: merge two sorted slices
fn merge_sorted<T: PartialOrd + Clone>(a: &[T], b: &[T]) -> Vec<T> {
    let mut result = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] <= b[j] {
            result.push(a[i].clone());
            i += 1;
        } else {
            result.push(b[j].clone());
            j += 1;
        }
    }
    result.extend_from_slice(&a[i..]);
    result.extend_from_slice(&b[j..]);
    result
}

fn main() {
    // Multiple bounds: Display + Clone
    let result = print_and_clone(String::from("hello"));
    println!("Cloned: {}\n", result);

    // Where clause
    let summary = summarize(&42, &"world");
    println!("{}\n", summary);

    // Default + Debug
    let ints: (i32, i32) = make_pair();
    let strings: (String, String) = make_pair();
    let bools: (bool, bool) = make_pair();
    println!("Default ints: {:?}", ints);
    println!("Default strings: {:?}", strings);
    println!("Default bools: {:?}\n", bools);

    // PartialOrd + Clone: merge sorted slices
    let a = vec![1, 3, 5, 7];
    let b = vec![2, 4, 6, 8, 9, 10];
    let merged = merge_sorted(&a, &b);
    println!("Merge {:?} + {:?} = {:?}", a, b, merged);

    // Works with strings too!
    let words_a = vec!["apple", "cherry", "fig"];
    let words_b = vec!["banana", "date", "elderberry"];
    let merged_words = merge_sorted(&words_a, &words_b);
    println!("Merged words: {:?}", merged_words);
}
```

## Explanation

The broken code declares `fn print_and_clone<T>(item: T)` with no bounds on `T`. Without bounds, the compiler knows nothing about `T` — it could be any type. You cannot:
- Print it (needs `Display`)
- Clone it (needs `Clone`)
- Compare it (needs `PartialEq` or `PartialOrd`)

**Trait bounds are promises** to the compiler: "`T` will always implement these traits." The compiler then allows you to use those traits' methods.

**Syntax comparison:**

```rust
// Inline bounds (good for simple cases)
fn f<T: Display + Clone>(x: T) { ... }

// Where clause (good for complex cases)
fn f<T, U>(x: T, y: U) -> String
where
    T: Display + Debug,
    U: Display + Clone,
{ ... }
```

Both are identical in meaning. Where clauses are preferred when:
- There are multiple type parameters with bounds
- Bounds are long or complex
- You want the function name and parameters visible without scrolling

**Common trait combinations:**

| Bounds | Enables |
|---|---|
| `Display` | `println!("{}", x)` — user-facing output |
| `Debug` | `println!("{:?}", x)` — developer output |
| `Clone` | `x.clone()` — explicit copy |
| `PartialOrd` | `x < y`, `x > y` — comparison |
| `Default` | `T::default()` — zero/empty value |
| `Display + Debug` | Both print formats |
| `PartialOrd + Clone` | Compare and copy (sorting, merging) |

## ⚠️ Caution

- Adding too many bounds makes functions less reusable. Only require traits you actually use in the function body.
- `Clone` and `Copy` are different: `Copy` is implicit (bit-for-bit copy), `Clone` is explicit (may allocate). Most bounds should use `Clone` unless you specifically need `Copy` semantics.
- Bounds on the struct vs bounds on the impl: prefer bounds on the impl. This allows the struct to be used in contexts that don't need those bounds.

## 💡 Tips

- Use `where` clauses for anything beyond two simple bounds — your future self will thank you.
- `T: Default` lets you create "empty" values generically. `Vec::new()`, `String::new()`, `0i32`, `false` are all defaults.
- You can bound on traits from any crate, not just `std`. This is how libraries create extensible APIs.
- The `Debug + Display + Clone + PartialEq` combination is so common that many types derive all four.

## Compiler Error Interpretation

```
error[E0277]: `T` doesn't implement `std::fmt::Display`
 --> main.rs:3:28
  |
3 |     println!("Item: {}", item);
  |                           ^^^^ `T` cannot be formatted with the default formatter
  |
  = note: in format strings you may be able to use `{:?}` (or {:#?} for pretty-print) instead
help: consider restricting type parameter `T`
  |
1 | fn print_and_clone<T: std::fmt::Display>(item: T) -> T {
  |                     +++++++++++++++++++
```

The compiler says "T doesn't implement Display" and suggests adding the bound. This is the generic contract: if you want to use a trait's functionality, you must declare it as a bound.

---

| [Prev: Generic Structs and Enums](#/katas/generic-structs-and-enums) | [Next: Derived Traits — Debug, Clone, PartialEq, and More](#/katas/derived-traits) |
