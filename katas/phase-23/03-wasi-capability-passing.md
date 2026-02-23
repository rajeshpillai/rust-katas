---
id: wasi-capability-passing
phase: 23
phase_title: "WASM Outside the Browser"
sequence: 3
title: WASI-Style Capability Passing — File Descriptors as Handles
hints:
  - WASI does not give guests an open() syscall for arbitrary paths
  - The host pre-opens directories and passes file descriptors to the guest
  - The guest can only access files through descriptors the host provided
  - Authority comes from possession of a handle, not from naming a resource
---

## Description

WASI (WebAssembly System Interface) uses a capability-based model. Unlike POSIX, where any process can call `open("/etc/passwd")` if it has file permissions, WASI guests have no `open()` syscall at all. Instead, the host pre-opens specific directories and passes file descriptors to the guest. The guest can only read and write through these provided descriptors. This kata demonstrates the difference between ambient authority (directly accessing files by name) and capability-based authority (accessing only what was explicitly granted).

## Broken Code

```rust
use std::collections::HashMap;

struct WasiGuest {
    allowed_paths: Vec<String>,
}

impl WasiGuest {
    fn new(allowed_paths: Vec<String>) -> Self {
        WasiGuest { allowed_paths }
    }

    fn read_file(&self, path: &str) -> String {
        // BUG: Directly accessing the filesystem by path name.
        // This is ambient authority -- the guest decides what to access.
        // The "allowed_paths" check happens AFTER attempting to read.
        let content = std::fs::read_to_string(path).unwrap();

        // Post-check: too late, the file was already read!
        if !self.allowed_paths.iter().any(|p| path.starts_with(p)) {
            panic!("Access denied: {} is not in allowed paths", path);
        }

        content
    }
}

fn main() {
    let guest = WasiGuest::new(vec!["/tmp/safe/".to_string()]);

    // Try to read a file that does not exist and is not in allowed paths
    let content = guest.read_file("/etc/shadow");
    println!("Content: {}", content);
}
```

## Correct Code

```rust
use std::collections::HashMap;

#[derive(Debug)]
enum WasiError {
    BadDescriptor(u32),
    NotFound(String),
    PermissionDenied,
}

/// A pre-opened file with its content (simulating a file descriptor).
struct OpenFile {
    path: String,
    content: String,
}

/// WASI-style guest: can only access files through provided descriptors.
struct WasiGuest {
    file_table: HashMap<u32, OpenFile>,
}

impl WasiGuest {
    fn new() -> Self {
        WasiGuest {
            file_table: HashMap::new(),
        }
    }

    /// The HOST provides pre-opened file descriptors.
    /// The guest has no way to create descriptors on its own.
    fn provide_fd(&mut self, fd: u32, path: &str, content: &str) {
        self.file_table.insert(fd, OpenFile {
            path: path.to_string(),
            content: content.to_string(),
        });
    }

    /// Read from a file descriptor. The guest cannot specify a path --
    /// it can only use descriptors the host provided.
    fn fd_read(&self, fd: u32) -> Result<&str, WasiError> {
        match self.file_table.get(&fd) {
            Some(file) => Ok(&file.content),
            None => Err(WasiError::BadDescriptor(fd)),
        }
    }

    /// List available file descriptors (for debugging).
    fn list_fds(&self) -> Vec<(u32, &str)> {
        let mut fds: Vec<_> = self.file_table.iter()
            .map(|(&fd, file)| (fd, file.path.as_str()))
            .collect();
        fds.sort_by_key(|&(fd, _)| fd);
        fds
    }
}

fn main() {
    let mut guest = WasiGuest::new();

    // Host pre-opens specific files and gives the guest descriptors
    guest.provide_fd(3, "/app/config.toml", "port = 8080\nhost = 'localhost'");
    guest.provide_fd(4, "/app/data/input.txt", "Hello, WASI world!");

    // Guest can list what it has access to
    println!("Available file descriptors:");
    for (fd, path) in guest.list_fds() {
        println!("  fd {} -> {}", fd, path);
    }

    // Guest reads through provided descriptors
    match guest.fd_read(3) {
        Ok(content) => println!("\nConfig (fd 3):\n{}", content),
        Err(e) => println!("Error: {:?}", e),
    }

    match guest.fd_read(4) {
        Ok(content) => println!("\nData (fd 4):\n{}", content),
        Err(e) => println!("Error: {:?}", e),
    }

    // Guest cannot access anything beyond provided descriptors
    match guest.fd_read(5) {
        Ok(content) => println!("Content: {}", content),
        Err(e) => println!("\nfd 5: {:?} (no access -- not provided by host)", e),
    }

    // The guest physically cannot call "open" -- the function does not exist
    // in its API. There is no method like guest.open("/etc/passwd").
    // Authority is structural: if you do not have a descriptor, you cannot read.
}
```

## Explanation

The broken version tries to read a file using `std::fs::read_to_string(path)` -- directly accessing the filesystem by path name. This is ambient authority: the program decides what to access based on a string name. The "allowed_paths" check is an afterthought that runs after the file was already read (or in this case, the read fails because `/etc/shadow` is not accessible, producing a panic on `unwrap()`).

**Ambient authority vs capability-based security:**

| Aspect | Ambient (POSIX) | Capability (WASI) |
|--------|-----------------|-------------------|
| Access method | Name a path | Use a provided descriptor |
| Who decides | The guest | The host |
| Default access | Everything (modulo permissions) | Nothing |
| Restriction | Deny list (block some paths) | Allow list (grant specific descriptors) |
| Bypass risk | Guest can try any path | Guest cannot name paths at all |

**How WASI works:**

1. The host decides which directories/files the guest can access
2. The host pre-opens these and passes file descriptors (integers) to the guest
3. The guest calls `fd_read(fd, buf, buf_len)` with a descriptor it was given
4. If the guest tries to use a descriptor it was not given, `fd_read` returns `EBADF` (bad file descriptor)

There is no `path_open` equivalent that takes an arbitrary path. The guest literally cannot construct a call to access `/etc/shadow` because the WASI API does not accept path strings for file access.

**The structural advantage:**

In the broken (ambient) model, security depends on correctly implementing the deny list. One missed path and the guest can read sensitive files. In the capability model, security is structural: if you did not give the guest a descriptor, it cannot access the resource. The guest would have to find a bug in the runtime itself to escape the sandbox.

The invariant violated in the broken code: **in capability-based systems, authority comes from possession of a handle, not from naming a resource; the guest should not be able to specify file paths.**

## Compiler Error Interpretation

```
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value:
  Os { code: 2, kind: NotFound, message: "No such file or directory" }',
  src/main.rs:15:57
```

This is a runtime panic from the direct filesystem access:

1. **"No such file or directory"** -- `std::fs::read_to_string("/etc/shadow")` failed because the file either does not exist or is not accessible with current permissions.
2. **"called `Result::unwrap()` on an `Err` value"** -- the code assumed the file read would succeed and panicked when it did not.

But notice: the error is "not found", not "access denied". The code attempted to read a sensitive system file -- the only thing that stopped it was filesystem permissions. In the capability model, the code would not have been able to attempt the read at all. The path `/etc/shadow` is not a valid argument because the API does not accept paths.
