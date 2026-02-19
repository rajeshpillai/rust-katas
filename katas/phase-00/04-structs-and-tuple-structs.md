---
id: structs-and-tuple-structs
phase: 0
phase_title: Rust as a Language
sequence: 4
title: Structs and Tuple Structs
hints:
  - Every field must be provided when constructing a struct
  - Rust does not have default values for struct fields
  - Tuple structs use positional fields instead of named ones
---

## Description

Structs are Rust's primary way to group related data together. A regular struct has named fields, while a tuple struct has positional fields. When constructing a struct, you must provide every field — Rust does not have default values, and it will not silently initialize missing fields to zero or null.

## Broken Code

```rust
struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

struct Meters(f64);

fn main() {
    let sky = Color {
        red: 135,
        blue: 235,
    };

    let distance = Meters(100.0);

    println!("Sky color: ({}, {}, {})", sky.red, sky.green, sky.blue);
    println!("Distance: {} meters", distance.0);
}
```

## Correct Code

```rust
struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

struct Meters(f64);

fn main() {
    let sky = Color {
        red: 135,
        green: 206,
        blue: 235,
    };

    let distance = Meters(100.0);

    println!("Sky color: ({}, {}, {})", sky.red, sky.green, sky.blue);
    println!("Distance: {} meters", distance.0);
}
```

## Explanation

When constructing a struct in Rust, you must supply a value for every single field. In the broken version, the `green` field is missing from the `Color` construction. Unlike languages such as Go (which zero-initialize missing fields) or JavaScript (which sets them to `undefined`), Rust requires explicitness. Every field must be deliberately initialized.

This is a safety guarantee: you can never have a struct with an uninitialized field. If you want a default value, you can implement the `Default` trait, but you must still call it explicitly (e.g., `Color { red: 135, ..Default::default() }`).

Tuple structs like `Meters(f64)` work the same way but use positional access (`.0`, `.1`, etc.) instead of named fields. They are especially useful for the newtype pattern — wrapping a single value to give it a distinct type. `Meters(100.0)` and `f64` are different types, even though the underlying data is the same.

## Compiler Error Interpretation

```
error[E0063]: missing field `green` in initializer of `Color`
 --> main.rs:11:15
  |
11 |     let sky = Color {
  |               ^^^^^ missing `green`
```

The compiler tells you exactly which field is missing. Error code E0063 means you are constructing a struct but have not provided all required fields. The fix is straightforward: add the missing field. Rust treats this as a hard error because a partially initialized struct would be unsound — every field must have a known, valid value.
