---
id: plugin-system-traits
phase: 23
phase_title: "WASM Outside the Browser"
sequence: 1
title: Plugin System — Dynamic Loading via Trait Objects
hints:
  - A trait is unsized -- you cannot put it directly in a Vec
  - Trait objects need indirection through Box, &, or Arc
  - "Vec<Box<dyn Plugin>>" stores different plugin types in the same collection
  - This models how WASM runtimes load modules with a shared interface
---

## Description

WASM enables plugin systems where untrusted modules run in sandboxes with a shared interface. Each plugin implements the same contract (exports), but the host does not know the concrete types at compile time. In Rust, this maps to trait objects: `Box<dyn Plugin>` allows storing different plugin implementations in the same collection. This kata demonstrates that traits are unsized and cannot be stored directly in a `Vec` -- you must use `Box<dyn Trait>` for dynamic dispatch.

## Broken Code

```rust
trait Plugin {
    fn name(&self) -> &str;
    fn execute(&self, input: &str) -> String;
}

struct UppercasePlugin;
impl Plugin for UppercasePlugin {
    fn name(&self) -> &str { "uppercase" }
    fn execute(&self, input: &str) -> String {
        input.to_uppercase()
    }
}

struct ReversePlugin;
impl Plugin for ReversePlugin {
    fn name(&self) -> &str { "reverse" }
    fn execute(&self, input: &str) -> String {
        input.chars().rev().collect()
    }
}

struct CountPlugin;
impl Plugin for CountPlugin {
    fn name(&self) -> &str { "count" }
    fn execute(&self, input: &str) -> String {
        format!("{} chars", input.len())
    }
}

struct PluginHost {
    // BUG: Plugin is a trait, not a concrete type. Traits are unsized.
    // You cannot store an unsized type directly in a Vec.
    plugins: Vec<Plugin>,
}

impl PluginHost {
    fn new() -> Self {
        PluginHost { plugins: Vec::new() }
    }

    fn register(&mut self, plugin: Plugin) {
        self.plugins.push(plugin);
    }

    fn run_all(&self, input: &str) {
        for plugin in &self.plugins {
            let result = plugin.execute(input);
            println!("[{}] {} -> {}", plugin.name(), input, result);
        }
    }
}

fn main() {
    let mut host = PluginHost::new();
    host.register(UppercasePlugin);
    host.register(ReversePlugin);
    host.register(CountPlugin);
    host.run_all("Hello, WASM!");
}
```

## Correct Code

```rust
trait Plugin {
    fn name(&self) -> &str;
    fn execute(&self, input: &str) -> String;
}

struct UppercasePlugin;
impl Plugin for UppercasePlugin {
    fn name(&self) -> &str { "uppercase" }
    fn execute(&self, input: &str) -> String {
        input.to_uppercase()
    }
}

struct ReversePlugin;
impl Plugin for ReversePlugin {
    fn name(&self) -> &str { "reverse" }
    fn execute(&self, input: &str) -> String {
        input.chars().rev().collect()
    }
}

struct CountPlugin;
impl Plugin for CountPlugin {
    fn name(&self) -> &str { "count" }
    fn execute(&self, input: &str) -> String {
        format!("{} chars", input.len())
    }
}

struct PluginHost {
    // Correct: Box<dyn Plugin> provides indirection for the unsized trait.
    // Each Box is a fat pointer: data pointer + vtable pointer.
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginHost {
    fn new() -> Self {
        PluginHost { plugins: Vec::new() }
    }

    fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    fn run_all(&self, input: &str) {
        for plugin in &self.plugins {
            let result = plugin.execute(input);
            println!("[{}] {} -> {}", plugin.name(), input, result);
        }
    }

    fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}

fn main() {
    let mut host = PluginHost::new();
    host.register(Box::new(UppercasePlugin));
    host.register(Box::new(ReversePlugin));
    host.register(Box::new(CountPlugin));

    println!("Loaded {} plugins:", host.plugin_count());
    host.run_all("Hello, WASM!");
}
```

## Explanation

The broken version uses `Vec<Plugin>` where `Plugin` is a trait. Traits are *unsized types* -- their size is not known at compile time because different implementations have different sizes (`UppercasePlugin` might have 0 bytes, while another plugin might have 100 bytes of state). A `Vec` requires all elements to have the same size, so it cannot hold an unsized type directly.

**How `Box<dyn Plugin>` works:**

`Box<dyn Plugin>` is a *fat pointer* containing:
1. A pointer to the heap-allocated data (the concrete plugin struct)
2. A pointer to the *vtable* (a table of function pointers for the trait's methods)

Each `Box<dyn Plugin>` is exactly 16 bytes (two pointers), regardless of the concrete plugin type. The `Vec` stores these fixed-size fat pointers, and method calls are dispatched through the vtable at runtime (dynamic dispatch).

**How this maps to WASM plugins:**

In a WASM plugin system (Extism, Spin, Envoy proxy):
1. Each plugin is a separate WASM module with exported functions
2. All plugins export the same interface (e.g., `fn name() -> String`, `fn execute(input: String) -> String`)
3. The host loads modules dynamically and calls their exports through function tables

The function table in WASM is exactly a vtable: a table of function pointers indexed by function name. `Box<dyn Plugin>` in Rust models this mechanism precisely.

**Static vs dynamic dispatch:**

| Approach | Rust | WASM |
|----------|------|------|
| Static dispatch | `fn run<P: Plugin>(p: &P)` | Compile-time known module |
| Dynamic dispatch | `fn run(p: &dyn Plugin)` | Runtime-loaded module |

Plugin systems inherently require dynamic dispatch because the host does not know which plugins will be loaded at compile time.

The invariant violated in the broken code: **traits are unsized; use `Box<dyn Trait>` to store different implementations in the same collection via dynamic dispatch.**

## ⚠️ Caution

- `dyn Trait` is a fat pointer (data + vtable), which adds a layer of indirection. For performance-critical paths, prefer generics (static dispatch).
- Object safety rules limit what traits can be used as `dyn Trait`. Traits with generic methods or `Self` return types are not object-safe.

## 💡 Tips

- Use `Box<dyn Plugin>` for plugin registries where plugins are loaded at runtime.
- The vtable approach mirrors how WASM host functions work — function pointers to imported capabilities.
- For static plugin systems, prefer generics over trait objects for better performance.

## Compiler Error Interpretation

```
error[E0782]: expected a type, found a trait
  --> src/main.rs:33:18
   |
33 |     plugins: Vec<Plugin>,
   |                  ^^^^^^
   |
help: you can add the `dyn` keyword if you want a trait object
   |
33 |     plugins: Vec<dyn Plugin>,
   |                  +++
```

The compiler error explains:

1. **"expected a type, found a trait"** -- in Rust edition 2021, using a bare trait name (`Plugin`) in type position is rejected. The compiler requires the explicit `dyn` keyword to indicate a trait object.
2. **"you can add the `dyn` keyword"** -- the compiler suggests `Vec<dyn Plugin>`, but this still would not work because `dyn Plugin` is unsized. `Vec` requires all elements to have a known, fixed size.
3. **The full fix:** Use `Vec<Box<dyn Plugin>>`. The `dyn` keyword makes the trait object explicit, and `Box` provides the indirection needed for an unsized type. `Box<dyn Plugin>` is always 16 bytes (two pointers: data + vtable), regardless of the concrete type inside.
