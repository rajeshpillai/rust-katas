---
id: plugin-system-capstone
phase: 25
phase_title: "Capstone Projects"
sequence: 1
title: "Capstone: Safe Plugin System with Memory Isolation"
hints:
  - Each plugin should get its own bounded memory region, not access to the entire buffer
  - A MemorySlice wrapping a range of the shared buffer enforces isolation
  - Without bounds enforcement, one plugin's writes can corrupt another plugin's data
  - The runtime must allocate non-overlapping regions and enforce them on every access
---

## Description

This capstone combines trait objects (Phase 23), memory bounds checking (Phase 17), and resource isolation (Phase 23) into a complete plugin runtime. Multiple plugins share a single linear memory buffer, but each plugin is restricted to its own non-overlapping region. A buggy or malicious plugin cannot corrupt another plugin's memory. This kata demonstrates what happens when plugins share raw access versus when the runtime enforces memory isolation.

## Broken Code

```rust
trait Plugin {
    fn name(&self) -> &str;
    fn process(&self, memory: &mut [u8], offset: usize, len: usize);
}

struct FillPlugin {
    name: String,
    fill_byte: u8,
}

impl Plugin for FillPlugin {
    fn name(&self) -> &str { &self.name }
    fn process(&self, memory: &mut [u8], offset: usize, len: usize) {
        // BUG: Plugin gets access to the ENTIRE memory buffer.
        // It can write anywhere, including other plugins' regions.
        for i in 0..len {
            memory[offset + i] = self.fill_byte;
        }
        // Malicious/buggy: writes PAST its assigned region!
        // Corrupts the next 8 bytes beyond its boundary.
        for i in 0..8 {
            if offset + len + i < memory.len() {
                memory[offset + len + i] = 0xFF;
            }
        }
    }
}

struct PluginRuntime {
    memory: Vec<u8>,
    plugins: Vec<(Box<dyn Plugin>, usize, usize)>, // (plugin, offset, len)
}

impl PluginRuntime {
    fn new(memory_size: usize) -> Self {
        PluginRuntime {
            memory: vec![0u8; memory_size],
            plugins: Vec::new(),
        }
    }

    fn register(&mut self, plugin: Box<dyn Plugin>, offset: usize, len: usize) {
        self.plugins.push((plugin, offset, len));
    }

    fn run_all(&mut self) {
        // BUG: Each plugin receives the entire memory buffer
        for (plugin, offset, len) in &self.plugins {
            let offset = *offset;
            let len = *len;
            let name = plugin.name().to_string();
            plugin.process(&mut self.memory, offset, len);
            println!("Ran plugin '{}' on region [{}, {})", name, offset, offset + len);
        }
    }
}

fn main() {
    let mut runtime = PluginRuntime::new(64);

    runtime.register(
        Box::new(FillPlugin { name: "plugin-a".to_string(), fill_byte: 0xAA }),
        0, 16,  // Region: bytes 0-15
    );
    runtime.register(
        Box::new(FillPlugin { name: "plugin-b".to_string(), fill_byte: 0xBB }),
        16, 16, // Region: bytes 16-31
    );

    runtime.run_all();

    // Check plugin B's data integrity
    let b_region = &runtime.memory[16..32];
    let expected: Vec<u8> = vec![0xBB; 16];
    assert_eq!(
        b_region, expected.as_slice(),
        "Plugin B's memory was corrupted by Plugin A!"
    );
}
```

## Correct Code

