---
id: owned-borrowed-duality
phase: 6
phase_title: "Collections & the Owned/Borrowed Duality"
sequence: 7
title: "The Owned/Borrowed Duality: PathBuf/Path, OsString/OsStr"
hints:
  - "Rust pairs every owned type with a borrowed counterpart: String/&str, Vec<T>/&[T], PathBuf/&Path, OsString/&OsStr."
  - "Functions that only read data should accept the borrowed form (&Path, &str, &[T]) — this is more flexible."
  - "The owned form can always be borrowed (via Deref), but borrowing never allocates."
---

## Description

One of Rust's most important patterns is the **owned/borrowed duality**: for every heap-allocated, growable type, there is a corresponding borrowed, unsized slice type. You have already seen `String`/`&str` and `Vec<T>`/`&[T]`. This pattern extends throughout the standard library:

| Owned (heap, growable) | Borrowed (view, unsized) | Domain |
|---|---|---|
| `String` | `&str` | UTF-8 text |
| `Vec<T>` | `&[T]` | Sequences |
| `PathBuf` | `&Path` | File paths |
| `OsString` | `&OsStr` | OS strings |
| `CString` | `&CStr` | C interop strings |

Understanding this duality is understanding Rust. It governs how you design function signatures, how data flows through your program, and how you avoid unnecessary allocations.

## Broken Code

```rust
use std::path::PathBuf;

fn has_rust_extension(path: PathBuf) -> bool {
    // Takes ownership of the PathBuf — the caller loses it.
    match path.extension() {
        Some(ext) => ext == "rs",
        None => false,
    }
}

fn build_output_path(dir: String, name: String) -> String {
    // Manual string concatenation — fragile, not cross-platform.
    // On Windows, this produces wrong separators.
    format!("{}/{}.o", dir, name)
}

fn main() {
    let source = PathBuf::from("/home/user/project/main.rs");

    // First call works...
    println!("Is Rust? {}", has_rust_extension(source));

    // Second call fails — source was moved!
    println!("Path: {}", source.display());
}
```

## Correct Code

```rust
use std::path::{Path, PathBuf};
use std::ffi::{OsStr, OsString};

// Accept &Path (borrowed) instead of PathBuf (owned).
// This accepts PathBuf, &Path, &str, &String — anything that
// implements AsRef<Path>.
fn has_rust_extension(path: &Path) -> bool {
    path.extension() == Some(OsStr::new("rs"))
}

// Use Path methods for cross-platform path manipulation.
fn build_output_path(dir: &Path, name: &str) -> PathBuf {
    let mut output = dir.join(name);
    output.set_extension("o");
    output
}

// Even more flexible: accept anything that can be viewed as a Path.
fn print_file_info(path: impl AsRef<Path>) {
    let path = path.as_ref();
    println!("  Path: {}", path.display());
    println!("  File name: {:?}", path.file_name());
    println!("  Extension: {:?}", path.extension());
    println!("  Parent: {:?}", path.parent());
    println!("  Is absolute: {}", path.is_absolute());
}

fn main() {
    let source = PathBuf::from("/home/user/project/main.rs");

    // &Path — does not consume the PathBuf
    println!("Is Rust? {}", has_rust_extension(&source));
    println!("Is Rust? {}", has_rust_extension(&source)); // Can call again!

    // Cross-platform path building
    let output = build_output_path(Path::new("/home/user/build"), "main");
    println!("Output: {}", output.display());

    // AsRef<Path> accepts many types
    print_file_info(&source);          // &PathBuf
    print_file_info("/etc/hosts");     // &str
    print_file_info(String::from("/tmp/test.txt")); // String

    // --- The duality pattern ---
    // Owned types Deref to their borrowed counterparts:
    let owned_path: PathBuf = PathBuf::from("/home/user");
    let borrowed_path: &Path = &owned_path; // Deref coercion

    let owned_string: String = String::from("hello");
    let borrowed_str: &str = &owned_string; // Deref coercion

    let owned_vec: Vec<i32> = vec![1, 2, 3];
    let borrowed_slice: &[i32] = &owned_vec; // Deref coercion

    // OsString / OsStr — for non-UTF-8 OS strings
    let os_owned: OsString = OsString::from("config.toml");
    let os_borrowed: &OsStr = &os_owned;
    println!("\nOsStr: {:?}", os_borrowed);

    println!("\nBorrowed path: {}", borrowed_path.display());
    println!("Borrowed str: {}", borrowed_str);
    println!("Borrowed slice: {:?}", borrowed_slice);
}
```

