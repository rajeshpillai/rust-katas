---
id: writing-and-reading-text
phase: 26
phase_title: "File I/O in Rust"
sequence: 1
title: Writing and Reading Text Files
hints:
  - "There are two different functions for opening files — one for reading and one for writing. Check which one you need."
  - "`File::open()` opens a file for reading only. To write, you need a different constructor."
  - "Use `File::create()` to open a file for writing (it creates or truncates). Use `File::open()` to open for reading."
---

## Description

File I/O is fundamental to systems programming. In Rust, the `std::fs` module provides the core tools. The two most common entry points are:

- `File::open(path)` — Opens an **existing** file for **reading**
- `File::create(path)` — Creates a new file (or truncates an existing one) for **writing**

Both return `Result<File, io::Error>`. The `File` type implements both `Read` and `Write` traits, but which operations actually succeed depends on **how** the file was opened — this is a runtime property, not a compile-time guarantee.

## Broken Code

```rust
use std::fs::File;
use std::io::{Write, Read};

fn main() {
    let path = std::env::temp_dir().join("kata-text-demo.txt");

    // Attempt to write using File::open
    let mut file = File::open(&path).unwrap();
    file.write_all(b"Hello, Rust file I/O!\n").unwrap();
    file.write_all(b"This is line two.\n").unwrap();

    // Read it back
    let mut contents = String::new();
    let mut file = File::open(&path).unwrap();
    file.read_to_string(&mut contents).unwrap();
    println!("File contents:\n{}", contents);
}
```

## Correct Code

```rust
use std::fs::File;
use std::io::{Write, Read};

fn main() {
    let path = std::env::temp_dir().join("kata-text-demo.txt");

    // File::create opens for WRITING (creates or truncates)
    let mut file = File::create(&path).unwrap();
    file.write_all(b"Hello, Rust file I/O!\n").unwrap();
    file.write_all(b"This is line two.\n").unwrap();

    // File::open opens for READING
    let mut contents = String::new();
    let mut file = File::open(&path).unwrap();
    file.read_to_string(&mut contents).unwrap();
    println!("File contents:\n{}", contents);

    // Alternative: std::fs convenience functions
    std::fs::write(&path, "Written with fs::write\n").unwrap();
    let contents = std::fs::read_to_string(&path).unwrap();
    println!("After fs::write:\n{}", contents);

    // Clean up
    std::fs::remove_file(&path).unwrap();
}
```

## Explanation

The broken code calls `File::open(&path)` to write to a file. This fails for two reasons:

1. **The file does not exist yet.** `File::open` only opens *existing* files — it does not create new ones. The first `unwrap()` panics with "No such file or directory".

2. **Even if the file existed**, `File::open` opens it in **read-only** mode. Calling `write_all` on a read-only handle produces an OS-level error like "Bad file descriptor" or "Permission denied".

The fix is to use `File::create(&path)` for the write operation:

| Function | Mode | Creates? | Truncates? |
|---|---|---|---|
| `File::open(path)` | Read-only | No | No |
| `File::create(path)` | Write-only | Yes | Yes |

Both return the same `File` type. Rust's type system does **not** distinguish between a read handle and a write handle — both are `File`. The distinction is purely at the OS level. This is one of the rare cases where Rust does not give you compile-time safety.

For full control over open mode, use `OpenOptions`:

```rust
use std::fs::OpenOptions;

let file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .open(path)?;
```

**Convenience functions** in `std::fs` simplify common one-shot operations:
- `std::fs::write(path, contents)` — writes bytes/string to a file (creates or truncates)
- `std::fs::read_to_string(path)` — reads entire file into a `String`
- `std::fs::read(path)` — reads entire file into `Vec<u8>`

These are ideal for small files. For large files or line-by-line processing, use `File` with `BufReader` (next kata).

## ⚠️ Caution

- `File::create` **truncates** the file if it already exists. All previous contents are lost. If you want to append, use `OpenOptions::new().append(true).create(true).open(path)`.
- `read_to_string` loads the entire file into memory. For large files, use buffered line-by-line reading instead.
- Always handle `io::Error` properly in production code — `unwrap()` is acceptable only in examples and tests.

## 💡 Tips

- Use `std::fs::write` and `std::fs::read_to_string` for simple one-shot file operations — they handle open/close automatically.
- Paths in Rust use `PathBuf` (owned) and `&Path` (borrowed), following the same owned/borrowed duality as `String`/`&str`. The `join` method on paths is the safe way to build file paths.
- `write_all` writes raw bytes (`&[u8]`). Use the `write!` or `writeln!` macros for formatted text output to files.

## Compiler Error Interpretation

When the broken code runs:

```
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message: "No such file or directory" }'
```

This is a **runtime** error, not a compiler error. The program compiles fine because `File` implements the `Write` trait regardless of how it was opened. The error comes from the operating system when `File::open` tries to open a file that does not exist.

- **`code: 2`** — The POSIX error code for `ENOENT` (file not found)
- **`kind: NotFound`** — Rust's cross-platform `ErrorKind` classification
- **`File::open` only opens existing files** — it never creates new ones

The lesson: some invariants in Rust are enforced at compile time (ownership, borrowing), but file permissions and existence are runtime properties. You must use `Result` handling to deal with them.
