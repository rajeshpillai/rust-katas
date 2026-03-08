---
id: data-processor-pipeline
phase: 25
phase_title: "Capstone Projects"
sequence: 3
title: "Capstone: Data Processing Pipeline with Stable ABI"
hints:
  - Include version and header_size fields so the reader can adapt to different versions
  - The reader should use header_size to find the payload, not hardcode struct size
  - Modules compiled at different times may use different struct versions
  - The pipeline must handle version mismatches gracefully, not silently corrupt data
---

## Description

This capstone combines `#[repr(C)]` layout (Phase 18), versioned headers (Phase 20), zero-copy slices (Phase 22), and opaque handles (Phase 22) into a multi-stage data processing pipeline. Two processing stages communicate through a shared memory buffer using a self-describing binary protocol. Each message includes a version number and header size so the reader can handle different protocol versions without corruption. This is the architecture of multi-module WASM applications and component model proposals.

## Broken Code

```rust
/// V1 message header
#[repr(C)]
struct MessageV1 {
    kind: u32,       // 0 = data, 1 = control
    payload_len: u32,
}

/// V2 message header adds a priority field
#[repr(C)]
struct MessageV2 {
    kind: u32,
    priority: u32,    // New in V2!
    payload_len: u32,
}

fn write_message_v2(buffer: &mut Vec<u8>, kind: u32, priority: u32, payload: &[u8]) {
    let header = MessageV2 {
        kind,
        priority,
        payload_len: payload.len() as u32,
    };
    let header_bytes = unsafe {
        std::slice::from_raw_parts(
            &header as *const MessageV2 as *const u8,
            std::mem::size_of::<MessageV2>(),
        )
    };
    buffer.extend_from_slice(header_bytes);
    buffer.extend_from_slice(payload);
}

fn read_message_v1(buffer: &[u8]) -> (u32, &[u8]) {
    // BUG: Assumes V1 layout. Reads payload_len from wrong offset
    // if the writer used V2 format (which has an extra field).
    let kind = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    let payload_len = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
    let payload_start = std::mem::size_of::<MessageV1>();
    let payload = &buffer[payload_start..payload_start + payload_len as usize];
    (kind, payload)
}

fn main() {
    let mut buffer = Vec::new();

    // Stage 1 writes with V2 format
    let payload = b"Hello, Pipeline!";
    write_message_v2(&mut buffer, 0, 5, payload);

    // Stage 2 reads with V1 assumptions -- gets wrong payload_len!
    let (kind, read_payload) = read_message_v1(&buffer);

    println!("Kind: {}", kind);
    println!("Payload: {:?}", std::str::from_utf8(read_payload));

    assert_eq!(
        read_payload, payload,
        "Payload mismatch -- ABI version mismatch between pipeline stages!"
    );
}
```

## Correct Code

```rust
#[derive(Debug)]
enum PipelineError {
    BufferTooSmall { needed: usize, have: usize },
    UnknownVersion(u32),
    InvalidPayload(String),
}

/// Self-describing message header. The first two fields are always
/// version and header_size, so ANY reader can skip the header correctly.
#[repr(C)]
struct MessageHeader {
    version: u32,
    header_size: u32,
    kind: u32,
    priority: u32,     // Added in V2, zero for V1
    _reserved: [u32; 2], // Reserved for future versions
}

impl MessageHeader {
    fn new(kind: u32, priority: u32) -> Self {
        MessageHeader {
            version: 2,
            header_size: std::mem::size_of::<Self>() as u32,
            kind,
            priority,
            _reserved: [0; 2],
        }
    }
}

fn write_message(buffer: &mut Vec<u8>, kind: u32, priority: u32, payload: &[u8]) {
    let header = MessageHeader::new(kind, priority);
    let header_bytes = unsafe {
        std::slice::from_raw_parts(
            &header as *const MessageHeader as *const u8,
            std::mem::size_of::<MessageHeader>(),
        )
    };
    buffer.extend_from_slice(header_bytes);

    // Write payload length as u32 after header
    buffer.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buffer.extend_from_slice(payload);
}

fn read_message(buffer: &[u8]) -> Result<(u32, u32, u32, Vec<u8>), PipelineError> {
    // Step 1: Read version and header_size (always at fixed offsets)
    if buffer.len() < 8 {
        return Err(PipelineError::BufferTooSmall { needed: 8, have: buffer.len() });
    }

    let version = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    let header_size = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]) as usize;

    if buffer.len() < header_size + 4 {
        return Err(PipelineError::BufferTooSmall {
            needed: header_size + 4,
            have: buffer.len(),
        });
    }

    // Step 2: Read kind (always at offset 8)
    let kind = u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);

    // Step 3: Read priority (only in V2+, at offset 12)
    let priority = if version >= 2 && header_size >= 16 {
        u32::from_le_bytes([buffer[12], buffer[13], buffer[14], buffer[15]])
    } else {
        0 // Default for V1
    };

    // Step 4: Read payload length (always right after header, using header_size)
    let payload_len_offset = header_size;
    let payload_len = u32::from_le_bytes([
        buffer[payload_len_offset],
        buffer[payload_len_offset + 1],
        buffer[payload_len_offset + 2],
        buffer[payload_len_offset + 3],
    ]) as usize;

    let payload_start = payload_len_offset + 4;
    if buffer.len() < payload_start + payload_len {
        return Err(PipelineError::BufferTooSmall {
            needed: payload_start + payload_len,
            have: buffer.len(),
        });
    }

    let payload = buffer[payload_start..payload_start + payload_len].to_vec();

    Ok((version, kind, priority, payload))
}

fn main() {
    let mut buffer = Vec::new();

    // Stage 1: Write a V2 message with priority
    let payload = b"Hello, Pipeline!";
    write_message(&mut buffer, 0, 5, payload);
    println!("Wrote {} bytes to buffer", buffer.len());

    // Stage 2: Read with version-aware reader
    match read_message(&buffer) {
        Ok((version, kind, priority, read_payload)) => {
            println!("Version: {}", version);
            println!("Kind: {}", kind);
            println!("Priority: {}", priority);
            println!("Payload: {:?}", std::str::from_utf8(&read_payload).unwrap());

            assert_eq!(read_payload.as_slice(), payload, "Payload should match");
            assert_eq!(priority, 5, "Priority should be 5");
        }
        Err(e) => println!("Error: {:?}", e),
    }

    // Demonstrate forward compatibility: a V3 writer could add more fields,
    // and this reader would still work because it uses header_size to find
    // the payload, not hardcoded offsets.
    println!("\nPipeline processed successfully with versioned ABI!");
}
```

