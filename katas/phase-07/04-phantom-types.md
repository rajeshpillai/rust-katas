---
id: phantom-types
phase: 7
phase_title: "Data Modeling the Rust Way"
sequence: 4
title: "Phantom Types: Compile-Time State Without Runtime Cost"
hints:
  - "A phantom type parameter appears in the struct definition but not in any field — it exists only at compile time."
  - "PhantomData<T> tells the compiler 'pretend this struct uses T' without actually storing anything."
  - "Use phantom types to make invalid state transitions impossible at compile time."
---

## Description

A **phantom type** is a type parameter that appears in a struct's generic signature but is not used in any field. It exists purely at compile time — it takes zero bytes of memory and has no runtime effect. Its purpose is to carry information in the type system that the compiler can check.

The most common use: encoding **state** in the type, so that invalid transitions are caught at compile time rather than runtime.

## Broken Code

```rust
struct Ticket {
    id: u32,
    title: String,
    is_open: bool,
    is_assigned: bool,
    assignee: Option<String>,
}

impl Ticket {
    fn new(id: u32, title: &str) -> Self {
        Ticket {
            id,
            title: title.to_string(),
            is_open: true,
            is_assigned: false,
            assignee: None,
        }
    }

    fn assign(&mut self, person: &str) {
        // Bug: nothing stops you from assigning a closed ticket.
        self.is_assigned = true;
        self.assignee = Some(person.to_string());
    }

    fn close(&mut self) {
        // Bug: nothing stops you from closing an unassigned ticket.
        self.is_open = false;
    }

    fn reopen(&mut self) {
        // Bug: nothing stops you from reopening an already-open ticket.
        self.is_open = true;
    }
}

fn main() {
    let mut ticket = Ticket::new(1, "Fix login bug");
    ticket.close();    // Closed without assignment — should be invalid!
    ticket.assign("Alice"); // Assigning a closed ticket — should be invalid!
    ticket.reopen();   // Is this valid? The booleans make it hard to reason about.
    ticket.reopen();   // Reopening an already-open ticket — silent no-op.
}
```

## Correct Code

```rust
use std::marker::PhantomData;

// State markers — zero-sized types (ZSTs) that exist only in the type system.
struct Open;
struct Assigned;
struct Closed;

// The phantom type parameter S encodes the ticket's current state.
// PhantomData<S> tells the compiler we "use" S without storing anything.
struct Ticket<S> {
    id: u32,
    title: String,
    assignee: Option<String>,
    _state: PhantomData<S>,
}

// Methods available only on Open tickets
impl Ticket<Open> {
    fn new(id: u32, title: &str) -> Ticket<Open> {
        Ticket {
            id,
            title: title.to_string(),
            assignee: None,
            _state: PhantomData,
        }
    }

    // assign() consumes Ticket<Open> and returns Ticket<Assigned>.
    // The state transition is encoded in the type signature.
    fn assign(self, person: &str) -> Ticket<Assigned> {
        println!("  [{}] Assigned to {}", self.id, person);
        Ticket {
            id: self.id,
            title: self.title,
            assignee: Some(person.to_string()),
            _state: PhantomData,
        }
    }
}

// Methods available only on Assigned tickets
impl Ticket<Assigned> {
    fn close(self) -> Ticket<Closed> {
        println!("  [{}] Closed (was assigned to {:?})", self.id, self.assignee);
        Ticket {
            id: self.id,
            title: self.title,
            assignee: self.assignee,
            _state: PhantomData,
        }
    }
}

// Methods available only on Closed tickets
impl Ticket<Closed> {
    fn reopen(self) -> Ticket<Open> {
        println!("  [{}] Reopened", self.id);
        Ticket {
            id: self.id,
            title: self.title,
            assignee: None,
            _state: PhantomData,
        }
    }
}

// Methods available on ALL tickets, regardless of state
impl<S> Ticket<S> {
    fn describe(&self) -> String {
        format!("Ticket #{}: {}", self.id, self.title)
    }
}

fn main() {
    // Valid workflow: Open -> Assigned -> Closed
    let ticket = Ticket::new(1, "Fix login bug");
    println!("{}", ticket.describe());

    let ticket = ticket.assign("Alice");   // Open -> Assigned
    let ticket = ticket.close();           // Assigned -> Closed
    println!("Final: {}\n", ticket.describe());

    // Valid workflow: Open -> Assigned -> Closed -> Reopened -> Assigned -> Closed
    let ticket = Ticket::new(2, "Update docs");
    println!("{}", ticket.describe());

    let ticket = ticket.assign("Bob");
    let ticket = ticket.close();
    let ticket = ticket.reopen();          // Closed -> Open
    let ticket = ticket.assign("Carol");   // Open -> Assigned
    let _ticket = ticket.close();          // Assigned -> Closed
    println!("Done!\n");

    // --- These would NOT compile: ---
    // let ticket = Ticket::new(3, "Bad workflow");
    // ticket.close();       // ERROR: Ticket<Open> has no method `close`
    // ticket.reopen();      // ERROR: Ticket<Open> has no method `reopen`
}
```

