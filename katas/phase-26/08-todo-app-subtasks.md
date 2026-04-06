---
id: todo-app-subtasks
phase: 26
phase_title: "File I/O in Rust"
sequence: 8
title: "Todo App: Subtasks and Record Relationships"
hints:
  - "A subtask references its parent by `parent_id`. When the parent is deleted, what happens to its subtasks?"
  - "Deleting a parent without deleting its subtasks creates orphans — data that references something that no longer exists."
  - "Implement cascading delete: when a parent is soft-deleted, iterate through subtasks and soft-delete all that match the parent's ID."
---

## Description

This capstone kata extends the todo app with **subtasks** — child records that reference a parent task by ID. This introduces **record relationships** in flat binary files, teaching the same concepts that underpin relational databases.

The key challenge: **referential integrity**. When you delete a parent task, its subtasks become orphans — dangling references in your data. This mirrors Rust's ownership model at the data layer: dangling references are bugs whether they are pointers or record IDs.

## Broken Code

```rust
use std::fs::{File, OpenOptions};
use std::io::{Write, Read, Seek, SeekFrom};

const TITLE_LEN: usize = 64;
const TODO_SIZE: usize = 4 + TITLE_LEN + 1 + 1; // id(4) + title(64) + completed(1) + deleted(1) = 70
const SUBTASK_SIZE: usize = 4 + 4 + TITLE_LEN + 1 + 1; // id(4) + parent_id(4) + title(64) + completed(1) + deleted(1) = 74

struct Todo {
    id: u32,
    title: [u8; TITLE_LEN],
    completed: bool,
    is_deleted: bool,
}

struct Subtask {
    id: u32,
    parent_id: u32,
    title: [u8; TITLE_LEN],
    completed: bool,
    is_deleted: bool,
}

impl Todo {
    fn new(id: u32, title: &str) -> Self {
        let mut t = [0u8; TITLE_LEN];
        let len = title.as_bytes().len().min(TITLE_LEN);
        t[..len].copy_from_slice(&title.as_bytes()[..len]);
        Todo { id, title: t, completed: false, is_deleted: false }
    }
    fn title_str(&self) -> &str {
        let end = self.title.iter().position(|&b| b == 0).unwrap_or(TITLE_LEN);
        std::str::from_utf8(&self.title[..end]).unwrap_or("<invalid>")
    }
    fn to_bytes(&self) -> [u8; TODO_SIZE] {
        let mut buf = [0u8; TODO_SIZE];
        buf[0..4].copy_from_slice(&self.id.to_le_bytes());
        buf[4..4 + TITLE_LEN].copy_from_slice(&self.title);
        buf[4 + TITLE_LEN] = self.completed as u8;
        buf[4 + TITLE_LEN + 1] = self.is_deleted as u8;
        buf
    }
    fn from_bytes(bytes: &[u8; TODO_SIZE]) -> Self {
        let id = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let mut title = [0u8; TITLE_LEN];
        title.copy_from_slice(&bytes[4..4 + TITLE_LEN]);
        let completed = bytes[4 + TITLE_LEN] != 0;
        let is_deleted = bytes[4 + TITLE_LEN + 1] != 0;
        Todo { id, title, completed, is_deleted }
    }
}

impl Subtask {
    fn new(id: u32, parent_id: u32, title: &str) -> Self {
        let mut t = [0u8; TITLE_LEN];
        let len = title.as_bytes().len().min(TITLE_LEN);
        t[..len].copy_from_slice(&title.as_bytes()[..len]);
        Subtask { id, parent_id, title: t, completed: false, is_deleted: false }
    }
    fn title_str(&self) -> &str {
        let end = self.title.iter().position(|&b| b == 0).unwrap_or(TITLE_LEN);
        std::str::from_utf8(&self.title[..end]).unwrap_or("<invalid>")
    }
    fn to_bytes(&self) -> [u8; SUBTASK_SIZE] {
        let mut buf = [0u8; SUBTASK_SIZE];
        buf[0..4].copy_from_slice(&self.id.to_le_bytes());
        buf[4..8].copy_from_slice(&self.parent_id.to_le_bytes());
        buf[8..8 + TITLE_LEN].copy_from_slice(&self.title);
        buf[8 + TITLE_LEN] = self.completed as u8;
        buf[8 + TITLE_LEN + 1] = self.is_deleted as u8;
        buf
    }
    fn from_bytes(bytes: &[u8; SUBTASK_SIZE]) -> Self {
        let id = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let parent_id = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let mut title = [0u8; TITLE_LEN];
        title.copy_from_slice(&bytes[8..8 + TITLE_LEN]);
        let completed = bytes[8 + TITLE_LEN] != 0;
        let is_deleted = bytes[8 + TITLE_LEN + 1] != 0;
        Subtask { id, parent_id, title, completed, is_deleted }
    }
}

fn append_todo(path: &str, todo: &Todo) {
    let mut f = OpenOptions::new().create(true).append(true).open(path).unwrap();
    f.write_all(&todo.to_bytes()).unwrap();
}

fn append_subtask(path: &str, sub: &Subtask) {
    let mut f = OpenOptions::new().create(true).append(true).open(path).unwrap();
    f.write_all(&sub.to_bytes()).unwrap();
}

fn delete_task(todo_path: &str, index: usize) {
    // BUG: Only deletes the task — subtasks become orphans!
    let mut file = OpenOptions::new().read(true).write(true).open(todo_path).unwrap();
    let offset = (index * TODO_SIZE) as u64;
    file.seek(SeekFrom::Start(offset)).unwrap();
    let mut buf = [0u8; TODO_SIZE];
    file.read_exact(&mut buf).unwrap();
    let mut todo = Todo::from_bytes(&buf);
    todo.is_deleted = true;
    file.seek(SeekFrom::Start(offset)).unwrap();
    file.write_all(&todo.to_bytes()).unwrap();
    // No cascade to subtasks!
}

fn main() {
    let todo_path_buf = std::env::temp_dir().join("kata-subtask-tasks-broken.bin");
    let sub_path_buf = std::env::temp_dir().join("kata-subtask-subs-broken.bin");
    let todo_path = todo_path_buf.to_str().unwrap();
    let sub_path = sub_path_buf.to_str().unwrap();
    let _ = std::fs::remove_file(todo_path);
    let _ = std::fs::remove_file(sub_path);

    // Create tasks
    append_todo(todo_path, &Todo::new(1, "Plan vacation"));
    append_todo(todo_path, &Todo::new(2, "Home renovation"));

    // Create subtasks for task 1
    append_subtask(sub_path, &Subtask::new(1, 1, "Book flights"));
    append_subtask(sub_path, &Subtask::new(2, 1, "Reserve hotel"));
    // Subtask for task 2
    append_subtask(sub_path, &Subtask::new(3, 2, "Get paint samples"));

    // Delete task 1 (Plan vacation)
    delete_task(todo_path, 0);

    // List subtasks — orphans for deleted task 1 are still visible!
    let mut file = File::open(sub_path).unwrap();
    let size = file.metadata().unwrap().len() as usize;
    println!("Subtasks after deleting task 1:");
    for _ in 0..(size / SUBTASK_SIZE) {
        let mut buf = [0u8; SUBTASK_SIZE];
        file.read_exact(&mut buf).unwrap();
        let sub = Subtask::from_bytes(&buf);
        if !sub.is_deleted {
            println!("  #{} (parent={}): {}", sub.id, sub.parent_id, sub.title_str());
        }
    }
    println!("Bug: subtasks for parent_id=1 still exist but parent is deleted!");

    let _ = std::fs::remove_file(todo_path);
    let _ = std::fs::remove_file(sub_path);
}
```

