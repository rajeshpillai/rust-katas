---
id: lifetime-elision
phase: 3
phase_title: Lifetimes
sequence: 4
title: Lifetime Elision
hints:
  - The compiler applies three rules to infer lifetimes automatically
  - Elision fails when there are multiple input references and the output lifetime is ambiguous
  - When elision cannot determine the output lifetime, you must annotate explicitly
---

## Description

Rust has three lifetime elision rules that let the compiler infer lifetimes in common cases, so you do not have to annotate them. But when these rules are not enough — typically when a function takes multiple references and returns a reference — you must write explicit lifetime annotations. Understanding the elision rules helps you predict when annotations are needed.

## Broken Code

```rust
fn first_word_of(s: &str, prefix: &str) -> &str {
    if s.starts_with(prefix) {
        &s[..prefix.len()]
    } else {
        &s[..1]
    }
}

fn main() {
    let sentence = String::from("hello world");
    let prefix = "hel";
    let word = first_word_of(&sentence, prefix);
    println!("Result: {}", word);
}
```

## Correct Code

```rust
fn first_word_of<'a>(s: &'a str, prefix: &str) -> &'a str {
    if s.starts_with(prefix) {
        &s[..prefix.len()]
    } else {
        &s[..1]
    }
}

fn main() {
    let sentence = String::from("hello world");
    let prefix = "hel";
    let word = first_word_of(&sentence, prefix);
    println!("Result: {}", word);
}
```

## Explanation

The three lifetime elision rules are:

1. **Each input reference gets its own lifetime parameter.** A function with two `&str` parameters is treated as having two different lifetimes: `fn f<'a, 'b>(s: &'a str, prefix: &'b str)`.

2. **If there is exactly one input lifetime parameter, it is assigned to all output references.** For example, `fn f(s: &str) -> &str` becomes `fn f<'a>(s: &'a str) -> &'a str` automatically.

3. **If one of the input parameters is `&self` or `&mut self`, the lifetime of self is assigned to all output references.** This makes methods on structs ergonomic.

In the broken code, the function has two input references (`s` and `prefix`). Rule 1 gives them separate lifetimes. Rule 2 does not apply because there are two input lifetimes, not one. Rule 3 does not apply because this is not a method. So the compiler cannot determine the output lifetime — it does not know whether the returned `&str` borrows from `s` or from `prefix`.

The correct version resolves this by explicitly annotating: `s` gets lifetime `'a`, the return type gets lifetime `'a`, and `prefix` gets no named lifetime (it is unrelated to the output). This tells the compiler: "the returned reference borrows from `s`, not from `prefix`."

Notice that `prefix` does not need `'a` — it has its own anonymous lifetime. The returned reference never points into `prefix`, so there is no relationship to declare. Being precise about which lifetimes are connected (and which are not) gives the compiler the information it needs to verify safety.

Understanding elision rules has a practical benefit: when you see a function signature without lifetime annotations and it compiles, you know exactly what relationships the compiler inferred. And when you see explicit annotations, you know the elision rules were not sufficient and the programmer had to clarify the relationships manually.

## Compiler Error Interpretation

```
error[E0106]: missing lifetime specifier
 --> main.rs:1:44
  |
1 | fn first_word_of(s: &str, prefix: &str) -> &str {
  |                     ----          ----      ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `s` or `prefix`
help: consider introducing a named lifetime parameter
  |
1 | fn first_word_of<'a>(s: &'a str, prefix: &'a str) -> &'a str {
  |                 ++++     ++                ++          ++
```

Error E0106 asks the essential question: "is the return value borrowed from `s` or `prefix`?" The compiler cannot decide, so it asks you. The suggested fix ties all references to the same lifetime `'a`, which is safe but overly restrictive — it says the return value is related to both inputs. The better fix (shown in the correct code) only ties the return value to `s`, because that is the actual relationship. The compiler's suggestion is valid but imprecise; your annotations should reflect the true semantics of the function.
