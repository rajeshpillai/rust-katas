---
id: integration-and-doc-tests
phase: 10
phase_title: "Modules, Visibility & Testing"
sequence: 5
title: "Integration Tests and Doc Tests"
hints:
  - "Integration tests live in the `tests/` directory at the crate root — they are external to your crate and can only test the public API."
  - "Doc tests are code examples embedded in `///` comments. They are compiled and run by `cargo test`."
  - "Unit tests (inside `#[cfg(test)]`) can access private items. Integration and doc tests cannot."
---

## Description

Rust has three kinds of tests, each with a different scope:

1. **Unit tests** (`#[cfg(test)]` inside the module) — test internal logic, can access private items.
2. **Integration tests** (`tests/` directory) — test the public API as an external consumer would. They `use your_crate;` and can only call public functions.
3. **Doc tests** (`///` comments with code blocks) — executable examples in documentation. They serve double duty: they document usage AND verify it compiles and runs correctly.

This kata focuses on integration tests and doc tests — the two testing forms that ensure your public API works as documented.

## Broken Code

```rust
// src/lib.rs (or imagine this as a crate's public API)

/// Adds two numbers together.
///
/// ```
/// let result = add(2, 3)  // Missing semicolon
/// assert_eq!(result, 5)   // Missing semicolons, missing crate path
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Divides two numbers, returning None if the divisor is zero.
///
/// # Examples
///
/// // Not in a code block — this is just a comment, not a test!
/// let result = safe_divide(10, 3);
/// assert_eq!(result, Some(3));
pub fn safe_divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

// This function is private — integration tests cannot call it.
fn internal_validate(n: i32) -> bool {
    n > 0 && n < 1000
}

fn main() {
    println!("{}", add(2, 3));
}
```

## Correct Code

```rust
/// Adds two numbers together.
///
/// # Examples
///
/// ```
/// let result = kata::add(2, 3);
/// assert_eq!(result, 5);
///
/// // Negative numbers work too
/// assert_eq!(kata::add(-1, 1), 0);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Divides two numbers, returning `None` if the divisor is zero.
///
/// # Examples
///
/// ```
/// // Normal division
/// let result = kata::safe_divide(10, 3);
/// assert_eq!(result, Some(3));
///
/// // Division by zero returns None
/// assert_eq!(kata::safe_divide(10, 0), None);
/// ```
///
/// # Panics
///
/// This function does not panic.
pub fn safe_divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

/// Clamps a value to a range.
///
/// ```
/// assert_eq!(kata::clamp(5, 0, 10), 5);   // In range
/// assert_eq!(kata::clamp(-3, 0, 10), 0);  // Below minimum
/// assert_eq!(kata::clamp(15, 0, 10), 10); // Above maximum
/// ```
pub fn clamp(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

// Private function — unit tests (below) can test it, but integration
// tests and doc tests cannot.
fn internal_validate(n: i32) -> bool {
    n > 0 && n < 1000
}

fn main() {
    println!("add(2, 3) = {}", add(2, 3));
    println!("safe_divide(10, 3) = {:?}", safe_divide(10, 3));
    println!("safe_divide(10, 0) = {:?}", safe_divide(10, 0));
    println!("clamp(15, 0, 10) = {}", clamp(15, 0, 10));
}

// --- Unit tests: can access private functions ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_validate() {
        // Unit tests CAN call private functions
        assert!(internal_validate(42));
        assert!(!internal_validate(0));
        assert!(!internal_validate(1001));
    }

    #[test]
    fn test_add_basics() {
        assert_eq!(add(0, 0), 0);
        assert_eq!(add(i32::MAX, 0), i32::MAX);
    }
}

