---
id: repr-c-abi-layout
phase: 18
phase_title: "Rust to WASM Toolchain"
sequence: 1
title: ABI-Stable Layout — Why repr(C) Matters for WASM
hints:
  - Rust is free to reorder struct fields for optimization unless told otherwise
  - "#[repr(C)]" guarantees fields are laid out in declaration order
  - When data crosses a boundary as raw bytes, both sides must agree on the layout
  - Use std::mem::size_of and pointer casting to inspect the byte-level layout
---

## Description

When Rust compiles to WASM, data that crosses the boundary between the WASM module and the host is just bytes in linear memory. Both sides must agree on exactly where each field lives within those bytes. Without `#[repr(C)]`, the Rust compiler is free to reorder struct fields to minimize padding -- and different compiler versions or targets may choose different orderings. This means a struct serialized to bytes on one side may be deserialized with scrambled fields on the other. `#[repr(C)]` locks the field order to match the C ABI: fields appear in declaration order with predictable padding.

## Broken Code

```rust
// No #[repr(C)] -- Rust may reorder fields!
struct Header {
    version: u8,
    flags: u32,
    length: u16,
}

fn serialize_header(header: &Header) -> Vec<u8> {
    let size = std::mem::size_of::<Header>();
    let ptr = header as *const Header as *const u8;
    // Safety: we are reading the raw bytes of the struct
    let bytes = unsafe { std::slice::from_raw_parts(ptr, size) };
    bytes.to_vec()
}

fn deserialize_header(bytes: &[u8]) -> (u8, u32, u16) {
    // BUG: Assumes fields are in declaration order: version, flags, length.
    // Without repr(C), the compiler may have reordered them.
    // We read at offsets assuming: version at 0, flags at 4, length at 8.
    let version = bytes[0];
    let flags = u32::from_ne_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let length = u16::from_ne_bytes([bytes[8], bytes[9]]);
    (version, flags, length)
}

fn main() {
    let header = Header {
        version: 1,
        flags: 0xDEADBEEF,
        length: 1024,
    };

    let bytes = serialize_header(&header);
    let (v, f, l) = deserialize_header(&bytes);

    // These assertions may fail because the compiler reordered the fields
    assert_eq!(v, 1, "version mismatch");
    assert_eq!(f, 0xDEADBEEF, "flags mismatch");
    assert_eq!(l, 1024, "length mismatch");
    println!("Deserialized: version={}, flags={:#X}, length={}", v, f, l);
}
```

## Correct Code

```rust
// Correct: #[repr(C)] guarantees field order matches declaration order
#[repr(C)]
struct Header {
    version: u8,
    _pad1: [u8; 3],  // Explicit padding for clarity
    flags: u32,
    length: u16,
    _pad2: [u8; 2],  // Pad to align the struct size
}

fn serialize_header(header: &Header) -> Vec<u8> {
    let size = std::mem::size_of::<Header>();
    let ptr = header as *const Header as *const u8;
    let bytes = unsafe { std::slice::from_raw_parts(ptr, size) };
    bytes.to_vec()
}

fn deserialize_header(bytes: &[u8]) -> (u8, u32, u16) {
    // With repr(C), the layout is predictable:
    // offset 0: version (u8), offset 1-3: padding
    // offset 4: flags (u32), offset 8: length (u16)
    let version = bytes[0];
    let flags = u32::from_ne_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let length = u16::from_ne_bytes([bytes[8], bytes[9]]);
    (version, flags, length)
}

fn main() {
    let header = Header {
        version: 1,
        _pad1: [0; 3],
        flags: 0xDEADBEEF,
        length: 1024,
        _pad2: [0; 2],
    };

    println!("Header size: {} bytes", std::mem::size_of::<Header>());

    let bytes = serialize_header(&header);
    let (v, f, l) = deserialize_header(&bytes);

    assert_eq!(v, 1, "version mismatch");
    assert_eq!(f, 0xDEADBEEF, "flags mismatch");
    assert_eq!(l, 1024, "length mismatch");
    println!("Deserialized: version={}, flags={:#X}, length={}", v, f, l);
}
```

## Explanation

The broken version defines `Header` without `#[repr(C)]`. Rust's default representation (`repr(Rust)`) allows the compiler to reorder fields to minimize padding and improve alignment. For a struct with `u8`, `u32`, and `u16` fields, the compiler might rearrange them as `flags` (4 bytes), `length` (2 bytes), `version` (1 byte) to reduce padding. The deserialization code assumes the fields are at fixed offsets based on declaration order, which may not match the actual layout.

**What `#[repr(C)]` guarantees:**

1. Fields are laid out in **declaration order** -- the first field comes first in memory.
2. Padding follows **C rules** -- each field is aligned to its natural alignment, and the struct is padded to a multiple of its largest field's alignment.
3. The layout is **deterministic** across compilations, platforms, and compiler versions (for the same target).

**Why this matters for WASM:**

When WASM modules communicate with the host through linear memory, structured data is written as raw bytes at a specific offset. The host reads those bytes and interprets them according to a shared layout contract. If the layout is not deterministic, the host reads garbage. `#[repr(C)]` is mandatory for any type that crosses the WASM boundary.

**The explicit padding:**

The correct version includes `_pad1` and `_pad2` fields to make the padding visible. This is good practice for cross-boundary types because it makes the layout self-documenting. Without explicit padding, the C layout rules still insert padding, but it is invisible and easy to forget about.

The invariant violated in the broken code: **data that crosses a module boundary must have a deterministic layout; `#[repr(C)]` is required for any struct that is serialized to raw bytes.**

## Compiler Error Interpretation

```
thread 'main' panicked at 'assertion `left == right` failed: flags mismatch
  left: 256
 right: 3735928559', src/main.rs:31:5
```

This is a runtime panic from a failed assertion. The deserialized `flags` value is wrong because the bytes were read from the wrong offset:

1. **"assertion `left == right` failed"** -- `assert_eq!` compared the deserialized value with the expected value and they did not match.
2. **"left: 256, right: 3735928559"** -- `3735928559` is `0xDEADBEEF` (the value we stored). `256` is what we read back from the wrong byte offset -- the compiler reordered the fields, so the bytes at offset 4 are not `flags`.

This is a silent data corruption bug. The code compiles, runs without crashing (usually), and produces wrong results. In a WASM context, this means the host and guest disagree on the data layout, leading to corrupted communication across the boundary.
