---
id: extern-c-export-convention
phase: 18
phase_title: "Rust to WASM Toolchain"
sequence: 3
title: Export Convention — Functions the Host Can Call
hints:
  - Rust mangles function names by default to avoid symbol collisions
  - "#[no_mangle]" tells the compiler to keep the original function name
  - Without no_mangle, the symbol name becomes something like "_ZN8my_crate3add17h1234567890abcdef"
  - WASM exports must have predictable names so the host can find them
---

## Description

When Rust compiles to WASM, functions that should be callable by the host must be exported with a predictable name. By default, Rust *mangles* function names -- it encodes the crate name, module path, and a hash into the symbol name to avoid collisions. This means a function named `add` might become `_ZN8my_crate3add17habcdef1234567890E` in the compiled output. The host (JavaScript, Wasmtime, etc.) cannot find the function by its original name. To fix this, WASM-exported functions must use `#[no_mangle]` to preserve the original name and `extern "C"` for ABI compatibility.

## Broken Code

```rust
use std::collections::HashMap;

// This function should be an export, but it is missing #[no_mangle].
// Rust will mangle its name, making it unfindable by the host.
extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// Simulate a WASM module registry where the host looks up exports by name.
struct ModuleExports {
    functions: HashMap<String, fn(i32, i32) -> i32>,
}

impl ModuleExports {
    fn new() -> Self {
        ModuleExports {
            functions: HashMap::new(),
        }
    }

    fn register(&mut self, name: &str, f: fn(i32, i32) -> i32) {
        self.functions.insert(name.to_string(), f);
    }

    fn call(&self, name: &str, a: i32, b: i32) -> i32 {
        // BUG: The function is registered under its mangled name,
        // but the host looks it up by the original name.
        let f = self.functions.get(name).unwrap();
        f(a, b)
    }
}

fn main() {
    let mut exports = ModuleExports::new();

    // Simulate the linker registering functions under mangled names
    // (in real WASM, the compiler would mangle these automatically)
    exports.register("_ZN3add17habcdef1234567E", add);
    exports.register("_ZN8multiply17habcdef1234567E", multiply);

    // The host tries to look up by the clean name -- fails!
    let result = exports.call("add", 3, 4);
    println!("3 + 4 = {}", result);
}
```

## Correct Code

```rust
use std::collections::HashMap;

// Correct: #[no_mangle] preserves the original function name.
// extern "C" ensures the function uses the C calling convention.
#[no_mangle]
extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

struct ModuleExports {
    functions: HashMap<String, fn(i32, i32) -> i32>,
}

impl ModuleExports {
    fn new() -> Self {
        ModuleExports {
            functions: HashMap::new(),
        }
    }

    fn register(&mut self, name: &str, f: fn(i32, i32) -> i32) {
        self.functions.insert(name.to_string(), f);
    }

    fn call(&self, name: &str, a: i32, b: i32) -> Option<i32> {
        self.functions.get(name).map(|f| f(a, b))
    }
}

fn main() {
    let mut exports = ModuleExports::new();

    // With #[no_mangle], functions are registered under their clean names
    exports.register("add", add);
    exports.register("multiply", multiply);

    // The host looks up by name -- succeeds!
    match exports.call("add", 3, 4) {
        Some(result) => println!("3 + 4 = {}", result),
        None => println!("Export 'add' not found"),
    }

    match exports.call("multiply", 5, 6) {
        Some(result) => println!("5 * 6 = {}", result),
        None => println!("Export 'multiply' not found"),
    }

    // Attempting to call a non-existent export
    match exports.call("divide", 10, 2) {
        Some(result) => println!("10 / 2 = {}", result),
        None => println!("Export 'divide' not found -- not in module"),
    }
}
```

## Explanation

The broken version registers functions under mangled names (simulating what the Rust compiler would do without `#[no_mangle]`), but the host looks them up by their original names. The `HashMap::get("add")` call returns `None` because the key is `"_ZN3add17habcdef1234567E"`, not `"add"`. The `unwrap()` panics.

**What name mangling is:**

Rust mangles function names to prevent symbol collisions. If two crates both define a function called `add`, the linker would see a duplicate symbol. Mangling encodes the crate name, module path, and a hash into the symbol name, making each symbol globally unique. The mangled name for `my_crate::math::add` might be `_ZN8my_crate4math3add17habcdef1234567890E`.

**Why WASM exports need `#[no_mangle]`:**

WASM modules expose their exports by name in the export table. When JavaScript calls `instance.exports.add(3, 4)`, the runtime looks up the symbol `"add"` in the export table. If the symbol was mangled, the lookup fails. The `#[no_mangle]` attribute tells the compiler to use the exact function name as the symbol name.

**Why `extern "C"` is also needed:**

`extern "C"` specifies the calling convention. WASM functions use a well-defined ABI similar to C: arguments are passed in order, return values come back in a register. Without `extern "C"`, Rust might use its own (unstable) calling convention, which is not compatible with the WASM function call ABI.

**The pattern for WASM exports:**

```rust
#[no_mangle]
pub extern "C" fn exported_function(arg: i32) -> i32 {
    // ...
}
```

Three attributes work together: `#[no_mangle]` for the name, `extern "C"` for the ABI, and `pub` for visibility. This is the standard pattern for every function that should appear in the WASM export table.

The invariant violated in the broken code: **WASM-exported functions must use `#[no_mangle]` to preserve their name in the export table, so the host can find them.**

## ⚠️ Caution

- `#[no_mangle]` places the symbol in the global namespace. Two modules with the same `#[no_mangle]` function name will conflict at link time.
- Forgetting `extern "C"` means Rust uses its own (unstable) calling convention, which will not work for WASM exports or C interop.

## 💡 Tips

- Always use the three-attribute pattern together: `#[no_mangle]`, `extern "C"`, and `pub`. Missing any one causes subtle issues.
- Use `#[export_name = "custom_name"]` instead of `#[no_mangle]` when you want a specific export name that differs from the Rust function name.
- Only use WASM-compatible types (i32, i64, f32, f64) in exported function signatures.

## Compiler Error Interpretation

```
thread 'main' panicked at 'called `Option::unwrap()` on a `None` value',
  src/main.rs:35:52
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

This is a runtime panic from the `unwrap()` on a `HashMap::get` that returned `None`:

1. **"called `Option::unwrap()` on a `None` value"** -- the `HashMap` does not contain the key `"add"`. The function was registered under its mangled name, not its clean name.
2. **"src/main.rs:35:52"** -- the panic occurs in the `call` method where `unwrap()` is called on the lookup result.

In a real WASM runtime, this would be an instantiation error: "import 'add' not found in module exports." The module would fail to load, not crash at runtime. WASM validates all imports and exports at module load time, catching these mismatches before any code runs.

---

| [Prev: No Garbage Collector — Ownership IS the Memory Model](#/katas/no-gc-manual-drop) | [Next: Pointer Offset Arithmetic in Linear Memory](#/katas/offset-arithmetic) |
