---
id: manual-alloc-dealloc
phase: 19
phase_title: "WASM Memory Model"
sequence: 3
title: Manual Allocation and Deallocation in Linear Memory
hints:
  - WASM guests must expose alloc and dealloc functions for the host to manage memory
  - A bump allocator is the simplest allocator -- it just advances a pointer
  - Without bounds checking, the allocator will write past the end of the buffer
  - Return Option or Result to handle out-of-memory gracefully
---

## Description

WASM guests typically expose `alloc(size) -> ptr` and `dealloc(ptr, size)` functions that the host calls to manage memory in the guest's linear memory. The simplest allocator is a *bump allocator*: it maintains a pointer to the next free byte and advances it by the requested size for each allocation. Bump allocators are fast but cannot free individual allocations (only reset the entire allocator). This kata builds a bump allocator and demonstrates what happens when allocations exceed the available memory without bounds checking.

## Broken Code

```rust
struct BumpAllocator {
    memory: Vec<u8>,
    offset: usize,
}

impl BumpAllocator {
    fn new(size: usize) -> Self {
        BumpAllocator {
            memory: vec![0u8; size],
            offset: 0,
        }
    }

    fn alloc(&mut self, size: usize) -> usize {
        // BUG: No bounds check! If offset + size exceeds memory,
        // the returned offset is past the end of the buffer.
        let ptr = self.offset;
        self.offset += size;
        ptr
    }

    fn write(&mut self, offset: usize, data: &[u8]) {
        // This will panic when offset is out of bounds
        for (i, &byte) in data.iter().enumerate() {
            self.memory[offset + i] = byte;
        }
    }

    fn read(&self, offset: usize, size: usize) -> &[u8] {
        &self.memory[offset..offset + size]
    }
}

fn main() {
    // Create a small allocator with only 32 bytes
    let mut alloc = BumpAllocator::new(32);

    // Allocate 16 bytes -- fine
    let ptr1 = alloc.alloc(16);
    alloc.write(ptr1, b"Hello, World!!!!");
    println!("Block 1: {:?}", std::str::from_utf8(alloc.read(ptr1, 16)).unwrap());

    // Allocate another 16 bytes -- fine, uses up all memory
    let ptr2 = alloc.alloc(16);
    alloc.write(ptr2, b"Second block!!!!");
    println!("Block 2: {:?}", std::str::from_utf8(alloc.read(ptr2, 16)).unwrap());

    // Allocate 16 more bytes -- exceeds the 32-byte memory!
    // alloc() succeeds (returns offset 32) but write() will panic.
    let ptr3 = alloc.alloc(16);
    alloc.write(ptr3, b"This will crash!");
}
```

## Correct Code

```rust
#[derive(Debug)]
enum AllocError {
    OutOfMemory { requested: usize, available: usize },
    OutOfBounds { offset: usize, size: usize, capacity: usize },
}

struct BumpAllocator {
    memory: Vec<u8>,
    offset: usize,
}

impl BumpAllocator {
    fn new(size: usize) -> Self {
        BumpAllocator {
            memory: vec![0u8; size],
            offset: 0,
        }
    }

    fn alloc(&mut self, size: usize) -> Result<usize, AllocError> {
        // Correct: check bounds before advancing the offset
        let available = self.memory.len() - self.offset;
        if size > available {
            return Err(AllocError::OutOfMemory {
                requested: size,
                available,
            });
        }
        let ptr = self.offset;
        self.offset += size;
        Ok(ptr)
    }

    fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), AllocError> {
        if offset + data.len() > self.memory.len() {
            return Err(AllocError::OutOfBounds {
                offset,
                size: data.len(),
                capacity: self.memory.len(),
            });
        }
        self.memory[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }

    fn read(&self, offset: usize, size: usize) -> Result<&[u8], AllocError> {
        if offset + size > self.memory.len() {
            return Err(AllocError::OutOfBounds {
                offset,
                size,
                capacity: self.memory.len(),
            });
        }
        Ok(&self.memory[offset..offset + size])
    }

    fn reset(&mut self) {
        self.offset = 0;
        // Optionally zero the memory
        self.memory.fill(0);
    }

    fn used(&self) -> usize {
        self.offset
    }

    fn available(&self) -> usize {
        self.memory.len() - self.offset
    }
}

fn main() {
    let mut alloc = BumpAllocator::new(32);

    let ptr1 = alloc.alloc(16).unwrap();
    alloc.write(ptr1, b"Hello, World!!!!").unwrap();
    println!("Block 1: {:?}", std::str::from_utf8(alloc.read(ptr1, 16).unwrap()).unwrap());

    let ptr2 = alloc.alloc(16).unwrap();
    alloc.write(ptr2, b"Second block!!!!").unwrap();
    println!("Block 2: {:?}", std::str::from_utf8(alloc.read(ptr2, 16).unwrap()).unwrap());

    println!("Used: {} bytes, Available: {} bytes", alloc.used(), alloc.available());

    // Attempting to allocate more than available -- returns Err
    match alloc.alloc(16) {
        Ok(ptr) => println!("Allocated at offset {}", ptr),
        Err(e) => println!("Allocation failed: {:?}", e),
    }

    // Reset the allocator to reuse memory
    alloc.reset();
    println!("After reset -- Used: {}, Available: {}", alloc.used(), alloc.available());

    let ptr3 = alloc.alloc(16).unwrap();
    alloc.write(ptr3, b"Reused memory!!!").unwrap();
    println!("Block 3: {:?}", std::str::from_utf8(alloc.read(ptr3, 16).unwrap()).unwrap());
}
```

## Explanation

The broken version's `alloc` method always succeeds -- it advances the offset without checking if the resulting range is within the memory buffer. When `alloc(16)` is called with only 0 bytes remaining, it returns offset 32 (past the end). The subsequent `write` call tries to access `self.memory[32]`, which panics with an index-out-of-bounds error.

**How WASM guests expose allocators:**

A typical Rust WASM module exports two functions:

```rust
#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 { ... }

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, size: usize) { ... }
```

The host calls `alloc` to reserve space before writing data into linear memory. When done, the host calls `dealloc` to free the space. If `alloc` fails (out of memory), it returns a null pointer or 0. The host must check for this.

**Bump allocators in WASM:**

Bump allocators are popular in WASM because of their simplicity and speed. They work well for request-response patterns: allocate everything for a request, process it, then reset the allocator for the next request. The downside is that individual allocations cannot be freed -- you can only reset the entire allocator.

**What happens when WASM runs out of memory:**

In real WASM, the guest can call `memory.grow(pages)` to extend linear memory. If growth succeeds, the allocator has more space. If growth fails (the host's maximum is reached), the allocation fails. A well-designed allocator tries to grow memory before reporting out-of-memory to the caller.

The invariant violated in the broken code: **an allocator must check that the requested size fits within available memory before advancing the allocation pointer.**

## Compiler Error Interpretation

```
thread 'main' panicked at 'index out of bounds: the len is 32
  but the index is 32', src/main.rs:23:13
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

This is a runtime panic from the `write` method:

1. **"index out of bounds"** -- the `write` method tried to access `self.memory[32]`, but the memory buffer has only 32 elements (indices 0-31).
2. **"the len is 32 but the index is 32"** -- off by one past the end. The allocator returned offset 32 without checking bounds, and the write operation immediately crashed.

The `alloc` method silently succeeded (returned 32), creating a ticking time bomb. The crash only happens when someone tries to use the returned offset. In a real system, the time between allocation and crash could be long, making the bug very hard to trace back to the faulty allocator.
