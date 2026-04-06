---
id: error-handling-across-boundary
phase: 20
phase_title: "Host and Guest Contracts"
sequence: 2
title: Error Handling Across the Boundary — No Panics Allowed
hints:
  - WASM traps are unrecoverable -- a panic in the guest kills the instance
  - Errors must be communicated as return values, not panics
  - Use a repr(i32) enum to encode error codes that cross the boundary
  - catch_unwind requires the closure's captures to be UnwindSafe
---

## Description

In WASM, a trap (equivalent to Rust's `panic!`) is unrecoverable. The module instance is terminated and cannot be resumed. This means well-designed WASM APIs should never panic on invalid input -- they should return error codes. When the guest encounters an error, it returns a numeric code (like `-1` for invalid input, `-2` for out of memory). The host inspects the return value and handles the error. This kata demonstrates why `catch_unwind` is problematic at boundaries and how `Result`-based error codes are the correct pattern.

## Broken Code

```rust
use std::panic;

struct GuestState {
    counter: i32,
}

/// A guest function that panics on invalid input.
fn guest_divide(state: &mut GuestState, a: i32, b: i32) -> i32 {
    state.counter += 1;
    if b == 0 {
        // BUG: Panicking in a WASM guest kills the instance.
        // The host cannot recover from this.
        panic!("Division by zero!");
    }
    a / b
}

fn main() {
    let mut state = GuestState { counter: 0 };

    // Try to catch the panic at the "host" boundary
    let result = panic::catch_unwind(|| {
        // BUG: &mut GuestState is not UnwindSafe because catch_unwind
        // could observe a partially-modified state after a panic.
        guest_divide(&mut state, 10, 0)
    });

    match result {
        Ok(val) => println!("Result: {}", val),
        Err(_) => println!("Guest panicked!"),
    }
}
```

## Correct Code

```rust
/// Error codes that can cross the WASM boundary as integers.
#[repr(i32)]
#[derive(Debug)]
enum GuestError {
    DivisionByZero = -1,
    Overflow = -2,
    InvalidInput = -3,
}

struct GuestState {
    counter: i32,
}

/// A guest function that returns Result instead of panicking.
fn guest_divide(
    state: &mut GuestState,
    a: i32,
    b: i32,
) -> Result<i32, GuestError> {
    state.counter += 1;
    if b == 0 {
        // Correct: return an error code, do not panic
        return Err(GuestError::DivisionByZero);
    }
    Ok(a / b)
}

/// Simulate the WASM ABI: return a single i32 where
/// negative values are error codes and non-negative values are results.
fn guest_divide_abi(state: &mut GuestState, a: i32, b: i32) -> i32 {
    match guest_divide(state, a, b) {
        Ok(result) => result,
        Err(e) => e as i32, // Convert error enum to negative i32
    }
}

fn main() {
    let mut state = GuestState { counter: 0 };

    // Valid call
    let result = guest_divide_abi(&mut state, 10, 3);
    if result >= 0 {
        println!("10 / 3 = {}", result);
    } else {
        println!("Error code: {}", result);
    }

    // Error case -- returns error code, does not panic
    let result = guest_divide_abi(&mut state, 10, 0);
    if result >= 0 {
        println!("10 / 0 = {}", result);
    } else {
        println!("Error code: {} (division by zero)", result);
    }

    println!("Guest state is intact: counter = {}", state.counter);
}
```

## Explanation

The broken version has two problems:

1. **The guest panics** on invalid input (`panic!("Division by zero!")`). In WASM, this would terminate the instance -- the host cannot catch it.

2. **`catch_unwind` does not compile** because the closure captures `&mut GuestState`, which is not `UnwindSafe`. Rust requires this trait because after a panic, the mutable reference might point to a partially-modified state. Allowing the caller to observe this state would violate memory safety.

**Why WASM traps are unrecoverable:**

When a WASM module traps (division by zero, out-of-bounds access, `unreachable` instruction), the runtime halts execution immediately. Unlike exceptions in Java or Python, there is no unwinding, no catch block, no finally. The module instance is dead. If the host wants to run more WASM code, it must create a new instance from scratch.

**The error code pattern:**

Since WASM functions return numeric values (i32, i64, f32, f64), errors are encoded as special return values:

| Return Value | Meaning |
|-------------|---------|
| >= 0 | Success (the actual result) |
| -1 | Division by zero |
| -2 | Overflow |
| -3 | Invalid input |

This is the same pattern used by C system calls (`errno`), POSIX functions, and WASI. The `#[repr(i32)]` attribute on the error enum ensures each variant maps to a specific integer that both the host and guest agree on.

**Why `Result` is better than panics for boundaries:**

| Approach | Guest Impact | Host Impact | Recovery |
|----------|-------------|-------------|----------|
| `panic!` | Instance dies | Must create new instance | Not possible |
| `Result` / error code | State preserved | Inspects return value | Full recovery |

The invariant violated in the broken code: **functions that cross the WASM boundary must return error codes, not panic; traps are unrecoverable and destroy the module instance.**

## ⚠️ Caution

- Panics in WASM trap the entire module — the host gets no error information, just an abort. Always use error codes instead of panics for boundary functions.
- Negative error codes can conflict with valid negative return values. Document your error code convention clearly.

## 💡 Tips

- Use `#[repr(i32)]` enums for error codes to ensure ABI stability.
- Follow WASI conventions: 0 = success, positive = error code.
- Use `catch_unwind` as a last resort to convert panics to error codes at the boundary.

## Compiler Error Interpretation

```
error[E0277]: the type `&mut GuestState` may not be safely transferred
              across an unwind boundary
  --> src/main.rs:18:38
   |
18 |     let result = panic::catch_unwind(|| {
   |                  ------------------- ^^ `&mut GuestState` may not be
   |                  |                      safely transferred across an
   |                  |                      unwind boundary
   |                  required by a bound introduced by this call
   |
   = help: the trait `UnwindSafe` is not implemented for `&mut GuestState`
   = note: `UnwindSafe` is implemented for `&GuestState`, but not for
           `&mut GuestState`
```

The compiler error explains:

1. **"may not be safely transferred across an unwind boundary"** -- a `&mut` reference inside `catch_unwind` is dangerous because the panic might have left the data in an inconsistent state.
2. **"`UnwindSafe` is not implemented for `&mut GuestState`"** -- only shared references (`&T`) are `UnwindSafe` by default, not mutable references. This is because a panic could happen mid-mutation, leaving the state half-updated.
3. **The deeper lesson:** Even if you could catch the panic, the guest's state might be corrupted. The `Result`-based approach avoids this entirely -- the guest returns an error before any corruption can occur.

---

| [Prev: Import/Export Contracts — Type-Safe Module Boundaries](#/katas/import-export-contracts) | [Next: Stable Interface Versioning — Evolving Without Breaking](#/katas/stable-abi-versioning) |
