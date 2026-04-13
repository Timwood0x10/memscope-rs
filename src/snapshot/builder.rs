//! Snapshot builder - Reusable snapshot construction logic.
//!
//! This module provides a standalone function for building snapshots
//! from events, which can be reused by both SnapshotEngine and MemoryView.

use crate::core::types::TrackKind;
use crate::event_store::{MemoryEvent, MemoryEventType};
use crate::snapshot::types::{ActiveAllocation, MemorySnapshot, ThreadMemoryStats};
use std::collections::HashMap;

/// Build a memory snapshot from a slice of events.
///
/// This is the core snapshot building logic that can be reused
/// by both SnapshotEngine and MemoryView.
pub fn build_snapshot_from_events(events: &[MemoryEvent]) -> MemorySnapshot {
    let mut snapshot = MemorySnapshot::new();
    let mut ptr_to_allocation: HashMap<usize, ActiveAllocation> = HashMap::new();
    let mut thread_stats: HashMap<u64, ThreadMemoryStats> = HashMap::new();
    let mut peak_memory: usize = 0;
    let mut current_memory: usize = 0;

    for event in events {
        match event.event_type {
            MemoryEventType::Allocate => {
                let allocation = ActiveAllocation {
                    ptr: Some(event.ptr),
                    kind: TrackKind::HeapOwner {
                        ptr: event.ptr,
                        size: event.size,
                    },
                    size: event.size,
                    allocated_at: event.timestamp,
                    var_name: event.var_name.clone(),
                    type_name: event.type_name.clone(),
                    thread_id: event.thread_id,
                    call_stack_hash: event.call_stack_hash,
                };

                ptr_to_allocation.insert(event.ptr, allocation);

                snapshot.stats.total_allocations += 1;
                snapshot.stats.total_allocated += event.size;
                current_memory += event.size;

                let thread_stat =
                    thread_stats
                        .entry(event.thread_id)
                        .or_insert_with(|| ThreadMemoryStats {
                            thread_id: event.thread_id,
                            allocation_count: 0,
                            total_allocated: 0,
                            total_deallocated: 0,
                            current_memory: 0,
                            peak_memory: 0,
                        });
                thread_stat.allocation_count += 1;
                thread_stat.total_allocated += event.size;
                thread_stat.current_memory += event.size;
                if thread_stat.current_memory > thread_stat.peak_memory {
                    thread_stat.peak_memory = thread_stat.current_memory;
                }
            }
            MemoryEventType::Reallocate => {
                let old_allocation = ptr_to_allocation.get(&event.ptr).cloned();
                let old_size = event.old_size.unwrap_or_else(|| {
                    old_allocation
                        .as_ref()
                        .map(|a| a.size)
                        .unwrap_or_else(|| {
                            tracing::warn!(
                                "Reallocation without old_size or previous allocation: ptr={:#x}, new_size={}",
                                event.ptr,
                                event.size
                            );
                            0
                        })
                });

                let allocation = ActiveAllocation {
                    ptr: Some(event.ptr),
                    kind: TrackKind::HeapOwner {
                        ptr: event.ptr,
                        size: event.size,
                    },
                    size: event.size,
                    allocated_at: old_allocation
                        .map(|a| a.allocated_at)
                        .unwrap_or(event.timestamp),
                    var_name: event.var_name.clone(),
                    type_name: event.type_name.clone(),
                    thread_id: event.thread_id,
                    call_stack_hash: event.call_stack_hash,
                };

                ptr_to_allocation.insert(event.ptr, allocation);

                snapshot.stats.total_reallocations += 1;
                snapshot.stats.total_allocated += event.size;
                snapshot.stats.total_deallocated += old_size;

                current_memory = current_memory
                    .saturating_sub(old_size)
                    .saturating_add(event.size);

                let thread_stat =
                    thread_stats
                        .entry(event.thread_id)
                        .or_insert_with(|| ThreadMemoryStats {
                            thread_id: event.thread_id,
                            allocation_count: 0,
                            total_allocated: 0,
                            total_deallocated: 0,
                            current_memory: 0,
                            peak_memory: 0,
                        });
                thread_stat.total_allocated += event.size;
                thread_stat.total_deallocated += old_size;
                thread_stat.current_memory = thread_stat
                    .current_memory
                    .saturating_sub(old_size)
                    .saturating_add(event.size);
                if thread_stat.current_memory > thread_stat.peak_memory {
                    thread_stat.peak_memory = thread_stat.current_memory;
                }
            }
            MemoryEventType::Deallocate => {
                if let Some(allocation) = ptr_to_allocation.remove(&event.ptr) {
                    snapshot.stats.total_deallocations += 1;
                    snapshot.stats.total_deallocated += allocation.size;
                    current_memory = current_memory.saturating_sub(allocation.size);

                    if let Some(thread_stat) = thread_stats.get_mut(&event.thread_id) {
                        thread_stat.total_deallocated += allocation.size;
                        thread_stat.current_memory =
                            thread_stat.current_memory.saturating_sub(allocation.size);
                    }
                } else {
                    snapshot.stats.unmatched_deallocations += 1;
                    tracing::debug!(
                        "Unmatched deallocation: ptr={:#x}, thread_id={}",
                        event.ptr,
                        event.thread_id
                    );
                }
            }
            MemoryEventType::Move | MemoryEventType::Borrow | MemoryEventType::Return => {
                // These don't affect the current memory state
            }
            MemoryEventType::Metadata => {
                // Container/Value types - no heap allocation
            }
        }

        // Update peak memory
        if current_memory > peak_memory {
            peak_memory = current_memory;
        }
    }

    // Build final snapshot
    snapshot.active_allocations = ptr_to_allocation;
    snapshot.thread_stats = thread_stats;
    snapshot.stats.active_allocations = snapshot.active_allocations.len();
    snapshot.stats.current_memory = current_memory;
    snapshot.stats.peak_memory = peak_memory;

    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_snapshot_empty() {
        let events: Vec<MemoryEvent> = vec![];
        let snapshot = build_snapshot_from_events(&events);
        assert_eq!(snapshot.active_count(), 0);
    }

    #[test]
    fn test_build_snapshot_with_allocations() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 1024, 1),
            MemoryEvent::allocate(0x2000, 2048, 1),
        ];
        let snapshot = build_snapshot_from_events(&events);
        assert_eq!(snapshot.active_count(), 2);
        assert_eq!(snapshot.current_memory(), 3072);
    }

    #[test]
    fn test_build_snapshot_with_deallocations() {
        let events = vec![
            MemoryEvent::allocate(0x1000, 1024, 1),
            MemoryEvent::deallocate(0x1000, 1024, 1),
        ];
        let snapshot = build_snapshot_from_events(&events);
        assert_eq!(snapshot.active_count(), 0);
        assert_eq!(snapshot.current_memory(), 0);
    }
}
