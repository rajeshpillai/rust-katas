---
id: custom-error-enums
phase: 8
phase_title: "Error Handling as Design"
sequence: 1
title: Defining Custom Error Types as Enums
hints:
  - "If a function can fail in multiple ways, each failure mode should be a variant in a custom error enum."
  - "When different error types come back from different operations, you need a single unified error type for the function's return."
  - "Define an enum with one variant per error source, and return `Result<T, YourError>`."
---

## Description

In Rust, errors are values. A function that can fail returns `Result<T, E>`, where `E` is the error type. When a function calls multiple operations that can each fail with different error types, you need a **unified error type** that can represent all possible failure modes.

The idiomatic approach is a custom error enum with one variant per failure source. This gives callers the ability to match on the specific error and handle each case appropriately.

## Broken Code

```rust
use std::fs;
use std::num::ParseIntError;

fn read_age_from_file(path: &str) -> Result<u32, ???> {
    // fs::read_to_string returns Result<String, std::io::Error>
    let content = fs::read_to_string(path)?;

    // str::parse::<u32>() returns Result<u32, ParseIntError>
    let age: u32 = content.trim().parse()?;

    if age > 150 {
        // How do we return a custom "validation" error?
        return Err(???);
    }

    Ok(age)
}

fn main() {
    match read_age_from_file("age.txt") {
        Ok(age) => println!("Age: {}", age),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Correct Code

```rust
use std::fmt;
use std::fs;
use std::num::ParseIntError;

#[derive(Debug)]
enum AgeReadError {
    IoError(std::io::Error),
    ParseError(ParseIntError),
    ValidationError { age: u32, message: String },
}

impl fmt::Display for AgeReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgeReadError::IoError(e) => write!(f, "failed to read file: {}", e),
            AgeReadError::ParseError(e) => write!(f, "failed to parse age: {}", e),
            AgeReadError::ValidationError { age, message } => {
                write!(f, "invalid age {}: {}", age, message)
            }
        }
    }
}

// These From implementations allow the ? operator to automatically
// convert source errors into our custom error type.
impl From<std::io::Error> for AgeReadError {
    fn from(err: std::io::Error) -> Self {
        AgeReadError::IoError(err)
    }
}

impl From<ParseIntError> for AgeReadError {
    fn from(err: ParseIntError) -> Self {
        AgeReadError::ParseError(err)
    }
}

fn read_age_from_file(path: &str) -> Result<u32, AgeReadError> {
    // ? converts std::io::Error to AgeReadError via From impl
    let content = fs::read_to_string(path)?;

    // ? converts ParseIntError to AgeReadError via From impl
    let age: u32 = content.trim().parse()?;

    if age > 150 {
        return Err(AgeReadError::ValidationError {
            age,
            message: String::from("age cannot exceed 150"),
        });
    }

    Ok(age)
}

fn main() {
    match read_age_from_file("age.txt") {
        Ok(age) => println!("Age: {}", age),
        Err(AgeReadError::IoError(e)) => {
            println!("Could not read the file: {}", e);
        }
        Err(AgeReadError::ParseError(e)) => {
            println!("The file did not contain a valid number: {}", e);
        }
        Err(AgeReadError::ValidationError { age, message }) => {
            println!("The age {} is not acceptable: {}", age, message);
        }
    }
}
```

## Explanation

The broken version cannot even compile because there is no single type that represents all three possible failure modes:
1. `std::io::Error` from reading the file
2. `ParseIntError` from parsing the string
3. A custom validation error for out-of-range values

The `?` operator needs a consistent error type for the function's return. If the function returns `Result<u32, std::io::Error>`, the `parse()?` line will not compile because `ParseIntError` is not `std::io::Error`. If it returns `Result<u32, ParseIntError>`, the `read_to_string()?` line will not compile.

The solution is a **custom error enum** that unifies all error sources:

```rust
enum AgeReadError {
    IoError(std::io::Error),       // Wraps the file reading error
    ParseError(ParseIntError),      // Wraps the parsing error
    ValidationError { ... },        // Our own domain-specific error
}
```

Each variant wraps the original error, preserving all its information. The caller can `match` on the error to handle each case differently.

**The `From` implementations** are what make the `?` operator work seamlessly. When `?` encounters an error, it calls `.into()` on it, which uses the `From` trait to convert the source error into the function's error type. Without these `From` impls, you would need to manually map each error:

```rust
// Without From impls, you'd write:
let content = fs::read_to_string(path).map_err(AgeReadError::IoError)?;
let age: u32 = content.trim().parse().map_err(AgeReadError::ParseError)?;
```

Both approaches work, but `From` impls are more ergonomic when you use `?` frequently.

**Design principle: errors are part of your API.** The error type is just as important as the success type. When you define `Result<u32, AgeReadError>`, you are telling callers exactly how this function can fail and giving them the tools to handle each failure mode appropriately.

**Error enum design guidelines:**

| Guideline | Reason |
|---|---|
| One variant per error source | Callers can distinguish and handle each case |
| Wrap the original error | Preserve diagnostic information |
| Implement `Display` | Enable user-friendly error messages |
| Implement `Debug` (derive it) | Enable developer-friendly debugging |
| Implement `From` for each source | Enable the `?` operator |

## Compiler Error Interpretation

If you tried to use `?` without a unified error type:

```
error[E0277]: `?` couldn't convert the error to `std::io::Error`
  --> src/main.rs:8:47
   |
4  | fn read_age_from_file(path: &str) -> Result<u32, std::io::Error> {
   |                                      --------------------------- expected `std::io::Error` because of this
...
8  |     let age: u32 = content.trim().parse()?;
   |                                          ^ the trait `From<ParseIntError>` is not implemented for `std::io::Error`
   |
   = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
   = help: the following other types implement trait `From<T>`:
```

This error reveals how `?` works under the hood:

- **"`?` couldn't convert the error to `std::io::Error`"** — The `?` operator tried to convert `ParseIntError` into `std::io::Error` (the function's declared error type), but there is no `From` implementation for that conversion.
- **"the trait `From<ParseIntError>` is not implemented for `std::io::Error`"** — The specific missing trait. You cannot teach `std::io::Error` to convert from `ParseIntError` (you do not own either type).
- **"the question mark operation implicitly performs a conversion"** — The compiler explains the mechanism: `?` is not just early return, it is early return **with conversion**.

The solution: define your own error type that *can* be converted from both `std::io::Error` and `ParseIntError`.
