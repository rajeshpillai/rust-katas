---
id: operator-overloading
phase: 4
phase_title: "Traits & Generics"
sequence: 10
title: "Operator Overloading via Traits"
hints:
  - "Operators in Rust are syntactic sugar for trait method calls. `a + b` calls `a.add(b)`."
  - "To use `+` with your type, implement `std::ops::Add`. The trait has an associated type `Output` for the result type."
  - "Import the trait from `std::ops`: `Add`, `Sub`, `Mul`, `Neg`, `Index`, etc."
---

## Description

In Rust, operators like `+`, `-`, `*`, `==`, and `[]` are defined by traits in `std::ops`. To use an operator with your custom type, you implement the corresponding trait. This is **operator overloading** — giving operators custom behavior for your types.

Unlike some languages where overloading is implicit, Rust makes it explicit through traits. `a + b` is just syntactic sugar for `Add::add(a, b)`.

## Broken Code

```rust
struct Vec2 {
    x: f64,
    y: f64,
}

fn main() {
    let a = Vec2 { x: 1.0, y: 2.0 };
    let b = Vec2 { x: 3.0, y: 4.0 };

    // Bug: + is not defined for Vec2
    let c = a + b;
    println!("({}, {})", c.x, c.y);
}
```

## Correct Code

```rust
use std::ops::{Add, Sub, Mul, Neg};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

// Display: controls how println!("{}") works
impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.1}, {:.1})", self.x, self.y)
    }
}

// Add: a + b
impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

// Sub: a - b
impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

// Neg: -a (unary negation)
impl Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

// Mul<f64>: vec * scalar
impl Mul<f64> for Vec2 {
    type Output = Vec2;

    fn mul(self, scalar: f64) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }
}

// Mul<Vec2> for f64: scalar * vec (reverse order)
impl Mul<Vec2> for f64 {
    type Output = Vec2;

    fn mul(self, vec: Vec2) -> Vec2 {
        Vec2::new(self * vec.x, self * vec.y)
    }
}

fn main() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, 4.0);

    // Add and Sub
    println!("{} + {} = {}", a, b, a + b);
    println!("{} - {} = {}", a, b, a - b);

    // Negation
    println!("-{} = {}", a, -a);

    // Scalar multiplication (both orders)
    println!("{} * 3 = {}", a, a * 3.0);
    println!("3 * {} = {}", a, 3.0 * a);

    // Chaining operations
    let result = (a + b) * 2.0 - Vec2::new(1.0, 1.0);
    println!("\n(a + b) * 2 - (1,1) = {}", result);

    // Comparison (from PartialEq derive)
    println!("\na == a? {}", a == a);
    println!("a == b? {}", a == b);

    // Length (manual method)
    println!("\n|{}| = {:.3}", a, a.length());
    println!("|{}| = {:.3}", b, b.length());

    // --- Operator trait table ---
    println!("\n--- Operator traits ---");
    println!("  +   => std::ops::Add");
    println!("  -   => std::ops::Sub");
    println!("  *   => std::ops::Mul");
    println!("  /   => std::ops::Div");
    println!("  %   => std::ops::Rem");
    println!(" -x   => std::ops::Neg");
    println!(" a[i] => std::ops::Index");
    println!(" ==   => std::cmp::PartialEq");
    println!(" <    => std::cmp::PartialOrd");
}
```

## Explanation

The broken code uses `+` on `Vec2`, but Rust does not know how to add two `Vec2` values. Unlike dynamically typed languages, operators are not magically defined — you must implement the corresponding trait.

**Implementing `Add`:**

```rust
impl Add for Vec2 {
    type Output = Vec2;          // The result type of a + b
    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}
```

Now `a + b` compiles and calls this `add` method. The `type Output` associated type specifies the return type (it can differ from the operand types).

**Cross-type operations:** `Mul<f64> for Vec2` enables `vec * 3.0`. To also support `3.0 * vec`, you implement `Mul<Vec2> for f64`. Rust does not assume commutativity.

**Common operator traits:**

| Operator | Trait | Method |
|---|---|---|
| `a + b` | `Add` | `add(self, rhs)` |
| `a - b` | `Sub` | `sub(self, rhs)` |
| `a * b` | `Mul` | `mul(self, rhs)` |
| `a / b` | `Div` | `div(self, rhs)` |
| `-a` | `Neg` | `neg(self)` |
| `a[i]` | `Index` | `index(&self, idx)` |
| `a == b` | `PartialEq` | `eq(&self, other)` |
| `a < b` | `PartialOrd` | `partial_cmp(&self, other)` |

**`Display` is not an operator** but controls `{}` formatting. Implementing it gives your type a user-facing string representation.

**`Copy` makes operators ergonomic.** Without `Copy`, `a + b` would consume both `a` and `b`. With `#[derive(Copy)]`, `a` and `b` are copied, so you can reuse them after `+`.

## ⚠️ Caution

- Operator traits take `self` by value. Without `Copy`, the operands are consumed. Derive `Copy` for small types, or implement the trait for `&Vec2` references.
- Don't overload operators to mean something unintuitive. `+` should mean addition, concatenation, or union — not something surprising.
- `PartialEq` can be derived but `Display` cannot — you must always implement `Display` manually.

## 💡 Tips

- Derive `PartialEq` instead of implementing it manually — the derived version compares all fields, which is usually correct.
- For `Display`, implement `fmt::Display` and you automatically get `.to_string()` for free.
- Use `#[derive(Clone, Copy)]` on small types to make operators non-consuming.
- The `Index` trait is useful for custom collection types: `impl Index<usize> for MyList`.

## Compiler Error Interpretation

```
error[E0369]: cannot add `Vec2` to `Vec2`
 --> main.rs:11:17
  |
11|     let c = a + b;
  |             - ^ - Vec2
  |             |
  |             Vec2
  |
note: an implementation of `Add` might be missing for `Vec2`
  |
  = help: the trait `Add` is not implemented for `Vec2`
```

The compiler says `+` is not available because the `Add` trait is not implemented. It points you to the exact trait needed. Implement `impl Add for Vec2 { ... }` to make `+` work.

---

| [Prev: Associated Types vs Generic Parameters](#/katas/associated-types) | [Next: Closure Capture and Ownership](#/katas/closure-capture) |
