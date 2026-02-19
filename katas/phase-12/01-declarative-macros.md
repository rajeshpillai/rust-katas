---
id: declarative-macros
phase: 12
phase_title: Macros
sequence: 1
title: "Declarative Macros with macro_rules!"
hints:
  - macro_rules! defines macros using pattern matching on syntax fragments
  - Each arm of a macro has a pattern (matcher) and a template (transcriber)
  - If the invocation doesn't match any arm, the compiler reports a "no rules expected" error
  - Common fragment specifiers are expr (expression), ident (identifier), ty (type), and literal
---

## Description

Rust's `macro_rules!` system lets you define macros that transform syntax at compile time. Unlike functions, macros operate on the abstract syntax tree (AST) -- they receive code fragments as input and produce new code as output. Each macro definition consists of one or more **arms**, each with a **matcher** (the pattern) and a **transcriber** (the expansion template).

When you invoke a macro, the compiler tries each arm in order, checking if the invocation matches the matcher. If no arm matches, the compiler reports an error. Understanding how matchers work is essential to writing and debugging macros.

## Broken Code

```rust
macro_rules! make_greeting {
    // Only arm: expects a string literal
    ($name:literal) => {
        format!("Hello, {}! Welcome aboard.", $name)
    };
}

fn main() {
    // This works: passing a literal
    let msg1 = make_greeting!("Alice");
    println!("{}", msg1);

    // This fails: passing a variable (an expression, not a literal)
    let user = "Bob";
    let msg2 = make_greeting!(user);
    println!("{}", msg2);

    // This also fails: passing two arguments
    let msg3 = make_greeting!("Charlie", "morning");
    println!("{}", msg3);
}
```

## Correct Code

```rust
macro_rules! make_greeting {
    // Arm 1: a single expression (handles both literals and variables)
    ($name:expr) => {
        format!("Hello, {}! Welcome aboard.", $name)
    };
    // Arm 2: two expressions (name + time of day)
    ($name:expr, $time:expr) => {
        format!("Good {}, {}! Welcome aboard.", $time, $name)
    };
}

fn main() {
    // Arm 1 matches: a literal
    let msg1 = make_greeting!("Alice");
    println!("{}", msg1);

    // Arm 1 matches: a variable (variables are expressions)
    let user = "Bob";
    let msg2 = make_greeting!(user);
    println!("{}", msg2);

    // Arm 2 matches: two arguments
    let msg3 = make_greeting!("Charlie", "morning");
    println!("{}", msg3);
}
```

## Explanation

The broken version defines `make_greeting!` with a single arm that only accepts a `literal` fragment. The `$name:literal` matcher matches string literals like `"Alice"`, number literals like `42`, and boolean literals like `true`. But it does **not** match variables, function calls, or any other expressions. When you write `make_greeting!(user)`, `user` is an identifier/expression, not a literal, so no arm matches.

The broken version also has no arm that accepts two arguments, so `make_greeting!("Charlie", "morning")` fails.

The correct version fixes both issues:

**Fix 1:** Change `$name:literal` to `$name:expr`. The `expr` fragment specifier matches any Rust expression: variables, literals, function calls, arithmetic, blocks, and more. It is the most general and commonly used fragment specifier.

**Fix 2:** Add a second arm that matches two expressions separated by a comma. The compiler tries arms in order, so the single-expression arm should come first (otherwise the two-expression arm would need to be tried first, which could cause ambiguity).

Here are the key fragment specifiers:

| Specifier | Matches | Example |
|-----------|---------|---------|
| `expr` | Any expression | `x`, `1 + 2`, `foo()` |
| `ident` | An identifier | `my_var`, `String` |
| `ty` | A type | `i32`, `Vec<String>` |
| `literal` | A literal value | `"hello"`, `42`, `true` |
| `pat` | A pattern | `Some(x)`, `_`, `1..=5` |
| `stmt` | A statement | `let x = 5` |
| `block` | A block | `{ println!("hi"); }` |
| `tt` | A single token tree | Any single token or `()`/`[]`/`{}`-delimited group |

Macros are powerful but should be used judiciously. They make code harder to read and debug because the expanded code is not visible in the source. Prefer functions and generics when they suffice; reach for macros when you need variadic arguments, code generation, or syntax extensions that functions cannot express.

## Compiler Error Interpretation

```
error: no rules expected the token `user`
  --> src/main.rs:14:28
   |
1  | macro_rules! make_greeting {
   | -------------------------- when calling this macro
...
14 |     let msg2 = make_greeting!(user);
   |                                ^^^^ no rules expected this token in macro call
   |
note: while trying to match `literal`
  --> src/main.rs:3:6
   |
3  |     ($name:literal) => {
   |      ^^^^^^^^^^^^^^
```

The error message says "no rules expected the token `user`." The compiler tried every arm of the macro and none matched. The note shows which fragment it was trying to match against (`literal`) and why it failed: `user` is an identifier, not a literal. The fix is to widen the fragment specifier from `literal` to `expr`, or to add a new arm that matches identifiers. When you see "no rules expected the token," check which fragment specifier each arm uses and whether it covers the kind of syntax you are passing.
