//! Smart pointer tracking types.
//!
//! This module contains types for tracking Rc, Arc, Box, and Weak
//! smart pointer operations and reference counts.

use serde::{Deserialize, Serialize};

/// Smart pointer specific information for Rc/Arc tracking.
///
/// Tracks reference counting operations, clone relationships,
/// and lifecycle information for smart pointers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmartPointerInfo {
    /// Data pointer - points to the actual data being shared.
    pub data_ptr: usize,

    /// Clone relationship tracking - source of this clone.
    pub cloned_from: Option<usize>,
    /// Clones of this smart pointer.
    pub clones: Vec<usize>,

    /// Reference count history (timestamp, count).
    pub ref_count_history: Vec<RefCountSnapshot>,

    /// Weak reference information.
    pub weak_count: Option<usize>,
    /// Is this a weak reference?
    pub is_weak_reference: bool,

    /// Lifecycle information - is this the last strong reference?
    pub is_data_owner: bool,
    /// Was data deallocated when this was dropped?
    pub is_implicitly_deallocated: bool,

    /// Smart pointer type.
    pub pointer_type: SmartPointerType,
}

/// Reference count snapshot at a specific time.
///
/// Records the strong and weak reference counts at a specific timestamp.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefCountSnapshot {
    /// Timestamp of the snapshot.
    pub timestamp: u64,
    /// Strong reference count.
    pub strong_count: usize,
    /// Weak reference count.
    pub weak_count: usize,
}

/// Type of smart pointer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SmartPointerType {
    /// Rc smart pointer (single-threaded reference counted).
    Rc,
    /// Arc smart pointer (atomic reference counted).
    Arc,
    /// RcWeak smart pointer (weak reference to Rc).
    RcWeak,
    /// ArcWeak smart pointer (weak reference to Arc).
    ArcWeak,
    /// Box smart pointer (unique ownership).
    Box,
}

