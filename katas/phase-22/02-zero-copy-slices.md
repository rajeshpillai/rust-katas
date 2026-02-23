---
id: zero-copy-slices
phase: 22
phase_title: "Rust Ownership Patterns for WASM"
sequence: 2
title: Zero-Copy Data Access — Slices vs Clones
hints:
  - When the guest only reads data, it should reference it directly, not clone it
  - ".to_vec()" and ".clone()" copy all the data -- doubling memory usage
  - A slice "&[u8]" points to existing memory without copying
  - Track allocation size to verify no unnecessary copies are made
---

## Description

Copying data across the WASM boundary is expensive: it doubles memory usage and takes time proportional to the data size. When the host writes data into the guest's linear memory, the guest can read directly from that memory location using a slice -- no copy needed. Unnecessary cloning (`to_vec()`, `clone()`) is the second most common WASM performance mistake after chatty boundary crossings. This kata demonstrates the difference between cloning data and working with it in place.

## Broken Code

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static BYTES_COPIED: AtomicUsize = AtomicUsize::new(0);

/// Process image data. BUG: clones the input unnecessarily.
fn process_image(pixels: &[u8]) -> u8 {
    // BUG: Cloning the entire pixel buffer just to read it.
    // This doubles memory usage for no reason.
    let owned_copy = pixels.to_vec();
    BYTES_COPIED.fetch_add(owned_copy.len(), Ordering::SeqCst);

    // Compute average brightness (all we do is read)
    let sum: u64 = owned_copy.iter().map(|&b| b as u64).sum();
    (sum / owned_copy.len() as u64) as u8
}

fn main() {
    // Simulate a 1MB image in linear memory
    let image_data: Vec<u8> = (0..1_000_000)
        .map(|i| (i % 256) as u8)
        .collect();

    let brightness = process_image(&image_data);
    println!("Average brightness: {}", brightness);

    let copied = BYTES_COPIED.load(Ordering::SeqCst);
    println!("Bytes copied: {}", copied);

    assert_eq!(
        copied, 0,
        "Unnecessary copy detected: {} bytes copied when 0 were needed!",
        copied
    );
}
```

## Correct Code

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static BYTES_COPIED: AtomicUsize = AtomicUsize::new(0);

/// Process image data directly from the slice -- zero copy.
fn process_image(pixels: &[u8]) -> u8 {
    // Correct: work directly on the borrowed slice.
    // No allocation, no copy. The data stays in place.
    let sum: u64 = pixels.iter().map(|&b| b as u64).sum();
    (sum / pixels.len() as u64) as u8
}

/// When transformation is needed, write results to a separate output buffer
/// rather than cloning the input.
fn transform_image(pixels: &[u8], output: &mut Vec<u8>) {
    output.clear();
    output.reserve(pixels.len());
    // Invert each pixel: 255 - value
    for &byte in pixels {
        output.push(255 - byte);
    }
}

fn main() {
    let image_data: Vec<u8> = (0..1_000_000)
        .map(|i| (i % 256) as u8)
        .collect();

    // Read-only operation: zero copy
    let brightness = process_image(&image_data);
    println!("Average brightness: {}", brightness);

    let copied = BYTES_COPIED.load(Ordering::SeqCst);
    println!("Bytes copied: {}", copied);
    assert_eq!(copied, 0, "Should be zero-copy for read-only operations!");

    // Write operation: uses a separate output buffer, does not clone input
    let mut output = Vec::with_capacity(image_data.len());
    transform_image(&image_data, &mut output);
    println!("Transformed {} pixels", output.len());
    println!("First 5 output pixels: {:?}", &output[..5]);
}
```

## Explanation

The broken version calls `pixels.to_vec()` to clone the entire 1MB pixel buffer before processing it. The function only reads the data (computing an average), so the clone is completely unnecessary. It doubles the memory footprint and takes time proportional to the data size.

**Why zero-copy matters in WASM:**

WASM linear memory is a precious resource. It starts small (typically a few pages) and grows in 64KB increments. Each unnecessary copy wastes linear memory and may trigger `memory.grow`, which is expensive. For data-intensive operations (image processing, audio, video), the input data is often megabytes. Cloning it doubles the memory requirement.

**The zero-copy pattern in WASM:**

1. **Host writes data** to linear memory at a known offset
2. **Host calls guest** with `(offset, length)` parameters
3. **Guest creates a slice** pointing to that offset: `&memory[offset..offset+length]`
4. **Guest reads directly** from the slice -- no copy, no allocation
5. **Guest writes results** to a separate output region

The guest never needs to "own" the input data. A borrowed slice (`&[u8]`) is sufficient for reading.

**When you DO need to copy:**

Sometimes the guest needs to modify the data (transformation, filtering). In that case, the correct approach is to write results to a separate output buffer -- not to clone the input and modify the clone. The input stays untouched, and the output goes to a pre-allocated buffer (see Phase 22 Kata 1 on buffer reuse).

**Ownership model at WASM boundaries:**

This is where Rust's ownership model shines for WASM. Rust's distinction between `&T` (borrowed, read-only) and `T` (owned, can modify) maps directly to the WASM zero-copy pattern:
- `&[u8]` = read directly from linear memory (zero copy)
- `Vec<u8>` = own a copy in a new allocation (expensive)

The invariant violated in the broken code: **read-only operations should use borrowed slices, not owned copies; cloning data that is only read wastes memory and time.**

## Compiler Error Interpretation

```
thread 'main' panicked at 'assertion `left == right` failed: Unnecessary copy detected: 1000000 bytes copied when 0 were needed!
  left: 1000000
 right: 0', src/main.rs:22:5
```

This is a runtime panic from the copy-tracking assertion:

1. **"left: 1000000, right: 0"** -- the function copied 1,000,000 bytes (the entire image) when zero bytes needed to be copied.
2. **"Unnecessary copy detected"** -- the function only reads the data to compute an average. There is no reason to own a copy.

In a real WASM application processing images at 30fps, this would mean copying 30MB of data per second (for 1MP images) with zero benefit. The zero-copy version processes the same data with zero additional memory allocation.
