---
id: impl-trait
phase: 4
phase_title: "Traits & Generics"
sequence: 3
title: impl Trait in Argument and Return Position
hints:
  - "`impl Trait` in return position means the function returns exactly ONE concrete type that implements the trait."
  - "If you have two branches returning different concrete types, the compiler cannot determine a single concrete type at compile time."
  - "For returning different concrete types behind one trait, you need dynamic dispatch: `Box<dyn Trait>`."
---

## Description

Rust offers `impl Trait` as syntactic sugar in two positions:

- **Argument position**: `fn foo(x: impl Display)` is equivalent to `fn foo<T: Display>(x: T)`. The caller picks the concrete type.
- **Return position**: `fn bar() -> impl Display` means "I return some specific type that implements `Display`, but I'm not telling you which one." The compiler figures out the concrete type and monomorphizes it.

The critical constraint in return position is that the function must return **exactly one concrete type**. If different branches return different types (even if both implement the trait), the compiler cannot determine a single type at compile time.

## Broken Code

```rust
trait Describe {
    fn describe(&self) -> String;
}

struct Cat;
struct Dog;

impl Describe for Cat {
    fn describe(&self) -> String {
        String::from("I am a cat. I ignore you.")
    }
}

impl Describe for Dog {
    fn describe(&self) -> String {
        String::from("I am a dog. I love you!")
    }
}

// This fails: `impl Trait` must resolve to ONE concrete type,
// but this function can return either Cat or Dog.
fn get_pet(likes_dogs: bool) -> impl Describe {
    if likes_dogs {
        Dog
    } else {
        Cat
    }
}

fn main() {
    let pet = get_pet(true);
    println!("{}", pet.describe());
}
```

## Correct Code

```rust
trait Describe {
    fn describe(&self) -> String;
}

struct Cat;
struct Dog;

impl Describe for Cat {
    fn describe(&self) -> String {
        String::from("I am a cat. I ignore you.")
    }
}

impl Describe for Dog {
    fn describe(&self) -> String {
        String::from("I am a dog. I love you!")
    }
}

// Solution: Use Box<dyn Trait> for dynamic dispatch.
// The return type is now a heap-allocated trait object.
fn get_pet(likes_dogs: bool) -> Box<dyn Describe> {
    if likes_dogs {
        Box::new(Dog)
    } else {
        Box::new(Cat)
    }
}

fn main() {
    let pet = get_pet(true);
    println!("{}", pet.describe());
}
```

## Explanation

The key distinction is between **static dispatch** and **dynamic dispatch**:

**`impl Trait` in return position** uses static dispatch. The compiler needs to know the exact concrete type at compile time so it can allocate the right amount of stack space and inline the method calls. When two branches return different types (`Cat` vs `Dog`), the compiler cannot resolve this to a single type. `Cat` and `Dog` may have different sizes, different layouts, and different vtable entries.

**`Box<dyn Trait>`** uses dynamic dispatch. The value is heap-allocated behind a pointer, and method calls go through a vtable (a table of function pointers). The compiler does not need to know the concrete type at compile time — it just needs to know the trait. This has a small runtime cost (heap allocation + indirect function call), but it enables returning different types from the same function.

When to use which:

| Approach | Use When |
|---|---|
| `impl Trait` (return) | You always return the same concrete type |
| `Box<dyn Trait>` | You need to return different types depending on runtime conditions |
| Enum wrapping | You have a known, fixed set of types (often the best choice) |

An alternative correct solution using an enum:

```rust
enum Pet {
    Cat(Cat),
    Dog(Dog),
}

impl Describe for Pet {
    fn describe(&self) -> String {
        match self {
            Pet::Cat(c) => c.describe(),
            Pet::Dog(d) => d.describe(),
        }
    }
}

fn get_pet(likes_dogs: bool) -> Pet {
    if likes_dogs {
        Pet::Dog(Dog)
    } else {
        Pet::Cat(Cat)
    }
}
```

This avoids heap allocation entirely and keeps static dispatch, but requires knowing all variants at compile time.

## Compiler Error Interpretation

```
error[E0308]: `if` and `else` have incompatible types
  --> src/main.rs:22:9
   |
19 |       if likes_dogs {
   |  _____-
20 | |         Dog
   | |         --- expected because of this
21 | |     } else {
22 | |         Cat
   | |         ^^^ expected `Dog`, found `Cat`
23 | |     }
   | |_____- `if` and `else` have incompatible types
```

This error is deceptively simple but deeply meaningful:

- **"`if` and `else` have incompatible types"** — In Rust, both branches of an `if/else` expression must produce the same type. With `impl Trait` return, the compiler tries to unify both branches into a single concrete type.
- **"expected `Dog`, found `Cat`"** — The compiler saw `Dog` first, committed to that as the concrete type for `impl Describe`, and then found `Cat` in the other branch, which is a different type.
- The compiler does **not** say "both implement `Describe`" because that is irrelevant — `impl Trait` requires a single concrete type, not just trait compatibility.

This is a case where the compiler is protecting you from a fundamental ambiguity: stack-allocated values must have a known size at compile time, and the compiler cannot make the function's return slot simultaneously sized for both `Cat` and `Dog`.
