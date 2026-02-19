---
id: memory-layout
phase: 16
phase_title: "Advanced Systems Patterns"
sequence: 2
title: Memory Layout with repr(C) for FFI
hints:
  - Rust does not guarantee struct field ordering by default -- the compiler may reorder fields
  - C code expects a specific, predictable memory layout
  - "#[repr(C)]" forces Rust to lay out the struct the same way a C compiler would
  - Without repr(C), passing a Rust struct to C code is undefined behavior
---

## Description

By default, the Rust compiler is free to reorder struct fields, add padding, and optimize the memory layout however it sees fit. This is fine for pure Rust code -- the compiler handles all field access. But when you pass a struct across an FFI boundary to C code, the C code expects a specific, predictable layout: fields in declaration order with platform-defined alignment and padding. The `#[repr(C)]` attribute tells Rust to use the C memory layout, making the struct's layout predictable and compatible with C.

## Broken Code

```rust
// Simulating a C function that reads a Point struct.
// In real FFI, this would be an actual C function.

// BUG: No #[repr(C)] -- Rust may reorder fields.
// The C side expects {x, y} in that order, but Rust
// might lay it out as {y, x} or add unexpected padding.
#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

// Simulating what happens when C code reads raw memory
// with an expected layout
fn read_point_from_c_perspective(ptr: *const u8) -> (f64, f64) {
    unsafe {
        // C code expects: first 8 bytes = x, next 8 bytes = y
        let x = *(ptr as *const f64);
        let y = *(ptr.add(8) as *const f64);
        (x, y)
    }
}

extern "C" {
    // Example: a C function that takes a pointer to a Point struct
    // void process_point(const Point* p);
    //
    // If Point does not have #[repr(C)], the C function will
    // read garbage because the fields may be reordered.
}

fn main() {
    let point = Point { x: 1.0, y: 2.0 };

    let ptr = &point as *const Point as *const u8;
    let (read_x, read_y) = read_point_from_c_perspective(ptr);

    // With default Rust repr, these might be WRONG.
    // The compiler may have reordered x and y.
    println!("Expected: ({}, {})", point.x, point.y);
    println!("C reads:  ({}, {})", read_x, read_y);
}
```

## Correct Code

```rust
// Correct: #[repr(C)] guarantees C-compatible layout.
// Fields will be in declaration order with C-standard padding.
#[repr(C)]
#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

fn read_point_from_c_perspective(ptr: *const u8) -> (f64, f64) {
    unsafe {
        // C code expects: first 8 bytes = x, next 8 bytes = y
        // With #[repr(C)], this is guaranteed to be correct.
        let x = *(ptr as *const f64);
        let y = *(ptr.add(8) as *const f64);
        (x, y)
    }
}

fn main() {
    let point = Point { x: 1.0, y: 2.0 };

    let ptr = &point as *const Point as *const u8;
    let (read_x, read_y) = read_point_from_c_perspective(ptr);

    // With #[repr(C)], these are guaranteed correct.
    println!("Expected: ({}, {})", point.x, point.y);
    println!("C reads:  ({}, {})", read_x, read_y);
    assert_eq!(read_x, 1.0);
    assert_eq!(read_y, 2.0);
}
```

## Explanation

The broken version does not use `#[repr(C)]` on the `Point` struct. Without it, the Rust compiler uses its default representation, which makes no guarantees about field order, padding, or alignment. The compiler may reorder `x` and `y` in memory, add extra padding between them, or pack them differently than C would. When C code (or any code that assumes a specific memory layout) reads the struct's raw bytes, it may read `y` when it expects `x`, or encounter unexpected padding bytes.

**What #[repr(C)] guarantees:**

1. **Field order**: Fields are laid out in the order they are declared, matching C's behavior.
2. **Alignment**: Each field is aligned according to the C ABI rules for the target platform.
3. **Padding**: Padding is inserted between fields and at the end of the struct exactly as a C compiler would.
4. **Determinism**: The layout is fully determined by the struct definition and the target platform.

**What default Rust repr does:**

Rust's default representation (`repr(Rust)`) gives the compiler freedom to:
- Reorder fields to minimize padding (e.g., putting larger fields first)
- Use different alignment than C
- Change layout between compiler versions

This means two things: (1) Rust structs can be more memory-efficient than C structs, and (2) their layout is unpredictable, making them incompatible with FFI.

**A more complex example showing why this matters:**

```rust
// Without repr(C):
struct Mixed {
    a: u8,    // 1 byte
    b: u64,   // 8 bytes
    c: u16,   // 2 bytes
}
// Rust might reorder to: { b: u64, c: u16, a: u8 } to reduce padding
// Total: 16 bytes (Rust optimized) vs 24 bytes (C layout with padding)

#[repr(C)]
struct MixedC {
    a: u8,    // 1 byte + 7 bytes padding
    b: u64,   // 8 bytes
    c: u16,   // 2 bytes + 6 bytes padding
}
// C layout: { a(1) + pad(7) + b(8) + c(2) + pad(6) } = 24 bytes
```

**Other repr options:**

- `#[repr(transparent)]` -- the type has the same layout as its single field (useful for newtypes)
- `#[repr(packed)]` -- no padding between fields (can cause alignment issues)
- `#[repr(align(N))]` -- set minimum alignment

The invariant violated in the broken code: **passing a Rust struct to C without `#[repr(C)]` is undefined behavior because the memory layout is not guaranteed to match C's expectations.**

## Compiler Error Interpretation

This kata does not produce a compiler error -- the broken version compiles and runs. The bug is that the memory layout may not match expectations. This is one of the most dangerous categories of FFI bugs: the code compiles, may appear to work on one platform or compiler version, and silently produces wrong results on another.

The Rust compiler provides a tool to inspect layout: `std::mem::size_of::<T>()` and `std::mem::align_of::<T>()`. You can also use `#[cfg(test)]` to write compile-time assertions:

```rust
#[test]
fn check_layout() {
    assert_eq!(std::mem::size_of::<Point>(), 16);
    assert_eq!(std::mem::align_of::<Point>(), 8);
}
```

The Clippy linter also has a lint (`clippy::missing_repr`) that can warn about structs used in FFI without `#[repr(C)]`. Additionally, tools like `cbindgen` can automatically generate C header files from Rust types, but they require `#[repr(C)]` to work correctly.

The lesson: **in FFI, silence from the compiler does not mean safety. You must actively annotate types with `#[repr(C)]` and verify layouts match your C counterparts.**
