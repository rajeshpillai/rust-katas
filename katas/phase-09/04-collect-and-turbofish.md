---
id: collect-and-turbofish
phase: 9
phase_title: "Iterators & Zero-Cost Abstractions"
sequence: 4
title: "collect() and the Turbofish"
hints:
  - collect() can produce many different collection types (Vec, HashSet, HashMap, String, etc.)
  - The compiler needs to know which collection type you want -- it cannot guess
  - You can annotate the variable type or use the turbofish syntax .collect::<Vec<_>>()
  - The underscore _ in Vec<_> lets the compiler infer the element type while you specify the collection type
---

## Description

The `collect()` method is one of the most powerful consumers in Rust's iterator toolkit. It can transform an iterator into virtually any collection that implements `FromIterator`. But this power comes with an obligation: you must tell the compiler *which* collection type you want. Since `collect()` can produce a `Vec<T>`, a `HashSet<T>`, a `HashMap<K, V>`, a `String`, a `Result<Vec<T>, E>`, and many other types, the compiler cannot infer the target type from context alone.

There are two ways to provide this information: annotate the variable's type, or use the **turbofish** syntax (`::<>`).

## Broken Code

```rust
fn main() {
    let words = vec!["hello", "world", "from", "rust"];

    // collect() without any type information -- what collection do we want?
    let uppercased = words
        .iter()
        .map(|w| w.to_uppercase())
        .collect();

    println!("{:?}", uppercased);

    // Another example: collecting into... what?
    let numbers = vec![1, 2, 3, 4, 5];
    let result = numbers
        .iter()
        .filter(|&&x| x > 2)
        .collect();

    println!("{:?}", result);
}
```

## Correct Code

```rust
use std::collections::HashSet;

fn main() {
    let words = vec!["hello", "world", "from", "rust"];

    // Option A: Annotate the variable type
    let uppercased: Vec<String> = words
        .iter()
        .map(|w| w.to_uppercase())
        .collect();

    println!("{:?}", uppercased);

    // Option B: Use the turbofish syntax
    let numbers = vec![1, 2, 3, 4, 5];
    let result = numbers
        .iter()
        .filter(|&&x| x > 2)
        .collect::<Vec<&i32>>();

    println!("{:?}", result);

    // Option C: Turbofish with underscore to let Rust infer the element type
    let also_works = numbers
        .iter()
        .filter(|&&x| x > 2)
        .collect::<Vec<_>>();

    println!("{:?}", also_works);

    // Bonus: collect into a completely different collection type
    let unique: HashSet<&i32> = numbers.iter().collect();
    println!("Unique: {:?}", unique);
}
```

## Explanation

The broken version calls `.collect()` without telling the compiler what type of collection to produce. The `collect()` method signature is:

```rust
fn collect<B: FromIterator<Self::Item>>(self) -> B
```

The return type `B` is generic -- it can be any type that implements `FromIterator`. Since `Vec<T>`, `HashSet<T>`, `BTreeSet<T>`, `HashMap<K, V>`, `String`, `Result<V, E>`, and others all implement `FromIterator`, the compiler has no way to choose among them.

The correct version demonstrates three approaches:

**Approach A: Type annotation on the variable.** Write `let result: Vec<String> = ...collect()`. The compiler sees that `result` must be `Vec<String>` and works backward to resolve the generic parameter `B` in `collect()`.

**Approach B: Turbofish syntax.** Write `.collect::<Vec<&i32>>()`. The turbofish `::<>` provides the type parameter directly at the call site. This is useful when you want to keep the code on one line or when there is no variable binding to annotate (e.g., passing the result directly to a function).

**Approach C: Turbofish with `_`.** Write `.collect::<Vec<_>>()`. The underscore tells the compiler "I want a `Vec`, but figure out the element type yourself." This is the most common pattern in practice. You specify the collection shape and let inference handle the element type.

The name "turbofish" comes from the visual resemblance of `::<>` to a fish. Despite the playful name, it is an essential part of Rust syntax for disambiguating generic types at call sites.

Note that `collect()` is not just for `Vec`. You can collect an iterator of `char` into a `String`, an iterator of `(K, V)` tuples into a `HashMap`, or even an iterator of `Result<T, E>` into a `Result<Vec<T>, E>` -- which short-circuits on the first error. This last pattern is particularly powerful for error handling in iterator pipelines.

## Compiler Error Interpretation

```
error[E0282]: type annotations needed
 --> src/main.rs:7:9
  |
7 |     let uppercased = words
  |         ^^^^^^^^^^ consider giving `uppercased` a type
  |
  = note: cannot infer type for type parameter `B` declared on the method `collect`
```

This is one of the most common iterator errors. The compiler says it cannot infer the type parameter `B` on `collect()`. The fix is always the same: provide a type annotation. Either annotate the binding (`let x: Vec<_> = ...`) or use turbofish (`.collect::<Vec<_>>()`). When you see "type annotations needed" involving `collect`, the compiler is asking you to choose your collection type.
