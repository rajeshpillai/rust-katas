---
id: todo-app-record-layout
phase: 26
phase_title: "File I/O in Rust"
sequence: 6
title: "Todo App: Record Layout and File Storage"
hints:
  - "Random access requires every record to be the same size. A `String` field has variable length."
  - "To make a string field fixed-size, use a byte array like `[u8; 64]` and pad with zeros."
  - "Define `to_bytes()` and `from_bytes()` methods that serialize each field to a fixed number of bytes. The total must be constant."
---

## Description

This is the first kata in a three-part series building a **file-backed todo application** using random access. Before we can create, read, update, or delete records, we need a **fixed-size binary record format**.

The challenge: a todo item has a text title, which is variable-length. But random access requires every record to occupy exactly the same number of bytes. The solution is to **bound the string** to a maximum length and pad with null bytes.

This kata teaches the design pattern that databases, filesystems, and embedded systems use to store structured data in flat files.

## Broken Code

```rust
use std::fs::File;
use std::io::{Write, Read, Seek, SeekFrom};

struct Todo {
    id: u32,
    title: String,
    completed: bool,
}

fn write_todo(file: &mut File, todo: &Todo) {
    file.write_all(&todo.id.to_le_bytes()).unwrap();
    file.write_all(todo.title.as_bytes()).unwrap(); // Variable length!
    file.write_all(&[todo.completed as u8]).unwrap();
}

fn main() {
    let path = std::env::temp_dir().join("kata-todo-layout.bin");
    let mut file = File::create(&path).unwrap();

    write_todo(&mut file, &Todo {
        id: 1, title: "Buy milk".into(), completed: false,
    });
    write_todo(&mut file, &Todo {
        id: 2, title: "Write Rust code".into(), completed: true,
    });
    write_todo(&mut file, &Todo {
        id: 3, title: "Go".into(), completed: false,
    });

    let size = std::fs::metadata(&path).unwrap().len();
    println!("File size: {} bytes", size);
    println!("Expected 3 equal records, but:");
    println!("  'Buy milk' = {} title bytes", "Buy milk".len());
    println!("  'Write Rust code' = {} title bytes", "Write Rust code".len());
    println!("  'Go' = {} title bytes", "Go".len());
    println!("Records have different sizes — cannot seek to record N!");

    std::fs::remove_file(&path).unwrap();
}
```

## Correct Code

```rust
use std::fs::File;
use std::io::{Write, Read, Seek, SeekFrom};

const TITLE_LEN: usize = 64;
// Layout: id (4) + title (64) + completed (1) = 69 bytes
const RECORD_SIZE: usize = 4 + TITLE_LEN + 1;

#[derive(Debug)]
struct Todo {
    id: u32,
    title: [u8; TITLE_LEN],
    completed: bool,
}

impl Todo {
    fn new(id: u32, title: &str, completed: bool) -> Self {
        let mut title_bytes = [0u8; TITLE_LEN];
        let bytes = title.as_bytes();
        let len = bytes.len().min(TITLE_LEN);
        title_bytes[..len].copy_from_slice(&bytes[..len]);
        Todo { id, title: title_bytes, completed }
    }

    fn title_str(&self) -> &str {
        let end = self.title.iter().position(|&b| b == 0).unwrap_or(TITLE_LEN);
        std::str::from_utf8(&self.title[..end]).unwrap_or("<invalid utf8>")
    }

    fn to_bytes(&self) -> [u8; RECORD_SIZE] {
        let mut buf = [0u8; RECORD_SIZE];
        buf[0..4].copy_from_slice(&self.id.to_le_bytes());
        buf[4..4 + TITLE_LEN].copy_from_slice(&self.title);
        buf[4 + TITLE_LEN] = self.completed as u8;
        buf
    }

    fn from_bytes(bytes: &[u8; RECORD_SIZE]) -> Self {
        let id = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let mut title = [0u8; TITLE_LEN];
        title.copy_from_slice(&bytes[4..4 + TITLE_LEN]);
        let completed = bytes[4 + TITLE_LEN] != 0;
        Todo { id, title, completed }
    }
}

fn write_todo(file: &mut File, todo: &Todo) {
    file.write_all(&todo.to_bytes()).unwrap();
}

fn read_todo(file: &mut File) -> Todo {
    let mut buf = [0u8; RECORD_SIZE];
    file.read_exact(&mut buf).unwrap();
    Todo::from_bytes(&buf)
}

fn main() {
    let path = std::env::temp_dir().join("kata-todo-layout.bin");
    let mut file = File::create(&path).unwrap();

    // Write 3 todos — each is exactly RECORD_SIZE bytes
    write_todo(&mut file, &Todo::new(1, "Buy milk", false));
    write_todo(&mut file, &Todo::new(2, "Write Rust code", true));
    write_todo(&mut file, &Todo::new(3, "Go for a walk", false));

    let size = std::fs::metadata(&path).unwrap().len();
    println!("File size: {} bytes", size);
    println!("Record size: {} bytes", RECORD_SIZE);
    println!("Records: {}", size as usize / RECORD_SIZE);

    // Random access: read record at index 1 (second todo)
    let mut file = File::open(&path).unwrap();
    let offset = 1 * RECORD_SIZE as u64;
    file.seek(SeekFrom::Start(offset)).unwrap();
    let todo = read_todo(&mut file);
    println!("\nRecord 1: id={}, title='{}', completed={}", todo.id, todo.title_str(), todo.completed);

    // Read all records
    file.seek(SeekFrom::Start(0)).unwrap();
    println!("\nAll todos:");
    for i in 0..(size as usize / RECORD_SIZE) {
        let todo = read_todo(&mut file);
        let status = if todo.completed { "done" } else { "pending" };
        println!("  [{}] #{}: {} ({})", i, todo.id, todo.title_str(), status);
    }

    // Demonstrate title truncation
    let long_title = "This is a very long title that exceeds the sixty-four byte limit we set for titles";
    let todo = Todo::new(99, long_title, false);
    println!("\nTruncated title: '{}'", todo.title_str());
    println!("Original was {} bytes, stored {} bytes", long_title.len(), todo.title_str().len());

    std::fs::remove_file(&path).unwrap();
}
```

