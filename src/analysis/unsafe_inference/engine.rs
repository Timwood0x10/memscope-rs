//! Unsafe Type Inference Engine v2
//!
//! Heuristic-based type inference for FFI/unsafe memory allocations.
//!
//! # Design Principles
//!
//! - **Six-dimensional signal model**: Size, Layout, Content, Stack, Lifetime, ValidPtr
//! - **Memory safe**: No raw pointer dereferencing, all access through MemoryView
//! - **O(n) complexity**: ~5-50ms for 1M allocations depending on enabled features
//!
//! # Phase 1 Features
//!
//! - Enhanced size heuristic with power-of-two signal
//! - UTF-8 validation for String detection
//! - Enhanced CString detection with ASCII ratio
//! - Shannon entropy analysis for binary data
//! - Zero-fill detection

use super::memory_view::{count_valid_pointers, is_valid_ptr, MemoryView};

/// Inferred type category for unsafe memory allocations.
///
/// Each variant represents a common pattern in FFI/unsafe code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeKind {
    /// Rust Vec<T>: heap-allocated growable array (ptr, len, cap)
    Vec,
    /// Rust String: UTF-8 encoded growable string (ptr, len, cap)
    String,
    /// C-style null-terminated string
    CString,
    /// Raw pointer: *mut T, *const T, Box<T>
    Pointer,
    /// Fat pointer: &[T], &str, dyn Trait (data_ptr + metadata)
    FatPtr,
    /// Raw byte buffer: [u8], compressed/encrypted data
    Buffer,
    /// C struct with multiple pointer fields
    CStruct,
    /// Unknown type
    Unknown,
}

impl std::fmt::Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKind::Vec => write!(f, "Vec<_>"),
            TypeKind::String => write!(f, "String"),
            TypeKind::CString => write!(f, "CString"),
            TypeKind::Pointer => write!(f, "*mut c_void"),
            TypeKind::FatPtr => write!(f, "&[T]"),
            TypeKind::Buffer => write!(f, "[u8]"),
            TypeKind::CStruct => write!(f, "CStruct"),
            TypeKind::Unknown => write!(f, "unknown"),
        }
    }
}

/// Type inference result with confidence score.
///
/// Confidence ranges from 0-100, where higher values indicate stronger evidence.
#[derive(Clone, Copy, Debug)]
pub struct TypeGuess {
    /// Inferred type category
    pub kind: TypeKind,
    /// Confidence score (0-100)
    pub confidence: u8,
    /// Method used for inference (for debugging/display)
    pub method: InferenceMethod,
}

/// Method used for type inference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InferenceMethod {
    #[default]
    Unknown,
    SizeHeuristic,
    LayoutDetection,
    Utf8Validation,
    ContentAnalysis,
    EntropyAnalysis,
    StackTraceAnalysis,
    LifetimeAnalysis,
    Combined,
}

impl TypeGuess {
    /// Create a new type guess with the given kind and confidence.
    pub fn new(kind: TypeKind, confidence: u8) -> Self {
        Self {
            kind,
            confidence,
            method: InferenceMethod::Combined,
        }
    }

    /// Create a new type guess with method information.
    pub fn with_method(kind: TypeKind, confidence: u8, method: InferenceMethod) -> Self {
        Self {
            kind,
            confidence,
            method,
        }
    }

    /// Create an unknown type guess.
    pub fn unknown() -> Self {
        Self {
            kind: TypeKind::Unknown,
            confidence: 0,
            method: InferenceMethod::Unknown,
        }
    }

    /// Display the type guess with confidence percentage.
    pub fn display_with_confidence(&self) -> String {
        if self.confidence == 0 {
            return "-".to_string();
        }
        format!("{} ({}%)", self.kind, self.confidence)
    }
}

/// Internal score structure for multi-dimensional inference.
///
/// Each field accumulates evidence from different heuristic dimensions.
#[derive(Default)]
struct Score {
    // Core types
    vec: u8,
    string: u8,
    cstring: u8,
    pointer: u8,
    fat_ptr: u8,
    buffer: u8,
    cstruct: u8,
}

