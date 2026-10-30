//! Unit tests for the core memory tracker.

use super::*;
use std::sync::Arc;
use std::thread;

/// Objective: Verify MemoryTracker creation with default values
/// Invariants: New tracker should have zero allocations and fast_mode enabled in test
#[test]
fn test_memory_tracker_creation() {
    let tracker = MemoryTracker::new();
    assert!(
        tracker.is_fast_mode(),
        "Fast mode should be enabled in test mode"
    );

    let stats = tracker.get_stats().expect("Should get stats");
    assert_eq!(
        stats.total_allocations, 0,
        "Initial total allocations should be 0"
    );
    assert_eq!(
        stats.active_allocations, 0,
        "Initial active allocations should be 0"
    );
    assert_eq!(
        stats.peak_allocations, 0,
        "Initial peak allocations should be 0"
    );
}

/// Objective: Verify Default trait implementation
/// Invariants: Default should create same as new()
#[test]
fn test_memory_tracker_default() {
    let tracker = MemoryTracker::default();
    let stats = tracker.get_stats().expect("Should get stats");
    assert_eq!(
        stats.total_allocations, 0,
        "Default tracker should have 0 allocations"
    );
}

/// Objective: Verify track_allocation updates statistics correctly
/// Invariants: Should increment total_allocations and active_allocations
#[test]
fn test_track_allocation() {
    let tracker = MemoryTracker::new();
    let result = tracker.track_allocation(0x1000, 1024);
    assert!(result.is_ok(), "track_allocation should succeed");

    let stats = tracker.get_stats().expect("Should get stats");
    assert_eq!(stats.total_allocations, 1, "Total allocations should be 1");
    assert_eq!(
        stats.active_allocations, 1,
        "Active allocations should be 1"
    );
    assert_eq!(
        stats.total_allocated, 1024,
        "Total allocated should be 1024"
    );
}

/// Objective: Verify track_deallocation removes allocation correctly
/// Invariants: Should decrement active_allocations and increment total_deallocations
#[test]
fn test_track_deallocation() {
    let tracker = MemoryTracker::new();
    tracker.track_allocation(0x1000, 1024).unwrap();
    let result = tracker.track_deallocation(0x1000);
    assert!(result.is_ok(), "track_deallocation should succeed");
    assert!(
        result.unwrap(),
        "track_deallocation should return true for tracked pointer"
    );

    let stats = tracker.get_stats().expect("Should get stats");
    assert_eq!(
        stats.total_deallocations, 1,
        "Total deallocations should be 1"
    );
    assert_eq!(
        stats.active_allocations, 0,
        "Active allocations should be 0"
    );
    assert_eq!(
        stats.total_deallocated, 1024,
        "Total deallocated should be 1024"
    );
}

/// Objective: Verify deallocation of untracked pointer returns false
/// Invariants: Should return Ok(false) for untracked pointer
#[test]
fn test_deallocation_untracked_pointer() {
    let tracker = MemoryTracker::new();
    let result = tracker.track_deallocation(0xdead);
    assert!(result.is_ok(), "Should not error on untracked pointer");
    assert!(
        !result.unwrap(),
        "Should return false for untracked pointer"
    );
}

/// Objective: Verify get_allocation_size returns correct size
/// Invariants: Should return Some(size) for tracked pointer
#[test]
fn test_get_allocation_size() {
    let tracker = MemoryTracker::new();
    tracker.track_allocation(0x1000, 2048).unwrap();

    let size = tracker.get_allocation_size(0x1000);
    assert_eq!(size, Some(2048), "Should return correct allocation size");
}

/// Objective: Verify get_allocation_size returns None for untracked pointer
/// Invariants: Should return None for untracked pointer
#[test]
fn test_get_allocation_size_untracked() {
    let tracker = MemoryTracker::new();

    let size = tracker.get_allocation_size(0xdead);
    assert!(size.is_none(), "Should return None for untracked pointer");
}

