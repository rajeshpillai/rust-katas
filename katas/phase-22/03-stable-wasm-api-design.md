---
id: stable-wasm-api-design
phase: 22
phase_title: "Rust Ownership Patterns for WASM"
sequence: 3
title: Stable WASM API Design — Opaque Handles Instead of Pointers
hints:
  - Raw indices into a Vec are invalidated when elements are removed
  - A HashMap with monotonically increasing IDs creates stable handles
  - Accessing a freed handle should return None, not corrupt data
  - This is exactly how wasm-bindgen manages JavaScript objects from Rust
---

## Description

Exposing raw indices or pointers across the WASM boundary is fragile. If the guest stores resources in a `Vec` and returns indices to the host, removing an element shifts all subsequent indices, invalidating handles the host already holds. The correct pattern uses opaque handles: monotonically increasing integer IDs backed by a `HashMap`. Removing a resource does not affect other handles. This is exactly how `wasm-bindgen` manages JavaScript objects from Rust and how WASI manages file descriptors.

## Broken Code

```rust
struct Resource {
    name: String,
    data: Vec<u8>,
}

struct ResourceManager {
    resources: Vec<Resource>,
}

impl ResourceManager {
    fn new() -> Self {
        ResourceManager {
            resources: Vec::new(),
        }
    }

    /// Create a resource and return its index as a handle.
    fn create(&mut self, name: &str, data: Vec<u8>) -> usize {
        let index = self.resources.len();
        self.resources.push(Resource {
            name: name.to_string(),
            data,
        });
        index
    }

    /// Remove a resource by index.
    fn remove(&mut self, index: usize) {
        self.resources.remove(index);
        // BUG: Vec::remove shifts all elements after `index` down by one.
        // This invalidates all handles with index > the removed index.
    }

    fn get(&self, index: usize) -> &Resource {
        &self.resources[index]
    }
}

fn main() {
    let mut mgr = ResourceManager::new();

    let handle_a = mgr.create("texture-a", vec![1, 2, 3]);  // index 0
    let handle_b = mgr.create("texture-b", vec![4, 5, 6]);  // index 1
    let handle_c = mgr.create("texture-c", vec![7, 8, 9]);  // index 2

    println!("Created: A={}, B={}, C={}", handle_a, handle_b, handle_c);

    // Remove B (index 1) -- C shifts from index 2 to index 1
    mgr.remove(handle_b);

    // Try to access C using its original handle (2)
    // BUG: C is now at index 1, and index 2 is out of bounds!
    let resource = mgr.get(handle_c);
    println!("Resource C: {}", resource.name);
}
```

## Correct Code

```rust
use std::collections::HashMap;

struct Resource {
    name: String,
    data: Vec<u8>,
}

struct ResourceManager {
    resources: HashMap<u32, Resource>,
    next_id: u32,
}

impl ResourceManager {
    fn new() -> Self {
        ResourceManager {
            resources: HashMap::new(),
            next_id: 1, // Start at 1 so 0 can mean "null handle"
        }
    }

    /// Create a resource and return a stable opaque handle.
    fn create(&mut self, name: &str, data: Vec<u8>) -> u32 {
        let handle = self.next_id;
        self.next_id += 1;
        self.resources.insert(handle, Resource {
            name: name.to_string(),
            data,
        });
        handle
    }

    /// Remove a resource by handle. Does not affect other handles.
    fn remove(&mut self, handle: u32) -> bool {
        self.resources.remove(&handle).is_some()
    }

    /// Get a resource by handle. Returns None for freed handles.
    fn get(&self, handle: u32) -> Option<&Resource> {
        self.resources.get(&handle)
    }
}

fn main() {
    let mut mgr = ResourceManager::new();

    let handle_a = mgr.create("texture-a", vec![1, 2, 3]);  // handle 1
    let handle_b = mgr.create("texture-b", vec![4, 5, 6]);  // handle 2
    let handle_c = mgr.create("texture-c", vec![7, 8, 9]);  // handle 3

    println!("Created: A={}, B={}, C={}", handle_a, handle_b, handle_c);

    // Remove B -- does not affect A or C's handles
    let removed = mgr.remove(handle_b);
    println!("Removed B (handle {}): {}", handle_b, removed);

    // Access C using its original handle -- still works!
    match mgr.get(handle_c) {
        Some(r) => println!("Resource C: {}", r.name),
        None => println!("Handle {} not found", handle_c),
    }

    // Accessing freed handle B returns None, does not panic
    match mgr.get(handle_b) {
        Some(r) => println!("Resource B: {}", r.name),
        None => println!("Handle {} was freed (expected)", handle_b),
    }

    // A is still accessible
    match mgr.get(handle_a) {
        Some(r) => println!("Resource A: {}", r.name),
        None => println!("Handle {} not found", handle_a),
    }
}
```

