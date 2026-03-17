---
id: string-basics
phase: 0
phase_title: "Rust as a Language"
sequence: 13
title: "String Basics — String vs &str"
hints:
  - "Rust has two main string types: `String` (owned, heap-allocated, growable) and `&str` (borrowed, immutable reference to string data)."
  - "String literals like `\"hello\"` are `&str`, not `String`. Use `.to_string()` or `String::from()` to create an owned `String`."
  - "You cannot index a `String` by position with `s[0]` because Rust strings are UTF-8 and characters can be multi-byte."
---

## Description

Rust has two primary string types:

- **`String`** — An owned, heap-allocated, growable string. You can modify it.
- **`&str`** — A borrowed, immutable reference to string data (a "string slice"). String literals are `&str`.

This duality (`String` / `&str`) follows the same owned/borrowed pattern as `Vec<T>` / `&[T]`. Understanding it early makes the rest of Rust much clearer.

## Broken Code

```rust
fn main() {
    let greeting = "hello";

    // Bug 1: push_str is a String method, not available on &str
    greeting.push_str(", world!");

    // Bug 2: cannot index strings by byte position
    let first = greeting[0];

    println!("{}", greeting);
}
```

## Correct Code

```rust
fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(pos) => &s[..pos],
        None => s,
    }
}

fn main() {
    // --- Creating strings ---
    let literal: &str = "hello";               // String literal: &str
    let owned: String = String::from("hello");  // Owned String
    let also_owned: String = "hello".to_string(); // Another way
    println!("literal: {}, owned: {}, also_owned: {}", literal, owned, also_owned);

    // --- Converting between String and &str ---
    let s: String = String::from("hello world");
    let slice: &str = &s;            // String -> &str (borrows)
    let owned_again: String = slice.to_string(); // &str -> String (copies)
    println!("owned: {}, slice: {}, copy: {}", s, slice, owned_again);

    // --- Modifying Strings (only String, not &str) ---
    let mut greeting = String::from("hello");
    greeting.push_str(", world!");     // Append a string slice
    greeting.push('!');                // Append a single character
    println!("{}", greeting);

    // --- String methods ---
    let text = "  Hello, Rust!  ";
    println!("trim: '{}'", text.trim());
    println!("to_uppercase: '{}'", text.trim().to_uppercase());
    println!("to_lowercase: '{}'", text.trim().to_lowercase());
    println!("contains 'Rust': {}", text.contains("Rust"));
    println!("starts_with 'Hello': {}", text.trim().starts_with("Hello"));
    println!("len (bytes): {}", text.trim().len());

    // --- Splitting ---
    let csv = "apple,banana,cherry";
    let fruits: Vec<&str> = csv.split(',').collect();
    println!("\nSplit '{}': {:?}", csv, fruits);

    // Split and iterate
    for (i, word) in "one two three".split_whitespace().enumerate() {
        println!("  word {}: {}", i, word);
    }

    // --- String concatenation ---
    let first = String::from("Hello");
    let second = String::from(" World");

    // Option 1: format! macro (preferred — clear and flexible)
    let combined = format!("{}{}", first, second);
    println!("\nformat!: {}", combined);

    // Option 2: + operator (consumes the left operand)
    let combined = first + &second;  // first is moved!
    println!("+: {}", combined);
    // println!("{}", first);  // ERROR: first was moved

    // --- Parsing strings to numbers ---
    let num_str = "42";
    let num: i32 = num_str.parse().expect("not a number");
    println!("\nParsed '{}' to {}", num_str, num);

    // Parse with error handling
    for s in &["100", "abc", "3.14", "0"] {
        match s.parse::<i32>() {
            Ok(n) => println!("  '{}' => {}", s, n),
            Err(e) => println!("  '{}' => error: {}", s, e),
        }
    }

    // --- Why you can't index: UTF-8 ---
    let emoji = "Hello 🦀";
    println!("\n'{}' is {} bytes but {} characters",
        emoji, emoji.len(), emoji.chars().count());

    // Iterate over characters (not bytes)
    for (i, ch) in emoji.chars().enumerate() {
        println!("  char {}: '{}' ({} bytes)", i, ch, ch.len_utf8());
    }

    // --- Functions accepting &str work with both types ---
    let owned = String::from("hello world");
    let literal = "goodbye world";
    println!("\nFirst word of '{}': '{}'", owned, first_word(&owned));
    println!("First word of '{}': '{}'", literal, first_word(literal));
}
```

## Explanation

The broken code has two bugs:

**Bug 1: Calling `push_str` on `&str`.** String literals are `&str` — immutable, borrowed references. You cannot modify them. `push_str` is a method on `String` (the owned type). You must create a `String` first.

**Bug 2: Indexing with `s[0]`.** Rust strings are UTF-8 encoded. A single character might be 1, 2, 3, or 4 bytes. `s[0]` would return a byte, not a character — which is almost never what you want. Rust prevents this ambiguity by disallowing string indexing entirely.

**The two string types:**

| | `String` | `&str` |
|---|---|---|
| Ownership | Owned | Borrowed |
| Location | Heap | Anywhere (stack, heap, binary) |
| Mutable? | Yes (if `mut`) | No |
| Growable? | Yes | No |
| Created by | `String::from()`, `.to_string()` | Literals, `&string[..]` |

**Function parameters:** Prefer `&str` over `&String`. A function taking `&str` accepts both `&String` (auto-deref) and `&str` (string literals). This is more flexible:

```rust
fn greet(name: &str) { ... }  // Accepts String and &str
greet(&my_string);              // Works
greet("literal");               // Also works
```

**String iteration:** Use `.chars()` for characters, `.bytes()` for raw bytes. The `len()` method returns bytes, not characters.

## ⚠️ Caution

- `String` + `&str` works (`"Hello".to_string() + " world"`), but consumes the left `String`. Use `format!` to avoid ownership issues.
- `.len()` returns **bytes**, not characters. `"🦀".len()` is 4, not 1. Use `.chars().count()` for character count.
- Slicing with `&s[0..n]` works but panics if `n` falls in the middle of a multi-byte character. Use `.char_indices()` for safe slicing.

## 💡 Tips

- `format!("{} {}", a, b)` is the safest way to concatenate — it does not consume any arguments.
- `.trim()`, `.split()`, `.contains()`, `.replace()` all work on both `String` and `&str`.
- `.parse::<T>()` converts a string to any type that implements `FromStr` (numbers, booleans, etc.).
- Prefer `&str` in function signatures for maximum flexibility. Only use `String` when the function needs ownership.

## Compiler Error Interpretation

```
error[E0599]: no method named `push_str` found for reference `&str` in the current scope
 --> main.rs:4:14
  |
4 |     greeting.push_str(", world!");
  |              ^^^^^^^^ method not found in `&str`
```

`push_str` exists on `String`, not `&str`. The error says "method not found in `&str`" — the type is a borrowed reference, which cannot be modified. Create a `String` with `String::from("hello")` or `"hello".to_string()` first.

For indexing:

```
error[E0277]: the type `str` cannot be indexed by `{integer}`
 --> main.rs:7:17
  |
7 |     let first = greeting[0];
  |                 ^^^^^^^^^^^ string indices are ranges of `usize`
  |
  = help: the trait `SliceIndex<str>` is not implemented for `{integer}`
```

The compiler blocks integer indexing on strings because UTF-8 makes byte-level access unsafe. Use `.chars().nth(0)` for the first character, or `.as_bytes()[0]` if you explicitly want the first byte.
