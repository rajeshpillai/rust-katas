---
id: box-heap-allocation
phase: 11
phase_title: "Memory & Performance Intuition"
sequence: 1
title: "Box and Heap Allocation"
hints:
  - Recursive types have infinite size at compile time because the compiler cannot determine where the recursion ends
  - Box<T> is a pointer to heap-allocated data -- it has a known, fixed size (one pointer width)
  - Wrapping the recursive field in Box gives the type a finite, known size
  - Box provides heap allocation with zero runtime overhead beyond the allocation itself
---

## Description

Every value in Rust has a size known at compile time. The compiler must know exactly how many bytes each type occupies on the stack. This creates a problem for recursive types: a type that contains itself has theoretically infinite size. A linked list node, for example, contains a value and another node, which contains a value and another node, and so on.

`Box<T>` solves this by allocating the contained value on the heap and storing only a pointer on the stack. A pointer has a fixed, known size (8 bytes on 64-bit systems), so the compiler can determine the size of the outer type.

## Broken Code

```rust
// A cons list: each node holds a value and the rest of the list
enum List {
    Cons(i32, List),
    Nil,
}

use List::{Cons, Nil};

fn main() {
    let list = Cons(1, Cons(2, Cons(3, Nil)));

    print_list(&list);
}

fn print_list(list: &List) {
    match list {
        Cons(value, rest) => {
            print!("{} -> ", value);
            print_list(rest);
        }
        Nil => println!("Nil"),
    }
}
```

## Correct Code

```rust
// Wrap the recursive field in Box to give it a known size
enum List {
    Cons(i32, Box<List>),
    Nil,
}

use List::{Cons, Nil};

fn main() {
    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));

    print_list(&list);
}

fn print_list(list: &List) {
    match list {
        Cons(value, rest) => {
            print!("{} -> ", value);
            print_list(rest);
        }
        Nil => println!("Nil"),
    }
}
```

## Explanation

The broken version defines `List` as an enum where the `Cons` variant contains another `List` directly. The compiler tries to calculate the size of `List`:

- `List` = max(size of `Cons`, size of `Nil`)
- Size of `Cons` = size of `i32` + size of `List`
- Size of `List` = max(size of `Cons`, size of `Nil`)
- Size of `Cons` = size of `i32` + size of `List`
- ... (infinite recursion)

The compiler cannot determine the size, so it rejects the type.

The correct version wraps the recursive field in `Box<List>`. Now:

- Size of `Cons` = size of `i32` + size of `Box<List>`
- Size of `Box<List>` = size of a pointer (8 bytes on 64-bit)
- Size of `Cons` = 4 + 8 = 12 bytes (plus alignment padding)
- Size of `List` = max(12, 0) = 12 bytes (known at compile time)

The recursion is broken because `Box<List>` has a fixed size regardless of how deeply the list is nested. Each `Box::new(...)` allocates the inner `List` value on the heap and stores a pointer to it.

When should you use `Box<T>`?

1. **Recursive types** -- as shown here, Box breaks the infinite size problem.
2. **Large values you want to move cheaply** -- moving a `Box<[u8; 1_000_000]>` copies 8 bytes (the pointer), not 1 MB.
3. **Trait objects** -- `Box<dyn Trait>` stores a dynamically-dispatched value on the heap.
4. **Ownership transfer to the heap** -- when you need a value to outlive the current stack frame.

`Box` is the simplest heap-allocation smart pointer. It has:
- No reference counting overhead (unlike `Rc` or `Arc`)
- No runtime borrow checking (unlike `RefCell`)
- Exclusive ownership: exactly one owner, automatically freed when the `Box` goes out of scope

The `Deref` trait is implemented for `Box<T>`, so `&Box<T>` automatically coerces to `&T`. You can call methods on the inner type directly through the `Box` without manual dereferencing.

## Compiler Error Interpretation

```
error[E0072]: recursive type `List` has infinite size
 --> src/main.rs:2:1
  |
2 | enum List {
  | ^^^^^^^^^
3 |     Cons(i32, List),
  |               ---- recursive without indirection
  |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
  |
3 |     Cons(i32, Box<List>),
  |               ++++    +
```

This is one of Rust's most helpful error messages. The compiler identifies the exact field that causes the infinite size, explains why (recursive without indirection), and even suggests the fix: wrap it in `Box`. The term "indirection" means placing the value behind a pointer instead of embedding it inline. The compiler suggests `Box`, `Rc`, or `&` as options for indirection, each with different ownership semantics. For owned recursive data structures, `Box` is the standard choice.
