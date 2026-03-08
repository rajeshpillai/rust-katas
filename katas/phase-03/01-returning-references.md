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
    let string2 = String::from("xyz");

    let result = longest(string1.as_str(), string2.as_str());
    println!("The longest string is: {}", result);
}
```

## Explanation

The broken code has two problems.

**Problem 1: Missing lifetime annotations.** The `longest` function takes two `&str` references and returns a `&str`, but the compiler cannot figure out which input the output borrows from. This is a case where lifetime elision rules do not apply — there are two input references, so the compiler cannot pick one automatically. You must add `<'a>` to declare a lifetime parameter and annotate the references to establish the relationship.

**Problem 2: The inner block creates a dangling reference.** In the broken code, `string2` is created inside an inner block and dropped when that block ends. But `result` — which might point to `string2`'s data — is used after the block. Even with correct lifetime annotations, this would not compile because the returned reference is constrained to the shorter of the two input lifetimes, and `string2`'s lifetime ends at the closing `}` of the inner block.

The correct version fixes both problems:

1. The function signature `fn longest<'a>(x: &'a str, y: &'a str) -> &'a str` tells the compiler: "the returned reference lives at least as long as the shorter of `x` and `y`."
2. Both `string1` and `string2` are declared in the same scope, so they both outlive `result`. The lifetime constraint is satisfied.

The key insight: **lifetimes are not about how long something lives. They are about what relates to what.** The annotation `'a` says "these references are connected" — and the compiler uses that connection to prevent dangling references.

## ⚠️ Caution

- **Lifetime annotations do not change how long values live.** They only describe relationships. Adding `'a` does not extend a value's lifetime — it tells the compiler how references are connected so it can verify safety.
- **The return lifetime is constrained to the shortest input lifetime.** When you write `fn longest<'a>(x: &'a str, y: &'a str) -> &'a str`, the returned reference is only valid as long as both inputs are valid. If one input is dropped, the return value becomes invalid.

## 💡 Tips

- If a function always returns from one specific input, only annotate that input's lifetime: `fn first<'a>(s: &'a str, _prefix: &str) -> &'a str`. This gives the compiler more precise information.
- When lifetime annotations feel confusing, ask: "which input could the return value point to?" The answer tells you which lifetimes to connect.
- Lifetime syntax `'a` is pronounced "tick a" or "lifetime a." It is just a name — you can use `'input`, `'src`, etc. for readability.

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
