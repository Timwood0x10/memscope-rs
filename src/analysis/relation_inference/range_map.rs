//! RangeMap - Binary search index for allocation address ranges.
//!
//! Maps memory addresses to allocation indices using a sorted list
//! and binary search, enabling O(log n) pointer-to-allocation lookup.

use crate::analysis::is_virtual_pointer;
use crate::snapshot::types::ActiveAllocation;

/// A single entry in the range map.
#[derive(Debug, Clone)]
struct RangeEntry {
    /// Start address (inclusive).
    start: usize,
    /// End address (exclusive).
    end: usize,
    /// Index into the original allocations slice.
    alloc_id: usize,
}

/// An index structure that maps memory addresses to allocation indices.
///
/// Built once from a list of allocations, then queried many times
/// during pointer scanning.
#[derive(Debug, Default)]
pub struct RangeMap {
    /// Sorted range entries.
    entries: Vec<RangeEntry>,
}

impl RangeMap {
    /// Build a RangeMap from a list of active allocations.
    ///
    /// # Arguments
    ///
    /// * `allocations` - Slice of allocations to index. Each allocation's
    ///   index in this slice becomes its `alloc_id`.
    pub fn new(allocations: &[ActiveAllocation]) -> Self {
        let mut entries: Vec<RangeEntry> = allocations
            .iter()
            .enumerate()
            .filter_map(|(id, alloc)| {
                // Only include HeapOwner allocations with valid pointers
                // Skip virtual pointers used for Container types
                alloc.ptr.and_then(|ptr| {
                    if !is_virtual_pointer(ptr) {
                        Some(RangeEntry {
                            start: ptr,
                            end: ptr.saturating_add(alloc.size),
                            alloc_id: id,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect();

        // Sort by start address for binary search.
        entries.sort_by_key(|e| e.start);

        Self { entries }
    }
    /// Find the allocation ID that contains the given pointer.
    ///
    /// Returns `Some(alloc_id)` if `ptr` falls within `[start, end)` of
    /// some allocation, or `None` if no allocation contains it.
    ///
    /// # Complexity
    ///
    /// O(log n) via binary search.
    pub fn find_containing(&self, ptr: usize) -> Option<usize> {
        // partition_point returns the index of the first entry where
        // `entry.start > ptr` is false, i.e., the first entry with start > ptr.
        // The candidate is the entry just before it (last entry with start <= ptr).
        let idx = self.entries.partition_point(|e| e.start <= ptr);

        if idx > 0 {
            let candidate = &self.entries[idx - 1];
            if ptr < candidate.end {
                return Some(candidate.alloc_id);
            }
        }
        None
    }

    /// Find the allocation ID where the pointer equals the start address.
    ///
    /// Returns `Some(alloc_id)` if `ptr == alloc.start` for some allocation.
    pub fn find_exact_start(&self, ptr: usize) -> Option<usize> {
        let idx = self.entries.partition_point(|e| e.start < ptr);
        if idx < self.entries.len() && self.entries[idx].start == ptr {
            Some(self.entries[idx].alloc_id)
        } else {
            None
        }
    }

    /// Get the number of indexed allocations.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the range map is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::TrackKind::HeapOwner;

    fn make_alloc(ptr: usize, size: usize) -> ActiveAllocation {
        ActiveAllocation {
            kind: HeapOwner { ptr, size },
            ptr: Some(ptr),
            size,
            allocated_at: 0,
            var_name: None,
            type_name: None,
            thread_id: 0,
            call_stack_hash: None,
            module_path: None,
            stack_ptr: None,
        }
    }

    #[test]
    fn test_rangemap_empty() {
        let map = RangeMap::new(&[]);
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert!(map.find_containing(0x1000).is_none());
    }

    #[test]
    fn test_rangemap_single_allocation() {
        let allocs = vec![make_alloc(0x1000, 100)];
        let map = RangeMap::new(&allocs);

        assert_eq!(map.find_containing(0x1000), Some(0));
        assert_eq!(map.find_containing(0x1050), Some(0));
        assert_eq!(map.find_containing(0x1063), Some(0)); // last valid byte
        assert_eq!(map.find_containing(0x1064), None); // out of range
        assert_eq!(map.find_containing(0x0FFF), None); // before start
    }

    #[test]
    fn test_rangemap_multiple_allocations() {
        // Three non-overlapping allocations with gaps between them.
        // alloc[0]: [0x1000, 0x1064) size=100=0x64
        // alloc[1]: [0x2000, 0x20C8) size=200=0xC8
        // alloc[2]: [0x3000, 0x3032) size=50=0x32
        let allocs = vec![
            make_alloc(0x1000, 100),
            make_alloc(0x2000, 200),
            make_alloc(0x3000, 50),
        ];
        let map = RangeMap::new(&allocs);

        // Boundary tests: first byte, middle byte, last byte of each allocation.
        assert_eq!(map.find_containing(0x1000), Some(0)); // first byte of alloc[0]
        assert_eq!(map.find_containing(0x1050), Some(0)); // middle of alloc[0]
        assert_eq!(map.find_containing(0x1063), Some(0)); // last valid byte of alloc[0]
        assert_eq!(map.find_containing(0x1064), None); // just past alloc[0]

        assert_eq!(map.find_containing(0x2000), Some(1)); // first byte of alloc[1]
        assert_eq!(map.find_containing(0x2050), Some(1)); // middle of alloc[1]
        assert_eq!(map.find_containing(0x20C7), Some(1)); // last valid byte of alloc[1]
        assert_eq!(map.find_containing(0x20C8), None); // just past alloc[1]

        assert_eq!(map.find_containing(0x3000), Some(2)); // first byte of alloc[2]
        assert_eq!(map.find_containing(0x3031), Some(2)); // last valid byte of alloc[2]
        assert_eq!(map.find_containing(0x3032), None); // just past alloc[2]

        // Gap addresses should return None.
        assert_eq!(map.find_containing(0x1500), None); // gap between alloc[0] and alloc[1]
        assert_eq!(map.find_containing(0x2500), None); // gap between alloc[1] and alloc[2]
    }

    #[test]
    fn test_rangemap_adjacent_allocations() {
        let allocs = vec![make_alloc(0x1000, 100), make_alloc(0x1064, 100)];
        let map = RangeMap::new(&allocs);

        assert_eq!(map.find_containing(0x1063), Some(0));
        assert_eq!(map.find_containing(0x1064), Some(1));
    }

    #[test]
    fn test_rangemap_find_exact_start() {
        let allocs = vec![make_alloc(0x1000, 100), make_alloc(0x2000, 200)];
        let map = RangeMap::new(&allocs);

        assert_eq!(map.find_exact_start(0x1000), Some(0));
        assert_eq!(map.find_exact_start(0x2000), Some(1));
        assert_eq!(map.find_exact_start(0x1050), None); // inside but not start
        assert_eq!(map.find_exact_start(0x3000), None); // not present
    }

    #[test]
    fn test_rangemap_unsorted_input() {
        let allocs = vec![
            make_alloc(0x3000, 10),
            make_alloc(0x1000, 10),
            make_alloc(0x2000, 10),
        ];
        let map = RangeMap::new(&allocs);

        // Should still find the right allocation IDs.
        assert_eq!(map.find_containing(0x1000), Some(1));
        assert_eq!(map.find_containing(0x2000), Some(2));
        assert_eq!(map.find_containing(0x3000), Some(0));
    }

    #[test]
    fn test_rangemap_zero_size_allocation() {
        let allocs = vec![make_alloc(0x1000, 0)];
        let map = RangeMap::new(&allocs);

        // Zero-size allocation: start == end, nothing is contained.
        assert_eq!(map.find_containing(0x1000), None);
    }

    #[test]
    fn test_rangemap_large_number_of_allocations() {
        let allocs: Vec<ActiveAllocation> = (0..1000)
            .map(|i| make_alloc(0x10000 + i * 0x1000, 0x100))
            .collect();
        let map = RangeMap::new(&allocs);

        // Spot check.
        assert_eq!(map.find_containing(0x10000), Some(0));
        assert_eq!(map.find_containing(0x11000), Some(1));
        assert_eq!(map.find_containing(0x1F000), Some(15));
    }
}
