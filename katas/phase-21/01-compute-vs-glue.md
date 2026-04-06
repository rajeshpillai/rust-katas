---
id: compute-vs-glue
phase: 21
phase_title: "WASM in the Browser"
sequence: 1
title: Compute Kernel vs Glue Code — Separation of Concerns
hints:
  - WASM should handle compute-intensive hot paths, not I/O
  - Mixing I/O (like printing) into a computation loop destroys performance
  - Separate the pure computation from the reporting/display step
  - A pure function with no side effects is easier to optimize and test
---

## Description

WASM excels at compute-intensive tasks: image processing, physics simulation, data parsing. It is terrible at I/O and DOM manipulation because every such call crosses the WASM-to-host boundary. The correct architecture is: WASM computes, the host renders. Mixing I/O into WASM hot loops is the most common browser WASM performance mistake. This kata demonstrates the difference between a pure computation kernel and one polluted with I/O calls.

## Broken Code

```rust
/// Process data with I/O mixed into the hot loop.
/// In WASM, each println! would be a boundary-crossing call to the host.
fn process_with_io(data: &[i64]) -> i64 {
    let mut sum: i64 = 0;
    for (i, &val) in data.iter().enumerate() {
        sum += val * val;
        // BUG: I/O inside the hot loop. In WASM, this crosses the
        // boundary on every iteration, destroying performance.
        println!("Step {}: running sum = {}", i, sum);
    }
    sum
}

fn main() {
    // Generate test data
    let data: Vec<i64> = (1..=10000).collect();

    let start = std::time::Instant::now();
    let result = process_with_io(&data);
    let elapsed = start.elapsed();

    println!("Result: {}", result);
    println!("Time: {:?}", elapsed);

    // This should run in well under 50ms for 10000 elements,
    // but the println! calls make it far slower.
    assert!(
        elapsed.as_millis() < 50,
        "Too slow! Took {}ms -- I/O in the hot loop is killing performance",
        elapsed.as_millis()
    );
}
```

## Correct Code

```rust
/// Pure computation kernel -- no I/O, no side effects.
/// In WASM, this would run entirely within the module's linear memory.
fn compute_sum_of_squares(data: &[i64]) -> i64 {
    let mut sum: i64 = 0;
    for &val in data {
        sum += val * val;
    }
    sum
}

/// Reporting happens AFTER computation, outside the hot path.
/// In WASM, the host (JavaScript) would handle all display.
fn report_result(result: i64, elapsed: std::time::Duration) {
    println!("Result: {}", result);
    println!("Time: {:?}", elapsed);
}

fn main() {
    let data: Vec<i64> = (1..=10000).collect();

    // Compute -- pure, no I/O
    let start = std::time::Instant::now();
    let result = compute_sum_of_squares(&data);
    let elapsed = start.elapsed();

    // Report -- after computation is done
    report_result(result, elapsed);

    assert!(
        elapsed.as_millis() < 50,
        "Took {}ms -- should be well under 50ms for pure computation",
        elapsed.as_millis()
    );

    println!("Pure computation completed efficiently!");
}
```

## Explanation

The broken version calls `println!` inside the hot loop, once for every element. For 10000 elements, that is 10000 I/O operations. Each `println!` in the context of WASM would be a host function call that crosses the WASM-host boundary, formats a string, allocates memory, and writes to stdout. This completely dominates the computation time.

**The WASM performance model:**

| Operation | Approximate Cost |
|-----------|-----------------|
| i32.add (WASM instruction) | ~1 nanosecond |
| Host function call (JS ↔ WASM) | ~100-1000 nanoseconds |
| DOM manipulation (via host) | ~1000-10000 nanoseconds |

A single boundary crossing costs 100-1000x more than an arithmetic operation. Calling the host 10000 times from a tight loop turns a microsecond computation into a millisecond one.

**The correct architecture:**

1. **Host (JavaScript):** Handles events, DOM updates, user interaction. Writes input data to WASM linear memory.
2. **WASM module:** Reads input from linear memory, computes, writes output to linear memory. No I/O, no DOM calls.
3. **Host (JavaScript):** Reads output from linear memory, updates the UI.

The WASM module is a pure function: input bytes in, output bytes out. All side effects happen on the host side.

**Why this matters beyond performance:**

A pure computation kernel is also easier to test (no mocking needed), easier to reason about (no hidden state changes), and portable (runs on any host, not just browsers).

The invariant violated in the broken code: **WASM modules should be pure computation kernels; I/O and display should happen on the host side, outside the hot path.**

## ⚠️ Caution

- Moving computation to WASM only helps if the computation is CPU-bound. I/O-bound work (network, disk) gains nothing from WASM — keep it in the host.
- Every WASM call has overhead (context switch, validation). Micro-operations in WASM can be slower than equivalent JavaScript.

## 💡 Tips

- Profile before moving code to WASM. Only move hot compute paths where the overhead is amortized by the computation.
- Keep WASM modules focused: pure computation in, results out. No I/O, no DOM, no system calls.
- Batch data processing in WASM to minimize boundary crossings.

## Compiler Error Interpretation

```
thread 'main' panicked at 'assertion failed: elapsed.as_millis() < 50
Too slow! Took 312ms -- I/O in the hot loop is killing performance',
  src/main.rs:19:5
```

This is a runtime panic from the performance assertion:

1. **"Took 312ms"** -- the computation that should complete in under 1ms took over 300ms because of the I/O overhead.
2. **"I/O in the hot loop is killing performance"** -- the 10000 `println!` calls dominated the execution time.

The exact time varies by system, but the pattern is consistent: pure computation on 10000 integers takes microseconds, while printing 10000 lines takes hundreds of milliseconds. This 1000x slowdown is representative of what happens in browser WASM when compute kernels make host calls inside tight loops.

---

| [Prev: Stable Interface Versioning — Evolving Without Breaking](#/katas/stable-abi-versioning) | [Next: Batch Calls vs Chatty Calls — Minimizing Boundary Crossings](#/katas/batch-vs-chatty-calls) |
