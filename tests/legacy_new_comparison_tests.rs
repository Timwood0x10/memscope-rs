//! Legacy vs New System Comparison Tests
//!
//! This test module compares the legacy core::types system with the new capture::types system
//! to verify functional consistency and identify any differences.

use memscope_rs::capture::types::{
    AllocationInfo as NewAllocationInfo, BorrowInfo as NewBorrowInfo, CloneInfo as NewCloneInfo,
    MemoryStats as NewMemoryStats, TrackingError as NewTrackingError,
    TrackingResult as NewTrackingResult,
};
use memscope_rs::core::types::{
    AllocationInfo as LegacyAllocationInfo, BorrowInfo as LegacyBorrowInfo,
    CloneInfo as LegacyCloneInfo, MemoryStats as LegacyMemoryStats,
    TrackingError as LegacyTrackingError, TrackingResult as LegacyTrackingResult,
};

#[test]
fn test_allocation_info_comparison() {
    // Test legacy allocation info
    let legacy_info = LegacyAllocationInfo::new(0x1000, 1024);
    assert_eq!(legacy_info.ptr, 0x1000);
    assert_eq!(legacy_info.size, 1024);

    // Test new allocation info
    let new_info = NewAllocationInfo::new(0x1000, 1024);
    assert_eq!(new_info.ptr, 0x1000);
    assert_eq!(new_info.size, 1024);

    // Verify they have similar structure
    assert_eq!(legacy_info.ptr, new_info.ptr);
    assert_eq!(legacy_info.size, new_info.size);
}

#[test]
fn test_borrow_info_comparison() {
    // Test legacy borrow info
    let legacy_borrow = LegacyBorrowInfo::default();
    assert_eq!(legacy_borrow.immutable_borrows, 0);
    assert_eq!(legacy_borrow.mutable_borrows, 0);

    // Test new borrow info
    let new_borrow = NewBorrowInfo::default();
    assert_eq!(new_borrow.immutable_borrows, 0);
    assert_eq!(new_borrow.mutable_borrows, 0);

    // Verify they have similar default values
    assert_eq!(
        legacy_borrow.immutable_borrows,
        new_borrow.immutable_borrows
    );
    assert_eq!(legacy_borrow.mutable_borrows, new_borrow.mutable_borrows);
}

#[test]
fn test_clone_info_comparison() {
    // Test legacy clone info
    let legacy_clone = LegacyCloneInfo::default();
    assert_eq!(legacy_clone.clone_count, 0);

    // Test new clone info
    let new_clone = NewCloneInfo::default();
    assert_eq!(new_clone.clone_count, 0);

    // Verify they have similar default values
    assert_eq!(legacy_clone.clone_count, new_clone.clone_count);
}

#[test]
fn test_memory_stats_comparison() {
    // Test legacy memory stats
    let legacy_stats = LegacyMemoryStats::default();
    assert_eq!(legacy_stats.total_allocations, 0);
    assert_eq!(legacy_stats.active_allocations, 0);

    // Test new memory stats
    let new_stats = NewMemoryStats::default();
    assert_eq!(new_stats.total_allocations, 0);
    assert_eq!(new_stats.active_allocations, 0);

    // Verify they have similar default values
    assert_eq!(legacy_stats.total_allocations, new_stats.total_allocations);
    assert_eq!(
        legacy_stats.active_allocations,
        new_stats.active_allocations
    );
}

#[test]
fn test_tracking_error_comparison() {
    // Test legacy tracking error
    let legacy_error = LegacyTrackingError::TrackingDisabled;
    let _legacy_str = format!("{:?}", legacy_error);

    // Test new tracking error
    let new_error = NewTrackingError::TrackingDisabled;
    let _new_str = format!("{:?}", new_error);

    // Verify both systems have similar error types
    assert!(matches!(
        legacy_error,
        LegacyTrackingError::TrackingDisabled
    ));
    assert!(matches!(new_error, NewTrackingError::TrackingDisabled));
}