## Explanation

The broken version uses `Vec` indices as handles. When `remove(1)` is called, `Vec::remove` shifts element C from index 2 to index 1. The handle `2` (which the host holds for resource C) now points past the end of the vector, causing a panic.

**Why Vec indices are unstable handles:**

A `Vec` is a contiguous array. Removing an element in the middle shifts all subsequent elements down. This means:
- Insert A(0), B(1), C(2)
- Remove B → C moves from index 2 to index 1
- Handle 2 now points to nothing (or to the wrong resource if a new resource was added)

This is a use-after-free bug at the logical level: the handle became invalid without the holder knowing.

**The opaque handle pattern:**

A `HashMap<u32, Resource>` with monotonically increasing IDs solves this:
- Handles are never reused (monotonically increasing)
- Removing a handle does not affect other handles (HashMap does not shift entries)
- Looking up a freed handle returns `None`, not garbage

This is exactly how WASM API designs work in practice:
- **`wasm-bindgen`**: JavaScript objects are stored in a slab table with integer handles. When Rust holds a `JsValue`, it is actually an index into this table.
- **WASI file descriptors**: File handles are integers that map to an internal table. Closing a file descriptor invalidates that handle but does not affect others.
- **WebGPU**: GPU resources (buffers, textures) are tracked by opaque integer handles.

**Starting IDs at 1:**

The correct version starts `next_id` at 1 so that handle `0` can serve as a "null handle" -- similar to how null pointers are 0 and how WASI uses 0 as stdin's file descriptor. This is a common convention in C and WASM APIs.

The invariant violated in the broken code: **resource handles must be stable across mutations; removing one resource must not invalidate handles to other resources.**

## ⚠️ Caution

- Handle IDs wrapping around after u32::MAX allocations can cause handle reuse and security issues. Start IDs at 1 and consider generation counters.
- Leaking handles (creating without destroying) exhausts the handle space over time.

## 💡 Tips

- Use opaque integer handles (not pointers) for WASM API resources.
- Start handle IDs at 1 so that 0 can serve as a "null handle."
- Study real-world handle-based APIs: WASI file descriptors, WebGPU handles, wasm-bindgen slab tables.

## Compiler Error Interpretation

```
thread 'main' panicked at 'index out of bounds: the len is 2
  but the index is 2', src/main.rs:40:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

This is a runtime panic from accessing an invalid index:

1. **"the len is 2 but the index is 2"** -- after removing B, the Vec has 2 elements (A and C, at indices 0 and 1). But the code uses handle `2` (C's original index), which is now out of bounds.
2. The `get` method uses `self.resources[index]`, which panics for out-of-bounds access.

In a real WASM system, this could corrupt memory: if the Vec had grown again (a new resource added at index 2), the handle would silently point to the wrong resource. The HashMap-based approach prevents both panics and silent corruption by returning `None` for freed handles.

---

| [Prev: Zero-Copy Data Access — Slices vs Clones](#/katas/zero-copy-slices) | [Next: Plugin System — Dynamic Loading via Trait Objects](#/katas/plugin-system-traits) |
