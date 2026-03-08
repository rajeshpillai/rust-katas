---
id: option-and-result
phase: 0
phase_title: Rust as a Language
sequence: 3
title: Option and Result
hints:
  - Rust has no null — absence is represented by Option
  - You cannot use an Option value directly without handling the None case
  - Pattern matching forces you to handle all variants
---

## Description

Rust has no null. Instead, it uses `Option<T>` to represent values that might be absent, and `Result<T, E>` to represent operations that might fail. You must explicitly handle both cases — the compiler will not let you ignore the possibility of absence or failure.

## Broken Code

```rust
fn find_first_even(numbers: &[i32]) -> i32 {
    let result = numbers.iter().find(|&&n| n % 2 == 0);
    *result
}

fn main() {
    let nums = vec![1, 3, 4, 7, 8];
    println!("First even: {}", find_first_even(&nums));
}
```

## Correct Code

```rust
fn find_first_even(numbers: &[i32]) -> Option<i32> {
    numbers.iter().find(|&&n| n % 2 == 0).copied()
}

fn main() {
    let nums = vec![1, 3, 4, 7, 8];
    match find_first_even(&nums) {
        Some(n) => println!("First even: {}", n),
        None => println!("No even number found"),
    }
}
```

## Explanation

The `find` method returns `Option<&T>`, not `T`. This is Rust telling you: "this search might not find anything." You cannot dereference an `Option` directly — that would bypass the safety that `Option` provides.

The correct approach is to propagate the `Option` through your return type and let the caller decide what to do when the value is absent. Pattern matching with `match` forces you to handle both `Some` and `None` variants explicitly.

This is a core Rust principle: **acknowledge absence explicitly**. There is no null pointer, no undefined, no silent failure. If a value might not exist, the type system encodes that fact.

## ⚠️ Caution

- **`unwrap()` will panic at runtime if the value is `None`.** Never use `unwrap()` in production code unless you can mathematically prove the value is always `Some`. Prefer `match`, `if let`, or the `?` operator.
- **`Option` and `Result` are different types.** You cannot use `?` on an `Option` inside a function that returns `Result` (or vice versa) without converting first. Use `.ok_or()` to convert `Option` to `Result`.

## 💡 Tips

- Use `if let Some(value) = option` when you only care about the `Some` case and want to ignore `None`.
- Chain combinators like `.map()`, `.and_then()`, and `.unwrap_or()` for concise handling: `find_user(id).map(|u| u.name).unwrap_or("unknown".to_string())`.
- The `?` operator is the idiomatic way to propagate errors — you will learn it in Phase 8.

## Compiler Error Interpretation

```
error[E0614]: type `Option<&&i32>` cannot be dereferenced
 --> main.rs:3:5
  |
3 |     *result
  |     ^^^^^^^
```

The compiler tells you that `result` is an `Option`, not a reference. You cannot dereference it with `*` because `Option` is not a pointer type — it is an enum that might be `None`. You must pattern match or use methods like `unwrap()`, `map()`, or `?` to extract the inner value.