#[test]
fn test_tracking_result_comparison() {
    // Test legacy tracking result
    let legacy_ok: LegacyTrackingResult<i32> = Ok(42);
    let legacy_err: LegacyTrackingResult<i32> = Err(LegacyTrackingError::TrackingDisabled);

    assert!(legacy_ok.is_ok());
    assert!(legacy_err.is_err());

    // Test new tracking result
    let new_ok: NewTrackingResult<i32> = Ok(42);
    let new_err: NewTrackingResult<i32> = Err(NewTrackingError::TrackingDisabled);

    assert!(new_ok.is_ok());
    assert!(new_err.is_err());

    // Verify both systems have similar result behavior
    assert_eq!(legacy_ok.unwrap(), new_ok.unwrap());
}

#[test]
fn test_type_serialization_comparison() {
    // Test legacy allocation info serialization
    let legacy_info = LegacyAllocationInfo::new(0x1000, 1024);
    let legacy_json = serde_json::to_string(&legacy_info).expect("Failed to serialize legacy");

    // Test new allocation info serialization
    let new_info = NewAllocationInfo::new(0x1000, 1024);
    let new_json = serde_json::to_string(&new_info).expect("Failed to serialize new");

    // Both should serialize to JSON
    assert!(!legacy_json.is_empty());
    assert!(!new_json.is_empty());

    // Verify both contain the pointer and size information
    assert!(legacy_json.contains("4096")); // 0x1000 in decimal
    assert!(new_json.contains("4096"));
}

#[test]
fn test_type_deserialization_comparison() {
    // Test legacy allocation info serialization/deserialization cycle
    let legacy_original = LegacyAllocationInfo::new(0x1000, 1024);
    let legacy_json = serde_json::to_string(&legacy_original).expect("Failed to serialize legacy");
    let legacy_restored: LegacyAllocationInfo =
        serde_json::from_str(&legacy_json).expect("Failed to deserialize legacy");

    // Test new allocation info serialization/deserialization cycle
    let new_original = NewAllocationInfo::new(0x1000, 1024);
    let new_json = serde_json::to_string(&new_original).expect("Failed to serialize new");
    let new_restored: NewAllocationInfo =
        serde_json::from_str(&new_json).expect("Failed to deserialize new");

    // Verify both systems preserve key information through serialization
    assert_eq!(legacy_original.ptr, legacy_restored.ptr);
    assert_eq!(legacy_original.size, legacy_restored.size);
    assert_eq!(new_original.ptr, new_restored.ptr);
    assert_eq!(new_original.size, new_restored.size);
}

#[test]
fn test_error_handling_comparison() {
    // Test legacy error handling
    let legacy_result: LegacyTrackingResult<()> = Err(LegacyTrackingError::TrackingDisabled);
    if let Err(e) = legacy_result {
        assert!(matches!(e, LegacyTrackingError::TrackingDisabled));
    }

    // Test new error handling
    let new_result: NewTrackingResult<()> = Err(NewTrackingError::TrackingDisabled);
    if let Err(e) = new_result {
        assert!(matches!(e, NewTrackingError::TrackingDisabled));
    }
}

#[test]
fn test_type_fields_compatibility() {
    // Create instances of both systems
    let legacy_info = LegacyAllocationInfo::new(0x1000, 1024);
    let new_info = NewAllocationInfo::new(0x1000, 1024);

    // Verify both have pointer field
    assert_eq!(legacy_info.ptr, new_info.ptr);

    // Verify both have size field
    assert_eq!(legacy_info.size, new_info.size);

    // Additional compatibility checks can be added here
    // as more fields are identified as common between systems
}

#[test]
fn test_type_creation_patterns() {
    // Test legacy type creation
    let legacy_info = LegacyAllocationInfo::new(0x1000, 1024);

    // Test new type creation
    let new_info = NewAllocationInfo::new(0x1000, 1024);

    // Verify both use similar creation patterns
    assert_eq!(legacy_info.ptr, new_info.ptr);
    assert_eq!(legacy_info.size, new_info.size);
}
