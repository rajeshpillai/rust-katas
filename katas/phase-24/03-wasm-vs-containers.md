---
id: wasm-vs-containers
phase: 24
phase_title: "Advanced WASM Systems Patterns"
sequence: 3
title: WASM vs Containers — Isolation Through Type Boundaries
hints:
  - Containers check permissions at runtime (can be bypassed by bugs or exploits)
  - WASM checks capabilities at load time (structurally impossible to bypass)
  - If a function is not in the module's imports, the code that calls it cannot compile
  - Type-level restrictions are enforced by the compiler, not a runtime guard
---

## Description

Containers isolate processes using OS-level namespaces -- a runtime mechanism. WASM isolates modules through the type system and import mechanism -- a structural mechanism. A container with a misconfigured seccomp profile can still call dangerous syscalls. A WASM module without a `fd_write` import physically cannot write to files -- the function does not exist in its address space. This kata demonstrates both models and shows why type-level isolation (WASM-style) catches errors at compile time while runtime isolation (container-style) only catches them during execution.

## Broken Code

```rust
use std::collections::HashSet;

/// Container-style isolation: runtime permission checks
struct Container {
    name: String,
    permissions: HashSet<String>,
}

impl Container {
    fn new(name: &str, permissions: &[&str]) -> Self {
        Container {
            name: name.to_string(),
            permissions: permissions.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn exec(&self, command: &str) -> Result<String, String> {
        // Determine required permission based on command
        let required = if command.starts_with("read") {
            "filesystem"
        } else if command.starts_with("connect") {
            "network"
        } else if command.starts_with("kill") || command.starts_with("rm") {
            "dangerous"
        } else {
            "basic"
        };

        // BUG: Runtime check -- only catches errors when the code path is hit
        if !self.permissions.contains(required) {
            panic!(
                "SECURITY: Container '{}' attempted '{}' without '{}' permission!",
                self.name, command, required
            );
        }

        Ok(format!("[{}] Executed: {}", self.name, command))
    }
}

fn main() {
    let container = Container::new("web-app", &["filesystem", "basic"]);

    // These work
    println!("{}", container.exec("read /app/config").unwrap());
    println!("{}", container.exec("basic echo hello").unwrap());

    // This panics at runtime -- no "dangerous" permission
    println!("{}", container.exec("rm -rf /").unwrap());
}
```

## Correct Code

```rust
use std::marker::PhantomData;

// Capability tokens
struct Filesystem;
struct Network;
struct Basic;

// The module can only do what its type allows.
// No permission checks needed -- unauthorized methods do not exist.
struct WasmModule<Cap> {
    name: String,
    _cap: PhantomData<Cap>,
}

impl<Cap> WasmModule<Cap> {
    fn name(&self) -> &str {
        &self.name
    }
}

// Basic operations -- available to all modules
impl WasmModule<Basic> {
    fn new_basic(name: &str) -> Self {
        WasmModule { name: name.to_string(), _cap: PhantomData }
    }

    fn echo(&self, msg: &str) -> String {
        format!("[{}] {}", self.name, msg)
    }
}

// Filesystem operations -- only for modules with Filesystem capability
impl WasmModule<Filesystem> {
    fn new_fs(name: &str) -> Self {
        WasmModule { name: name.to_string(), _cap: PhantomData }
    }

    fn read_file(&self, path: &str) -> String {
        format!("[{}] Read: {}", self.name, path)
    }
}

// Network operations -- only for modules with Network capability
impl WasmModule<Network> {
    fn new_net(name: &str) -> Self {
        WasmModule { name: name.to_string(), _cap: PhantomData }
    }

    fn connect(&self, url: &str) -> String {
        format!("[{}] Connected to: {}", self.name, url)
    }
}

// No "dangerous" capability exists.
// There is no type parameter that would give access to rm or kill.
// The operations physically do not exist in the API.

fn main() {
    // A filesystem-only module
    let fs_module = WasmModule::<Filesystem>::new_fs("data-reader");
    println!("{}", fs_module.read_file("/app/config.toml"));
    // fs_module.connect("..."); // Would be a COMPILE ERROR
    // There is no rm, no kill, no dangerous operations at all.

    // A network-only module
    let net_module = WasmModule::<Network>::new_net("api-client");
    println!("{}", net_module.connect("https://api.example.com"));
    // net_module.read_file("..."); // Would be a COMPILE ERROR

    // A basic module -- can only echo
    let basic_module = WasmModule::<Basic>::new_basic("logger");
    println!("{}", basic_module.echo("Hello, isolated world!"));

    // No module of any type has "rm -rf /" available.
    // The operation does not exist in the type system.
    // It is not denied -- it is absent.

    println!("\nAll operations completed with type-level isolation!");
    println!("Dangerous operations are structurally impossible, not just forbidden.");
}
```

## Explanation

The broken version uses a `HashSet<String>` to track container permissions and checks them at runtime. The `rm -rf /` command triggers a panic because the container lacks the "dangerous" permission. But this protection depends entirely on the runtime check being present and correct.

**Container isolation (runtime):**

Containers use Linux namespaces, cgroups, and seccomp to restrict what processes can do. These are runtime mechanisms:
- A process calls `write()` → the kernel checks seccomp rules → allows or blocks
- The process compiled and loaded fine; the restriction happens during execution
- A misconfigured seccomp profile, a kernel bug, or a container escape can bypass the check

**WASM isolation (structural):**

WASM modules declare what functions they need (imports). If the host does not provide an import, the module cannot call it:
- The module tries to call `fd_write` → the function is not in the import table → validation error at load time
- The module never executes a single instruction
- There is nothing to bypass because the function does not exist in the module's world

**The Rust type-system equivalent:**

In the correct version, `WasmModule<Filesystem>` has `read_file` but not `connect`. There is no runtime check for "does this module have network access?" The method simply does not exist for that type. Trying to call it is a compile error, not a runtime error.

**Why structural isolation is stronger:**

| Attack Vector | Container | WASM |
|--------------|-----------|------|
| Bypass permission check | Possible (kernel bug, config error) | Impossible (no check to bypass) |
| Confused deputy | Possible (wrong permission string) | Impossible (type mismatch) |
| TOCTOU race | Possible (check-then-use gap) | Impossible (no gap -- compile time) |
| Privilege escalation | Possible (exploit runtime) | Much harder (must break type system) |

The invariant violated in the broken code: **security should be structural (unauthorized operations do not exist) rather than checked (operations exist but are guarded by runtime conditions).**

## Compiler Error Interpretation

```
thread 'main' panicked at 'SECURITY: Container 'web-app' attempted
  'rm -rf /' without 'dangerous' permission!', src/main.rs:31:13
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

This is a runtime panic from the permission check:

1. **"Container 'web-app' attempted 'rm -rf /'"** -- the container tried to execute a dangerous command.
2. **"without 'dangerous' permission"** -- the runtime check caught the unauthorized access.

But consider: this check only ran because the code path was exercised. In production, the `rm -rf /` path might only trigger under rare conditions -- it could pass all tests and code reviews, then trigger in production.

In the type-level version, the equivalent scenario produces a compile error:
```
error[E0599]: no method named `dangerous_operation` found for struct
              `WasmModule<Filesystem>` in the current scope
```

This is caught during compilation, long before the code can ever run in production.
