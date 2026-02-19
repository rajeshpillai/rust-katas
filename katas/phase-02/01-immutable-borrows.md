---
id: immutable-borrows
phase: 2
phase_title: Borrowing
sequence: 1
title: Immutable Borrows
hints:
  - An immutable reference &T lets you read data without taking ownership
  - You cannot modify data through an immutable reference
  - The & symbol means "I am borrowing this, not taking it"
---

## Description

Borrowing is Rust's mechanism for accessing data without taking ownership. An immutable reference (`&T`) lets you read a value while the original owner retains ownership. You cannot modify data through an immutable reference — this is the fundamental contract that makes shared access safe.

## Broken Code

```rust
fn add_greeting(name: &String) {
    name.push_str(", welcome!");
    println!("{}", name);
}

fn main() {
    let mut name = String::from("Alice");
    add_greeting(&name);
    println!("Name is still: {}", name);
}
```

## Correct Code

```rust
fn add_greeting(name: &mut String) {
    name.push_str(", welcome!");
    println!("{}", name);
}

fn main() {
    let mut name = String::from("Alice");
    add_greeting(&mut name);
    println!("Name is still: {}", name);
}
```

## Explanation

In the broken code, `add_greeting` takes `&String` — an immutable reference. This means the function promises not to modify the data. But then it calls `push_str`, which mutates the String. The compiler rejects this because it violates the borrowing contract.

The distinction between `&T` (immutable borrow) and `&mut T` (mutable borrow) is at the heart of Rust's safety model:

- `&T` means "I can read this, but I will not change it." Multiple immutable borrows can coexist because reading is safe to do concurrently.
- `&mut T` means "I can read and write this, and I am the only one who can access it right now." Only one mutable borrow can exist at a time.

The correct version changes the parameter to `&mut String` and passes `&mut name` at the call site. Both sides must agree: the function signature declares it needs mutable access, and the caller explicitly grants it.

Notice that the original `name` variable in `main` must be declared `let mut` — you cannot create a mutable reference to an immutable variable. The chain of mutability must be explicit at every level.

Also notice that after `add_greeting` returns, `name` is still usable in `main`. This is the whole point of borrowing: the function accessed the data temporarily without taking ownership. Unlike the ownership transfer patterns from Phase 1, no `.clone()` is needed, and no ownership is consumed.

## Compiler Error Interpretation

```
error[E0596]: cannot borrow `*name` as mutable, as it is behind a `&` reference
 --> main.rs:2:5
  |
2 |     name.push_str(", welcome!");
  |     ^^^^ `name` is a `&` reference, so the data it refers to cannot be borrowed as mutable
  |
help: consider changing this to be a mutable reference
  |
1 | fn add_greeting(name: &mut String) {
  |                       ~~~~~~~~~~~
```

Error E0596 tells you that you are trying to mutate through an immutable reference. The compiler identifies the exact location of the mutation attempt and traces it back to the parameter type. The suggested fix — changing to `&mut String` — is exactly right. Rust's error messages do not just say "this is wrong"; they explain the relationship between the reference type and the operation you attempted.