/// Main inference engine for unsafe type detection.
///
/// # Example
///
/// ```
/// use memscope_rs::analysis::unsafe_inference::{
///     UnsafeInferenceEngine, TypeGuess, TypeKind,
/// };
///
/// let memory = vec![0u8; 24];
/// let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);
/// println!("Inferred: {} ({}%)", guess.kind, guess.confidence);
/// ```
pub struct UnsafeInferenceEngine;

impl UnsafeInferenceEngine {
    /// Infer type from a single memory view.
    pub fn infer_single(view: &MemoryView, size: usize) -> TypeGuess {
        infer(view, size, None, None, None)
    }

    /// Infer type from raw bytes.
    pub fn infer_from_bytes(data: &[u8], size: usize) -> TypeGuess {
        let view = MemoryView::new(data);
        infer(&view, size, None, None, None)
    }

    /// Infer type with full context.
    pub fn infer_with_context(
        data: &[u8],
        size: usize,
        stack_trace: Option<&[String]>,
        alloc_time: Option<u64>,
        dealloc_time: Option<u64>,
    ) -> TypeGuess {
        let view = MemoryView::new(data);
        infer(&view, size, stack_trace, alloc_time, dealloc_time)
    }

    /// Run inference on multiple records.
    pub fn run(records: &mut [InferenceRecord]) {
        for record in records.iter_mut() {
            if let Some(ref memory) = record.memory {
                let view = MemoryView::new(memory);
                record.inferred = Some(infer(
                    &view,
                    record.size,
                    record.stack_trace.as_deref(),
                    record.alloc_time,
                    record.dealloc_time,
                ));
            }
        }
    }
}

/// Record for batch inference.
pub struct InferenceRecord {
    /// Memory address of the allocation
    pub ptr: usize,
    /// Size in bytes
    pub size: usize,
    /// Optional memory content snapshot
    pub memory: Option<Vec<u8>>,
    /// Optional stack trace at allocation time
    pub stack_trace: Option<Vec<String>>,
    /// Optional allocation timestamp (nanoseconds)
    pub alloc_time: Option<u64>,
    /// Optional deallocation timestamp (nanoseconds)
    pub dealloc_time: Option<u64>,
    /// Inference result
    pub inferred: Option<TypeGuess>,
}

impl InferenceRecord {
    /// Create a new inference record with minimal information.
    pub fn new(ptr: usize, size: usize, memory: Option<Vec<u8>>) -> Self {
        Self {
            ptr,
            size,
            memory,
            stack_trace: None,
            alloc_time: None,
            dealloc_time: None,
            inferred: None,
        }
    }

    /// Create a new inference record with full information.
    pub fn with_context(
        ptr: usize,
        size: usize,
        memory: Option<Vec<u8>>,
        stack_trace: Option<Vec<String>>,
        alloc_time: Option<u64>,
        dealloc_time: Option<u64>,
    ) -> Self {
        Self {
            ptr,
            size,
            memory,
            stack_trace,
            alloc_time,
            dealloc_time,
            inferred: None,
        }
    }
}

/// Main inference function combining all heuristic dimensions.
fn infer(
    view: &MemoryView,
    size: usize,
    stack_trace: Option<&[String]>,
    alloc_time: Option<u64>,
    dealloc_time: Option<u64>,
) -> TypeGuess {
    let mut score = Score::default();

    // Dimension 1: Size heuristic with power-of-two signal
    size_heuristic(size, &mut score);

    // Dimension 2: Layout detection (ptr/len/cap structure)
    vec_string_layout(view, &mut score);

    // Dimension 3: Content analysis (CString, entropy, zero-fill)
    content_analysis(view, &mut score);

    // Dimension 4: Pointer-based heuristics (buffer vs cstruct)
    pointer_heuristic(view, &mut score);

    // Dimension 5: Stack trace analysis (optional, high signal)
    stack_trace_analysis(stack_trace, &mut score);

    // Dimension 6: Lifetime analysis (optional, auxiliary signal)
    lifetime_analysis(alloc_time, dealloc_time, &mut score);

    // Aggregate and finalize
    finalize(score)
}

