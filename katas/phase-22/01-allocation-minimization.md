---
id: allocation-minimization
phase: 22
phase_title: "Rust Ownership Patterns for WASM"
sequence: 1
title: Allocation Minimization — Reuse Buffers Across Calls
hints:
  - Every allocation in WASM linear memory is expensive (fragmentation, overhead)
  - Creating a new Vec on every function call wastes time on repeated allocation
  - Pass a mutable buffer reference and clear it between uses instead
  - One allocation, many uses -- the buffer lives outside the loop
---

## Description

In WASM, every heap allocation is expensive: the allocator must find free space in linear memory, and repeated allocations fragment the memory space. Unlike native programs that benefit from sophisticated OS-level memory management, WASM allocators work within a flat byte array. Creating a new `Vec` on every function call means repeated allocation and deallocation for identical-sized buffers. The correct pattern is to allocate once and reuse the buffer across calls, clearing it between uses.

## Broken Code

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Process data and return the results.
/// BUG: Allocates a new Vec on every call.
fn process_chunk(data: &[u8]) -> Vec<u8> {
    ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
    let mut result = Vec::with_capacity(data.len());
    for &byte in data {
        // Simple transformation: double each byte value (saturating)
        result.push(byte.saturating_mul(2));
    }
    result
}

fn main() {
    let chunks: Vec<Vec<u8>> = (0..1000)
        .map(|i| vec![(i % 256) as u8; 64])
        .collect();

    let mut all_results = Vec::new();
    for chunk in &chunks {
        let result = process_chunk(chunk);
        all_results.push(result[0]); // Keep first byte of each result
    }

    let allocs = ALLOC_COUNT.load(Ordering::SeqCst);
    println!("Processed {} chunks", chunks.len());
    println!("Total allocations: {}", allocs);
    println!("First few results: {:?}", &all_results[..5]);

    assert_eq!(
        allocs, 1,
        "Expected 1 allocation (reusable buffer), got {} (one per call)!",
        allocs
    );
}
```

## Correct Code

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Process data into a reusable buffer.
/// The buffer is cleared and reused -- no new allocation needed.
fn process_chunk_into(data: &[u8], buffer: &mut Vec<u8>) {
    buffer.clear(); // Reset length to 0, capacity remains
    for &byte in data {
        buffer.push(byte.saturating_mul(2));
    }
}

fn main() {
    let chunks: Vec<Vec<u8>> = (0..1000)
        .map(|i| vec![(i % 256) as u8; 64])
        .collect();

    // Correct: allocate the buffer once, outside the loop
    ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
    let mut buffer = Vec::with_capacity(64);

    let mut all_results = Vec::new();
    for chunk in &chunks {
        process_chunk_into(chunk, &mut buffer);
        all_results.push(buffer[0]); // Read result before next iteration clears it
    }

    let allocs = ALLOC_COUNT.load(Ordering::SeqCst);
    println!("Processed {} chunks", chunks.len());
    println!("Total allocations: {}", allocs);
    println!("First few results: {:?}", &all_results[..5]);

    assert_eq!(
        allocs, 1,
        "Expected 1 allocation (reusable buffer), got {}!",
        allocs
    );
}
```

## Explanation

The broken version creates a new `Vec::with_capacity(data.len())` on every call to `process_chunk`. For 1000 chunks, that means 1000 allocations and 1000 deallocations. Each allocation requires the WASM linear memory allocator to find free space, and each deallocation creates fragmentation.

**Why buffer reuse matters in WASM:**

Native programs benefit from the OS's virtual memory system, which can efficiently recycle recently freed memory. WASM's linear memory is a flat byte array with no virtual memory. A simple bump allocator cannot reuse freed memory at all. Even more sophisticated allocators (like `dlmalloc`, which Rust uses for WASM) have significant overhead for small, frequent allocations.

**The `clear()` trick:**

`Vec::clear()` sets the length to 0 but preserves the allocated capacity. Subsequent `push()` calls reuse the existing allocation without asking the allocator for more memory. This is O(1) -- no allocation, no deallocation, no fragmentation.

**The pattern in real WASM code:**

```rust
// Allocate once during module initialization
static mut BUFFER: Vec<u8> = Vec::new();

#[no_mangle]
pub extern "C" fn init(capacity: usize) {
    unsafe { BUFFER = Vec::with_capacity(capacity); }
}

#[no_mangle]
pub extern "C" fn process(input_ptr: *const u8, input_len: usize) -> usize {
    unsafe {
        BUFFER.clear();
        // ... process into BUFFER ...
        BUFFER.as_ptr() as usize // Return pointer to result
    }
}
```

The host calls `init` once to set up the buffer, then calls `process` repeatedly. The buffer is reused on every call.

The invariant violated in the broken code: **in WASM, allocate buffers once and reuse them across calls; repeated allocation of same-sized buffers wastes time and fragments linear memory.**

## ⚠️ Caution

- `static mut` is unsafe and not thread-safe. In single-threaded WASM it is acceptable, but document why.
- `Vec::clear()` preserves capacity but drops all elements. If elements have expensive `Drop` implementations, clear is not "free."

## 💡 Tips

- Pre-allocate buffers at module initialization and reuse them across calls with `clear()`.
- Use `Vec::with_capacity()` to avoid reallocations when the expected size is known.
- Measure allocation counts with a custom allocator in tests.

## Compiler Error Interpretation

```
thread 'main' panicked at 'assertion `left == right` failed: Expected 1 allocation (reusable buffer), got 1000 (one per call)!
  left: 1000
 right: 1', src/main.rs:26:5
```

This is a runtime panic from the allocation counter assertion:

1. **"left: 1000, right: 1"** -- the code made 1000 allocations (one per `process_chunk` call) instead of the expected single allocation.
2. **"one per call"** -- each function call allocated a new Vec, even though every Vec had the same capacity.

In a real WASM application processing video frames at 60fps, this would mean 60 allocations per second for each buffer. Over minutes of use, the linear memory would fragment, grow unnecessarily, and degrade performance.

---

| [Prev: WASM Cannot Touch the DOM — Host Callbacks Required](#/katas/no-dom-from-wasm) | [Next: Zero-Copy Data Access — Slices vs Clones](#/katas/zero-copy-slices) |
