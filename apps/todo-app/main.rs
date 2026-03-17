use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;

// --- Record Layout ---

const TITLE_LEN: usize = 128;
const TODO_SIZE: usize = 4 + TITLE_LEN + 1 + 1; // id(4) + title(128) + completed(1) + deleted(1) = 134
const SUBTASK_SIZE: usize = 4 + 4 + TITLE_LEN + 1 + 1; // id(4) + parent_id(4) + title(128) + completed(1) + deleted(1) = 138

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

    fn set_title(&mut self, title: &str) {
        self.title = [0u8; TITLE_LEN];
        let len = title.as_bytes().len().min(TITLE_LEN);
        self.title[..len].copy_from_slice(&title.as_bytes()[..len]);
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

    fn set_title(&mut self, title: &str) {
        self.title = [0u8; TITLE_LEN];
        let len = title.as_bytes().len().min(TITLE_LEN);
        self.title[..len].copy_from_slice(&title.as_bytes()[..len]);
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

// --- Storage ---

fn data_dir() -> PathBuf {
    let dir = dirs_or_home().join(".rust-todo");
    fs::create_dir_all(&dir).expect("Failed to create data directory");
    dir
}

fn dirs_or_home() -> PathBuf {
    env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().unwrap())
}

fn tasks_path() -> PathBuf {
    data_dir().join("tasks.bin")
}

fn subtasks_path() -> PathBuf {
    data_dir().join("subtasks.bin")
}

fn read_all_todos() -> Vec<Todo> {
    let path = tasks_path();
    let Ok(mut file) = File::open(&path) else { return Vec::new() };
    let size = file.metadata().map(|m| m.len()).unwrap_or(0) as usize;
    let mut todos = Vec::new();
    for _ in 0..(size / TODO_SIZE) {
        let mut buf = [0u8; TODO_SIZE];
        if file.read_exact(&mut buf).is_ok() {
            todos.push(Todo::from_bytes(&buf));
        }
    }
    todos
}

fn read_all_subtasks() -> Vec<Subtask> {
    let path = subtasks_path();
    let Ok(mut file) = File::open(&path) else { return Vec::new() };
    let size = file.metadata().map(|m| m.len()).unwrap_or(0) as usize;
    let mut subs = Vec::new();
    for _ in 0..(size / SUBTASK_SIZE) {
        let mut buf = [0u8; SUBTASK_SIZE];
        if file.read_exact(&mut buf).is_ok() {
            subs.push(Subtask::from_bytes(&buf));
        }
    }
    subs
}

fn next_todo_id() -> u32 {
    read_all_todos()
        .iter()
        .map(|t| t.id)
        .max()
        .unwrap_or(0) + 1
}

fn next_subtask_id() -> u32 {
    read_all_subtasks()
        .iter()
        .map(|s| s.id)
        .max()
        .unwrap_or(0) + 1
}

fn write_todo_at(index: usize, todo: &Todo) {
    let path = tasks_path();
    let mut file = OpenOptions::new()
        .read(true).write(true).create(true)
        .open(&path).unwrap();
    let offset = (index * TODO_SIZE) as u64;
    file.seek(SeekFrom::Start(offset)).unwrap();
    file.write_all(&todo.to_bytes()).unwrap();
}

fn write_subtask_at(index: usize, sub: &Subtask) {
    let path = subtasks_path();
    let mut file = OpenOptions::new()
        .read(true).write(true).create(true)
        .open(&path).unwrap();
    let offset = (index * SUBTASK_SIZE) as u64;
    file.seek(SeekFrom::Start(offset)).unwrap();
    file.write_all(&sub.to_bytes()).unwrap();
}

fn append_todo(todo: &Todo) {
    let path = tasks_path();
    let mut file = OpenOptions::new()
        .create(true).append(true)
        .open(&path).unwrap();
    file.write_all(&todo.to_bytes()).unwrap();
}

fn append_subtask(sub: &Subtask) {
    let path = subtasks_path();
    let mut file = OpenOptions::new()
        .create(true).append(true)
        .open(&path).unwrap();
    file.write_all(&sub.to_bytes()).unwrap();
}

// --- Commands ---

fn cmd_add(title: &str) {
    let id = next_todo_id();
    let todo = Todo::new(id, title);
    append_todo(&todo);
    println!("Added task #{}: {}", id, title);
}

fn cmd_list() {
    let todos = read_all_todos();
    let subtasks = read_all_subtasks();

    let active: Vec<_> = todos.iter().filter(|t| !t.is_deleted).collect();
    if active.is_empty() {
        println!("No tasks. Use 'add' to create one.");
        return;
    }

    for todo in &active {
        let check = if todo.completed { "x" } else { " " };
        let title = todo.title_str();
        if todo.completed {
            println!("  [{}] #{}: \x1b[9m{}\x1b[0m", check, todo.id, title);
        } else {
            println!("  [{}] #{}: {}", check, todo.id, title);
        }

        let subs: Vec<_> = subtasks.iter()
            .filter(|s| s.parent_id == todo.id && !s.is_deleted)
            .collect();
        for sub in &subs {
            let check = if sub.completed { "x" } else { " " };
            let title = sub.title_str();
            if sub.completed {
                println!("       [{}] #{}: \x1b[9m{}\x1b[0m", check, sub.id, title);
            } else {
                println!("       [{}] #{}: {}", check, sub.id, title);
            }
        }
    }

    let done = active.iter().filter(|t| t.completed).count();
    println!("\n  {} task(s), {} done", active.len(), done);
}

fn cmd_done(id: u32) {
    let todos = read_all_todos();
    for (i, todo) in todos.iter().enumerate() {
        if todo.id == id && !todo.is_deleted {
            let mut updated = Todo::from_bytes(&todo.to_bytes());
            updated.completed = !updated.completed;
            write_todo_at(i, &updated);
            let status = if updated.completed { "completed" } else { "reopened" };
            println!("Task #{} {}: {}", id, status, updated.title_str());
            return;
        }
    }
    eprintln!("Task #{} not found", id);
}

fn cmd_edit(id: u32, title: &str) {
    let todos = read_all_todos();
    for (i, todo) in todos.iter().enumerate() {
        if todo.id == id && !todo.is_deleted {
            let mut updated = Todo::from_bytes(&todo.to_bytes());
            updated.set_title(title);
            write_todo_at(i, &updated);
            println!("Task #{} updated: {}", id, title);
            return;
        }
    }
    eprintln!("Task #{} not found", id);
}

fn cmd_delete(id: u32) {
    let todos = read_all_todos();
    let subtasks = read_all_subtasks();

    // Soft-delete the task
    let mut found = false;
    for (i, todo) in todos.iter().enumerate() {
        if todo.id == id && !todo.is_deleted {
            let mut updated = Todo::from_bytes(&todo.to_bytes());
            updated.is_deleted = true;
            write_todo_at(i, &updated);
            println!("Deleted task #{}: {}", id, updated.title_str());
            found = true;
            break;
        }
    }

    if !found {
        eprintln!("Task #{} not found", id);
        return;
    }

    // Cascade: soft-delete subtasks
    let mut cascade_count = 0;
    for (i, sub) in subtasks.iter().enumerate() {
        if sub.parent_id == id && !sub.is_deleted {
            let mut updated = Subtask::from_bytes(&sub.to_bytes());
            updated.is_deleted = true;
            write_subtask_at(i, &updated);
            cascade_count += 1;
        }
    }
    if cascade_count > 0 {
        println!("  (cascade-deleted {} subtask(s))", cascade_count);
    }
}

fn cmd_sub_add(parent_id: u32, title: &str) {
    let todos = read_all_todos();
    let parent_exists = todos.iter().any(|t| t.id == parent_id && !t.is_deleted);
    if !parent_exists {
        eprintln!("Parent task #{} not found", parent_id);
        return;
    }

    let id = next_subtask_id();
    let sub = Subtask::new(id, parent_id, title);
    append_subtask(&sub);
    println!("Added subtask #{} to task #{}: {}", id, parent_id, title);
}

fn cmd_sub_done(id: u32) {
    let subtasks = read_all_subtasks();
    for (i, sub) in subtasks.iter().enumerate() {
        if sub.id == id && !sub.is_deleted {
            let mut updated = Subtask::from_bytes(&sub.to_bytes());
            updated.completed = !updated.completed;
            write_subtask_at(i, &updated);
            let status = if updated.completed { "completed" } else { "reopened" };
            println!("Subtask #{} {}: {}", id, status, updated.title_str());
            return;
        }
    }
    eprintln!("Subtask #{} not found", id);
}

fn cmd_sub_edit(id: u32, title: &str) {
    let subtasks = read_all_subtasks();
    for (i, sub) in subtasks.iter().enumerate() {
        if sub.id == id && !sub.is_deleted {
            let mut updated = Subtask::from_bytes(&sub.to_bytes());
            updated.set_title(title);
            write_subtask_at(i, &updated);
            println!("Subtask #{} updated: {}", id, title);
            return;
        }
    }
    eprintln!("Subtask #{} not found", id);
}

fn cmd_sub_delete(id: u32) {
    let subtasks = read_all_subtasks();
    for (i, sub) in subtasks.iter().enumerate() {
        if sub.id == id && !sub.is_deleted {
            let mut updated = Subtask::from_bytes(&sub.to_bytes());
            updated.is_deleted = true;
            write_subtask_at(i, &updated);
            println!("Deleted subtask #{}: {}", id, updated.title_str());
            return;
        }
    }
    eprintln!("Subtask #{} not found", id);
}

fn cmd_clear() {
    let _ = fs::remove_file(tasks_path());
    let _ = fs::remove_file(subtasks_path());
    println!("All tasks cleared.");
}

fn print_usage() {
    println!("Rust Todo App (File I/O — Phase 26)");
    println!();
    println!("Usage: todo <command> [args]");
    println!();
    println!("Commands:");
    println!("  add <title>              Add a new task");
    println!("  list                     List all tasks and subtasks");
    println!("  done <id>                Toggle task completion");
    println!("  edit <id> <title>        Edit task title");
    println!("  delete <id>              Delete task (cascades to subtasks)");
    println!("  sub <parent_id> <title>  Add a subtask");
    println!("  sub-done <id>            Toggle subtask completion");
    println!("  sub-edit <id> <title>    Edit subtask title");
    println!("  sub-delete <id>          Delete a subtask");
    println!("  clear                    Delete all data");
    println!();
    println!("Data stored in: ~/.rust-todo/");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // Default to list
        cmd_list();
        return;
    }

    match args[1].as_str() {
        "add" => {
            if args.len() < 3 {
                eprintln!("Usage: todo add <title>");
                return;
            }
            cmd_add(&args[2..].join(" "));
        }
        "list" | "ls" => cmd_list(),
        "done" => {
            if args.len() < 3 {
                eprintln!("Usage: todo done <id>");
                return;
            }
            let id: u32 = args[2].parse().expect("Invalid task ID");
            cmd_done(id);
        }
        "edit" => {
            if args.len() < 4 {
                eprintln!("Usage: todo edit <id> <title>");
                return;
            }
            let id: u32 = args[2].parse().expect("Invalid task ID");
            cmd_edit(id, &args[3..].join(" "));
        }
        "delete" | "rm" => {
            if args.len() < 3 {
                eprintln!("Usage: todo delete <id>");
                return;
            }
            let id: u32 = args[2].parse().expect("Invalid task ID");
            cmd_delete(id);
        }
        "sub" => {
            if args.len() < 4 {
                eprintln!("Usage: todo sub <parent_id> <title>");
                return;
            }
            let parent_id: u32 = args[2].parse().expect("Invalid parent ID");
            cmd_sub_add(parent_id, &args[3..].join(" "));
        }
        "sub-done" => {
            if args.len() < 3 {
                eprintln!("Usage: todo sub-done <id>");
                return;
            }
            let id: u32 = args[2].parse().expect("Invalid subtask ID");
            cmd_sub_done(id);
        }
        "sub-edit" => {
            if args.len() < 4 {
                eprintln!("Usage: todo sub-edit <id> <title>");
                return;
            }
            let id: u32 = args[2].parse().expect("Invalid subtask ID");
            cmd_sub_edit(id, &args[3..].join(" "));
        }
        "sub-delete" | "sub-rm" => {
            if args.len() < 3 {
                eprintln!("Usage: todo sub-delete <id>");
                return;
            }
            let id: u32 = args[2].parse().expect("Invalid subtask ID");
            cmd_sub_delete(id);
        }
        "clear" => cmd_clear(),
        "help" | "--help" | "-h" => print_usage(),
        other => {
            eprintln!("Unknown command: {}", other);
            print_usage();
        }
    }
}