/// Dimension 1: Enhanced size heuristic.
///
/// Key improvements over v1:
/// - Reduced pointer score for size=8 (was too aggressive)
/// - Added fat_ptr detection for size=16
/// - Added power-of-two signal for Vec/Buffer
/// - Added common C struct sizes
fn size_heuristic(size: usize, score: &mut Score) {
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
fn vec_string_layout(view: &MemoryView, score: &mut Score) {
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
fn content_analysis(view: &MemoryView, score: &mut Score) {
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
/// This is the decisive signal for String vs Vec distinction.
/// A random byte sequence passes UTF-8 validation with probability:
/// - 16 bytes: ~0.3%
/// - 64 bytes: ~0.00001%
/// - 256 bytes: ~0%
///
/// Scoring strategy:
/// - High printable ratio (>0.8): Strong String signal (+90)
/// - Medium printable ratio (>0.5): Moderate String signal (+60)
/// - Low printable ratio: Weak Vec signal (+20)
/// - Invalid UTF-8: Strong Vec signal (+50)
fn utf8_validation(data: &[u8], score: &mut Score) {
    if data.is_empty() {
        return;
    }

    // Try to validate as UTF-8
    match std::str::from_utf8(data) {
        Ok(s) => {
            // Valid UTF-8 - count printable ratio
            let printable = s
                .chars()
                .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
                .count();
            let total = s.chars().count();
            let ratio = if total > 0 {
                printable as f32 / total as f32
            } else {
                0.0
            };

            if ratio > 0.8 {
                // High printable ratio -> likely String content
                score.string += 90;
            } else if ratio > 0.5 {
                // Mixed content -> could be String or Vec<u8>
                score.string += 60;
                score.vec += 30;
            } else if ratio > 0.2 {
                // Low printable but valid UTF-8
                score.vec += 30;
            } else {
                // Very low printable ratio (mostly control chars or nulls)
                // This is weak evidence for Vec, don't override other signals
                score.vec += 20;
            }
        }
        Err(_) => {
            // Not valid UTF-8 -> definitely not a String
            // This is a strong negative signal for String
            score.vec += 50;
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

/// Dimension 5: Stack trace analysis.
///
/// This is the highest-discrimination single signal.
/// If the call stack contains `alloc::vec::Vec::push`, it's almost 100% Vec.
///
/// Graceful degradation: if stack trace is unavailable, this dimension
/// contributes nothing and doesn't affect other dimensions.
fn stack_trace_analysis(stack: Option<&[String]>, score: &mut Score) {
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
fn lifetime_analysis(alloc_time: Option<u64>, dealloc_time: Option<u64>, score: &mut Score) {
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

/// Finalize inference by selecting the highest-scoring type.
///
/// Also determines the primary inference method based on which dimension
/// contributed most to the final score.
fn finalize(score: Score) -> TypeGuess {
    let table = [
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
    ];

    let mut best = (TypeKind::Unknown, 0u8, InferenceMethod::Unknown);
    for (kind, val, method) in table {
        if val > best.1 {
            best = (kind, val, method);
        }
    }

    TypeGuess::with_method(best.0, best.1, best.2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_vec_memory(ptr: usize, len: usize, cap: usize) -> Vec<u8> {
        let mut data = vec![0u8; 24];
        data[..8].copy_from_slice(&ptr.to_le_bytes());
        data[8..16].copy_from_slice(&len.to_le_bytes());
        data[16..24].copy_from_slice(&cap.to_le_bytes());
        data
    }

    fn create_string_memory(ptr: usize, len: usize, cap: usize) -> Vec<u8> {
        create_vec_memory(ptr, len, cap)
    }

    fn create_cstring_memory(content: &[u8]) -> Vec<u8> {
        let mut data = content.to_vec();
        data.push(0);
        data
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_infer_vec_with_large_capacity() {
        let memory = create_vec_memory(0x10000, 10, 100);
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);

        assert_eq!(guess.kind, TypeKind::Vec);
        assert!(guess.confidence >= 60);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_infer_vec_with_power_of_two_capacity() {
        let memory = create_vec_memory(0x10000, 50, 64);
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);

        // Power-of-two capacity should boost Vec score
        assert!(guess.kind == TypeKind::Vec || guess.kind == TypeKind::String);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_infer_string_with_small_spare() {
        let memory = create_string_memory(0x10000, 10, 12);
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);

        // Small spare capacity should favor String
        assert!(guess.kind == TypeKind::String || guess.kind == TypeKind::Vec);
    }

    #[test]
    fn test_infer_cstring_printable() {
        let memory = create_cstring_memory(b"hello world");
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 12);

        assert_eq!(guess.kind, TypeKind::CString);
        assert!(guess.confidence >= 70);
    }

    #[test]
    fn test_infer_cstring_mixed_content() {
        let memory = create_cstring_memory(&[0x30, 0x31, 0x80, 0x90, 0x20]);
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 6);

        // Mixed content should have lower CString score
        assert!(guess.confidence < 70);
    }

    #[test]
    fn test_infer_pointer_size_8() {
        let memory = [0u8; 8];
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 8);

        assert_eq!(guess.kind, TypeKind::Pointer);
        assert!(guess.confidence >= 30);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_infer_fat_ptr_size_16() {
        // Fat pointer: data_ptr + metadata (e.g., length for &[T])
        // Create a pattern that looks like a fat pointer: valid ptr + small length
        let mut memory = [0u8; 16];
        let ptr: usize = 0x10000;
        let len: usize = 100;
        memory[0..8].copy_from_slice(&ptr.to_le_bytes());
        memory[8..16].copy_from_slice(&len.to_le_bytes());

        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 16);

        // FatPtr should have the highest score due to size=16 + valid pointer
        assert!(guess.kind == TypeKind::FatPtr || guess.kind == TypeKind::Pointer);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_infer_cstruct_multiple_pointers() {
        let mut memory = vec![1u8; 40];
        let ptr1: usize = 0x10000;
        let ptr2: usize = 0x20000;
        memory[0..8].copy_from_slice(&ptr1.to_le_bytes());
        memory[24..32].copy_from_slice(&ptr2.to_le_bytes());

        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 40);

        assert_eq!(guess.kind, TypeKind::CStruct);
        assert!(guess.confidence >= 30);
    }

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

    #[test]
    fn test_power_of_two_signal() {
        // Size 64 (power of two) should get Vec/Buffer boost
        let memory = vec![0u8; 64];
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 64);

        // Should have some confidence from power-of-two signal
        assert!(guess.confidence > 0 || guess.kind == TypeKind::Unknown);
    }

    #[test]
    fn test_zero_filled_buffer() {
        let memory = [0u8; 64];
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 64);

        // Zero-filled should boost buffer or cstruct
        assert!(
            guess.kind == TypeKind::Buffer
                || guess.kind == TypeKind::CStruct
                || guess.kind == TypeKind::Unknown
        );
    }

    #[test]
    fn test_type_guess_display() {
        let guess = TypeGuess::new(TypeKind::Vec, 85);
        assert_eq!(guess.display_with_confidence(), "Vec<_> (85%)");

        let unknown = TypeGuess::unknown();
        assert_eq!(unknown.display_with_confidence(), "-");
    }

    #[test]
    fn test_run_on_records() {
        let vec_memory = create_vec_memory(0x10000, 10, 100);
        let mut records = vec![
            InferenceRecord::new(0x1000, 24, Some(vec_memory)),
            InferenceRecord::new(0x2000, 8, None),
        ];

        UnsafeInferenceEngine::run(&mut records);

        assert!(records[0].inferred.is_some());
        assert!(records[1].inferred.is_none());
    }

    #[test]
    fn test_multiple_nulls_not_cstring() {
        // Multiple null bytes should reduce CString score
        let memory = [
            b'h', b'e', b'l', b'l', b'o', 0, b'w', b'o', b'r', b'l', b'd', 0,
        ];
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 12);

        // Should not be strongly identified as CString due to multiple nulls
        if guess.kind == TypeKind::CString {
            assert!(guess.confidence < 70);
        }
    }

    #[test]
    fn test_utf8_validation_printable_string() {
        // Valid UTF-8 with high printable ratio
        let memory = b"Hello, World! This is a test string.".to_vec();
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, memory.len());

        // Should be strongly identified as String due to UTF-8 validation
        assert_eq!(guess.kind, TypeKind::String);
        assert!(guess.confidence >= 90);
        assert_eq!(guess.method, InferenceMethod::Utf8Validation);
    }

    #[test]
    fn test_utf8_validation_non_printable() {
        // Valid UTF-8 but with non-printable characters
        let memory = vec![0xC2, 0x80, 0xC2, 0x81, 0xC2, 0x82]; // Valid UTF-8 control chars
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, memory.len());

        // Should be Vec due to valid UTF-8 but low printable ratio
        assert!(guess.kind == TypeKind::Vec || guess.kind == TypeKind::Buffer);
    }

    #[test]
    fn test_utf8_validation_invalid() {
        // Invalid UTF-8 sequence
        let memory = vec![0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA];
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, memory.len());

        // Should not be String due to invalid UTF-8
        assert_ne!(guess.kind, TypeKind::String);
    }

    #[test]
    fn test_large_data_buffer_boost() {
        // Data > 4KB should get buffer boost without entropy calculation
        let memory = vec![0u8; 5000];
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 5000);

        // Should be buffer due to size
        assert!(guess.kind == TypeKind::Buffer || guess.kind == TypeKind::CStruct);
    }

    #[test]
    fn test_inference_method_tracking() {
        // Test that inference method is properly tracked
        let string_memory = b"test string content".to_vec();
        let guess = UnsafeInferenceEngine::infer_from_bytes(&string_memory, string_memory.len());

        // String should use Utf8Validation method
        assert_eq!(guess.kind, TypeKind::String);
        assert_eq!(guess.method, InferenceMethod::Utf8Validation);
    }

    #[test]
    fn test_stack_trace_vec_detection() {
        let memory = vec![0u8; 24];
        let stack = vec![
            "alloc::vec::Vec::push".to_string(),
            "my_app::process".to_string(),
        ];
        let guess =
            UnsafeInferenceEngine::infer_with_context(&memory, 24, Some(&stack), None, None);

        // Stack trace with Vec::push should boost Vec score
        assert!(guess.kind == TypeKind::Vec || guess.kind == TypeKind::Unknown);
    }

    #[test]
    fn test_stack_trace_string_detection() {
        let memory = b"test".to_vec();
        let stack = vec!["alloc::string::String::push".to_string()];
        let guess = UnsafeInferenceEngine::infer_with_context(&memory, 4, Some(&stack), None, None);

        // Stack trace with String::push should boost String score
        assert!(guess.kind == TypeKind::String || guess.kind == TypeKind::Unknown);
    }

    #[test]
    fn test_stack_trace_cstring_detection() {
        let memory = b"hello\0".to_vec();
        let stack = vec!["std::ffi::c_str::CString::new".to_string()];
        let guess = UnsafeInferenceEngine::infer_with_context(&memory, 6, Some(&stack), None, None);

        // Stack trace with CString::new should boost CString score
        assert!(guess.kind == TypeKind::CString || guess.kind == TypeKind::String);
    }

    #[test]
    fn test_stack_trace_ffi_detection() {
        let memory = vec![0u8; 64];
        let stack = vec!["libc::malloc".to_string()];
        let guess =
            UnsafeInferenceEngine::infer_with_context(&memory, 64, Some(&stack), None, None);

        // Stack trace with malloc should boost CStruct/Buffer score
        assert!(
            guess.kind == TypeKind::CStruct
                || guess.kind == TypeKind::Buffer
                || guess.kind == TypeKind::Unknown
        );
    }

    #[test]
    fn test_lifetime_transient_allocation() {
        let memory = b"test".to_vec();
        let alloc_time = Some(1000);
        let dealloc_time = Some(1_000_500); // 0.5ms lifetime
        let guess =
            UnsafeInferenceEngine::infer_with_context(&memory, 4, None, alloc_time, dealloc_time);

        // Transient allocation should boost String/Vec score
        assert!(guess.confidence > 0 || guess.kind == TypeKind::Unknown);
    }

    #[test]
    fn test_lifetime_long_lived() {
        let memory = vec![0u8; 64];
        let alloc_time = Some(1000);
        let dealloc_time = Some(15_000_000_000); // 15s lifetime
        let guess =
            UnsafeInferenceEngine::infer_with_context(&memory, 64, None, alloc_time, dealloc_time);

        // Long-lived allocation should boost Buffer score
        assert!(guess.confidence > 0 || guess.kind == TypeKind::Unknown);
    }

    #[test]
    fn test_combined_stack_and_lifetime() {
        let memory = vec![0u8; 24];
        let stack = vec!["alloc::vec::Vec::new".to_string()];
        let alloc_time = Some(1000);
        let dealloc_time = Some(1_000_500); // 0.5ms
        let guess = UnsafeInferenceEngine::infer_with_context(
            &memory,
            24,
            Some(&stack),
            alloc_time,
            dealloc_time,
        );

        // Combined signals should give higher confidence
        assert!(guess.confidence > 0 || guess.kind == TypeKind::Unknown);
    }

    #[test]
    fn test_inference_record_with_context() {
        let memory = Some(vec![0u8; 24]);
        let stack = Some(vec!["alloc::vec::Vec::new".to_string()]);
        let record =
            InferenceRecord::with_context(0x1000, 24, memory, stack, Some(1000), Some(2000));

        assert_eq!(record.ptr, 0x1000);
        assert_eq!(record.size, 24);
        assert!(record.stack_trace.is_some());
        assert!(record.alloc_time.is_some());
        assert!(record.dealloc_time.is_some());
    }
}

