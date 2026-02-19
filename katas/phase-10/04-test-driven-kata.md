---
id: test-driven-kata
phase: 10
phase_title: "Modules, Visibility & Testing"
sequence: 4
title: "Test-Driven Development"
hints:
  - "todo!() compiles but panics at runtime -- it is a placeholder for unfinished code"
  - The tests define the contract; your implementation must satisfy all of them
  - Start with the simplest failing test and work your way to the more complex ones
  - "unimplemented!() is a synonym for todo!() but todo!() is preferred in modern Rust"
---

## Description

Test-Driven Development (TDD) is a discipline where you write tests before writing the implementation. In Rust, `todo!()` is a macro that compiles successfully but panics at runtime with the message "not yet implemented." This lets you define your function signatures and write tests against them, then run the tests to confirm they fail, and finally implement the function to make them pass.

This kata demonstrates the TDD cycle: tests are already written, but the function body is `todo!()`. Your job is to replace the placeholder with a real implementation.

## Broken Code

```rust
/// Parses a comma-separated string of integers and returns the sum.
/// Ignores any entries that are not valid integers.
///
/// Examples:
///   "1,2,3" -> 6
///   "10, 20, 30" -> 60 (handles spaces)
///   "1,abc,3" -> 4 (ignores invalid entries)
///   "" -> 0
fn sum_csv(input: &str) -> i64 {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sum() {
        assert_eq!(sum_csv("1,2,3"), 6);
    }

    #[test]
    fn test_with_spaces() {
        assert_eq!(sum_csv("10, 20, 30"), 60);
    }

    #[test]
    fn test_ignores_invalid() {
        assert_eq!(sum_csv("1,abc,3"), 4);
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(sum_csv(""), 0);
    }

    #[test]
    fn test_single_value() {
        assert_eq!(sum_csv("42"), 42);
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(sum_csv("-1,2,-3"), -2);
    }

    #[test]
    fn test_all_invalid() {
        assert_eq!(sum_csv("abc,def,ghi"), 0);
    }
}

fn main() {
    println!("Sum of '1,2,3' = {}", sum_csv("1,2,3"));
}
```

## Correct Code

```rust
/// Parses a comma-separated string of integers and returns the sum.
/// Ignores any entries that are not valid integers.
///
/// Examples:
///   "1,2,3" -> 6
///   "10, 20, 30" -> 60 (handles spaces)
///   "1,abc,3" -> 4 (ignores invalid entries)
///   "" -> 0
fn sum_csv(input: &str) -> i64 {
    input
        .split(',')
        .map(|s| s.trim())
        .filter_map(|s| s.parse::<i64>().ok())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sum() {
        assert_eq!(sum_csv("1,2,3"), 6);
    }

    #[test]
    fn test_with_spaces() {
        assert_eq!(sum_csv("10, 20, 30"), 60);
    }

    #[test]
    fn test_ignores_invalid() {
        assert_eq!(sum_csv("1,abc,3"), 4);
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(sum_csv(""), 0);
    }

    #[test]
    fn test_single_value() {
        assert_eq!(sum_csv("42"), 42);
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(sum_csv("-1,2,-3"), -2);
    }

    #[test]
    fn test_all_invalid() {
        assert_eq!(sum_csv("abc,def,ghi"), 0);
    }
}

fn main() {
    println!("Sum of '1,2,3' = {}", sum_csv("1,2,3"));
}
```

## Explanation

The broken version compiles successfully. The `todo!()` macro is type-compatible with any return type because it diverges (panics, so it never actually returns). This means the function signature is valid and the tests compile. But when any test calls `sum_csv`, the `todo!()` macro panics with "not yet implemented," causing the test to fail.

This is the TDD starting point: code compiles, tests fail.

The correct implementation uses a chain of iterator methods:

1. **`.split(',')`** -- Splits the input string on commas, producing an iterator of `&str` slices.

2. **`.map(|s| s.trim())`** -- Strips leading and trailing whitespace from each segment. This handles inputs like `"10, 20, 30"` where spaces follow the commas.

3. **`.filter_map(|s| s.parse::<i64>().ok())`** -- This combines filtering and mapping in one step. `s.parse::<i64>()` returns `Result<i64, ParseIntError>`. Calling `.ok()` converts it to `Option<i64>` -- `Some(n)` for valid integers, `None` for invalid strings. `filter_map` keeps only the `Some` values and unwraps them. Invalid entries like `"abc"` are silently discarded.

4. **`.sum()`** -- Consumes the iterator and sums all values. The `Sum` trait is implemented for numeric types. For an empty iterator, it returns 0, which correctly handles both the empty string case and the all-invalid case.

The TDD workflow in Rust:

1. **Write the function signature** with `todo!()` as the body.
2. **Write tests** that define the expected behavior.
3. **Run `cargo test`** -- all tests should fail with "not yet implemented."
4. **Implement the function** to make tests pass.
5. **Run `cargo test`** again -- all tests should pass.
6. **Refactor** with confidence, knowing the tests catch regressions.

Notice that `todo!()` is different from returning a wrong value. `todo!()` panics, which means every test fails loudly. If you had written `return 0;` as a placeholder, the empty-string test would pass but others would fail with confusing assertion errors. `todo!()` makes the failure mode clear: "this is not implemented yet."

## Compiler Error Interpretation

```
---- tests::test_simple_sum stdout ----
thread 'tests::test_simple_sum' panicked at 'not yet implemented', src/main.rs:11:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- tests::test_with_spaces stdout ----
thread 'tests::test_with_spaces' panicked at 'not yet implemented', src/main.rs:11:5
```

This is not a compiler error -- the code compiles fine. These are **runtime panics** from `cargo test`. The message "not yet implemented" comes from the `todo!()` macro. Each test runs in its own thread, so one panic does not prevent other tests from running. The output shows every test failing at the same line (`src/main.rs:11:5`), which is where `todo!()` lives. Once you replace `todo!()` with a real implementation, these panics disappear and the tests either pass or fail based on your logic.
