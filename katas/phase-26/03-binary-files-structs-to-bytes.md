---
id: binary-files-structs-to-bytes
phase: 26
phase_title: "File I/O in Rust"
sequence: 3
title: "Binary Files: Structs to Bytes"
hints:
  - "`write_all` expects `&[u8]` — a slice of bytes. A struct reference is not a byte slice."
  - "Rust has no automatic serialization. You must convert each field to bytes individually."
  - "Use `to_le_bytes()` on numeric types to get a fixed-size byte array, then write each field separately."
---

## Description

Binary files store raw bytes rather than human-readable text. They are compact and fast to parse, but require explicit conversion between Rust types and byte representations.

Every numeric primitive in Rust provides:
- `to_le_bytes()` — converts to little-endian byte array
- `to_be_bytes()` — converts to big-endian byte array
- `from_le_bytes()` / `from_be_bytes()` — converts back

There is **no implicit serialization** in Rust. You cannot simply write a struct to a file — you must convert each field to bytes explicitly.

## Broken Code

```rust
use std::fs::File;
use std::io::{Write, Read};

struct SensorReading {
    timestamp: u64,
    temperature: f32,
    humidity: f32,
}

fn main() {
    let path = std::env::temp_dir().join("kata-binary-demo.bin");
    let reading = SensorReading {
        timestamp: 1700000000,
        temperature: 22.5,
        humidity: 65.0,
    };

    // Write the struct to a binary file
    let mut file = File::create(&path).unwrap();
    file.write_all(&reading).unwrap(); // Trying to write struct as bytes

    // Read it back
    let mut file = File::open(&path).unwrap();
    let mut buf = vec![0u8; 16]; // 8 + 4 + 4 bytes
    file.read_exact(&mut buf).unwrap();
    println!("Read {} bytes", buf.len());
}
```

## Correct Code

```rust
use std::fs::File;
use std::io::{Write, Read};

struct SensorReading {
    timestamp: u64,
    temperature: f32,
    humidity: f32,
}

impl SensorReading {
    // Serialize to bytes: u64 (8) + f32 (4) + f32 (4) = 16 bytes
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16);
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.temperature.to_le_bytes());
        bytes.extend_from_slice(&self.humidity.to_le_bytes());
        bytes
    }

    // Deserialize from bytes
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 16 {
            return None;
        }
        let timestamp = u64::from_le_bytes(bytes[0..8].try_into().ok()?);
        let temperature = f32::from_le_bytes(bytes[8..12].try_into().ok()?);
        let humidity = f32::from_le_bytes(bytes[12..16].try_into().ok()?);
        Some(SensorReading { timestamp, temperature, humidity })
    }
}

fn main() {
    let path = std::env::temp_dir().join("kata-binary-demo.bin");

    // Write three readings
    let readings = vec![
        SensorReading { timestamp: 1700000000, temperature: 22.5, humidity: 65.0 },
        SensorReading { timestamp: 1700000060, temperature: 22.7, humidity: 64.8 },
        SensorReading { timestamp: 1700000120, temperature: 23.1, humidity: 63.5 },
    ];

    let mut file = File::create(&path).unwrap();
    for reading in &readings {
        file.write_all(&reading.to_bytes()).unwrap();
    }

    // Read all readings back
    let mut file = File::open(&path).unwrap();
    let mut all_bytes = Vec::new();
    file.read_to_end(&mut all_bytes).unwrap();

    println!("File size: {} bytes ({} records)", all_bytes.len(), all_bytes.len() / 16);

    for (i, chunk) in all_bytes.chunks(16).enumerate() {
        if let Some(reading) = SensorReading::from_bytes(chunk) {
            println!(
                "Reading {}: ts={}, temp={:.1}°C, humidity={:.1}%",
                i, reading.timestamp, reading.temperature, reading.humidity
            );
        }
    }

    std::fs::remove_file(&path).unwrap();
}
```

## Explanation

