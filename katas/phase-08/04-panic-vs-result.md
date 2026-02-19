---
id: panic-vs-result
phase: 8
phase_title: "Error Handling as Design"
sequence: 4
title: When to Panic vs When to Return Result
hints:
  - "Library code should almost never panic. Panicking takes away the caller's ability to handle the error."
  - "Using `unwrap()` or `expect()` on `None` or `Err` causes a panic — the program crashes."
  - "Return `Result` or `Option` to let the caller decide how to handle failure."
---

## Description

Rust has two distinct error handling mechanisms:

- **Panic** (`panic!`, `unwrap()`, `expect()`, array out-of-bounds): Crashes the thread. Cannot be caught in normal code. Represents bugs or unrecoverable situations.
- **Result/Option**: Returns error information to the caller. The caller decides what to do. Represents expected, recoverable failures.

The choice between them is a design decision with real consequences. Using `panic!` in library code forces every caller into a crash-or-nothing situation. Using `Result` gives callers control.

## Broken Code

```rust
/// A library function that looks up a user by ID.
/// This is meant to be used by other parts of the application.
fn find_user(users: &[(&str, u32)], target_id: u32) -> &str {
    // BUG: Using .find() + .unwrap() means this function panics
    // if the user is not found. The caller has no way to handle
    // a missing user gracefully.
    let (name, _) = users
        .iter()
        .find(|(_, id)| *id == target_id)
        .unwrap(); // PANIC if not found!
    name
}

/// Another library function that parses a config value.
fn get_port(config: &str) -> u16 {
    // BUG: .expect() is just .unwrap() with a message.
    // Still panics. Still kills the caller's thread.
    let port_str = config
        .split('=')
        .nth(1)
        .expect("config must have a value"); // PANIC!

    port_str
        .trim()
        .parse::<u16>()
        .expect("port must be a number") // PANIC!
}

fn main() {
    let users = vec![("Alice", 1), ("Bob", 2), ("Charlie", 3)];

    // This works fine:
    let name = find_user(&users, 1);
    println!("Found: {}", name);

    // This panics at runtime — the entire program crashes:
    let name = find_user(&users, 99);
    println!("Found: {}", name); // Never reached
}
```

## Correct Code

```rust
use std::fmt;
use std::num::ParseIntError;

/// A library function that looks up a user by ID.
/// Returns Option — None if not found, Some if found.
fn find_user<'a>(users: &'a [(&str, u32)], target_id: u32) -> Option<&'a str> {
    users
        .iter()
        .find(|(_, id)| *id == target_id)
        .map(|(name, _)| *name)
}

#[derive(Debug)]
enum ConfigError {
    MissingValue,
    InvalidPort(ParseIntError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingValue => write!(f, "config line missing '=' separator"),
            ConfigError::InvalidPort(e) => write!(f, "invalid port number: {}", e),
        }
    }
}

/// A library function that parses a config value.
/// Returns Result — the caller decides what to do on failure.
fn get_port(config: &str) -> Result<u16, ConfigError> {
    let port_str = config
        .split('=')
        .nth(1)
        .ok_or(ConfigError::MissingValue)?;

    let port = port_str
        .trim()
        .parse::<u16>()
        .map_err(ConfigError::InvalidPort)?;

    Ok(port)
}

fn main() {
    let users = vec![("Alice", 1), ("Bob", 2), ("Charlie", 3)];

    // The caller handles the "not found" case gracefully.
    match find_user(&users, 1) {
        Some(name) => println!("Found: {}", name),
        None => println!("User not found"),
    }

    match find_user(&users, 99) {
        Some(name) => println!("Found: {}", name),
        None => println!("User 99 not found — but the program continues"),
    }

    // The caller handles config errors gracefully.
    match get_port("port=8080") {
        Ok(port) => println!("Port: {}", port),
        Err(e) => println!("Config error: {}", e),
    }

    match get_port("invalid-config-line") {
        Ok(port) => println!("Port: {}", port),
        Err(e) => println!("Config error: {}", e),
    }

    match get_port("port=banana") {
        Ok(port) => println!("Port: {}", port),
        Err(e) => println!("Config error: {}", e),
    }

    println!("Program continues running after all errors.");
}
```

## Explanation

The broken version uses `unwrap()` and `expect()` in library functions. These methods panic when they encounter `None` or `Err`, crashing the current thread. This is problematic because:

1. **The caller loses control.** They cannot decide how to handle the error — the decision has been made for them (crash).

2. **Panics are not part of the type signature.** Looking at `fn find_user(...) -> &str`, the caller has no way to know this function might panic. It looks infallible.

3. **Panics are contagious.** If library code panics, the application crashes, even if the error was perfectly recoverable (a missing user is not a reason to crash a web server).

The correct version returns `Option` and `Result`, giving callers full control:

- `find_user` returns `Option<&str>` — `None` means "not found." The caller might show an error page, try a different lookup, or use a default.
- `get_port` returns `Result<u16, ConfigError>` — the caller might use a default port, ask the user for input, or propagate the error upstream.

**When to panic vs when to return Result:**

| Situation | Use | Example |
|---|---|---|
| Bug in the program (logic error) | `panic!` | Index out of bounds in trusted code |
| Violation of a function's precondition that indicates a bug | `panic!` / `assert!` | `assert!(slice.len() > 0)` in internal code |
| Environment setup failure that prevents program from running | `panic!` | Cannot initialize logger at startup |
| User input is invalid | `Result` | Bad email format, missing field |
| External system failure | `Result` | File not found, network timeout |
| Data not found | `Option` | User not in database |
| Any library code that others call | `Result` / `Option` | Almost always |

**The `unwrap()` / `expect()` guidelines:**

- **In tests**: `unwrap()` is fine — test failures should panic.
- **In examples / prototypes**: `unwrap()` is acceptable for brevity.
- **In `main()` at the top level**: `expect()` with a good message is reasonable.
- **In library code**: Almost never use `unwrap()` or `expect()`. Return `Result` or `Option`.

**Converting between `Option` and `Result`:**

```rust
// Option -> Result: provide an error value for the None case
let value: Result<T, E> = option.ok_or(MyError::NotFound)?;

// Result -> Option: discard the error information
let value: Option<T> = result.ok();
```

**The golden rule:** Library code should be a good citizen. It should inform the caller of errors, not make unilateral decisions about what to do. Let the caller decide. Return values, not panics.

## Compiler Error Interpretation

The broken version compiles without errors — panics are legal Rust code. This is a **design** problem, not a syntax problem. The compiler will not warn you about `unwrap()` in library code (though the `clippy` linter will, with the `clippy::unwrap_used` lint).

When a panic occurs at runtime, you see:

```
thread 'main' panicked at 'called `Option::unwrap()` on a `None` value', src/main.rs:7:10
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Compare this to the correct version, where errors are handled gracefully:

```
User 99 not found — but the program continues
Config error: config line missing '=' separator
Config error: invalid port number: invalid digit found in string
Program continues running after all errors.
```

The runtime panic message tells you:
- **"called `Option::unwrap()` on a `None` value"** — The `unwrap()` call hit a `None`. This is a crash, not a handled error.
- **"src/main.rs:7:10"** — Where it happened. In production, this is a stack trace in your logs, not a graceful error message for your users.

The lesson: if you see `unwrap()` in non-test code, ask yourself: "What happens when this is `None`? Is crashing really the right response?" Almost always, the answer is no. Return `Option` or `Result` and let the caller decide.

**Using clippy to catch this:**

```bash
cargo clippy -- -W clippy::unwrap_used
```

This will warn on every `unwrap()` call, helping you find places where panics might be hiding in your codebase.
