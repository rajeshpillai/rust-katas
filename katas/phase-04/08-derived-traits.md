---
id: derived-traits
phase: 4
phase_title: "Traits & Generics"
sequence: 8
title: "Derived Traits — Debug, Clone, PartialEq, and More"
hints:
  - "`#[derive(Debug)]` auto-generates a `Debug` implementation, enabling `{:?}` formatting."
  - "Derive only works if all fields implement the trait. A struct with a field that isn't `Clone` cannot derive `Clone`."
  - "Common derives: `Debug` (printing), `Clone` (explicit copy), `PartialEq` (comparison), `Default` (zero values)."
---

## Description

Rust's `#[derive]` attribute automatically generates trait implementations for your types. Instead of writing boilerplate `impl Debug for MyStruct { ... }`, you write `#[derive(Debug)]` and the compiler generates it for you.

Deriving works when all fields of the struct implement the trait. If any field does not, the derive fails.

## Broken Code

```rust
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

fn main() {
    let c = Color { r: 255, g: 128, b: 0 };

    // Bug: Color doesn't implement Debug
    println!("Color: {:?}", c);

    // Bug: Color doesn't implement PartialEq
    let c2 = Color { r: 255, g: 128, b: 0 };
    if c == c2 {
        println!("Colors are equal");
    }

    // Bug: Color doesn't implement Clone
    let c3 = c.clone();
}
```

## Correct Code

```rust
// Derive multiple traits at once
#[derive(Debug, Clone, PartialEq, Default)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

// Derive works on enums too
#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Triangle(f64, f64, f64),
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => std::f64::consts::PI * r * r,
            Shape::Rectangle(w, h) => w * h,
            Shape::Triangle(a, b, c) => {
                let s = (a + b + c) / 2.0;
                (s * (s - a) * (s - b) * (s - c)).sqrt()
            }
        }
    }
}

// Copy — for small, stack-only types
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    // --- Debug: {:?} formatting ---
    let orange = Color::new(255, 165, 0);
    println!("Debug:  {:?}", orange);
    println!("Pretty: {:#?}", orange); // Multi-line format

    // --- Clone: explicit copying ---
    let red = Color::new(255, 0, 0);
    let also_red = red.clone();
    println!("\nOriginal: {:?}", red);
    println!("Clone:    {:?}", also_red);

    // --- PartialEq: == and != ---
    println!("\nred == also_red? {}", red == also_red);
    println!("red != orange?   {}", red != orange);

    // --- Default: zero/empty values ---
    let black: Color = Color::default();
    println!("\nDefault color: {:?}", black); // r=0, g=0, b=0

    // Default works in generic contexts
    let colors: Vec<Color> = vec![Color::default(); 3];
    println!("Three defaults: {:?}", colors);

    // --- Copy: implicit copying (no .clone() needed) ---
    let p1 = Point { x: 1.0, y: 2.0 };
    let p2 = p1;      // Copy, not move!
    let p3 = p1;      // Still works — p1 wasn't moved
    println!("\np1: {:?}", p1);
    println!("p2: {:?}", p2);
    println!("p3: {:?}", p3);

    // --- Enums with derive ---
    let shapes = vec![
        Shape::Circle(5.0),
        Shape::Rectangle(4.0, 6.0),
        Shape::Triangle(3.0, 4.0, 5.0),
    ];

    for shape in &shapes {
        println!("\n{:?} => area = {:.2}", shape, shape.area());
    }

    // Clone and compare enums
    let s1 = Shape::Circle(5.0);
    let s2 = s1.clone();
    println!("\nShapes equal? {}", s1 == s2);

    // --- What each derive gives you ---
    println!("\n--- Trait summary ---");
    println!("Debug:     enables {{:?}} formatting");
    println!("Clone:     enables .clone() (explicit deep copy)");
    println!("Copy:      enables implicit copy (no move)");
    println!("PartialEq: enables == and !=");
    println!("Eq:        marker for total equality (add with PartialEq)");
    println!("Hash:      enables use as HashMap key");
    println!("Default:   enables Type::default() for zero values");
    println!("PartialOrd/Ord: enables <, >, sorting");
}
```

## Explanation

The broken code defines `Color` without any derives. By default, a struct has no `Debug`, `Clone`, or `PartialEq` implementation. Without these, you cannot:
- Print with `{:?}` (needs `Debug`)
- Compare with `==` (needs `PartialEq`)
- Call `.clone()` (needs `Clone`)

**Adding `#[derive(Debug, Clone, PartialEq, Default)]` gives you all four** with zero boilerplate.

**How derive works:** The compiler looks at each field's type. If all fields implement the trait, the derive succeeds. For example, `#[derive(Clone)]` on `Color` works because `u8` implements `Clone`. If one field did not implement `Clone`, the derive would fail.

**The most common derivable traits:**

| Trait | What it enables | When to use |
|---|---|---|
| `Debug` | `{:?}` and `{:#?}` formatting | Almost always — essential for debugging |
| `Clone` | `.clone()` explicit copy | When you need to duplicate values |
| `Copy` | Implicit copy (no move) | Small, stack-only types (requires `Clone`) |
| `PartialEq` | `==` and `!=` | Comparison and testing |
| `Eq` | Marker: total equality | Add when `PartialEq` + no NaN-like values |
| `Hash` | Use as `HashMap`/`HashSet` key | Requires `Eq` |
| `Default` | `Type::default()` | Zero values, struct builder patterns |
| `PartialOrd` | `<`, `>`, `<=`, `>=` | Sorting, min/max |
| `Ord` | Total ordering | `sort()`, `BTreeMap` keys |

**`Copy` vs `Clone`:**
- `Clone` is explicit: you must call `.clone()`
- `Copy` is implicit: assignment copies instead of moving
- `Copy` requires `Clone` and only works for types that are entirely stack-allocated
- `String`, `Vec`, and other heap types cannot be `Copy`

## ⚠️ Caution

- `#[derive(Copy)]` requires `#[derive(Clone)]` — always list both: `#[derive(Clone, Copy)]`.
- Types containing `String`, `Vec`, `Box`, or other heap data cannot derive `Copy`. Use `Clone` instead.
- `PartialEq` on floats: `f64::NAN != f64::NAN` (by IEEE 754). For `Eq` and `Hash`, avoid float fields or use a wrapper.
- Derived `PartialEq` compares all fields. If you want to exclude a field from comparison, implement `PartialEq` manually.

## 💡 Tips

- Start every struct with `#[derive(Debug)]` — you will always need to print it eventually.
- `#[derive(Debug, Clone, PartialEq)]` is the most common trio. Add `Default` for structs with sensible zero values.
- In tests, `assert_eq!(a, b)` requires `Debug + PartialEq`. Derive both for any type you test.
- You can derive and manually implement different traits on the same type. Manual impls override derived ones.

## Compiler Error Interpretation

```
error[E0277]: `Color` doesn't implement `Debug`
 --> main.rs:9:28
  |
9 |     println!("Color: {:?}", c);
  |                             ^ `Color` cannot be formatted using `{:?}`
  |
  = help: the trait `Debug` is not implemented for `Color`
  = note: add `#[derive(Debug)]` to `Color` or manually `impl Debug for Color`
```

The compiler tells you exactly what is missing and how to fix it: add `#[derive(Debug)]`. This pattern repeats for every derivable trait — the error always names the missing trait and suggests deriving it.
