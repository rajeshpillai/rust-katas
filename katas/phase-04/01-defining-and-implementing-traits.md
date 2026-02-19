---
id: defining-and-implementing-traits
phase: 4
phase_title: "Traits & Generics"
sequence: 1
title: Defining and Implementing Traits
hints:
  - "Check whether the type you're calling the method on actually implements the trait."
  - "A trait is just a contract — a type must explicitly opt in with an `impl TraitName for TypeName` block."
  - "Look at the compiler error: it will tell you exactly which trait is missing for which type."
---

## Description

Traits in Rust define shared behavior. They are similar to interfaces in other languages, but with important differences: a type does not implement a trait unless you explicitly write an `impl` block for it. The compiler enforces this strictly — you cannot call a trait method on a type that has not opted in.

This kata teaches you to define a trait, and then observe what happens when you try to use it on a type that lacks the implementation.

## Broken Code

```rust
trait Greet {
    fn hello(&self) -> String;
}

struct User {
    name: String,
}

// No impl Greet for User — we forgot to implement the trait!

fn main() {
    let user = User {
        name: String::from("Alice"),
    };

    // This will fail: User does not implement Greet
    let greeting = user.hello();
    println!("{}", greeting);
}
```

## Correct Code

```rust
trait Greet {
    fn hello(&self) -> String;
}

struct User {
    name: String,
}

impl Greet for User {
    fn hello(&self) -> String {
        format!("Hello, my name is {}!", self.name)
    }
}

fn main() {
    let user = User {
        name: String::from("Alice"),
    };

    let greeting = user.hello();
    println!("{}", greeting);
}
```

## Explanation

In the broken version, we defined the `Greet` trait with a method `hello`, and we created a `User` struct, but we never wrote the `impl Greet for User` block. Rust does not automatically connect traits to types. Unlike some languages where a type satisfies an interface implicitly if it has matching methods, Rust requires an explicit `impl` block.

This is a deliberate design choice. It means:

1. **You always know which traits a type implements** by looking at its `impl` blocks.
2. **The compiler catches mismatches early** — if you forget to implement a trait, you get a clear error at the call site.
3. **Traits are opt-in contracts**, not accidental structural matches.

The fix is straightforward: add the `impl Greet for User` block and provide the body of the `hello` method. Once this block exists, Rust knows that `User` satisfies the `Greet` contract, and the method call compiles.

Note that the trait must also be **in scope** at the call site. If `Greet` were defined in another module, you would need to `use` it before calling `user.hello()`.

## Compiler Error Interpretation

```
error[E0599]: no method named `hello` found for struct `User` in the current scope
 --> src/main.rs:14:28
  |
5 | struct User {
  | ----------- method `hello` not found for this struct
...
14|     let greeting = user.hello();
  |                         ^^^^^ method not found in `User`
  |
  = help: items from traits can only be used if the trait is implemented and in scope
  = note: the following trait defines an item `hello`, perhaps you need to implement it:
          candidate #1: `Greet`
```

This error tells you several things:

- **"no method named `hello` found for struct `User`"** — Rust looked at `User` and could not find a method called `hello`, neither as an inherent method nor via any implemented trait.
- **"items from traits can only be used if the trait is implemented and in scope"** — Rust knows `hello` exists on the `Greet` trait but `User` has not implemented it.
- **"candidate #1: `Greet`"** — The compiler is even helpful enough to suggest which trait you probably meant to implement. This is the compiler being your ally.
