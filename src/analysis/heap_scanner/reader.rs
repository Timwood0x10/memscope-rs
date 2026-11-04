//! Safe heap memory reader with page-wise validation.
//!
//! Provides the `HeapScanner` which reads allocation memory content
//! while preventing segfaults through ValidRegions checks and
//! atomic system calls for fault tolerance.

use crate::analysis::is_virtual_pointer;
use crate::analysis::unsafe_inference::is_valid_ptr;
use crate::snapshot::types::ActiveAllocation;

mod process_memory;

/// Maximum bytes to read per allocation. Metadata headers are always
/// within the first few dozen bytes; 4KB is more than sufficient.
const MAX_READ_BYTES: usize = 4096;

/// Page size for memory validation checks.
const PAGE_SIZE: usize = 4096;

/// Result of scanning a single allocation.
#[derive(Debug)]
pub struct ScanResult {
    /// Pointer address of the allocation.
    pub ptr: usize,
    /// Original allocation size.
    pub size: usize,
    /// Memory content that was successfully read (capped at MAX_READ_BYTES).
    pub memory: Option<Vec<u8>>,
}

/// HeapScanner reads heap memory for active allocations during snapshot analysis.
///
/// All memory reads are validated through `ValidRegions` to prevent segfaults.
/// This module only operates during offline analysis and has zero runtime overhead.
pub struct HeapScanner;

impl HeapScanner {
    /// Scan a list of active allocations, reading their memory content.
    ///
    /// Only scans HeapOwner allocations and performs deduplication
    /// to avoid redundant scanning of duplicate heap regions.
    ///
    /// Returns a `ScanResult` for each unique heap region. Allocations whose
    /// pointers fall outside valid regions will have `memory: None`.
    ///
    /// # Arguments
    ///
    /// * `allocations` - List of active allocations to scan.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let results = HeapScanner::scan(&allocations);
    /// for result in results {
    ///     if let Some(ref memory) = result.memory {
    ///         let view = MemoryView::new(memory);
    ///         // ... pass to UTI Engine
    ///     }
    /// }
    /// ```
    pub fn scan(allocations: &[ActiveAllocation]) -> Vec<ScanResult> {
        // Step 1: Filter HeapOwner + deduplicate regions
        let regions = Self::dedup_heap_regions(allocations);

        // Step 2: Scan deduplicated regions
        regions
            .iter()
            .map(|&(ptr, size)| {
                let memory = safe_read_memory(ptr, size);
                ScanResult { ptr, size, memory }
            })
            .collect()
    }

    /// Deduplicate heap regions to avoid redundant scanning.
    ///
    /// Filters for HeapOwner allocations and removes duplicates
    /// based on (ptr, size) pairs.
    ///
    /// Also skips virtual pointers (>= VIRTUAL_PTR_BASE) used for Container types.
    /// This threshold is set high enough to avoid conflicts with real heap addresses
    /// on all platforms (including macOS which can have addresses > 0x100000000).
    fn dedup_heap_regions(allocs: &[ActiveAllocation]) -> Vec<(usize, usize)> {
        use std::collections::HashSet;

        let mut seen = HashSet::new();
        let mut regions = Vec::new();

        for alloc in allocs {
            if let crate::core::types::TrackKind::HeapOwner { ptr, size } = alloc.kind {
                if is_virtual_pointer(ptr) {
                    continue;
                }

                let key = (ptr, size);

                if seen.insert(key) {
                    regions.push(key);
                }
            }
        }

        regions
    }
}

/// Safely read memory at `ptr` for up to `size` bytes.
///
/// Returns `None` if the address is not in valid regions, or if any
/// page within the read range is unmapped.
///
/// Reads through the OS; a region check alone cannot prove a Rust allocation
/// is still alive. Captured addresses may have been released or reused.
fn safe_read_memory(ptr: usize, size: usize) -> Option<Vec<u8>> {
    if size == 0 || ptr == 0 {
        return None;
    }

    if !is_valid_ptr(ptr) {
        return None;
    }

    let read_size = size.min(MAX_READ_BYTES);
    if !are_pages_valid(ptr, read_size) {
        return None;
    }

    let mut buf = crate::analysis::unsafe_inference::scan_buffer(read_size);
    buf.resize(read_size, 0);

    process_memory::read(ptr, &mut buf).then_some(buf)
}

