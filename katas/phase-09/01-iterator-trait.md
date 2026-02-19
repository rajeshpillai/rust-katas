---
id: iterator-trait
phase: 9
phase_title: "Iterators & Zero-Cost Abstractions"
sequence: 1
title: The Iterator Trait
hints:
  - A for loop desugars into a call to IntoIterator and then repeated calls to next()
  - To use a custom struct in a for loop, it must implement the Iterator trait
  - The Iterator trait requires a type Item and a method fn next(&mut self) -> Option<Self::Item>
---

## Description

In Rust, the `for` loop is not magic. It is syntactic sugar over the `Iterator` trait. When you write `for x in collection`, the compiler calls `.into_iter()` on the collection and then repeatedly calls `.next()` until it returns `None`. This means any type you want to iterate over must implement the `Iterator` trait (or `IntoIterator`, which we cover in a later kata).

The `Iterator` trait requires exactly one method: `next()`, which returns `Option<Self::Item>`. Each call either yields `Some(value)` or signals exhaustion with `None`.

## Broken Code

```rust
struct Countdown {
    value: u32,
}

impl Countdown {
    fn new(start: u32) -> Self {
        Countdown { value: start }
    }
}

fn main() {
    let countdown = Countdown::new(5);

    for number in countdown {
        println!("{}!", number);
    }

    println!("Liftoff!");
}
```

## Correct Code

```rust
struct Countdown {
    value: u32,
}

impl Countdown {
    fn new(start: u32) -> Self {
        Countdown { value: start }
    }
}

impl Iterator for Countdown {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.value > 0 {
            let current = self.value;
            self.value -= 1;
            Some(current)
        } else {
            None
        }
    }
}

fn main() {
    let countdown = Countdown::new(5);

    for number in countdown {
        println!("{}!", number);
    }

    println!("Liftoff!");
}
```

## Explanation

The broken version defines a `Countdown` struct but never implements the `Iterator` trait for it. When Rust encounters `for number in countdown`, it looks for an implementation of `IntoIterator` on `Countdown`. The `Iterator` trait automatically provides an `IntoIterator` implementation (every `Iterator` is `IntoIterator`), but since `Countdown` implements neither, the compiler rejects the code.

The correct version adds `impl Iterator for Countdown` with:

1. **`type Item = u32`** -- This associated type tells the compiler what kind of values this iterator produces. Every call to `next()` will return `Option<u32>`.

2. **`fn next(&mut self) -> Option<Self::Item>`** -- This is the heart of iteration. Each call decrements the internal counter and returns `Some(current_value)`. When the counter reaches zero, it returns `None`, signaling that iteration is complete.

Notice that `next()` takes `&mut self`. This is essential: the iterator must track its own progress by mutating internal state. The `for` loop calls `next()` repeatedly, and each call must advance the iterator to the next position.

Once you implement `Iterator`, you get dozens of methods for free: `map`, `filter`, `fold`, `take`, `skip`, `zip`, `enumerate`, and many more. These are all default methods on the `Iterator` trait that are built on top of your single `next()` implementation. This is the power of Rust's trait system -- one method unlocks an entire algebra of transformations.

## Compiler Error Interpretation

```
error[E0277]: `Countdown` is not an iterator
 --> src/main.rs:13:20
   |
13 |     for number in countdown {
   |                    ^^^^^^^^^ `Countdown` is not an iterator
   |
   = help: the trait `Iterator` is not implemented for `Countdown`
   = note: required for `Countdown` to implement `IntoIterator`
```

The compiler tells you exactly what is missing. The `for` loop requires `IntoIterator`, and the note explains that `Iterator` must be implemented for `Countdown` to satisfy that requirement. The help message points directly at the missing trait. When you see "is not an iterator," your first step should be to implement `Iterator` with its required `type Item` and `fn next()`.
