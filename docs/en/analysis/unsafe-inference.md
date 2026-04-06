# Unsafe Type Inference Engine

> Heuristic-based type detection for FFI allocations and raw pointers

---

## Overview

**File:** `src/analysis/unsafe_inference/engine.rs` (670 lines)

When type information is unavailable — such as for FFI allocations, raw pointers, or memory obtained through `malloc`/`calloc` — the inference engine uses **multi-dimensional heuristic analysis** to guess the likely type.

It operates on raw byte patterns, not Rust type metadata, making it applicable to any memory region regardless of origin.

---

## Design: Six-Dimensional Scoring

The engine uses a **scoring model** where six independent dimensions each contribute evidence toward different type hypotheses. The type with the highest aggregate score wins.

```
                    ┌──────────────────────┐
                    │   Score Aggregator   │
                    │   max(score) → Guess │
                    └──────────┬───────────┘
         ┌─────────┬──────────┼──────────┬─────────┐
         ▼         ▼          ▼          ▼         ▼
      Size     Layout     Content    Pointer   Power-of-Two
   Heuristic  Detect    Analysis   Counting    Capacity
    (O(1))    (O(1))     (O(n))     (O(n))       (O(1))
```

### Internal Score Structure

```rust
// engine.rs:126-135
#[derive(Default)]
struct Score {
    vec: u8,       // Evidence for Vec<T>
    string: u8,    // Evidence for String
    cstring: u8,   // Evidence for C-style null-terminated string
    pointer: u8,   // Evidence for raw pointer (*mut T, *const T)
    fat_ptr: u8,   // Evidence for fat pointer (&[T], &str, dyn Trait)
    buffer: u8,    // Evidence for raw byte buffer ([u8])
    cstruct: u8,   // Evidence for C struct with pointer fields
}
```

---

## Dimension 1: Size Heuristic

**File:** `engine.rs:226-255`

Maps allocation size to likely type categories based on common Rust/C type layouts:

```rust
fn size_heuristic(size: usize, score: &mut Score) {
    match size {
        // Raw pointer: *mut T, *const T, &T, Box<T>
        // Reduced from 60 to 30 to reduce false positives
        8 => score.pointer += 30,

        // Fat pointer: &[T], &str, dyn Trait (data_ptr + metadata)
        16 => score.fat_ptr += 25,

        // Vec/String triplet: (ptr, len, cap) — 3 × usize = 24 bytes
        24 => {
            score.vec += 15;
            score.string += 15;
        }

        // Common C struct sizes
        32 | 48 | 64 => score.cstruct += 10,

        _ => {}
    }

    // Power-of-two signal: Rust Vec capacity grows by powers of 2
    // A random size being power-of-two has probability ~1/size
    // For 64 bytes, this is only ~1.5% false positive rate
    if size.is_power_of_two() && size >= 64 {
        score.vec += 10;
        score.buffer += 5;
    }
}
```

**Why power-of-two is a strong signal:** Rust's `Vec` growth strategy is `new_cap = max(old_cap * 2, 1)`. So actual heap buffer sizes are almost always powers of 2.

---

## Dimension 2: Vec/String Layout Detection

**File:** `engine.rs:261-303`

Detects the `(ptr, len, cap)` triplet structure that both `Vec<T>` and `String` share:

```rust
fn vec_string_layout(view: &MemoryView, score: &mut Score) {
    if view.len() < 24 { return; }

    let ptr_val = view.read_usize(0);
    let len     = view.read_usize(8);
    let cap     = view.read_usize(16);

    let (Some(p), Some(l), Some(c)) = (ptr_val, len, cap) else { return; };

    // Basic structure validation
    if !is_valid_ptr(p) || c < l || c == 0 || c > 10_000_000 { return; }

    // Distinguish Vec from String based on spare capacity
    let spare = c.saturating_sub(l);

    if spare < 16 && l > 0 {
        // Small spare capacity → more likely String
        score.string += 50;
        score.vec += 20;
    } else if spare > 0 {
        // Large spare capacity → more likely Vec (pre-allocated growth space)
        score.vec += 60;
        score.string += 15;
    } else {
        // cap == len → could be either
        score.vec += 30;
        score.string += 30;
    }

    // Additional Vec signal: capacity is power of two
    if c.is_power_of_two() {
        score.vec += 15;
    }
}
```

