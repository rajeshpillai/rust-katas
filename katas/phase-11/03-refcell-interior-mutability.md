---
id: refcell-interior-mutability
phase: 11
phase_title: "Memory & Performance Intuition"
sequence: 3
title: "RefCell and Interior Mutability"
hints:
  - Rc<T> only gives you shared (immutable) access to T
  - To mutate data behind an Rc, you need interior mutability
  - RefCell<T> moves Rust's borrow checking from compile time to runtime
  - The pattern Rc<RefCell<T>> gives you shared ownership with interior mutability
  - RefCell will panic at runtime if you violate borrow rules (e.g., two mutable borrows at once)
---

## Description

Rust's borrow checker normally operates at compile time: you cannot have a mutable reference while shared references exist. But sometimes you need to mutate data that is shared between multiple owners. `Rc<T>` gives shared ownership but only immutable access. Enter `RefCell<T>`, which provides **interior mutability** -- the ability to mutate data even when the outer reference is immutable.

`RefCell<T>` enforces borrowing rules at **runtime** instead of compile time. You call `.borrow()` for shared access and `.borrow_mut()` for exclusive access. If you violate the rules (e.g., calling `.borrow_mut()` while a `.borrow()` is still active), the program **panics** instead of failing to compile.

## Broken Code

```rust
use std::rc::Rc;

#[derive(Debug)]
struct Account {
    name: String,
    balance: i64,
}

impl Account {
    fn new(name: &str, balance: i64) -> Self {
        Account {
            name: String::from(name),
            balance,
        }
    }

    fn deposit(&mut self, amount: i64) {
        self.balance += amount;
    }
}

fn main() {
    // Two services share the same account
    let shared_account = Rc::new(Account::new("Alice", 100));

    let bank_ref = Rc::clone(&shared_account);
    let audit_ref = Rc::clone(&shared_account);

    // Try to deposit through the shared reference
    // Rc only gives &T, not &mut T!
    bank_ref.deposit(50);

    println!("Audit sees: {:?}", audit_ref);
}
```

## Correct Code

```rust
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
struct Account {
    name: String,
    balance: i64,
}

impl Account {
    fn new(name: &str, balance: i64) -> Self {
        Account {
            name: String::from(name),
            balance,
        }
    }

    fn deposit(&mut self, amount: i64) {
        self.balance += amount;
    }
}

fn main() {
    // Wrap in Rc<RefCell<T>> for shared ownership + interior mutability
    let shared_account = Rc::new(RefCell::new(Account::new("Alice", 100)));

    let bank_ref = Rc::clone(&shared_account);
    let audit_ref = Rc::clone(&shared_account);

    // borrow_mut() returns a RefMut<Account>, which acts like &mut Account
    bank_ref.borrow_mut().deposit(50);

    // borrow() returns a Ref<Account>, which acts like &Account
    println!("Audit sees: {:?}", audit_ref.borrow());
    println!("Balance: {}", audit_ref.borrow().balance);

    // DANGER: This would panic at runtime!
    // let mut_ref = shared_account.borrow_mut();
    // let immut_ref = shared_account.borrow(); // PANIC: already mutably borrowed
}
```

## Explanation

The broken version wraps `Account` in `Rc<Account>`. `Rc` provides shared ownership, but it only gives immutable access to the inner value. Calling `bank_ref.deposit(50)` requires `&mut self`, but dereferencing an `Rc<Account>` only yields `&Account`. The compiler rejects the mutable method call on an immutable reference.

The correct version uses `Rc<RefCell<Account>>`. This layered structure provides:

- **`Rc`** -- shared ownership (multiple owners can hold references)
- **`RefCell`** -- interior mutability (runtime-checked mutable access)

To mutate: call `.borrow_mut()`, which returns a `RefMut<Account>` guard. This guard implements `DerefMut`, so you can call `&mut self` methods on it. When the guard is dropped (goes out of scope), the mutable borrow is released.

To read: call `.borrow()`, which returns a `Ref<Account>` guard. This guard implements `Deref`, so you can read fields and call `&self` methods.

**The critical tradeoff: `RefCell` can panic.** The borrow rules are still enforced, just at runtime:

| Operation | Compile-time `&` / `&mut` | Runtime `RefCell` |
|-----------|---------------------------|-------------------|
| Multiple shared borrows | Allowed | Allowed |
| One mutable borrow, no shared | Allowed | Allowed |
| Mutable + shared simultaneously | Compile error | **Runtime panic** |
| Two mutable borrows simultaneously | Compile error | **Runtime panic** |

This means bugs that would be caught at compile time with regular references become runtime panics with `RefCell`. You trade compile-time safety for flexibility. This is why `RefCell` should be used judiciously -- it is a sharp tool for situations where the borrow checker is too conservative, not a way to bypass safety.

Common patterns with `RefCell`:

```rust
// Observer pattern: multiple observers share mutable state
let state = Rc::new(RefCell::new(Vec::new()));

// Graph structures: nodes that need to mutate neighbors
let node = Rc::new(RefCell::new(GraphNode::new()));

// Caching: a struct with a &self API that caches internally
struct Fibonacci {
    cache: RefCell<HashMap<u64, u64>>,
}
```

When `RefCell` feels wrong, consider whether your design can be restructured to use normal borrowing. `RefCell` is a sign that your ownership model is more complex than Rust's default -- sometimes that is necessary, sometimes it signals a design issue.

## Compiler Error Interpretation

```
error[E0596]: cannot borrow data in an `Rc` as mutable
  --> src/main.rs:30:5
   |
30 |     bank_ref.deposit(50);
   |     ^^^^^^^^ cannot borrow as mutable
   |
   = help: trait `DerefMut` is required to modify through a dereference,
           but it is not implemented for `Rc<Account>`
```

The compiler explains that `Rc<Account>` does not implement `DerefMut`, which means you cannot get a `&mut Account` from it. `Rc` intentionally omits `DerefMut` because shared ownership and unrestricted mutation would violate memory safety. The solution is to introduce `RefCell` between the `Rc` and the data: `Rc<RefCell<Account>>`. This moves the borrow check to runtime, where `RefCell` can dynamically ensure that mutable access is exclusive.
