---
id: module-visibility
phase: 10
phase_title: "Modules, Visibility & Testing"
sequence: 1
title: "Modules and Visibility Rules"
hints:
  - In Rust, everything is private by default
  - The pub keyword makes an item visible outside its module
  - pub(crate) makes an item visible anywhere within the same crate but not to external users
  - A struct can be pub but its fields remain private unless individually marked pub
---

## Description

Rust's module system enforces encapsulation at the language level. By default, every function, struct, enum, constant, and module is **private** -- visible only within the module where it is defined (and its child modules). To make something accessible from outside its module, you must explicitly mark it with `pub`.

This is not merely a convention -- the compiler enforces it. If you try to access a private item from outside its module, the code will not compile. This forces you to think about your public API surface and prevents accidental coupling to internal implementation details.

## Broken Code

```rust
mod database {
    struct Connection {
        url: String,
        connected: bool,
    }

    fn connect(url: &str) -> Connection {
        Connection {
            url: String::from(url),
            connected: true,
        }
    }

    fn is_healthy(conn: &Connection) -> bool {
        conn.connected
    }
}

fn main() {
    let conn = database::connect("postgres://localhost/mydb");

    if database::is_healthy(&conn) {
        println!("Database is healthy!");
    }

    println!("Connected to: {}", conn.url);
}
```

## Correct Code

```rust
mod database {
    pub struct Connection {
        url: String,         // private: callers don't need direct access
        connected: bool,     // private: internal state
    }

    impl Connection {
        pub fn url(&self) -> &str {
            &self.url
        }
    }

    pub fn connect(url: &str) -> Connection {
        Connection {
            url: String::from(url),
            connected: true,
        }
    }

    pub fn is_healthy(conn: &Connection) -> bool {
        conn.connected
    }
}

fn main() {
    let conn = database::connect("postgres://localhost/mydb");

    if database::is_healthy(&conn) {
        println!("Database is healthy!");
    }

    // Access through the public method, not the private field
    println!("Connected to: {}", conn.url());
}
```

## Explanation

The broken version has three visibility problems:

1. **`connect` is private.** The function `connect` is defined inside `mod database` but is not marked `pub`. Only code within the `database` module (and its children) can call it. `main()` is outside the module, so `database::connect(...)` fails.

2. **`is_healthy` is private.** Same issue. The function needs `pub` to be callable from `main()`.

3. **`Connection` is private, and so are its fields.** Even if we made the struct `pub`, writing `conn.url` would still fail because individual struct fields default to private. This is intentional: a `pub struct` with private fields is a common pattern in Rust. It means external code can hold and pass around a `Connection`, but cannot construct one directly or access its internals.

The correct version makes `connect` and `is_healthy` public with `pub`, and makes the `Connection` struct public too. But it deliberately keeps the fields private. Instead, it provides a `pub fn url(&self) -> &str` method as a controlled accessor. This preserves encapsulation: the module author can change the internal representation without breaking callers.

Rust also offers `pub(crate)`, which makes an item visible throughout the entire crate but not to external consumers. This is useful for items that need to be shared across modules within a library but should not appear in the public API:

```rust
pub(crate) fn internal_helper() { /* ... */ }
```

Other visibility modifiers include `pub(super)` (visible to the parent module) and `pub(in path)` (visible to a specific ancestor module).

The key design principle: **make the minimal surface area public.** Every `pub` item is a commitment -- changing it later may break downstream code.

## Compiler Error Interpretation

```
error[E0603]: function `connect` is private
 --> src/main.rs:20:26
   |
20 |     let conn = database::connect("postgres://localhost/mydb");
   |                          ^^^^^^^ private function
   |
note: the function `connect` is defined here
 --> src/main.rs:8:5
   |
8  |     fn connect(url: &str) -> Connection {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

The compiler tells you exactly which item is private and where it is defined. The fix is to add `pub` before the item's definition. When you see "is private," check whether the item should truly be public. If it should, add `pub`. If it should not, reconsider your design -- maybe you need a public wrapper function that provides controlled access to the private internals.
