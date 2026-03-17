---
id: random-access-seek
phase: 26
phase_title: "File I/O in Rust"
sequence: 5
title: Random Access with Seek
hints:
  - "When seeking to a record in a binary file, the position is in bytes, not record numbers."
  - "If each record is 16 bytes, record 3 starts at byte `3 * 16`, not byte `3`."
  - "Multiply the record index by `RECORD_SIZE` to compute the correct byte offset for `SeekFrom::Start`."
---

## Description

Sequential file access reads from beginning to end. **Random access** lets you jump directly to any position in the file using the `Seek` trait.

The `Seek` trait provides one method:
```rust
fn seek(&mut self, pos: SeekFrom) -> io::Result<u64>;
```

`SeekFrom` has three variants:
- `SeekFrom::Start(n)` — position `n` bytes from the beginning
- `SeekFrom::End(n)` — position `n` bytes from the end (usually negative)
- `SeekFrom::Current(n)` — position `n` bytes from the current position

Random access only makes sense with **fixed-size records** — if every record is the same number of bytes, you can compute any record's position as `index * record_size`.

## Broken Code

```rust
use std::fs::File;
use std::io::{Write, Read, Seek, SeekFrom};

const RECORD_SIZE: u64 = 16; // u64 (8 bytes) + f64 (8 bytes)

fn write_record(file: &mut File, id: u64, value: f64) {
    file.write_all(&id.to_le_bytes()).unwrap();
    file.write_all(&value.to_le_bytes()).unwrap();
}

fn read_record(file: &mut File) -> (u64, f64) {
    let mut id_buf = [0u8; 8];
    let mut val_buf = [0u8; 8];
    file.read_exact(&mut id_buf).unwrap();
    file.read_exact(&mut val_buf).unwrap();
    (u64::from_le_bytes(id_buf), f64::from_le_bytes(val_buf))
}

fn main() {
    let path = std::env::temp_dir().join("kata-seek-demo.bin");

    // Write 5 records
    let mut file = File::create(&path).unwrap();
    for i in 0u64..5 {
        write_record(&mut file, i, (i as f64) * 10.0);
    }

    // Read record at index 3
    let mut file = File::open(&path).unwrap();
    file.seek(SeekFrom::Start(3)).unwrap(); // BUG: seeks to byte 3, not record 3!

    let (id, value) = read_record(&mut file);
    println!("Record 3: id={}, value={:.1}", id, value);
    // Prints garbage — the bytes are misaligned!

    std::fs::remove_file(&path).unwrap();
}
```

## Correct Code

```rust
use std::fs::{File, OpenOptions};
use std::io::{Write, Read, Seek, SeekFrom};

const RECORD_SIZE: u64 = 16; // u64 (8 bytes) + f64 (8 bytes)

fn write_record(file: &mut File, id: u64, value: f64) {
    file.write_all(&id.to_le_bytes()).unwrap();
    file.write_all(&value.to_le_bytes()).unwrap();
}

fn read_record(file: &mut File) -> (u64, f64) {
    let mut id_buf = [0u8; 8];
    let mut val_buf = [0u8; 8];
    file.read_exact(&mut id_buf).unwrap();
    file.read_exact(&mut val_buf).unwrap();
    (u64::from_le_bytes(id_buf), f64::from_le_bytes(val_buf))
}

fn main() {
    let path = std::env::temp_dir().join("kata-seek-demo.bin");

    // Write 5 records
    let mut file = File::create(&path).unwrap();
    for i in 0u64..5 {
        write_record(&mut file, i, (i as f64) * 10.0);
    }

    // Read record at index 3 — byte offset = index * RECORD_SIZE
    let mut file = File::open(&path).unwrap();
    let offset = 3 * RECORD_SIZE;
    file.seek(SeekFrom::Start(offset)).unwrap();

    let (id, value) = read_record(&mut file);
    println!("Record 3: id={}, value={:.1}", id, value);
    // Correct: id=3, value=30.0

    // Read all records sequentially from the start
    file.seek(SeekFrom::Start(0)).unwrap();
    let file_size = file.metadata().unwrap().len();
    let num_records = file_size / RECORD_SIZE;
    println!("\nAll {} records:", num_records);
    for _ in 0..num_records {
        let (id, value) = read_record(&mut file);
        println!("  id={}, value={:.1}", id, value);
    }

    // Update record 2 in place
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();

    let offset = 2 * RECORD_SIZE;
    file.seek(SeekFrom::Start(offset)).unwrap();
    write_record(&mut file, 2, 999.9); // Overwrite record 2

    // Verify the update
    file.seek(SeekFrom::Start(offset)).unwrap();
    let (id, value) = read_record(&mut file);
    println!("\nAfter update — Record 2: id={}, value={:.1}", id, value);

    std::fs::remove_file(&path).unwrap();
}
```

