---
id: tuples-and-destructuring
phase: 0
phase_title: "Rust as a Language"
sequence: 9
title: Tuples and Destructuring
hints:
  - "A tuple groups multiple values of different types into one compound value."
  - "Access tuple elements with `.0`, `.1`, `.2` — not with square brackets."
  - "Destructuring lets you unpack a tuple into named variables: `let (x, y) = point;`"
---

## Description

A **tuple** groups a fixed number of values with potentially different types into one compound value. Tuples have a fixed length — once declared, they cannot grow or shrink.

You access elements by position (`.0`, `.1`) or by **destructuring** — unpacking the tuple into individual variables. Destructuring is a fundamental Rust pattern that appears in `let` bindings, function parameters, and `match` arms.

## Broken Code

```rust
fn min_max(numbers: &[i32]) -> (i32, i32) {
    let mut min = numbers[0];
    let mut max = numbers[0];
    for &n in numbers {
        if n < min { min = n; }
        if n > max { max = n; }
    }
    (min, max)
}

fn main() {
    let result = min_max(&[3, 7, 1, 9, 4]);

    // Bug: trying to use array syntax on a tuple
    println!("Min: {}", result[0]);
    println!("Max: {}", result[1]);
}
```

## Correct Code

```rust
fn min_max(numbers: &[i32]) -> (i32, i32) {
    let mut min = numbers[0];
    let mut max = numbers[0];
    for &n in numbers {
        if n < min { min = n; }
        if n > max { max = n; }
    }
    (min, max)
}

fn main() {
    let result = min_max(&[3, 7, 1, 9, 4]);

    // Access by position with dot syntax
    println!("Min: {}", result.0);
    println!("Max: {}", result.1);

    // Or destructure into named variables (preferred)
    let (min, max) = min_max(&[3, 7, 1, 9, 4]);
    println!("Min: {}, Max: {}", min, max);

    // Tuples can hold different types
    let person: (&str, u32, bool) = ("Alice", 30, true);
    println!("{} is {} years old, active: {}", person.0, person.1, person.2);

    // Destructuring with type annotation
    let (name, age, active) = person;
    println!("{} (age {}) active={}", name, age, active);

    // Ignore fields with _
    let (name, _, _) = person;
    println!("Just the name: {}", name);

    // Nested tuple destructuring
    let nested = ((1, 2), (3, 4));
    let ((a, b), (c, d)) = nested;
    println!("Corners: ({},{}) and ({},{})", a, b, c, d);

    // The unit type () is an empty tuple
    let nothing: () = ();
    println!("Unit value: {:?}", nothing);

    // Swap values using tuple destructuring
    let mut x = 10;
    let mut y = 20;
    println!("Before swap: x={}, y={}", x, y);
    (x, y) = (y, x);  // Tuple swap — no temp variable needed
    println!("After swap:  x={}, y={}", x, y);
}
```

## Explanation

The broken code tries `result[0]` to access a tuple element. Tuples use **dot syntax** (`.0`, `.1`), not index syntax (`[0]`, `[1]`). Square bracket indexing is for arrays and slices.

**Why dot syntax?** Tuples can hold different types in each position. `(i32, String, bool)` has three elements of three different types. The dot syntax with a literal number (`.0`, `.1`) lets the compiler know the exact type at compile time. Array indexing (`[i]`) allows runtime indices, which would require all elements to be the same type.

**Destructuring** is the idiomatic way to work with tuples:

```rust
let (min, max) = min_max(&data);  // Unpack into named variables
```

This is clearer than `result.0` and `result.1` because the names convey meaning.

**Key tuple properties:**

| Property | Detail |
|---|---|
| Fixed size | Length is part of the type: `(i32, i32)` ≠ `(i32, i32, i32)` |
| Heterogeneous | Each position can be a different type |
| Stack allocated | Tuples live on the stack (like arrays) |
| Access | `.0`, `.1`, `.2` or destructuring |
| Unit type | `()` is the empty tuple — Rust's "void" equivalent |

**The unit type `()`** is everywhere in Rust. Functions that return nothing actually return `()`. Expressions ending with `;` evaluate to `()`. It is a real type with exactly one value: `()`.

## ⚠️ Caution

- Tuples are limited to 12 elements for most trait implementations (Debug, Clone, etc.). For more fields, use a struct.
- Tuple indexing must use literal numbers — you cannot use a variable: `tuple.i` is invalid.
- Destructuring patterns must match the tuple's length exactly. `let (a, b) = (1, 2, 3)` is a compile error.

## 💡 Tips

- Use destructuring to give tuple elements meaningful names at the call site.
- The `_` pattern ignores a field: `let (x, _, z) = triple;` — useful when you only need some values.
- Tuple swap `(a, b) = (b, a)` is idiomatic Rust — no temporary variable needed.
- Functions returning `Result<(), Error>` use the unit type to say "success has no data, only the absence of failure."

## Compiler Error Interpretation

```
error[E0608]: cannot index into a value of type `(i32, i32)`
 --> main.rs:13:27
  |
13|     println!("Min: {}", result[0]);
  |                         ^^^^^^^^^
  |
help: to access tuple elements, use `result.0` or destructuring
```

The compiler tells you: tuples are not arrays. Use `.0` syntax or destructuring to access elements. The distinction exists because tuple positions have independent types — something array indexing cannot express.
