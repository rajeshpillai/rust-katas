---
id: macro-repetition
phase: 12
phase_title: Macros
sequence: 2
title: "Macro Repetition Patterns"
hints:
  - "Repetition in macros uses the syntax $( ... ),* where * means zero or more"
  - "The comma before * is the separator -- it appears between repeated elements"
  - "Use + instead of * for one or more repetitions"
  - "Inside the transcriber, $( ... ),* expands each captured element with the same separator"
---

## Description

Real-world macros often need to accept a variable number of arguments. Rust's `macro_rules!` system supports this through **repetition patterns**. The syntax `$( pattern ),*` matches zero or more occurrences of `pattern` separated by commas. The `*` means "zero or more" and `+` means "one or more."

This is what makes macros strictly more powerful than functions in Rust: functions have a fixed number of parameters, but macros can accept any number of arguments. The `vec![]` macro, `println!()`, and `format!()` all use repetition patterns internally.

## Broken Code

```rust
// This macro tries to accept multiple key-value pairs but has no repetition
macro_rules! make_map {
    ($key:expr => $value:expr) => {
        {
            let mut map = std::collections::HashMap::new();
            map.insert($key, $value);
            map
        }
    };
}

fn main() {
    // Single pair works
    let single = make_map!("name" => "Alice");
    println!("{:?}", single);

    // Multiple pairs fail: the macro only accepts one pair
    let multiple = make_map!(
        "name" => "Bob",
        "age" => "30",
        "city" => "Portland"
    );
    println!("{:?}", multiple);
}
```

## Correct Code

```rust
macro_rules! make_map {
    // Use repetition: $( key => value ),* for zero or more pairs
    ( $( $key:expr => $value:expr ),* ) => {
        {
            let mut map = std::collections::HashMap::new();
            // Repeat the insert for each captured pair
            $( map.insert($key, $value); )*
            map
        }
    };
}

fn main() {
    // Zero pairs: creates an empty map
    let empty: std::collections::HashMap<&str, &str> = make_map!();
    println!("Empty: {:?}", empty);

    // Single pair
    let single = make_map!("name" => "Alice");
    println!("Single: {:?}", single);

    // Multiple pairs
    let multiple = make_map!(
        "name" => "Bob",
        "age" => "30",
        "city" => "Portland"
    );
    println!("Multiple: {:?}", multiple);
}
```

## Explanation

The broken version defines `make_map!` with a single arm that accepts exactly one key-value pair: `$key:expr => $value:expr`. When invoked with three pairs, the macro fails because the invocation does not match the pattern.

The correct version uses repetition syntax to accept zero or more pairs:

**In the matcher:**
```
$( $key:expr => $value:expr ),*
```

This reads as: "Match zero or more occurrences of `expression => expression`, separated by commas." Each occurrence captures a `$key` and a `$value`. After matching, `$key` and `$value` are not single values -- they are lists of values, one for each repetition.

**In the transcriber:**
```
$( map.insert($key, $value); )*
```

This reads as: "For each captured pair, emit `map.insert($key, $value);`." The `$( ... )*` in the transcriber expands once for each captured repetition. If you invoked with three pairs, this expands to three `insert` calls.

The full expansion of `make_map!("name" => "Bob", "age" => "30", "city" => "Portland")` is:

```rust
{
    let mut map = std::collections::HashMap::new();
    map.insert("name", "Bob");
    map.insert("age", "30");
    map.insert("city", "Portland");
    map
}
```

Key details about repetition syntax:

- **`$( ... ),*`** -- zero or more, separated by commas
- **`$( ... ),+`** -- one or more, separated by commas
- **`$( ... );*`** -- zero or more, separated by semicolons
- **`$( ... )*`** -- zero or more, no separator (elements are adjacent)

The separator (`,`, `;`, etc.) appears between the closing `)` and the `*` or `+`. It can be any token. The separator only appears between elements, not after the last one.

An important rule: every metavariable captured in a repetition must be used in a repetition of the same nesting level in the transcriber. If you capture `$key` inside `$( ... )*` in the matcher, you must expand it inside `$( ... )*` in the transcriber. The compiler enforces this.

This pattern is the foundation of macros like `vec![]`:

```rust
macro_rules! vec {
    ( $( $elem:expr ),* ) => {
        {
            let mut v = Vec::new();
            $( v.push($elem); )*
            v
        }
    };
}
```

## Compiler Error Interpretation

```
error: no rules expected the token `"age"`
  --> src/main.rs:18:9
   |
2  | macro_rules! make_map {
   | ---------------------- when calling this macro
...
18 |         "age" => "30",
   |         ^^^^^ no rules expected this token in macro call
```

The compiler successfully matched the first key-value pair (`"name" => "Bob"`) and then encountered a comma followed by `"age"`. Since the macro's pattern only accepts a single pair and has no repetition, it does not expect anything after the first pair. The token `"age"` is unexpected because the macro definition ended after one `$key => $value`. Adding `$( ... ),*` repetition allows the macro to continue matching additional pairs after each comma.
