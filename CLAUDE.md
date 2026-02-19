
---

## Learning Sequence (MANDATORY ORDER)

You must follow this order **strictly**, even if it contradicts popular tutorials.

---

## PHASE 0 — Rust as a Language

**Goal:** Comfort with syntax, types, and values

Teach:
- Variables, mutability
- Shadowing vs mutation
- Primitive types and type inference
- Structs: defining, instantiating, tuple structs
- Enums and pattern matching
- `Option` and `Result` (early)
- `String` vs `&str` (awareness, not mastery)

Key insight:
> Rust forces you to acknowledge absence and failure explicitly.

---

## PHASE 1 — Ownership (Core Concept)

**Goal:** Understand moves before sharing

Teach:
- Move vs Copy
- Ownership transfer
- Returning ownership
- Clone as an explicit cost

Rule:
- Do NOT introduce borrowing yet

---

## PHASE 2 — Borrowing (Shared Access)

**Goal:** Share data safely

Teach:
- Immutable borrows
- Mutable borrows
- Borrow scopes
- Why aliasing + mutation is forbidden

Key insight:
> Rust encodes time in types.

---

## PHASE 3 — Lifetimes (Relationships, Not Syntax)

**Goal:** Lifetimes as constraints

Teach:
- Returning references
- Structs holding references
- Lifetime elision rules
- When the compiler can infer

Important:
- Lifetimes are not about "how long"
- They are about "what relates to what"

---

## PHASE 4 — Traits & Generics

**Goal:** Polymorphism through contracts

Teach:
- Defining and implementing traits
- Generics and trait bounds (`T: Display`)
- `impl Trait` in argument and return position
- Associated types vs generic parameters
- Derived traits (`Debug`, `Clone`, `PartialEq`)
- Operator overloading via traits (`Add`, `Index`)
- Dynamic dispatch (`dyn Trait`) vs static dispatch (monomorphization)
- Object safety rules

Key insight:
> Traits are Rust's answer to "what can this type do?" — not "what is this type?"

Rule:
- Teach trait objects only after generics are comfortable

---

## PHASE 5 — Closures & the Fn Traits

**Goal:** Functions as values, governed by ownership

Teach:
- Closure syntax and capture semantics
- `Fn`, `FnMut`, `FnOnce` — what each requires
- Move closures (`move ||`)
- Why closures and ownership are inseparable
- Closures as trait objects vs generic parameters

Key insight:
> A closure's type is determined by how it uses what it captures.

Rule:
- Closures must come before iterators — iterators depend on them

---

## PHASE 6 — Collections & the Owned/Borrowed Duality

**Goal:** Practical data structures, ownership reinforced

Teach:
- `Vec<T>` and `&[T]` (slices)
- `String` and `&str` (deep dive)
- `HashMap<K, V>` and `BTreeMap<K, V>`
- `HashSet<T>` and `BTreeSet<T>`
- The owned/borrowed pattern: `PathBuf`/`&Path`, `OsString`/`&OsStr`
- Entry API for maps
- Iterating over collections and ownership implications

Key insight:
> Every major Rust type has an owned form and a borrowed form. Understanding this duality is understanding Rust.

---

## PHASE 7 — Data Modeling the Rust Way

**Goal:** Make illegal states impossible

Teach:
- Enum-driven state machines
- Newtype pattern
- Smart constructors
- Intro to phantom types

---

## PHASE 8 — Error Handling as Design

**Goal:** Errors are values

Teach:
- Custom error enums
- `Result<T, E>` as API contracts
- `?` operator mechanics
- Recoverable vs unrecoverable errors
- `From` and `Into` for error conversion
- When to `panic!` vs when to return `Err`

---

## PHASE 9 — Iterators & Zero-Cost Abstractions

**Goal:** Expressive and fast code

Teach:
- Iterator trait and `IntoIterator`
- Lazy evaluation
- `map`, `filter`, `fold`, `collect`
- Chaining and composing iterators
- Writing custom iterators
- Why iterators beat loops

---

## PHASE 10 — Modules, Visibility & Testing

**Goal:** Organize code and prove it works

Teach:
- `mod`, `use`, `pub`, `pub(crate)`
- File and directory module structure
- Crate vs module vs package
- `#[test]` and `#[cfg(test)]`
- Integration tests (`tests/` directory)
- Doc tests
- Test-driven kata workflow

Key insight:
> Visibility is Rust's way of enforcing API boundaries at compile time.

Rule:
- Katas are inherently test-driven — make testing explicit, not assumed

---

## PHASE 11 — Memory & Performance Intuition

**Goal:** Understand layout and cost

Teach:
- Stack vs heap
- `Box`, `Rc`, `Arc`
- `Deref`, `DerefMut`, and the `Drop` trait
- Reference counting tradeoffs
- Interior mutability (`Cell`, `RefCell`)

Explain:
- Why `RefCell` can panic
- Compile-time vs runtime checks

---

## PHASE 12 — Macros

**Goal:** Metaprogramming with discipline

Teach:
- `macro_rules!` — declarative macros
- Pattern matching in macros
- Repetition (`$(...)*`)
- When macros are appropriate vs when traits suffice
- Procedural macros (awareness): derive, attribute, function-like

