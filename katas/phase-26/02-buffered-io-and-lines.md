---
id: buffered-io-and-lines
phase: 26
phase_title: "File I/O in Rust"
sequence: 2
title: Buffered I/O and Line-by-Line Reading
hints:
  - "The `.lines()` method is not available on `File` directly. It belongs to a different trait."
  - "`File` implements `Read`, but `.lines()` requires the `BufRead` trait. You need a wrapper."
  - "Wrap `File` in `BufReader` and add `use std::io::BufRead` to bring the trait into scope."
---

## Description

Reading a file byte-by-byte with raw `Read` calls is inefficient — each `read` call is a system call. **Buffered I/O** wraps a reader in an internal buffer, batching many small reads into fewer large ones.

In Rust:
- `BufReader<R>` wraps any `Read` type and adds buffering + the `BufRead` trait
- `BufWriter<W>` wraps any `Write` type and adds write buffering
- The `BufRead` trait provides `.lines()`, `.read_line()`, and `.split()`

The `.lines()` method returns an iterator of `Result<String, io::Error>` — one `String` per line, with the trailing newline stripped.

## Broken Code

```rust
use std::fs::File;
use std::io::Write;

fn main() {
    let path = std::env::temp_dir().join("kata-lines-demo.txt");

    // Write some lines
    let mut file = File::create(&path).unwrap();
    writeln!(file, "first line").unwrap();
    writeln!(file, "second line").unwrap();
    writeln!(file, "third line").unwrap();

    // Try to read line by line
    let file = File::open(&path).unwrap();
    for line in file.lines() {
        println!("{}", line.unwrap());
    }

    std::fs::remove_file(&path).unwrap();
}
```

## Correct Code

```rust
use std::fs::File;
use std::io::{Write, BufRead, BufReader, BufWriter};

fn main() {
    let path = std::env::temp_dir().join("kata-lines-demo.txt");

    // BufWriter batches writes for efficiency
    let file = File::create(&path).unwrap();
    let mut writer = BufWriter::new(file);
    writeln!(writer, "first line").unwrap();
    writeln!(writer, "second line").unwrap();
    writeln!(writer, "third line").unwrap();
    // BufWriter flushes on drop, but explicit flush is good practice
    drop(writer);

    // BufReader enables line-by-line reading via the BufRead trait
    let file = File::open(&path).unwrap();
    let reader = BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        let line = line.unwrap(); // Each line is Result<String, io::Error>
        println!("Line {}: {}", i + 1, line);
    }

    // Alternative: read_line into a reusable buffer (avoids allocation per line)
    let file = File::open(&path).unwrap();
    let mut reader = BufReader::new(file);
    let mut buf = String::new();
    loop {
        buf.clear();
        let bytes_read = reader.read_line(&mut buf).unwrap();
        if bytes_read == 0 {
            break; // EOF
        }
        print!("  raw: {:?}", buf); // Includes trailing newline
    }

    std::fs::remove_file(&path).unwrap();
}
```

## Explanation

The broken code calls `.lines()` directly on a `File`. This fails to compile because:

1. `.lines()` is defined on the **`BufRead`** trait, not on `Read` or `File`.
2. `File` implements `Read`, but **not** `BufRead`.
3. You must wrap the `File` in `BufReader` to get `BufRead`.

The trait hierarchy:

```
Read            (raw byte reading)
  └── BufRead   (buffered reading: lines(), read_line(), split())
        └── BufReader<R: Read> implements BufRead
```

`BufReader` adds an internal buffer (default 8 KB). When you call `.lines()`, it reads a large chunk into the buffer and then yields lines from it — far fewer system calls than reading one byte at a time.

**Two ways to read lines:**

| Method | Allocation | Newline |
|---|---|---|
| `.lines()` iterator | New `String` per line | Stripped |
| `.read_line(&mut buf)` | Reuses one `String` | Included |

Use `.lines()` for simplicity. Use `.read_line()` when processing millions of lines and allocation matters.

**`BufWriter`** does the same for writes — it buffers small writes and flushes them in batches. Important: `BufWriter` flushes its buffer when dropped, but if the flush fails during drop, the error is silently ignored. Call `.flush()` explicitly if you need to handle write errors.

**Trait imports matter.** Even after wrapping in `BufReader`, you must have `use std::io::BufRead` in scope. Without the trait import, the compiler cannot find the `.lines()` method.

## ⚠️ Caution

- `BufWriter` silently ignores flush errors on drop. Always call `.flush()` explicitly when write errors must be handled.
- `.lines()` allocates a new `String` for every line. For very large files (millions of lines), `.read_line()` with a reusable buffer is more efficient.
- `.lines()` strips `\n` and `\r\n`. If you need to preserve the original line endings, use `.read_line()` instead.

## 💡 Tips

- The default buffer size for `BufReader` is 8 KB. You can customize it with `BufReader::with_capacity(size, reader)`.
- `BufRead` also provides `.split(byte)` which splits on any byte delimiter, not just newlines — useful for binary protocols.
- When chaining `.lines()` with iterator adapters like `.filter()` and `.map()`, you get expressive file processing pipelines with lazy evaluation.

## Compiler Error Interpretation

```
error[E0599]: no method named `lines` found for struct `File` in the current scope
  --> main.rs:14:23
   |
14 |     for line in file.lines() {
   |                      ^^^^^ method not found in `File`
   |
   = help: items from traits can only be used if the trait is in scope
help: the following traits which provide `lines` are implemented but not in scope; perhaps you want to import one of them
   |
1  + use std::io::BufRead;
   |
```

This error reveals a key Rust pattern:

- **"no method named `lines` found for struct `File`"** — `File` does not have a `.lines()` method. The method exists, but not on this type.
- **"items from traits can only be used if the trait is in scope"** — The compiler knows that `BufRead` has `.lines()`, but `File` does not implement `BufRead`. Even if it did, you would need the `use` import.
- **The suggestion to `use std::io::BufRead`** — The compiler helps, but the real fix is also wrapping `File` in `BufReader`, since `File` alone does not implement `BufRead`.

This is a two-part fix: wrap in `BufReader` AND import the trait.

---

| [Prev: Writing and Reading Text Files](#/katas/writing-and-reading-text) | [Next: Binary Files: Structs to Bytes](#/katas/binary-files-structs-to-bytes) |
