---
id: control-flow
phase: 0
phase_title: "Rust as a Language"
sequence: 8
title: Control Flow — if, loops, and match
hints:
  - "In Rust, conditions in `if` expressions do not use parentheses, and the condition must be a `bool` — integers are not truthy."
  - "`if` is an expression in Rust, not just a statement. It can return a value."
  - "Use `for item in collection` for iteration. Ranges like `0..5` give you 0, 1, 2, 3, 4."
---

## Description

Rust has three loop constructs: `loop` (infinite), `while` (conditional), and `for` (iteration). Unlike C-family languages, `if` is an **expression** — it returns a value. Conditions must be explicitly `bool` — there is no implicit truthiness.

## Broken Code

```rust
fn main() {
    let x = 5;

    // Bug 1: Parentheses are not needed, but the real bug is
    // using an integer as a condition — Rust has no truthiness
    if x {
        println!("x is truthy");
    }

    // Bug 2: if arms return different types
    let label = if x > 3 {
        "big"
    } else {
        42
    };

    // Bug 3: inclusive range in for loop — off by one
    for i in 0..3 {
        println!("i = {}", i); // Prints 0, 1, 2 — misses 3!
    }
}
```

## Correct Code

```rust
fn main() {
    let x = 5;

    // --- if / else ---
    // Conditions must be bool — no implicit truthiness
    if x > 0 {
        println!("{} is positive", x);
    } else if x < 0 {
        println!("{} is negative", x);
    } else {
        println!("zero");
    }

    // if is an expression — it returns a value
    let label = if x > 3 { "big" } else { "small" };
    println!("{} is {}", x, label);

    // --- for loops ---
    // Exclusive range: 0..5 gives 0, 1, 2, 3, 4
    print!("Exclusive 0..5: ");
    for i in 0..5 {
        print!("{} ", i);
    }
    println!();

    // Inclusive range: 0..=5 gives 0, 1, 2, 3, 4, 5
    print!("Inclusive 0..=5: ");
    for i in 0..=5 {
        print!("{} ", i);
    }
    println!();

    // Iterating over an array
    let fruits = ["apple", "banana", "cherry"];
    for fruit in &fruits {
        println!("I like {}", fruit);
    }

    // Enumerate gives (index, value) pairs
    for (i, fruit) in fruits.iter().enumerate() {
        println!("  [{}] {}", i, fruit);
    }

    // --- while loops ---
    let mut countdown = 3;
    while countdown > 0 {
        println!("{}...", countdown);
        countdown -= 1;
    }
    println!("Go!");

    // --- loop (infinite until break) ---
    let mut sum = 0;
    let mut n = 1;
    loop {
        sum += n;
        if sum > 20 {
            break;  // Exit the loop
        }
        n += 1;
    }
    println!("Sum exceeded 20 at n={}, sum={}", n, sum);

    // loop can return a value via break
    let result = loop {
        n += 1;
        if n % 7 == 0 {
            break n;  // Returns n from the loop
        }
    };
    println!("First multiple of 7 after {}: {}", n - 1, result);

    // --- match as control flow ---
    let grade = 85;
    let letter = match grade {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    };
    println!("Grade {}: {}", grade, letter);

    // match with guards
    let temperature = 35;
    let description = match temperature {
        t if t < 0 => "freezing",
        t if t < 15 => "cold",
        t if t < 25 => "comfortable",
        t if t < 35 => "warm",
        _ => "hot",
    };
    println!("{}°C is {}", temperature, description);
}
```

## Explanation

The broken code has three bugs:

**Bug 1: Integer used as condition.** Rust has no concept of "truthiness." In C/Python, `if 5` is valid because nonzero integers are truthy. In Rust, `if` requires a `bool`. You must write `if x > 0` or `if x != 0`.

**Bug 2: `if` arms return different types.** Since `if` is an expression that returns a value, both arms must return the **same type**. `"big"` is `&str` but `42` is `i32` — the compiler rejects this.

**Bug 3: Off-by-one with ranges.** `0..3` is exclusive — it produces 0, 1, 2. For inclusive ranges, use `0..=3` which produces 0, 1, 2, 3.

**Rust's three loop types:**

| Loop | Use when |
|---|---|
| `for x in collection` | Iterating over known data |
| `while condition` | Looping until a condition changes |
| `loop` | Infinite loop with explicit `break` |

**`loop` is special** — it can return a value via `break value`. This is useful when you need to compute something in a loop and assign the result.

**`match` for control flow** goes beyond enum pattern matching. You can match on integers, ranges (`80..=89`), and use guard clauses (`t if t < 0`). The `_` wildcard matches everything else.

## ⚠️ Caution

- `for i in 0..n` is **exclusive** — it stops before `n`. Use `0..=n` for inclusive. This is the most common off-by-one error in Rust.
- `loop` without `break` runs forever. The compiler will warn if a `loop` has no reachable `break`.
- All `match` arms must return the same type when used as an expression.

## 💡 Tips

- Use `for _ in 0..n` (underscore) when you need to repeat N times but don't need the index.
- `continue` skips to the next iteration. `break` exits the loop. Both work in `for`, `while`, and `loop`.
- Label loops for nested break/continue: `'outer: for i in 0..10 { 'inner: for j in 0..10 { break 'outer; } }`.
- Prefer `for` over `while` when iterating over collections — it is safer (no off-by-one) and more idiomatic.

## Compiler Error Interpretation

```
error[E0308]: mismatched types
 --> main.rs:4:8
  |
4 |     if x {
  |        ^ expected `bool`, found integer
```

Clear message: `if` requires a `bool`, not an integer. Rust's lack of implicit truthiness prevents bugs like `if (ptr)` in C where a null pointer silently passes.

For the mismatched `if` arms:

```
error[E0308]: `if` and `else` have incompatible types
 --> main.rs:8:9
  |
6 |     let label = if x > 3 {
  |                 - `if` and `else` have incompatible types
7 |         "big"
  |         ----- expected because of this
8 |         42
  |         ^^ expected `&str`, found integer
```

Since `if` is an expression, both branches must agree on the return type. This constraint catches bugs at compile time.

---

| [Prev: Functions — The Basic Unit of Code](#/katas/functions) | [Next: Tuples and Destructuring](#/katas/tuples-and-destructuring) |