## Correct Code

```rust
use std::fs::{File, OpenOptions};
use std::io::{Write, Read, Seek, SeekFrom};

const TITLE_LEN: usize = 64;
const TODO_SIZE: usize = 4 + TITLE_LEN + 1 + 1;
const SUBTASK_SIZE: usize = 4 + 4 + TITLE_LEN + 1 + 1;

#[derive(Debug)]
struct Todo {
    id: u32,
    title: [u8; TITLE_LEN],
    completed: bool,
    is_deleted: bool,
}

#[derive(Debug)]
struct Subtask {
    id: u32,
    parent_id: u32,
    title: [u8; TITLE_LEN],
    completed: bool,
    is_deleted: bool,
}

impl Todo {
    fn new(id: u32, title: &str) -> Self {
        let mut t = [0u8; TITLE_LEN];
        let len = title.as_bytes().len().min(TITLE_LEN);
        t[..len].copy_from_slice(&title.as_bytes()[..len]);
        Todo { id, title: t, completed: false, is_deleted: false }
    }
    fn title_str(&self) -> &str {
        let end = self.title.iter().position(|&b| b == 0).unwrap_or(TITLE_LEN);
        std::str::from_utf8(&self.title[..end]).unwrap_or("<invalid>")
    }
    fn to_bytes(&self) -> [u8; TODO_SIZE] {
        let mut buf = [0u8; TODO_SIZE];
        buf[0..4].copy_from_slice(&self.id.to_le_bytes());
        buf[4..4 + TITLE_LEN].copy_from_slice(&self.title);
        buf[4 + TITLE_LEN] = self.completed as u8;
        buf[4 + TITLE_LEN + 1] = self.is_deleted as u8;
        buf
    }
    fn from_bytes(bytes: &[u8; TODO_SIZE]) -> Self {
        let id = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let mut title = [0u8; TITLE_LEN];
        title.copy_from_slice(&bytes[4..4 + TITLE_LEN]);
        let completed = bytes[4 + TITLE_LEN] != 0;
        let is_deleted = bytes[4 + TITLE_LEN + 1] != 0;
        Todo { id, title, completed, is_deleted }
    }
}

impl Subtask {
    fn new(id: u32, parent_id: u32, title: &str) -> Self {
        let mut t = [0u8; TITLE_LEN];
        let len = title.as_bytes().len().min(TITLE_LEN);
        t[..len].copy_from_slice(&title.as_bytes()[..len]);
        Subtask { id, parent_id, title: t, completed: false, is_deleted: false }
    }
    fn title_str(&self) -> &str {
        let end = self.title.iter().position(|&b| b == 0).unwrap_or(TITLE_LEN);
        std::str::from_utf8(&self.title[..end]).unwrap_or("<invalid>")
    }
    fn to_bytes(&self) -> [u8; SUBTASK_SIZE] {
        let mut buf = [0u8; SUBTASK_SIZE];
        buf[0..4].copy_from_slice(&self.id.to_le_bytes());
        buf[4..8].copy_from_slice(&self.parent_id.to_le_bytes());
        buf[8..8 + TITLE_LEN].copy_from_slice(&self.title);
        buf[8 + TITLE_LEN] = self.completed as u8;
        buf[8 + TITLE_LEN + 1] = self.is_deleted as u8;
        buf
    }
    fn from_bytes(bytes: &[u8; SUBTASK_SIZE]) -> Self {
        let id = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let parent_id = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let mut title = [0u8; TITLE_LEN];
        title.copy_from_slice(&bytes[8..8 + TITLE_LEN]);
        let completed = bytes[8 + TITLE_LEN] != 0;
        let is_deleted = bytes[8 + TITLE_LEN + 1] != 0;
        Subtask { id, parent_id, title, completed, is_deleted }
    }
}

// --- File helpers ---

fn append_todo(path: &str, todo: &Todo) {
    let mut f = OpenOptions::new().create(true).append(true).open(path).unwrap();
    f.write_all(&todo.to_bytes()).unwrap();
}

fn append_subtask(path: &str, sub: &Subtask) {
    let mut f = OpenOptions::new().create(true).append(true).open(path).unwrap();
    f.write_all(&sub.to_bytes()).unwrap();
}

fn read_all_todos(path: &str) -> Vec<(usize, Todo)> {
    let mut file = File::open(path).unwrap();
    let size = file.metadata().unwrap().len() as usize;
    let mut todos = Vec::new();
    for i in 0..(size / TODO_SIZE) {
        let mut buf = [0u8; TODO_SIZE];
        file.read_exact(&mut buf).unwrap();
        todos.push((i, Todo::from_bytes(&buf)));
    }
    todos
}

fn read_all_subtasks(path: &str) -> Vec<(usize, Subtask)> {
    let Ok(mut file) = File::open(path) else { return Vec::new(); };
    let size = file.metadata().unwrap().len() as usize;
    let mut subs = Vec::new();
    for i in 0..(size / SUBTASK_SIZE) {
        let mut buf = [0u8; SUBTASK_SIZE];
        file.read_exact(&mut buf).unwrap();
        subs.push((i, Subtask::from_bytes(&buf)));
    }
    subs
}

// Verify parent exists before adding a subtask
fn add_subtask_checked(todo_path: &str, sub_path: &str, sub: &Subtask) -> Result<(), String> {
    let todos = read_all_todos(todo_path);
    let parent_exists = todos.iter().any(|(_, t)| t.id == sub.parent_id && !t.is_deleted);
    if !parent_exists {
        return Err(format!("Parent task with id {} does not exist or is deleted", sub.parent_id));
    }
    append_subtask(sub_path, sub);
    Ok(())
}

// Cascading delete: delete task AND all its subtasks
fn delete_task_cascade(todo_path: &str, sub_path: &str, task_index: usize) {
    // Step 1: Read the task to get its ID
    let mut file = OpenOptions::new().read(true).write(true).open(todo_path).unwrap();
    let offset = (task_index * TODO_SIZE) as u64;
    file.seek(SeekFrom::Start(offset)).unwrap();
    let mut buf = [0u8; TODO_SIZE];
    file.read_exact(&mut buf).unwrap();
    let mut todo = Todo::from_bytes(&buf);
    let task_id = todo.id;

    // Step 2: Soft-delete the task
    todo.is_deleted = true;
    file.seek(SeekFrom::Start(offset)).unwrap();
    file.write_all(&todo.to_bytes()).unwrap();

    // Step 3: Cascade — soft-delete all subtasks with this parent_id
    if let Ok(mut sub_file) = OpenOptions::new().read(true).write(true).open(sub_path) {
        let size = sub_file.metadata().unwrap().len() as usize;
        let num_subs = size / SUBTASK_SIZE;

        for i in 0..num_subs {
            let offset = (i * SUBTASK_SIZE) as u64;
            sub_file.seek(SeekFrom::Start(offset)).unwrap();
            let mut buf = [0u8; SUBTASK_SIZE];
            sub_file.read_exact(&mut buf).unwrap();
            let mut sub = Subtask::from_bytes(&buf);

            if sub.parent_id == task_id && !sub.is_deleted {
                sub.is_deleted = true;
                sub_file.seek(SeekFrom::Start(offset)).unwrap();
                sub_file.write_all(&sub.to_bytes()).unwrap();
            }
        }
    }
}

// Check for orphaned subtasks
fn find_orphans(todo_path: &str, sub_path: &str) -> Vec<(usize, Subtask)> {
    let todos = read_all_todos(todo_path);
    let active_ids: Vec<u32> = todos.iter()
        .filter(|(_, t)| !t.is_deleted)
        .map(|(_, t)| t.id)
        .collect();

    read_all_subtasks(sub_path)
        .into_iter()
        .filter(|(_, s)| !s.is_deleted && !active_ids.contains(&s.parent_id))
        .collect()
}

fn print_tree(todo_path: &str, sub_path: &str) {
    let todos = read_all_todos(todo_path);
    let subtasks = read_all_subtasks(sub_path);

    for (idx, todo) in &todos {
        if todo.is_deleted { continue; }
        let status = if todo.completed { "x" } else { " " };
        println!("[{}] #{}: {} [{}]", idx, todo.id, todo.title_str(), status);

        for (_, sub) in &subtasks {
            if sub.is_deleted || sub.parent_id != todo.id { continue; }
            let status = if sub.completed { "x" } else { " " };
            println!("    #{}: {} [{}]", sub.id, sub.title_str(), status);
        }
    }
}

fn main() {
    let todo_path_buf = std::env::temp_dir().join("kata-subtask-tasks.bin");
    let sub_path_buf = std::env::temp_dir().join("kata-subtask-subs.bin");
    let todo_path = todo_path_buf.to_str().unwrap();
    let sub_path = sub_path_buf.to_str().unwrap();
    let _ = std::fs::remove_file(todo_path);
    let _ = std::fs::remove_file(sub_path);

    // Create tasks
    append_todo(todo_path, &Todo::new(1, "Plan vacation"));
    append_todo(todo_path, &Todo::new(2, "Home renovation"));
    append_todo(todo_path, &Todo::new(3, "Learn Rust"));

    // Add subtasks (with parent existence check)
    add_subtask_checked(todo_path, sub_path, &Subtask::new(1, 1, "Book flights")).unwrap();
    add_subtask_checked(todo_path, sub_path, &Subtask::new(2, 1, "Reserve hotel")).unwrap();
    add_subtask_checked(todo_path, sub_path, &Subtask::new(3, 1, "Pack bags")).unwrap();
    add_subtask_checked(todo_path, sub_path, &Subtask::new(4, 2, "Get paint samples")).unwrap();
    add_subtask_checked(todo_path, sub_path, &Subtask::new(5, 2, "Hire contractor")).unwrap();
    add_subtask_checked(todo_path, sub_path, &Subtask::new(6, 3, "Complete Phase 26 katas")).unwrap();

    // Try adding a subtask to a non-existent parent
    let result = add_subtask_checked(todo_path, sub_path, &Subtask::new(7, 99, "Orphan subtask"));
    println!("Adding subtask to non-existent parent: {:?}\n", result);

    println!("=== All tasks with subtasks ===");
    print_tree(todo_path, sub_path);

    // Cascading delete: delete "Plan vacation" and all its subtasks
    println!("\n--- Deleting 'Plan vacation' (cascading) ---\n");
    delete_task_cascade(todo_path, sub_path, 0);

    println!("=== After cascading delete ===");
    print_tree(todo_path, sub_path);

    // Verify: no orphans
    let orphans = find_orphans(todo_path, sub_path);
    println!("\nOrphaned subtasks: {}", orphans.len());

    // Show file statistics
    let todo_size = std::fs::metadata(todo_path).unwrap().len();
    let sub_size = std::fs::metadata(sub_path).unwrap().len();
    println!("\nFile stats:");
    println!("  tasks.bin: {} bytes ({} records, {} active)",
        todo_size, todo_size as usize / TODO_SIZE,
        read_all_todos(todo_path).iter().filter(|(_, t)| !t.is_deleted).count());
    println!("  subtasks.bin: {} bytes ({} records, {} active)",
        sub_size, sub_size as usize / SUBTASK_SIZE,
        read_all_subtasks(sub_path).iter().filter(|(_, s)| !s.is_deleted).count());

    let _ = std::fs::remove_file(todo_path);
    let _ = std::fs::remove_file(sub_path);
}
```

