//! TrackKind - Core enum for three-layer object model
//!
//! This module defines the `TrackKind` enum which classifies memory allocations
//! into three semantic roles: HeapOwner, Container, and Value. This classification
//! enables the tracker to handle complex types like HashMap without fake pointers
//! and optimizes HeapScanner performance by only scanning HeapOwner allocations.

use serde::{Deserialize, Serialize};

/// Memory allocation semantic role classification
///
/// This enum represents the three-layer object model used by memscope:
/// - **HeapOwner**: Objects that truly own heap memory (Vec, Box, Arc, Rc, String)
/// - **Container**: Objects that organize data but don't directly expose heap (HashMap, BTreeMap)
/// - **Value**: Plain data without heap allocation (primitive types, structs)
/// - **StackOwner**: Objects that are allocated on the stack but contain pointers to heap (Arc, Rc)
///
/// # Examples
///
/// ```ignore
/// // Vec is a HeapOwner - it owns its heap buffer
/// let v: Vec<i32> = vec![1, 2, 3];
/// assert!(matches!(v.track_kind(), TrackKind::HeapOwner { .. }));
///
/// // HashMap is a Container - it organizes data internally
/// let mut map: HashMap<&str, i32> = HashMap::new();
/// assert_eq!(map.track_kind(), TrackKind::Container);
///
/// // Arc is a StackOwner - it's on stack but points to heap
/// let arc = Arc::new(42);
/// assert!(matches!(arc.track_kind(), TrackKind::StackOwner { .. }));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrackKind {
    /// Objects that truly own heap memory
    ///
    /// These types have stable heap pointers that can be safely scanned by HeapScanner.
    /// Examples include Vec, Box, Arc, Rc, and String.
    HeapOwner { ptr: usize, size: usize },

    /// Containers that organize data but don't directly expose heap
    ///
    /// These types have internal heap allocations but the pointers are not stable
    /// or not accessible through standard APIs. They are tracked as graph nodes
    /// but not scanned by HeapScanner.
    /// Examples include HashMap, BTreeMap, VecDeque.
    Container,

    /// Plain data without heap allocation
    ///
    /// These types don't produce heap allocations. They are tracked for completeness
    /// but don't participate in heap scanning or pointer analysis.
    /// Examples include primitive types, simple structs, enums.
    Value,

    /// Objects allocated on stack that contain pointers to heap
    ///
    /// These types are allocated on the stack (8 bytes for pointer) but point to
    /// heap allocations. Examples include Arc and Rc smart pointers.
    StackOwner {
        ptr: usize,
        heap_ptr: usize,
        size: usize,
    },
}

impl TrackKind {
    /// Check if this is a HeapOwner allocation
    pub fn is_heap_owner(&self) -> bool {
        matches!(self, TrackKind::HeapOwner { .. })
    }

    /// Check if this is a Container
    pub fn is_container(&self) -> bool {
        matches!(self, TrackKind::Container)
    }

    /// Check if this is a Value
    pub fn is_value(&self) -> bool {
        matches!(self, TrackKind::Value)
    }

    /// Check if this is a StackOwner allocation
    pub fn is_stack_owner(&self) -> bool {
        matches!(self, TrackKind::StackOwner { .. })
    }

    /// Get heap pointer and size if this is a HeapOwner
    pub fn as_heap_owner(&self) -> Option<(usize, usize)> {
        if let TrackKind::HeapOwner { ptr, size } = self {
            Some((*ptr, *size))
        } else {
            None
        }
    }

    /// Get stack pointer, heap pointer, and size if this is a StackOwner
    pub fn as_stack_owner(&self) -> Option<(usize, usize, usize)> {
        if let TrackKind::StackOwner {
            ptr,
            heap_ptr,
            size,
        } = self
        {
            Some((*ptr, *heap_ptr, *size))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_kind_heap_owner() {
        let kind = TrackKind::HeapOwner {
            ptr: 0x1000,
            size: 1024,
        };
        assert!(kind.is_heap_owner());
        assert!(!kind.is_container());
        assert!(!kind.is_value());
        assert_eq!(kind.as_heap_owner(), Some((0x1000, 1024)));
    }

    #[test]
    fn test_track_kind_container() {
        let kind = TrackKind::Container;
        assert!(!kind.is_heap_owner());
        assert!(kind.is_container());
        assert!(!kind.is_value());
        assert_eq!(kind.as_heap_owner(), None);
    }

    #[test]
    fn test_track_kind_value() {
        let kind = TrackKind::Value;
        assert!(!kind.is_heap_owner());
        assert!(!kind.is_container());
        assert!(kind.is_value());
        assert_eq!(kind.as_heap_owner(), None);
    }

    #[test]
    fn test_track_kind_serialization() {
        let kinds = vec![
            TrackKind::HeapOwner {
                ptr: 0x1000,
                size: 1024,
            },
            TrackKind::Container,
            TrackKind::Value,
        ];

        for kind in kinds {
            let serialized = serde_json::to_string(&kind).unwrap();
            let deserialized: TrackKind = serde_json::from_str(&serialized).unwrap();
            assert_eq!(kind, deserialized);
        }
    }

    #[test]
    fn test_track_kind_matches_pattern() {
        let kind = TrackKind::HeapOwner {
            ptr: 0x1000,
            size: 1024,
        };
        assert!(matches!(kind, TrackKind::HeapOwner { .. }));

        let kind = TrackKind::Container;
        assert!(matches!(kind, TrackKind::Container));

        let kind = TrackKind::Value;
        assert!(matches!(kind, TrackKind::Value));
    }
}
