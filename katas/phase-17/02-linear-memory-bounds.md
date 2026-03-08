---
id: linear-memory-bounds
phase: 17
phase_title: "What is WebAssembly Really?"
sequence: 2
title: Linear Memory — Bounds-Checked Byte Arrays
hints:
  - WASM linear memory is a contiguous, resizable byte array
  - Every memory access must be bounds-checked by the runtime
  - Out-of-bounds access in WASM causes a trap (immediate halt), not undefined behavior
  - Model memory with Vec<u8> and return Result instead of panicking
---

## Description

WASM linear memory is a contiguous array of bytes. It starts at a fixed initial size (measured in 64KB pages) and can grow at runtime. Every read and write is bounds-checked by the runtime -- if code tries to access memory beyond the current size, the WASM instance traps (halts immediately). This is fundamentally different from C, where out-of-bounds access is undefined behavior. In WASM, bounds checking is a safety guarantee. This kata simulates linear memory in Rust and demonstrates the difference between unchecked and checked access.

## Broken Code

```rust
struct LinearMemory {
    data: Vec<u8>,
}

impl LinearMemory {
    fn new(pages: usize) -> Self {
        // Each WASM page is 64KB (65536 bytes)
        let size = pages * 65536;
        LinearMemory {
            data: vec![0u8; size],
        }
    }

    fn write_byte(&mut self, offset: usize, value: u8) {
        // BUG: No bounds checking. If offset >= data.len(), this panics.
        self.data[offset] = value;
    }

    fn read_byte(&self, offset: usize) -> u8 {
        // BUG: No bounds checking here either.
        self.data[offset]
    }
}

fn main() {
    // Create 1 page of linear memory (65536 bytes)
    let mut mem = LinearMemory::new(1);

    // Write within bounds -- works fine
    mem.write_byte(0, 42);
    mem.write_byte(100, 99);
    println!("mem[0] = {}", mem.read_byte(0));
    println!("mem[100] = {}", mem.read_byte(100));

    // Write out of bounds -- panics!
    mem.write_byte(70000, 1);
}
```

## Correct Code

```rust
#[derive(Debug)]
enum MemoryError {
    OutOfBounds { offset: usize, size: usize },
    GrowExceedsMax { requested: usize, max: usize },
}

struct LinearMemory {
    data: Vec<u8>,
    max_pages: usize,
}

impl LinearMemory {
    fn new(initial_pages: usize, max_pages: usize) -> Self {
        let size = initial_pages * 65536;
        LinearMemory {
            data: vec![0u8; size],
            max_pages,
        }
    }

    fn size(&self) -> usize {
        self.data.len()
    }

    fn write_byte(&mut self, offset: usize, value: u8) -> Result<(), MemoryError> {
        // Correct: bounds check before access
        if offset >= self.data.len() {
            return Err(MemoryError::OutOfBounds {
                offset,
                size: self.data.len(),
            });
        }
        self.data[offset] = value;
        Ok(())
    }

    fn read_byte(&self, offset: usize) -> Result<u8, MemoryError> {
        if offset >= self.data.len() {
            return Err(MemoryError::OutOfBounds {
                offset,
                size: self.data.len(),
            });
        }
        Ok(self.data[offset])
    }

    fn grow(&mut self, additional_pages: usize) -> Result<usize, MemoryError> {
        let current_pages = self.data.len() / 65536;
        let new_pages = current_pages + additional_pages;
        if new_pages > self.max_pages {
            return Err(MemoryError::GrowExceedsMax {
                requested: new_pages,
                max: self.max_pages,
            });
        }
        let old_size = self.data.len();
        self.data.resize(new_pages * 65536, 0);
        Ok(old_size / 65536)
    }
}

fn main() {
    let mut mem = LinearMemory::new(1, 4);

    // Write within bounds
    mem.write_byte(0, 42).unwrap();
    mem.write_byte(100, 99).unwrap();
    println!("mem[0] = {}", mem.read_byte(0).unwrap());
    println!("mem[100] = {}", mem.read_byte(100).unwrap());

    // Attempt out-of-bounds write -- returns Err, does not panic
    match mem.write_byte(70000, 1) {
        Ok(()) => println!("Write succeeded"),
        Err(e) => println!("Trap: {:?}", e),
    }

    // Grow memory and try again
    let old_pages = mem.grow(1).unwrap();
    println!("Grew memory from {} to {} pages", old_pages, old_pages + 1);
    mem.write_byte(70000, 1).unwrap();
    println!("mem[70000] = {}", mem.read_byte(70000).unwrap());
}
```

## Explanation

The broken version uses direct indexing (`self.data[offset]`) without checking whether `offset` is within the memory bounds. When the program writes to offset 70000 in a 65536-byte memory, the `Vec` panics with an index-out-of-bounds error.

**How WASM linear memory works:**

WASM linear memory has three key properties:

1. **Contiguous bytes.** It is a single flat array, not a tree or hash table. This makes access fast (a single base + offset calculation) but means all addresses must be within the array.

2. **Bounds-checked.** Every `load` and `store` instruction checks that `offset + access_size <= memory_size`. If the check fails, the WASM instance *traps* -- execution halts immediately. This is not undefined behavior (as it would be in C). It is a deterministic, safe failure.

3. **Growable in pages.** The `memory.grow` instruction extends the memory by a given number of 64KB pages. It returns the previous size (in pages) on success or -1 on failure (if the maximum would be exceeded). Growth never shrinks existing data.

**Why this matters for safety:**

WASM's bounds checking is what makes it safe to run untrusted code. A WASM module can never read or write memory outside its own linear memory. This is enforced by the runtime on every access, with no exceptions. Combined with the lack of ambient capabilities (no file access, no network access), this creates a true sandbox.

The invariant violated in the broken code: **every memory access must be bounds-checked; out-of-bounds access must trap (fail safely), never cause undefined behavior.**

## ⚠️ Caution

- In real WASM, out-of-bounds access traps (deterministic abort), unlike C where it is undefined behavior. Your Rust simulation should mirror this with explicit bounds checks, not panics.
- WASM memory grows in 64KB pages. You cannot shrink memory once grown — only grow it.

## 💡 Tips

- Always bounds-check before accessing linear memory: `if offset + size > memory.len() { trap(); }`.
- Use `memory.grow(pages)` to request more memory — it returns the previous size or -1 on failure.

## Compiler Error Interpretation

```
thread 'main' panicked at 'index out of bounds: the len is 65536
  but the index is 70000', src/main.rs:17:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

This is a runtime panic from Rust's built-in bounds checking on `Vec` indexing:

1. **"index out of bounds"** -- the index operator `[]` on `Vec` checks bounds and panics if the index is invalid. This is Rust's safety guarantee for safe code.
2. **"the len is 65536 but the index is 70000"** -- the memory is 65536 bytes (1 page), but the code tried to access byte 70000, which is 4464 bytes past the end.

In a real WASM runtime, this would be a *trap*, not a panic. The WASM spec defines that out-of-bounds memory access traps the instance. Our `Result`-based approach models this behavior: the caller receives an error and can decide how to handle it (print a message, terminate the module, grow memory and retry).
