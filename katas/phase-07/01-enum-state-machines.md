---
id: enum-state-machines
phase: 7
phase_title: "Data Modeling the Rust Way"
sequence: 1
title: Enum-Driven State Machines
hints:
  - "Using a `String` field for state means any string value is valid — the compiler cannot check state transitions."
  - "Enums let you define exactly which states are valid, and `match` forces you to handle every one."
  - "Each enum variant can carry its own data — data that only exists in that state."
---

## Description

One of Rust's most powerful modeling techniques is using enums to represent state machines. Instead of a struct with a "status" field (like a string or integer), you define an enum where each variant represents a distinct state, carrying only the data relevant to that state.

This makes **illegal states unrepresentable**. The compiler ensures you handle every state transition, and data that only exists in certain states cannot be accessed in other states.

## Broken Code

```rust
struct Order {
    id: u64,
    status: String,
    items: Vec<String>,
    tracking_number: Option<String>,
}

impl Order {
    fn new(id: u64, items: Vec<String>) -> Self {
        Order {
            id,
            status: String::from("pending"),
            items,
            tracking_number: None,
        }
    }

    fn ship(&mut self, tracking: String) {
        // BUG: Nothing prevents calling ship() on an already-delivered order.
        // Nothing prevents setting status to "shippd" (typo).
        // The tracking_number exists even when the order is pending.
        self.status = String::from("shipped");
        self.tracking_number = Some(tracking);
    }

    fn deliver(&mut self) {
        self.status = String::from("delivered");
    }

    fn tracking_info(&self) -> String {
        // BUG: We have to check the status string at runtime.
        // A typo in the status string would silently break this.
        if self.status == "shipped" {
            format!(
                "Order {} tracking: {}",
                self.id,
                self.tracking_number.as_ref().unwrap()
            )
        } else {
            format!("Order {} has no tracking info", self.id)
        }
    }
}

fn main() {
    let mut order = Order::new(1, vec!["Book".into(), "Pen".into()]);
    order.ship(String::from("TRACK-123"));
    println!("{}", order.tracking_info());

    // BUG: We can ship a delivered order, set nonsense statuses, etc.
    order.deliver();
    order.ship(String::from("TRACK-456")); // Should not be possible!
}
```

## Correct Code

```rust
struct OrderId(u64);

enum Order {
    Pending {
        id: OrderId,
        items: Vec<String>,
    },
    Shipped {
        id: OrderId,
        items: Vec<String>,
        tracking_number: String,
    },
    Delivered {
        id: OrderId,
        items: Vec<String>,
        tracking_number: String,
    },
}

impl Order {
    fn new(id: u64, items: Vec<String>) -> Self {
        Order::Pending {
            id: OrderId(id),
            items,
        }
    }

    // ship() consumes the current state and returns a new state.
    // It can only be called on a Pending order.
    fn ship(self, tracking: String) -> Result<Self, Self> {
        match self {
            Order::Pending { id, items } => Ok(Order::Shipped {
                id,
                items,
                tracking_number: tracking,
            }),
            // If the order is not Pending, return it unchanged as an error.
            other => Err(other),
        }
    }

    fn deliver(self) -> Result<Self, Self> {
        match self {
            Order::Shipped {
                id,
                items,
                tracking_number,
            } => Ok(Order::Delivered {
                id,
                items,
                tracking_number,
            }),
            other => Err(other),
        }
    }

    fn tracking_info(&self) -> String {
        match self {
            // tracking_number only exists in Shipped and Delivered variants.
            // No Option, no unwrap, no runtime check needed.
            Order::Shipped {
                id,
                tracking_number,
                ..
            } => {
                format!("Order {} tracking: {}", id.0, tracking_number)
            }
            Order::Delivered {
                id,
                tracking_number,
                ..
            } => {
                format!(
                    "Order {} delivered (was tracking: {})",
                    id.0, tracking_number
                )
            }
            Order::Pending { id, .. } => {
                format!("Order {} has no tracking info (pending)", id.0)
            }
        }
    }
}

fn main() {
    let order = Order::new(1, vec!["Book".into(), "Pen".into()]);

    // Transitions are explicit and type-safe.
    let order = order.ship(String::from("TRACK-123")).unwrap();
    println!("{}", order.tracking_info());

    let order = order.deliver().unwrap();
    println!("{}", order.tracking_info());

    // This would return Err — you cannot ship a delivered order.
    let result = order.ship(String::from("TRACK-456"));
    match result {
        Ok(_) => println!("Shipped again (should not happen)"),
        Err(_) => println!("Cannot ship a delivered order — correct!"),
    }
}
```

## Explanation

The broken version uses a `String` field to represent state. This has several problems:

1. **Any string is valid.** You can set `status` to `"shippd"`, `"SHIPPED"`, `""`, or `"banana"`. The compiler has no idea which strings are meaningful.

2. **Data exists in wrong states.** `tracking_number` is `Option<String>` on every order, even pending ones. You must constantly check whether it is `Some` or `None`, and the compiler cannot help you know when it should be `Some`.

3. **Invalid transitions are not prevented.** Nothing stops you from calling `ship()` on a delivered order, or `deliver()` on a pending order. These are logical bugs that the type system could catch but does not.

4. **Runtime string comparisons are fragile.** The `tracking_info` method compares `self.status == "shipped"` — a typo in either the setter or the getter silently breaks the logic.

The correct version uses an enum where:

- **Each state is a distinct variant** with its own data. `tracking_number` only exists in `Shipped` and `Delivered` — it is structurally impossible to access it on a `Pending` order.

- **State transitions consume the current state** and produce a new state. The `ship(self)` method takes ownership of the order, so the old state cannot be used after the transition.

- **Invalid transitions are handled explicitly.** If you try to ship a delivered order, the `match` arm returns `Err(other)`, giving the caller back the unchanged order.

- **`match` is exhaustive.** If you add a new variant (say `Cancelled`), the compiler forces you to handle it everywhere. No variant is accidentally ignored.

**This is what "making illegal states unrepresentable" means.** The type system itself prevents the bugs. You do not need unit tests to verify that shipped orders have tracking numbers — the structure of the code guarantees it.

## Compiler Error Interpretation

The broken version compiles without errors — that is the problem. The bugs are **logical**, not syntactic. The compiler cannot help you because the types are too permissive (`String` accepts anything).

The correct version turns logical errors into type errors. For example, if you tried to access `tracking_number` on a `Pending` variant:

```
error[E0026]: variant `Order::Pending` does not have a field named `tracking_number`
 --> src/main.rs:XX:XX
  |
  |             Order::Pending { tracking_number, .. } => {
  |                              ^^^^^^^^^^^^^^^ variant `Order::Pending` does not have this field
```

This error is the **entire point** of the refactoring. The compiler now knows that `Pending` orders do not have tracking numbers, and it prevents you from pretending they do. The invariant is encoded in the type, not in comments or documentation.

When the compiler says "variant does not have this field," it is enforcing your domain model. This is Rust's data modeling philosophy: **let the compiler enforce your business rules**.