/// Objective: Verify associate_var updates allocation metadata
/// Invariants: Should set var_name and type_name correctly
#[test]
fn test_associate_var() {
    let tracker = MemoryTracker::new();
    tracker.track_allocation(0x1000, 1024).unwrap();
    tracker
        .associate_var(
            0x1000,
            "test_var".to_string(),
            "String".to_string(),
            Some("test.rs"),
            Some(42),
        )
        .unwrap();

    let allocations = tracker.get_active_allocations().unwrap();
    assert_eq!(
        allocations[0].var_name,
        Some("test_var".to_string()),
        "Variable name should be set"
    );
    assert_eq!(
        allocations[0].type_name,
        Some("String".to_string()),
        "Type name should be set"
    );
}

/// Objective: Verify associate_var for non-existent pointer
/// Invariants: Should succeed silently without error
#[test]
fn test_associate_var_nonexistent() {
    let tracker = MemoryTracker::new();
    let result = tracker.associate_var(
        0xdead,
        "test_var".to_string(),
        "String".to_string(),
        None,
        None,
    );
    assert!(
        result.is_ok(),
        "Should succeed silently for non-existent pointer"
    );
}

/// Objective: Verify fast_track_allocation with variable name
/// Invariants: Should track allocation with var_name in one call
#[test]
fn test_fast_track_allocation() {
    let tracker = MemoryTracker::new();
    tracker
        .fast_track_allocation(0x1000, 1024, "test_var".to_string())
        .unwrap();

    let allocations = tracker.get_active_allocations().unwrap();
    assert_eq!(
        allocations[0].var_name,
        Some("test_var".to_string()),
        "Variable name should be set"
    );
    assert_eq!(allocations[0].size, 1024, "Size should be correct");
}

/// Objective: Verify peak allocations tracking
/// Invariants: Peak should track maximum concurrent allocations
#[test]
fn test_peak_tracking() {
    let tracker = MemoryTracker::new();

    tracker.track_allocation(0x1000, 1024).unwrap();
    tracker.track_allocation(0x2000, 2048).unwrap();
    tracker.track_allocation(0x3000, 4096).unwrap();

    let stats = tracker.get_stats().unwrap();
    assert_eq!(stats.peak_allocations, 3, "Peak allocations should be 3");
    assert_eq!(stats.peak_memory, 7168, "Peak memory should be 7168");

    tracker.track_deallocation(0x1000).unwrap();
    let stats = tracker.get_stats().unwrap();
    assert_eq!(
        stats.peak_allocations, 3,
        "Peak should remain 3 after deallocation"
    );
    assert_eq!(
        stats.active_allocations, 2,
        "Active should be 2 after deallocation"
    );
}

/// Objective: Verify fast mode toggle
/// Invariants: Fast mode should be toggleable
#[test]
fn test_fast_mode() {
    let tracker = MemoryTracker::new();
    tracker.set_fast_mode(true);
    assert!(tracker.is_fast_mode(), "Fast mode should be enabled");

    tracker.set_fast_mode(false);
    assert!(!tracker.is_fast_mode(), "Fast mode should be disabled");

    tracker.enable_fast_mode();
    assert!(tracker.is_fast_mode(), "Fast mode should be enabled again");
}

/// Objective: Verify detect_leaks returns correct counts
/// Invariants: Should return count and size of active allocations
#[test]
fn test_detect_leaks() {
    let tracker = MemoryTracker::new();
    tracker.track_allocation(0x1000, 1024).unwrap();
    tracker.track_allocation(0x2000, 2048).unwrap();

    let (count, size) = tracker.detect_leaks();
    assert_eq!(count, 2, "Should detect 2 leaks");
    assert_eq!(size, 3072, "Total leak size should be 3072");

    tracker.track_deallocation(0x1000).unwrap();
    let (count, _) = tracker.detect_leaks();
    assert_eq!(count, 1, "Should detect 1 leak after deallocation");
}