## Explanation

The broken version takes `PathBuf` by value, consuming the caller's path. It also builds paths using string concatenation, which is fragile and platform-dependent (Windows uses `\` not `/`).

**The core principle:** functions that only read data should accept the borrowed form.

```rust
// Bad: takes ownership unnecessarily
fn check(path: PathBuf) -> bool { ... }

// Good: borrows — accepts PathBuf, &Path, &str, etc.
fn check(path: &Path) -> bool { ... }
```

This works because of **Deref coercion**. `PathBuf` implements `Deref<Target = Path>`, so when you pass `&path_buf` where `&Path` is expected, Rust automatically dereferences. The same mechanism makes `&String` coerce to `&str` and `&Vec<T>` coerce to `&[T]`.

**The full duality:**

Each pair follows the same pattern:
- The **owned** type allocates on the heap, is growable, and implements `Deref` to its borrowed counterpart.
- The **borrowed** type is unsized (`[T]`, `str`, `Path`, `OsStr`), always behind a reference, and provides read-only access with no allocation.
- You create the owned form, pass the borrowed form, and return the owned form when the function creates new data.

**Why `OsStr` and `OsString` exist:**

File paths on Unix are arbitrary bytes (not necessarily UTF-8). On Windows, they are WTF-16 (not necessarily valid UTF-16). Neither maps cleanly to Rust's `str` (which is always valid UTF-8). `OsStr`/`OsString` handle platform-native strings without requiring UTF-8 validity. `Path`/`PathBuf` are thin wrappers around `OsStr`/`OsString` with path-specific methods.

**The design rule:**

| You need to... | Accept | Return |
|---|---|---|
| Read a path | `&Path` or `impl AsRef<Path>` | — |
| Create a new path | — | `PathBuf` |
| Read a string | `&str` or `impl AsRef<str>` | — |
| Create a new string | — | `String` |
| Read a sequence | `&[T]` | — |
| Create a new sequence | — | `Vec<T>` |

## ⚠️ Caution

- `Path::new("...")` creates a `&Path` from a string literal — it does not allocate. `PathBuf::from("...")` allocates on the heap.
- File paths are not guaranteed to be valid UTF-8. Methods like `path.to_str()` return `Option<&str>` because conversion can fail. Use `path.display()` for printing (it replaces invalid characters).
- Do not use string concatenation for paths. Use `path.join()` and `path.set_extension()` for cross-platform correctness.

## 💡 Tips

- Use `impl AsRef<Path>` for maximum flexibility — it accepts `&str`, `String`, `&Path`, `PathBuf`, and `&OsStr`.
- When designing APIs, follow the standard library's convention: accept borrowed, return owned.
- Remember that `Deref` coercion is implicit. You rarely need to call `.as_path()` or `.as_str()` explicitly — just pass a reference.

## Compiler Error Interpretation

```
error[E0382]: borrow of moved value: `source`
 --> src/main.rs:18:28
  |
14 |     let source = PathBuf::from("/home/user/project/main.rs");
   |         ------ move occurs because `source` has type `PathBuf`, which does not implement the `Copy` trait
15 |
16 |     println!("Is Rust? {}", has_rust_extension(source));
   |                                                ------ value moved here
17 |
18 |     println!("Path: {}", source.display());
   |                          ^^^^^^ value borrowed here after move
```

This is the same E0382 from Phase 1, but now applied to `PathBuf`. The function took ownership when it only needed to read. The fix is to change the parameter from `PathBuf` to `&Path` — the function borrows instead of consuming. This is the owned/borrowed duality in action: accept the borrowed form to be a good API citizen.

---

| [Prev: HashSet, BTreeMap, and Ordered Collections](#/katas/hashset-and-btreemap) | [Next: Enum-Driven State Machines](#/katas/enum-state-machines) |
