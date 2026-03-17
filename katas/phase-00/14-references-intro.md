---
id: references-intro
phase: 0
phase_title: "Rust as a Language"
sequence: 14
title: "References — Pointers Without the Danger"
hints:
  - "A reference `&x` lets you read data without taking ownership. The original variable stays valid."
  - "A mutable reference `&mut x` lets you modify data without taking ownership — but only one mutable reference at a time."
  - "References are not the same as ownership transfer. `&x` borrows, `x` moves."
---

## Description

A **reference** is a pointer to data owned by someone else. Unlike raw pointers in C, Rust references are always valid — the compiler guarantees they never dangle.

- `&T` — an immutable reference (you can read but not modify)
- `&mut T` — a mutable reference (you can read and modify)

References let you pass data to functions without giving up ownership. This is the foundation for Rust's borrowing system (Phase 2), but understanding what references **are** comes first.

## Broken Code

```rust
fn calculate_length(s: String) -> usize {
    s.len()
}

fn main() {
    let name = String::from("Rust");

    let len = calculate_length(name);
    println!("{} has {} characters", name, len);
    // Bug: name was moved into calculate_length — can't use it here
}
```

## Correct Code

```rust
// Takes a reference — borrows the String without taking ownership
fn calculate_length(s: &String) -> usize {
    s.len()
}

// Takes a mutable reference — can modify the data
fn add_exclamation(s: &mut String) {
    s.push('!');
}

// Prefer &str over &String for read-only string access
fn first_char(s: &str) -> Option<char> {
    s.chars().next()
}

fn main() {
    // --- Immutable references: &T ---
    let name = String::from("Rust");

    // &name creates a reference — name is borrowed, not moved
    let len = calculate_length(&name);
    println!("'{}' has {} bytes", name, len); // name is still valid!

    // Multiple immutable references are fine
    let r1 = &name;
    let r2 = &name;
    println!("r1: {}, r2: {}", r1, r2);

    // --- Mutable references: &mut T ---
    let mut greeting = String::from("Hello");
    println!("Before: {}", greeting);

    add_exclamation(&mut greeting);
    println!("After: {}", greeting); // "Hello!"

    // --- References to other types ---
    let numbers = vec![10, 20, 30, 40, 50];

    // Reference to a Vec — doesn't take ownership
    let sum: i32 = numbers.iter().sum();
    println!("\nNumbers: {:?}", numbers); // Still accessible
    println!("Sum: {}", sum);

    // Reference to a single element
    let first: &i32 = &numbers[0];
    println!("First element: {}", first);

    // --- The & and * operators ---
    let x: i32 = 42;
    let r: &i32 = &x;       // & creates a reference
    let val: i32 = *r;      // * dereferences (follows the pointer)
    println!("\nx = {}, *r = {}, val = {}", x, *r, val);

    // Usually you don't need * — Rust auto-dereferences
    println!("r = {} (auto-deref)", r); // Rust auto-derefs for Display

    // --- References with functions ---
    let text = String::from("hello world");
    // &String auto-coerces to &str, so this works:
    println!("\nFirst char of '{}': {:?}", text, first_char(&text));
    // &str directly:
    println!("First char of literal: {:?}", first_char("goodbye"));

    // --- Visualizing references ---
    let owner = String::from("data");
    let ref1 = &owner;    // ref1 points to owner
    let ref2 = &owner;    // ref2 also points to owner

    println!("\nOwner: '{}' at {:p}", owner, &owner);
    println!("ref1:  '{}' points to {:p}", ref1, ref1 as *const String);
    println!("ref2:  '{}' points to {:p}", ref2, ref2 as *const String);
    println!("Same address? {}", std::ptr::eq(ref1, ref2));
}
```

## Explanation

The broken code passes `name` by value to `calculate_length`. In Rust, passing a `String` by value **moves** it — the function takes ownership, and the caller can no longer use it. After `calculate_length(name)`, `name` is gone.

**The fix: pass a reference instead.**

```rust
fn calculate_length(s: &String) -> usize  // Borrows, doesn't own
    ...
let len = calculate_length(&name);        // & creates a reference
```

Now `calculate_length` borrows `name` temporarily. When the function returns, the borrow ends and `name` is still usable.

**Reference vs ownership:**

| Passing style | Syntax | Ownership | After call |
|---|---|---|---|
| By value | `fn f(s: String)` | Moves to function | Caller loses access |
| By reference | `fn f(s: &String)` | Borrowed | Caller keeps access |
| By mut reference | `fn f(s: &mut String)` | Mutably borrowed | Caller keeps access |

**The `&` and `*` operators:**
- `&x` creates a reference to `x` (like "take the address of")
- `*r` dereferences `r` (like "follow the pointer to the value")

In practice, you rarely need `*` because Rust **auto-dereferences** in most contexts — calling methods, printing, and comparing all work through references automatically.

**Deref coercion:** When you pass `&String` to a function expecting `&str`, Rust automatically converts it. This is called deref coercion and is why `&str` is preferred in function signatures — it accepts both `String` references and string literals.

**This kata introduces the concept.** Phase 2 (Borrowing) will teach the **rules** — why you can't have `&mut T` while `&T` exists, and why these restrictions prevent bugs.

## ⚠️ Caution

- A reference must not outlive the data it points to. Returning `&local_variable` from a function is a compile error (the data would be freed).
- You cannot modify data through an `&T` reference. Use `&mut T` for modification.
- Only one `&mut T` reference can exist at a time. This prevents data races — but that's a Phase 2 topic.

## 💡 Tips

- Think of `&T` as "I'm looking at your data" and `&mut T` as "I'm temporarily modifying your data."
- Prefer `&str` over `&String`, and `&[T]` over `&Vec<T>` in function parameters — they're more flexible.
- References are zero-cost at runtime — they're just pointers. The safety guarantees are all compile-time.
- Rust's auto-deref means you almost never need to write `*` explicitly. Trust the compiler.

## Compiler Error Interpretation

```
error[E0382]: borrow of moved value: `name`
  --> main.rs:8:40
   |
5  |     let name = String::from("Rust");
   |         ---- move occurs because `name` has type `String`, which does not implement the `Copy` trait
6  |
7  |     let len = calculate_length(name);
   |                                ---- value moved here
8  |     println!("{} has {} characters", name, len);
   |                                      ^^^^ value borrowed here after move
```

This is one of Rust's most important errors:

- **"move occurs because `name` has type `String`"** — `String` does not implement `Copy`, so passing it by value moves it.
- **"value moved here"** — `calculate_length(name)` took ownership of `name`.
- **"value borrowed here after move"** — `println!` tries to use `name`, but it was already moved.

The fix: pass `&name` (a reference) instead of `name` (the value). The function gets read access without taking ownership.
