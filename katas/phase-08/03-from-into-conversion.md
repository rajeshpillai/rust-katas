---
id: from-into-conversion
phase: 8
phase_title: "Error Handling as Design"
sequence: 3
title: "From/Into for Error Type Conversion with ?"
hints:
  - "The `?` operator calls `.into()` on the error, which uses the `From` trait to convert it."
  - "If no `From<SourceError>` is implemented for your target error type, the `?` conversion fails."
  - "Implement `From<SourceError> for YourError` so `?` can automatically convert between error types."
---

## Description

The `?` operator does more than just early-return on errors. It also performs **automatic error conversion** using the `From` trait. When you write `expression?` in a function that returns `Result<T, TargetError>`, and the expression returns `Result<U, SourceError>`, the `?` operator calls `SourceError.into()` to convert it to `TargetError`.

This only works if `impl From<SourceError> for TargetError` exists. Without it, the types are incompatible, and the `?` operator fails to compile.

## Broken Code

```rust
use std::fs;
use std::num::ParseIntError;

#[derive(Debug)]
enum AppError {
    FileError(String),
    ParseError(String),
}

fn read_config_value(path: &str) -> Result<i64, AppError> {
    // fs::read_to_string returns Result<String, io::Error>.
    // But our function returns Result<i64, AppError>.
    // There is no From<io::Error> for AppError, so ? cannot convert.
    let content = fs::read_to_string(path)?;

    // str::parse returns Result<i64, ParseIntError>.
    // There is no From<ParseIntError> for AppError either.
    let value: i64 = content.trim().parse()?;

    Ok(value)
}

fn main() {
    match read_config_value("config.txt") {
        Ok(val) => println!("Config value: {}", val),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

## Correct Code

```rust
use std::fmt;
use std::fs;
use std::num::ParseIntError;

#[derive(Debug)]
enum AppError {
    FileError(String),
    ParseError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::FileError(msg) => write!(f, "file error: {}", msg),
            AppError::ParseError(msg) => write!(f, "parse error: {}", msg),
        }
    }
}

// This From impl allows ? to convert io::Error into AppError.
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::FileError(err.to_string())
    }
}

// This From impl allows ? to convert ParseIntError into AppError.
impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> Self {
        AppError::ParseError(err.to_string())
    }
}

fn read_config_value(path: &str) -> Result<i64, AppError> {
    // ? now works: io::Error is converted to AppError via From
    let content = fs::read_to_string(path)?;

    // ? now works: ParseIntError is converted to AppError via From
    let value: i64 = content.trim().parse()?;

    Ok(value)
}

fn main() {
    match read_config_value("config.txt") {
        Ok(val) => println!("Config value: {}", val),
        Err(AppError::FileError(msg)) => {
            println!("Could not read config file: {}", msg);
        }
        Err(AppError::ParseError(msg)) => {
            println!("Config file contains invalid data: {}", msg);
        }
    }
}
```

## Explanation

The `?` operator is syntactic sugar for:

```rust
match expression {
    Ok(val) => val,
    Err(err) => return Err(From::from(err)),
    //                      ^^^^^^^^^^^^^^
    //                      This is the key part!
}
```

The `From::from(err)` call is what converts the source error type into the target error type. If no `From` implementation exists for that conversion, the compiler rejects the code.

In the broken version:
- `fs::read_to_string(path)` returns `Result<String, io::Error>`
- The function returns `Result<i64, AppError>`
- The `?` tries to do `AppError::from(io_error)`, but `From<io::Error>` is not implemented for `AppError`

The fix adds two `From` implementations:

1. `From<std::io::Error> for AppError` — converts I/O errors into the `FileError` variant
2. `From<ParseIntError> for AppError` — converts parse errors into the `ParseError` variant

Now `?` can perform the conversion automatically.

**The `From`/`Into` relationship:**

`From` and `Into` are reciprocal traits. If you implement `From<A> for B`, you automatically get `Into<B> for A`. The `?` operator uses `From` (technically it uses `Into`, which delegates to `From`).

```rust
// You implement this:
impl From<io::Error> for AppError { ... }

// You get this for free:
// impl Into<AppError> for io::Error { ... }
```

**Design decision: wrapping vs converting:**

In the correct version, we convert the original error to a `String`. This is simple but loses the original error type. An alternative is to **wrap** the original error:

```rust
#[derive(Debug)]
enum AppError {
    FileError(std::io::Error),      // Wraps the original error
    ParseError(ParseIntError),      // Wraps the original error
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::FileError(err)    // Wraps, not converts
    }
}
```

Wrapping preserves the original error, which is useful for:
- Downcasting to the specific error type later
- Implementing `std::error::Error::source()` for error chains
- Providing maximum diagnostic information

**The `map_err` alternative:**

If you do not want to implement `From`, you can use `map_err` to convert errors manually at each call site:

```rust
let content = fs::read_to_string(path)
    .map_err(|e| AppError::FileError(e.to_string()))?;
```

This is more verbose but gives you fine-grained control at each conversion point. Use `From` when the conversion is always the same; use `map_err` when you need context-specific conversion.

## Compiler Error Interpretation

```
error[E0277]: `?` couldn't convert the error to `AppError`
  --> src/main.rs:13:44
   |
10 | fn read_config_value(path: &str) -> Result<i64, AppError> {
   |                                     ---------------------- expected `AppError` because of this
...
13 |     let content = fs::read_to_string(path)?;
   |                                            ^ the trait `From<std::io::Error>` is not implemented for `AppError`
   |
   = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
   = help: the following other types implement trait `From<T>`:
             <AppError from ParseIntError>  (if it existed)
   = note: required for `Result<i64, AppError>` to implement `FromResidual<Result<Infallible, std::io::Error>>`
```

This error precisely identifies the missing link:

- **"`?` couldn't convert the error to `AppError`"** — The `?` operator tried and failed to convert.
- **"the trait `From<std::io::Error>` is not implemented for `AppError`"** — The exact trait implementation that is missing. This tells you exactly what to write: `impl From<std::io::Error> for AppError`.
- **"the question mark operation implicitly performs a conversion"** — The compiler reminds you of the mechanism, so you understand why `From` is needed.
- **"expected `AppError` because of this"** — Points to the function's return type, showing why `AppError` is the target type.

The error message is essentially a specification for the fix: implement `From<std::io::Error> for AppError`, and the `?` operator will work.
