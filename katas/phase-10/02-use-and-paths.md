---
id: use-and-paths
phase: 10
phase_title: "Modules, Visibility & Testing"
sequence: 2
title: "use Statements and Module Paths"
hints:
  - "crate:: refers to the root of the current crate"
  - "self:: refers to the current module"
  - "super:: refers to the parent module"
  - A use statement brings a name into scope so you don't have to write the full path every time
---

## Description

Rust's module system uses paths to locate items. There are three anchor points for building paths:

- **`crate::`** starts from the root of the current crate.
- **`self::`** starts from the current module.
- **`super::`** starts from the parent module.

The `use` keyword brings items from these paths into the current scope, so you can refer to them by their short name. Getting the path wrong -- or forgetting to make parent modules and items public -- results in a "module not found" or "unresolved import" error.

## Broken Code

```rust
mod network {
    pub mod server {
        pub fn start() {
            println!("Server starting...");
            // Try to call a sibling module's function
            client::connect();
        }
    }

    pub mod client {
        pub fn connect() {
            println!("Client connecting...");
        }
    }
}

// Wrong path: trying to use 'server' as if it were at the crate root
use server::start;

fn main() {
    start();
}
```

## Correct Code

```rust
mod network {
    pub mod server {
        pub fn start() {
            println!("Server starting...");
            // Use super:: to go up to the parent (network), then into client
            super::client::connect();
        }
    }

    pub mod client {
        pub fn connect() {
            println!("Client connecting...");
        }
    }
}

// Correct path: crate root -> network -> server -> start
use crate::network::server::start;

fn main() {
    start();
}
```

## Explanation

The broken version has two path errors:

**Error 1: `use server::start`** at the crate root. The module `server` is not at the crate root -- it is nested inside `network`. The full path from the crate root is `crate::network::server::start`. Without the correct path, the compiler reports "unresolved import."

**Error 2: `client::connect()` inside `server::start()`**. From the perspective of the `server` module, `client` is not a child -- it is a sibling. The `server` module cannot see `client` directly. It must navigate up to the parent module (`network`) using `super::`, and then down into `client`. The correct path is `super::client::connect()`.

Understanding these three anchors is essential:

- **`crate::`** is always the root of the current crate. Think of it like `/` in a filesystem. `use crate::network::server::start` is an absolute path.
- **`super::`** goes up one level, like `../` in a filesystem. It is indispensable when sibling modules need to reference each other.
- **`self::`** refers to the current module. It is rarely needed explicitly, but can clarify intent: `use self::helpers::format_output`.

You can also use `use` to rename items with `as`:

```rust
use crate::network::server::start as start_server;
```

And you can group imports with curly braces:

```rust
use crate::network::{server, client};
```

A common mistake is to confuse the file-system layout with the module tree. The module tree is defined by `mod` declarations, not by file names. A file only becomes part of the module tree when a parent declares it with `mod filename;`.

## Compiler Error Interpretation

```
error[E0432]: unresolved import `server`
 --> src/main.rs:19:5
   |
19 | use server::start;
   |     ^^^^^^ use of undeclared crate or module `server`
```

The compiler cannot find a module named `server` at the crate root. It looks at the top level and sees only `network`. The fix is to provide the full path: `use crate::network::server::start`. When you see "unresolved import" or "use of undeclared crate or module," trace the module hierarchy from the crate root to the item you want and ensure every segment of the path matches a real, public module.

```
error[E0433]: failed to resolve: use of undeclared crate or module `client`
 --> src/main.rs:5:13
  |
5 |             client::connect();
  |             ^^^^^^ use of undeclared crate or module `client`
```

Inside `server`, there is no child module called `client`. The compiler only searches the current module's children. Use `super::client::connect()` to navigate to the sibling module through the parent.
