---
id: procedural-macros
phase: 12
phase_title: "Macros"
sequence: 4
title: "Procedural Macros: Derive, Attribute, and Function-Like"
hints:
  - "Declarative macros (`macro_rules!`) match patterns on token trees. Procedural macros operate on the AST as Rust code that generates Rust code."
  - "`#[derive(Debug, Clone)]` invokes procedural derive macros — the compiler runs Rust code that generates the trait impl."
  - "Procedural macros must live in a separate crate with `proc-macro = true` in Cargo.toml."
---

## Description

Rust has two macro systems:

1. **Declarative macros** (`macro_rules!`) — pattern matching on token trees. You have learned these.
2. **Procedural macros** — Rust functions that take a token stream as input and produce a token stream as output. They are actual Rust code that runs at compile time.

You use procedural macros every day: `#[derive(Debug)]`, `#[derive(Clone)]`, `#[derive(serde::Serialize)]`. This kata builds awareness of how they work, when to use them, and why they require a separate crate.

This is an **awareness kata** — you will not write a procedural macro from scratch, but you will understand what they do, how to recognize them, and when to reach for them vs declarative macros.

## Broken Code

```rust
// Attempt to create a "derive-like" macro using macro_rules!
// This quickly becomes unwieldy and fragile.

macro_rules! impl_describe {
    ($name:ident { $($field:ident),* }) => {
        impl $name {
            fn describe(&self) -> String {
                let mut parts = vec![format!("{}:", stringify!($name))];
                $(
                    parts.push(format!("  {} = {:?}", stringify!($field), self.$field));
                )*
                parts.join("\n")
            }
        }
    };
}

struct User {
    name: String,
    age: u32,
    email: String,
}

// We have to repeat the struct's field names in the macro invocation.
// If we add a field to the struct, we must update the macro call too.
// There is no way for macro_rules! to "see" the struct definition.
impl_describe!(User { name, age, email });

struct Product {
    title: String,
    price: f64,
}

impl_describe!(Product { title, price });

fn main() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };
    println!("{}", user.describe());

    let product = Product {
        title: "Widget".to_string(),
        price: 9.99,
    };
    println!("\n{}", product.describe());
}
```

## Correct Code

```rust
// In real Rust, you would use a derive macro from a proc-macro crate.
// This kata demonstrates the CONCEPT using hand-written impls that
// mirror what a derive macro would generate.
//
// A real derive macro like #[derive(Describe)] would:
// 1. Parse the struct definition (field names and types)
// 2. Generate the impl block automatically
// 3. Stay in sync when fields are added or removed

// --- What a derive macro generates (conceptually) ---

#[derive(Debug)]  // This IS a procedural derive macro from std!
struct User {
    name: String,
    age: u32,
    email: String,
}

// This is what #[derive(Debug)] generates behind the scenes:
// impl fmt::Debug for User {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("User")
//             .field("name", &self.name)
//             .field("age", &self.age)
//             .field("email", &self.email)
//             .finish()
//     }
// }

// --- The three kinds of procedural macros ---

// 1. DERIVE MACROS — #[derive(TraitName)]
//    Generate trait implementations from struct/enum definitions.
//    Most common. Examples: Debug, Clone, Serialize, PartialEq.
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

// 2. ATTRIBUTE MACROS — #[macro_name]
//    Transform the annotated item. Examples: #[test], #[tokio::main].
//    (We cannot demonstrate custom ones without a proc-macro crate,
//     but #[test] is one you already use.)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]  // <-- This is an attribute procedural macro!
    fn test_point_equality() {
        let a = Point { x: 1.0, y: 2.0 };
        let b = a.clone();  // clone() from #[derive(Clone)]
        assert_eq!(a, b);   // PartialEq from #[derive(PartialEq)]
    }
}

// 3. FUNCTION-LIKE MACROS — macro_name!(...)
//    Look like macro_rules! but can do arbitrary computation.
//    Example: sqlx::query!("SELECT * FROM users WHERE id = $1", id)
//    The macro parses the SQL at compile time and verifies it.

fn main() {
    // derive(Debug) gives us {:?} formatting
    let p = Point { x: 3.0, y: 4.0 };
    println!("Debug:  {:?}", p);
    println!("Pretty: {:#?}", p);

    // derive(Clone) gives us .clone()
    let p2 = p.clone();

    // derive(PartialEq) gives us == and !=
    println!("Equal:  {}", p == p2);
    println!("Not equal: {}", p != Point { x: 0.0, y: 0.0 });

    // --- Common derive macros you will encounter ---
    println!("\n--- Derive macro reference ---");
    println!("std:   Debug, Clone, Copy, PartialEq, Eq, Hash, Default");
    println!("std:   PartialOrd, Ord");
    println!("serde: Serialize, Deserialize");
    println!("thiserror: Error");
    println!("clap:  Parser, Subcommand");

    // --- When to use which macro system ---
    println!("\n--- Decision guide ---");
    println!("Pattern matching on tokens? -> macro_rules!");
    println!("Generate trait impls from struct? -> derive macro");
    println!("Transform an entire function/struct? -> attribute macro");
    println!("Parse custom syntax at compile time? -> function-like proc macro");
    println!("Simple repetition? -> macro_rules! (simpler, no extra crate)");
}
```

