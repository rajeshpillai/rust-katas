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
