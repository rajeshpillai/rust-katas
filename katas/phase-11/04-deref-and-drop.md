---
id: deref-and-drop
phase: 11
phase_title: "Memory & Performance Intuition"
sequence: 4
title: "Deref Coercion and the Drop Trait"
hints:
  - The Deref trait allows a type to behave like a reference to its inner type
  - Without Deref, calling methods on the inner type through a wrapper requires manual dereferencing
  - Rust performs deref coercion automatically -- &MyBox<String> becomes &String becomes &str
  - The Drop trait lets you run cleanup code when a value goes out of scope
---

## Description

Smart pointers in Rust work seamlessly because of two traits: `Deref` and `Drop`.

**`Deref`** enables "deref coercion" -- the compiler automatically converts `&T` to `&U` when `T: Deref<Target = U>`. This is why you can pass a `&String` where a `&str` is expected, or call `String` methods on a `Box<String>`. Without `Deref`, you would need to manually dereference every smart pointer.

**`Drop`** lets you define cleanup logic that runs when a value goes out of scope. This is Rust's mechanism for deterministic destruction -- resources like file handles, network connections, and heap memory are freed automatically and predictably.

## Broken Code

```rust
struct SmartString {
    data: String,
}

impl SmartString {
    fn new(s: &str) -> Self {
        println!("  [SmartString created: \"{}\"]", s);
        SmartString {
            data: String::from(s),
        }
    }
}

// No Deref implementation -- can't access String methods through SmartString

fn print_length(s: &str) {
    println!("  Length of '{}' is {}", s, s.len());
}

fn main() {
    let greeting = SmartString::new("Hello, Rust!");

    // Try to call a &str method through SmartString
    print_length(&greeting);

    // Try to call String methods directly
    println!("  Uppercase: {}", greeting.to_uppercase());
}
```

## Correct Code

```rust
use std::ops::Deref;
use std::fmt;

struct SmartString {
    data: String,
}

impl SmartString {
    fn new(s: &str) -> Self {
        println!("  [SmartString created: \"{}\"]", s);
        SmartString {
            data: String::from(s),
        }
    }
}

// Implement Deref so SmartString behaves like a &String (and transitively &str)
impl Deref for SmartString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

// Implement Drop for automatic cleanup logging
impl Drop for SmartString {
    fn drop(&mut self) {
        println!("  [SmartString dropped: \"{}\"]", self.data);
    }
}

// Implement Display for convenience
impl fmt::Display for SmartString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

fn print_length(s: &str) {
    println!("  Length of '{}' is {}", s, s.len());
}

fn main() {
    println!("Entering main scope:");
    let greeting = SmartString::new("Hello, Rust!");

    // Deref coercion: &SmartString -> &String -> &str
    print_length(&greeting);

    // Deref coercion lets us call String methods directly
    println!("  Uppercase: {}", greeting.to_uppercase());

    // Deref coercion also works with len(), contains(), etc.
    println!("  Contains 'Rust': {}", greeting.contains("Rust"));

    {
        println!("\n  Entering inner scope:");
        let temp = SmartString::new("temporary");
        println!("  temp = {}", temp);
        println!("  Leaving inner scope:");
    } // temp is dropped here -- Drop::drop() is called automatically

    println!("\n  Back in main scope.");
    println!("  greeting is still alive: {}", greeting);
    println!("\nLeaving main scope:");
} // greeting is dropped here
```

## Explanation

The broken version defines `SmartString` as a wrapper around `String`, but without implementing `Deref`. When you try to pass `&greeting` (a `&SmartString`) to `print_length` (which expects `&str`), the compiler cannot perform the conversion. Similarly, calling `greeting.to_uppercase()` fails because `to_uppercase` is a method on `str`, not on `SmartString`.

The correct version implements two traits:

**`Deref` with `Target = String`:**

```rust
impl Deref for SmartString {
    type Target = String;
    fn deref(&self) -> &String { &self.data }
}
```

This tells the compiler: "when you need a `&String` from a `&SmartString`, call `deref()` to get it." Because `String` itself implements `Deref<Target = str>`, the compiler can chain coercions:

```
&SmartString -> &String -> &str
```

This multi-step deref coercion happens automatically whenever a type mismatch occurs in a reference context. The rules are:

- `&T` to `&U` when `T: Deref<Target = U>`
- `&mut T` to `&mut U` when `T: DerefMut<Target = U>`
- `&mut T` to `&U` when `T: Deref<Target = U>` (mutable to immutable is always safe)

**`Drop` for automatic cleanup:**

```rust
impl Drop for SmartString {
    fn drop(&mut self) { println!("dropped: {}", self.data); }
}
```

When a `SmartString` goes out of scope, Rust automatically calls `Drop::drop()`. This happens deterministically:
- When a variable leaves its block (the `}` at the end of a scope)
- In reverse order of declaration (last created = first dropped)
- You cannot call `drop()` manually on a value -- use `std::mem::drop(value)` instead if you need to force early cleanup

The `Drop` trait is how Rust implements RAII (Resource Acquisition Is Initialization). Files close themselves, mutexes unlock themselves, memory frees itself. No garbage collector, no finalizer queue, no deferred cleanup -- resources are freed the moment they are no longer needed.

Together, `Deref` and `Drop` are what make smart pointers work in Rust. `Box<T>`, `Rc<T>`, `Arc<T>`, `MutexGuard<T>`, `String`, and `Vec<T>` all implement `Deref` (so their inner values are accessible transparently) and `Drop` (so their resources are cleaned up automatically).

## Compiler Error Interpretation

```
error[E0308]: mismatched types
  --> src/main.rs:22:18
   |
22 |     print_length(&greeting);
   |     ------------ ^^^^^^^^^ expected `&str`, found `&SmartString`
   |     |
   |     arguments to this function are incorrect
   |
   = note: expected reference `&str`
              found reference `&SmartString`
```

The compiler expected `&str` but received `&SmartString`. Without `Deref`, these are unrelated types and no automatic conversion exists. After implementing `Deref<Target = String>`, the compiler can coerce `&SmartString` to `&String` to `&str` automatically, and the error disappears. When you see a type mismatch between a wrapper type and its inner type, implementing `Deref` is usually the answer.
