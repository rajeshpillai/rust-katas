---
id: multidimensional-collections
phase: 6
phase_title: "Collections & the Owned/Borrowed Duality"
sequence: 5
title: "Multidimensional Collections — Grids and Matrices"
hints:
  - "`Vec<Vec<T>>` is the dynamic equivalent of a 2D array. Each inner Vec is a row."
  - "For flat (cache-friendly) storage, use a single `Vec<T>` with manual index calculation: `row * cols + col`."
  - "Watch for borrow conflicts when modifying one cell based on another — you may need to read first, then write."
---

## Description

Dynamic multidimensional data in Rust uses `Vec<Vec<T>>` (vector of vectors) or a flat `Vec<T>` with computed indices. Each approach has trade-offs in ergonomics and performance.

This kata demonstrates both patterns and a common ownership pitfall when modifying grid cells.

## Broken Code

```rust
fn main() {
    let mut grid = vec![vec![0; 3]; 3];

    // Fill with values
    for r in 0..3 {
        for c in 0..3 {
            grid[r][c] = r * 3 + c + 1;
        }
    }

    // Bug: borrow conflict — reading grid[0][0] while mutably borrowing grid[1]
    grid[1][0] = grid[0][0] + grid[0][1];
    // (This specific case actually works! But this pattern fails:)

    // Bug: can't borrow grid as mutable and immutable at the same time
    let row0 = &grid[0];       // Immutable borrow of grid
    grid[1][0] = row0[0] + 1;  // Mutable borrow of grid — conflict!

    println!("{:?}", grid);
}
```

## Correct Code

```rust
fn print_grid(label: &str, grid: &Vec<Vec<i32>>) {
    println!("{}:", label);
    for row in grid {
        for (i, &val) in row.iter().enumerate() {
            if i > 0 { print!("  "); }
            print!("{:3}", val);
        }
        println!();
    }
    println!();
}

fn main() {
    // --- Vec<Vec<T>>: dynamic 2D grid ---
    let rows = 3;
    let cols = 4;
    let mut grid: Vec<Vec<i32>> = vec![vec![0; cols]; rows];

    // Fill: grid[row][col]
    for r in 0..rows {
        for c in 0..cols {
            grid[r][c] = (r * cols + c + 1) as i32;
        }
    }
    print_grid("Initial grid", &grid);

    // Reading from one cell to write to another — read first, then write
    let value = grid[0][0] + grid[0][1]; // Read first (no borrow held)
    grid[1][0] = value;                   // Then write
    print_grid("After copying values", &grid);

    // --- split_at_mut for simultaneous mutable access to different rows ---
    let (top, bottom) = grid.split_at_mut(1);
    // top = &mut [row0], bottom = &mut [row1, row2]
    bottom[0][3] = top[0][0] + top[0][1]; // Read from row 0, write to row 1
    print_grid("After split_at_mut", &grid);

    // --- Flat Vec<T>: cache-friendly alternative ---
    let rows = 3;
    let cols = 4;
    let mut flat: Vec<i32> = vec![0; rows * cols];

    // Fill using row * cols + col
    for r in 0..rows {
        for c in 0..cols {
            flat[r * cols + c] = (r * cols + c + 1) as i32;
        }
    }

    println!("Flat grid:");
    for r in 0..rows {
        for c in 0..cols {
            print!("{:3} ", flat[r * cols + c]);
        }
        println!();
    }

    // Flat access is simple and fast
    flat[1 * cols + 2] = 99; // row 1, col 2
    println!("\nAfter flat[1][2] = 99:");
    for r in 0..rows {
        print!("  ");
        for c in 0..cols {
            print!("{:3} ", flat[r * cols + c]);
        }
        println!();
    }

    // --- Row and column operations ---
    let data = vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ];

    // Row sums
    println!("\nRow sums:");
    for (i, row) in data.iter().enumerate() {
        let sum: i32 = row.iter().sum();
        println!("  Row {}: {}", i, sum);
    }

    // Column sums (iterate by column index)
    println!("Column sums:");
    for c in 0..data[0].len() {
        let sum: i32 = data.iter().map(|row| row[c]).sum();
        println!("  Col {}: {}", c, sum);
    }

    // Diagonal
    let diag_sum: i32 = (0..data.len()).map(|i| data[i][i]).sum();
    println!("Diagonal sum: {}", diag_sum);

    // --- Vec<Vec<T>> vs flat Vec<T> ---
    println!("\n--- Comparison ---");
    println!("Vec<Vec<T>>:  ergonomic [r][c], rows can differ in length");
    println!("Flat Vec<T>:  cache-friendly, single allocation, fixed stride");
}
```

