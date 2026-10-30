//! Heuristic scoring dimensions for unsafe type inference.
//!
//! Each dimension adds evidence to a [`Score`] accumulator; none of them decides
//! anything. Acceptance is decided by
//! [`super::policy::InferencePolicy`] in `engine::finalize`, so a signal can never
//! leak into the output just because it happened to be the only non-zero entry.
//!
//! Split out of `engine.rs` to keep both files within the project line budget.

use super::engine::{InferenceMethod, TypeKind};
use super::memory_view::{count_valid_pointers, is_valid_ptr, MemoryView};

/// Internal score structure for multi-dimensional inference.
///
/// Each field accumulates evidence from different heuristic dimensions.
#[derive(Default)]
pub(super) struct Score {
    // Core types. `u16` keeps each signal's contribution visible instead of
    // saturating at 255 as the previous `u8` accumulators did. Fields are visible
    // to `engine::finalize`, which is the only place that ranks them.
    pub(super) vec: u16,
    pub(super) string: u16,
    pub(super) cstring: u16,
    pub(super) pointer: u16,
    pub(super) fat_ptr: u16,
    pub(super) buffer: u16,
    pub(super) cstruct: u16,
}

/// Dimension 1: Enhanced size heuristic.
///
/// Key improvements over v1:
/// - Reduced pointer score for size=8 (was too aggressive)
/// - Added fat_ptr detection for size=16
/// - Added power-of-two signal for Vec/Buffer
/// - Added common C struct sizes
pub(super) fn size_heuristic(size: usize, score: &mut Score) {
    match size {
        // Raw pointer: *mut T, *const T, &T, Box<T>
        // Reduced from 60 to 30 to reduce false positives
        8 => score.pointer += 30,

        // Fat pointer: &[T], &str, dyn Trait (data_ptr + metadata)
        // This is a strong signal for slice references
        16 => score.fat_ptr += 25,

        // Vec/String triplet: (ptr, len, cap)
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

/// Dimension 2: Vec/String layout detection.
///
/// Detects the (ptr, len, cap) triplet structure.
/// Key improvement: UTF-8 validation for String detection.
pub(super) fn vec_string_layout(view: &MemoryView, score: &mut Score) {
    let usize_size = std::mem::size_of::<usize>();
    let min_len = usize_size * 3; // ptr + len + cap
    if view.len() < min_len {
        return;
    }

    let ptr_val = view.read_usize(0);
    let len = view.read_usize(usize_size);
    let cap = view.read_usize(usize_size * 2);

    let (Some(p), Some(l), Some(c)) = (ptr_val, len, cap) else {
        return;
    };

    // Basic structure validation
    if !is_valid_ptr(p) || c < l || c == 0 || c > 10_000_000 {
        return;
    }

    // Distinguish Vec from String based on capacity vs length
    // String typically has cap close to len (small spare capacity)
    // Vec often has cap >> len (pre-allocated growth space)
    let spare = c.saturating_sub(l);

    if spare < 16 && l > 0 {
        // Small spare capacity → more likely String
        score.string += 50;
        score.vec += 20;
    } else if spare > 0 {
        // Large spare capacity → more likely Vec
        score.vec += 60;
        score.string += 15;
    } else {
        // cap == len → could be either
        score.vec += 30;
        score.string += 30;
    }

    // Additional Vec signal: capacity is power of two
    // Rust's Vec growth strategy: new_cap = max(old_cap * 2, 1)
    if c.is_power_of_two() {
        score.vec += 15;
    }
}

/// Dimension 3: Content analysis.
///
/// Enhanced detection for CString, entropy, and zero-fill patterns.
pub(super) fn content_analysis(view: &MemoryView, score: &mut Score) {
    let data = view.as_slice();
    if data.is_empty() {
        return;
    }

    // UTF-8 validation for String detection (decisive signal)
    utf8_validation(data, score);

    // Enhanced CString detection
    cstring_enhanced(data, score);

    // Entropy analysis for binary data detection
    // Skip for large data (>4KB) to avoid performance cost
    if data.len() >= 32 && data.len() <= 4096 {
        entropy_analysis(data, score);
    } else if data.len() > 4096 {
        // Large data is likely buffer, skip expensive analysis
        score.buffer += 40;
    }

    // Zero-fill detection
    zero_fill_detection(data, score);
}

/// UTF-8 validation for String detection.
///
/// This is the decisive signal for String vs Vec distinction. Valid UTF-8 is
/// strong evidence for `String`; invalid UTF-8 is strong evidence against it.
///
/// Scoring strategy:
/// - Mostly non-control characters (>0.8): strong String signal (+90)
/// - Mixed non-control characters (>0.5): moderate String signal (+60)
/// - Mostly control characters: no String signal and **no** Vec signal either,
///   because valid UTF-8 is not evidence for a byte buffer
/// - Invalid UTF-8: strong Vec signal (+50)
///
/// The "printable" test is deliberately Unicode-aware: counting only ASCII
/// graphics would classify every CJK character, accented letter and emoji as
/// non-printable and push perfectly valid text towards `Vec`.
fn utf8_validation(data: &[u8], score: &mut Score) {
    if data.is_empty() {
        return;
    }

    match std::str::from_utf8(data) {
        Ok(s) => {
            let total = s.chars().count();
            if total == 0 {
                return;
            }
            let printable = s.chars().filter(|c| !c.is_control()).count();
            let ratio = printable as f32 / total as f32;
            // A NUL byte is the defining feature of a C string, not of printable
            // text, so it must not justify the strongest String signal. Without
            // this, `b"hello\0"` scored CString 95 vs String 90 — a tie the old
            // code resolved by table order.
            let has_nul = data.contains(&0);

            if ratio > 0.8 && !has_nul {
                score.string += 90;
            } else if ratio > 0.5 {
                score.string += 60;
                score.vec += 20;
            } else {
                // Valid UTF-8 that is mostly control characters: not a String, but
                // also not evidence for Vec. Let the other dimensions decide.
                score.buffer += 10;
            }
        }
        Err(_) => {
            // Not valid UTF-8 rules out `String`, but it is not by itself evidence for
            // `Vec`: plenty of byte buffers are valid ASCII, and a pointer/len/cap
            // header carries the real Vec signal. Keep this a weak, negative signal
            // so it cannot drown out a valid pointer layout (e.g. a fat pointer whose
            // metadata bytes are not valid UTF-8).
            score.vec += 20;
        }
    }
}

/// Enhanced CString detection with ASCII ratio analysis.
///
/// Key improvements:
/// - Check printable ASCII ratio, not just trailing null
/// - Detect multiple nulls (likely binary, not CString)
/// - Require minimum content length
/// - CString (null-terminated) should beat String when null terminator is present
fn cstring_enhanced(data: &[u8], score: &mut Score) {
    // Find first null byte
    let null_pos = match data.iter().position(|&b| b == 0) {
        Some(pos) => pos,
        None => return, // No null terminator
    };

    // Empty or too short
    if null_pos < 3 {
        return;
    }

    let content = &data[..null_pos];

    // Count printable ASCII characters (0x20-0x7E)
    let printable_count = content
        .iter()
        .filter(|&&b| (0x20..=0x7E).contains(&b))
        .count();

    let printable_ratio = printable_count as f32 / content.len() as f32;

    // High printable ratio with null terminator → likely CString
    // Score higher than String (+90) because null terminator is definitive for CString
    if printable_ratio > 0.9 {
        score.cstring += 95;
    } else if printable_ratio > 0.7 {
        score.cstring += 60;
    } else if printable_ratio > 0.5 {
        score.cstring += 30;
    }

    // Multiple nulls → likely binary data, not CString
    let null_count = data.iter().filter(|&&b| b == 0).count();
    if null_count > 1 {
        score.cstring = score.cstring.saturating_sub(20);
        score.buffer += 15;
    }
}

/// Shannon entropy analysis for binary data detection.
///
/// Entropy ranges from 0.0 (all same byte) to 8.0 (perfectly random).
///
/// Typical values:
/// - English text: 4.0-4.5
/// - Source code: 4.5-5.0
/// - Compressed data: 7.8-8.0
/// - Encrypted data: 7.9-8.0
fn entropy_analysis(data: &[u8], score: &mut Score) {
    let entropy = shannon_entropy(data);

    // High entropy → compressed/encrypted/serialized data
    if entropy > 7.5 {
        score.buffer += 30;
    } else if entropy > 6.5 {
        score.buffer += 15;
    }
    // Low entropy → repetitive data or text
    else if entropy < 3.0 {
        score.cstruct += 5;
    }
}

/// Calculate Shannon entropy of a byte sequence.
fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut freq = [0u32; 256];
    for &b in data {
        freq[b as usize] += 1;
    }

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

/// Detect zero-filled memory regions.
///
/// High zero ratio often indicates:
/// - Uninitialized struct padding
/// - Zeroed buffers
/// - Sparse data structures
fn zero_fill_detection(data: &[u8], score: &mut Score) {
    if data.len() < 16 {
        return;
    }

    let zero_count = data.iter().filter(|&&b| b == 0).count();
    let zero_ratio = zero_count as f32 / data.len() as f32;

    if zero_ratio > 0.9 {
        // Mostly zeros → likely zeroed buffer or struct with padding
        score.buffer += 15;
        score.cstruct += 10;
    }
}

/// Dimension 4: Pointer-based heuristics.
///
/// Distinguish buffer from C struct based on pointer count.
pub(super) fn pointer_heuristic(view: &MemoryView, score: &mut Score) {
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

/// Dimension 5: Stack trace analysis.
///
/// This is the highest-discrimination single signal.
/// If the call stack contains `alloc::vec::Vec::push`, it's almost 100% Vec.
///
/// Graceful degradation: if stack trace is unavailable, this dimension
/// contributes nothing and doesn't affect other dimensions.
pub(super) fn stack_trace_analysis(stack: Option<&[String]>, score: &mut Score) {
    let Some(frames) = stack else {
        return;
    };

    if frames.is_empty() {
        return;
    }

    for frame in frames {
        let f = frame.to_lowercase();

        // Rust standard library signals
        if f.contains("alloc::vec::vec") || f.contains("vec::from_elem") {
            score.vec += 50;
        }
        if f.contains("alloc::string::string") || f.contains("from_utf8") {
            score.string += 50;
        }
        if f.contains("alloc::boxed") {
            score.pointer += 40;
        }
        if f.contains("ffi::c_str::cstring") || f.contains("from_bytes_with_nul") {
            score.cstring += 60;
        }

        // FFI signals
        if f.contains("malloc") || f.contains("calloc") || f.contains("realloc") {
            score.cstruct += 20;
            score.buffer += 15;
        }
        if f.contains("libc::") || f.contains("std::ffi") {
            score.cstring += 15;
            score.cstruct += 10;
        }
    }
}

/// Dimension 6: Lifetime analysis.
///
/// Uses allocation-deallocation time difference to infer type.
/// This is an auxiliary signal with lower weight.
pub(super) fn lifetime_analysis(
    alloc_time: Option<u64>,
    dealloc_time: Option<u64>,
    score: &mut Score,
) {
    let Some(alloc) = alloc_time else {
        return;
    };

    let Some(dealloc) = dealloc_time else {
        // Not deallocated → possibly leaked or long-lived
        return;
    };

    let lifetime_ns = dealloc.saturating_sub(alloc);
    let lifetime_ms = lifetime_ns / 1_000_000;

    match lifetime_ms {
        // Transient allocation (0ms) → possibly temporary String or small Vec
        0 => {
            score.string += 10;
            score.vec += 5;
        }

        // Short-lived (1-100ms) → possibly function-local variable
        1..=100 => {
            score.cstruct += 5;
        }

        // Long-lived (> 10s) → possibly global cache or leaked
        10000.. => {
            score.buffer += 10;
        }

        _ => {}
    }
}

/// Rank the candidates for a score vector.
///
/// The table order is the deterministic tie-break: the sort is stable, so two
/// candidates with the same score keep their declared order instead of being
/// decided by an enum cast.
pub(super) fn candidate_table(score: &Score) -> [(TypeKind, u16, InferenceMethod); 7] {
    [
        (TypeKind::Vec, score.vec, InferenceMethod::LayoutDetection),
        (
            TypeKind::String,
            score.string,
            InferenceMethod::Utf8Validation,
        ),
        (
            TypeKind::CString,
            score.cstring,
            InferenceMethod::ContentAnalysis,
        ),
        (
            TypeKind::Pointer,
            score.pointer,
            InferenceMethod::SizeHeuristic,
        ),
        (
            TypeKind::FatPtr,
            score.fat_ptr,
            InferenceMethod::SizeHeuristic,
        ),
        (
            TypeKind::Buffer,
            score.buffer,
            InferenceMethod::EntropyAnalysis,
        ),
        (TypeKind::CStruct, score.cstruct, InferenceMethod::Combined),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_calculation() {
        // All same byte → entropy = 0
        let data = [0u8; 100];
        assert!((shannon_entropy(&data) - 0.0).abs() < 0.01);

        // Alternating bytes → lower entropy
        let data: Vec<u8> = (0..100).map(|i| if i % 2 == 0 { 0 } else { 255 }).collect();
        assert!(shannon_entropy(&data) < 2.0);

        // Random-ish data → higher entropy
        let data: Vec<u8> = (0..100).map(|i| i as u8).collect();
        assert!(shannon_entropy(&data) > 5.0);
    }
}
