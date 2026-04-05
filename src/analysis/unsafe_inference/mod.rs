//! Unsafe Type Inference Module v2
//!
//! Heuristic-based type inference for FFI/unsafe memory allocations.
//!
//! # Design Principles
//!
//! - **Six-dimensional signal model**: Size, Layout, Content, Stack, Lifetime, ValidPtr
//! - **Memory safe**: 100% safe Rust, no raw pointer dereferencing
//! - **O(n) complexity**: ~5-50ms for 1M allocations depending on enabled features
//! - **Explainable**: Each inference has a clear reasoning chain
//!
//! # Usage
//!
//! ```
//! use memscope_rs::analysis::unsafe_inference::{
//!     UnsafeInferenceEngine, TypeGuess, TypeKind, MemoryView,
//! };
//!
//! let memory = vec![0u8; 24];
//! let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);
//! println!("Inferred type: {} ({}% confidence)", guess.kind, guess.confidence);
//! ```

mod engine;
mod memory_view;

pub use engine::{InferenceMethod, InferenceRecord, TypeGuess, TypeKind, UnsafeInferenceEngine};
pub use memory_view::{
    count_valid_pointers, get_valid_regions, is_valid_ptr, is_valid_ptr_static, MemoryRegion,
    MemoryView, ValidRegions,
};
