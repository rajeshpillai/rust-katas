---
id: generic-structs-and-enums
phase: 4
phase_title: "Traits & Generics"
sequence: 6
title: Generic Structs and Enums
hints:
  - "When implementing methods on a generic struct, the `impl` block must also declare the type parameter: `impl<T> MyStruct<T>`."
  - "Forgetting the `<T>` on `impl` makes the compiler think `T` is a concrete type name, not a parameter."
  - "You can add trait bounds on the `impl` block to restrict which types get certain methods."
---

## Description

Generic structs and enums let you create data structures that work with any type. `Vec<T>`, `Option<T>`, and `Result<T, E>` are all generic types from the standard library.

When you implement methods on a generic type, the `impl` block must declare the same type parameters. You can also write specialized `impl` blocks that only apply to specific types.

## Broken Code

```rust
#[derive(Debug)]
struct Wrapper<T> {
    value: T,
}

// Bug: missing <T> on impl — compiler thinks T is a concrete type
impl Wrapper<T> {
    fn new(value: T) -> Self {
        Wrapper { value }
    }

    fn get(&self) -> &T {
        &self.value
    }
}

fn main() {
    let w = Wrapper::new(42);
    println!("Value: {}", w.get());
}
```

## Correct Code

```rust
#[derive(Debug)]
struct Wrapper<T> {
    value: T,
}

// Generic impl — works for all T
impl<T> Wrapper<T> {
    fn new(value: T) -> Self {
        Wrapper { value }
    }

    fn get(&self) -> &T {
        &self.value
    }

    fn into_inner(self) -> T {
        self.value
    }
}

// Specialized impl — only for Wrapper<f64>
impl Wrapper<f64> {
    fn round(&self) -> f64 {
        self.value.round()
    }
}

// Specialized impl — only when T implements Display
impl<T: std::fmt::Display> Wrapper<T> {
    fn display(&self) {
        println!("Wrapped: {}", self.value);
    }
}

// --- Generic enum ---
#[derive(Debug)]
enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }

    fn is_right(&self) -> bool {
        matches!(self, Either::Right(_))
    }
}

impl<L: std::fmt::Display, R: std::fmt::Display> Either<L, R> {
    fn describe(&self) -> String {
        match self {
            Either::Left(val) => format!("Left({})", val),
            Either::Right(val) => format!("Right({})", val),
        }
    }
}

// --- Generic struct with multiple type parameters ---
#[derive(Debug)]
struct KeyValue<K, V> {
    key: K,
    value: V,
}

impl<K: std::fmt::Display, V: std::fmt::Debug> KeyValue<K, V> {
    fn print(&self) {
        println!("{} => {:?}", self.key, self.value);
    }
}

fn main() {
    // --- Wrapper with different types ---
    let int_w = Wrapper::new(42);
    let str_w = Wrapper::new("hello");
    let vec_w = Wrapper::new(vec![1, 2, 3]);

    println!("int: {:?}", int_w);
    println!("str: {:?}", str_w);
    println!("vec: {:?}", vec_w);

    // display() works because i32 and &str implement Display
    int_w.display();
    str_w.display();

    // round() only works on Wrapper<f64>
    let float_w = Wrapper::new(3.14159);
    println!("\nRounded: {}", float_w.round());
    // int_w.round(); // ERROR: round() is only defined for Wrapper<f64>

    // into_inner consumes the wrapper
    let value = int_w.into_inner();
    println!("Unwrapped: {}", value);
    // println!("{:?}", int_w); // ERROR: int_w was moved

    // --- Either enum ---
    let a: Either<i32, String> = Either::Left(42);
    let b: Either<i32, String> = Either::Right("hello".to_string());

    println!("\n{:?} is_left: {}", a, a.is_left());
    println!("{:?} is_right: {}", b, b.is_right());
    println!("{}", a.describe());
    println!("{}", b.describe());

    // --- KeyValue with different type combinations ---
    let kv1 = KeyValue { key: "name", value: "Alice" };
    let kv2 = KeyValue { key: "age", value: 30 };
    let kv3 = KeyValue { key: 1, value: vec![10, 20, 30] };

    println!();
    kv1.print();
    kv2.print();
    kv3.print();
}
```

## Explanation

The broken code writes `impl Wrapper<T>` without declaring `T` as a type parameter. The compiler thinks `T` is a concrete type name (like `i32` or `String`) and fails because no type named `T` exists.

**The fix: declare `T` on the `impl` block:**

```rust
impl<T> Wrapper<T> {  // <T> after impl declares the type parameter
    ...
}
```

**Specialized `impl` blocks** are powerful. You can add methods that only exist for certain types:

```rust
impl Wrapper<f64> {           // Only for Wrapper<f64>
    fn round(&self) -> f64 { ... }
}

impl<T: Display> Wrapper<T> { // Only when T implements Display
    fn display(&self) { ... }
}
```

This means `Wrapper::new(42).display()` works (because `i32: Display`), but `Wrapper::new(vec![1,2]).display()` does not (unless `Vec` implements `Display` — it does not, only `Debug`).

**Generic enums** follow the same pattern. `Option<T>` and `Result<T, E>` are generic enums:

```rust
enum Option<T> { Some(T), None }
enum Result<T, E> { Ok(T), Err(E) }
```

**Multiple type parameters:** `KeyValue<K, V>` uses two type parameters, allowing the key and value to be different types. This is the same pattern as `HashMap<K, V>`.

## ⚠️ Caution

- Forgetting `<T>` on `impl<T>` is a very common mistake. If the struct has type parameters, the `impl` block must declare them too (unless you're writing a specialized impl for a concrete type).
- `Self` in a generic `impl<T>` refers to the full type: `Self` = `Wrapper<T>`. You can use either.
- Trait bounds on `impl` blocks restrict which types get those methods. This is more fine-grained than bounds on the struct itself.

## 💡 Tips

- Use `Self` instead of repeating the full type name in method signatures.
- Specialized impls let you add type-specific behavior without affecting the generic version. This is Rust's answer to "method overloading."
- The `matches!` macro is convenient for enum variant checks: `matches!(self, Either::Left(_))`.
- Generic types with `#[derive(Debug)]` only derive `Debug` when all type parameters implement `Debug`.

## Compiler Error Interpretation

```
error[E0412]: cannot find type `T` in this scope
 --> main.rs:7:14
  |
7 | impl Wrapper<T> {
  |              ^ not found in this scope
  |
help: you might be missing a type parameter
  |
7 | impl<T> Wrapper<T> {
  |     +++
```

The compiler cannot find a type called `T`. It helpfully suggests "you might be missing a type parameter" and shows exactly where to add `<T>`. This is the fix: `impl<T> Wrapper<T>` declares `T` as a parameter, not a concrete type.

---

| [Prev: What Are Generics? — One Function, Many Types](#/katas/what-are-generics) | [Next: Multiple Trait Bounds and Where Clauses](#/katas/multiple-trait-bounds) |