The broken code tries to pass `&reading` (a reference to a `SensorReading` struct) to `write_all`, which expects `&[u8]` (a byte slice). Rust has no implicit conversion from arbitrary structs to bytes — this is a compile-time type mismatch.

**Why Rust requires explicit serialization:**

In C, you might write `fwrite(&struct, sizeof(struct), 1, file)` and it "works" by reinterpreting the struct's memory as bytes. This is fragile because:
- Struct layout depends on compiler padding and alignment
- Byte order depends on the CPU architecture
- The resulting file is not portable

Rust forces you to be explicit about the byte representation. This makes binary formats **portable** and **predictable**.

**The serialization pattern:**

```rust
// Writing: type → bytes
let bytes: [u8; 8] = value.to_le_bytes();
file.write_all(&bytes)?;

// Reading: bytes → type
let mut buf = [0u8; 8];
file.read_exact(&mut buf)?;
let value = u64::from_le_bytes(buf);
```

Each numeric type knows its exact byte size:

| Type | Size | Method |
|---|---|---|
| `u8` / `i8` | 1 byte | `to_le_bytes()` → `[u8; 1]` |
| `u16` / `i16` | 2 bytes | `to_le_bytes()` → `[u8; 2]` |
| `u32` / `i32` / `f32` | 4 bytes | `to_le_bytes()` → `[u8; 4]` |
| `u64` / `i64` / `f64` | 8 bytes | `to_le_bytes()` → `[u8; 8]` |

**`read_exact` vs `read`:** `read_exact` reads exactly N bytes or returns an error. Plain `read` may return fewer bytes than requested (partial reads). For binary records, always use `read_exact`.

**Byte slicing with `try_into`:** `from_le_bytes` takes a fixed-size array (e.g., `[u8; 8]`), but slicing a `Vec<u8>` gives `&[u8]`. The `.try_into()` call converts the slice to a fixed-size array, returning `Err` if the lengths do not match.

## ⚠️ Caution

- Little-endian (`le`) is standard for file storage on most platforms. Big-endian (`be`) is used for network protocols. Pick one and document it.
- `read_exact` panics if the file has fewer bytes than expected. Always check file size or handle the error.
- Do not use `std::mem::transmute` or pointer casts to convert structs to bytes — this is `unsafe`, non-portable, and breaks if struct layout changes.

## 💡 Tips

- Define a `RECORD_SIZE` constant derived from field sizes (e.g., `const RECORD_SIZE: usize = 8 + 4 + 4;`) rather than hardcoding magic numbers.
- The `.chunks(N)` method on slices is perfect for iterating over fixed-size records read from a binary file.
- For complex serialization needs in real projects, use the `serde` + `bincode` crates. Manual byte conversion is best for learning and for simple, performance-critical formats.

## Compiler Error Interpretation

```
error[E0308]: mismatched types
 --> main.rs:18:21
  |
18|     file.write_all(&reading).unwrap();
  |          --------- ^^^^^^^^ expected `&[u8]`, found `&SensorReading`
  |          |
  |          arguments to this method are incorrect
  |
note: method defined here
 --> /rustc/.../library/std/src/io/mod.rs
  |
  |     fn write_all(&mut self, buf: &[u8]) -> Result<()>;
  |        ^^^^^^^^^
```

This error clearly states:

- **"expected `&[u8]`, found `&SensorReading`"** — `write_all` works with raw bytes, not arbitrary types. A struct is not bytes.
- **No `From` or `Into` conversion exists** between your struct and `&[u8]`. You must write the conversion yourself.
- Unlike languages with reflection or runtime serialization, Rust requires you to explicitly define how your data maps to bytes. This is the cost of zero-cost abstractions — nothing is hidden.

---

| [Prev: Buffered I/O and Line-by-Line Reading](#/katas/buffered-io-and-lines) | [Next: Delimited File Parsing](#/katas/delimited-file-parsing) |