// --- Integration test (would normally be in tests/api_test.rs): ---
// ```
// // tests/api_test.rs
// use kata::{add, safe_divide, clamp};
//
// #[test]
// fn test_public_api_workflow() {
//     let sum = add(10, 20);
//     let divided = safe_divide(sum, 3).unwrap();
//     let clamped = clamp(divided, 0, 5);
//     assert_eq!(clamped, 5); // 30 / 3 = 10, clamped to max of 5
// }
//
// #[test]
// fn test_divide_by_zero_is_none() {
//     assert_eq!(safe_divide(42, 0), None);
// }
// ```
```

## Explanation

The broken version has three distinct documentation/test issues:

**1. Doc test syntax errors.** The code block under `add` is missing semicolons and does not qualify the function with the crate name. Doc tests run as if they are separate crates — they must `use` or fully qualify your public items.

**2. Missing code fences.** The example under `safe_divide` is not inside triple backticks (` ``` `). Without code fences, `cargo test` treats it as plain text, not executable code. The "test" silently does not exist.

**3. Integration tests vs unit tests.** The private function `internal_validate` cannot be tested from an integration test. Integration tests are external — they can only access `pub` items.

**The three test scopes:**

| Kind | Location | Accesses | Runs when |
|---|---|---|---|
| Unit | `#[cfg(test)]` in source | Public + private | `cargo test` |
| Integration | `tests/*.rs` | Public only | `cargo test` |
| Doc | `///` code blocks | Public only | `cargo test --doc` |

**Doc test rules:**

1. Code must be inside ` ``` ` fences within `///` comments
2. Functions must be qualified with the crate name (e.g., `kata::add(2, 3)`)
3. Lines starting with `#` are hidden from docs but still compiled
4. Add `no_run` to compile but not execute: ` ```no_run `
5. Add `ignore` to skip entirely: ` ```ignore `
6. Add `should_panic` to expect a panic: ` ```should_panic `

**Why doc tests matter:**

Documentation rots. Code examples in comments become wrong as APIs evolve. Doc tests solve this: if you change `add` to return `Result<i32, Error>`, the doc test fails, and you are forced to update the example. **Your documentation is always correct because it is tested.**

**Integration test structure:**

```
my_crate/
├── src/
│   └── lib.rs
├── tests/
│   ├── api_test.rs      # Each file is a separate test crate
│   └── workflow_test.rs
└── Cargo.toml
```

Each file in `tests/` is compiled as its own crate that depends on your library. This means integration tests see your crate exactly as external users do — only `pub` items are visible.

## ⚠️ Caution

- Doc tests are slower to compile than unit tests because each code block is compiled as a separate program. For large crates, `cargo test --lib` runs only unit tests (faster iteration).
- Integration tests only work for library crates (`lib.rs`). Binary crates (`main.rs` only) cannot have integration tests — extract shared logic into a library.

## 💡 Tips

- Use `# ` prefix in doc tests to hide boilerplate (imports, setup) that would clutter the documentation but is needed for compilation.
- Run only doc tests with `cargo test --doc`.
- Run a specific integration test file with `cargo test --test api_test`.
- Every public function should have at least one doc test — it is both documentation and a regression test.

## Compiler Error Interpretation

If a doc test fails to compile:

```
---- kata::add (line 4) - compile fail ----
error[E0425]: cannot find function `add` in this scope
 --> kata::add (line 4):1:14
  |
1 | let result = add(2, 3);
  |              ^^^ not found in this scope
  |
help: consider importing this function
  |
1 + use kata::add;
  |
```

Doc tests are compiled as external crates. The function `add` is not in scope — you must either `use kata::add;` or write `kata::add(2, 3)`. This is a feature: it forces your doc examples to show the import path that real users need, making the documentation practically useful.

If an integration test tries to access a private function:

```
error[E0603]: function `internal_validate` is private
 --> tests/api_test.rs:5:10
  |
5 | kata::internal_validate(42);
  |       ^^^^^^^^^^^^^^^^^^^ private function
```

This is correct behavior. Integration tests exist to verify the public contract. If you need to test internal logic, use unit tests (`#[cfg(test)]` inside the module). The visibility boundary is enforced at compile time.

---

| [Prev: Test-Driven Development](#/katas/test-driven-kata) | [Next: Box and Heap Allocation](#/katas/box-heap-allocation) |
