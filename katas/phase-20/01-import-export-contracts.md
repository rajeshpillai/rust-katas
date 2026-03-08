---
id: import-export-contracts
phase: 20
phase_title: "Host and Guest Contracts"
sequence: 1
title: Import/Export Contracts — Type-Safe Module Boundaries
hints:
  - WASM modules declare imports with specific type signatures
  - If the host provides a function with a different signature, instantiation fails
  - Traits in Rust model the same contract -- implementations must match exactly
  - Check that your method signature matches the trait definition
---

## Description

WASM modules declare their imports (functions they need from the host) and exports (functions they provide to the host) with precise type signatures. If the host provides an import function whose signature does not match what the module declared, the module fails to instantiate -- it never runs. This is a *link-time* check, not a runtime check. In Rust, traits model the same idea: an implementation must match the trait's method signatures exactly. This kata demonstrates what happens when an implementation's method signature diverges from the trait contract.

## Broken Code

```rust
/// The contract: what the guest module expects from the host.
/// This is the WASM "import" declaration.
trait HostImports {
    fn log(&self, message: &str);
    fn get_time(&self) -> u64;
}

/// The host's implementation of the import contract.
struct BrowserHost;

impl HostImports for BrowserHost {
    // BUG: Wrong parameter type! The trait says &str, but this says &String.
    fn log(&self, message: &String) {
        println!("[LOG] {}", message);
    }

    fn get_time(&self) -> u64 {
        1708700000
    }
}

/// The guest module that depends on host imports.
struct GuestModule<H: HostImports> {
    host: H,
}

impl<H: HostImports> GuestModule<H> {
    fn new(host: H) -> Self {
        GuestModule { host }
    }

    fn run(&self) {
        self.host.log("Module started");
        let time = self.host.get_time();
        self.host.log(&format!("Current time: {}", time));
    }
}

fn main() {
    let host = BrowserHost;
    let module = GuestModule::new(host);
    module.run();
}
```

## Correct Code

```rust
/// The contract: what the guest module expects from the host.
trait HostImports {
    fn log(&self, message: &str);
    fn get_time(&self) -> u64;
}

/// The host's implementation -- signature matches exactly.
struct BrowserHost;

impl HostImports for BrowserHost {
    // Correct: parameter type matches the trait definition (&str)
    fn log(&self, message: &str) {
        println!("[LOG] {}", message);
    }

    fn get_time(&self) -> u64 {
        1708700000
    }
}

/// A second host implementation for testing.
struct MockHost {
    fixed_time: u64,
}

impl HostImports for MockHost {
    fn log(&self, message: &str) {
        // Capture logs instead of printing
        println!("[MOCK LOG] {}", message);
    }

    fn get_time(&self) -> u64 {
        self.fixed_time
    }
}

struct GuestModule<H: HostImports> {
    host: H,
}

impl<H: HostImports> GuestModule<H> {
    fn new(host: H) -> Self {
        GuestModule { host }
    }

    fn run(&self) {
        self.host.log("Module started");
        let time = self.host.get_time();
        self.host.log(&format!("Current time: {}", time));
    }
}

fn main() {
    // Production host
    let host = BrowserHost;
    let module = GuestModule::new(host);
    module.run();

    println!("---");

    // Test host with controlled time
    let mock = MockHost { fixed_time: 5000 };
    let test_module = GuestModule::new(mock);
    test_module.run();
}
```

## Explanation

The broken version implements `log` with the parameter type `&String` instead of `&str`. Even though `&String` can be coerced to `&str` in many contexts, a trait implementation must match the trait's method signature *exactly*. The compiler rejects the mismatched type with error E0053.

**How this maps to WASM:**

In WASM, a module declares its imports with precise type signatures:

```wasm
(import "env" "log" (func $log (param i32 i32)))  ;; ptr, len
(import "env" "get_time" (func $get_time (result i64)))
```

When the host instantiates the module, it must provide functions that match these signatures exactly. If the host's `log` function expects three parameters instead of two, or returns a value when the module expects none, instantiation fails. The module never executes a single instruction.

**Why strict matching matters:**

Loose type matching at boundaries leads to subtle bugs:
- In C, passing a `float` where a `double` is expected silently produces wrong values
- In JavaScript, passing a string where a number is expected may work due to implicit coercion, but produces surprising results

WASM and Rust both take the strict approach: the types must match, period. This catches interface mismatches at compile time (Rust) or link time (WASM), not at runtime when the data is already corrupted.

The invariant violated in the broken code: **a trait implementation's method signatures must exactly match the trait definition; interface contracts are enforced by the compiler.**

## ⚠️ Caution

- WASM import/export contracts are checked at instantiation time, not compile time. A type mismatch between host and guest is a runtime error.
- Renaming or reordering function parameters is an ABI-breaking change, even if the types are the same.

## 💡 Tips

- Model WASM contracts as Rust traits — this gives you compile-time checking of the contract shape.
- Version your contracts explicitly so hosts and guests can negotiate compatibility.
- Test contract compliance with integration tests that instantiate the WASM module.

## Compiler Error Interpretation

```
error[E0053]: method `log` has an incompatible type for trait
  --> src/main.rs:13:29
   |
4  |     fn log(&self, message: &str);
   |                            ---- type in trait
...
13 |     fn log(&self, message: &String) {
   |                             ^^^^^^ expected `str`, found `String`
   |
   = note: expected signature `fn(&BrowserHost, &str)`
              found signature `fn(&BrowserHost, &String)`
```

The compiler error is precise:

1. **"method `log` has an incompatible type for trait"** -- the method exists but its signature does not match.
2. **"expected `str`, found `String`"** -- the trait says `&str`, but the implementation says `&String`. These are different types. `&str` is a borrowed string slice; `&String` is a reference to a heap-allocated `String`.
3. **"expected signature / found signature"** -- the compiler shows both signatures side by side for easy comparison.

This is Rust's equivalent of a WASM link error. Just as a WASM runtime rejects a module when import signatures do not match, the Rust compiler rejects an implementation when method signatures do not match the trait.
