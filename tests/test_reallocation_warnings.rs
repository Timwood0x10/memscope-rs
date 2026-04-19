// Test reallocation warning logs
use memscope_rs::event_store::{EventStore, MemoryEvent, MemoryEventType};
use memscope_rs::snapshot::engine::SnapshotEngine;
use std::sync::Arc;

#[test]
fn test_reallocation_without_old_size_logs_warning() {
    let event_store = EventStore::new();

    // Create a reallocation event without old_size or previous allocation
    let ptr = 0x1000;
    event_store.record(MemoryEvent {
        timestamp: 1,
        event_type: MemoryEventType::Reallocate,
        ptr,
        size: 2000,
        old_size: None,
        thread_id: 1,
        var_name: None,
        type_name: None,
        call_stack_hash: None,
        thread_name: None,
        source_file: None,
        source_line: None,
        module_path: None,
        clone_source_ptr: None,
        clone_target_ptr: None,
        stack_ptr: None,
    });

    let engine = SnapshotEngine::new(Arc::new(event_store));
    let snapshot = engine.build_snapshot();

    // The snapshot should still be created, but a warning should have been logged
    assert_eq!(snapshot.stats.total_reallocations, 1);
    assert_eq!(snapshot.stats.total_deallocated, 0); // Should be 0, no phantom deallocation
    assert_eq!(snapshot.stats.current_memory, 2000);
}

#[test]
fn test_reallocation_with_old_size() {
    let event_store = EventStore::new();

    // Create a reallocation event with old_size
    let ptr = 0x1000;
    event_store.record(MemoryEvent {
        timestamp: 1,
        event_type: MemoryEventType::Reallocate,
        ptr,
        size: 2000,
        old_size: Some(1000),
        thread_id: 1,
        var_name: None,
        type_name: None,
        call_stack_hash: None,
        thread_name: None,
        source_file: None,
        source_line: None,
        module_path: None,
        clone_source_ptr: None,
        clone_target_ptr: None,
        stack_ptr: None,
    });

    let engine = SnapshotEngine::new(Arc::new(event_store));
    let snapshot = engine.build_snapshot();

    assert_eq!(snapshot.stats.total_reallocations, 1);
    assert_eq!(snapshot.stats.total_deallocated, 1000);
    assert_eq!(snapshot.stats.current_memory, 2000);
}

#[test]
fn test_reallocation_with_previous_allocation() {
    let event_store = EventStore::new();

    let ptr = 0x1000;

    // First allocation
    event_store.record(MemoryEvent {
        timestamp: 1,
        event_type: MemoryEventType::Allocate,
        ptr,
        size: 1000,
        old_size: None,
        thread_id: 1,
        var_name: None,
        type_name: None,
        call_stack_hash: None,
        thread_name: None,
        source_file: None,
        source_line: None,
        module_path: None,
        clone_source_ptr: None,
        clone_target_ptr: None,
        stack_ptr: None,
    });

    // Reallocation without old_size but with previous allocation
    event_store.record(MemoryEvent {
        timestamp: 2,
        event_type: MemoryEventType::Reallocate,
        ptr,
        size: 2000,
        old_size: None,
        thread_id: 1,
        var_name: None,
        type_name: None,
        call_stack_hash: None,
        thread_name: None,
        source_file: None,
        source_line: None,
        module_path: None,
        clone_source_ptr: None,
        clone_target_ptr: None,
        stack_ptr: None,
    });

    let engine = SnapshotEngine::new(Arc::new(event_store));
    let snapshot = engine.build_snapshot();

    assert_eq!(snapshot.stats.total_allocations, 1);
    assert_eq!(snapshot.stats.total_reallocations, 1);
    assert_eq!(snapshot.stats.total_deallocated, 1000); // Should use previous allocation size
    assert_eq!(snapshot.stats.current_memory, 2000);
}
