---
id: stable-abi-versioning
phase: 20
phase_title: "Host and Guest Contracts"
sequence: 3
title: Stable Interface Versioning — Evolving Without Breaking
hints:
  - Adding a field to a repr(C) struct changes its size
  - If the reader expects the old size, it will read the wrong offsets for subsequent data
  - Include a version field and a header_size field so the reader can adapt
  - Reserved padding bytes allow adding fields without changing the struct size
---

## Description

WASM module interfaces must be stable across versions. When the guest evolves its data format (adding a field, changing a type), the host must still be able to read the data correctly. If the host assumes the old struct size but the guest sends the new (larger) struct, the host will read data at wrong offsets, producing silent corruption. This kata demonstrates the versioned header pattern: include a `version` and `header_size` field so the reader can adapt to different versions of the data format.

## Broken Code

```rust
/// Version 1 of the message format
#[repr(C)]
struct MessageV1 {
    kind: u32,
    payload_offset: u32,
}

/// Version 2 adds a flags field
#[repr(C)]
struct MessageV2 {
    kind: u32,
    flags: u32,        // New field!
    payload_offset: u32,
}

fn read_message_v1(bytes: &[u8]) -> (u32, u32) {
    // Assumes V1 layout: kind at offset 0, payload_offset at offset 4
    let kind = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let payload_offset = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    (kind, payload_offset)
}

fn main() {
    // The guest sends a V2 message
    let msg = MessageV2 {
        kind: 1,
        flags: 0xFF,
        payload_offset: 64,
    };

    // Serialize V2 to bytes
    let size = std::mem::size_of::<MessageV2>();
    let ptr = &msg as *const MessageV2 as *const u8;
    let bytes = unsafe { std::slice::from_raw_parts(ptr, size) };

    // BUG: Host reads with V1 layout -- gets wrong payload_offset!
    // V1 expects payload_offset at offset 4, but in V2 it is at offset 8
    // (because flags occupies offset 4-7).
    let (kind, payload_offset) = read_message_v1(bytes);

    println!("Kind: {}", kind);
    println!("Payload offset: {}", payload_offset);

    // payload_offset should be 64, but we read the flags field (0xFF) instead
    assert_eq!(payload_offset, 64, "Payload offset is wrong -- ABI mismatch!");
}
```

## Correct Code

```rust
/// Versioned header: always starts with version and header_size.
/// The reader uses header_size to skip to the payload, regardless of version.
#[repr(C)]
struct MessageHeader {
    version: u32,
    header_size: u32,
    kind: u32,
    // V2 fields
    flags: u32,
    // Reserved for future versions
    _reserved: [u32; 4],
}

impl MessageHeader {
    fn new_v1(kind: u32) -> Self {
        MessageHeader {
            version: 1,
            header_size: std::mem::size_of::<Self>() as u32,
            kind,
            flags: 0,
            _reserved: [0; 4],
        }
    }

    fn new_v2(kind: u32, flags: u32) -> Self {
        MessageHeader {
            version: 2,
            header_size: std::mem::size_of::<Self>() as u32,
            kind,
            flags,
            _reserved: [0; 4],
        }
    }
}

fn read_message(bytes: &[u8]) -> Result<(u32, u32, u32), String> {
    if bytes.len() < 8 {
        return Err("Buffer too small for header".to_string());
    }

    let version = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let header_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

    if bytes.len() < header_size as usize {
        return Err(format!("Buffer too small: need {}, have {}", header_size, bytes.len()));
    }

    // kind is always at offset 8, regardless of version
    let kind = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

    // flags is only present in V2+
    let flags = if version >= 2 {
        u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]])
    } else {
        0 // Default for V1 messages
    };

    Ok((version, kind, flags))
}

fn main() {
    // V1 message
    let v1 = MessageHeader::new_v1(1);
    let v1_size = std::mem::size_of::<MessageHeader>();
    let v1_ptr = &v1 as *const MessageHeader as *const u8;
    let v1_bytes = unsafe { std::slice::from_raw_parts(v1_ptr, v1_size) };

    let (ver, kind, flags) = read_message(v1_bytes).unwrap();
    println!("V1 message: version={}, kind={}, flags={}", ver, kind, flags);

    // V2 message -- same reader handles it correctly
    let v2 = MessageHeader::new_v2(2, 0xFF);
    let v2_size = std::mem::size_of::<MessageHeader>();
    let v2_ptr = &v2 as *const MessageHeader as *const u8;
    let v2_bytes = unsafe { std::slice::from_raw_parts(v2_ptr, v2_size) };

    let (ver, kind, flags) = read_message(v2_bytes).unwrap();
    println!("V2 message: version={}, kind={}, flags={:#X}", ver, kind, flags);
    assert_eq!(flags, 0xFF, "Flags should be 0xFF");

    println!("Both versions handled correctly by the same reader!");
}
```

