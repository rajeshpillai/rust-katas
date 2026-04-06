---
id: type-inference-and-annotations
phase: 0
phase_title: Rust as a Language
sequence: 6
title: Type Inference and Annotations
hints:
  - Rust can infer types from usage, but some operations are ambiguous
  - The turbofish syntax ::<Type> tells a generic function what type to produce
  - collect() needs to know what collection type to build
---

## Description

Rust has powerful type inference — most of the time, you do not need to write type annotations. But there are situations where the compiler cannot figure out the type on its own. The two most common cases are `collect()` (which can produce many different collection types) and `parse()` (which can parse into many different types). In these cases, you must give the compiler a hint.

## Broken Code

```rust
fn main() {
    let numbers = vec!["1", "2", "3", "4", "5"];

    let parsed = numbers
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();

    println!("Sum: {}", parsed.iter().sum::<i32>());
}
```

## Correct Code

```rust
fn main() {
    let numbers = vec!["1", "2", "3", "4", "5"];

    let parsed: Vec<i32> = numbers
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();

    println!("Sum: {}", parsed.iter().sum::<i32>());
}
```

## Explanation

In the broken code, the compiler faces two ambiguities at once. First, `parse()` is generic — it can parse a string into an `i32`, `f64`, `u8`, or any type that implements `FromStr`. Second, `collect()` is generic — it can build a `Vec<T>`, `HashSet<T>`, `String`, or any type that implements `FromIterator`. The compiler needs you to resolve at least one of these ambiguities.

There are three equivalent ways to fix this:

1. **Type annotation on the binding:** `let parsed: Vec<i32> = ...` — this is the most common and readable approach.
2. **Turbofish on collect:** `.collect::<Vec<i32>>()` — useful when you do not want to annotate the variable.
3. **Turbofish on parse:** `.parse::<i32>()` combined with `.collect::<Vec<_>>()` — the `_` tells the compiler to infer the element type from context.

The key insight is that Rust's type inference works both forward and backward. When you annotate `let parsed: Vec<i32>`, the compiler propagates that information backward through `collect()` and into `parse()`, resolving both ambiguities at once. This is why a single annotation is often enough.

Type inference is not a convenience feature — it is integral to how Rust works with generics. Understanding when it needs help will save you time and frustration.

## ⚠️ Caution

- **Do not over-annotate.** Adding types everywhere makes code harder to read and defeats the purpose of inference. Only annotate when the compiler asks or when it improves clarity.
- **`parse().unwrap()` hides two potential failures.** The parse can fail (wrong format) and the unwrap can panic. In production code, handle the `Result` from `parse()` explicitly.

## 💡 Tips

- Use `_` as a partial type hint: `Vec<_>` tells the compiler "I want a Vec, figure out the element type yourself." This works in many contexts.
- The turbofish `::<Type>` goes on the method, not the variable: `.collect::<Vec<i32>>()`, not `collect<Vec<i32>>()`.
- When the compiler says "type annotations needed," start by annotating the `let` binding — that usually resolves everything due to bidirectional inference.

## Compiler Error Interpretation

```
error[E0283]: type annotations needed
 --> main.rs:6:10
  |
6 |         .collect();
  |          ^^^^^^^ cannot infer type
  |
  = note: multiple `impl`s satisfying `_: FromIterator<_>` found
help: consider giving `parsed` an explicit type
  |
4 |     let parsed: Vec<_> = numbers
  |               ++++++++
```

Error E0283 means the compiler found multiple possible types that satisfy the trait bounds, and it cannot choose between them. The help message suggests adding a type annotation to `parsed`. Notice the compiler even suggests `Vec<_>` — using `_` for the element type, hoping it can infer that part from other context. In this case, you need `Vec<i32>` because `parse()` is also ambiguous.

---

| [Prev: Pattern Matching Exhaustiveness](#/katas/pattern-matching-exhaustiveness) | [Next: Functions — The Basic Unit of Code](#/katas/functions) |