## Explanation

The broken version uses boolean flags (`is_open`, `is_assigned`) to track state. This means every transition must check these flags at runtime, and nothing prevents invalid transitions except programmer discipline. A closed ticket can be assigned. An open ticket can be reopened. The compiler cannot help.

The correct version uses **phantom types** to encode state in the type system:

1. **State markers** (`Open`, `Assigned`, `Closed`) are zero-sized types (ZSTs). They have no fields and take zero bytes of memory.

2. **`PhantomData<S>`** tells the compiler that `Ticket<S>` "uses" the type parameter `S`, even though no field actually stores a value of type `S`. Without `PhantomData`, the compiler would reject the unused type parameter.

3. **State-specific `impl` blocks** ensure methods are only available on tickets in the right state:
   - `assign()` only exists on `Ticket<Open>` — you cannot assign a closed ticket.
   - `close()` only exists on `Ticket<Assigned>` — you cannot close an unassigned ticket.
   - `reopen()` only exists on `Ticket<Closed>` — you cannot reopen an open ticket.

4. **Transitions consume the old state** by taking `self` (not `&self` or `&mut self`). This means the old `Ticket<Open>` is gone after `assign()` returns `Ticket<Assigned>`. You cannot accidentally use the ticket in its old state.

**Zero runtime cost:** `PhantomData<S>` is a ZST — it compiles to nothing. The `Ticket` struct in memory is identical regardless of the state parameter. All the state checking happens at compile time and is erased before code generation.

**This is the typestate pattern**: encoding states as types and transitions as functions that consume one type and produce another. Invalid state machines simply do not compile.

## ⚠️ Caution

- `PhantomData<T>` affects variance and drop checking. If your phantom type parameter represents a type you "own," use `PhantomData<T>`. If it represents a type you reference, use `PhantomData<&'a T>` or `PhantomData<fn() -> T>` to avoid unnecessary lifetime constraints.
- The typestate pattern increases the number of types and `impl` blocks. It is worth the complexity when invalid states cause real bugs, but overkill for simple two-state toggles.

## 💡 Tips

- Use the typestate pattern for workflows, protocols, and builders where invalid transitions are dangerous (network connections, file handles, authentication flows).
- Methods shared across all states go in `impl<S> Ticket<S>` — a generic impl block.
- Combine with the builder pattern: `Builder<NoHost>.host("...") -> Builder<HasHost>.build() -> Connection`.

## Compiler Error Interpretation

If you try to call `close()` on a `Ticket<Open>`:

```
error[E0599]: no method named `close` found for struct `Ticket<Open>` in the current scope
 --> src/main.rs:XX:XX
  |
X |     ticket.close();
  |            ^^^^^ method not found in `Ticket<Open>`
  |
  = note: the method was found for `Ticket<Assigned>`
```

The compiler tells you exactly what happened:
- **"no method named `close` found for struct `Ticket<Open>`"** — The method exists, but not for this state.
- **"the method was found for `Ticket<Assigned>`"** — The compiler even tells you which state has the method.

This is an invalid state transition caught at compile time. The ticket must be assigned before it can be closed. The type system enforces the workflow, and the error message guides you to the correct sequence.

---

| [Prev: Smart Constructors for Invariant Enforcement](#/katas/smart-constructors) | [Next: Defining Custom Error Types as Enums](#/katas/custom-error-enums) |
