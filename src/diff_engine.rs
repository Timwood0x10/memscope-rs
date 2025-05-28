// src/diff_engine.rs
use crate::export::MemorySnapshot; // For MemorySnapshot struct
use crate::tracker::AllocationInfo; // For AllocationInfo struct
// use crate::diff_engine::SnapshotDiff; // SnapshotDiff is in the same file
use std::collections::HashMap; // For HashMap used in comparison
use serde::{Serialize, Deserialize}; // For SnapshotDiff derive

// Represents an allocation that is present in the newer snapshot but not the older one.
// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)] // PartialEq for easier testing
// pub struct NewAllocation {
//     pub info: AllocationInfo,
// }

// Represents an allocation that was present in the older snapshot but not the newer one.
// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
// pub struct FreedAllocation {
//     pub info: AllocationInfo,
// }

// Decided to simplify: SnapshotDiff will just contain Vec<AllocationInfo>
// for new and freed, as the context (new/freed) is given by the field name.

/// Represents the differences found when comparing two memory snapshots.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SnapshotDiff {
    /// Allocations present in the second (newer) snapshot but not in the first (older).
    pub new_allocations: Vec<AllocationInfo>,
    /// Allocations present in the first (older) snapshot but not in the second (newer).
    pub freed_allocations: Vec<AllocationInfo>,
    // Optional: Could add a field for allocations common to both, if needed later.
    // pub common_allocations: Vec<usize>, // e.g., just pointers
}

impl SnapshotDiff {
    // Helper constructor
    pub fn new() -> Self {
        SnapshotDiff {
            new_allocations: Vec::new(),
            freed_allocations: Vec::new(),
        }
    }
}

/// Compares two memory snapshots and identifies new and freed allocations.
///
/// Allocations are compared based on their pointer addresses.
///
/// # Arguments
/// * `snapshot1` - The older memory snapshot.
/// * `snapshot2` - The newer memory snapshot.
///
/// # Returns
/// A `SnapshotDiff` struct containing lists of new and freed allocations.
pub fn compare_snapshots(snapshot1: &MemorySnapshot, snapshot2: &MemorySnapshot) -> SnapshotDiff {
    let mut diff = SnapshotDiff::new();

    // Create HashMaps for efficient lookup by pointer (ptr is usize)
    // The active_allocations field in MemorySnapshot is Vec<AllocationInfo>
    let map1: HashMap<usize, &AllocationInfo> = snapshot1.active_allocations.iter()
        .map(|alloc_info| (alloc_info.ptr, alloc_info))
        .collect();

    let map2: HashMap<usize, &AllocationInfo> = snapshot2.active_allocations.iter()
        .map(|alloc_info| (alloc_info.ptr, alloc_info))
        .collect();

    // Identify freed allocations (in snapshot1 but not in snapshot2)
    for (ptr, alloc_info1) in &map1 {
        if !map2.contains_key(ptr) {
            diff.freed_allocations.push((*alloc_info1).clone());
        }
    }

    // Identify new allocations (in snapshot2 but not in snapshot1)
    for (ptr, alloc_info2) in &map2 {
        if !map1.contains_key(ptr) {
            diff.new_allocations.push((*alloc_info2).clone());
        }
    }

    // Optional: Identify common allocations or changes in size/metadata if AllocationInfo supports it
    // For now, this basic diff covers new and freed.

    diff
}