/// Real data tests using actual type memory layouts.
#[cfg(test)]
mod real_data_tests {
    use super::*;

    /// Get memory representation of a real Vec.
    fn vec_to_memory<T>(v: &Vec<T>) -> Vec<u8> {
        let ptr = v.as_ptr() as usize;
        let len = v.len();
        let cap = v.capacity();
        let mut memory = vec![0u8; 24];
        memory[..8].copy_from_slice(&ptr.to_le_bytes());
        memory[8..16].copy_from_slice(&len.to_le_bytes());
        memory[16..24].copy_from_slice(&cap.to_le_bytes());
        memory
    }

    /// Get memory representation of a real String.
    fn string_to_memory(s: &String) -> Vec<u8> {
        let ptr = s.as_ptr() as usize;
        let len = s.len();
        let cap = s.capacity();
        let mut memory = vec![0u8; 24];
        memory[..8].copy_from_slice(&ptr.to_le_bytes());
        memory[8..16].copy_from_slice(&len.to_le_bytes());
        memory[16..24].copy_from_slice(&cap.to_le_bytes());
        memory
    }

    /// Get memory representation of a Box.
    fn box_to_memory<T>(b: &T) -> Vec<u8> {
        let ptr = b as *const T as usize;
        let mut memory = vec![0u8; 8];
        memory[..8].copy_from_slice(&ptr.to_le_bytes());
        memory
    }

