//! Semantic classifications shared by dashboard projections.

use serde::{Deserialize, Serialize};

/// Ownership state — who owns a given allocation at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum OwnershipState {
    /// Owned by Rust code
    OwnedByRust,
    /// Borrowed by Rust code
    BorrowedByRust,
    /// Owned by foreign (FFI) code
    OwnedByForeign,
    /// Borrowed by foreign code
    BorrowedByForeign,
    /// Shared ownership (e.g., Arc)
    Shared,
    /// Ownership has been transferred
    Transferred,
    /// Ownership released
    Released,
    /// Ownership state unknown
    #[default]
    Unknown,
}

/// Unsafe invariant category — which Nomicon principle may be violated.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum UnsafeInvariant {
    /// Aliasing violation
    Aliasing,
    /// Invalid value / validity invariant
    Validity,
    /// Uninitialized memory access
    Initialized,
    /// Layout mismatch
    Layout,
    /// Drop / destructor violation
    Drop,
    /// Thread safety violation
    ThreadSafety,
    /// FFI ownership confusion
    FfiOwnership,
    /// Allocator family mismatch
    AllocatorFamily,
    /// Invariant unknown
    #[default]
    Unknown,
}

/// Lifetime kind — distinguishes different lifetime scopes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum LifetimeKind {
    /// Lifetime of the raw heap allocation
    AllocationLifetime,
    /// Lifetime of the logical owner
    OwnerLifetime,
    /// Lifetime of a borrow reference
    BorrowLifetime,
    /// Lifetime scoped to a task
    TaskLifetime,
    /// Lifetime scoped to a thread
    ThreadLifetime,
    /// Lifetime while exposed to FFI
    FfiExposureLifetime,
    /// Lifetime kind unknown
    #[default]
    Unknown,
}

/// Classification of a clone operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum CloneKind {
    /// Deep copy of heap data
    DeepClone,
    /// Rc reference-count bump
    RcClone,
    /// Arc reference-count bump
    ArcClone,
    /// Handle/copy-on-write clone
    HandleClone,
    /// Bitwise copy (Copy trait)
    CopyClone,
    /// Weak reference clone
    WeakClone,
    /// Clone kind unknown
    #[default]
    Unknown,
}
