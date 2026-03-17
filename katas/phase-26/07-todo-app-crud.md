---
id: todo-app-crud
phase: 26
phase_title: "File I/O in Rust"
sequence: 7
title: "Todo App: CRUD Operations"
hints:
  - "`File::create` truncates the file — every existing record is destroyed. You need a different way to open for writing."
  - "Use `OpenOptions::new().read(true).write(true).open(path)` for in-place updates without truncation."
  - "A soft-delete flag (`is_deleted: bool`) lets you mark records as removed without shifting other records. Zeroing bytes creates ghost records."
---

## Description

With the fixed-size record layout from the previous kata, we can now implement full **CRUD** (Create, Read, Update, Delete) operations:

- **Create** — Append a new record to the end of the file
- **Read** — Seek to a record by index and deserialize it
- **Update** — Seek to a record and overwrite it in place
- **Delete** — Mark a record as deleted (soft delete)

Each operation requires the correct `OpenOptions` combination. Getting this wrong can silently destroy data.

## Broken Code

```rust
use std::fs::{File, OpenOptions};
use std::io::{Write, Read, Seek, SeekFrom};

const TITLE_LEN: usize = 64;
const RECORD_SIZE: usize = 4 + TITLE_LEN + 1 + 1; // id + title + completed + deleted = 70

struct Todo {
    id: u32,
    title: [u8; TITLE_LEN],
    completed: bool,
    is_deleted: bool,
}

impl Todo {
    fn new(id: u32, title: &str, completed: bool) -> Self {
        let mut title_bytes = [0u8; TITLE_LEN];
        let len = title.as_bytes().len().min(TITLE_LEN);
        title_bytes[..len].copy_from_slice(&title.as_bytes()[..len]);
        Todo { id, title: title_bytes, completed, is_deleted: false }
    }
    fn title_str(&self) -> &str {
        let end = self.title.iter().position(|&b| b == 0).unwrap_or(TITLE_LEN);
        std::str::from_utf8(&self.title[..end]).unwrap_or("<invalid>")
    }
    fn to_bytes(&self) -> [u8; RECORD_SIZE] {
        let mut buf = [0u8; RECORD_SIZE];
        buf[0..4].copy_from_slice(&self.id.to_le_bytes());
        buf[4..4 + TITLE_LEN].copy_from_slice(&self.title);
        buf[4 + TITLE_LEN] = self.completed as u8;
        buf[4 + TITLE_LEN + 1] = self.is_deleted as u8;
        buf
    }
    fn from_bytes(bytes: &[u8; RECORD_SIZE]) -> Self {
        let id = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let mut title = [0u8; TITLE_LEN];
        title.copy_from_slice(&bytes[4..4 + TITLE_LEN]);
        let completed = bytes[4 + TITLE_LEN] != 0;
        let is_deleted = bytes[4 + TITLE_LEN + 1] != 0;
        Todo { id, title, completed, is_deleted }
    }
}

fn add_todo(path: &str, todo: &Todo) {
    let mut file = OpenOptions::new().create(true).append(true).open(path).unwrap();
    file.write_all(&todo.to_bytes()).unwrap();
}

fn update_todo(path: &str, index: usize, todo: &Todo) {
    // BUG: File::create truncates the file — all records are destroyed!
    let mut file = File::create(path).unwrap();
    let offset = (index * RECORD_SIZE) as u64;
    file.seek(SeekFrom::Start(offset)).unwrap();
    file.write_all(&todo.to_bytes()).unwrap();
}

fn delete_todo(path: &str, index: usize) {
    // BUG: writes zeros instead of setting is_deleted flag — ghost record
    let mut file = OpenOptions::new().write(true).open(path).unwrap();
    let offset = (index * RECORD_SIZE) as u64;
    file.seek(SeekFrom::Start(offset)).unwrap();
    file.write_all(&[0u8; RECORD_SIZE]).unwrap();
}

fn main() {
    let path_buf = std::env::temp_dir().join("kata-todo-crud-broken.bin");
    let path = path_buf.to_str().unwrap();

    // Remove any leftover file
    let _ = std::fs::remove_file(path);

    // Create 3 todos
    add_todo(path, &Todo::new(1, "Buy groceries", false));
    add_todo(path, &Todo::new(2, "Clean the house", false));
    add_todo(path, &Todo::new(3, "Write Rust katas", false));
    println!("Created 3 todos");

    // Try to update todo at index 1
    let updated = Todo::new(2, "Clean the house", true);
    update_todo(path, 1, &updated);

    // Check what happened
    let size = std::fs::metadata(path).unwrap().len();
    println!("File size after update: {} bytes", size);
    println!("Expected: {} bytes (3 records)", 3 * RECORD_SIZE);
    println!("File::create truncated the file! Data is lost.");

    std::fs::remove_file(path).unwrap();
}
```

