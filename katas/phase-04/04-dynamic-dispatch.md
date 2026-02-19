---
id: dynamic-dispatch
phase: 4
phase_title: "Traits & Generics"
sequence: 4
title: Dynamic Dispatch and Object Safety
hints:
  - "Not every trait can be used as `dyn Trait`. The trait must be 'object safe.'"
  - "A trait is NOT object safe if it has methods that return `Self` or have generic type parameters."
  - "The compiler needs a vtable with fixed function signatures. Generic methods would require infinite vtable entries."
---

## Description

When you use `dyn Trait`, Rust creates a **trait object**: a fat pointer containing a pointer to the data and a pointer to a vtable of method implementations. This enables dynamic dispatch — the method to call is determined at runtime, not compile time.

However, not every trait can be turned into a trait object. The trait must be **object safe**. A trait is object safe if the compiler can construct a vtable for it. This fails when:

1. A method returns `Self` (the concrete type is erased, so the compiler does not know the size).
2. A method has generic type parameters (each instantiation would need its own vtable entry — the vtable would be infinite).

This kata explores what happens when you try to use a non-object-safe trait as `dyn Trait`.

## Broken Code

```rust
trait Cloneable {
    fn clone_self(&self) -> Self;
}

struct Widget {
    id: u32,
}

impl Cloneable for Widget {
    fn clone_self(&self) -> Self {
        Widget { id: self.id }
    }
}

struct Gadget {
    name: String,
}

impl Cloneable for Gadget {
    fn clone_self(&self) -> Self {
        Gadget {
            name: self.name.clone(),
        }
    }
}

// This fails: Cloneable is not object safe because
// clone_self returns Self, and behind dyn the concrete
// type (and thus the return size) is unknown.
fn clone_thing(thing: &dyn Cloneable) {
    let _copy = thing.clone_self();
}

fn main() {
    let w = Widget { id: 42 };
    clone_thing(&w);
}
```

## Correct Code

```rust
// Solution: Return a Box<dyn CloneableBoxed> instead of Self,
// so the return type has a known size regardless of the concrete type.

trait CloneableBoxed {
    fn clone_boxed(&self) -> Box<dyn CloneableBoxed>;
    fn describe(&self) -> String;
}

struct Widget {
    id: u32,
}

impl CloneableBoxed for Widget {
    fn clone_boxed(&self) -> Box<dyn CloneableBoxed> {
        Box::new(Widget { id: self.id })
    }

    fn describe(&self) -> String {
        format!("Widget #{}", self.id)
    }
}

struct Gadget {
    name: String,
}

impl CloneableBoxed for Gadget {
    fn clone_boxed(&self) -> Box<dyn CloneableBoxed> {
        Box::new(Gadget {
            name: self.name.clone(),
        })
    }

    fn describe(&self) -> String {
        format!("Gadget '{}'", self.name)
    }
}

fn clone_thing(thing: &dyn CloneableBoxed) -> Box<dyn CloneableBoxed> {
    thing.clone_boxed()
}

fn main() {
    let w = Widget { id: 42 };
    let cloned = clone_thing(&w);
    println!("Original: {}", w.describe());
    println!("Cloned: {}", cloned.describe());
}
```

## Explanation

The fundamental issue is that **trait objects erase the concrete type**. When you have a `&dyn Cloneable`, Rust no longer knows at compile time whether the value behind the pointer is a `Widget` (4 bytes) or a `Gadget` (24 bytes on 64-bit). The vtable tells Rust which `clone_self` to call, but the return type `Self` requires knowing the concrete type to allocate the correct amount of space on the stack.

This is the **object safety** rule. A trait is object safe if all its methods satisfy these conditions:

1. **No `Self` in return position** — because the size of `Self` is unknown behind `dyn`.
2. **No generic type parameters on methods** — because each instantiation would need a separate vtable entry, making the vtable unbounded.
3. **The method must have a receiver** (`&self`, `&mut self`, `self`, `Box<Self>`, etc.) — free functions have no way to be dispatched dynamically.

The fix replaces `-> Self` with `-> Box<dyn CloneableBoxed>`. Now the return type is always the same size (a `Box` is just a pointer), regardless of the concrete type. The heap allocation absorbs the size difference.

**Rules of thumb for object safety:**

| Feature | Object Safe? | Why |
|---|---|---|
| `&self` methods | Yes | Receiver is a known pointer size |
| `-> Self` return | No | Unknown size behind `dyn` |
| Generic methods `fn foo<T>()` | No | Would need infinite vtable entries |
| `where Self: Sized` methods | Skipped | Opted out of dynamic dispatch |
| Associated types | Yes | Resolved per impl, fixed in vtable |

You can also mark specific methods as unavailable for trait objects using `where Self: Sized`:

```rust
trait Cloneable {
    fn clone_self(&self) -> Self
    where
        Self: Sized; // This method is excluded from dyn dispatch

    fn describe(&self) -> String; // This method is still available via dyn
}
```

This lets the trait be used as `dyn Cloneable`, but only the `describe` method is available through the trait object.

## Compiler Error Interpretation

```
error[E0038]: the trait `Cloneable` cannot be made into an object
  --> src/main.rs:27:27
   |
27 | fn clone_thing(thing: &dyn Cloneable) {
   |                           ^^^^^^^^^ `Cloneable` cannot be made into an object
   |
note: for a trait to be "object safe" it needs to allow building a vtable to allow the call to be resolvable dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-safety>
  --> src/main.rs:2:27
   |
1  | trait Cloneable {
   |       --------- this trait cannot be made into an object...
2  |     fn clone_self(&self) -> Self;
   |                             ^^^^ ...because method `clone_self` references the `Self` type in its return type
   = help: consider moving `clone_self` to another trait
```

This is one of the most informative error messages in Rust:

- **"`Cloneable` cannot be made into an object"** — Direct statement of the problem: this trait is not object safe.
- **"it needs to allow building a vtable"** — Rust explains the underlying mechanism. A vtable is a table of function pointers, and each entry must have a fixed signature.
- **"method `clone_self` references the `Self` type in its return type"** — Pinpoints the exact method and the exact reason. `Self` in return position means the vtable entry would need to return a value of unknown size.
- **"consider moving `clone_self` to another trait"** — One possible fix is to split the trait so that the object-safe methods are in one trait and the non-object-safe methods are in another.
