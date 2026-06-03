//! Allocation reconstruction from raw memory events.
//!
//! This module provides the `rebuild_allocations_from_events` function
//! which converts the raw `MemoryEvent` stream into structured allocation records
//! for dashboard consumption. It handles the full event lifecycle:
//! allocate, deallocate, reallocate, clone, and metadata events.

use crate::capture::types::AllocationInfo;
use crate::event_store::event::MemoryEventType;
use std::collections::HashMap;

/// Rebuild all allocations from event_store (unified data source).
///
/// Processes a slice of `MemoryEvent` and reconstructs the set of live
/// allocations along with their metadata (type, variable, thread, clone info).
pub fn rebuild_allocations_from_events(
    events: &[crate::event_store::event::MemoryEvent],
) -> Vec<AllocationInfo> {
    let mut active_allocations: Vec<AllocationInfo> = Vec::new();
    let mut container_allocations: Vec<AllocationInfo> = Vec::new();
    let mut clone_info_map: HashMap<usize, crate::capture::types::CloneInfo> = HashMap::new();
    let mut ptr_generations: HashMap<usize, usize> = HashMap::new();

    for event in events {
        match event.event_type {
            MemoryEventType::Allocate => {
                let stack_trace = event
                    .source_file
                    .as_ref()
                    .map(|file| format!("{}:{}", file, event.source_line.unwrap_or(0)));
                let generation = {
                    let entry = ptr_generations.entry(event.ptr).or_insert(0);
                    *entry += 1;
                    *entry
                };
                let mut alloc = AllocationInfo::new(event.ptr, event.size);
                alloc.timestamp_alloc = event.timestamp;
                alloc.var_name = event.var_name.clone();
                alloc.type_name = event.type_name.clone();
                alloc.thread_id = std::thread::current().id();
                alloc.thread_id_u64 = event.thread_id;
                alloc.stack_trace = stack_trace.map(|s| vec![s]);
                alloc.module_path = event.module_path.clone();
                alloc.stack_ptr = event.stack_ptr;
                alloc.generation_id = generation;
                active_allocations.push(alloc);
            }
            MemoryEventType::Metadata => {
                let stack_trace = event
                    .source_file
                    .as_ref()
                    .map(|file| format!("{}:{}", file, event.source_line.unwrap_or(0)));
                let mut alloc = AllocationInfo::new(0, event.size);
                alloc.timestamp_alloc = event.timestamp;
                alloc.var_name = event.var_name.clone();
                alloc.type_name = event.type_name.clone();
                alloc.thread_id = std::thread::current().id();
                alloc.thread_id_u64 = event.thread_id;
                alloc.stack_trace = stack_trace.map(|s| vec![s]);
                alloc.module_path = event.module_path.clone();
                container_allocations.push(alloc);
            }
            MemoryEventType::Clone => {
                if let (Some(source_ptr), Some(target_ptr)) =
                    (event.clone_source_ptr, event.clone_target_ptr)
                {
                    clone_info_map
                        .entry(source_ptr)
                        .and_modify(|info| info.clone_count += 1)
                        .or_insert(crate::capture::types::CloneInfo {
                            clone_count: 1,
                            is_clone: false,
                            original_ptr: None,
                            _source: None,
                            _confidence: None,
                        });

                    clone_info_map
                        .entry(target_ptr)
                        .and_modify(|info| {
                            info.is_clone = true;
                            info.original_ptr = Some(source_ptr);
                        })
                        .or_insert(crate::capture::types::CloneInfo {
                            clone_count: 0,
                            is_clone: true,
                            original_ptr: Some(source_ptr),
                            _source: None,
                            _confidence: None,
                        });
                }
            }
            MemoryEventType::Deallocate => {
                active_allocations.retain(|alloc| alloc.ptr != event.ptr);
            }
            _ => {}
        }
    }

    let mut all_allocations = active_allocations;

    for alloc in &mut all_allocations {
        let type_name_lower = alloc
            .type_name
            .as_ref()
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        let (is_smart_pointer, pointer_type) = if type_name_lower.contains("arc") {
            (
                true,
                crate::capture::types::smart_pointer::SmartPointerType::Arc,
            )
        } else if type_name_lower.contains("rc") {
            (
                true,
                crate::capture::types::smart_pointer::SmartPointerType::Rc,
            )
        } else if type_name_lower.contains("box") {
            (
                true,
                crate::capture::types::smart_pointer::SmartPointerType::Box,
            )
        } else {
            (
                false,
                crate::capture::types::smart_pointer::SmartPointerType::Rc,
            )
        };

        if is_smart_pointer {
            if let Some(clone_info) = clone_info_map.get(&alloc.ptr) {
                let smart_info = crate::capture::types::smart_pointer::SmartPointerInfo {
                    data_ptr: alloc.ptr,
                    pointer_type,
                    is_data_owner: !clone_info.is_clone,
                    ref_count_history: vec![],
                    weak_count: None,
                    cloned_from: clone_info.original_ptr,
                    clones: vec![],
                    is_implicitly_deallocated: false,
                    is_weak_reference: false,
                };
                alloc.smart_pointer_info = Some(smart_info);
            } else {
                let smart_info = crate::capture::types::smart_pointer::SmartPointerInfo {
                    data_ptr: alloc.ptr,
                    pointer_type,
                    is_data_owner: true,
                    ref_count_history: vec![],
                    weak_count: None,
                    cloned_from: None,
                    clones: vec![],
                    is_implicitly_deallocated: false,
                    is_weak_reference: false,
                };
                alloc.smart_pointer_info = Some(smart_info);
            }
        }
    }

    for alloc in &mut all_allocations {
        if let Some(clone_info) = clone_info_map.remove(&alloc.ptr) {
            alloc.clone_info = Some(clone_info);
        }
    }

    let container_allocations_with_virtual_ptrs: Vec<_> = container_allocations
        .into_iter()
        .enumerate()
        .map(|(index, mut alloc)| {
            alloc.ptr = crate::analysis::VIRTUAL_PTR_BASE + index;
            alloc
        })
        .collect();

    all_allocations.extend(container_allocations_with_virtual_ptrs);
    all_allocations
}