impl SmartPointerInfo {
    /// Create new smart pointer info for Rc/Arc types.
    ///
    /// # Arguments
    ///
    /// * `data_ptr` - Pointer to the shared data
    /// * `pointer_type` - Type of the smart pointer
    /// * `strong_count` - Initial strong reference count
    /// * `weak_count` - Initial weak reference count
    ///
    /// # Examples
    ///
    /// ```
    /// use memscope_rs::capture::types::smart_pointer::{SmartPointerInfo, SmartPointerType};
    ///
    /// let info = SmartPointerInfo::new_rc_arc(0x1000, SmartPointerType::Rc, 1, 0);
    /// assert_eq!(info.data_ptr, 0x1000);
    /// assert!(info.is_data_owner);
    /// ```
    pub fn new_rc_arc(
        data_ptr: usize,
        pointer_type: SmartPointerType,
        strong_count: usize,
        weak_count: usize,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            data_ptr,
            cloned_from: None,
            clones: Vec::new(),
            ref_count_history: vec![RefCountSnapshot {
                timestamp,
                strong_count,
                weak_count,
            }],
            weak_count: Some(weak_count),
            is_weak_reference: false,
            is_data_owner: strong_count == 1,
            is_implicitly_deallocated: false,
            pointer_type,
        }
    }

    /// Create new smart pointer info for Weak references.
    ///
    /// # Arguments
    ///
    /// * `data_ptr` - Pointer to the shared data
    /// * `pointer_type` - Type of the weak pointer (RcWeak or ArcWeak)
    /// * `weak_count` - Initial weak reference count
    ///
    /// # Examples
    ///
    /// ```
    /// use memscope_rs::capture::types::smart_pointer::{SmartPointerInfo, SmartPointerType};
    ///
    /// let info = SmartPointerInfo::new_weak(0x2000, SmartPointerType::ArcWeak, 1);
    /// assert!(info.is_weak_reference);
    /// assert!(!info.is_data_owner);
    /// ```
    pub fn new_weak(data_ptr: usize, pointer_type: SmartPointerType, weak_count: usize) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            data_ptr,
            cloned_from: None,
            clones: Vec::new(),
            ref_count_history: vec![RefCountSnapshot {
                timestamp,
                strong_count: 0, // Weak references don't contribute to strong count
                weak_count,
            }],
            weak_count: Some(weak_count),
            is_weak_reference: true,
            is_data_owner: false,
            is_implicitly_deallocated: false,
            pointer_type,
        }
    }

    /// Record a clone relationship.
    ///
    /// # Arguments
    ///
    /// * `clone_ptr` - Pointer address of the clone
    /// * `source_ptr` - Pointer address of the source
    pub fn record_clone(&mut self, clone_ptr: usize, source_ptr: usize) {
        if self.cloned_from.is_none() {
            self.cloned_from = Some(source_ptr);
        }
        self.clones.push(clone_ptr);
    }

    /// Update reference count.
    ///
    /// Records a new snapshot of the reference counts.
    ///
    /// # Arguments
    ///
    /// * `strong_count` - Current strong reference count
    /// * `weak_count` - Current weak reference count
    pub fn update_ref_count(&mut self, strong_count: usize, weak_count: usize) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.ref_count_history.push(RefCountSnapshot {
            timestamp,
            strong_count,
            weak_count,
        });

        self.weak_count = Some(weak_count);
        self.is_data_owner = strong_count == 1 && !self.is_weak_reference;
    }

    /// Mark as implicitly deallocated.
    ///
    /// Called when data was freed because this pointer was the last strong reference.
    pub fn mark_implicitly_deallocated(&mut self) {
        self.is_implicitly_deallocated = true;
    }

    /// Get the latest reference counts.
    ///
    /// Returns the most recent reference count snapshot, or None if no history exists.
    pub fn latest_ref_counts(&self) -> Option<&RefCountSnapshot> {
        self.ref_count_history.last()
    }

    /// Check if this smart pointer is a reference counting type (Rc or Arc).
    pub fn is_reference_counted(&self) -> bool {
        matches!(
            self.pointer_type,
            SmartPointerType::Rc
                | SmartPointerType::Arc
                | SmartPointerType::RcWeak
                | SmartPointerType::ArcWeak
        )
    }

    /// Check if this smart pointer is thread-safe (Arc or ArcWeak).
    pub fn is_thread_safe(&self) -> bool {
        matches!(
            self.pointer_type,
            SmartPointerType::Arc | SmartPointerType::ArcWeak
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_pointer_info_rc_arc() {
        let info = SmartPointerInfo::new_rc_arc(0x1000, SmartPointerType::Rc, 1, 0);

        assert_eq!(info.data_ptr, 0x1000);
        assert!(matches!(info.pointer_type, SmartPointerType::Rc));
        assert!(info.is_data_owner);
        assert!(!info.is_weak_reference);
        assert_eq!(info.ref_count_history.len(), 1);
    }

    #[test]
    fn test_smart_pointer_info_weak() {
        let info = SmartPointerInfo::new_weak(0x2000, SmartPointerType::RcWeak, 1);

        assert_eq!(info.data_ptr, 0x2000);
        assert!(matches!(info.pointer_type, SmartPointerType::RcWeak));
        assert!(!info.is_data_owner);
        assert!(info.is_weak_reference);
    }

    #[test]
    fn test_smart_pointer_info_record_clone() {
        let mut info = SmartPointerInfo::new_rc_arc(0x1000, SmartPointerType::Arc, 1, 0);

        info.record_clone(0x2000, 0x1000);
        assert_eq!(info.cloned_from, Some(0x1000));
        assert_eq!(info.clones.len(), 1);
        assert_eq!(info.clones[0], 0x2000);
    }

    #[test]
    fn test_smart_pointer_info_update_ref_count() {
        let mut info = SmartPointerInfo::new_rc_arc(0x1000, SmartPointerType::Rc, 1, 0);

        info.update_ref_count(2, 1);
        assert_eq!(info.ref_count_history.len(), 2);
        assert_eq!(info.weak_count, Some(1));
        assert!(!info.is_data_owner); // Not owner when count > 1
    }

    #[test]
    fn test_smart_pointer_info_mark_implicitly_deallocated() {
        let mut info = SmartPointerInfo::new_rc_arc(0x1000, SmartPointerType::Arc, 1, 0);
        assert!(!info.is_implicitly_deallocated);

        info.mark_implicitly_deallocated();
        assert!(info.is_implicitly_deallocated);
    }

    #[test]
    fn test_smart_pointer_type_variants() {
        let types = vec![
            SmartPointerType::Rc,
            SmartPointerType::Arc,
            SmartPointerType::RcWeak,
            SmartPointerType::ArcWeak,
            SmartPointerType::Box,
        ];

        for ptr_type in types {
            assert!(!format!("{ptr_type:?}").is_empty());
        }
    }

    #[test]
    fn test_is_reference_counted() {
        let rc = SmartPointerInfo::new_rc_arc(0x1000, SmartPointerType::Rc, 1, 0);
        assert!(rc.is_reference_counted());

        let arc = SmartPointerInfo::new_rc_arc(0x1000, SmartPointerType::Arc, 1, 0);
        assert!(arc.is_reference_counted());

        let rc_weak = SmartPointerInfo::new_weak(0x1000, SmartPointerType::RcWeak, 1);
        assert!(rc_weak.is_reference_counted());

        let box_ptr = SmartPointerInfo::new_rc_arc(0x1000, SmartPointerType::Box, 1, 0);
        assert!(!box_ptr.is_reference_counted());
    }

    #[test]
    fn test_is_thread_safe() {
        let arc = SmartPointerInfo::new_rc_arc(0x1000, SmartPointerType::Arc, 1, 0);
        assert!(arc.is_thread_safe());

        let arc_weak = SmartPointerInfo::new_weak(0x1000, SmartPointerType::ArcWeak, 1);
        assert!(arc_weak.is_thread_safe());

        let rc = SmartPointerInfo::new_rc_arc(0x1000, SmartPointerType::Rc, 1, 0);
        assert!(!rc.is_thread_safe());
    }

    #[test]
    fn test_ref_count_snapshot() {
        let snapshot = RefCountSnapshot {
            timestamp: 1000,
            strong_count: 2,
            weak_count: 1,
        };

        assert_eq!(snapshot.timestamp, 1000);
        assert_eq!(snapshot.strong_count, 2);
        assert_eq!(snapshot.weak_count, 1);
    }
}