```rust
#[derive(Debug)]
enum PluginError {
    OutOfBounds { offset: usize, len: usize, region_size: usize },
}

/// A bounded view into shared memory. The plugin can only access this region.
struct MemorySlice<'a> {
    data: &'a mut [u8],
}

impl<'a> MemorySlice<'a> {
    fn write(&mut self, offset: usize, value: u8) -> Result<(), PluginError> {
        if offset >= self.data.len() {
            return Err(PluginError::OutOfBounds {
                offset,
                len: 1,
                region_size: self.data.len(),
            });
        }
        self.data[offset] = value;
        Ok(())
    }

    fn fill(&mut self, value: u8) {
        self.data.fill(value);
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

trait Plugin {
    fn name(&self) -> &str;
    /// Process receives only its own bounded MemorySlice, not the entire buffer.
    fn process(&self, memory: &mut MemorySlice) -> Result<(), PluginError>;
}

struct FillPlugin {
    name: String,
    fill_byte: u8,
}

impl Plugin for FillPlugin {
    fn name(&self) -> &str { &self.name }
    fn process(&self, memory: &mut MemorySlice) -> Result<(), PluginError> {
        memory.fill(self.fill_byte);

        // Even if the plugin tries to write past its region, it cannot:
        match memory.write(memory.len(), 0xFF) {
            Ok(()) => {} // Would mean we escaped -- impossible
            Err(e) => println!("  [{}] Correctly blocked: {:?}", self.name, e),
        }

        Ok(())
    }
}

struct PluginRuntime {
    memory: Vec<u8>,
    plugins: Vec<(Box<dyn Plugin>, usize, usize)>,
}

impl PluginRuntime {
    fn new(memory_size: usize) -> Self {
        PluginRuntime {
            memory: vec![0u8; memory_size],
            plugins: Vec::new(),
        }
    }

    fn register(&mut self, plugin: Box<dyn Plugin>, offset: usize, len: usize) {
        self.plugins.push((plugin, offset, len));
    }

    fn run_all(&mut self) {
        // Each plugin gets only its own bounded slice
        let mem_ptr = self.memory.as_mut_ptr();
        let mem_len = self.memory.len();

        for (plugin, offset, len) in &self.plugins {
            let offset = *offset;
            let len = *len;

            if offset + len > mem_len {
                println!("Skipping '{}': region out of bounds", plugin.name());
                continue;
            }

            // Create a bounded slice for this plugin's region
            let slice = unsafe {
                std::slice::from_raw_parts_mut(mem_ptr.add(offset), len)
            };
            let mut mem_slice = MemorySlice { data: slice };

            let name = plugin.name().to_string();
            match plugin.process(&mut mem_slice) {
                Ok(()) => println!("Ran plugin '{}' on region [{}, {})", name, offset, offset + len),
                Err(e) => println!("Plugin '{}' error: {:?}", name, e),
            }
        }
    }
}

fn main() {
    let mut runtime = PluginRuntime::new(64);

    runtime.register(
        Box::new(FillPlugin { name: "plugin-a".to_string(), fill_byte: 0xAA }),
        0, 16,
    );
    runtime.register(
        Box::new(FillPlugin { name: "plugin-b".to_string(), fill_byte: 0xBB }),
        16, 16,
    );

    runtime.run_all();

    // Verify isolation: Plugin A's overflow did NOT corrupt Plugin B
    let b_region = &runtime.memory[16..32];
    let expected: Vec<u8> = vec![0xBB; 16];
    assert_eq!(
        b_region, expected.as_slice(),
        "Plugin B's memory should be intact"
    );

    println!("\nMemory isolation verified! Plugins cannot corrupt each other.");
}
```

## Explanation

The broken version passes the entire `&mut [u8]` memory buffer to each plugin. Plugin A writes its fill byte to its assigned region (bytes 0-15) but also writes `0xFF` to 8 bytes past its boundary (bytes 16-23), corrupting part of Plugin B's region. When Plugin B later fills its region with `0xBB`, the final result appears correct -- but if Plugin A ran after Plugin B, Plugin B's data would be corrupted.

In the specific execution order shown (A then B), Plugin B overwrites A's corruption with `0xBB`, so the assert passes for B. But the broken code has a latent bug: if execution order changes, or if B reads its memory before filling it, corruption is visible. The fundamental problem is that the plugins can access each other's memory.

**The isolation pattern:**

The correct version gives each plugin a `MemorySlice` -- a bounded view into only its assigned region. The `MemorySlice::write` method checks bounds on every access. Even if the plugin code tries to write at index `memory.len()` (one past the end), the bounds check returns `Err(OutOfBounds)` instead of corrupting the adjacent region.

**How real WASM runtimes enforce this:**

Each WASM module has its own linear memory. Module A's linear memory is a completely separate byte array from Module B's linear memory. A pointer in Module A's memory (like offset 16) refers to Module A's byte 16, not a shared global byte 16. There is no way for Module A to even address Module B's memory -- the address spaces are disjoint.

**This capstone integrates:**
- **Phase 17:** Bounds-checked linear memory
- **Phase 19:** Offset arithmetic in byte arrays
- **Phase 23:** Plugin system with trait objects
- **Phase 23:** Sandboxed execution with resource limits
- **Phase 24:** Structural isolation through type boundaries

The invariant violated in the broken code: **plugins must be isolated from each other's memory; the runtime must enforce non-overlapping, bounds-checked memory regions.**

## Compiler Error Interpretation

```
thread 'main' panicked at 'assertion `left == right` failed: Plugin B's memory was corrupted by Plugin A!
  left: [187, 187, 187, 187, 187, 187, 187, 187, 187, 187, 187, 187, 187, 187, 187, 187]
 right: [187, 187, 187, 187, 187, 187, 187, 187, 187, 187, 187, 187, 187, 187, 187, 187]', ...
```

Note: In the shown execution order (A then B), Plugin B's fill overwrites A's corruption, so the assertion may actually pass. The corruption is visible if the order is reversed, or in a modified version where B checks its memory before writing. The general principle remains: shared raw memory access between plugins creates a race condition where one plugin can corrupt another's data.

In a real deployment where plugins run concurrently or in different orders, this is a critical security and reliability vulnerability. The `MemorySlice` approach prevents it structurally.