## Explanation

The broken version has two incompatible struct definitions: `MessageV1` (8 bytes: kind + payload_len) and `MessageV2` (12 bytes: kind + priority + payload_len). The writer uses V2 format, but the reader assumes V1 layout. The reader reads `priority` (value 5) at offset 4 where it expects `payload_len`. It then tries to read 5 bytes of "payload" starting at offset 8, getting the actual `payload_len` field and part of the real payload -- corrupted data.

**The self-describing protocol:**

The correct version uses a versioned header where the first two fields are always:
1. `version: u32` -- which version of the protocol this message uses
2. `header_size: u32` -- how many bytes to skip to find the payload

Any reader, regardless of which version it was compiled against, can:
1. Read `header_size` to skip the header correctly
2. Read `version` to know which fields are available
3. Use default values for fields added in newer versions

**Why `header_size` is critical:**

A reader that only checks `version` must know the exact size of every version. Adding version 3 requires updating all readers. But `header_size` allows the reader to skip unknown fields: "I do not know what V3 added, but I know the header is 32 bytes, so the payload starts at byte 32."

**This capstone integrates:**
- **Phase 18:** `#[repr(C)]` for deterministic layout
- **Phase 19:** Offset arithmetic for reading fields from byte buffers
- **Phase 20:** Versioned headers for forward-compatible protocols
- **Phase 22:** Zero-copy reads where possible (the reader uses buffer slices)

**Real-world parallels:**
- **WASI:** Every WASI syscall structure includes version fields
- **Protocol Buffers:** Wire format is self-describing with field tags
- **WebGPU descriptors:** Every GPU descriptor includes `nextInChain` for extensibility
- **ELF headers:** Binary format includes version and header size fields

The invariant violated in the broken code: **binary protocols between modules must be self-describing; include version and header size so readers can adapt to different protocol versions without corruption.**

## ⚠️ Caution

- Binary protocol corruption is silent — there is no runtime error until you try to interpret garbage data. Always include checksums or magic numbers for validation.
- Versioned headers must be read first and validated before interpreting any subsequent fields. Reading fields before checking the version is a protocol violation.

## 💡 Tips

- Use the pattern: magic number -> version -> size -> payload for every binary protocol.
- Test with corrupted inputs (wrong version, truncated data, invalid offsets) to verify error handling.
- Study real protocols: ELF headers, Protocol Buffers wire format, WASI snapshot versions.

## Compiler Error Interpretation

```
thread 'main' panicked at 'assertion `left == right` failed: Payload mismatch -- ABI version mismatch between pipeline stages!
  left: [16, 0, 0, 0, 72]
 right: [72, 101, 108, 108, 111, 44, 32, 80, 105, 112, 101, 108, 105, 110, 101, 33]', src/main.rs:44:5
```

This is a runtime panic from the payload assertion:

1. **left: `[16, 0, 0, 0, 72]`** -- the reader got 5 bytes of garbage. The `16, 0, 0, 0` is actually the `payload_len` field (value 16, little-endian), and `72` is the first byte of "H" from "Hello". The reader read these as "payload" because it got the wrong `payload_len` from the wrong offset.
2. **right: `[72, 101, 108, 108, 111, ...]`** -- the expected payload "Hello, Pipeline!" (UTF-8 bytes).

The ABI mismatch caused the reader to misinterpret the binary layout, producing corrupted output. In a real multi-module WASM pipeline processing financial data or medical records, this kind of silent corruption could have catastrophic consequences.
