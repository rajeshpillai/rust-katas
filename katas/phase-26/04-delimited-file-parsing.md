---
id: delimited-file-parsing
phase: 26
phase_title: "File I/O in Rust"
sequence: 4
title: Delimited File Parsing
hints:
  - "The first line of a CSV is usually a header, not data. Parsing it as data will fail."
  - "Using `unwrap()` on `parse()` panics on the first invalid value. You need per-line error handling."
  - "Use `.enumerate()` to track line numbers, skip the header with `.skip(1)`, and handle parse errors with `match` or `if let`."
---

## Description

Delimited files (CSV, TSV) are the most common data exchange format. Each line is a record, and fields are separated by a delimiter (usually `,` or `\t`).

Parsing delimited data involves:
1. Reading line by line (using `BufReader`)
2. Splitting each line on the delimiter
3. Parsing each field into the correct type
4. Handling errors gracefully — headers, malformed rows, missing fields

In Rust, this combines file I/O, string processing, error handling, and iterators.

## Broken Code

```rust
use std::fs::File;
use std::io::{Write, BufRead, BufReader};

fn main() {
    let path = std::env::temp_dir().join("kata-csv-demo.csv");

    // Write sample CSV data
    let mut file = File::create(&path).unwrap();
    writeln!(file, "name,age,score").unwrap();
    writeln!(file, "Alice,30,95.5").unwrap();
    writeln!(file, "Bob,25,87.3").unwrap();
    writeln!(file, "Charlie,bad_age,91.0").unwrap();
    writeln!(file, "Diana,28,72.1").unwrap();

    // Parse the CSV
    let file = File::open(&path).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let fields: Vec<&str> = line.split(',').collect();
        let name = fields[0];
        let age: u32 = fields[1].parse().unwrap(); // Panics on header AND on "bad_age"
        let score: f64 = fields[2].parse().unwrap();
        println!("{}: age={}, score={:.1}", name, age, score);
    }

    std::fs::remove_file(&path).unwrap();
}
```

## Correct Code

```rust
use std::fs::File;
use std::io::{Write, BufRead, BufReader};

#[derive(Debug)]
struct Student {
    name: String,
    age: u32,
    score: f64,
}

fn parse_student(line: &str) -> Result<Student, String> {
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() != 3 {
        return Err(format!("expected 3 fields, got {}", fields.len()));
    }

    let name = fields[0].to_string();
    let age: u32 = fields[1]
        .parse()
        .map_err(|e| format!("invalid age '{}': {}", fields[1], e))?;
    let score: f64 = fields[2]
        .parse()
        .map_err(|e| format!("invalid score '{}': {}", fields[2], e))?;

    Ok(Student { name, age, score })
}

fn main() {
    let path = std::env::temp_dir().join("kata-csv-demo.csv");

    // Write sample CSV data (with a bad row)
    let mut file = File::create(&path).unwrap();
    writeln!(file, "name,age,score").unwrap();
    writeln!(file, "Alice,30,95.5").unwrap();
    writeln!(file, "Bob,25,87.3").unwrap();
    writeln!(file, "Charlie,bad_age,91.0").unwrap();
    writeln!(file, "Diana,28,72.1").unwrap();

    // Parse with error handling
    let file = File::open(&path).unwrap();
    let reader = BufReader::new(file);

    let mut students = Vec::new();
    let mut errors = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.unwrap();

        // Skip header
        if line_num == 0 {
            println!("Header: {}", line);
            continue;
        }

        match parse_student(&line) {
            Ok(student) => students.push(student),
            Err(e) => errors.push(format!("Line {}: {}", line_num + 1, e)),
        }
    }

    println!("\nParsed {} students:", students.len());
    for s in &students {
        println!("  {} (age {}): {:.1}", s.name, s.age, s.score);
    }

    if !errors.is_empty() {
        println!("\n{} error(s):", errors.len());
        for e in &errors {
            println!("  {}", e);
        }
    }

    // Calculate average score
    if !students.is_empty() {
        let avg: f64 = students.iter().map(|s| s.score).sum::<f64>() / students.len() as f64;
        println!("\nAverage score: {:.1}", avg);
    }

    std::fs::remove_file(&path).unwrap();
}
```

## Explanation

The broken code has two fatal flaws:

1. **It does not skip the header line.** The first iteration tries to parse `"name"` as `u32` — instant panic.

2. **It uses `unwrap()` on user data.** The third data row has `"bad_age"` in the age field. Even if the header were skipped, this row would panic.

Both are instances of the same mistake: **treating external data as trusted**. File contents are user input — they can contain anything.

**The correct approach separates parsing from error handling:**

```rust
fn parse_student(line: &str) -> Result<Student, String> { ... }
```

This function returns `Result` — it never panics. The caller decides what to do with errors: skip the row, collect errors for reporting, or abort.

**Key patterns demonstrated:**

- **`.enumerate()`** gives `(index, value)` pairs — essential for reporting line numbers in errors
- **`.skip(1)` or `if line_num == 0`** to handle the header line
- **`map_err`** to add context to parse errors (which field failed, what the bad value was)
- **Separate collection** of successes and errors — process what you can, report what you cannot
- **A dedicated parse function** that returns `Result` — keeps the main loop clean

**Why `split(',')` is naive:**

Simple comma-splitting breaks if fields contain commas (e.g., `"Smith, Jr.",30,95`). Real CSV parsing must handle:
- Quoted fields: `"value with, comma"`
- Escaped quotes: `"value with ""quote"""`
- Different line endings: `\n`, `\r\n`

For production CSV parsing, use the `csv` crate. Manual splitting is fine for learning and for simple, controlled formats.

## ⚠️ Caution

- Never `unwrap()` on data from external files. Files can contain anything — malformed rows, encoding errors, truncated lines.
- `split(',')` does not handle quoted fields. If your data may contain commas within fields, use a proper CSV parser.
- Watch for off-by-one errors with line numbering: `.enumerate()` starts at 0, but humans count lines from 1.

## 💡 Tips

- Collect errors separately rather than failing on the first bad row. This lets you report all problems at once — much more useful for data quality checks.
- Use a struct to represent parsed records. This gives you type safety and makes the parsed data easy to work with.
- The `?` operator works naturally inside a function returning `Result`. Use `map_err` to convert parse errors into your error type with added context.

## Compiler Error Interpretation

The broken code compiles fine — the errors are at **runtime**:

```
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }', main.rs:18:47
```

- **`ParseIntError { kind: InvalidDigit }`** — `"name".parse::<u32>()` found a non-digit character. The header line is being treated as data.
- **`unwrap()` on an `Err` value`** — The code assumed parsing would always succeed. It did not.

If you fix the header issue but keep `unwrap()`, the `"bad_age"` row produces the same panic. The only safe approach is to handle `Result` properly for every parse operation on external data.

The broader lesson: **compile-time safety covers types and ownership; runtime safety for I/O is your responsibility.** Rust gives you `Result` as the tool — use it.

---

| [Prev: Binary Files: Structs to Bytes](#/katas/binary-files-structs-to-bytes) | [Next: Random Access with Seek](#/katas/random-access-seek) |