## Correct Code

```rust
use std::fs::{File, OpenOptions};
use std::io::{Write, Read, Seek, SeekFrom};

const TITLE_LEN: usize = 64;
const RECORD_SIZE: usize = 4 + TITLE_LEN + 1 + 1; // id + title + completed + deleted = 70

#[derive(Debug)]
struct Todo {
    id: u32,
    title: [u8; TITLE_LEN],
    completed: bool,
    is_deleted: bool,
}

impl Todo {
    fn new(id: u32, title: &str, completed: bool) -> Self {
        let mut title_bytes = [0u8; TITLE_LEN];
        let len = title.as_bytes().len().min(TITLE_LEN);
        title_bytes[..len].copy_from_slice(&title.as_bytes()[..len]);
        Todo { id, title: title_bytes, completed, is_deleted: false }
    }

    fn title_str(&self) -> &str {
        let end = self.title.iter().position(|&b| b == 0).unwrap_or(TITLE_LEN);
        std::str::from_utf8(&self.title[..end]).unwrap_or("<invalid>")
    }

    fn to_bytes(&self) -> [u8; RECORD_SIZE] {
        let mut buf = [0u8; RECORD_SIZE];
        buf[0..4].copy_from_slice(&self.id.to_le_bytes());
        buf[4..4 + TITLE_LEN].copy_from_slice(&self.title);
        buf[4 + TITLE_LEN] = self.completed as u8;
        buf[4 + TITLE_LEN + 1] = self.is_deleted as u8;
        buf
    }

    fn from_bytes(bytes: &[u8; RECORD_SIZE]) -> Self {
        let id = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let mut title = [0u8; TITLE_LEN];
        title.copy_from_slice(&bytes[4..4 + TITLE_LEN]);
        let completed = bytes[4 + TITLE_LEN] != 0;
        let is_deleted = bytes[4 + TITLE_LEN + 1] != 0;
        Todo { id, title, completed, is_deleted }
    }
}

// CREATE: append a new record to the file
fn add_todo(path: &str, todo: &Todo) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();
    file.write_all(&todo.to_bytes()).unwrap();
}

// READ: read a record by index
fn read_todo(path: &str, index: usize) -> Todo {
    let mut file = File::open(path).unwrap();
    let offset = (index * RECORD_SIZE) as u64;
    file.seek(SeekFrom::Start(offset)).unwrap();
    let mut buf = [0u8; RECORD_SIZE];
    file.read_exact(&mut buf).unwrap();
    Todo::from_bytes(&buf)
}

// UPDATE: overwrite a record in place (no truncation!)
fn update_todo(path: &str, index: usize, todo: &Todo) {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();
    let offset = (index * RECORD_SIZE) as u64;
    file.seek(SeekFrom::Start(offset)).unwrap();
    file.write_all(&todo.to_bytes()).unwrap();
}

// DELETE: soft-delete by setting the is_deleted flag
fn delete_todo(path: &str, index: usize) {
    let mut todo = read_todo(path, index);
    todo.is_deleted = true;
    update_todo(path, index, &todo);
}

// LIST: read all non-deleted records
fn list_todos(path: &str) -> Vec<(usize, Todo)> {
    let mut file = File::open(path).unwrap();
    let file_size = file.metadata().unwrap().len() as usize;
    let num_records = file_size / RECORD_SIZE;

    let mut todos = Vec::new();
    for i in 0..num_records {
        let mut buf = [0u8; RECORD_SIZE];
        file.read_exact(&mut buf).unwrap();
        let todo = Todo::from_bytes(&buf);
        if !todo.is_deleted {
            todos.push((i, todo));
        }
    }
    todos
}

fn print_todos(path: &str) {
    let todos = list_todos(path);
    if todos.is_empty() {
        println!("  (no todos)");
    }
    for (idx, todo) in &todos {
        let status = if todo.completed { "x" } else { " " };
        println!("  [{}] #{}: {} [{}]", idx, todo.id, todo.title_str(), status);
    }
}

fn main() {
    let path_buf = std::env::temp_dir().join("kata-todo-crud.bin");
    let path = path_buf.to_str().unwrap();

    // Clean slate
    let _ = std::fs::remove_file(path);

    // CREATE — add 4 todos
    add_todo(path, &Todo::new(1, "Buy groceries", false));
    add_todo(path, &Todo::new(2, "Clean the house", false));
    add_todo(path, &Todo::new(3, "Write Rust katas", false));
    add_todo(path, &Todo::new(4, "Go for a run", false));
    println!("After adding 4 todos:");
    print_todos(path);

    // UPDATE — mark todo at index 2 as completed
    let mut todo = read_todo(path, 2);
    todo.completed = true;
    update_todo(path, 2, &todo);
    println!("\nAfter completing 'Write Rust katas':");
    print_todos(path);

    // Verify file size is unchanged (update, not append)
    let size = std::fs::metadata(path).unwrap().len();
    println!("\nFile size: {} bytes ({} records)", size, size as usize / RECORD_SIZE);

    // DELETE — soft-delete todo at index 0
    delete_todo(path, 0);
    println!("\nAfter deleting 'Buy groceries':");
    print_todos(path);

    // The file still has 4 records, but one is marked deleted
    let size = std::fs::metadata(path).unwrap().len();
    println!("\nFile still has {} records (1 soft-deleted)", size as usize / RECORD_SIZE);

    // READ — read a specific todo
    let todo = read_todo(path, 3);
    println!("\nDirect read of index 3: '{}' (deleted: {})", todo.title_str(), todo.is_deleted);

    std::fs::remove_file(path).unwrap();
}
```

