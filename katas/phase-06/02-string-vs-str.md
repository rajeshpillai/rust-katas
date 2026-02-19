---
id: string-vs-str
phase: 6
phase_title: "Collections & the Owned/Borrowed Duality"
sequence: 2
title: "String (Owned) vs &str (Borrowed)"
hints:
  - "A `&str` is a reference to string data owned by someone else. It cannot outlive the owner."
  - "If the string data is created inside a function (as a local `String`), a `&str` pointing into it becomes a dangling reference when the function returns."
  - "Either return an owned `String`, or borrow from data that the caller owns."
---

## Description

`String` and `&str` are the most fundamental owned/borrowed pair in Rust:

- **`String`** is a heap-allocated, growable, owned UTF-8 string. It owns its buffer.
- **`&str`** is a borrowed slice of UTF-8 data. It is a fat pointer: a pointer to the data plus a length. It does not own the data.

The relationship mirrors `Vec<T>` and `&[T]`. A `String` can be borrowed as `&str` (cheaply, via deref coercion), but a `&str` cannot magically become a `String` without allocating (you must call `.to_string()` or `.to_owned()`).

This kata explores the dangling reference trap: creating a `String` inside a function and trying to return a `&str` that points into it.

## Broken Code

```rust
fn greeting(name: &str) -> &str {
    // `full_greeting` is a local String, allocated on the heap
    // but owned by this function's stack frame.
    let full_greeting = format!("Hello, {}! Welcome aboard.", name);

    // We try to return a &str that borrows from `full_greeting`.
    // But `full_greeting` will be dropped when this function returns.
    // The returned &str would point to freed memory.
    &full_greeting
}

fn main() {
    let msg = greeting("Alice");
    println!("{}", msg);
}
```

## Correct Code

```rust
// Solution 1: Return an owned String.
// The caller receives ownership of the data.
fn greeting(name: &str) -> String {
    format!("Hello, {}! Welcome aboard.", name)
}

fn main() {
    let msg = greeting("Alice");
    println!("{}", msg);
}
```

## Explanation

The broken version creates a `String` (`full_greeting`) inside the function, then tries to return a `&str` slice of it. The problem is a lifetime violation:

1. `full_greeting` is a local variable. Its heap buffer is allocated when `format!` runs.
2. `&full_greeting` creates a `&str` borrowing the heap buffer.
3. The function returns. `full_greeting` is dropped. Its heap buffer is deallocated.
4. The returned `&str` now points to freed memory — a dangling reference.

Rust prevents this at compile time. The lifetime of the returned `&str` would need to be at least as long as the caller's scope, but the data it borrows from (`full_greeting`) only lives until the function returns.

**Solution 1 (return `String`):** The simplest fix. Return an owned `String` so the caller gets ownership of the data. There is no borrowing, no lifetime issue. This is the most common approach.

**Solution 2 (borrow from input):** If the function does not need to create new data, it can return a `&str` that borrows from the input:

```rust
fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}
```

This works because the returned `&str` borrows from `s`, which the caller owns. The lifetime of the return value is tied to the lifetime of the input, and the compiler can verify this.

**Solution 3 (use `Cow<str>` for flexibility):** When a function sometimes returns borrowed data and sometimes needs to allocate:

```rust
use std::borrow::Cow;

fn maybe_transform(s: &str) -> Cow<str> {
    if s.contains("world") {
        // Returns borrowed — no allocation
        Cow::Borrowed(s)
    } else {
        // Returns owned — allocates a new String
        Cow::Owned(format!("{} (modified)", s))
    }
}
```

**The owned/borrowed duality for strings:**

| Type | Owns data | Growable | Common use |
|---|---|---|---|
| `String` | Yes | Yes | Building strings, storing in structs |
| `&str` | No | No | Reading strings, function parameters |
| `Cow<str>` | Sometimes | Via owned variant | Functions that may or may not allocate |

**Best practice for function parameters:** Accept `&str` when you only need to read a string. This is more flexible than accepting `&String`, because `&str` can come from a `String`, a string literal, a slice, or any other source of UTF-8 data.

## Compiler Error Interpretation

```
error[E0515]: cannot return reference to local variable `full_greeting`
 --> src/main.rs:6:5
  |
6 |     &full_greeting
  |     ^^^^^^^^^^^^^^ returns a reference to data owned by the current function
```

This is one of Rust's most concise and clear error messages:

- **"cannot return reference to local variable `full_greeting`"** — The compiler knows exactly what is wrong: you are trying to create a reference that outlives the data it points to.
- **"returns a reference to data owned by the current function"** — The data is owned locally. When the function exits, the data is destroyed. A reference to it would dangle.

The error code `E0515` specifically covers the case of returning references to local data. The fix is always one of:
1. Return an owned value instead of a reference.
2. Restructure the code so the reference points to data with a longer lifetime (e.g., data passed in by the caller).

There is no way to "extend" the lifetime of a local variable. Lifetimes are not flexible — they are determined by scope.