Rule:
- Macros are a last resort, not a first tool
- Teach only after learners have seen enough patterns to appreciate the abstraction

Key insight:
> A macro should make correct code easier to write, not incorrect code easier to hide.

---

## PHASE 13 — Concurrency the Rust Way

**Goal:** Fearless concurrency

Teach:
- Thread ownership transfer
- Message passing with channels
- Shared state with `Arc<Mutex<T>>`
- Why data races don't compile

---

## PHASE 14 — Async Rust (Carefully)

**Goal:** Async as state machines

Teach:
- Futures as state machines
- `async/await` desugaring
- Executors
- Common async pitfalls

Rule:
- Async only after ownership mastery

---

## PHASE 15 — Unsafe Rust (Controlled Power)

**Goal:** Unsafe enables safe abstractions

Teach:
- Why unsafe exists
- Raw pointers
- Writing safe wrappers
- `Send` and `Sync` intuition

Golden rule:
> Unsafe code must create safe boundaries.

---

## PHASE 16 — Advanced Systems Patterns

**Goal:** Rust as a systems language

Teach:
- Memory pools
- Lock-free intro
- FFI boundaries
- Allocators (conceptual)

These are capstone-level katas.

---

# WEBASSEMBLY EXTENSION (AFTER CORE RUST)

---

## PHASE 17 — What is WebAssembly Really?

**Goal:** Demystify WASM

Teach:
- WASM as a virtual machine
- Stack-based execution
- Linear memory
- Deterministic sandboxing

Key insight:
> WASM is closer to a process than a library.

---

## PHASE 18 — Rust → WASM Toolchain

**Goal:** Understand compilation pipeline

Teach:
- `rustc` → LLVM → WASM
- `wasm32-unknown-unknown`
- What `wasm-bindgen` actually does
- Why WASM has no GC

---

## PHASE 19 — WASM Memory Model

**Goal:** Make memory explicit again

Teach:
- Linear memory
- Offsets and pointers
- Passing primitives vs complex data
- Manual allocation and deallocation

Key insight:
> Ownership does not cross the WASM boundary automatically.

---

## PHASE 20 — Host ↔ Guest Contracts

**Goal:** Boundaries are everything

Teach:
- ABI constraints
- Imports and exports
- Error handling across boundary
- Designing stable interfaces

Key insight:
> WASM APIs are protocols, not functions.

---

## PHASE 21 — WASM in the Browser (Correctly)

**Goal:** WASM as a compute engine

Teach:
- Why WASM should not manage DOM
- Hot paths vs glue code
- JS vs WASM performance tradeoffs

---

## PHASE 22 — Rust Ownership Patterns for WASM

**Goal:** Apply Rust thinking across boundary

Teach:
- Allocation minimization
- Boundary crossing costs
- Zero-copy patterns (when possible)
- Stable WASM APIs

---

## PHASE 23 — WASM Outside the Browser

**Goal:** WASM as a universal runtime

Teach:
- WASM in Node.js
- WASI
- Plugin systems
- Sandboxed execution

---

## PHASE 24 — Advanced WASM Systems Patterns

**Goal:** Professional mastery

Teach:
- Capability-based security
- WASM runtimes (Wasmtime, Wasmer)
- Performance profiling
- WASM vs containers

---

## PHASE 25 — Capstone Projects (Rust + WASM)

**Goal:** Integration mastery

Capstones may include:
- WASM-powered plugin system
- Sandboxed user script execution
- Rust → WASM image processor
- Secure extension runtime

Each capstone must:
- Respect memory boundaries
- Use unsafe responsibly
- Expose safe APIs
- Be debuggable

---

## Kata Design Rules (MANDATORY)

Each kata must include:
- ❌ a broken version
- ✅ a correct version
- 🧠 explanation of violated invariant
- 🔍 compiler error interpretation

---

## Teaching Rules (VERY IMPORTANT)

You must:
- Explain *why* something fails
- Treat compiler messages as allies
- Build intuition before optimization
- Emphasize invariants and boundaries

You must NOT:
- Hide compiler errors
- Jump to advanced topics early
- Treat WASM as a frontend trick
- Encourage unsafe without justification

---

## Success Criteria

This system is successful if learners can:
- Predict compiler errors
- Design safe Rust APIs
- Use traits and generics to write polymorphic code
- Write and run tests as part of every kata
- Understand memory boundaries
- Use WASM beyond "web demos"
- Think like systems engineers

---

## Final Instruction

Teach Rust as a **discipline of correctness**, and WASM as a **boundary-aware runtime**.

Coding Conventions:

- All file and folder names should be lowercase-hyphenated
- The  Code and Preview/output sectoin should be resizable and should have maximize button
- The sidebar should have kata sequence number and title and should be collpasible (burge menu)
- Add Light/Dark toggle theme option

Landing Page:

- The landing page must display two cards:
  1. **Katas** — The structured learning sequence (Phases 0–25) described in this document. Links into the kata browser/sidebar.  All katas to be documented as markdown files.
  2. **Applications** — Real-world Rust + WASM projects. Content to be planned after kata completion. Show as a "Coming Soon" card until then.


When in doubt:
- Choose invariants over convenience
- Choose compiler guidance over intuition
- Choose safety before power

Proceed deliberately.
Explain everything.
Never assume.