/// Objective: Verify get_memory_by_type groups allocations
/// Invariants: Should group allocations by type name
#[test]
fn test_get_memory_by_type() {
    let tracker = MemoryTracker::new();
    tracker.track_allocation(0x1000, 1024).unwrap();
    tracker.track_allocation(0x2000, 2048).unwrap();

    tracker
        .associate_var(0x1000, "v1".to_string(), "String".to_string(), None, None)
        .unwrap();
    tracker
        .associate_var(0x2000, "v2".to_string(), "String".to_string(), None, None)
        .unwrap();

    let by_type = tracker.get_memory_by_type().unwrap();
    assert_eq!(
        by_type.get("String"),
        Some(&3072),
        "String type should have 3072 bytes"
    );
}

/// Objective: Verify get_memory_by_type with unknown types
/// Invariants: Unknown types should be grouped as "unknown"
#[test]
fn test_get_memory_by_type_unknown() {
    let tracker = MemoryTracker::new();
    tracker.track_allocation(0x1000, 1024).unwrap();

    let by_type = tracker.get_memory_by_type().unwrap();
    assert_eq!(
        by_type.get("unknown"),
        Some(&1024),
        "Unknown type should have 1024 bytes"
    );
}

/// Objective: Verify export_to_json creates valid file
/// Invariants: Should create file with valid JSON content
#[test]
fn test_export_to_json() {
    let tracker = MemoryTracker::new();
    tracker.track_allocation(0x1000, 1024).unwrap();

    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_export.json");
    let result = tracker.export_to_json(&file_path);
    assert!(result.is_ok(), "Export should succeed");
    assert!(file_path.exists(), "Export file should exist");

    let content = std::fs::read_to_string(&file_path).expect("Should read file");
    assert!(!content.is_empty(), "JSON content should not be empty");
    assert!(content.contains("size"), "JSON should contain size field");
}

/// Objective: Verify export_to_memscope creates file
/// Invariants: Should create file with valid content
#[test]
fn test_export_to_memscope() {
    let tracker = MemoryTracker::new();
    tracker.track_allocation(0x1000, 1024).unwrap();

    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_export.memscope");
    let result = tracker.export_to_memscope(&file_path);
    assert!(result.is_ok(), "Export should succeed: {:?}", result);

    assert!(
        file_path.exists(),
        "Export file should exist at {:?}",
        file_path
    );

    let content = std::fs::read_to_string(&file_path).expect("Should read file");
    assert!(!content.is_empty(), "Export content should not be empty");
}

/// Objective: Verify ensure_memory_analysis_path creates directory
/// Invariants: Should create parent directory if needed
#[test]
fn test_ensure_memory_analysis_path() {
    let tracker = MemoryTracker::new();
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let nested_path = temp_dir.path().join("nested").join("dir").join("file.json");

    let result = tracker.ensure_memory_analysis_path(&nested_path);
    assert!(
        result.parent().unwrap().exists(),
        "Parent directory should be created"
    );
}

/// Objective: Verify global tracker singleton behavior
/// Invariants: Should return same instance when using global strategy
#[test]
fn test_global_tracker_singleton() {
    let tracker1 = GLOBAL_TRACKER
        .get_or_init(|| Arc::new(MemoryTracker::new()))
        .clone();
    let tracker2 = GLOBAL_TRACKER
        .get_or_init(|| Arc::new(MemoryTracker::new()))
        .clone();

    assert!(
        Arc::ptr_eq(&tracker1, &tracker2),
        "Should return same instance from GLOBAL_TRACKER"
    );
}

/// Objective: Verify thread-local tracker behavior
/// Invariants: Should register thread-local tracker
#[test]
fn test_thread_local_tracker() {
    configure_tracking_strategy(true);

    let tracker1 = get_tracker();
    let trackers = collect_all_trackers_local();

    assert!(!trackers.is_empty(), "Should have at least one tracker");
    assert!(
        trackers.iter().any(|t| Arc::ptr_eq(t, &tracker1)),
        "Current thread's tracker should be in registry"
    );
}

