// Test reallocation handling in snapshot engine
use memscope_rs::event_store::{EventStore, MemoryEvent, MemoryEventType, SharedEventStore};
use memscope_rs::snapshot::SnapshotEngine;
use std::sync::Arc;

#[test]
fn test_reallocation_updates_thread_stats() {
    let event_store = EventStore::new();
    let thread_id = 1;

    // Initial allocation
    event_store.record(MemoryEvent {
        ptr: 0x1000,
        size: 1000,
        old_size: None,
        event_type: MemoryEventType::Allocate,
        timestamp: 100,
        thread_id,
        var_name: None,
        type_name: Some("Vec<i32>".to_string()),
        call_stack_hash: None,
        thread_name: None,
        source_file: None,
        source_line: None,
        module_path: None,
        clone_source_ptr: None,
        clone_target_ptr: None,
        stack_ptr: None,
        task_id: None,
    });

    // Reallocation - increase size
    event_store.record(MemoryEvent {
        ptr: 0x1000,
        size: 2000,
        old_size: Some(1000),
        event_type: MemoryEventType::Reallocate,
        timestamp: 200,
        thread_id,
        var_name: None,
        type_name: Some("Vec<i32>".to_string()),
        call_stack_hash: None,
        thread_name: None,
        source_file: None,
        source_line: None,
        module_path: None,
        clone_source_ptr: None,
        clone_target_ptr: None,
        stack_ptr: None,
        task_id: None,
    });

    // Build snapshot from events
    let shared_store: SharedEventStore = Arc::new(event_store);
    let snapshot = SnapshotEngine::new(shared_store).build_snapshot();

    // Check reallocation stats
    let thread_stats = snapshot.thread_stats.get(&thread_id).unwrap();
    assert_eq!(thread_stats.total_allocated, 3000); // 1000 + 2000
    assert_eq!(thread_stats.total_deallocated, 1000); // Old size from reallocation
    assert_eq!(thread_stats.current_memory, 2000); // Final size
}

#[test]
fn test_deallocation_updates_thread_stats() {
    let event_store = EventStore::new();
    let thread_id = 1;

    // Allocate
    event_store.record(MemoryEvent {
        ptr: 0x1000,
        size: 1000,
        old_size: None,
        event_type: MemoryEventType::Allocate,
        timestamp: 100,
        thread_id,
        var_name: None,
        type_name: Some("Vec<i32>".to_string()),
        call_stack_hash: None,
        thread_name: None,
        source_file: None,
        source_line: None,
        module_path: None,
        clone_source_ptr: None,
        clone_target_ptr: None,
        stack_ptr: None,
        task_id: None,
    });

    // Deallocate
    event_store.record(MemoryEvent {
        ptr: 0x1000,
        size: 1000,
        old_size: None,
        event_type: MemoryEventType::Deallocate,
        timestamp: 200,
        thread_id,
        var_name: None,
        type_name: Some("Vec<i32>".to_string()),
        call_stack_hash: None,
        thread_name: None,
        source_file: None,
        source_line: None,
        module_path: None,
        clone_source_ptr: None,
        clone_target_ptr: None,
        stack_ptr: None,
        task_id: None,
    });

    // Build snapshot from events
    let shared_store: SharedEventStore = Arc::new(event_store);
    let snapshot = SnapshotEngine::new(shared_store).build_snapshot();

    // Check stats
    let thread_stats = snapshot.thread_stats.get(&thread_id).unwrap();
    assert_eq!(thread_stats.total_allocated, 1000);
    assert_eq!(thread_stats.total_deallocated, 1000);
    assert_eq!(thread_stats.current_memory, 0);
}
