---
id: pattern-matching-exhaustiveness
phase: 0
phase_title: Rust as a Language
sequence: 5
title: Pattern Matching Exhaustiveness
hints:
  - A match expression must cover every possible variant of the enum
  - The compiler knows all variants and will tell you which ones are missing
  - You can use a wildcard _ to match "everything else"
---

## Description

Rust's `match` expression must be exhaustive — it must cover every possible value of the type being matched. For enums, this means every variant must have a corresponding arm. The compiler enforces this at compile time, making it impossible to forget a case.

## Broken Code

```rust
enum TrafficLight {
    Red,
    Yellow,
    Green,
}

fn action(light: TrafficLight) -> &'static str {
    match light {
        TrafficLight::Red => "Stop",
        TrafficLight::Green => "Go",
    }
}

fn main() {
    let light = TrafficLight::Yellow;
    println!("Action: {}", action(light));
}
```

## Correct Code

```rust
enum TrafficLight {
    Red,
    Yellow,
    Green,
}

fn action(light: TrafficLight) -> &'static str {
    match light {
        TrafficLight::Red => "Stop",
        TrafficLight::Yellow => "Slow down",
        TrafficLight::Green => "Go",
    }
}

fn main() {
    let light = TrafficLight::Yellow;
    println!("Action: {}", action(light));
}
```

## Explanation

The broken code is missing a match arm for `TrafficLight::Yellow`. In many languages, a switch statement without a default case silently falls through or does nothing. Rust takes a different approach: if you are matching on an enum, you must handle every variant. There are no surprises at runtime.

This is one of Rust's most powerful safety features. When you add a new variant to an enum later, the compiler will immediately show you every match expression that needs to be updated. This makes refactoring fearless — the compiler catches forgotten cases for you.

You have two choices when you do not want to handle every variant explicitly:

1. Add a specific arm for every variant (preferred when each variant needs distinct handling).
2. Use a wildcard pattern `_ => ...` as a catch-all for variants you want to handle the same way.

The wildcard approach is convenient but has a tradeoff: if you add a new variant later, the wildcard will silently catch it, and you will not get a compiler warning. Use wildcards deliberately, not as a way to avoid thinking about cases.

## Compiler Error Interpretation

```
error[E0004]: non-exhaustive patterns: `TrafficLight::Yellow` not covered
 --> main.rs:8:11
  |
8 |     match light {
  |           ^^^^^ pattern `TrafficLight::Yellow` not covered
  |
note: `TrafficLight` defined here
 --> main.rs:3:5
  |
1 | enum TrafficLight {
  |      ------------
...
3 |     Yellow,
  |     ^^^^^^ not covered
  = note: the matched value is of type `TrafficLight`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
  |
10~         TrafficLight::Green => "Go",
11~         TrafficLight::Yellow => todo!(),
  |
```

Error E0004 means "non-exhaustive patterns." The compiler names the exact variant you forgot (`TrafficLight::Yellow`) and even suggests adding it. This is the compiler acting as your ally — it is telling you about a logical gap in your code before it ever runs.