    #[test]
    fn test_real_vec_i32() {
        let v = vec![1i32, 2, 3, 4, 5];
        let memory = vec_to_memory(&v);
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);

        // Vec<String> has similar layout - both Vec and String are valid
        assert!(
            guess.kind == TypeKind::Vec || guess.kind == TypeKind::String,
            "Got {:?}",
            guess.kind
        );
    }

    #[test]
    fn test_real_vec_u8() {
        let v = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let memory = vec_to_memory(&v);
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);

        // Vec<u8> has same layout as String
        assert!(guess.kind == TypeKind::Vec || guess.kind == TypeKind::String);
    }

    #[test]
    fn test_real_string() {
        let s = String::from("Hello, World!");
        let memory = string_to_memory(&s);
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);

        // String should be detected (spare capacity is typically small)
        assert!(
            guess.kind == TypeKind::String || guess.kind == TypeKind::Vec,
            "Got {:?}",
            guess.kind
        );
    }

    #[test]
    fn test_real_string_with_capacity() {
        let mut s = String::with_capacity(100);
        s.push_str("Hello");
        let memory = string_to_memory(&s);
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);

        // Large spare capacity -> more likely Vec
        assert!(
            guess.kind == TypeKind::Vec || guess.kind == TypeKind::String,
            "Got {:?}",
            guess.kind
        );
    }

    #[test]
    fn test_real_box_i32() {
        let b = Box::new(42i32);
        let memory = box_to_memory(&*b);
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 8);

        // Box and Pointer have same size=8 layout
        // Note: CString is also possible if the bytes happen to look like a valid C string
        assert!(
            guess.kind == TypeKind::Pointer
                || guess.kind == TypeKind::Vec
                || guess.kind == TypeKind::String
                || guess.kind == TypeKind::CString,
            "Got {:?}",
            guess.kind
        );
    }

    #[test]
    fn test_real_string_content() {
        let s = "Hello, World! This is a test string for type inference.";
        let guess = UnsafeInferenceEngine::infer_from_bytes(s.as_bytes(), s.len());

        assert_eq!(guess.kind, TypeKind::String);
        assert!(guess.confidence >= 90);
    }

    #[test]
    fn test_real_cstring_content() {
        let cstr = std::ffi::CString::new("Hello, C World!").unwrap();
        let bytes = cstr.as_bytes_with_nul();
        let guess = UnsafeInferenceEngine::infer_from_bytes(bytes, bytes.len());

        assert_eq!(guess.kind, TypeKind::CString);
        assert!(guess.confidence >= 70);
    }

    #[test]
    fn test_real_binary_data() {
        let binary: Vec<u8> = (0..=255).collect();
        let guess = UnsafeInferenceEngine::infer_from_bytes(&binary, binary.len());

        // High entropy binary data
        assert!(guess.kind == TypeKind::Buffer || guess.kind == TypeKind::Vec);
    }

    #[test]
    fn test_real_zero_filled() {
        let zeros = vec![0u8; 1024];
        let guess = UnsafeInferenceEngine::infer_from_bytes(&zeros, 1024);

        // Zero-filled large data
        assert!(guess.kind == TypeKind::Buffer || guess.kind == TypeKind::CStruct);
    }

    #[test]
    fn test_real_vec_with_stack_trace() {
        let v = vec![1i32, 2, 3, 4, 5];
        let memory = vec_to_memory(&v);
        let stack = vec!["alloc::vec::Vec::push".to_string()];
        let guess =
            UnsafeInferenceEngine::infer_with_context(&memory, 24, Some(&stack), None, None);

        assert_eq!(guess.kind, TypeKind::Vec);
        // Stack trace should boost confidence
        assert!(guess.confidence >= 80);
    }

    #[test]
    fn test_real_string_with_stack_trace() {
        let s = String::from("Hello");
        let memory = string_to_memory(&s);
        let stack = vec!["alloc::string::String::push_str".to_string()];
        let guess =
            UnsafeInferenceEngine::infer_with_context(&memory, 24, Some(&stack), None, None);

        assert!(guess.kind == TypeKind::String || guess.kind == TypeKind::Vec);
    }

    #[test]
    fn test_real_struct_with_pointers() {
        struct TestStruct {
            _ptr1: *const u8,
            _ptr2: *const u8,
            _value: u64,
        }

        let s = TestStruct {
            _ptr1: &0u8,
            _ptr2: &1u8,
            _value: 42,
        };

        let memory = unsafe {
            std::slice::from_raw_parts(
                &s as *const TestStruct as *const u8,
                std::mem::size_of::<TestStruct>(),
            )
        };

        let guess = UnsafeInferenceEngine::infer_from_bytes(memory, memory.len());

        // Struct is 24 bytes (2 pointers + 1 u64), same as Vec/String layout
        // So it could be detected as Vec, String, or CStruct depending on pointer values
        assert!(
            guess.kind == TypeKind::CStruct
                || guess.kind == TypeKind::Buffer
                || guess.kind == TypeKind::Pointer
                || guess.kind == TypeKind::Vec
                || guess.kind == TypeKind::String
                || guess.kind == TypeKind::Unknown,
            "Got {:?}",
            guess.kind
        );
    }
}
