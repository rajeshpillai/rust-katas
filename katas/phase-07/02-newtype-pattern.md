---
id: newtype-pattern
phase: 7
phase_title: "Data Modeling the Rust Way"
sequence: 2
title: The Newtype Pattern for Type Safety
hints:
  - "Two `u32` values are interchangeable to the compiler — it has no idea one is a user ID and the other is a product ID."
  - "Wrap each in a distinct struct: `struct UserId(u32)`. Now they are different types."
  - "Newtypes have zero runtime cost — the wrapper is erased during compilation."
---

## Description

The newtype pattern wraps a primitive type in a single-field tuple struct to create a distinct type. This prevents accidentally mixing up values that have the same underlying representation but different semantic meaning.

A `u32` is a `u32` — the compiler does not know whether it represents a user ID, a product ID, a port number, or a temperature. By wrapping each in its own type, you enlist the compiler to catch mix-ups at compile time, with zero runtime overhead.

## Broken Code

```rust
fn process_order(user_id: u32, product_id: u32, quantity: u32) {
    println!(
        "User {} ordered {} units of product {}",
        user_id, product_id, quantity
    );
}

fn main() {
    let user_id: u32 = 42;
    let product_id: u32 = 1001;
    let quantity: u32 = 3;

    // BUG: Arguments are swapped! product_id is passed as user_id
    // and vice versa. The compiler cannot catch this because both
    // are just u32.
    process_order(product_id, user_id, quantity);
    // Prints: "User 1001 ordered 3 units of product 42"
    // This is silently wrong. No compiler error. No runtime error.
    // Just incorrect behavior.
}
```

## Correct Code

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UserId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ProductId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Quantity(u32);

fn process_order(user_id: UserId, product_id: ProductId, quantity: Quantity) {
    println!(
        "User {:?} ordered {:?} units of product {:?}",
        user_id, product_id, quantity
    );
}

fn main() {
    let user_id = UserId(42);
    let product_id = ProductId(1001);
    let quantity = Quantity(3);

    // This would now cause a compile error:
    // process_order(product_id, user_id, quantity);
    //               ^^^^^^^^^^  ^^^^^^^
    // expected `UserId`, found `ProductId`

    // Correct order — enforced by the type system:
    process_order(user_id, product_id, quantity);
}
```

## Explanation

The broken version uses `u32` for three semantically distinct values. The compiler treats them as interchangeable — you can pass a product ID where a user ID is expected, and the code compiles and runs without complaint. The bug is entirely silent.

This class of bug is insidious because:
1. The code compiles without warnings.
2. The code runs without panicking.
3. The output looks reasonable (it is still numbers).
4. The bug may only manifest as incorrect business logic, possibly in production.

The correct version wraps each concept in its own **newtype**: a tuple struct with a single field. Now `UserId`, `ProductId`, and `Quantity` are distinct types. Passing a `ProductId` where a `UserId` is expected is a **type error** — caught at compile time, not in production.

**Zero-cost abstraction:** Newtypes are a compile-time-only concept. In the generated machine code, `UserId(42)` is represented as just `42` — the struct wrapper is erased. There is no runtime overhead: no indirection, no extra memory, no function call overhead.

**Deriving traits:** Since the newtype wraps a primitive, you often want to derive common traits:

| Trait | Purpose |
|---|---|
| `Debug` | Allows `{:?}` formatting |
| `Clone`, `Copy` | Allows copying (since `u32` is `Copy`) |
| `PartialEq`, `Eq` | Allows `==` comparison |
| `Hash` | Allows use as a `HashMap` key |
| `PartialOrd`, `Ord` | Allows ordering (if meaningful) |

**When NOT to derive:** Think carefully about which operations make sense. Should you be able to add two `UserId`s? Probably not. By not implementing `Add`, the compiler prevents nonsensical arithmetic on IDs.

**Accessing the inner value:** Use `.0` to access the wrapped value when needed:

```rust
let id = UserId(42);
let raw: u32 = id.0;  // Extract the inner u32
```

Or provide an explicit method:

```rust
impl UserId {
    fn as_u32(&self) -> u32 {
        self.0
    }
}
```

**The newtype pattern in practice:**

```rust
struct Meters(f64);
struct Kilometers(f64);
struct Miles(f64);

// These are all f64 underneath, but the compiler prevents:
// - Adding Meters to Miles without conversion
// - Passing Kilometers where Meters is expected
// - The kind of bug that crashed the Mars Climate Orbiter
```

The Mars Climate Orbiter was lost in 1999 because one team used Imperial units and another used metric. Both were floating-point numbers. A newtype pattern would have caught this at compile time.

## Compiler Error Interpretation

When you try to pass arguments in the wrong order with newtypes:

```
error[E0308]: mismatched types
  --> src/main.rs:18:20
   |
18 |     process_order(product_id, user_id, quantity);
   |     ------------- ^^^^^^^^^^ expected `UserId`, found `ProductId`
   |     |
   |     arguments to this function are incorrect
   |
note: function defined here
  --> src/main.rs:8:4
   |
8  | fn process_order(user_id: UserId, product_id: ProductId, quantity: Quantity) {
   |    ^^^^^^^^^^^^^ --------------   --------------------   -----------------
```

This error is exactly what we wanted:

- **"expected `UserId`, found `ProductId`"** — The compiler caught the semantic mix-up. These types may both wrap `u32`, but they are not interchangeable.
- The error points to the function signature, showing you what type each parameter expects.
- This is a **compile-time guarantee** that the arguments are in the correct order. No unit test needed. No runtime check needed. The type system enforces correctness.

The broken version would have produced **no error at all** — and that silence is the real danger. The newtype pattern turns silent bugs into loud, unmissable compiler errors.
