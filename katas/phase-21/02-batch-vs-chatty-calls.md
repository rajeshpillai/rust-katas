---
id: batch-vs-chatty-calls
phase: 21
phase_title: "WASM in the Browser"
sequence: 2
title: Batch Calls vs Chatty Calls — Minimizing Boundary Crossings
hints:
  - Each call across the WASM boundary has overhead (argument marshaling, context switching)
  - Processing items one at a time means N boundary crossings for N items
  - Batching sends all items at once -- one boundary crossing regardless of N
  - Write data to a shared buffer and process the entire buffer in one call
---

## Description

Every function call across the WASM boundary has overhead: arguments must be marshaled, the runtime switches context between host and guest, and return values must be extracted. When processing many items, calling the guest function once per item (chatty) is far slower than passing all items at once (batch). This kata demonstrates the performance difference by simulating boundary-crossing overhead and shows why batch processing is the standard pattern for WASM data processing.

## Broken Code

```rust
use std::sync::atomic::{AtomicU64, Ordering};

/// Count total boundary crossings
static BOUNDARY_CROSSINGS: AtomicU64 = AtomicU64::new(0);

/// Simulate boundary-crossing overhead.
/// In real WASM, this overhead comes from argument marshaling,
/// context switching, and validation.
fn cross_boundary() {
    BOUNDARY_CROSSINGS.fetch_add(1, Ordering::SeqCst);
    // Simulate ~1 microsecond of overhead per crossing
    let mut dummy = 0u64;
    for i in 0..500 {
        dummy = dummy.wrapping_add(i);
    }
    std::hint::black_box(dummy);
}

/// Chatty pattern: process one item per boundary crossing.
fn process_item_chatty(value: i64) -> i64 {
    cross_boundary();
    value * value
}

fn main() {
    let data: Vec<i64> = (1..=2000).collect();
    let mut results = Vec::with_capacity(data.len());

    // BUG: One boundary crossing per item -- 2000 crossings total!
    let start = std::time::Instant::now();
    for &item in &data {
        results.push(process_item_chatty(item));
    }
    let elapsed = start.elapsed();

    let crossings = BOUNDARY_CROSSINGS.load(Ordering::SeqCst);
    println!("Results computed: {}", results.len());
    println!("Boundary crossings: {}", crossings);
    println!("Time: {:?}", elapsed);

    // 2000 crossings is far too many for processing 2000 items
    assert!(
        crossings <= 10,
        "Too many boundary crossings: {}! Batch your calls.",
        crossings
    );
}
```

## Correct Code

```rust
use std::sync::atomic::{AtomicU64, Ordering};

static BOUNDARY_CROSSINGS: AtomicU64 = AtomicU64::new(0);

fn cross_boundary() {
    BOUNDARY_CROSSINGS.fetch_add(1, Ordering::SeqCst);
    let mut dummy = 0u64;
    for i in 0..500 {
        dummy = dummy.wrapping_add(i);
    }
    std::hint::black_box(dummy);
}

/// Batch pattern: process ALL items in a single boundary crossing.
/// The caller writes all input to a shared buffer, calls this once,
/// and reads all output from the result buffer.
fn process_batch(data: &[i64]) -> Vec<i64> {
    cross_boundary(); // One crossing for the entire batch
    data.iter().map(|&v| v * v).collect()
}

fn main() {
    let data: Vec<i64> = (1..=2000).collect();

    // Correct: one boundary crossing for all 2000 items
    let start = std::time::Instant::now();
    let results = process_batch(&data);
    let elapsed = start.elapsed();

    let crossings = BOUNDARY_CROSSINGS.load(Ordering::SeqCst);
    println!("Results computed: {}", results.len());
    println!("Boundary crossings: {}", crossings);
    println!("Time: {:?}", elapsed);

    assert!(
        crossings <= 10,
        "Too many boundary crossings: {}!",
        crossings
    );

    // Verify correctness
    assert_eq!(results[0], 1);      // 1^2
    assert_eq!(results[1], 4);      // 2^2
    assert_eq!(results[1999], 2000 * 2000); // 2000^2

    println!("Batch processing completed efficiently!");
}
```

## Explanation

The broken version calls `process_item_chatty` once per item, resulting in 2000 boundary crossings for 2000 items. Each crossing has overhead (simulated by the busy loop). The assertion fails because the code made 2000 crossings where at most 10 were acceptable.

**The real cost of WASM boundary crossings:**

In a browser, each JavaScript-to-WASM call involves:
1. Marshaling arguments (converting JS values to WASM types)
2. Entering the WASM execution context
3. Running the WASM function
4. Exiting the WASM context
5. Marshaling return values back to JS

This overhead is small per call (~100ns-1us), but multiplied by thousands of calls it dominates the computation.

**The batch pattern:**

Instead of:
```
JS -> WASM(item1) -> JS -> WASM(item2) -> JS -> ... (N crossings)
```

Do:
```
JS: write items to linear memory
JS -> WASM(process_all) -> JS  (1 crossing)
JS: read results from linear memory
```

The single-crossing version is O(1) in overhead regardless of how many items are processed. The actual computation time is the same either way, but the overhead difference can be 100-1000x.

**When batching is not possible:**

Sometimes items must be processed sequentially (each depends on the previous result). Even then, you can batch the entire chain into a single WASM call that processes the chain internally, rather than bouncing back and forth between host and guest.

The invariant violated in the broken code: **minimize boundary crossings by batching work; send all data at once rather than processing items individually across the boundary.**

## ⚠️ Caution

- Batching only helps when operations are independent. If step N depends on the result of step N-1, you cannot batch them.
- Very large batches can cause memory pressure — the entire input must fit in WASM linear memory. Balance batch size with memory constraints.

## 💡 Tips

- Aim for one boundary crossing per logical operation, not per data element.
- Pass arrays/buffers across the boundary instead of individual values.
- Measure the boundary crossing cost in your specific setup to determine optimal batch size.

## Compiler Error Interpretation

```
thread 'main' panicked at 'assertion failed: crossings <= 10
Too many boundary crossings: 2000! Batch your calls.',
  src/main.rs:33:5
```

This is a runtime panic from the assertion:

1. **"crossings: 2000"** -- the code made 2000 individual calls across the simulated boundary, one per item.
2. **"Batch your calls"** -- the message prescribes the fix: send all data at once.

The assertion threshold of 10 is generous -- for this workload, the ideal number is 1 (a single batch call). Real WASM applications may need a few calls (one to allocate, one to process, one to read results), but never one per data item.

---

| [Prev: Compute Kernel vs Glue Code — Separation of Concerns](#/katas/compute-vs-glue) | [Next: WASM Cannot Touch the DOM — Host Callbacks Required](#/katas/no-dom-from-wasm) |
