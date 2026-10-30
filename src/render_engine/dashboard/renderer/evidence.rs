//! Evidence, risk-confidence and provenance enums shared by dashboard DTOs.
//!
//! Kept out of `types.rs` so the DTO definitions stay readable and within the
//! project line budget. These enums describe *how* a conclusion was reached; they
//! must not be derived from an allocation identity (plan A11).

use serde::{Deserialize, Serialize};

/// Evidence level for analysis findings — how the conclusion was reached.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum EvidenceLevel {
    /// Directly observed from events (highest confidence)
    Observed,
    /// Inferred from multiple data sources with strong correlation
    Inferred,
    /// Best-guess heuristic based on type/pattern matching
    Heuristic,
    /// No evidence available
    #[default]
    Unknown,
}

/// Risk confidence — how certain the analysis is about a risk finding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum RiskConfidence {
    /// Bug is confirmed by direct evidence
    Confirmed,
    /// Strong evidence but not conclusive
    Likely,
    /// Possible but needs more evidence
    Possible,
    /// Risk level unknown
    #[default]
    Unknown,
}

/// Pointer provenance — indicates where a pointer originated.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum PointerProvenance {
    /// Fresh allocation from the global allocator
    Allocator,
    /// Created via Clone (includes Rc/Arc clone)
    Clone,
    /// Wrapped in a smart pointer (Box, Rc, Arc)
    SmartPointer,
    /// Address was previously used by a different allocation
    Reallocation,
    /// Reallocated address that later leaked
    ReallocatedThenLeaked,
    /// Received from FFI boundary
    FfiInput,
    /// Sent to FFI boundary
    FfiOutput,
    /// Origin unknown
    #[default]
    Unknown,
}

/// Drop expectation — what the analysis expects regarding deallocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum DropExpectation {
    /// Normal Rust drop expected
    NormalDrop,
    /// ManualDrop — deallocation may be intentionally suppressed
    ManualDrop,
    /// mem::forget was called
    Forgotten,
    /// Memory should be freed by foreign code
    ForeignFree,
    /// Rust should reclaim from foreign code
    RustReclaim,
    /// No drop needed (e.g. static data, ZST)
    NoDropNeeded,
    /// Drop expectation unknown
    #[default]
    Unknown,
}
