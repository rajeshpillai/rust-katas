---
id: clone-as-explicit-cost
phase: 1
phase_title: Ownership
sequence: 4
title: Clone as Explicit Cost
hints:
  - Clone creates a deep copy of heap-allocated data
  - Cloning is always explicit — Rust never copies heap data implicitly
  - Clone solves ownership problems but at a runtime cost
---

## Description

When you need two independent copies of heap-allocated data, Rust requires you to call `.clone()` explicitly. Unlike languages that silently copy data behind the scenes, Rust makes every allocation visible. This is a design choice: cloning has a real runtime cost, and Rust wants you to acknowledge it.

## Broken Code

```rust
fn analyze(data: String) -> usize {
    data.len()
}

fn archive(data: String) {
    println!("Archived: {}", data);
}

fn main() {
    let log_entry = String::from("2024-01-15: System started successfully");

    let length = analyze(log_entry);
    archive(log_entry);

    println!("Log entry was {} bytes", length);
}
```

## Correct Code

```rust
fn analyze(data: String) -> usize {
    data.len()
}

fn archive(data: String) {
    println!("Archived: {}", data);
}

fn main() {
    let log_entry = String::from("2024-01-15: System started successfully");

    let length = analyze(log_entry.clone());
    archive(log_entry);

    println!("Log entry was {} bytes", length);
}
```

## Explanation

In the broken code, `log_entry` is moved into `analyze` on the first call. When `archive(log_entry)` runs next, `log_entry` is already gone — its ownership was consumed by `analyze`.

The fix is to clone `log_entry` before passing it to `analyze`. The call `analyze(log_entry.clone())` creates an entirely new `String` with its own heap allocation, its own pointer, its own copy of the bytes. The original `log_entry` remains untouched and can be passed to `archive`.

This works, but you should understand the cost:
- `.clone()` on a `String` allocates new heap memory
- It copies every byte from the original to the new allocation
- For a small string, the cost is negligible
- For a large `Vec<String>` with thousands of entries, cloning could be expensive

Rust makes this explicit because implicit deep copies are a common source of performance problems in other languages. In C++, passing a `std::string` by value silently invokes the copy constructor. In Rust, that same operation is a compile error — you must choose between moving (free, but invalidates the original) and cloning (costs memory and time, but preserves the original).

In Phase 2, you will learn about borrowing, which often eliminates the need to clone entirely. Borrowing lets functions read data without taking ownership. But for now, the lesson is clear: **clone is always explicit, and it always has a cost.**

## Compiler Error Interpretation

```
error[E0382]: use of moved value: `log_entry`
 --> main.rs:13:13
  |
10 |     let log_entry = String::from("2024-01-15: System started successfully");
   |         --------- move occurs because `log_entry` has type `String`, which does not implement the `Copy` trait
11 |
12 |     let length = analyze(log_entry);
   |                          --------- value moved here
13 |     archive(log_entry);
   |             ^^^^^^^^^ value used here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
12 |     let length = analyze(log_entry.clone());
   |                                   ++++++++
```

The compiler's suggestion is notable: "consider cloning the value **if the performance cost is acceptable**." Rust does not just tell you how to fix the error — it reminds you that cloning is a tradeoff. This phrasing is deliberate. The compiler is your collaborator: it helps you make informed decisions, not just silence errors.
