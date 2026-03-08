---
id: deterministic-sandboxing
phase: 17
phase_title: "What is WebAssembly Really?"
sequence: 3
title: Deterministic Sandboxing — No Ambient Authority
hints:
  - WASM modules have zero implicit access to the outside world
  - All capabilities (time, I/O, randomness) must be explicitly provided by the host
  - If a module needs a Clock, the type system should require a Clock to be passed in
  - Use generics to enforce that dependencies are injected, not assumed
---

## Description

WASM modules are deterministic sandboxes. They cannot read the clock, access the filesystem, or make network requests unless the host explicitly provides these capabilities as import functions. This is not a convention -- it is structural. A WASM module physically cannot call `clock_gettime` because that function does not exist in its address space unless the host provides it. This kata demonstrates the capability injection pattern using Rust generics, showing how the type system can enforce that modules declare their dependencies explicitly.

## Broken Code

```rust
trait Clock {
    fn now_millis(&self) -> u64;
}

struct SystemClock;

impl Clock for SystemClock {
    fn now_millis(&self) -> u64 {
        // Simulate getting the current time
        1708700000000
    }
}

struct SandboxedModule<C: Clock> {
    clock: C,
    name: String,
}

impl<C: Clock> SandboxedModule<C> {
    fn new(name: String, clock: C) -> Self {
        SandboxedModule { clock, name }
    }

    fn elapsed_since(&self, start: u64) -> u64 {
        self.clock.now_millis() - start
    }
}

fn main() {
    let name = String::from("my-module");

    // BUG: Trying to create a SandboxedModule without specifying
    // the Clock implementation. The compiler cannot infer C.
    let module = SandboxedModule {
        clock: SystemClock,
        name,
    };

    // This line tries to use a module created with struct literal syntax,
    // but we also try to create one without providing the clock at all:
    let broken_module: SandboxedModule<_> = SandboxedModule {
        name: String::from("broken"),
        // BUG: missing field `clock` -- cannot create a sandboxed module
        // without providing its required capability
    };

    println!("{}", module.elapsed_since(1708700000000));
}
```

## Correct Code

```rust
trait Clock {
    fn now_millis(&self) -> u64;
}

struct SystemClock;

impl Clock for SystemClock {
    fn now_millis(&self) -> u64 {
        1708700000000
    }
}

struct FakeClock {
    fixed_time: u64,
}

impl Clock for FakeClock {
    fn now_millis(&self) -> u64 {
        self.fixed_time
    }
}

struct SandboxedModule<C: Clock> {
    clock: C,
    name: String,
}

impl<C: Clock> SandboxedModule<C> {
    fn new(name: String, clock: C) -> Self {
        SandboxedModule { clock, name }
    }

    fn elapsed_since(&self, start: u64) -> u64 {
        self.clock.now_millis() - start
    }
}

fn main() {
    // Correct: explicitly provide the Clock capability
    let module = SandboxedModule::new(
        String::from("production-module"),
        SystemClock,
    );
    println!("Elapsed: {}ms", module.elapsed_since(1708699999000));

    // In tests, inject a fake clock for deterministic behavior
    let test_module = SandboxedModule::new(
        String::from("test-module"),
        FakeClock { fixed_time: 5000 },
    );
    println!("Test elapsed: {}ms", test_module.elapsed_since(3000));
}
```

## Explanation

The broken version tries to construct a `SandboxedModule` with a missing `clock` field. The compiler rejects this because all fields of a struct must be initialized. You cannot create a sandboxed module without providing its required capability.

**Why this models WASM correctly:**

In WASM, a module declares its *imports* -- functions it requires from the host. If the host does not provide all declared imports, the module cannot be instantiated. It is a link-time error, not a runtime error. The module never starts executing.

In our Rust simulation:

- The `Clock` trait represents a WASM import (a capability the module needs)
- The generic parameter `C: Clock` forces the module creator to provide an implementation
- Missing the capability is a *compile error*, which is even stronger than WASM's link-time error

**Determinism through injection:**

The `FakeClock` in the correct version demonstrates why this matters. If the module directly called `SystemTime::now()`, every execution would produce different results. By injecting the clock, we can:

1. Test with fixed time values (deterministic tests)
2. Replay executions with the same inputs (debugging)
3. Run the same module in different environments (browser, server, embedded)

This is the fundamental principle of WASM sandboxing: the module is pure computation, and the host controls all interactions with the outside world.

**Key insight:** WASM is closer to a process than a library. A library shares your address space and can call any function. A WASM module has its own memory and can only call functions the host explicitly provides. The type system in Rust, through generics and trait bounds, can model this same constraint.

The invariant violated in the broken code: **a sandboxed module must receive all its capabilities explicitly; it cannot be constructed with missing dependencies.**

## ⚠️ Caution

- WASM is deterministic only if all imports are deterministic. Importing host functions like `random()` or `clock()` introduces non-determinism. Design sandbox APIs carefully.
- A sandbox that provides too many capabilities is no longer a sandbox. Follow the principle of least privilege.

## 💡 Tips

- Model WASM sandbox capabilities as Rust trait bounds — functions can only use what is injected.
- Use fake/mock implementations for testing deterministic behavior.
- Capability-based design maps naturally to Rust's type system.

## Compiler Error Interpretation

```
error[E0063]: missing field `clock` in initializer of
              `SandboxedModule<_>`
  --> src/main.rs:35:47
   |
35 |     let broken_module: SandboxedModule<_> = SandboxedModule {
   |                                             ^^^^^^^^^^^^^^^ missing `clock`
```

The compiler error tells you:

1. **"missing field `clock` in initializer"** -- you are constructing a `SandboxedModule` but did not provide the `clock` field. Every field must be initialized.
2. **`SandboxedModule<_>`** -- the compiler knows the struct is generic over `C: Clock`, but since no `clock` value was provided, it cannot even infer the type parameter.

This maps directly to WASM: if a module declares `(import "env" "clock_gettime" (func ...))` but the host does not provide that import, instantiation fails with a link error. The module cannot pretend the capability does not exist. The host must provide it, or the module does not run.
