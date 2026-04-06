---
id: arc-atomic-reference-counting
phase: 11
phase_title: "Memory & Performance Intuition"
sequence: 5
title: "Arc: Shared Ownership Across Threads"
hints:
  - "Rc<T> uses non-atomic reference counting — it is fast but not thread-safe."
  - "Arc<T> uses atomic operations for its reference counter, making it safe to share across threads."
  - "Arc provides shared immutable access. For shared mutable access across threads, combine Arc with Mutex: Arc<Mutex<T>>."
---

## Description

`Rc<T>` provides shared ownership within a single thread. But what happens when multiple threads need to share the same data? `Rc` is explicitly `!Send` — the compiler will refuse to let you move it to another thread.

`Arc<T>` (Atomic Reference Counted) is the thread-safe counterpart. It has the same API as `Rc`, but its reference counter uses **atomic operations** — CPU instructions that are safe even when multiple threads increment or decrement simultaneously. The tradeoff is a small performance cost per clone and drop.

## Broken Code

```rust
use std::rc::Rc;
use std::thread;

fn main() {
    let data = Rc::new(vec![1, 2, 3, 4, 5]);

    let mut handles = vec![];

    for i in 0..3 {
        let data_clone = Rc::clone(&data);
        // This fails: Rc cannot be sent between threads.
        let handle = thread::spawn(move || {
            let sum: i32 = data_clone.iter().sum();
            println!("Thread {}: sum = {}", i, sum);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## Correct Code

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    // --- Arc: shared immutable access across threads ---
    let data = Arc::new(vec![1, 2, 3, 4, 5]);

    let mut handles = vec![];

    for i in 0..3 {
        // Arc::clone increments the atomic reference count.
        // Each thread gets its own Arc pointer to the same heap data.
        let data_clone = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let sum: i32 = data_clone.iter().sum();
            println!("Thread {}: sum = {}", i, sum);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // `data` is still valid — we cloned the Arc, not moved it.
    println!("Main thread still has data: {:?}", data);
    println!("Reference count: {}", Arc::strong_count(&data));

    // --- Arc + Mutex: shared mutable access across threads ---
    use std::sync::Mutex;

    let counter = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    for _ in 0..5 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            // .lock() acquires the mutex, returning a MutexGuard.
            // The guard implements DerefMut, so we can modify the value.
            // The lock is released when the guard is dropped.
            let mut num = counter_clone.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nFinal counter: {}", *counter.lock().unwrap());
}
```

## Explanation

The broken version uses `Rc` and passes it to `thread::spawn`. This fails because `Rc` is not `Send` — the compiler prevents it from crossing thread boundaries.

**Why `Rc` is not thread-safe:**

`Rc`'s reference counter uses normal (non-atomic) integer operations:

```
// Simplified Rc::clone
counter += 1;  // NOT atomic — two threads can read the same value,
               // both add 1, and write back, losing an increment.
```

If two threads clone an `Rc` simultaneously, the counter could be incremented only once instead of twice. Later, when drops decrement the counter, it would reach zero too early, freeing the data while a thread still holds a reference. This is a use-after-free bug.

**How `Arc` fixes this:**

`Arc` uses atomic CPU instructions for the counter:

```
// Simplified Arc::clone
atomic_fetch_add(&counter, 1);  // Atomic — guaranteed correct even with
                                 // concurrent access from multiple threads.
```

Atomic operations are slightly more expensive than normal arithmetic (they require memory barriers and cache coordination), but they guarantee correctness under concurrent access.

**The `Send` and `Sync` connection:**

- `Rc<T>` is `!Send` — cannot be moved to another thread.
- `Arc<T>` is `Send` (if `T: Send + Sync`) — can be shared across threads.
- The compiler checks these traits at compile time, preventing data races before your code runs.

**When to use each:**

| Type | Thread-safe | Performance | Use when |
|---|---|---|---|
| `Rc<T>` | No | Faster | Single-threaded shared ownership |
| `Arc<T>` | Yes | Slightly slower | Multi-threaded shared ownership |
| `Arc<Mutex<T>>` | Yes | Slowest | Multi-threaded shared + mutable |

**The `Arc<Mutex<T>>` pattern:**

`Arc` provides shared access but only immutably (just like `Rc`). To mutate shared data across threads, wrap the data in a `Mutex`:

- `Arc` handles the "multiple owners across threads" part.
- `Mutex` handles the "only one can mutate at a time" part.

`lock()` returns a `MutexGuard` that implements `DerefMut`. The guard holds the lock until it is dropped (usually at the end of the scope).

## ⚠️ Caution

- Do not use `Arc` when `Rc` suffices. The atomic operations are unnecessary overhead in single-threaded code.
- `Mutex::lock()` returns `Result` because a mutex can be "poisoned" if a thread panics while holding the lock. In most cases, `.unwrap()` is acceptable — a poisoned mutex indicates a bug elsewhere.
- Holding a `MutexGuard` across an `.await` point in async code will block the entire thread. Use `tokio::sync::Mutex` for async contexts.

## 💡 Tips

- Clone the `Arc` before moving it into the thread: `let clone = Arc::clone(&data);` then `move || { use clone }`.
- Use `Arc::strong_count()` for debugging to verify reference counts match expectations.
- For read-heavy workloads, consider `Arc<RwLock<T>>` — it allows multiple concurrent readers but exclusive writers.
- If the shared data is immutable, `Arc<T>` alone is sufficient — no `Mutex` needed.

## Compiler Error Interpretation

```
error[E0277]: `Rc<Vec<i32>>` cannot be sent between threads safely
   --> src/main.rs:11:36
    |
11  |         let handle = thread::spawn(move || {
    |                      ------------- ^------
    |                      |             |
    |                      |             `Rc<Vec<i32>>` cannot be sent between threads safely
    |                      required by a bound introduced by this call
    |
    = help: the trait `Send` is not implemented for `Rc<Vec<i32>>`
    = note: required because it appears within the type `[closure]`
note: required by a bound in `spawn`
```

This error is the type system preventing a data race:

- **"`Rc<Vec<i32>>` cannot be sent between threads safely"** — `Rc` does not implement `Send`, so it cannot cross thread boundaries.
- **"the trait `Send` is not implemented for `Rc<Vec<i32>>`"** — The root cause: `Rc`'s non-atomic counter is unsafe for concurrent access.
- **"required by a bound in `spawn`"** — `thread::spawn` requires the closure to be `Send`. Since the closure captures an `Rc`, the closure itself is `!Send`.

The fix: replace `Rc` with `Arc`. `Arc` implements `Send` (when the inner type does), satisfying the `spawn` requirement. The compiler error directly guides you from the unsafe choice to the safe one.

---

| [Prev: Deref Coercion and the Drop Trait](#/katas/deref-and-drop) | [Next: Declarative Macros with macro_rules!](#/katas/declarative-macros) |