## Explanation

The broken version has two struct definitions (`MessageV1` and `MessageV2`) with different sizes. When the guest sends a V2 message but the host reads it with V1 assumptions, the host reads `flags` (at offset 4) where it expects `payload_offset`. The data is silently corrupted -- the host gets `0xFF` instead of `64`.

**The versioned header pattern:**

A stable binary protocol always starts with:

1. **`version: u32`** -- tells the reader which fields to expect
2. **`header_size: u32`** -- tells the reader how many bytes to skip to find the payload, regardless of how many fields were added

This pattern is used by:
- WASI (WebAssembly System Interface) -- every syscall uses versioned structures
- WebGPU WASM bindings -- GPU descriptor structures include version fields
- Protocol Buffers / FlatBuffers -- wire format includes schema version

**Reserved fields:**

The correct version includes `_reserved: [u32; 4]` -- 16 bytes of space reserved for future fields. When a future V3 needs a new field, it can use a reserved slot without changing the struct size. Old readers ignore reserved fields (they read as zeros). This avoids the need to resize the struct, which would break all existing readers.

**Why this matters for WASM:**

WASM modules are often independently deployable. A host might run modules compiled at different times, with different versions of the data format. If the format is not forward-compatible, upgrading any module forces upgrading all modules simultaneously -- defeating the purpose of modular architecture.

The invariant violated in the broken code: **binary protocols that cross module boundaries must include version and size information so readers can handle different versions without corruption.**

## ⚠️ Caution

- Reserved fields in versioned headers only help if readers check the version before interpreting the struct. Without version checks, reserved fields provide no safety.
- Changing field alignment in a versioned struct is an ABI break even if the version number is bumped — older readers may access misaligned data.

## 💡 Tips

- Include version + size in every binary protocol header so readers can skip unknown fields.
- Add reserved fields for future expansion — they cost nothing and prevent ABI breaks.
- Study real-world examples: WASI, Protocol Buffers, ELF headers.

## Compiler Error Interpretation

```
thread 'main' panicked at 'assertion `left == right` failed: Payload offset is wrong -- ABI mismatch!
  left: 255
 right: 64', src/main.rs:35:5
```

This is a runtime panic from a failed assertion:

1. **"left: 255, right: 64"** -- the host read `255` (`0xFF`) where it expected `64`. The value `0xFF` is the `flags` field from the V2 struct, not the `payload_offset`. The host was reading at the wrong offset.
2. **"ABI mismatch!"** -- the assertion message diagnoses the problem: the host and guest disagree on the data layout.

This is silent data corruption. If there were no assertion, the host would proceed to read the "payload" at offset 255 instead of offset 64, producing further cascading corruption. In WASM systems with multiple modules communicating through shared memory, one ABI mismatch can corrupt an entire pipeline.

---

| [Prev: Error Handling Across the Boundary — No Panics Allowed](#/katas/error-handling-across-boundary) | [Next: Compute Kernel vs Glue Code — Separation of Concerns](#/katas/compute-vs-glue) |