## Explanation

The broken version tries to use `macro_rules!` to auto-generate a `describe()` method for structs. The fundamental limitation: **`macro_rules!` cannot inspect a struct's definition**. You must manually repeat the field names in the macro invocation. If you add a field and forget to update the macro call, the output is silently wrong.

**Procedural macros solve this** because they receive the full token stream of the annotated item. A derive macro on a struct receives the struct's name, fields, types, attributes — everything. It can generate code that automatically includes all fields.

**The three kinds:**

**1. Derive macros (`#[derive(TraitName)]`):**
- Input: the struct or enum definition
- Output: a trait implementation
- Most common. You use `#[derive(Debug, Clone, PartialEq)]` constantly
- Each derive macro is a Rust function: `fn derive_debug(input: TokenStream) -> TokenStream`

**2. Attribute macros (`#[macro_name]`):**
- Input: the annotated item + any arguments
- Output: a transformed or replaced item
- Examples: `#[test]`, `#[tokio::main]`, `#[wasm_bindgen]`
- More powerful than derive — can modify or replace the entire item

**3. Function-like macros (`macro_name!(...)`):**
- Input: arbitrary tokens inside the parentheses
- Output: arbitrary tokens
- Example: `sqlx::query!("SELECT ...")` parses SQL at compile time
- Like `macro_rules!` but can do arbitrary computation

**Why a separate crate?**

Procedural macros must be compiled and run during compilation of the crate that uses them. This creates a bootstrapping problem — the macro code must be compiled before the code that invokes it. Rust solves this by requiring proc macros in a separate crate with `proc-macro = true` in `Cargo.toml`.

```toml
# my-derive/Cargo.toml
[lib]
proc-macro = true

[dependencies]
syn = "2"       # Parse Rust token streams
quote = "1"     # Generate Rust token streams
proc-macro2 = "1"
```

**Decision guide:**

| Need | Use |
|---|---|
| Simple pattern-based code generation | `macro_rules!` |
| Derive a trait for a struct/enum | Derive proc macro |
| Transform a function or module | Attribute proc macro |
| Custom syntax that is verified at compile time | Function-like proc macro |
| Anything where `macro_rules!` becomes unwieldy | Consider a proc macro |

## ⚠️ Caution

- Procedural macros increase compile times because they run Rust code at compile time. The `syn` crate (used to parse token streams) is a significant dependency.
- Proc macro errors can be cryptic. Use `cargo expand` (install with `cargo install cargo-expand`) to see the generated code.
- Do not reach for procedural macros prematurely. If `macro_rules!` or a trait with default methods can solve the problem, prefer those — they are simpler and do not require a separate crate.

## 💡 Tips

- Use `cargo expand` to see what any derive macro generates — this demystifies `#[derive(Debug)]` and friends.
- The `syn` and `quote` crates are the standard tools for writing proc macros. `syn` parses token streams into an AST; `quote!` generates token streams from template code.
- When choosing between `macro_rules!` and a proc macro, ask: "Do I need to inspect the structure of the input (field names, types)?" If yes, you need a proc macro. If you just need pattern-based substitution, `macro_rules!` is simpler.

## Compiler Error Interpretation

When a derive macro generates invalid code, the error points to the derive attribute:

```
error[E0277]: `MyStruct` doesn't implement `std::fmt::Display`
 --> src/main.rs:1:10
  |
1 | #[derive(Display)]
  |          ^^^^^^^ `MyStruct` doesn't implement `std::fmt::Display`
```

If you see an error on a `#[derive(...)]` line, it usually means:
- The derive macro is not in scope (missing `use` or dependency)
- The derive macro generated code that references a trait not implemented by a field type
- You used `#[derive(Display)]` but `Display` is not a derivable trait in `std` — you need a crate like `derive_more`

Use `cargo expand` to see exactly what code the macro generated, then debug the generated code as if you wrote it by hand.

---

| [Prev: When to Use Macros vs Generics](#/katas/when-to-use-macros) | [Next: Thread Ownership Transfer](#/katas/thread-ownership) |
