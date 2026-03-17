---
id: methods-and-impl
phase: 0
phase_title: "Rust as a Language"
sequence: 12
title: Methods and impl Blocks
hints:
  - "Methods are defined inside an `impl` block for the type. The first parameter is always `self` (or `&self` or `&mut self`)."
  - "A method that takes `&self` borrows the instance. A method that takes `self` consumes it."
  - "Associated functions (like `new`) don't take `self` — they're called with `Type::name()`, not `instance.name()`."
---

## Description

In Rust, methods are functions associated with a type, defined inside an `impl` block. The first parameter determines how the method accesses the instance:

- `&self` — borrows immutably (read-only access)
- `&mut self` — borrows mutably (can modify)
- `self` — takes ownership (consumes the instance)

Functions without `self` are **associated functions** (like static methods), called with `Type::name()`.

## Broken Code

```rust
struct Rectangle {
    width: f64,
    height: f64,
}

// Bug: methods defined as free functions, not in impl block
fn area(rect: &Rectangle) -> f64 {
    rect.width * rect.height
}

fn main() {
    let r = Rectangle { width: 10.0, height: 5.0 };
    // Bug: calling as method, but area is a free function
    println!("Area: {}", r.area());
}
```

## Correct Code

```rust
#[derive(Debug)]
struct Rectangle {
    width: f64,
    height: f64,
}

impl Rectangle {
    // Associated function (no self) — constructor
    fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    // Method: &self borrows immutably
    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }

    fn is_square(&self) -> bool {
        (self.width - self.height).abs() < f64::EPSILON
    }

    // Method: &mut self borrows mutably — can modify
    fn scale(&mut self, factor: f64) {
        self.width *= factor;
        self.height *= factor;
    }

    // Method: self takes ownership — consumes the rectangle
    fn into_square(self) -> Rectangle {
        let side = self.width.max(self.height);
        Rectangle::new(side, side)
    }
}

fn main() {
    // Associated function — called on the type, not an instance
    let mut rect = Rectangle::new(10.0, 5.0);
    println!("Rectangle: {:?}", rect);

    // &self methods — read-only access
    println!("Area: {}", rect.area());
    println!("Perimeter: {}", rect.perimeter());
    println!("Is square? {}", rect.is_square());

    // &mut self method — modifies the instance
    rect.scale(2.0);
    println!("\nAfter scale(2.0): {:?}", rect);
    println!("New area: {}", rect.area());

    // self method — consumes the instance
    let square = rect.into_square();
    println!("\nSquare: {:?}", square);
    println!("Square area: {}", square.area());
    // println!("{:?}", rect);  // ERROR: rect was moved into into_square()

    // Multiple impl blocks are allowed
    println!("\nFormatted: {}", square.describe());
}

// You can split methods across multiple impl blocks
impl Rectangle {
    fn describe(&self) -> String {
        format!("{}x{} rectangle (area: {:.1})", self.width, self.height, self.area())
    }
}
```

## Explanation

The broken code defines `area` as a **free function** (standalone), not as a method inside an `impl` block. Free functions are called as `area(&r)`, not `r.area()`. The dot syntax `r.area()` only works for methods defined in an `impl` block.

**The three `self` flavors:**

| Parameter | Access | Instance after call |
|---|---|---|
| `&self` | Read-only | Still usable |
| `&mut self` | Read and write | Still usable (was mutably borrowed) |
| `self` | Full ownership | Consumed — cannot use afterward |

**`Self` vs `self`:**
- `self` (lowercase) — the instance being called on
- `Self` (uppercase) — an alias for the type itself (useful in constructors)

**Associated functions** don't take `self`. They are called on the type: `Rectangle::new(10.0, 5.0)`. The most common pattern is a `new` constructor. Rust has no special constructor syntax — `new` is just a convention.

**Auto-referencing:** When you call `rect.area()`, Rust automatically adds `&` to make it `(&rect).area()`. You don't need to write `(&rect).area()` explicitly. This also works for `&mut self` methods on mutable bindings.

**Multiple `impl` blocks** are allowed for the same type. This is useful for organizing code — you might put constructors in one block and methods in another.

## ⚠️ Caution

- `self` (by value) consumes the instance. After calling `rect.into_square()`, `rect` is no longer usable. Choose `&self` unless you have a specific reason to take ownership.
- Methods and associated functions share the same namespace. You cannot have both `fn area(&self)` and `fn area()` (without self) in the same `impl` block.
- Rust does not have inheritance. Use traits (Phase 4) for shared behavior across types.

## 💡 Tips

- Use `Self` instead of repeating the type name in `impl` blocks: `fn new() -> Self` instead of `fn new() -> Rectangle`.
- Method chaining: return `&mut self` to enable `builder.width(10).height(5).build()`.
- The `#[derive(Debug)]` attribute auto-generates a debug format. Add it to any struct you want to print with `{:?}`.
- You can call methods on references too: if `r: &Rectangle`, `r.area()` works because `&self` matches.

## Compiler Error Interpretation

```
error[E0599]: no method named `area` found for struct `Rectangle` in the current scope
  --> main.rs:13:30
   |
1  | struct Rectangle {
   | ---------------- method `area` not found for this struct
...
13 |     println!("Area: {}", r.area());
   |                            ^^^^ method not found in `Rectangle`
   |
   = help: items from traits can only be used if the trait is in scope
```

The compiler says "no method named `area`" on `Rectangle`. The function `area` exists, but it is a free function, not a method. To make it a method, move it into `impl Rectangle { ... }` and change the first parameter to `&self`.
