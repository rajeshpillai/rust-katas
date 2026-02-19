---
id: rc-shared-ownership
phase: 11
phase_title: "Memory & Performance Intuition"
sequence: 2
title: "Rc and Shared Ownership"
hints:
  - Rust's ownership model normally allows exactly one owner per value
  - When two data structures need to share the same data, a move gives it to one and leaves the other empty
  - Rc<T> (Reference Counted) allows multiple owners by tracking how many references exist
  - Rc::clone() is cheap -- it only increments a counter, it does not deep-copy the data
---

## Description

Rust's default ownership model is strict: every value has exactly one owner. When you assign a value to a new variable or pass it to a function, ownership **moves** and the original variable becomes invalid. But some data structures require shared ownership -- a directed acyclic graph (DAG), for example, where multiple nodes may point to the same child.

`Rc<T>` (Reference Counted) solves this for single-threaded scenarios. It wraps a value in a reference-counted heap allocation. Each call to `Rc::clone()` increments the count; each drop decrements it. When the count reaches zero, the value is freed.

## Broken Code

```rust
enum List {
    Cons(i32, Box<List>),
    Nil,
}

use List::{Cons, Nil};

fn main() {
    // Shared tail: both list_b and list_c should share list_a as a suffix
    //
    //   list_b = 4 -> 1 -> 2 -> 3 -> Nil
    //   list_c = 5 -> 1 -> 2 -> 3 -> Nil
    //                 ^--- shared ---^

    let list_a = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));

    // First use: list_b takes ownership of list_a
    let list_b = Cons(4, Box::new(list_a));

    // Second use: list_a has been moved!
    let list_c = Cons(5, Box::new(list_a));

    println!("list_b starts with 4");
    println!("list_c starts with 5");
}
```

## Correct Code

```rust
use std::rc::Rc;

enum List {
    Cons(i32, Rc<List>),
    Nil,
}

use List::{Cons, Nil};

fn main() {
    // Shared tail: both list_b and list_c share list_a as a suffix
    //
    //   list_b = 4 -> \
    //                  1 -> 2 -> 3 -> Nil  (shared, reference counted)
    //   list_c = 5 -> /

    let list_a = Rc::new(Cons(1, Rc::new(Cons(2, Rc::new(Cons(3, Rc::new(Nil)))))));

    // Rc::clone increments the reference count -- it does NOT deep-copy the list
    let list_b = Cons(4, Rc::clone(&list_a));
    let list_c = Cons(5, Rc::clone(&list_a));

    println!("list_b starts with 4");
    println!("list_c starts with 5");
    println!("Shared tail reference count: {}", Rc::strong_count(&list_a));
}
```

## Explanation

The broken version uses `Box<List>`, which provides exclusive ownership. When `list_a` is placed inside `Box::new(list_a)` for `list_b`, ownership of `list_a` moves into that `Box`. When the code tries to use `list_a` again for `list_c`, the value has already been moved, and the compiler rejects it.

The correct version replaces `Box<List>` with `Rc<List>`. Now `list_a` is an `Rc<List>`, and creating `list_b` and `list_c` uses `Rc::clone(&list_a)` instead of moving `list_a`. Each `Rc::clone` is cheap: it increments an internal reference counter (from 1 to 2, then 2 to 3) and returns a new `Rc` pointer to the same heap allocation. No data is copied.

Key facts about `Rc<T>`:

1. **`Rc::clone()` is NOT a deep clone.** Despite the name `clone`, it only increments a counter and copies a pointer. The Rust community deliberately chose this name to make it explicit that you are creating a new reference. Use `Rc::clone(&x)` rather than `x.clone()` to signal that this is a reference count increment, not a data copy.

2. **`Rc<T>` provides immutable access only.** You can call methods on the inner `T` through the `Rc`, but you cannot get a `&mut T`. Shared ownership and mutation are incompatible (this is the aliasing + mutation rule). If you need shared ownership AND mutation, combine `Rc` with `RefCell` (covered in the next kata).

3. **`Rc<T>` is single-threaded only.** It does not use atomic operations for the reference counter, so it is not safe to share across threads. For multi-threaded shared ownership, use `Arc<T>` (Atomic Reference Counted), which has the same API but uses atomic counter operations.

4. **Reference cycles cause memory leaks.** If `Rc<A>` holds an `Rc<B>` which holds an `Rc<A>`, the count never reaches zero and the memory is never freed. Rust does not prevent this at compile time. Use `Weak<T>` references to break cycles.

The function `Rc::strong_count()` returns the current reference count. In the correct version, after creating both `list_b` and `list_c`, the count for `list_a` is 3: one for `list_a` itself, one for `list_b`'s reference, and one for `list_c`'s reference.

## Compiler Error Interpretation

```
error[E0382]: use of moved value: `list_a`
  --> src/main.rs:20:38
   |
14 |     let list_a = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
   |         ------ move occurs because `list_a` has type `List`, which does not implement the `Copy` trait
...
17 |     let list_b = Cons(4, Box::new(list_a));
   |                                   ------ value moved here
...
20 |     let list_c = Cons(5, Box::new(list_a));
   |                                   ^^^^^^ value used here after move
```

The compiler pinpoints the move and the invalid reuse. It also explains *why* the move happens: `List` does not implement `Copy`, so passing it to `Box::new()` transfers ownership. The solution is not to implement `Copy` (recursive types with heap data cannot be `Copy`), but to use `Rc` for shared ownership. When you see "use of moved value" and your intent is to share data between multiple owners, `Rc` (or `Arc` for concurrent code) is the answer.