**How it distinguishes Vec from String:**
- `String` typically has `cap` close to `len` (small spare capacity) because strings are built incrementally
- `Vec` often has `cap >> len` (large spare capacity) due to pre-allocation growth strategy

---

## Dimension 3: Content Analysis

**File:** `engine.rs:308-438`

Three sub-analyses on the raw memory content:

### 3a. Enhanced CString Detection

```rust
fn cstring_enhanced(data: &[u8], score: &mut Score) {
    // Find first null byte
    let null_pos = match data.iter().position(|&b| b == 0) {
        Some(pos) => pos,
        None => return,  // No null terminator
    };

    if null_pos < 3 { return; }  // Too short

    let content = &data[..null_pos];

    // Count printable ASCII characters (0x20-0x7E)
    let printable_count = content.iter()
        .filter(|&&b| (0x20..=0x7E).contains(&b))
        .count();
    let printable_ratio = printable_count as f32 / content.len() as f32;

    // High printable ratio → likely CString
    if printable_ratio > 0.9 { score.cstring += 70; }
    else if printable_ratio > 0.7 { score.cstring += 40; }
    else if printable_ratio > 0.5 { score.cstring += 20; }

    // Multiple nulls → likely binary data, not CString
    let null_count = data.iter().filter(|&&b| b == 0).count();
    if null_count > 1 {
        score.cstring = score.cstring.saturating_sub(20);
        score.buffer += 15;
    }
}
```

### 3b. Shannon Entropy Analysis

```rust
fn entropy_analysis(data: &[u8], score: &mut Score) {
    let entropy = shannon_entropy(data);

    // High entropy → compressed/encrypted/serialized data
    if entropy > 7.5 { score.buffer += 30; }
    else if entropy > 6.5 { score.buffer += 15; }
    // Low entropy → repetitive data or text
    else if entropy < 3.0 { score.cstruct += 5; }
}

fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() { return 0.0; }
    let mut freq = [0u32; 256];
    for &b in data { freq[b as usize] += 1; }
    let n = data.len() as f64;
    let mut entropy = 0.0;
    for &count in &freq {
        if count > 0 {
            let p = count as f64 / n;
            entropy -= p * p.log2();
        }
    }
    entropy
}
```

**Typical entropy values:**

| Data Type | Entropy Range |
|-----------|--------------|
| English text | 4.0-4.5 |
| Source code | 4.5-5.0 |
| Compressed data | 7.8-8.0 |
| Encrypted data | 7.9-8.0 |
| Pointer arrays | 3.0-5.0 |
| Zero-filled | 0.0 |

### 3c. Zero-Fill Detection

```rust
fn zero_fill_detection(data: &[u8], score: &mut Score) {
    if data.len() < 16 { return; }
    let zero_ratio = data.iter().filter(|&&b| b == 0).count() as f32 / data.len() as f32;
    if zero_ratio > 0.9 {
        score.buffer += 15;
        score.cstruct += 10;  // Zeroed struct padding
    }
}
```

---

## Dimension 4: Pointer Heuristic

**File:** `engine.rs:443-457`

Counts valid pointers in the memory region to distinguish buffers from C structs:

```rust
fn pointer_heuristic(view: &MemoryView, score: &mut Score) {
    let ptr_count = count_valid_pointers(view);

    if ptr_count == 0 && view.len() > 8 {
        // No valid pointers → likely buffer
        score.buffer += 40;
    } else if ptr_count == 1 {
        // Single pointer → could be Box or simple struct
        score.pointer += 10;
        score.cstruct += 5;
    } else if ptr_count >= 2 {
        // Multiple pointers → likely C struct
        score.cstruct += 30;
    }
}
```

### Pointer Validation

**File:** `memory_view.rs:58-60`

```rust
pub fn is_valid_ptr(p: usize) -> bool {
    p > MIN_VALID_ADDR && p < MAX_USER_ADDR
}
// MIN_VALID_ADDR = 0x1000 (first page, reserved by OS)
// MAX_USER_ADDR = 0x7fff_ffff_ffff (128TB on 64-bit)
```

Counts valid pointers by scanning memory in 8-byte chunks:

