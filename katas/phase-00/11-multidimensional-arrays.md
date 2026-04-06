---
id: multidimensional-arrays
phase: 0
phase_title: "Rust as a Language"
sequence: 11
title: Multidimensional Arrays and Grids
hints:
  - "A 2D array in Rust is an array of arrays: `[[i32; COLS]; ROWS]`."
  - "Access elements with `grid[row][col]`. The outer index selects the row, the inner selects the column."
  - "For dynamic-sized grids, use `Vec<Vec<T>>`. Each inner Vec is a row."
---

## Description

Rust does not have a special multidimensional array type. Instead, you nest arrays: a 2D grid is an array of arrays. For fixed sizes known at compile time, use `[[T; COLS]; ROWS]`. For dynamic sizes, use `Vec<Vec<T>>`.

## Broken Code

```rust
fn main() {
    // Bug: dimensions are swapped — this creates 4 rows of 3 columns,
    // but the code tries to access [3][0] (row 3) which only has indices 0..3
    let grid: [[i32; 4]; 3] = [
        [1, 2, 3, 4],
        [5, 6, 7, 8],
        [9, 10, 11, 12],
    ];

    // Trying to access row 3 — but there are only 3 rows (indices 0, 1, 2)
    println!("Element at [3][0] = {}", grid[3][0]);
}
```

## Correct Code

```rust
fn main() {
    // --- Fixed-size 2D array: [[T; COLS]; ROWS] ---
    let grid: [[i32; 4]; 3] = [
        [ 1,  2,  3,  4],
        [ 5,  6,  7,  8],
        [ 9, 10, 11, 12],
    ];

    // Access: grid[row][col]
    println!("3 rows x 4 cols:");
    println!("grid[0][0] = {}", grid[0][0]); // 1
    println!("grid[1][2] = {}", grid[1][2]); // 7
    println!("grid[2][3] = {}", grid[2][3]); // 12

    // Iterate over all elements
    println!("\nAll elements:");
    for row in &grid {
        for &val in row {
            print!("{:3} ", val);
        }
        println!();
    }

    // --- Dynamic 2D grid: Vec<Vec<T>> ---
    let rows = 3;
    let cols = 5;

    // Initialize with zeros
    let mut dynamic_grid: Vec<Vec<i32>> = vec![vec![0; cols]; rows];

    // Fill with values
    for r in 0..rows {
        for c in 0..cols {
            dynamic_grid[r][c] = (r * cols + c + 1) as i32;
        }
    }

    println!("\nDynamic {}x{} grid:", rows, cols);
    for row in &dynamic_grid {
        for &val in row {
            print!("{:3} ", val);
        }
        println!();
    }

    // --- Matrix operations ---
    // Transpose a matrix
    let original = vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
    ];

    let rows = original.len();
    let cols = original[0].len();
    let mut transposed = vec![vec![0; rows]; cols];
    for r in 0..rows {
        for c in 0..cols {
            transposed[c][r] = original[r][c];
        }
    }

    println!("\nOriginal (2x3):");
    for row in &original {
        println!("  {:?}", row);
    }
    println!("Transposed (3x2):");
    for row in &transposed {
        println!("  {:?}", row);
    }

    // --- Sum of each row ---
    let data = vec![
        vec![10, 20, 30],
        vec![40, 50, 60],
        vec![70, 80, 90],
    ];

    println!("\nRow sums:");
    for (i, row) in data.iter().enumerate() {
        let sum: i32 = row.iter().sum();
        println!("  Row {}: {:?} => sum = {}", i, row, sum);
    }

    // Total sum
    let total: i32 = data.iter().flat_map(|row| row.iter()).sum();
    println!("Total: {}", total);
}
```

## Explanation

The broken code declares `[[i32; 4]; 3]` — that is 3 rows, each with 4 columns. Valid row indices are 0, 1, 2. Accessing `grid[3][0]` is an **out-of-bounds** access that panics at runtime.

**Reading the type `[[i32; COLS]; ROWS]`:** Read inside-out. The inner `[i32; 4]` is one row of 4 integers. The outer `[_; 3]` is an array of 3 such rows. So it is **3 rows x 4 columns**.

**Fixed vs dynamic grids:**

| Approach | Type | Size known at | Location |
|---|---|---|---|
| Fixed | `[[T; COLS]; ROWS]` | Compile time | Stack |
| Dynamic | `Vec<Vec<T>>` | Runtime | Heap |

**`vec![vec![0; cols]; rows]`** creates a `Vec` of `rows` elements, where each element is a `Vec` of `cols` zeros. This is the dynamic equivalent of `[[0; COLS]; ROWS]`.

**Common grid operations demonstrated:**
- Access: `grid[row][col]`
- Iteration: nested `for` loops or `.iter().enumerate()`
- Transpose: swap rows and columns
- Row sums: `.iter().sum()` on each row
- Flat iteration: `.flat_map(|row| row.iter())` flattens 2D to 1D

**Memory layout:** Fixed 2D arrays are contiguous in memory (row-major). `Vec<Vec<T>>` stores each row as a separate heap allocation — rows are not necessarily contiguous. For performance-critical code, consider a flat `Vec<T>` with manual `row * cols + col` indexing.

## ⚠️ Caution

- `grid[row][col]` — the **row comes first**. This is the opposite of mathematical `(x, y)` notation where x is horizontal.
- Out-of-bounds access panics at runtime. Use `.get(index)` for safe checked access.
- `Vec<Vec<T>>` rows can have different lengths (jagged array). If you need a true rectangular matrix, validate lengths or use a flat `Vec<T>` with computed indices.

## 💡 Tips

- `vec![vec![0; cols]; rows]` is the idiomatic way to create a zero-initialized dynamic grid.
- For high-performance matrix operations, consider a flat `Vec<T>` with `index = row * cols + col`. This avoids pointer indirection and is cache-friendly.
- `.flat_map()` is powerful for treating a 2D structure as a flat sequence.
- The `ndarray` crate provides true N-dimensional arrays for scientific computing, but understanding the manual approach is essential for systems programming.

## Compiler Error Interpretation

The broken code compiles but panics at runtime:

```
thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 3'
```

Array bounds are checked at runtime in Rust. Unlike C (undefined behavior on out-of-bounds), Rust panics with a clear message: the array has 3 elements (indices 0, 1, 2) but you tried to access index 3. This prevents memory corruption at the cost of a runtime check.

---

| [Prev: Arrays and Slices](#/katas/arrays-and-slices) | [Next: Methods and impl Blocks](#/katas/methods-and-impl) |
