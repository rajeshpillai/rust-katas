# Rust Katas

A structured learning platform for Rust and WebAssembly. Learn through deliberate practice — broken code, compiler errors, and invariants.

## Stack

- **Backend:** Rust (Axum) — serves kata content and runs a Rust playground
- **Frontend:** SolidJS + TailwindCSS (Vite)
- **Kata content:** Markdown files with YAML frontmatter

## Getting Started

### Prerequisites

- Rust toolchain (`rustup`)
- Node.js 18+

### Development

Start the backend (default port 6000, override with `PORT` env var):

```sh
cd backend
cargo run
# or: PORT=8080 cargo run
```

Start the frontend dev server (proxies `/api` to backend):

```sh
cd frontend
npm install
npm run dev
```

Open `http://localhost:5173`.

### Production

```sh
cd frontend && npm run build
cd ../backend && cargo run
```

Open `http://localhost:6000`.

## Project Structure

```
rust-katas/
├── backend/          Axum server (API + playground)
├── frontend/         SolidJS + TailwindCSS UI
├── katas/            Markdown kata content by phase
├── CLAUDE.md         Syllabus and teaching rules
└── todo.md           Phase progress tracker
```

## Adding Katas

Create a markdown file in `katas/phase-NN/` with YAML frontmatter:

```markdown
---
id: my-kata-id
phase: 0
phase_title: Rust as a Language
sequence: 4
title: My Kata Title
hints:
  - First hint
  - Second hint
---

## Description
...

## Broken Code
\```rust
...
\```

## Correct Code
\```rust
...
\```

## Explanation
...

## Compiler Error Interpretation
...
```

Restart the backend to pick up new katas.

## Apps

### CLI Todo App (Phase 26 — File I/O)

A standalone terminal todo app built with the binary file I/O patterns taught in Phase 26. No external dependencies — compiles with `rustc` alone.

**Build:**

```sh
rustc --edition 2021 apps/todo-app/main.rs -o todo
```

**Usage:**

```sh
./todo                           # List all tasks (default)
./todo add Buy groceries         # Add a task
./todo done 1                    # Toggle task #1 completion
./todo edit 1 Buy organic food   # Edit task title
./todo delete 1                  # Delete task (cascades to subtasks)
./todo sub 1 Get milk            # Add subtask to task #1
./todo sub-done 1                # Toggle subtask completion
./todo sub-edit 1 Get oat milk   # Edit subtask title
./todo sub-delete 1              # Delete a subtask
./todo clear                     # Wipe all data
```

**Technical details:**

- Data stored as fixed-size binary records in `~/.rust-todo/` (`tasks.bin` + `subtasks.bin`)
- Record layout: `id (4B) + title (128B, null-padded) + completed (1B) + deleted (1B) = 134 bytes` per task
- Subtask records add a `parent_id (4B)` field = 138 bytes each
- Random access via `Seek` — reads/writes individual records by offset
- Soft delete with cascading (deleting a task deletes its subtasks)
- Uses `OpenOptions` for in-place updates without truncation

## License

This project is licensed under the [GNU Affero General Public License v3.0](LICENSE) (AGPL-3.0).