## Explanation

The broken code writes `todo.title.as_bytes()` directly — a slice whose length depends on the title string. The three records occupy different numbers of bytes:

- Record 1: `4 + 8 + 1 = 13` bytes ("Buy milk" is 8 bytes)
- Record 2: `4 + 15 + 1 = 20` bytes ("Write Rust code" is 15 bytes)
- Record 3: `4 + 2 + 1 = 7` bytes ("Go" is 2 bytes)

Total: 40 bytes for 3 records of different sizes. There is no way to compute "record N starts at byte X" because each record has a different size. **Random access is impossible.**

**The fix: bounded, fixed-size fields.**

The title is stored as `[u8; 64]` — always exactly 64 bytes. Short titles are padded with null bytes (`0x00`). Long titles are truncated. This guarantees every record is exactly `RECORD_SIZE` bytes.

```
Record layout (69 bytes total):
┌──────────┬──────────────────────────┬───────────┐
│ id (4B)  │ title (64B, null-padded) │ done (1B) │
└──────────┴──────────────────────────┴───────────┘
```

**Key design decisions:**

| Decision | Trade-off |
|---|---|
| Fixed 64-byte title | Wastes space on short titles, truncates long ones |
| Null-byte padding | Simple to implement, easy to find string end |
| `RECORD_SIZE` constant | Single source of truth for all seek calculations |
| `to_bytes()` / `from_bytes()` | Encapsulates serialization, making the format easy to change |

**This is how real systems work.** Traditional database engines (SQLite's table format), filesystems (ext4 directory entries), and network protocols (DNS packets) all use fixed-size fields with bounded strings. The trade-off between space efficiency and random-access capability is fundamental.

## ⚠️ Caution

- Title truncation is silent. If a user enters a title longer than 64 bytes, data is lost without warning. In production, validate and reject or warn.
- UTF-8 strings can use multiple bytes per character. Truncating at byte 64 might cut a multi-byte character in half, producing invalid UTF-8. The `title_str()` method handles this with `unwrap_or`, but a robust implementation should truncate at a character boundary.
- The `RECORD_SIZE` constant must exactly match the sum of field sizes. If you add a field and forget to update it, seeks will be misaligned.

## 💡 Tips

- Derive `RECORD_SIZE` from field sizes: `const RECORD_SIZE: usize = 4 + TITLE_LEN + 1;` — never hardcode the total.
- Consider adding a version byte at the start of the file (not per record) to handle format migrations.
- For UTF-8 safety, truncate at the last valid character boundary before the byte limit. The `str::floor_char_boundary` method (nightly) or a manual scan can help.
- This pattern (fixed-size struct with `to_bytes`/`from_bytes`) is a building block you will reuse in the next two katas.

## Compiler Error Interpretation

The broken code compiles and runs — this is a **design-level** error, not a compiler or runtime error.

The output shows:

```
File size: 40 bytes
Expected 3 equal records, but:
  'Buy milk' = 8 title bytes
  'Write Rust code' = 15 title bytes
  'Go' = 2 title bytes
Records have different sizes — cannot seek to record N!
```

The program demonstrates its own flaw: three records that should be identically sized occupy 13, 20, and 7 bytes respectively. The fundamental invariant of random access — **all records have the same size** — is violated.

This is not a bug the compiler can catch. It is an architectural mistake. The compiler enforces memory safety, but data layout correctness is your responsibility. The lesson: **design your record format before writing a single line of I/O code.**

---

| [Prev: Random Access with Seek](#/katas/random-access-seek) | [Next: Todo App: CRUD Operations](#/katas/todo-app-crud) |
