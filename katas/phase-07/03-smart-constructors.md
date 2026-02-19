---
id: smart-constructors
phase: 7
phase_title: "Data Modeling the Rust Way"
sequence: 3
title: Smart Constructors for Invariant Enforcement
hints:
  - "If a struct's fields are public, anyone can construct it with invalid data."
  - "Make fields private and provide a `fn new()` that validates the data before construction."
  - "Return `Result` from the constructor so the caller must handle the validation failure."
---

## Description

A smart constructor is a function that validates data before creating a value. By making a struct's fields private and only allowing construction through a validating function, you guarantee that **every instance of the type satisfies your invariants**.

This is a fundamental principle: if invalid data cannot be created, you never need to check for it downstream. Every function that receives the type can trust that it is valid.

## Broken Code

```rust
struct Email {
    // Public fields: anyone can construct an Email with any string.
    pub address: String,
}

struct Age {
    pub value: i32,
}

fn send_welcome(email: &Email, age: &Age) {
    // We have to validate here because we cannot trust the data.
    // But what if another function forgets to validate?
    if !email.address.contains('@') {
        panic!("Invalid email!"); // Panic in library code — bad practice
    }
    if age.value < 0 || age.value > 150 {
        panic!("Invalid age!"); // Panic in library code — bad practice
    }
    println!(
        "Welcome! Email: {}, Age: {}",
        email.address, age.value
    );
}

fn main() {
    // Anyone can create invalid instances — the type provides no guarantees.
    let bad_email = Email {
        address: String::from("not-an-email"),
    };
    let bad_age = Age { value: -5 };

    // This will panic at runtime instead of failing at compile time.
    send_welcome(&bad_email, &bad_age);
}
```

## Correct Code

```rust
use std::fmt;

#[derive(Debug, Clone)]
struct Email {
    // Private field: cannot be constructed outside this module.
    address: String,
}

#[derive(Debug)]
enum EmailError {
    Empty,
    MissingAtSign,
    MissingDomain,
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmailError::Empty => write!(f, "email address cannot be empty"),
            EmailError::MissingAtSign => write!(f, "email address must contain '@'"),
            EmailError::MissingDomain => write!(f, "email address must have a domain after '@'"),
        }
    }
}

impl Email {
    // Smart constructor: validates the input and returns Result.
    fn new(address: &str) -> Result<Self, EmailError> {
        if address.is_empty() {
            return Err(EmailError::Empty);
        }
        let at_pos = address.find('@').ok_or(EmailError::MissingAtSign)?;
        if at_pos + 1 >= address.len() {
            return Err(EmailError::MissingDomain);
        }
        Ok(Email {
            address: address.to_string(),
        })
    }

    // Provide read-only access to the inner value.
    fn as_str(&self) -> &str {
        &self.address
    }
}

#[derive(Debug, Clone, Copy)]
struct Age {
    value: u8, // Private field. u8 range is 0-255, further constrained by constructor.
}

#[derive(Debug)]
struct AgeError {
    given: i32,
}

impl fmt::Display for AgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "age must be between 0 and 150, got {}", self.given)
    }
}

impl Age {
    fn new(value: i32) -> Result<Self, AgeError> {
        if value < 0 || value > 150 {
            return Err(AgeError { given: value });
        }
        Ok(Age { value: value as u8 })
    }

    fn value(&self) -> u8 {
        self.value
    }
}

// This function does NOT need to validate — it trusts the types.
// If you have an Email, it is guaranteed to be valid.
// If you have an Age, it is guaranteed to be in range.
fn send_welcome(email: &Email, age: &Age) {
    println!(
        "Welcome! Email: {}, Age: {}",
        email.as_str(),
        age.value()
    );
}

fn main() {
    // Construction can fail — the caller handles the error.
    let email = match Email::new("alice@example.com") {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Invalid email: {}", err);
            return;
        }
    };

    let age = match Age::new(30) {
        Ok(a) => a,
        Err(err) => {
            eprintln!("Invalid age: {}", err);
            return;
        }
    };

    // No validation needed here — the types guarantee correctness.
    send_welcome(&email, &age);

    // Invalid data is caught at the point of creation:
    match Email::new("not-an-email") {
        Ok(_) => println!("Should not reach here"),
        Err(err) => println!("Caught at creation: {}", err),
    }

    match Age::new(-5) {
        Ok(_) => println!("Should not reach here"),
        Err(err) => println!("Caught at creation: {}", err),
    }
}
```

## Explanation

The broken version has public fields, which means anyone can construct an `Email` or `Age` with arbitrary data. The type provides no guarantee about validity. Every function that uses these types must validate them, and if even one function forgets, you have a bug.

The correct version uses the smart constructor pattern:

1. **Private fields**: The struct's fields are private (the default in Rust). Outside the module, you cannot write `Email { address: ... }` — the struct literal syntax is unavailable.

2. **Public `fn new()` that validates**: The only way to create an `Email` is through `Email::new()`, which checks the invariants and returns `Result<Email, EmailError>`. If validation fails, you get an error — not a panic, not an invalid value.

3. **Invariants hold everywhere**: Once you have an `Email` value, you know it contains `@` and has a domain. Every function downstream can trust this without re-checking. The validation happens once, at the boundary.

4. **Read-only access**: The `as_str()` method provides read access to the inner data without exposing the field for mutation. The caller cannot modify the address after construction and break the invariant.

**Why this matters at scale:**

In a small program, you might remember to validate everywhere. In a large codebase with many developers, relying on "everyone remembers to validate" is a recipe for bugs. Smart constructors move the guarantee from "developer discipline" to "compiler enforcement."

**The parse-don't-validate principle:**

This pattern embodies the idea of "parse, don't validate":
- **Validating** means checking data and proceeding with the same untyped representation (a `String` that you hope is a valid email).
- **Parsing** means checking data and converting it into a typed representation (`Email`) that carries the proof of validity in its type.

After parsing, you never need to validate again. The type *is* the proof.

**Combining with newtypes:**

Smart constructors work beautifully with newtypes:

```rust
struct PortNumber(u16);

impl PortNumber {
    fn new(port: u16) -> Result<Self, String> {
        if port == 0 {
            Err("port 0 is reserved".to_string())
        } else {
            Ok(PortNumber(port))
        }
    }
}
```

## Compiler Error Interpretation

If you try to construct the struct directly with private fields from outside the module:

```
error[E0423]: expected function, found struct `Email`
 --> src/main.rs:XX:XX
  |
  |     let bad_email = Email { address: String::from("bad") };
  |                     ^^^^^
  |
  = help: you might have meant to use the struct literal syntax

error[E0451]: field `address` of struct `Email` is private
 --> src/main.rs:XX:XX
  |
  |     let bad_email = Email { address: String::from("bad") };
  |                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ private field
```

This error is the smart constructor doing its job:

- **"field `address` of struct `Email` is private"** — You cannot access the field directly. This prevents constructing the struct with invalid data.
- The only way to create an `Email` is through `Email::new()`, which validates the input.

This is a compile-time guarantee. It is not possible to create an invalid `Email` — not through carelessness, not through malice, not through a new developer who does not know the rules. The type system enforces the invariant.

Note that within the same module, private fields are accessible. If your module is large, consider placing the type in its own sub-module to minimize the "trusted code" surface area.
