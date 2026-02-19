---
id: returning-references
phase: 3
phase_title: Lifetimes
sequence: 1
title: Returning References
hints:
  - When a function returns a reference, the compiler needs to know which input it relates to
  - Lifetime parameters describe the relationship between input and output references
  - The syntax 'a (tick a) names a lifetime
---

## Description

When a function takes multiple references as input and returns a reference, the compiler needs to know: which input does the output reference relate to? Lifetime parameters answer this question. They do not change how long values live — they describe relationships between references so the compiler can verify safety.

## Broken Code

```rust
fn longest(x: &str, y: &str) -> &str {
    if x.len() >= y.len() {
        x
    } else {
        y
    }
}

fn main() {
    let result;
    let string1 = String::from("long string");

    {
        let string2 = String::from("xyz");
        result = longest(string1.as_str(), string2.as_str());
    }

    println!("The longest string is: {}", result);
}
```

## Correct Code

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() {
        x
    } else {
        y
    }
}

fn main() {
    let string1 = String::from("long string");
    let result;

    {
        let string2 = String::from("xyz");
        result = longest(string1.as_str(), string2.as_str());
    }

    println!("The longest string is: {}", result);
}
```

## Explanation

The first problem in the broken code is that the `longest` function signature has no lifetime annotations. The compiler cannot figure out the relationship between the input references and the output reference on its own — this is a case where lifetime elision rules do not apply (because there are two input references and the compiler cannot know which one the return value borrows from).

Adding `<'a>` to the function and annotating all references with `'a` tells the compiler: "The returned reference will live at least as long as the shorter of the two input references." This is a constraint, not a duration. The compiler uses it to check that every call site respects this relationship.

But even with correct lifetime annotations, the `main` function in both versions has a subtlety. The call `longest(string1.as_str(), string2.as_str())` means the returned reference is constrained to the shorter lifetime — which is `string2`'s lifetime (it is dropped at the end of the inner block). If `result` holds a reference borrowed from `string2` and we use `result` outside that block, we would have a dangling reference.

In this particular case, the correct code will still fail to compile because `result` is used after `string2` is dropped. The lifetime `'a` is constrained to the shorter of the two inputs, which is `string2`'s scope. To truly fix this, you would need to ensure `result` is used within the inner block, or ensure both strings live long enough:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() {
        x
    } else {
        y
    }
}

fn main() {
    let string1 = String::from("long string");
    let string2 = String::from("xyz");

    let result = longest(string1.as_str(), string2.as_str());
    println!("The longest string is: {}", result);
}
```

This version works because both `string1` and `string2` outlive `result`.

The key insight: **lifetimes are not about how long something lives. They are about what relates to what.** The annotation `'a` says "these references are connected" — and the compiler uses that connection to prevent dangling references.

## Compiler Error Interpretation

```
error[E0106]: missing lifetime specifier
 --> main.rs:1:33
  |
1 | fn longest(x: &str, y: &str) -> &str {
  |               ----     ----      ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `x` or `y`
help: consider introducing a named lifetime parameter
  |
1 | fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
  |           ++++     ++          ++           ++
```

Error E0106 is remarkably helpful. The compiler explains the fundamental problem: "the return type contains a borrowed value, but the signature does not say whether it is borrowed from `x` or `y`." It then suggests the exact fix — adding a lifetime parameter `'a` to connect the inputs and output. The compiler is teaching you what lifetimes mean: they are relationships between references.