/// Objective: Verify registry statistics
/// Invariants: Should return correct thread count
#[test]
fn test_registry_stats() {
    configure_tracking_strategy(true);
    get_tracker();

    let stats = get_registry_stats_local();
    assert!(
        stats.total_threads_registered >= 1,
        "Should have at least one thread registered"
    );
    assert_eq!(stats.dead_references, 0, "Should have no dead references");
}

/// Objective: Verify has_active_trackers_local
/// Invariants: Should return true when trackers exist
#[test]
fn test_has_active_trackers() {
    configure_tracking_strategy(true);
    get_tracker();

    assert!(has_active_trackers_local(), "Should have active trackers");
}

/// Objective: Verify concurrent allocation tracking
/// Invariants: Should handle concurrent allocations correctly
#[test]
fn test_concurrent_allocations() {
    let tracker = Arc::new(MemoryTracker::new());
    let mut handles = vec![];

    for i in 0..10 {
        let tracker_clone = tracker.clone();
        let handle = thread::spawn(move || {
            let ptr = 0x1000 + i * 0x100;
            tracker_clone.track_allocation(ptr, 1024).unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = tracker.get_stats().unwrap();
    assert_eq!(
        stats.total_allocations, 10,
        "Should have 10 allocations from 10 threads"
    );
}

/// Objective: Verify concurrent allocation and deallocation
/// Invariants: Should maintain consistency under concurrent operations
#[test]
fn test_concurrent_alloc_dealloc() {
    let tracker = Arc::new(MemoryTracker::new());
    let mut handles = vec![];

    for i in 0..5 {
        let tracker_clone = tracker.clone();
        let handle = thread::spawn(move || {
            let ptr = 0x1000 + i * 0x100;
            tracker_clone.track_allocation(ptr, 1024).unwrap();
            tracker_clone.track_deallocation(ptr).unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = tracker.get_stats().unwrap();
    assert_eq!(
        stats.active_allocations, 0,
        "All allocations should be deallocated"
    );
    assert_eq!(
        stats.total_allocations, 5,
        "Should have 5 total allocations"
    );
    assert_eq!(
        stats.total_deallocations, 5,
        "Should have 5 total deallocations"
    );
}

/// Objective: Verify zero-size allocation handling
/// Invariants: Should handle zero-size allocation without error
#[test]
fn test_zero_size_allocation() {
    let tracker = MemoryTracker::new();
    let result = tracker.track_allocation(0x1000, 0);
    assert!(result.is_ok(), "Zero-size allocation should succeed");

    let stats = tracker.get_stats().unwrap();
    assert_eq!(
        stats.total_allocations, 1,
        "Should count zero-size allocation"
    );
}

/// Objective: Verify large allocation handling
/// Invariants: Should handle large allocations correctly
#[test]
fn test_large_allocation() {
    let tracker = MemoryTracker::new();
    let large_size = 1024 * 1024 * 1024;
    let result = tracker.track_allocation(0x1000, large_size);
    assert!(result.is_ok(), "Large allocation should succeed");

    let stats = tracker.get_stats().unwrap();
    assert_eq!(
        stats.total_allocated as usize, large_size,
        "Should track large allocation size"
    );
}

/// Objective: Verify a repeated sighting of the same address with a new size is
/// treated as an in-place reallocation, not as a second allocation.
/// Invariants: one active entry with the new size, one allocation counted, and
/// allocated/deallocated bytes stay balanced (plan A06).
#[test]
fn test_duplicate_address_allocation() {
    let tracker = MemoryTracker::new();
    tracker.track_allocation(0x1000, 1024).unwrap();
    tracker.track_allocation(0x1000, 2048).unwrap();

    let stats = tracker.get_stats().unwrap();
    assert_eq!(
        stats.active_allocations, 1,
        "Should have 1 active allocation"
    );
    assert_eq!(
        stats.total_allocations, 2,
        "a different size at the same address cannot be proven to be the same block"
    );
    assert_eq!(
        stats.active_memory, 2048,
        "only the surviving block's size is live"
    );
    assert_eq!(
        tracker.replaced_bindings(),
        1,
        "the superseded binding must be reported as a diagnostic"
    );
    assert_eq!(
        stats.total_allocated,
        stats.total_deallocated + stats.active_memory,
        "allocated minus deallocated must equal the live bytes"
    );

    let size = tracker.get_allocation_size(0x1000);
    assert_eq!(size, Some(2048), "Should have later allocation size");
}

/// Objective: Verify Drop implementation logs warnings
/// Invariants: Should not panic when dropping with active allocations
#[test]
fn test_drop_with_active_allocations() {
    let tracker = MemoryTracker::new();
    tracker.track_allocation(0x1000, 1024).unwrap();

    drop(tracker);
}

/// Objective: Verify get_active_allocations returns all allocations
/// Invariants: Should return all active allocations
#[test]
fn test_get_active_allocations() {
    let tracker = MemoryTracker::new();
    tracker.track_allocation(0x1000, 1024).unwrap();
    tracker.track_allocation(0x2000, 2048).unwrap();
    tracker.track_deallocation(0x1000).unwrap();

    let allocations = tracker.get_active_allocations().unwrap();
    assert_eq!(allocations.len(), 1, "Should have 1 active allocation");
    assert_eq!(
        allocations[0].ptr, 0x2000,
        "Remaining allocation should be at 0x2000"
    );
}

/// Objective: Verify configure_tracking_strategy logging
/// Invariants: Should accept both strategies
#[test]
fn test_configure_tracking_strategy() {
    configure_tracking_strategy(false);
    assert_eq!(
        TRACKING_STRATEGY.load(Ordering::Relaxed),
        STRATEGY_GLOBAL_SINGLETON,
        "Should set global singleton strategy"
    );

    configure_tracking_strategy(true);
    assert_eq!(
        TRACKING_STRATEGY.load(Ordering::Relaxed),
        STRATEGY_THREAD_LOCAL,
        "Should set thread-local strategy"
    );
}

/// Objective: Verify that observing the same address twice does not add bytes
/// or allocations twice (plan A06).
/// Invariants: one entry, one allocation, one block's worth of bytes, and the
/// duplicate is visible only through the diagnostic counter.
#[test]
fn test_duplicate_observation_is_not_double_counted() {
    let tracker = MemoryTracker::new();
    tracker
        .track_allocation(0x1000, 1024)
        .expect("first observation must succeed");
    tracker
        .track_allocation(0x1000, 1024)
        .expect("second observation must succeed");

    let stats = tracker.get_stats().expect("stats must be readable");
    assert_eq!(
        stats.total_allocations, 1,
        "the same address must not be counted as two allocations"
    );
    assert_eq!(
        stats.total_allocated, 1024,
        "bytes must not be added twice for the same block"
    );
    assert_eq!(
        stats.active_memory, 1024,
        "live bytes must equal the one live block"
    );
    assert_eq!(
        tracker.duplicate_observations(),
        1,
        "the duplicate must be reported through the diagnostic counter"
    );
}

/// Objective: Verify that a different size at a known address becomes a new binding
/// instead of being merged into the old one (address reuse with a lost release).
/// Invariants: two allocations are counted, the superseded bytes are booked, live bytes
/// equal the surviving block, and a full release balances the ledger.
#[test]
fn test_reused_address_opens_new_binding() {
    let tracker = MemoryTracker::new();
    tracker
        .track_allocation(0x2000, 1024)
        .expect("first observation must succeed");
    tracker
        .track_allocation(0x2000, 2048)
        .expect("second observation must succeed");

    let replaced = tracker.get_stats().expect("stats must be readable");
    assert_eq!(
        replaced.total_allocations, 2,
        "the second block must get its own identity"
    );
    assert_eq!(
        replaced.active_memory, 2048,
        "only the surviving block is live"
    );
    assert_eq!(
        tracker.replaced_bindings(),
        1,
        "the superseded binding must be reported as a diagnostic, not hidden"
    );

    assert!(
        tracker
            .track_deallocation(0x2000)
            .expect("release must succeed"),
        "the tracked address must be released"
    );

    let released = tracker.get_stats().expect("stats must be readable");
    assert_eq!(
        released.active_memory, 0,
        "releasing the only live block must leave zero live bytes, got {}",
        released.active_memory
    );
    assert_eq!(
        released.total_allocated, released.total_deallocated,
        "allocated and deallocated bytes must balance after full release"
    );
}

/// Objective: Verify that distinct addresses still accumulate normally.
/// Invariants: two addresses produce two allocations and the sum of their sizes.
#[test]
fn test_distinct_addresses_accumulate() {
    let tracker = MemoryTracker::new();
    tracker
        .track_allocation(0x3000, 512)
        .expect("first track must succeed");
    tracker
        .track_allocation(0x4000, 256)
        .expect("second track must succeed");

    let stats = tracker.get_stats().expect("stats must be readable");
    assert_eq!(stats.total_allocations, 2, "two addresses, two allocations");
    assert_eq!(stats.total_allocated, 768, "bytes must be summed");
    assert_eq!(stats.active_memory, 768, "both blocks are live");
    assert_eq!(
        tracker.duplicate_observations(),
        0,
        "distinct addresses are not duplicates"
    );
}

/// Objective: Verify a moving reallocation keeps one ledger entry at the new
/// address and conserves bytes.
/// Invariants: the old address is gone, the new one carries the new size, the
/// allocation counter does not grow, and releasing the new address zeroes live
/// bytes.
#[test]
fn test_moved_reallocation_moves_the_single_entry() {
    let tracker = MemoryTracker::new();
    tracker
        .track_allocation(0x5000, 512)
        .expect("track must succeed");
    tracker
        .track_reallocation(0x5000, 0x6000, 512, 1024)
        .expect("reallocation must succeed");

    assert_eq!(
        tracker.get_allocation_size(0x5000),
        None,
        "the old address must be closed"
    );
    assert_eq!(
        tracker.get_allocation_size(0x6000),
        Some(1024),
        "the new address must carry the new size"
    );

    let moved = tracker.get_stats().expect("stats must be readable");
    assert_eq!(
        moved.total_allocations, 1,
        "a reallocation is not a new allocation"
    );
    assert_eq!(moved.active_memory, 1024, "only the new size is live");

    assert!(
        tracker
            .track_deallocation(0x6000)
            .expect("release must succeed"),
        "the moved block must be releasable at its new address"
    );
    let released = tracker.get_stats().expect("stats must be readable");
    assert_eq!(released.active_memory, 0, "no live bytes may remain");
    assert_eq!(
        released.total_allocated, released.total_deallocated,
        "allocated and deallocated bytes must balance"
    );
}

/// Objective: Verify a reallocation of an unobserved block books only the new block
/// (plan A06 follow-up).
/// Invariants: one live entry at the new address, allocated == live bytes, no
/// deallocated bytes invented for the block that was never seen allocated.
#[test]
fn unmatched_reallocation_books_only_the_new_block() {
    let tracker = MemoryTracker::new();
    tracker
        .track_reallocation(0x1000, 0x2000, 64, 128)
        .expect("reallocation must succeed");

    assert_eq!(
        tracker.get_allocation_size(0x1000),
        None,
        "the old address was never tracked"
    );
    assert_eq!(
        tracker.get_allocation_size(0x2000),
        Some(128),
        "the new block must be recorded"
    );

    let stats = tracker.get_stats().expect("stats must be readable");
    assert_eq!(stats.total_allocations, 1, "the new block is one binding");
    assert_eq!(stats.active_allocations, 1, "one live binding");
    assert_eq!(
        stats.active_memory, 128,
        "live bytes must equal the observable block"
    );
    assert_eq!(stats.total_allocated, 128, "only the new size is allocated");
    assert_eq!(
        stats.total_deallocated, 0,
        "no bytes may be released for a block that was never allocated"
    );
    assert_eq!(
        stats.total_allocated - stats.total_deallocated,
        stats.active_memory,
        "allocated minus deallocated must equal the live bytes"
    );
}
