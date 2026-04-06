---
id: what-are-generics
phase: 4
phase_title: "Traits & Generics"
sequence: 5
title: "What Are Generics? — One Function, Many Types"
hints:
  - "Without generics, you'd need a separate function for each type: `largest_i32`, `largest_f64`, `largest_char`."
  - "Generics let you write `fn largest<T>(list: &[T]) -> &T` — one function that works with any type `T`."
  - "The compiler generates specialized versions for each type you actually use. This is called monomorphization — zero runtime cost."
---

## Description

**Generics** let you write code that works with many types without duplicating it. Instead of writing `largest_i32`, `largest_f64`, and `largest_char`, you write one `largest<T>` that works with any type `T`.

The `<T>` syntax means "this function/struct accepts a type parameter." The compiler generates specialized code for each concrete type at compile time — there is no runtime cost. This is called **monomorphization**.

## Broken Code

```rust
fn largest_i32(list: &[i32]) -> &i32 {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn largest_f64(list: &[f64]) -> &f64 {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn largest_char(list: &[char]) -> &char {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn main() {
    let numbers = vec![34, 50, 25, 100, 65];
    println!("Largest number: {}", largest_i32(&numbers));

    let floats = vec![1.5, 3.7, 2.1];
    println!("Largest float: {}", largest_f64(&floats));

    let chars = vec!['y', 'm', 'a', 'q'];
    println!("Largest char: {}", largest_char(&chars));

    // What about u32? u64? i8? We'd need even more copies!
}
```

## Correct Code

```rust
// One generic function replaces all the duplicates
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

// A generic struct — works with any numeric type
#[derive(Debug)]
struct Point<T> {
    x: T,
    y: T,
}

// A generic struct with two type parameters
#[derive(Debug)]
struct Pair<A, B> {
    first: A,
    second: B,
}

fn main() {
    // --- Generic function: one definition, many types ---
    let numbers = vec![34, 50, 25, 100, 65];
    println!("Largest i32: {}", largest(&numbers));

    let floats = vec![1.5, 3.7, 2.1];
    println!("Largest f64: {}", largest(&floats));

    let chars = vec!['y', 'm', 'a', 'q'];
    println!("Largest char: {}", largest(&chars));

    let words = vec!["hello", "world", "rust"];
    println!("Largest &str: {}", largest(&words));

    // --- Generic struct ---
    let int_point = Point { x: 5, y: 10 };
    let float_point = Point { x: 1.5, y: 3.7 };
    println!("\nInteger point: {:?}", int_point);
    println!("Float point: {:?}", float_point);

    // Two different type parameters
    let pair = Pair { first: "name", second: 42 };
    println!("Pair: {:?}", pair);

    // --- Generic Option and Result (from std) ---
    // Option<T> is a generic enum: Some(T) or None
    let some_number: Option<i32> = Some(42);
    let some_string: Option<&str> = Some("hello");
    let no_value: Option<i32> = None;
    println!("\nOption<i32>: {:?}", some_number);
    println!("Option<&str>: {:?}", some_string);
    println!("Option<i32> None: {:?}", no_value);

    // Result<T, E> is a generic enum: Ok(T) or Err(E)
    let ok: Result<i32, String> = Ok(42);
    let err: Result<i32, String> = Err("something went wrong".to_string());
    println!("\nResult Ok: {:?}", ok);
    println!("Result Err: {:?}", err);

    // --- The turbofish ::<> for disambiguation ---
    let parsed = "42".parse::<i32>().unwrap();
    let also_parsed: i32 = "42".parse().unwrap();
    println!("\nParsed: {}, {}", parsed, also_parsed);

    // Vec::<i32>::new() or let v: Vec<i32> = Vec::new()
    let v = Vec::<f64>::new();
    println!("Empty Vec<f64>: {:?}", v);
}
```

## Explanation

The broken code has three nearly identical functions — `largest_i32`, `largest_f64`, `largest_char`. The logic is the same; only the type changes. This violates DRY (Don't Repeat Yourself) and becomes unmaintainable as types multiply.

**Generics solve this:**

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T
```

This reads: "a function `largest` that accepts a type parameter `T`, where `T` must support comparison (`PartialOrd`). It takes a slice of `T` and returns a reference to `T`."

**How generics work under the hood:**

When you call `largest(&numbers)` where `numbers` is `Vec<i32>`, the compiler generates:

```rust
fn largest_i32(list: &[i32]) -> &i32 { ... }  // Auto-generated!
```

When you call `largest(&chars)`, it generates a `char` version. This is **monomorphization** — the generic code is specialized at compile time. The runtime cost is zero; you get the same performance as hand-written specialized functions.

**Generic structs** work the same way:

```rust
struct Point<T> { x: T, y: T }  // T must be the same type for both fields
struct Pair<A, B> { first: A, second: B }  // Different types allowed
```

**You already use generics daily:** `Vec<T>`, `Option<T>`, `Result<T, E>`, `HashMap<K, V>` are all generic types from the standard library.

**The `: PartialOrd` part** is a **trait bound** — it restricts `T` to types that support `>` comparison. Without it, the compiler cannot guarantee that `item > largest` works. Trait bounds are covered in depth in the existing katas.

## ⚠️ Caution

- `Point<T>` requires both `x` and `y` to be the same type. `Point { x: 5, y: 3.0 }` fails because `i32 ≠ f64`. Use `Point<A, B>` for different types.
- The compiler generates code for every concrete type used. Excessive generic instantiation can increase binary size (code bloat).
- Generics without trait bounds are very limited — you can store and return values, but cannot do much with them (no printing, comparing, or arithmetic).

## 💡 Tips

- Read `<T>` as "for any type T." Read `<T: Display>` as "for any type T that can be displayed."
- Use meaningful type parameter names: `<K, V>` for key-value, `<T, E>` for success-error. Single letters are convention for truly generic parameters.
- The turbofish `::<Type>` is needed when the compiler cannot infer the type: `"42".parse::<i32>()`.
- Generics are compile-time only — they have zero runtime overhead. This is Rust's "zero-cost abstraction" principle.

## Compiler Error Interpretation

If you write `largest` without the `PartialOrd` bound:

```
error[E0369]: binary operation `>` cannot be applied to type `&T`
 --> main.rs:4:17
  |
4 |         if item > largest {
  |            ---- ^ ------- &T
  |            |
  |            &T
  |
help: consider restricting type parameter `T`
  |
1 | fn largest<T: std::cmp::PartialOrd>(list: &[T]) -> &T {
  |             ++++++++++++++++++++++
```

The compiler says: "I don't know if `T` supports `>`." Without a trait bound, `T` could be anything — a struct, a closure, a file handle. The compiler suggests adding `: PartialOrd` to promise that `T` supports comparison. This is the bridge between generics and traits.

---

| [Prev: Dynamic Dispatch and Object Safety](#/katas/dynamic-dispatch) | [Next: Generic Structs and Enums](#/katas/generic-structs-and-enums) |
