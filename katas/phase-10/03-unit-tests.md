---
id: unit-tests
phase: 10
phase_title: "Modules, Visibility & Testing"
sequence: 3
title: "Writing Unit Tests"
hints:
  - Every test function must be annotated with #[test]
  - Test modules are conventionally annotated with #[cfg(test)] to exclude them from non-test builds
  - Tests within the same module can access private functions -- this is by design
  - assert_eq! compares two values; the order is (left, right) and both must implement PartialEq and Debug
---

## Description

Rust has first-class support for testing built into the language and toolchain. Unit tests live alongside the code they test, typically in a `#[cfg(test)]` module at the bottom of the same file. The `#[test]` attribute marks a function as a test case, and `cargo test` discovers and runs all such functions.

A crucial design decision in Rust: **tests defined within the same module can access private functions**. This is because the test module is a child of the module it tests, and child modules can see their parent's private items. This eliminates the need for awkward workarounds to test internal logic.

## Broken Code

```rust
fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

fn classify_temperature(celsius: f64) -> &'static str {
    if celsius < 0.0 {
        "freezing"
    } else if celsius < 20.0 {
        "cold"
    } else if celsius < 30.0 {
        "comfortable"
    } else {
        "hot"
    }
}

// Missing #[cfg(test)]
mod tests {
    // Missing: bringing parent functions into scope

    // Missing #[test] attribute -- this function won't be run by cargo test
    fn test_boiling_point() {
        let result = fahrenheit_to_celsius(212.0);
        assert_eq!(result, 100.0);
    }

    // Wrong assertion: arguments swapped won't cause a compile error,
    // but the message will be confusing. More importantly, this test
    // has a logic error in the expected value.
    fn test_freezing_point() {
        let result = fahrenheit_to_celsius(32.0);
        assert_eq!(result, 32.0); // Wrong! 32°F = 0°C, not 32°C
    }

    fn test_classification() {
        assert_eq!(classify_temperature(-5.0), "freezing");
        assert_eq!(classify_temperature(15.0), "cold");
        assert_eq!(classify_temperature(25.0), "comfortable");
    }
}

fn main() {
    let temp = fahrenheit_to_celsius(72.0);
    println!("72°F = {:.1}°C -- {}", temp, classify_temperature(temp));
}
```

## Correct Code

```rust
fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

fn classify_temperature(celsius: f64) -> &'static str {
    if celsius < 0.0 {
        "freezing"
    } else if celsius < 20.0 {
        "cold"
    } else if celsius < 30.0 {
        "comfortable"
    } else {
        "hot"
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Bring all parent items into scope (including private ones!)

    #[test]
    fn test_boiling_point() {
        let result = fahrenheit_to_celsius(212.0);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_freezing_point() {
        let result = fahrenheit_to_celsius(32.0);
        assert_eq!(result, 0.0); // Correct: 32°F = 0°C
    }

    #[test]
    fn test_classification() {
        assert_eq!(classify_temperature(-5.0), "freezing");
        assert_eq!(classify_temperature(15.0), "cold");
        assert_eq!(classify_temperature(25.0), "comfortable");
    }

    #[test]
    fn test_hot_classification() {
        assert_eq!(classify_temperature(35.0), "hot");
    }
}

fn main() {
    let temp = fahrenheit_to_celsius(72.0);
    println!("72°F = {:.1}°C -- {}", temp, classify_temperature(temp));
}
```

## Explanation

The broken version has four distinct issues:

**1. Missing `#[cfg(test)]` on the test module.** Without this attribute, the `tests` module is compiled into the production binary. The `#[cfg(test)]` attribute is a conditional compilation flag that says "only compile this module when running `cargo test`." This keeps test code out of release builds.

**2. Missing `#[test]` attribute on test functions.** Without `#[test]`, `cargo test` does not recognize these functions as tests. They are just ordinary functions that never get called. This is a silent failure -- your tests exist but never run, giving false confidence.

**3. Missing `use super::*`.** The test module is a child module. To call `fahrenheit_to_celsius` and `classify_temperature`, it needs to bring them into scope. `use super::*` imports everything from the parent module. The crucial insight: **this includes private items**. Because the test module is a child of the module being tested, Rust's visibility rules grant it access to private functions. This is intentional and is one of the strongest arguments for colocating tests with the code they test.

**4. Wrong expected value in `test_freezing_point`.** The test asserts that 32 degrees Fahrenheit converts to 32 degrees Celsius. But 32 degrees Fahrenheit is the freezing point of water, which is 0 degrees Celsius. This is a logic error in the test itself. Incorrect assertions are insidious because they can mask real bugs -- or worse, they fail and you "fix" the production code to match the wrong expectation.

When `cargo test` runs, it:
1. Compiles the code with `cfg(test)` enabled
2. Discovers all functions marked `#[test]`
3. Runs each one in a separate thread
4. Reports pass/fail for each test

A test passes if it returns without panicking. `assert_eq!` panics (and thus fails the test) if its two arguments are not equal. The error message shows both values, making it easy to diagnose failures.

## Compiler Error Interpretation

```
error[E0425]: cannot find function `fahrenheit_to_celsius` in this scope
 --> src/main.rs:23:22
   |
23 |         let result = fahrenheit_to_celsius(212.0);
   |                      ^^^^^^^^^^^^^^^^^^^^^ not found in this scope
   |
help: consider importing this function
   |
20 |     use super::fahrenheit_to_celsius;
   |     +++++++++++++++++++++++++++++++++
```

The compiler cannot find `fahrenheit_to_celsius` inside the `tests` module because it was not imported. The help message suggests using `use super::fahrenheit_to_celsius` to bring it into scope. Using `use super::*` is a common shorthand that imports everything from the parent, which is convenient for test modules that need access to many items.

Note: the missing `#[test]` attribute does not produce a compiler error -- it silently causes your tests to be ignored. The compiler will warn about unused functions if no code calls them, which can be a clue, but it is easy to overlook. Always verify your tests actually run by checking `cargo test` output for the expected test names.
