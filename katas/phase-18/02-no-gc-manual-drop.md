---
id: no-gc-manual-drop
phase: 18
phase_title: "Rust to WASM Toolchain"
sequence: 2
title: No Garbage Collector — Ownership IS the Memory Model
hints:
  - WASM has no garbage collector for Rust-compiled modules
  - Box::into_raw converts ownership to a raw pointer -- someone must call Box::from_raw to free it
  - Every allocation must have exactly one deallocation
  - Track allocations with a counter to prove they balance
---

## Description

WebAssembly has no garbage collector (for Rust-compiled modules). Every heap allocation in linear memory must be explicitly freed. Rust's ownership model handles this automatically -- when a `Box`, `Vec`, or `String` goes out of scope, `Drop` runs and the memory is freed. But when you convert owned data to a raw pointer using `Box::into_raw()` (to hand it across the WASM boundary), Rust no longer tracks that allocation. If you forget to call `Box::from_raw()` to reclaim it, the memory is leaked permanently. This is the most common WASM memory bug.

## Broken Code

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Simulate handing data across the WASM boundary.
/// Returns a raw pointer (simulating a WASM linear memory offset).
fn guest_create_string(s: &str) -> *mut String {
    let boxed = Box::new(String::from(s));
    ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
    // Convert to raw pointer -- ownership is now "across the boundary"
    Box::into_raw(boxed)
}

/// Simulate the host reading the string through the pointer.
fn host_read_string(ptr: *mut String) -> String {
    // Safety: ptr was created by guest_create_string
    let s = unsafe { &*ptr };
    s.clone()
}

fn main() {
    let mut pointers = Vec::new();

    // Guest creates several strings and hands them to the host
    for i in 0..5 {
        let ptr = guest_create_string(&format!("message-{}", i));
        let value = host_read_string(ptr);
        println!("Host received: {}", value);
        pointers.push(ptr);
    }

    // BUG: The host never frees the strings!
    // The raw pointers are just numbers now -- no Drop runs.
    // Memory is leaked permanently.

    let allocs = ALLOC_COUNT.load(Ordering::SeqCst);
    let deallocs = DEALLOC_COUNT.load(Ordering::SeqCst);
    println!("Allocations: {}, Deallocations: {}", allocs, deallocs);
    assert_eq!(allocs, deallocs, "Memory leak detected!");
}
```

## Correct Code

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

fn guest_create_string(s: &str) -> *mut String {
    let boxed = Box::new(String::from(s));
    ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
    Box::into_raw(boxed)
}

fn host_read_string(ptr: *mut String) -> String {
    let s = unsafe { &*ptr };
    s.clone()
}

/// Correct: reclaim the allocation by converting the raw pointer back to a Box.
/// When the Box goes out of scope, Drop runs and the memory is freed.
fn guest_free_string(ptr: *mut String) {
    // Safety: ptr was created by guest_create_string using Box::into_raw.
    // This is the only call to from_raw for this pointer (no double-free).
    let _boxed = unsafe { Box::from_raw(ptr) };
    DEALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
    // _boxed is dropped here, freeing the String and its heap buffer
}

fn main() {
    let mut pointers = Vec::new();

    for i in 0..5 {
        let ptr = guest_create_string(&format!("message-{}", i));
        let value = host_read_string(ptr);
        println!("Host received: {}", value);
        pointers.push(ptr);
    }

    // Correct: free every allocation through the guest's deallocator
    for ptr in pointers {
        guest_free_string(ptr);
    }

    let allocs = ALLOC_COUNT.load(Ordering::SeqCst);
    let deallocs = DEALLOC_COUNT.load(Ordering::SeqCst);
    println!("Allocations: {}, Deallocations: {}", allocs, deallocs);
    assert_eq!(allocs, deallocs, "Memory leak detected!");
}
```

## Explanation

The broken version calls `Box::into_raw()` to convert each `Box<String>` to a raw pointer (simulating handing the allocation across the WASM boundary), but never calls `Box::from_raw()` to reclaim the memory. The raw pointers are just numbers -- Rust does not run `Drop` for them when they go out of scope. The assertion fails because 5 allocations were made but 0 deallocations occurred.

**Why WASM has no GC (for Rust):**

Rust does not need a garbage collector because ownership provides deterministic deallocation. When you compile Rust to WASM, this property is preserved: memory is freed when the owning value goes out of scope, just as in native Rust. However, when data crosses the WASM boundary (passed to the host via a pointer into linear memory), ownership is effectively transferred outside Rust's tracking. The host must explicitly call back into the guest's `dealloc` function to free the memory.

**The `into_raw` / `from_raw` contract:**

- `Box::into_raw(b)` -- consumes the `Box`, returns a raw pointer, does NOT free the memory. Rust forgets about the allocation.
- `Box::from_raw(ptr)` -- creates a new `Box` from a raw pointer. When this `Box` is dropped, the memory is freed.
- **Rule:** Every `into_raw` must have exactly one matching `from_raw`. Zero means a leak. Two means a double-free (undefined behavior).

**How `wasm-bindgen` handles this:**

The `wasm-bindgen` tool generates glue code that tracks allocations and automatically pairs `alloc` with `dealloc`. When you pass a `String` from Rust to JavaScript, `wasm-bindgen` allocates in linear memory, lets JavaScript read the bytes, and then frees the allocation. Understanding the manual process helps you debug when the automatic process fails.

The invariant violated in the broken code: **every `Box::into_raw` must be paired with exactly one `Box::from_raw` to prevent memory leaks.**

## Compiler Error Interpretation

```
thread 'main' panicked at 'assertion `left == right` failed: Memory leak detected!
  left: 5
 right: 0', src/main.rs:35:5
```

This is a runtime panic from the leak detection assertion:

1. **"left: 5, right: 0"** -- 5 allocations were made (via `guest_create_string`), but 0 deallocations occurred. Every `Box::into_raw` created a raw pointer that was never reclaimed.
2. **"Memory leak detected!"** -- the assertion message makes the bug explicit.

In a real WASM module, this leak would accumulate over time. Linear memory can grow (via `memory.grow`) but never shrinks. Leaked allocations permanently reduce available memory. In long-running WASM modules (servers, plugins), this eventually exhausts linear memory and causes allocation failures.
