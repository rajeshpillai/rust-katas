---
id: capability-based-security
phase: 24
phase_title: "Advanced WASM Systems Patterns"
sequence: 1
title: Capability-Based Security — Authority Through Handles
hints:
  - Runtime permission checks can be bypassed if the check is forgotten
  - Type-level capabilities make unauthorized access a compile error, not a runtime error
  - Zero-sized types (like struct CanRead;) carry no runtime cost
  - If the capability is not in the type parameter, the code cannot compile
---

## Description

In capability-based systems, you can only do what your handles allow. There is no ambient authority, no superuser mode. WASM enforces this structurally: a module without a `fd_write` import physically cannot write to files. This kata contrasts runtime permission checking (fragile, easy to forget) with type-level capability enforcement (the Rust compiler rejects unauthorized access at compile time).

## Broken Code

```rust
use std::collections::HashSet;

struct Module {
    name: String,
    capabilities: HashSet<String>,
}

impl Module {
    fn new(name: &str, caps: &[&str]) -> Self {
        Module {
            name: name.to_string(),
            capabilities: caps.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn read_file(&self, path: &str) -> String {
        // Runtime check -- can be forgotten, can be bypassed
        if !self.capabilities.contains("filesystem") {
            panic!("Permission denied: {} lacks 'filesystem' capability", self.name);
        }
        format!("Content of {}", path)
    }

    fn send_network(&self, url: &str) -> String {
        if !self.capabilities.contains("network") {
            panic!("Permission denied: {} lacks 'network' capability", self.name);
        }
        format!("Response from {}", url)
    }
}

fn main() {
    // Module with only filesystem access
    let module = Module::new("data-processor", &["filesystem"]);

    // This works -- has filesystem capability
    println!("{}", module.read_file("/data/input.csv"));

    // This panics at runtime -- no network capability
    println!("{}", module.send_network("https://api.example.com"));
}
```

## Correct Code

```rust
use std::marker::PhantomData;

// Zero-sized capability tokens
struct CanReadFiles;
struct CanNetwork;
struct CanAllocate;

// Trait to mark types as capabilities
trait Capability {}
impl Capability for CanReadFiles {}
impl Capability for CanNetwork {}
impl Capability for CanAllocate {}

// A module that holds specific capabilities in its type
struct Module<C> {
    name: String,
    _cap: PhantomData<C>,
}

impl<C> Module<C> {
    fn new(name: &str) -> Self {
        Module {
            name: name.to_string(),
            _cap: PhantomData,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// File operations require CanReadFiles capability
impl Module<CanReadFiles> {
    fn read_file(&self, path: &str) -> String {
        format!("[{}] Read: {}", self.name, path)
    }
}

// Network operations require CanNetwork capability
impl Module<CanNetwork> {
    fn send_network(&self, url: &str) -> String {
        format!("[{}] Fetched: {}", self.name, url)
    }
}

// Modules with combined capabilities (using tuples)
impl Module<(CanReadFiles, CanNetwork)> {
    fn read_file(&self, path: &str) -> String {
        format!("[{}] Read: {}", self.name, path)
    }

    fn send_network(&self, url: &str) -> String {
        format!("[{}] Fetched: {}", self.name, url)
    }
}

fn main() {
    // Module with only filesystem access
    let fs_module: Module<CanReadFiles> = Module::new("data-processor");
    println!("{}", fs_module.read_file("/data/input.csv"));
    // fs_module.send_network("...") would be a COMPILE ERROR -- method does not exist

    // Module with only network access
    let net_module: Module<CanNetwork> = Module::new("api-client");
    println!("{}", net_module.send_network("https://api.example.com"));
    // net_module.read_file("...") would be a COMPILE ERROR

    // Module with both capabilities
    let full_module: Module<(CanReadFiles, CanNetwork)> = Module::new("full-access");
    println!("{}", full_module.read_file("/data/config.toml"));
    println!("{}", full_module.send_network("https://api.example.com"));

    println!("\nAll operations completed with compile-time capability enforcement!");
}
```

## Explanation

The broken version uses runtime capability checks: a `HashSet<String>` stores the module's permissions, and each operation checks the set before proceeding. If the check is missing or the string is misspelled, the unauthorized operation succeeds silently. The check only triggers at runtime, and only if the specific code path is executed.

**Runtime checks vs compile-time enforcement:**

| Aspect | Runtime (HashSet) | Compile-time (Type system) |
|--------|-------------------|---------------------------|
| Enforcement | Must remember to check | Automatic -- no method exists |
| Bypass risk | Forget a check, misspell a string | Impossible without changing types |
| Discovery | In production, under load | During compilation, before deployment |
| Performance | HashMap lookup per operation | Zero cost (types erased at runtime) |

**How WASM enforces capabilities structurally:**

In WASM, if the host does not provide a `fd_write` import, the module does not have the function. Period. It is not that calling `fd_write` returns "permission denied" -- the function does not exist in the module's function table. Trying to call a nonexistent function is a validation error caught at load time.

The type-level approach in Rust mirrors this: if `Module<CanReadFiles>` does not have `send_network()` in its impl block, calling it is a compile error. The method does not exist for that type. This is the Rust equivalent of "the import was not provided."

**Zero-sized types (ZSTs):**

`CanReadFiles`, `CanNetwork`, and `CanAllocate` are zero-sized types. They take up no memory at runtime. `PhantomData<C>` also takes up no memory. The capability enforcement is entirely at the type level -- it costs nothing at runtime.

The invariant violated in the broken code: **security capabilities should be enforced structurally (through the type system), not through runtime checks that can be forgotten or bypassed.**

## Compiler Error Interpretation

```
thread 'main' panicked at 'Permission denied: data-processor lacks
  'network' capability', src/main.rs:24:13
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

This is a runtime panic from the permission check:

1. **"data-processor lacks 'network' capability"** -- the module was created with only `["filesystem"]`, and the network check caught the unauthorized access.
2. But this check only works because someone remembered to write it. If the `send_network` method forgot the check, the operation would succeed without the capability.

In the type-level version, unauthorized access is a compile error:
```
error[E0599]: no method named `send_network` found for struct
              `Module<CanReadFiles>` in the current scope
```

This error is found during compilation, not during testing, not in production. The unauthorized code path cannot exist in the compiled binary.