## Explanation

The broken code deletes a parent task but leaves its subtasks untouched. After deleting task 1 ("Plan vacation"), the subtasks "Book flights" and "Reserve hotel" still exist with `parent_id = 1` — but task 1 no longer exists. These are **orphaned records**: data that references something that has been deleted.

This is the **data-layer equivalent of a dangling pointer**. In Rust's ownership model, the compiler prevents dangling references at the memory level. But in a file-based data store, **you** are the compiler — there is no automatic enforcement.

**The correct approach has three layers of defense:**

1. **Cascading delete**: When a parent is deleted, iterate through all subtasks and soft-delete any that reference the deleted parent's ID.

2. **Existence check on create**: Before adding a subtask, verify that the parent task exists and is not deleted. Reject the operation if the parent is missing.

3. **Orphan detection**: A `find_orphans` function scans for subtasks whose parent no longer exists — a consistency check you can run to verify data integrity.

**Two-file design:**

```
tasks.bin:    [Todo][Todo][Todo]...        (70 bytes per record)
subtasks.bin: [Subtask][Subtask][Subtask]... (74 bytes per record)
```

Using separate files for different record types is simpler than mixing record types in one file (which would require a type discriminator byte and variable-size logic). Each file has uniform record sizes, so random access works independently within each file.