## Explanation

The broken code has two critical bugs:

**Bug 1: `File::create` in `update_todo` truncates the file.**

```rust
let mut file = File::create(path).unwrap(); // DESTROYS all data!
```

`File::create` opens the file for writing and **truncates it to zero bytes**. Every existing record is gone. The seek and write that follow only create a small file with one record preceded by zero bytes.

The fix uses `OpenOptions::new().read(true).write(true).open(path)` — this opens the file with both read and write access **without truncation**.

**Bug 2: `delete_todo` writes zeros instead of using a deletion flag.**

```rust
file.write_all(&[0u8; RECORD_SIZE]).unwrap(); // Ghost record!
```

This creates a record with `id=0`, empty title, `completed=false`, `is_deleted=false` — a ghost record that appears in listings as a valid (empty) todo. The data is not deleted; it is corrupted into something that looks real.

The fix uses **soft delete**: read the existing record, set `is_deleted = true`, and write it back. The listing function filters out deleted records.

**OpenOptions cheat sheet for CRUD:**

| Operation | OpenOptions |
|---|---|
| Create (append) | `.create(true).append(true)` |
| Read | `File::open(path)` (read-only) |
| Update (in-place) | `.read(true).write(true)` |
| Delete (soft) | Same as Update — read, modify flag, write back |

**Soft delete vs hard delete:**

- **Soft delete** (flag): Simple, O(1), preserves indices. Wastes space over time.
- **Hard delete** (compaction): Reclaims space but shifts all subsequent records, invalidating indices. O(n) operation.

For a file-backed store, soft delete is almost always the right first choice. Add compaction later if space becomes an issue.

## ⚠️ Caution

- `File::create` is the most dangerous function for existing files. It silently destroys all contents. Always use `OpenOptions` for files you want to modify.
- Soft-deleted records still occupy space. Over many delete/create cycles, the file grows unbounded. Production systems need a compaction strategy.
- If the process crashes mid-write, a record may be partially written. For robustness, consider writing to a temporary file and renaming (atomic on most filesystems).

## 💡 Tips

- Keep each CRUD function focused on one operation with the minimum `OpenOptions` needed. This prevents accidentally writing when you only need to read.
- The pattern of read → modify → write back (for updates and soft deletes) is extremely common in file-backed storage.
- For the next evolution, consider auto-incrementing IDs: read the last record's ID and add 1, or store a counter in a file header.
- Soft delete + periodic compaction is exactly how databases like PostgreSQL work internally (MVCC + VACUUM).

## Compiler Error Interpretation

The broken code compiles and runs — the errors are **runtime data loss** and **logical corruption**.

When the broken `update_todo` runs:

```
File size after update: 140 bytes
Expected: 210 bytes (3 records)
File::create truncated the file! Data is lost.
```

The file should contain 3 records (210 bytes), but `File::create` wiped it and the subsequent seek+write only produced a partial file. Two of three records are permanently gone.

This is **silent data destruction**. The program does not crash — it happily writes to the truncated file. The only way to notice is by checking the file size or reading back the data.

The lesson: **understand what every `OpenOptions` flag does before opening a file for writing.** `File::create` = `OpenOptions::new().write(true).create(true).truncate(true)`. That `truncate(true)` is the silent killer.
