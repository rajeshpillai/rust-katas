---
id: offset-arithmetic
phase: 19
phase_title: "WASM Memory Model"
sequence: 1
title: Pointer Offset Arithmetic in Linear Memory
hints:
  - WASM addresses are byte offsets into a flat array
  - An i32 occupies 4 bytes, so two consecutive i32 values need offsets 4 apart
  - Writing at overlapping offsets corrupts previously written data
  - Always account for the size of the type when calculating offsets
---

## Description

WASM linear memory is byte-addressable. When you store an `i32` at offset 0, it occupies bytes 0 through 3. The next `i32` must go at offset 4 or later, not offset 2. Overlapping writes corrupt data silently -- there is no runtime check for overlapping regions. This kata demonstrates the importance of correct offset arithmetic when manually managing data layout in linear memory. Getting the math wrong leads to data corruption that is extremely difficult to debug.

## Broken Code

```rust
fn write_i32(memory: &mut [u8], offset: usize, value: i32) {
    let bytes = value.to_le_bytes();
    memory[offset] = bytes[0];
    memory[offset + 1] = bytes[1];
    memory[offset + 2] = bytes[2];
    memory[offset + 3] = bytes[3];
}

fn read_i32(memory: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes([
        memory[offset],
        memory[offset + 1],
        memory[offset + 2],
        memory[offset + 3],
    ])
}

fn main() {
    let mut memory = vec![0u8; 64];

    // Write first value at offset 0
    write_i32(&mut memory, 0, 42);

    // BUG: Write second value at offset 2 -- overlaps with the first!
    // i32 takes 4 bytes, so offset 2 overlaps bytes 2-3 of the first value.
    write_i32(&mut memory, 2, 99);

    // Read back the first value -- it has been corrupted
    let first = read_i32(&memory, 0);
    let second = read_i32(&memory, 2);

    println!("First value (expected 42): {}", first);
    println!("Second value (expected 99): {}", second);
    assert_eq!(first, 42, "First value was corrupted by overlapping write!");
}
```

## Correct Code

```rust
fn write_i32(memory: &mut [u8], offset: usize, value: i32) -> Result<(), String> {
    if offset + 4 > memory.len() {
        return Err(format!(
            "Out of bounds: offset {} + 4 > memory size {}",
            offset, memory.len()
        ));
    }
    let bytes = value.to_le_bytes();
    memory[offset..offset + 4].copy_from_slice(&bytes);
    Ok(())
}

fn read_i32(memory: &[u8], offset: usize) -> Result<i32, String> {
    if offset + 4 > memory.len() {
        return Err(format!(
            "Out of bounds: offset {} + 4 > memory size {}",
            offset, memory.len()
        ));
    }
    Ok(i32::from_le_bytes([
        memory[offset],
        memory[offset + 1],
        memory[offset + 2],
        memory[offset + 3],
    ]))
}

fn main() {
    let mut memory = vec![0u8; 64];

    // Write first value at offset 0 (occupies bytes 0-3)
    write_i32(&mut memory, 0, 42).unwrap();

    // Correct: write second value at offset 4 (occupies bytes 4-7)
    // No overlap -- each i32 gets its own 4-byte region
    write_i32(&mut memory, 4, 99).unwrap();

    let first = read_i32(&memory, 0).unwrap();
    let second = read_i32(&memory, 4).unwrap();

    println!("First value: {}", first);
    println!("Second value: {}", second);
    assert_eq!(first, 42);
    assert_eq!(second, 99);

    // Demonstrate sequential layout for an array of i32
    let values = [10, 20, 30, 40, 50];
    for (i, &v) in values.iter().enumerate() {
        let offset = 16 + i * std::mem::size_of::<i32>();
        write_i32(&mut memory, offset, v).unwrap();
    }
    for i in 0..values.len() {
        let offset = 16 + i * std::mem::size_of::<i32>();
        let v = read_i32(&memory, offset).unwrap();
        println!("memory[{}] = {}", offset, v);
    }
}
```

## Explanation

The broken version writes two `i32` values at offsets 0 and 2. Since an `i32` occupies 4 bytes, the first value uses bytes 0-3 and the second uses bytes 2-5. Bytes 2 and 3 are written by both operations. The second write overwrites part of the first value, corrupting it. The `assert_eq!` fails because reading back offset 0 returns a mixed value, not 42.

**How WASM handles this:**

WASM itself does not prevent overlapping writes. The `i32.store` instruction writes 4 bytes at a given offset -- if two stores overlap, the second one wins for the overlapping bytes. This is by design: WASM gives you a flat byte array and trusts you to manage layout correctly.

**The offset calculation rule:**

For sequential values of type `T`, the offsets must be spaced by `size_of::<T>()` (or more, for alignment). For an array of `i32`:

| Value | Offset | Bytes Used |
|-------|--------|------------|
| first | 0 | 0, 1, 2, 3 |
| second | 4 | 4, 5, 6, 7 |
| third | 8 | 8, 9, 10, 11 |

The formula is: `offset = base + index * size_of::<T>()`

**Endianness:**

WASM specifies little-endian byte order for all memory operations. Our code uses `to_le_bytes()` and `from_le_bytes()` to match this convention. On a big-endian host, using native byte order would produce wrong results.

The invariant violated in the broken code: **consecutive values in linear memory must not overlap; each value must occupy its own non-overlapping region of bytes.**

## Compiler Error Interpretation

```
thread 'main' panicked at 'assertion `left == right` failed: First value was corrupted by overlapping write!
  left: 6422528
 right: 42', src/main.rs:30:5
```

This is a runtime panic from a failed assertion:

1. **"left: 6422528, right: 42"** -- the value read from offset 0 is `6422528`, not `42`. This is because bytes 2 and 3 of the original value (42 = `0x0000002A`) were overwritten by the first two bytes of 99 (`0x00000063`). The resulting bytes at offset 0 are a mix of both values.
2. **"First value was corrupted by overlapping write!"** -- the assertion message describes exactly what happened.

This kind of bug is especially dangerous because it does not crash -- it produces wrong data silently. In a WASM application processing images, audio, or financial data, a misaligned write could produce subtly incorrect results that pass all obvious checks.
