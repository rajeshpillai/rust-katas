---
id: type-state-pattern
phase: 16
phase_title: "Advanced Systems Patterns"
sequence: 3
title: Type-State Pattern — Encoding State in the Type System
hints:
  - The type-state pattern uses different types (or generic parameters) to represent different states
  - Methods are only available on the state where they make sense
  - Transitioning between states consumes the old value and returns a new one of a different type
  - PhantomData lets you use a generic parameter without storing data of that type
---

## Description

The type-state pattern encodes an object's state in its type, so that invalid operations are compile-time errors instead of runtime bugs. For example, a connection that has not been authenticated should not be able to send data. Instead of checking a boolean at runtime, you make "unauthenticated" and "authenticated" different types. Methods like `send()` only exist on the authenticated type. If you try to call `send()` on an unauthenticated connection, the code does not compile. This pattern uses Rust's generics and zero-sized types to make illegal states unrepresentable.

## Broken Code

```rust
struct Connection {
    address: String,
    authenticated: bool,
}

impl Connection {
    fn new(address: &str) -> Self {
        Connection {
            address: address.to_string(),
            authenticated: false,
        }
    }

    fn authenticate(&mut self, _credentials: &str) {
        self.authenticated = true;
    }

    fn send(&self, message: &str) {
        // BUG: Runtime check that could be forgotten or bypassed.
        // Nothing stops a caller from removing or ignoring this check.
        if !self.authenticated {
            panic!("Cannot send on unauthenticated connection!");
        }
        println!("Sending to {}: {}", self.address, message);
    }
}

fn main() {
    let conn = Connection::new("example.com");

    // BUG: Forgot to call authenticate()!
    // This compiles fine but panics at runtime.
    conn.send("hello");
}
```

## Correct Code

```rust
use std::marker::PhantomData;

// State marker types -- zero-sized, exist only in the type system
struct Disconnected;
struct Connected;
struct Authenticated;

// The Connection type is generic over its state.
// PhantomData<S> makes the compiler treat Connection<S> as if
// it contains an S, without actually storing one (zero cost).
struct Connection<S> {
    address: String,
    _state: PhantomData<S>,
}

// Methods available ONLY in the Disconnected state
impl Connection<Disconnected> {
    fn new(address: &str) -> Self {
        Connection {
            address: address.to_string(),
            _state: PhantomData,
        }
    }

    // connect() consumes the Disconnected connection
    // and returns a Connected connection.
    fn connect(self) -> Connection<Connected> {
        println!("Connecting to {}...", self.address);
        Connection {
            address: self.address,
            _state: PhantomData,
        }
    }
}

// Methods available ONLY in the Connected state
impl Connection<Connected> {
    // authenticate() consumes the Connected connection
    // and returns an Authenticated connection.
    fn authenticate(self, _credentials: &str) -> Connection<Authenticated> {
        println!("Authenticating...");
        Connection {
            address: self.address,
            _state: PhantomData,
        }
    }
}

// Methods available ONLY in the Authenticated state
impl Connection<Authenticated> {
    fn send(&self, message: &str) {
        // No runtime check needed!
        // If this method is callable, the connection IS authenticated.
        println!("Sending to {}: {}", self.address, message);
    }
}

fn main() {
    let conn = Connection::new("example.com");

    // Each step transitions to a new type:
    // Connection<Disconnected> -> Connection<Connected> -> Connection<Authenticated>
    let conn = conn.connect();
    let conn = conn.authenticate("secret");

    // send() is only available on Connection<Authenticated>
    conn.send("hello");
}
```

## Explanation

The broken version uses a runtime boolean (`authenticated: bool`) to track state. This has several problems:

1. **The check can be forgotten.** A developer writing `send()` might forget to check `self.authenticated`. The compiler will not warn them.
2. **The check can be bypassed.** The `authenticated` field is part of the struct; any code with `&mut Connection` can set it to `true` without actually authenticating.
3. **Errors are discovered at runtime.** The program compiles successfully and only panics when the invalid path is exercised. In testing, this path might never be hit.

The correct version eliminates all three problems by encoding state in the type system.

**How the type-state pattern works:**

1. **State marker types**: `Disconnected`, `Connected`, and `Authenticated` are empty structs (zero-sized types). They exist only to carry information in the type system. They take no memory at runtime.

2. **Generic state parameter**: `Connection<S>` is generic over its state. `Connection<Disconnected>` and `Connection<Authenticated>` are different types at compile time, even though they have the same runtime representation.

3. **State-specific methods**: `send()` is implemented only for `Connection<Authenticated>`. It does not exist for `Connection<Disconnected>` or `Connection<Connected>`. Calling `conn.send("hello")` on an unauthenticated connection is a type error, not a runtime error.

4. **Consuming transitions**: `connect()` takes `self` (not `&self` or `&mut self`), consuming the `Connection<Disconnected>` and returning a `Connection<Connected>`. The old value is gone -- you cannot accidentally use a disconnected connection after connecting.

5. **PhantomData**: `PhantomData<S>` is a zero-sized type that tells the compiler "this struct is parameterized over S" without actually storing an S. It affects type checking but has zero runtime cost.

**Why this is a systems pattern:**

The type-state pattern is used in real systems for:
- Network protocol state machines (TCP: SYN_SENT, ESTABLISHED, etc.)
- Builder patterns (ensuring required fields are set before build)
- File handles (opened, locked, closed)
- Cryptographic operations (key generated, encrypted, signed)

The cost is zero at runtime -- the state is erased during compilation. The benefit is that invalid state transitions are compile errors.

The invariant violated in the broken code: **runtime state checks can be forgotten or bypassed; encoding state in the type system makes invalid operations unrepresentable.**

## Compiler Error Interpretation

If you try to call `send()` on the wrong state in the correct version:

```rust
fn main() {
    let conn = Connection::new("example.com");
    conn.send("hello"); // Connection<Disconnected> has no method send()
}
```

You get:

```
error[E0599]: no method named `send` found for struct
              `Connection<Disconnected>` in the current scope
  --> src/main.rs:58:10
   |
11 | struct Connection<S> {
   | -------------------- method `send` not found for this struct
...
58 |     conn.send("hello");
   |          ^^^^ method not found in `Connection<Disconnected>`
   |
   = note: the method was found for
           - `Connection<Authenticated>`
```

The compiler error is remarkably clear:

1. **"no method named `send` found for struct `Connection<Disconnected>`"** -- `send()` does not exist on `Connection<Disconnected>`. The method literally does not exist for this type.
2. **"the method was found for `Connection<Authenticated>`"** -- the compiler tells you which type *does* have the method. This is a roadmap: you need to transition from `Disconnected` to `Authenticated` first.

This is what "making illegal states unrepresentable" means in practice. The programmer does not need to remember to check a boolean. The compiler enforces the state machine. If you try to skip a step, the code does not compile -- and the error message tells you exactly what step you missed.