/// Allocation sampling metadata for dashboard transparency.
///
/// When the allocation set is large, the dashboard may sample the data
/// to keep render times manageable. This struct records what was done
/// so the frontend can display a sampling notice.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SamplingMetadata {
    /// Total number of raw allocations before sampling
    pub original_count: usize,
    /// Number of allocations actually included in the dashboard
    pub displayed_count: usize,
    /// Whether sampling was applied
    pub is_sampled: bool,
    /// Description of the sampling strategy used
    pub sampling_strategy: String,
}

impl SamplingMetadata {
    /// Create sampling metadata for an unsampled (complete) dataset
    pub fn complete(count: usize) -> Self {
        Self {
            original_count: count,
            displayed_count: count,
            is_sampled: false,
            sampling_strategy: "complete".to_string(),
        }
    }

    /// Create sampling metadata for a truncated dataset
    pub fn truncated(original_count: usize, displayed_count: usize, strategy: &str) -> Self {
        Self {
            original_count,
            displayed_count,
            is_sampled: true,
            sampling_strategy: strategy.to_string(),
        }
    }
}

/// Maximum allocations to include in dashboard output before sampling.
/// Set to `None` to disable sampling entirely.
pub const MAX_DASHBOARD_ALLOCATIONS: Option<usize> = None;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::event::MemoryEvent;

    /// Objective: Verify that rebuild_allocations_from_events correctly
    /// reconstructs allocate/deallocate pairs.
    /// Invariants: After allocate+deallocate, no active allocations remain.
    #[test]
    fn test_rebuild_allocate_deallocate() {
        let events = vec![
            MemoryEvent::allocate(0x100, 64, 1),
            MemoryEvent::deallocate(0x100, 64, 1),
        ];

        let result = rebuild_allocations_from_events(&events);
        assert!(
            result.is_empty(),
            "Allocations should be empty after deallocate"
        );
    }

    /// Objective: Verify that rebuild_allocations_from_events preserves
    /// allocation attributes from events.
    /// Invariants: Variable name, type name, and thread_id should be
    /// carried forward to the reconstructed allocation.
    #[test]
    fn test_rebuild_preserves_attributes() {
        let mut event = MemoryEvent::allocate(0x200, 128, 42);
        event.var_name = Some("my_var".to_string());
        event.type_name = Some("Vec<u8>".to_string());

        let events = vec![event];
        let result = rebuild_allocations_from_events(&events);

        assert_eq!(result.len(), 1, "Should have one allocation");
        assert_eq!(result[0].var_name.as_deref(), Some("my_var"));
        assert_eq!(result[0].type_name.as_deref(), Some("Vec<u8>"));
        assert_eq!(result[0].thread_id_u64, 42);
    }

    /// Objective: Verify that address reuse does not cause false
    /// duplicate allocations. After deallocate, a new allocation at
    /// the same pointer should be treated as a new allocation.
    /// Invariants: Two allocations with the same ptr but separated by
    /// a deallocation should both appear in the final set (the second one).
    #[test]
    fn test_rebuild_address_reuse() {
        let events = vec![
            MemoryEvent::allocate(0x100, 64, 1),
            MemoryEvent::deallocate(0x100, 64, 1),
            MemoryEvent::allocate(0x100, 128, 1),
        ];

        let result = rebuild_allocations_from_events(&events);
        assert_eq!(
            result.len(),
            1,
            "Should have one active allocation after reuse"
        );
        assert_eq!(
            result[0].size, 128,
            "Should reflect the newer allocation size"
        );
    }

    /// Objective: Verify that clone events produce correct clone info.
    /// Invariants: Clone source should have clone_count > 0,
    /// clone target should have is_clone = true.
    #[test]
    fn test_rebuild_clone_relationship() {
        let events = vec![
            MemoryEvent::allocate(0x100, 64, 1),
            MemoryEvent::allocate(0x200, 64, 1),
            MemoryEvent::clone_event(0x100, 0x200, 64, 1, None, None),
        ];

        let result = rebuild_allocations_from_events(&events);
        let source = result.iter().find(|a| a.ptr == 0x100).unwrap();
        let target = result.iter().find(|a| a.ptr == 0x200).unwrap();

        assert_eq!(
            source
                .clone_info
                .as_ref()
                .map(|c| c.clone_count)
                .unwrap_or(0),
            1,
            "Source should have clone count 1"
        );
        assert!(
            target
                .clone_info
                .as_ref()
                .map(|c| c.is_clone)
                .unwrap_or(false),
            "Target should be marked as clone"
        );
    }

    /// Objective: Verify that SamplingMetadata indicates unsampled for complete data.
    /// Invariants: is_sampled is false, original_count == displayed_count.
    #[test]
    fn test_sampling_metadata_complete() {
        let meta = SamplingMetadata::complete(100);
        assert!(!meta.is_sampled);
        assert_eq!(meta.original_count, 100);
        assert_eq!(meta.displayed_count, 100);
        assert_eq!(meta.sampling_strategy, "complete");
    }

    /// Objective: Verify that SamplingMetadata indicates sampled for truncated data.
    /// Invariants: is_sampled is true, original_count > displayed_count.
    #[test]
    fn test_sampling_metadata_truncated() {
        let meta = SamplingMetadata::truncated(1000, 100, "top-100-by-size");
        assert!(meta.is_sampled);
        assert_eq!(meta.original_count, 1000);
        assert_eq!(meta.displayed_count, 100);
        assert_eq!(meta.sampling_strategy, "top-100-by-size");
    }
}
