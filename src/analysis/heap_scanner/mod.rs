//! Heap Scanner - Safe heap memory reading for offline analysis.
//!
//! Reads memory content of active allocations during snapshot analysis,
//! providing MemoryView data to the UTI Engine and Relation Engine.
//!
//! # Safety Model
//!
//! - All reads go through `ValidRegions` to prevent segfaults.
//! - Page-wise validation before any memory access.
//! - Maximum 4096 bytes read per allocation (metadata is always at the head).

mod reader;

pub use reader::{HeapScanner, ScanResult};
