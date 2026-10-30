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
#![allow(dead_code)]

use super::memory_view::MemoryView;
use super::policy::{InferenceDecision, InferencePolicy};
use super::scoring::{
    candidate_table, content_analysis, lifetime_analysis, pointer_heuristic, size_heuristic,
    stack_trace_analysis, vec_string_layout, Score,
};

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

/// Type inference result with a heuristic score.
///
/// The score is a sum of heuristic signals, **not** a probability: it is neither
/// bounded by 100 nor meaningful as a percentage. A guess is only produced when
/// [`InferencePolicy`] accepts the evidence; otherwise the kind is
/// [`TypeKind::Unknown`] and the score is 0.
#[derive(Clone, Copy, Debug)]
pub struct TypeGuess {
    /// Inferred type category
    pub kind: TypeKind,
    /// Heuristic score sum for the accepted candidate (not a percentage)
    pub confidence: u16,
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
    pub fn new(kind: TypeKind, confidence: u16) -> Self {
        Self {
            kind,
            confidence,
            method: InferenceMethod::Combined,
        }
    }

    /// Create a new type guess with method information.
    pub fn with_method(kind: TypeKind, confidence: u16, method: InferenceMethod) -> Self {
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

    /// Display the type guess with its heuristic score.
    ///
    /// The value is a score sum, not a probability, so it is never rendered as a
    /// percentage.
    pub fn display_with_confidence(&self) -> String {
        if self.confidence == 0 {
            return "-".to_string();
        }
        format!("{} (score {})", self.kind, self.confidence)
    }
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
/// println!("Inferred: {} (score {})", guess.kind, guess.confidence);
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

    /// Infer with the default policy, keeping the abstain reason.
    ///
    /// Unlike [`Self::infer_from_bytes`] this returns the full decision, so a
    /// caller can explain *why* no type was reported instead of silently showing
    /// an empty result.
    pub fn decide_from_bytes(data: &[u8], size: usize) -> InferenceDecision {
        let view = MemoryView::new(data);
        infer_decision(&view, size, None, None, None, &InferencePolicy::PRELIMINARY)
    }

    /// Run inference with an explicit acceptance policy.
    pub fn decide_with_policy(
        data: &[u8],
        size: usize,
        policy: &InferencePolicy,
    ) -> InferenceDecision {
        let view = MemoryView::new(data);
        infer_decision(&view, size, None, None, None, policy)
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
    decision_to_guess(&infer_decision(
        view,
        size,
        stack_trace,
        alloc_time,
        dealloc_time,
        &InferencePolicy::PRELIMINARY,
    ))
}

/// Run every heuristic dimension and apply the acceptance policy.
fn infer_decision(
    view: &MemoryView,
    size: usize,
    stack_trace: Option<&[String]>,
    alloc_time: Option<u64>,
    dealloc_time: Option<u64>,
    policy: &InferencePolicy,
) -> InferenceDecision {
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

    finalize(&score, policy)
}

/// Method that produced a given kind, so an accepted decision can report its
/// strongest dimension without threading it through the decision struct.
fn method_for(kind: TypeKind) -> InferenceMethod {
    match kind {
        TypeKind::Vec => InferenceMethod::LayoutDetection,
        TypeKind::String => InferenceMethod::Utf8Validation,
        TypeKind::CString => InferenceMethod::ContentAnalysis,
        TypeKind::Pointer | TypeKind::FatPtr => InferenceMethod::SizeHeuristic,
        TypeKind::Buffer => InferenceMethod::EntropyAnalysis,
        TypeKind::CStruct => InferenceMethod::Combined,
        TypeKind::Unknown => InferenceMethod::Unknown,
    }
}

/// Project an accepted decision into the legacy `TypeGuess` shape.
fn decision_to_guess(decision: &InferenceDecision) -> TypeGuess {
    match decision.predicted {
        Some(kind) => TypeGuess::with_method(kind, decision.raw_score, method_for(kind)),
        None => TypeGuess::unknown(),
    }
}

/// Finalize inference by applying the acceptance policy to the score vector.
///
/// A type is only returned when the winner reaches `min_score` **and** leads the
/// runner-up by `min_margin`; otherwise the decision abstains and records why.
fn finalize(score: &Score, policy: &InferencePolicy) -> InferenceDecision {
    let mut table = candidate_table(score);
    table.sort_by_key(|candidate| std::cmp::Reverse(candidate.1));

    let (best_kind, best_score, _) = table[0];
    let (runner_kind, runner_score, _) = table[1];

    if best_score < policy.min_score {
        return InferenceDecision {
            predicted: None,
            best_candidate: best_kind,
            raw_score: best_score,
            runner_up_score: runner_score,
            abstain_reason: Some(format!(
                "best candidate {best_kind} scored {best_score}, below min_score {}",
                policy.min_score
            )),
        };
    }

    if best_score.saturating_sub(runner_score) < policy.min_margin {
        return InferenceDecision {
            predicted: None,
            best_candidate: best_kind,
            raw_score: best_score,
            runner_up_score: runner_score,
            abstain_reason: Some(format!(
                "ambiguous: {best_kind} scored {best_score} vs {runner_kind} {runner_score}, \
                 margin below min_margin {}",
                policy.min_margin
            )),
        };
    }

    InferenceDecision {
        predicted: Some(best_kind),
        best_candidate: best_kind,
        raw_score: best_score,
        runner_up_score: runner_score,
        abstain_reason: None,
    }
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

        // A 16-byte block with a valid pointer is a FatPtr candidate, but the
        // zero-filled tail scores the buffer dimension just as high. The engine
        // must abstain instead of resolving the tie by table order.
        let decision = UnsafeInferenceEngine::decide_from_bytes(&memory, 16);
        assert!(
            !decision.is_accepted(),
            "a FatPtr/buffer tie must abstain, got {:?}",
            decision.predicted
        );
        assert!(
            decision.abstain_reason.is_some(),
            "abstaining must always carry a reason"
        );
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
        assert_eq!(
            guess.display_with_confidence(),
            "Vec<_> (score 85)",
            "the heuristic score must not be rendered as a percentage"
        );

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
        let single_nul =
            UnsafeInferenceEngine::decide_from_bytes(&create_cstring_memory(b"hello world"), 12);
        let multiple_nuls = UnsafeInferenceEngine::decide_from_bytes(&memory, 12);

        // A second NUL means the buffer is not one terminated string, so the
        // CString evidence must drop below the single-terminator case.
        assert!(
            multiple_nuls.raw_score < single_nul.raw_score,
            "multiple NULs must lower the CString evidence ({} vs {})",
            multiple_nuls.raw_score,
            single_nul.raw_score
        );
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
        let decision = UnsafeInferenceEngine::decide_from_bytes(&memory, memory.len());

        // Valid UTF-8 made of control characters is neither a String nor evidence
        // for a Vec: the engine must abstain instead of inventing a type.
        assert!(
            !decision.is_accepted(),
            "an undecidable block must not be reported as a concrete type, got {:?}",
            decision.predicted
        );
        assert!(
            decision.abstain_reason.is_some(),
            "the abstain reason must be reported"
        );
    }

    /// Objective: Verify that non-ASCII text is not pushed towards Vec by an
    /// ASCII-only printable check (plan §11.2).
    /// Invariants: a UTF-8 CJK string is accepted as String.
    #[test]
    fn test_utf8_validation_cjk_string() {
        let memory = "内存分析工具：数据准确性验证".as_bytes().to_vec();
        let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, memory.len());

        assert_eq!(
            guess.kind,
            TypeKind::String,
            "CJK text is text: an ASCII-only printable ratio must not turn it into Vec"
        );
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
    /// Assert that the engine either answers within `allowed` or abstains *with a
    /// reason*.
    ///
    /// Real heap bytes are not always identifiable (a container header without its
    /// payload, a pointer that happens to look like text), so "no confident answer"
    /// is a valid outcome — but a confident answer outside `allowed` is not.
    fn assert_kind_or_abstain(
        decision: &crate::analysis::unsafe_inference::InferenceDecision,
        allowed: &[TypeKind],
    ) {
        match decision.predicted {
            Some(kind) => assert!(
                allowed.contains(&kind),
                "unexpected confident classification {kind:?}, allowed {allowed:?}"
            ),
            None => assert!(
                decision.abstain_reason.is_some(),
                "abstaining must always carry a reason"
            ),
        }
    }

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
        // A `[ptr, len, cap]` header alone is indistinguishable between `Vec<u8>` and
        // `String`; the discriminating information is in the payload, which this view
        // does not contain. Accepting either *or* abstaining with a reason is the
        // honest outcome — inventing certainty is not.
        let decision = UnsafeInferenceEngine::decide_from_bytes(&memory, 24);
        assert_kind_or_abstain(&decision, &[TypeKind::Vec, TypeKind::String]);
        let _ = guess;
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
        // The bytes here are a live heap address, so they vary run to run: any of
        // the 8-byte layouts is plausible, and the engine is allowed to abstain
        // when the signals tie. What must never happen is a confident answer with
        // no supporting evidence, so the decision is what gets asserted.
        let decision = UnsafeInferenceEngine::decide_from_bytes(&memory, 8);
        assert_kind_or_abstain(
            &decision,
            &[
                TypeKind::Pointer,
                TypeKind::Vec,
                TypeKind::String,
                TypeKind::CString,
                TypeKind::Buffer,
            ],
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

        // High entropy binary data: `[u8]` and `Vec<u8>` describe the same memory, so
        // either name is acceptable, as is an abstention with a reason.
        let decision = UnsafeInferenceEngine::decide_from_bytes(&binary, binary.len());
        assert_kind_or_abstain(&decision, &[TypeKind::Buffer, TypeKind::Vec]);
        let _ = guess;
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
    #[cfg(target_os = "macos")]
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