## Explanation

The broken code seeks to **byte** 3 instead of **record** 3. Since each record is 16 bytes, record 3 starts at byte 48 (`3 * 16`), not byte 3.

When you seek to byte 3 and read 16 bytes, you get bytes 3–18 of the file. This spans parts of record 0 and record 1, producing garbled data. The bytes decode to *something* — `u64::from_le_bytes` always succeeds — but the values are meaningless. **There is no error, no crash, just wrong data.**

This is the most dangerous class of bug in binary file programming: **silent data corruption**.

**The fix is a single multiplication:**

```rust
// Wrong: seeks to byte 3
file.seek(SeekFrom::Start(3)).unwrap();

// Right: seeks to the start of record 3
file.seek(SeekFrom::Start(3 * RECORD_SIZE)).unwrap();
```

**`OpenOptions` for in-place updates:**

`File::open` is read-only. `File::create` truncates. For reading AND writing an existing file (updating records in place), you need:

```rust
OpenOptions::new()
    .read(true)
    .write(true)
    .open(path)
```

This opens the file with both read and write access without truncating it.

**The three `SeekFrom` variants:**

| Variant | Use case |
|---|---|
| `SeekFrom::Start(n)` | Jump to absolute byte position |
| `SeekFrom::Current(n)` | Skip forward/backward relative to current position |
| `SeekFrom::End(n)` | Jump relative to file end (use negative for "N bytes before end") |

`seek` returns the new absolute position, which you can use to verify you are where you expect.

## ⚠️ Caution

- Seek offsets are in **bytes**, not records. Always multiply by `RECORD_SIZE`.
- Seeking past the end of a file is allowed — it does not produce an error. But reading at that position will fail with an unexpected EOF.
- Binary file bugs are **silent**. Misaligned reads produce wrong data, not errors. Always validate critical reads.

## 💡 Tips

- Define a helper function: `fn record_offset(index: u64) -> u64 { index * RECORD_SIZE }`. This eliminates the most common seek bug.
- Use `file.metadata().unwrap().len() / RECORD_SIZE` to count records in a file.
- `SeekFrom::Current(0)` returns the current position without moving — useful for debugging.
- `OpenOptions` is the Swiss army knife of file opening. Learn its options: `read`, `write`, `append`, `create`, `truncate`, `create_new`.

## Compiler Error Interpretation

This kata has **no compiler error** — the broken code compiles and runs without panicking. It produces wrong output silently.

When the broken code runs, it prints something like:

```
Record 3: id=2748779069440, value=0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003
```

The values are garbage because the 16 bytes starting at offset 3 do not align with any record boundary. The `u64` and `f64` decodings succeed (every 8-byte sequence is a valid `u64`, and almost every 8-byte sequence is a valid `f64`), but the results are meaningless.

**This is the hardest kind of bug to find:** no crash, no error, just subtly wrong results. The defense is:
1. Always compute offsets with `index * RECORD_SIZE`
2. Assert/validate after reading: does the `id` match what you expected?
3. Encapsulate seek logic in a function so the multiplication cannot be forgotten
