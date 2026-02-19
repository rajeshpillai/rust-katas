---
id: returning-ownership
phase: 1
phase_title: Ownership
sequence: 3
title: Returning Ownership
hints:
  - A function can give ownership to its caller by returning a value
  - If a function creates a value, the caller receives ownership of it
  - Multiple values can be returned using a tuple
---

## Description

Functions can transfer ownership to their caller by returning a value. When a function creates a `String` or any other owned type and returns it, the caller receives ownership. This is how values escape from functions — they are moved out, not copied.

## Broken Code

```rust
fn create_greeting(name: String) -> String {
    let greeting = format!("Hello, {}!", name);
    greeting
}

fn add_exclamation(s: String) -> String {
    let result = format!("{}!!", s);
    result
}

fn main() {
    let name = String::from("Rustacean");
    let greeting = create_greeting(name);
    let excited = add_exclamation(greeting);

    println!("{}", greeting);
    println!("{}", excited);
}
```

## Correct Code

```rust
fn create_greeting(name: String) -> String {
    let greeting = format!("Hello, {}!", name);
    greeting
}

fn add_exclamation(s: String) -> String {
    let result = format!("{}!!", s);
    result
}

fn main() {
    let name = String::from("Rustacean");
    let greeting = create_greeting(name);
    let excited = add_exclamation(greeting);

    println!("{}", excited);
}
```

## Explanation

In this kata, the chain of ownership is: `name` moves into `create_greeting`, which creates a new `String` and returns it. That returned value is bound to `greeting`. Then `greeting` moves into `add_exclamation`, which creates another new `String` and returns it to `excited`.

The broken code tries to print `greeting` after it has been moved into `add_exclamation`. At that point, `greeting` is gone — its ownership was transferred to the `s` parameter of `add_exclamation`.

Follow the ownership chain step by step:
1. `name` is created in `main` and moved into `create_greeting` — `name` is now invalid.
2. `create_greeting` creates a new `String` and returns it — `greeting` is now the owner.
3. `greeting` is moved into `add_exclamation` — `greeting` is now invalid.
4. `add_exclamation` creates a new `String` and returns it — `excited` is now the owner.

At the end, only `excited` is valid. Each value has exactly one owner at any point, and ownership flows linearly through the program.

The correct version simply removes the `println!("{}", greeting)` line because `greeting` no longer exists when that line would execute. If you need both values, you would need to clone `greeting` before passing it to `add_exclamation`, or restructure the code to avoid the move.

## Compiler Error Interpretation

```
error[E0382]: borrow of moved value: `greeting`
  --> main.rs:15:20
   |
13 |     let greeting = create_greeting(name);
   |         -------- move occurs because `greeting` has type `String`, which does not implement the `Copy` trait
14 |     let excited = add_exclamation(greeting);
   |                                   -------- value moved here
15 |     println!("{}", greeting);
   |                    ^^^^^^^^ value borrowed here after move
```

Again E0382 — the pattern is consistent. The compiler shows the full ownership chain: `greeting` was created on line 13, moved on line 14, and invalidly used on line 15. Notice how Rust tracks ownership across function boundaries. It does not matter that `create_greeting` and `add_exclamation` return values — the compiler understands the flow of ownership through the entire program.
