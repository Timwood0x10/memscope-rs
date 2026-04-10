//! Node ID Module
//!
//! This module provides a unified node identity system for the memscope graph model.
//!
//! # Design Principles
//!
//! - Graph identity is independent of memory address
//! - NodeID is globally unique and thread-safe
//! - Container types do not require a pointer
//!
//! # Architecture
//!
//! ```text
//! Allocation → NodeID (unique)
//! Memory Address → Optional pointer
//! Graph → NodeID-based edges
//! ```

use std::sync::atomic::{AtomicU64, Ordering};

/// Unique node identifier
///
/// This type represents a unique identifier for each node in the ownership graph.
/// It is globally unique and thread-safe.
///
/// # Examples
///
/// ```
/// use memscope_rs::analysis::node_id::NodeId;
///
/// let id1 = NodeId::new();
/// let id2 = NodeId::new();
/// assert_ne!(id1, id2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u64);

impl NodeId {
    /// Create a new unique node ID
    ///
    /// This uses an atomic counter to ensure global uniqueness and thread safety.
    ///
    /// # Returns
    ///
    /// A new unique `NodeId`.
    ///
    /// # Examples
    ///
    /// ```
    /// use memscope_rs::analysis::node_id::NodeId;
    ///
    /// let id = NodeId::new();
    /// assert!(id.0 > 0);
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self(NODE_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Create NodeID from a raw value
    ///
    /// # Safety
    ///
    /// This function should only be used when you are certain that the value
    /// does not conflict with existing node IDs. Use with caution.
    ///
    /// # Arguments
    ///
    /// * `value` - The raw u64 value
    ///
    /// # Returns
    ///
    /// A `NodeId` with the specified value.
    #[inline]
    pub const fn from_raw(value: u64) -> Self {
        Self(value)
    }

    /// Get the raw u64 value
    ///
    /// # Returns
    ///
    /// The raw u64 value of this NodeID.
    #[inline]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Check if this NodeID is valid (non-zero)
    ///
    /// # Returns
    ///
    /// `true` if the NodeID is valid, `false` otherwise.
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.0 > 0
    }
}

impl Default for NodeId {
    /// Default implementation for NodeId
    ///
    /// Creates a new unique NodeID.
    ///
    /// # Returns
    ///
    /// A new unique `NodeId`.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    /// Display implementation for NodeId
    ///
    /// # Returns
    ///
    /// Formatted string representation of the NodeId.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Virtual pointer base address for Container types.
///
/// Container types (like HashMap, Vec, etc.) don't have real heap pointers.
/// We use virtual addresses in a reserved range to avoid collisions with
/// real heap allocations.
///
/// The 1TB (0x10000000000) threshold is chosen to be:
/// - High enough to avoid conflicts with real heap addresses on any platform
/// - Low enough to fit in 64-bit address space comfortably
/// - Aligned to a large boundary for easy identification
pub const VIRTUAL_PTR_BASE: usize = 0x10000000000;

/// Check if a pointer is a virtual pointer used for Container types.
///
/// Returns `true` if the pointer is >= VIRTUAL_PTR_BASE (indicating it's
/// a virtual pointer for Container types rather than a real heap address).
#[inline]
pub const fn is_virtual_pointer(ptr: usize) -> bool {
    ptr >= VIRTUAL_PTR_BASE
}

/// Global node counter
///
/// This atomic counter ensures that all NodeIDs are globally unique
/// and thread-safe across the entire application.
static NODE_COUNTER: AtomicU64 = AtomicU64::new(1);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_uniqueness() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_node_id_validity() {
        let id = NodeId::new();
        assert!(id.is_valid());

        let invalid_id = NodeId::from_raw(0);
        assert!(!invalid_id.is_valid());
    }

    #[test]
    fn test_node_id_from_raw() {
        let value = 42u64;
        let id = NodeId::from_raw(value);
        assert_eq!(id.as_u64(), value);
    }

    #[test]
    fn test_node_id_ord() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        assert!(id1 < id2);
    }
}
