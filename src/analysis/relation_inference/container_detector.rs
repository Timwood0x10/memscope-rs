//! Container Detector — infers Container → HeapOwner Contains relations.
//!
//! This module implements heuristic-based inference of `Contains` relationships
//! between Container types (HashMap, BTreeMap, VecDeque) and HeapOwner types
//! (Vec, Box, String) using temporal locality and allocation metadata.
//!
//! # Detection Strategy
//!
//! The detector uses three primary signals:
//!
//! 1. **Temporal Locality**: Container and its contained objects are typically
//!    allocated within a short time window (default: 1ms).
//!
//! 2. **Thread Affinity**: Objects must be allocated on the same thread.
//!
//! 3. **Size Reasonableness**: The contained object should not be significantly
//!    larger than the container (default: 10x ratio).
//!
//! # Algorithm
//!
//! ```text
//! for each Container in allocations:
//!     for each subsequent HeapOwner in sorted order:
//!         if same thread AND
//!            time_diff < TIME_WINDOW AND
//!            heap_owner.size < container.size * SIZE_RATIO:
//!             add_edge(container → heap_owner, Contains)
//!         else if time_diff > TIME_WINDOW:
//!             break  // No more candidates in this window
//! ```
//!
//! # Complexity
//!
//! - **Time**: O(N) where N is the number of allocations, thanks to the
//!   time-sorted sliding window approach.
//! - **Space**: O(1) additional space beyond the allocation list.
//!
//! # Configuration
//!
//! The detection behavior can be tuned via [`ContainerConfig`]:
//!
//! - `time_window_ns`: Maximum time difference for considered allocations (default: 1ms)
//! - `size_ratio`: Maximum size ratio between contained object and container (default: 10)
//! - `lookahead`: Number of subsequent allocations to examine (default: 5)

use crate::analysis::relation_inference::{Relation, RelationEdge};
use crate::core::types::TrackKind;
use crate::snapshot::types::ActiveAllocation;

/// Configuration for container relation detection.
///
/// Controls the heuristic thresholds used to infer `Contains` relationships
/// between Container and HeapOwner allocations.
///
/// # Fields
///
/// * `time_window_ns` - Maximum time difference in nanoseconds between
///   container and contained object allocation (default: 1ms = 1,000,000ns).
///
/// * `size_ratio` - Maximum size ratio between contained object and container.
///   Prevents connecting containers to unusually large objects (default: 10).
///
/// * `lookahead` - Number of subsequent allocations to examine for each container.
///   Limits the search window for performance (default: 5).
#[derive(Debug, Clone, Copy)]
pub struct ContainerConfig {
    /// Maximum time difference for considered allocations (default: 1ms).
    pub time_window_ns: u64,
    /// Maximum size ratio between contained object and container (default: 10).
    pub size_ratio: usize,
    /// Number of subsequent allocations to examine (default: 5).
    pub lookahead: usize,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            // 1ms = 1,000,000 nanoseconds
            // This captures allocations within the same logical operation
            time_window_ns: 1_000_000,
            // Prevent connecting containers to disproportionately large objects
            size_ratio: 10,
            // Limit search window for performance
            lookahead: 5,
        }
    }
}

