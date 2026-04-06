---
id: closures-as-trait-objects
phase: 5
phase_title: "Closures & the Fn Traits"
sequence: 4
title: "Closures as Trait Objects vs Generic Parameters"
hints:
  - "Every closure has a unique, anonymous type — you cannot name it directly."
  - "Generics (`impl Fn`) create a specialized copy for each closure type (static dispatch). `dyn Fn` uses a vtable (dynamic dispatch)."
  - "Use generics when you know the closure type at compile time. Use `dyn Fn` when you need to store different closures in the same collection."
---

## Description

Every closure in Rust has a unique, anonymous type that the compiler generates. You cannot write this type by name. So how do you accept a closure as a parameter, store it in a struct, or put different closures in a collection?

There are two approaches:

1. **Generics with trait bounds** (`impl Fn(...)` or `fn foo<F: Fn(...)>(f: F)`) — static dispatch, monomorphized, zero-cost. The compiler generates a specialized version for each closure.
2. **Trait objects** (`Box<dyn Fn(...)>` or `&dyn Fn(...)`) — dynamic dispatch, vtable lookup at runtime. Allows storing heterogeneous closures in the same collection.

Choosing between them is a design decision with real tradeoffs.

## Broken Code

```rust
struct EventSystem {
    // We want to store multiple different event handlers.
    // Each handler is a closure, but every closure has a different type.
    // Vec<impl Fn(...)> does not work — impl Trait is not allowed here.
    handlers: Vec<impl Fn(&str)>,
}

impl EventSystem {
    fn new() -> Self {
        EventSystem { handlers: vec![] }
    }

    fn on_event(&mut self, handler: impl Fn(&str)) {
        self.handlers.push(handler);
    }

    fn emit(&self, event: &str) {
        for handler in &self.handlers {
            handler(event);
        }
    }
}

fn main() {
    let mut system = EventSystem::new();

    system.on_event(|e| println!("Logger: {}", e));
    system.on_event(|e| println!("Analytics: {}", e));

    system.emit("user_login");
}
```

## Correct Code

```rust
// --- Approach 1: Trait objects (dyn) — for heterogeneous collections ---

struct EventSystem {
    // Box<dyn Fn(&str)> erases the concrete closure type.
    // All handlers have the same type: Box<dyn Fn(&str)>.
    // This lets us store different closures in the same Vec.
    handlers: Vec<Box<dyn Fn(&str)>>,
}

impl EventSystem {
    fn new() -> Self {
        EventSystem { handlers: vec![] }
    }

    fn on_event(&mut self, handler: Box<dyn Fn(&str)>) {
        self.handlers.push(handler);
    }

    fn emit(&self, event: &str) {
        for handler in &self.handlers {
            handler(event);
        }
    }
}

// --- Approach 2: Generics — for single closures, zero-cost ---

fn apply_twice<F: Fn(i32) -> i32>(f: F, value: i32) -> i32 {
    f(f(value))
}

// Equivalent syntax using `impl Trait` in argument position:
fn apply_once(f: impl Fn(i32) -> i32, value: i32) -> i32 {
    f(value)
}

fn main() {
    // --- Trait objects: heterogeneous collection ---
    let mut system = EventSystem::new();

    system.on_event(Box::new(|e| println!("Logger: {}", e)));
    system.on_event(Box::new(|e| println!("Analytics: {}", e)));

    // Both handlers run despite having different closure types.
    system.emit("user_login");

    // --- Generics: static dispatch, zero-cost ---
    let double = |x| x * 2;
    let result = apply_twice(double, 3); // 3 -> 6 -> 12
    println!("\napply_twice(double, 3) = {}", result);

    // impl Fn in argument position — syntactic sugar for generics
    let add_one = |x| x + 1;
    println!("apply_once(add_one, 10) = {}", apply_once(add_one, 10));
}
```

## Explanation

The broken code tries to use `Vec<impl Fn(&str)>` as a field type. This fails because `impl Trait` in field position is not allowed — `impl Fn(&str)` means "some single concrete type that implements `Fn(&str)`," but the compiler needs to know the exact type to lay out the struct in memory. Different closures have different types, so a `Vec` of `impl Fn` is contradictory: it would need to hold values of different sizes.

**The two solutions represent a fundamental design choice:**

**Static dispatch (generics):**

```rust
fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 { f(x) }
```

The compiler generates a separate `apply` function for each closure type. No heap allocation, no vtable, the closure call is inlined. Use this when:
- You accept a single closure as a parameter
- Performance matters (hot loops, tight code)
- You know the closure type at compile time

**Dynamic dispatch (trait objects):**

```rust
handlers: Vec<Box<dyn Fn(&str)>>
```

`Box<dyn Fn(&str)>` erases the concrete type and stores a vtable pointer alongside the data pointer. Every closure is heap-allocated and called through indirection. Use this when:
- You need to store multiple different closures together
- The set of closures is determined at runtime (plugin systems, event handlers)
- You need to return different closures from the same function

**Cost comparison:**

| Aspect | Generics (`impl Fn`) | Trait objects (`dyn Fn`) |
|---|---|---|
| Dispatch | Static (inlined) | Dynamic (vtable) |
| Heap allocation | No | Yes (with `Box`) |
| Binary size | Larger (monomorphization) | Smaller |
| Heterogeneous collection | No | Yes |
| Runtime flexibility | No | Yes |

**Why closures make this distinction sharp:**

Unlike named types where you can write `Vec<MyType>`, closures have anonymous types. You literally cannot name them. This forces the choice: either let the compiler monomorphize (generics), or erase the type behind a pointer (trait objects). There is no third option.

## ⚠️ Caution

- `Box<dyn Fn()>` requires the closure to be `Fn` (can be called multiple times). If the closure captures by move and consumes its captures, you need `Box<dyn FnOnce()>`, but `FnOnce` closures can only be called once.
- Trait objects add one level of indirection per call. In tight loops, this can prevent inlining and hurt performance. Profile before reaching for `dyn`.
- If the closure needs to be `Send` (for use across threads), write `Box<dyn Fn(&str) + Send>`.

## 💡 Tips

- Default to generics (`impl Fn(...)`) for function parameters — it is simpler and faster.
- Reach for `Box<dyn Fn(...)>` only when you need to store closures in collections, structs, or return them dynamically.
- The `impl Fn(A) -> B` syntax in argument position is sugar for a generic parameter with a trait bound. In return position, it means "one concrete type that the caller cannot name."
- You can combine: accept a generic closure parameter and box it for storage: `fn register(f: impl Fn(&str) + 'static) { self.handlers.push(Box::new(f)); }`

## Compiler Error Interpretation

```
error[E0562]: `impl Trait` is not allowed in field types
 --> src/main.rs:5:18
  |
5 |     handlers: Vec<impl Fn(&str)>,
  |                   ^^^^^^^^^^^^^
  |
  = note: `impl Trait` is only allowed in function and inherent method
          argument and return types
```

The compiler is clear: `impl Trait` can appear in function signatures (arguments and return types), but not in struct fields, `let` bindings, or `static` items. For struct fields, you must choose between:

1. A generic parameter: `struct EventSystem<F: Fn(&str)>` — but then all handlers must be the same closure type.
2. A trait object: `Vec<Box<dyn Fn(&str)>>` — allows different closure types, at the cost of heap allocation and dynamic dispatch.

For an event system where handlers are registered at runtime, trait objects are the right choice.

---

| [Prev: move Closures and Ownership Transfer](#/katas/move-closures) | [Next: Vec<T> and Slices: Owned vs Borrowed Collections](#/katas/vec-and-slices) |