## Explanation

The broken code creates a reference to `grid[0]` (borrowing `grid` immutably) and then tries to modify `grid[1]` (borrowing `grid` mutably). Rust's borrow rules forbid this: you cannot have an immutable and mutable borrow of the same data simultaneously.

**The fix: don't hold a reference across the mutation.**

```rust
// Bad: holds an immutable borrow while mutating
let row0 = &grid[0];
grid[1][0] = row0[0] + 1;  // CONFLICT

// Good: read the value, release the borrow, then write
let value = grid[0][0] + 1;  // Temporary borrow — released immediately
grid[1][0] = value;           // New borrow — no conflict
```

**`split_at_mut`** is the escape hatch for simultaneous mutable access to different parts of a slice. It splits a mutable slice into two non-overlapping mutable slices, proving to the compiler that they don't alias.

**Vec<Vec<T>> vs flat Vec<T>:**

| | `Vec<Vec<T>>` | Flat `Vec<T>` |
|---|---|---|
| Access | `grid[r][c]` | `flat[r * cols + c]` |
| Memory | Multiple heap allocations | Single allocation |
| Cache | Rows may not be contiguous | Fully contiguous |
| Rows | Can differ in length (jagged) | Must be same length |
| Resize | Per-row flexibility | Must resize whole grid |

For most applications, `Vec<Vec<T>>` is fine. For performance-critical numerical code, the flat layout is significantly faster due to cache locality.

## ⚠️ Caution

- `Vec<Vec<T>>` rows are independent allocations. They can have different lengths (jagged array), which is sometimes a feature and sometimes a bug.
- The flat approach requires manual index math. Off-by-one errors produce silent wrong results, not crashes (unless you exceed the flat vec's length).
- `split_at_mut` requires a known split point. For arbitrary multi-cell access patterns, read all needed values first, then write.

## 💡 Tips

- Wrap flat grid access in a helper struct with `fn get(&self, row: usize, col: usize) -> &T` to avoid manual index math everywhere.
- `.iter().flat_map(|row| row.iter())` flattens a `Vec<Vec<T>>` into a single iterator.
- For large numeric grids, consider the `ndarray` crate which provides N-dimensional arrays with optimized operations.
- Column sums in a `Vec<Vec<T>>` require iterating all rows: `(0..cols).map(|c| data.iter().map(|row| row[c]).sum())`.

## Compiler Error Interpretation

```
error[E0502]: cannot borrow `grid` as mutable because it is also borrowed as immutable
  --> main.rs:14:5
   |
13 |     let row0 = &grid[0];
   |                 ---- immutable borrow occurs here
14 |     grid[1][0] = row0[0] + 1;
   |     ^^^^ mutable borrow occurs here
15 |     println!("{}", row0[0]);
   |                    ---- immutable borrow later used here
```

The compiler shows: `grid` is borrowed immutably on line 13 (via `row0`), and you try to borrow it mutably on line 14. Since `row0` is still alive, the borrows overlap. The fix: drop the reference before mutating, or use `split_at_mut` for non-overlapping mutable access.

---

| [Prev: Fixed Arrays vs Vec — Stack vs Heap](#/katas/arrays-vs-vec) | [Next: HashSet, BTreeMap, and Ordered Collections](#/katas/hashset-and-btreemap) |
