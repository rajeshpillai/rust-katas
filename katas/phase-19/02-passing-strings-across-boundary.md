---
id: passing-strings-across-boundary
phase: 19
phase_title: "WASM Memory Model"
sequence: 2
title: Passing Strings Across the Boundary — Pointer and Length
hints:
  - WASM has no string type -- strings are just bytes in linear memory
  - A string must be passed as (pointer, length), not just a pointer
  - Without the length, the reader does not know where the string ends
  - Reading past the string into uninitialized memory produces invalid UTF-8
---

## Description

WASM has no built-in string type. When a WASM module needs to pass a string to the host (or vice versa), it writes the string's UTF-8 bytes into linear memory and communicates the offset and length. If the length is omitted, the host does not know where the string ends and may read into uninitialized or unrelated memory. Unlike C strings, WASM strings are not null-terminated -- the length is the only way to know the extent. This kata demonstrates the `(pointer, length)` protocol for passing strings across the WASM boundary.

## Broken Code

```rust
fn write_string_to_memory(memory: &mut Vec<u8>, s: &str) -> usize {
    let offset = memory.len();
    memory.extend_from_slice(s.as_bytes());
    // BUG: Only returns the offset, not the length.
    // The reader has no way to know where the string ends.
    offset
}

fn read_string_from_memory(memory: &[u8], offset: usize) -> String {
    // Without a length, we guess -- read 64 bytes and hope for the best.
    // This reads past the actual string into uninitialized memory.
    let end = std::cmp::min(offset + 64, memory.len());
    let bytes = &memory[offset..end];
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn main() {
    let mut memory = vec![0u8; 256];

    // Guest writes a short string
    let offset = write_string_to_memory(&mut memory, "Hello, WASM!");

    // Host tries to read it back without knowing the length
    let result = read_string_from_memory(&memory, offset);
    println!("Read: '{}'", result);

    // The result includes garbage bytes after the actual string
    assert_eq!(result, "Hello, WASM!", "String contains extra bytes!");
}
```

## Correct Code

```rust
/// Write a string to linear memory. Returns (offset, length).
fn write_string_to_memory(memory: &mut Vec<u8>, s: &str) -> (usize, usize) {
    let offset = memory.len();
    let length = s.len();
    memory.extend_from_slice(s.as_bytes());
    // Correct: return both offset and length
    (offset, length)
}

/// Read a string from linear memory using offset and length.
fn read_string_from_memory(
    memory: &[u8],
    offset: usize,
    length: usize,
) -> Result<String, String> {
    if offset + length > memory.len() {
        return Err(format!(
            "Out of bounds: offset {} + length {} > memory size {}",
            offset, length, memory.len()
        ));
    }
    let bytes = &memory[offset..offset + length];
    String::from_utf8(bytes.to_vec())
        .map_err(|e| format!("Invalid UTF-8: {}", e))
}

fn main() {
    let mut memory = vec![0u8; 256];

    // Guest writes a string and returns (offset, length)
    let (offset, length) = write_string_to_memory(&mut memory, "Hello, WASM!");
    println!("String written at offset={}, length={}", offset, length);

    // Host reads exactly the right number of bytes
    let result = read_string_from_memory(&memory, offset, length).unwrap();
    println!("Read: '{}'", result);
    assert_eq!(result, "Hello, WASM!");

    // Write a second string
    let (offset2, length2) = write_string_to_memory(&mut memory, "Rust is safe!");
    let result2 = read_string_from_memory(&memory, offset2, length2).unwrap();
    println!("Read: '{}'", result2);
    assert_eq!(result2, "Rust is safe!");
}
```

## Explanation

The broken version only returns the offset when writing a string to memory. The reader does not know how many bytes to read, so it guesses (64 bytes). Since the actual string `"Hello, WASM!"` is only 12 bytes, the reader picks up 52 extra bytes of zero-initialized memory. The `String::from_utf8` call may succeed (zeros are not valid UTF-8 for printable text, but `\0` bytes are technically valid UTF-8), but the resulting string will not match the original because it contains trailing null bytes.

**Why WASM strings are not null-terminated:**

C strings use a null byte (`\0`) to mark the end. This has well-known problems: you cannot embed null bytes in a string, and finding the length requires scanning the entire string (O(n)). WASM avoids this by requiring explicit lengths. The `(pointer, length)` pair is sometimes called a "fat pointer" -- it carries both the location and the extent of the data.

**The `wasm-bindgen` protocol:**

When `wasm-bindgen` passes a `String` from Rust to JavaScript:

1. The Rust side allocates space in linear memory and writes the UTF-8 bytes
2. It returns the offset and length to JavaScript
3. JavaScript reads the bytes from linear memory using `TextDecoder`
4. The Rust side frees the allocation

Without step 2 including the length, step 3 would read garbage.

**Generalizing beyond strings:**

This pattern applies to all variable-length data: `Vec<T>`, byte arrays, serialized structs. Anything that is not a fixed-size primitive must be passed as `(pointer, length)` across the WASM boundary. This is a fundamental protocol of WASM linear memory communication.

The invariant violated in the broken code: **variable-length data must be passed as (pointer, length); a pointer alone is insufficient because the reader cannot determine the extent of the data.**

## ⚠️ Caution

- WASM strings are NOT null-terminated like C strings. Always pass (pointer, length) pairs across the boundary. Assuming null termination causes reads beyond the intended data.
- UTF-8 validity is not guaranteed when reading bytes from linear memory. Always validate with `std::str::from_utf8()` or use `from_utf8_unchecked()` only with a safety comment.

## 💡 Tips

- The `wasm-bindgen` protocol for strings is: allocate in WASM, write from host, pass (ptr, len).
- For any variable-length data (strings, arrays, structs with dynamic fields), always use the (pointer, length) pattern.
- Consider passing UTF-8 validation responsibility to the producer side to avoid redundant checks.

## Compiler Error Interpretation

```
thread 'main' panicked at 'assertion `left == right` failed: String contains extra bytes!
  left: "Hello, WASM!\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"
 right: "Hello, WASM!"', src/main.rs:19:5
```

This is a runtime panic from the assertion:

1. **The left value** contains the 12 bytes of `"Hello, WASM!"` followed by 52 null bytes. The reader grabbed 64 bytes total because it did not know the string's length.
2. **The right value** is the expected clean string with exactly 12 bytes.
3. The strings are not equal because the left side has trailing garbage.

In some cases, the extra bytes might not be zero -- they could be data from a previous allocation, another string, or uninitialized memory. This would either produce an invalid UTF-8 error (panic on `unwrap()`) or a string containing random characters. Either way, the data is corrupted because the reader did not know where to stop.