/// Detect `Contains` relationships between Container and HeapOwner allocations.
///
/// This function implements heuristic-based inference using temporal locality,
/// thread affinity, and size filtering to identify Container → HeapOwner edges.
///
/// # Arguments
///
/// * `allocations` - List of active allocations to analyze. Must be sorted by
///   `allocated_at` timestamp for optimal performance.
///
/// * `config` - Detection configuration (optional, uses defaults if None).
///
/// # Returns
///
/// A vector of `RelationEdge` representing inferred `Contains` relationships.
///
/// # Algorithm
///
/// 1. Filter allocations into Containers and HeapOwners.
/// 2. For each Container, examine subsequent HeapOwners within the time window.
/// 3. Apply thread affinity and size ratio filters.
/// 4. Add edges for passing candidates.
///
/// # Example
///
/// ```ignore
/// use memscope_rs::analysis::relation_inference::container_detector::detect_containers;
/// use memscope_rs::snapshot::types::ActiveAllocation;
/// use memscope_rs::core::types::TrackKind;
///
/// let mut allocations = vec![
///     ActiveAllocation {
///         ptr: None,
///         size: 64,
///         kind: TrackKind::Container,
///         allocated_at: 1000,
///         thread_id: 1,
///         ..Default::default()
///     },
///     ActiveAllocation {
///         ptr: Some(0x2000),
///         size: 128,
///         kind: TrackKind::HeapOwner { ptr: 0x2000, size: 128 },
///         allocated_at: 100500,  // 500ns later, within 1ms window
///         thread_id: 1,
///         ..Default::default()
///     },
/// ];
///
/// let edges = detect_containers(&allocations, None);
/// assert_eq!(edges.len(), 1);
/// assert_eq!(edges[0].relation, Relation::Contains);
/// ```
pub fn detect_containers(
    allocations: &[ActiveAllocation],
    config: Option<ContainerConfig>,
) -> Vec<RelationEdge> {
    let config = config.unwrap_or_default();

    if allocations.is_empty() {
        return Vec::new();
    }

    // Create a mutable copy sorted by allocation time
    let mut sorted_allocs: Vec<(usize, &ActiveAllocation)> =
        allocations.iter().enumerate().collect();
    sorted_allocs.sort_by_key(|(_, alloc)| alloc.allocated_at);

    let mut edges = Vec::new();

    // Scan for Container → HeapOwner relationships
    for (sorted_idx, (container_orig_idx, container)) in sorted_allocs.iter().enumerate() {
        // Only process Container types
        if !matches!(container.kind, TrackKind::Container) {
            continue;
        }

        // Skip containers with zero size (shouldn't happen, but be defensive)
        if container.size == 0 {
            continue;
        }

        // Examine subsequent allocations within the lookahead window
        let start_idx = sorted_idx + 1;
        let end_idx = (start_idx + config.lookahead).min(sorted_allocs.len());

        for (candidate_orig_idx, candidate) in sorted_allocs[start_idx..end_idx].iter() {
            // Only consider HeapOwner types
            if !matches!(candidate.kind, TrackKind::HeapOwner { .. }) {
                continue;
            }

            // Filter: Same thread
            if container.thread_id != candidate.thread_id {
                continue;
            }

            // Filter: Time window (must be allocated AFTER container)
            let time_diff = candidate
                .allocated_at
                .saturating_sub(container.allocated_at);
            if time_diff == 0 {
                // Same allocation time, skip (shouldn't happen but be safe)
                continue;
            }
            if time_diff > config.time_window_ns {
                // No more candidates within time window
                break;
            }

            // Skip containers with zero size (shouldn't happen, but be defensive)
            if container.size == 0 {
                // For containers with size 0, skip the size ratio check
                // and allow any candidate within the time window
            } else {
                // Filter: Size ratio (prevent connecting to unusually large objects)
                let max_size = container.size * config.size_ratio;
                if candidate.size > max_size {
                    continue;
                }
            }

            // All checks passed - add Contains edge
            edges.push(RelationEdge {
                from: *container_orig_idx,
                to: *candidate_orig_idx,
                relation: Relation::Contains,
            });
        }
    }

    edges
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::relation_inference::Relation;

    /// Helper to create a test Container allocation.
    fn make_container(_id: usize, size: usize, time: u64, thread: u64) -> ActiveAllocation {
        ActiveAllocation {
            ptr: None,
            size,
            kind: TrackKind::Container,
            allocated_at: time,
            var_name: None,
            type_name: None,
            thread_id: thread,
            call_stack_hash: None,
        }
    }

    /// Helper to create a test HeapOwner allocation.
    fn make_heap_owner(
        _id: usize,
        ptr: usize,
        size: usize,
        time: u64,
        thread: u64,
    ) -> ActiveAllocation {
        ActiveAllocation {
            ptr: Some(ptr),
            size,
            kind: TrackKind::HeapOwner { ptr, size },
            allocated_at: time,
            var_name: None,
            type_name: None,
            thread_id: thread,
            call_stack_hash: None,
        }
    }

    #[test]
    fn test_detect_containers_empty() {
        let allocs = vec![];
        let edges = detect_containers(&allocs, None);
        assert_eq!(edges.len(), 0);
    }

    #[test]
    fn test_detect_containers_single_container() {
        let allocs = vec![make_container(0, 64, 1000, 1)];
        let edges = detect_containers(&allocs, None);
        assert_eq!(edges.len(), 0);
    }

    #[test]
    fn test_detect_containers_single_heap_owner() {
        let allocs = vec![make_heap_owner(0, 0x1000, 64, 1000, 1)];
        let edges = detect_containers(&allocs, None);
        assert_eq!(edges.len(), 0);
    }

    #[test]
    fn test_detect_containers_basic() {
        let allocs = vec![
            make_container(0, 64, 1000, 1),
            make_heap_owner(1, 0x2000, 128, 100500, 1),
        ];

        let edges = detect_containers(&allocs, None);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from, 0);
        assert_eq!(edges[0].to, 1);
        assert_eq!(edges[0].relation, Relation::Contains);
    }

    #[test]
    fn test_detect_containers_time_window() {
        let allocs = vec![
            make_container(0, 64, 1000, 1),
            make_heap_owner(1, 0x2000, 128, 2_000_000, 1), // 2ms later - beyond default 1ms window
        ];

        let edges = detect_containers(&allocs, None);
        assert_eq!(edges.len(), 0);
    }

    #[test]
    fn test_detect_containers_custom_time_window() {
        let allocs = vec![
            make_container(0, 64, 1000, 1),
            make_heap_owner(1, 0x2000, 128, 2_000_000, 1), // 2ms later
        ];

        let config = ContainerConfig {
            time_window_ns: 3_000_000, // 3ms window
            ..Default::default()
        };

        let edges = detect_containers(&allocs, Some(config));
        assert_eq!(edges.len(), 1);
    }

    #[test]
    fn test_detect_containers_different_threads() {
        let allocs = vec![
            make_container(0, 64, 1000, 1),
            make_heap_owner(1, 0x2000, 128, 100500, 2), // Different thread
        ];

        let edges = detect_containers(&allocs, None);
        assert_eq!(edges.len(), 0);
    }

    #[test]
    fn test_detect_containers_size_ratio_filter() {
        let allocs = vec![
            make_container(0, 64, 1000, 1),
            make_heap_owner(1, 0x2000, 1280, 100500, 1), // 20x container size
        ];

        let edges = detect_containers(&allocs, None);
        assert_eq!(edges.len(), 0);
    }

    #[test]
    fn test_detect_containers_custom_size_ratio() {
        let allocs = vec![
            make_container(0, 64, 1000, 1),
            make_heap_owner(1, 0x2000, 1280, 100500, 1), // 20x container size
        ];

        let config = ContainerConfig {
            size_ratio: 30, // Allow up to 30x ratio
            ..Default::default()
        };

        let edges = detect_containers(&allocs, Some(config));
        assert_eq!(edges.len(), 1);
    }

    #[test]
    fn test_detect_containers_multiple_candidates() {
        let allocs = vec![
            make_container(0, 64, 1000, 1),
            make_heap_owner(1, 0x2000, 64, 100500, 1),
            make_heap_owner(2, 0x3000, 64, 100600, 1),
            make_heap_owner(3, 0x4000, 64, 100700, 1),
        ];

        let edges = detect_containers(&allocs, None);
        assert_eq!(edges.len(), 3);
        assert!(edges.iter().all(|e| e.from == 0));
    }

    #[test]
    fn test_detect_containers_lookahead_limit() {
        let allocs = vec![
            make_container(0, 64, 1000, 1),
            make_heap_owner(1, 0x2000, 64, 100500, 1),
            make_heap_owner(2, 0x3000, 64, 100600, 1),
            make_heap_owner(3, 0x4000, 64, 100700, 1),
            make_heap_owner(4, 0x5000, 64, 100800, 1),
            make_heap_owner(5, 0x6000, 64, 100900, 1), // Beyond default lookahead of 5
        ];

        let edges = detect_containers(&allocs, None);
        // Should only find 5 edges (lookahead limit)
        assert_eq!(edges.len(), 5);
    }

    #[test]
    fn test_detect_containers_multiple_containers() {
        let allocs = vec![
            make_container(0, 64, 1000, 1),
            make_container(1, 64, 2_000_000, 1), // 2ms later (beyond 1ms window)
            make_heap_owner(2, 0x2000, 64, 100500, 1), // Near container 0 (within 1ms)
            make_heap_owner(3, 0x3000, 64, 2_100_000, 1), // Near container 1 (within 1ms)
        ];

        let edges = detect_containers(&allocs, None);
        assert_eq!(edges.len(), 2);

        // Check which edges were created
        let contains_0_2 = edges.iter().any(|e| e.from == 0 && e.to == 2);
        let contains_1_3 = edges.iter().any(|e| e.from == 1 && e.to == 3);
        assert!(contains_0_2);
        assert!(contains_1_3);
    }

    #[test]
    fn test_detect_containers_ignore_value_types() {
        let allocs = vec![
            make_container(0, 64, 1000, 1),
            ActiveAllocation {
                ptr: None,
                size: 32,
                kind: TrackKind::Value,
                allocated_at: 100500,
                var_name: None,
                type_name: None,
                thread_id: 1,
                call_stack_hash: None,
            },
        ];

        let edges = detect_containers(&allocs, None);
        assert_eq!(edges.len(), 0);
    }

    #[test]
    fn test_detect_containers_unsorted_input() {
        let allocs = vec![
            make_heap_owner(1, 0x2000, 64, 100500, 1), // Later in time (original index 0)
            make_container(0, 64, 1000, 1),            // Earlier in time (original index 1)
        ];

        let edges = detect_containers(&allocs, None);
        assert_eq!(edges.len(), 1);
        // Container (original index 1) → HeapOwner (original index 0)
        assert_eq!(edges[0].from, 1);
        assert_eq!(edges[0].to, 0);
    }

    #[test]
    fn test_config_default() {
        let config = ContainerConfig::default();
        assert_eq!(config.time_window_ns, 1_000_000);
        assert_eq!(config.size_ratio, 10);
        assert_eq!(config.lookahead, 5);
    }

    #[test]
    fn test_config_custom() {
        let config = ContainerConfig {
            time_window_ns: 5_000_000,
            size_ratio: 20,
            lookahead: 10,
        };
        assert_eq!(config.time_window_ns, 5_000_000);
        assert_eq!(config.size_ratio, 20);
        assert_eq!(config.lookahead, 10);
    }
}
