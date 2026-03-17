---
id: associated-types
phase: 4
phase_title: "Traits & Generics"
sequence: 9
title: "Associated Types vs Generic Parameters"
hints:
  - "An associated type is declared inside a trait with `type Output;` — the implementor chooses the concrete type."
  - "Use associated types when there is exactly one natural implementation per type. Use generic parameters when multiple implementations make sense."
  - "The `Iterator` trait uses an associated type: `type Item;` — each iterator has exactly one item type."
---

## Description

Traits can have **associated types** — type placeholders that the implementor fills in. The `Iterator` trait's `type Item` is the most common example. Associated types differ from generic parameters: a type can implement `Iterator` only once (with one `Item`), but could implement a generic trait `Convert<T>` for many different `T`.

## Broken Code

```rust
// Bug: using a generic parameter where an associated type is needed
trait Summable<Output> {
    fn sum_all(&self) -> Output;
}

// This compiles, but now Vec<i32> could implement Summable<i32>,
// Summable<f64>, Summable<String>... which one is "the" sum?
impl Summable<i32> for Vec<i32> {
    fn sum_all(&self) -> i32 {
        self.iter().sum()
    }
}

fn print_sum<T: Summable<???>>( items: &T) {
    // Bug: what type do we put for Output? The caller shouldn't need to know.
    println!("Sum: {}", items.sum_all());
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    println!("Sum: {}", numbers.sum_all());
}
```

## Correct Code

```rust
use std::fmt;

// Associated type: each implementor has exactly one Output
trait Summable {
    type Output: fmt::Display;

    fn sum_all(&self) -> Self::Output;
}

impl Summable for Vec<i32> {
    type Output = i32;

    fn sum_all(&self) -> i32 {
        self.iter().sum()
    }
}

impl Summable for Vec<f64> {
    type Output = f64;

    fn sum_all(&self) -> f64 {
        self.iter().sum()
    }
}

impl Summable for Vec<&str> {
    type Output = String;

    fn sum_all(&self) -> String {
        self.join(", ")
    }
}

// Callers don't need to specify Output — it's determined by the implementor
fn print_sum<T: Summable>(items: &T) {
    println!("Sum: {}", items.sum_all());
}

// --- Demonstrating the Iterator trait's associated type ---
struct Countdown {
    value: u32,
}

impl Countdown {
    fn new(start: u32) -> Self {
        Countdown { value: start }
    }
}

impl Iterator for Countdown {
    type Item = u32;  // Associated type: this iterator yields u32

    fn next(&mut self) -> Option<Self::Item> {
        if self.value == 0 {
            None
        } else {
            let current = self.value;
            self.value -= 1;
            Some(current)
        }
    }
}

// --- When to use generic parameters instead ---
trait ConvertTo<T> {
    fn convert(&self) -> T;
}

// Same type can implement ConvertTo for multiple target types
impl ConvertTo<f64> for i32 {
    fn convert(&self) -> f64 {
        *self as f64
    }
}

impl ConvertTo<String> for i32 {
    fn convert(&self) -> String {
        self.to_string()
    }
}

fn main() {
    // --- Associated type: one natural output per type ---
    let ints = vec![1, 2, 3, 4, 5];
    let floats = vec![1.1, 2.2, 3.3];
    let words = vec!["hello", "world", "rust"];

    print_sum(&ints);
    print_sum(&floats);
    print_sum(&words);

    // --- Iterator's associated type ---
    println!("\nCountdown:");
    for n in Countdown::new(5) {
        print!("{} ", n);
    }
    println!();

    // Collect uses the associated type to know what to collect into
    let collected: Vec<u32> = Countdown::new(3).collect();
    println!("Collected: {:?}", collected);

    // --- Generic parameter: multiple conversions ---
    let x: i32 = 42;
    let as_float: f64 = x.convert();
    let as_string: String = x.convert();
    println!("\n42 as f64: {}", as_float);
    println!("42 as String: {}", as_string);

    // --- Summary ---
    println!("\n--- When to use which ---");
    println!("Associated type: one implementation per type (Iterator::Item)");
    println!("Generic param:   multiple implementations per type (From<T>)");
}
```

## Explanation

The broken code uses a generic parameter `Summable<Output>` where an associated type is more appropriate. The problem: when calling `print_sum`, the caller must specify `Output` — but the whole point is that `Output` is determined by the type, not by the caller.

**Associated types vs generic parameters:**

| | Associated type | Generic parameter |
|---|---|---|
| Syntax | `type Output;` | `trait Foo<T>` |
| Implementations per type | Exactly one | Multiple allowed |
| Caller specifies? | No — determined by implementor | Yes — or inferred |
| Example | `Iterator::Item` | `From<T>` |

**Rule of thumb:** Use associated types when the relationship is one-to-one (each iterator has one item type). Use generic parameters when one type can implement the trait for multiple other types (a number can be `From<i32>` and `From<u8>`).

**`Self::Output` in trait methods** refers to the associated type. The caller writes `T: Summable` without mentioning `Output` — the compiler resolves it from the implementation.

**The `Iterator` trait** is the canonical example:

```rust
trait Iterator {
    type Item;  // Associated type
    fn next(&mut self) -> Option<Self::Item>;
}
```

A `Countdown` iterator always yields `u32`. Making `Item` an associated type means you cannot accidentally implement `Iterator` twice with different item types.

## ⚠️ Caution

- You cannot implement a trait with associated types more than once for the same type. If you need multiple implementations, use a generic parameter.
- Associated types can have trait bounds: `type Output: Display;` ensures all implementors' Output types are displayable.
- When reading trait signatures, `Self::Item` means "whatever type the implementor chose for `Item`."

## 💡 Tips

- If you're designing a trait, start with associated types. Switch to generic parameters only if you need multiple implementations per type.
- `type Output: Display + Debug;` adds bounds to the associated type — all implementors must choose a type that satisfies these bounds.
- In the standard library: `Iterator::Item`, `Add::Output`, `Deref::Target` are associated types. `From<T>`, `Into<T>`, `AsRef<T>` use generic parameters.

## Compiler Error Interpretation

With the broken generic parameter approach, calling `print_sum` fails:

```
error[E0283]: type annotations needed
 --> main.rs:14:5
  |
14|     print_sum(&numbers);
  |     ^^^^^^^^^ cannot infer type of the type parameter `Output`
```

The compiler cannot determine which `Output` type to use because the generic parameter approach allows multiple implementations. With an associated type, this ambiguity disappears — each type has exactly one `Output`.
