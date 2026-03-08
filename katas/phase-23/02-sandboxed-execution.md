---
id: sandboxed-execution
phase: 23
phase_title: "WASM Outside the Browser"
sequence: 2
title: Sandboxed Execution — Resource Limits and Isolation
hints:
  - WASM runtimes enforce memory limits by controlling memory.grow
  - The sandbox must check resource budgets BEFORE operations, not after
  - If the guest goes through the sandbox's allocator, all allocations are tracked
  - Return Result from the allocator to handle out-of-memory gracefully
---

## Description

WASM runtimes (Wasmtime, Wasmer) run untrusted code in sandboxes with controlled resource limits. The guest cannot allocate more memory than the host allows, and instruction count limits prevent infinite loops. The key principle: the sandbox mediates all resource access. If the guest bypasses the sandbox, limits are not enforced. This kata demonstrates a sandbox that checks memory budgets before allocation, preventing the guest from exceeding its limits.

## Broken Code

```rust
struct Sandbox {
    memory_limit: usize,
    memory_used: usize,
}

impl Sandbox {
    fn new(memory_limit: usize) -> Self {
        Sandbox {
            memory_limit,
            memory_used: 0,
        }
    }
}

fn guest_process(sandbox: &mut Sandbox, input_size: usize) -> Vec<u8> {
    // BUG: The guest allocates directly, bypassing the sandbox.
    // The sandbox has no chance to enforce the memory limit.
    let buffer = vec![0u8; input_size];

    // The sandbox "checks" after the allocation -- too late!
    sandbox.memory_used += input_size;
    if sandbox.memory_used > sandbox.memory_limit {
        panic!(
            "Memory limit exceeded: used {} > limit {}",
            sandbox.memory_used, sandbox.memory_limit
        );
    }

    buffer
}

fn main() {
    let mut sandbox = Sandbox::new(1024); // 1KB limit

    // Small allocation -- works
    let _buf1 = guest_process(&mut sandbox, 512);
    println!("Allocated 512 bytes (used: {})", sandbox.memory_used);

    // This exceeds the limit but the allocation already happened!
    let _buf2 = guest_process(&mut sandbox, 2048);
    println!("This line should not be reached");
}
```

## Correct Code

```rust
#[derive(Debug)]
enum SandboxError {
    MemoryLimitExceeded { requested: usize, available: usize },
}

struct Sandbox {
    memory_limit: usize,
    memory_used: usize,
}

impl Sandbox {
    fn new(memory_limit: usize) -> Self {
        Sandbox {
            memory_limit,
            memory_used: 0,
        }
    }

    /// Allocate memory through the sandbox. Checks limits BEFORE allocating.
    fn alloc(&mut self, size: usize) -> Result<Vec<u8>, SandboxError> {
        let available = self.memory_limit.saturating_sub(self.memory_used);
        if size > available {
            return Err(SandboxError::MemoryLimitExceeded {
                requested: size,
                available,
            });
        }
        // Limit check passed -- safe to allocate
        self.memory_used += size;
        Ok(vec![0u8; size])
    }

    fn free(&mut self, size: usize) {
        self.memory_used = self.memory_used.saturating_sub(size);
    }

    fn used(&self) -> usize {
        self.memory_used
    }

    fn available(&self) -> usize {
        self.memory_limit.saturating_sub(self.memory_used)
    }
}

fn guest_process(sandbox: &mut Sandbox, input_size: usize) -> Result<Vec<u8>, SandboxError> {
    // Correct: all allocation goes through the sandbox
    let buffer = sandbox.alloc(input_size)?;
    Ok(buffer)
}

fn main() {
    let mut sandbox = Sandbox::new(1024);

    // Small allocation -- works
    match guest_process(&mut sandbox, 512) {
        Ok(_buf) => println!("Allocated 512 bytes (used: {}, available: {})",
            sandbox.used(), sandbox.available()),
        Err(e) => println!("Failed: {:?}", e),
    }

    // Exceeds limit -- returns Err, no panic, no allocation
    match guest_process(&mut sandbox, 2048) {
        Ok(_buf) => println!("This should not happen"),
        Err(e) => println!("Correctly denied: {:?}", e),
    }

    // Can still allocate within the remaining budget
    match guest_process(&mut sandbox, 256) {
        Ok(_buf) => println!("Allocated 256 bytes (used: {}, available: {})",
            sandbox.used(), sandbox.available()),
        Err(e) => println!("Failed: {:?}", e),
    }

    println!("Sandbox intact: used={}, limit=1024", sandbox.used());
}
```

## Explanation

The broken version allocates memory directly (`vec![0u8; input_size]`) before checking the sandbox limit. The allocation succeeds (the Rust allocator does not know about our sandbox limit), and then the code checks the limit after the fact. By the time the panic occurs, the memory has already been allocated and the system is in an inconsistent state.

**The principle: check before, not after.**

In WASM runtimes, the sandbox intercepts the `memory.grow` instruction:

1. Guest calls `memory.grow(1)` (requests 1 page = 64KB)
2. Runtime checks: `current_pages + 1 <= max_pages?`
3. If yes: extends linear memory, returns old page count
4. If no: returns -1 (failure), memory is unchanged

The key is that step 2 happens *before* step 3. If the limit would be exceeded, the growth never happens. The guest sees a failure return code and must handle it.

**Why post-check is dangerous:**

If the check happens after the allocation:
- The system has already committed the resources
- Aborting (via panic) leaves the system in an inconsistent state
- In a real WASM runtime, the memory has already been mapped -- you cannot easily "un-map" it
- Other guests sharing the same host may be starved of memory

**The sandbox as mediator:**

The correct pattern ensures all resource acquisition goes through the sandbox:
- Memory: `sandbox.alloc(size)` instead of `Vec::with_capacity(size)`
- I/O: `sandbox.write(fd, data)` instead of `std::fs::write(path, data)`
- CPU: `sandbox.consume_fuel(cost)` before executing each instruction

If the guest bypasses the sandbox (calls the allocator directly), limits are not enforced. In real WASM, this is structurally impossible because the guest's only allocator IS the sandbox (linear memory controlled by the runtime).

The invariant violated in the broken code: **resource limits must be checked before allocation, not after; the sandbox must mediate all resource access.**

## ⚠️ Caution

- Checking resource limits after allocation is too late — the damage (OOM, excessive CPU) has already occurred. Always check BEFORE allocating.
- Resource limits must cover ALL resources: memory, CPU, file handles, network connections. Missing one creates an escape vector.

## 💡 Tips

- Use the "pre-check" pattern: validate resource availability before every operation.
- Implement a `ResourceBudget` struct that tracks usage and returns `Err` when limits are exceeded.
- WASM runtimes like Wasmtime provide built-in fuel metering and memory limits.

## Compiler Error Interpretation

```
thread 'main' panicked at 'Memory limit exceeded: used 2560 > limit 1024',
  src/main.rs:23:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

This is a runtime panic from the post-allocation check:

1. **"used 2560 > limit 1024"** -- after allocating 512 + 2048 = 2560 bytes, the total exceeds the 1024-byte limit.
2. The 2048-byte allocation already succeeded (the memory exists on the heap), but the limit check happened too late.

The panic is a poor recovery mechanism: it unwinds the stack, potentially leaving other sandbox state inconsistent. The `Result`-based approach in the correct version prevents the allocation from happening at all, keeping the system in a consistent state.