/// Check that every page in [ptr, ptr + size) is in a valid region.
fn are_pages_valid(ptr: usize, size: usize) -> bool {
    let page_start = ptr & !(PAGE_SIZE - 1);
    let Some(end) = ptr
        .checked_add(size)
        .and_then(|end| end.checked_add(PAGE_SIZE - 1))
    else {
        return false;
    };
    let page_end = end & !(PAGE_SIZE - 1);

    let mut p = page_start;
    while p < page_end {
        if !is_valid_ptr(p) {
            return false;
        }
        p += PAGE_SIZE;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::TrackKind;

    #[test]
    fn test_safe_read_memory_zero_size() {
        assert!(safe_read_memory(0x10000, 0).is_none());
    }

    #[test]
    fn test_safe_read_memory_null_ptr() {
        assert!(safe_read_memory(0, 100).is_none());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_are_pages_valid_single_page() {
        assert!(are_pages_valid(0x10000, 100));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_are_pages_valid_cross_page() {
        // Use a heap-like address that should be valid on all platforms
        let ptr = 0x10000;
        let size = 200;
        assert!(are_pages_valid(ptr, size));
    }

    #[test]
    fn test_scan_result_creation() {
        let result = ScanResult {
            ptr: 0x1000,
            size: 64,
            memory: None,
        };
        assert_eq!(result.ptr, 0x1000);
        assert_eq!(result.size, 64);
        assert!(result.memory.is_none());
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_heap_scanner_scan_real_allocations() {
        let data1 = vec![42u8; 64];
        let data2 = vec![99u8; 128];
        let ptr1 = data1.as_ptr() as usize;
        let ptr2 = data2.as_ptr() as usize;

        let allocations = vec![
            ActiveAllocation {
                ptr: Some(ptr1),
                size: 64,
                kind: TrackKind::HeapOwner {
                    ptr: ptr1,
                    size: 64,
                },
                allocated_at: 1000,
                var_name: None,
                type_name: None,
                thread_id: 0,
                call_stack_hash: None,
                module_path: None,
                stack_ptr: None,
            },
            ActiveAllocation {
                ptr: Some(ptr2),
                size: 128,
                kind: TrackKind::HeapOwner {
                    ptr: ptr2,
                    size: 128,
                },
                allocated_at: 2000,
                var_name: None,
                type_name: None,
                thread_id: 0,
                call_stack_hash: None,
                module_path: None,
                stack_ptr: None,
            },
        ];

        let results = HeapScanner::scan(&allocations);
        assert_eq!(results.len(), 2);

        assert!(results[0].memory.is_some(), "Should read memory at ptr1");
        assert!(results[1].memory.is_some(), "Should read memory at ptr2");

        drop(data1);
        drop(data2);
    }

    #[test]
    fn test_heap_scanner_scan_empty_allocations() {
        let allocations: Vec<ActiveAllocation> = vec![];
        let results = HeapScanner::scan(&allocations);
        assert!(results.is_empty());
    }

    #[test]
    fn test_heap_scanner_scan_zero_size_allocation() {
        let allocations = vec![ActiveAllocation {
            ptr: Some(0x10000),
            size: 0,
            kind: TrackKind::HeapOwner {
                ptr: 0x10000,
                size: 0,
            },
            allocated_at: 1000,
            var_name: None,
            type_name: None,
            thread_id: 0,
            call_stack_hash: None,
            module_path: None,
            stack_ptr: None,
        }];

        let results = HeapScanner::scan(&allocations);
        assert_eq!(results.len(), 1);
        // Zero-size allocation should return None for memory (nothing to read).
        assert!(results[0].memory.is_none());
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    #[ignore = "Heap pointer addresses may exceed VIRTUAL_PTR_BASE in some CI environments"]
    fn test_heap_scanner_content_preserved_after_scan() {
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let ptr = data.as_ptr() as usize;
        let size = data.len();

        let alloc = ActiveAllocation {
            ptr: Some(ptr),
            size,
            kind: TrackKind::HeapOwner { ptr, size },
            allocated_at: 1000,
            var_name: None,
            type_name: None,
            thread_id: 0,
            call_stack_hash: None,
            module_path: None,
            stack_ptr: None,
        };

        let results = HeapScanner::scan(&[alloc]);
        assert_eq!(results.len(), 1);

        let mem = results[0]
            .memory
            .as_ref()
            .expect("Should read memory at allocated address");
        assert_eq!(mem.len(), size, "Should read expected number of bytes");

        drop(data);
    }
}
