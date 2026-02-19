---
id: structs-holding-references
phase: 3
phase_title: Lifetimes
sequence: 2
title: Structs Holding References
hints:
  - A struct that holds a reference must declare a lifetime parameter
  - The lifetime parameter ensures the reference inside the struct stays valid
  - Without the lifetime, the compiler cannot verify the struct does not outlive its data
---

## Description

When a struct holds a reference, it must declare a lifetime parameter. This tells the compiler: "an instance of this struct cannot outlive the data it references." Without this annotation, the struct could hold a dangling reference — a pointer to memory that has been freed.

## Broken Code

```rust
struct Excerpt {
    text: &str,
}

fn main() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence;

    {
        let words = novel.as_str();
        first_sentence = Excerpt {
            text: &words[..16],
        };
    }

    println!("Excerpt: {}", first_sentence.text);
}
```

## Correct Code

```rust
struct Excerpt<'a> {
    text: &'a str,
}

fn main() {
    let novel = String::from("Call me Ishmael. Some years ago...");

    let first_sentence = Excerpt {
        text: &novel[..16],
    };

    println!("Excerpt: {}", first_sentence.text);
}
```

## Explanation

The broken code has two problems. First, the struct `Excerpt` holds a `&str` but has no lifetime parameter. The compiler requires one because it needs to track the relationship between the struct instance and the data it borrows.

Second, even with the lifetime annotation, the original `main` function structure is problematic. The `words` variable is created inside an inner block, and the `Excerpt` borrows from it. When the inner block ends, if `words` were a new `String`, the reference would dangle. However, `words` here is just `novel.as_str()`, which borrows from `novel` — so the reference actually points to `novel`'s data, which lives in the outer scope. The inner block is unnecessarily confusing.

The correct version simplifies the code: the struct declares `<'a>` and the field uses `&'a str`, establishing the contract. The `Excerpt` borrows directly from `novel`, and both live in the same scope.

The lifetime `'a` on the struct means: "for any instance of `Excerpt<'a>`, the reference in `text` is valid for at least the lifetime `'a`." In practice, this means the compiler will prevent you from using an `Excerpt` after the data it references has been dropped.

Think of the lifetime parameter as a **promise**: the struct promises not to outlive its data. The compiler holds the struct to that promise at every use site.

When should a struct hold a reference vs an owned value?
- Use `&'a str` when the struct is a temporary view into existing data (like an excerpt, a parser token, or a search result).
- Use `String` when the struct should own its data independently.

## Compiler Error Interpretation

```
error[E0106]: missing lifetime specifier
 --> main.rs:2:11
  |
2 |     text: &str,
  |           ^ expected named lifetime parameter
  |
help: consider introducing a named lifetime parameter
  |
1 ~ struct Excerpt<'a> {
2 ~     text: &'a str,
  |
```

Error E0106 again — the same error code as with function signatures. The compiler requires a lifetime parameter on any struct that holds a reference. The suggested fix adds `<'a>` to the struct definition and `'a` to the reference. This pattern is mechanical: every `&` in a struct field needs a lifetime, and every lifetime must be declared on the struct itself.