**The `parent_id` pattern** is exactly how foreign keys work in relational databases:
- The subtask's `parent_id` references the task's `id`
- `ON DELETE CASCADE` in SQL is equivalent to our `delete_task_cascade` function
- A foreign key constraint is equivalent to our `add_subtask_checked` function

The difference: a database engine enforces these constraints automatically. In a flat file, every constraint is code you must write and maintain.

## ⚠️ Caution

- Cascading deletes must scan the entire subtask file — this is O(n). For large files, consider maintaining an index (a separate file mapping `parent_id` → subtask offsets).
- The two-step cascade (delete parent, then delete children) is not atomic. If the process crashes between steps, you get orphans. Production systems use write-ahead logs or transactions for atomicity.
- IDs are stored in records, not derived from position. If you reuse deleted IDs, old subtasks might accidentally match new parents. Use monotonically increasing IDs.

## 💡 Tips

- Run `find_orphans` as a periodic integrity check, like a database's `VACUUM` or `fsck`.
- The pattern of "two files with a linking ID" scales: you could add tags, comments, or attachments as additional files, all linked by task ID.
- This capstone combines every concept from Phase 26: text-to-bytes conversion, fixed-size records, `Seek`, `OpenOptions`, and error handling. If you understand this kata, you understand file-backed data storage.
- Real-world applications of this pattern: SQLite's page-based storage, Git's object database, log-structured storage engines.

## Compiler Error Interpretation

The broken code compiles and runs. The output reveals the design flaw:

```
Subtasks after deleting task 1:
  #1 (parent=1): Book flights
  #2 (parent=1): Reserve hotel
  #3 (parent=2): Get paint samples
Bug: subtasks for parent_id=1 still exist but parent is deleted!
```

Subtasks #1 and #2 reference `parent_id=1`, but task 1 has been deleted. These are orphans. The data is inconsistent — a state that would be impossible in a properly constrained database.

The broader lesson: **data integrity constraints that exist only in your head will eventually be violated.** Write them as code:
- Validate on write (existence checks)
- Cascade on delete
- Verify periodically (orphan detection)

This is the discipline of data correctness — the same discipline Rust applies to memory correctness, but at a layer the compiler cannot reach.

---

| [Prev: Todo App: CRUD Operations](#/katas/todo-app-crud) |  |