```rust
// memory_view.rs:62-76
pub fn count_valid_pointers(view: &MemoryView) -> usize {
    let mut count = 0;
    for chunk in view.chunks(8) {
        if chunk.len() < 8 { break; }
        let mut buf = [0u8; 8];
        buf.copy_from_slice(chunk);
        let v = usize::from_le_bytes(buf);
        if is_valid_ptr(v) { count += 1; }
    }
    count
}
```

---

## Aggregation: Finalize

**File:** `engine.rs:460-479`

```rust
fn finalize(score: Score) -> TypeGuess {
    let table = [
        (TypeKind::Vec, score.vec),
        (TypeKind::String, score.string),
        (TypeKind::CString, score.cstring),
        (TypeKind::Pointer, score.pointer),
        (TypeKind::FatPtr, score.fat_ptr),
        (TypeKind::Buffer, score.buffer),
        (TypeKind::CStruct, score.cstruct),
    ];

    let mut best = (TypeKind::Unknown, 0u8);
    for (kind, val) in table {
        if val > best.1 { best = (kind, val); }
    }

    TypeGuess::new(best.0, best.1)
}
```

Picks the type with the highest aggregate score. Confidence is the raw score value (0-100+).

---

## Public API

```rust
pub struct UnsafeInferenceEngine;

impl UnsafeInferenceEngine {
    /// Infer type from a single memory view
    pub fn infer_single(view: &MemoryView, size: usize) -> TypeGuess;

    /// Infer type from raw bytes
    pub fn infer_from_bytes(data: &[u8], size: usize) -> TypeGuess;

    /// Run inference on multiple records
    pub fn run(records: &mut [InferenceRecord]);
}
```

### Usage

```rust
use memscope_rs::analysis::unsafe_inference::{
    UnsafeInferenceEngine, TypeKind,
};

let memory = /* raw bytes from FFI allocation */;
let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, size);
println!("Likely type: {} ({}% confidence)", guess.kind, guess.confidence);
```

---

## Type Guess Structure

```rust
pub struct TypeGuess {
    pub kind: TypeKind,       // Inferred type category
    pub confidence: u8,       // Confidence score (0-100+)
    pub method: InferenceMethod,  // How the inference was made
}

pub enum TypeKind {
    Vec,       // Rust Vec<T>
    String,    // Rust String
    CString,   // C-style null-terminated string
    Pointer,   // Raw pointer (*mut T, *const T, Box<T>)
    FatPtr,    // Fat pointer (&[T], &str, dyn Trait)
    Buffer,    // Raw byte buffer ([u8])
    CStruct,   // C struct with multiple pointer fields
    Unknown,   // Could not determine
}

pub enum InferenceMethod {
    Unknown,
    SizeHeuristic,
    LayoutDetection,
    Utf8Validation,
    ContentAnalysis,
    EntropyAnalysis,
    Combined,  // Multiple dimensions contributed
}
```

---

## Performance

| Metric | Value |
|--------|-------|
| Complexity | O(n) per allocation where n = memory size |
| Typical runtime | ~5-50ms for 1M allocations |
| Runs during | Snapshot analysis only (not tracking hot path) |
| Memory cost | Minimal — operates on existing memory snapshots |

---

## Accuracy Estimates

| Type | Expected Accuracy | Key Signals |
|------|------------------|-------------|
| Vec | ~70-85% | Size=24, power-of-two cap, large spare |
| String | ~60-80% | Size=24, small spare, (UTF-8 planned) |
| CString | ~65-80% | Null terminator, high printable ASCII |
| Pointer | ~60-75% | Size=8, single valid pointer |
| Buffer | ~60-75% | Zero pointers, high entropy |
| CStruct | ~50-65% | Multiple pointers, common sizes |

---

## Known Limitations

1. **No UTF-8 validation yet** — The design specifies UTF-8 validation for String detection, but it's not implemented. String vs Vec distinction relies solely on spare capacity.
2. **Wide pointer validation range** — `is_valid_ptr` accepts any address from 0x1000 to 0x7fff_ffff_ffff (128TB), leading to false positives.
3. **No stack trace integration** — The design specifies using call stack symbols for type hints, but this is not implemented.
4. **No lifetime analysis** — The design specifies using allocation lifetime patterns, but this is not implemented.
5. **Synthetic data bias** — All 19 tests use artificially constructed byte arrays. Real-world accuracy may be lower.
