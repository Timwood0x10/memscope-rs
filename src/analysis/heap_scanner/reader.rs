//! Safe heap memory reader with page-wise validation.
//!
//! Provides the `HeapScanner` which reads allocation memory content
//! while preventing segfaults through ValidRegions checks and
//! atomic system calls for fault tolerance.

use crate::analysis::is_virtual_pointer;
use crate::analysis::unsafe_inference::is_valid_ptr;
use crate::snapshot::types::ActiveAllocation;

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
    /// Also skips virtual pointers (>= 0x10000000000) used for Container types.
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
/// On Linux, uses `process_vm_readv` which is an atomic syscall that cannot
/// be interrupted by signals mid-read, eliminating TOCTOU issues.
///
/// On other platforms, falls back to volatile byte-by-byte reads with
/// pre-validation of all pages before reading begins.
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

    let mut buf = vec![0u8; read_size];

    #[cfg(target_os = "linux")]
    {
        if safe_read_linux(ptr, &mut buf) {
            Some(buf)
        } else {
            None
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        if read_bytes_volatile(ptr, &mut buf) {
            Some(buf)
        } else {
            None
        }
    }
}

#[cfg(target_os = "linux")]
mod linux_read {
    use libc::{iovec, process_vm_readv};

    /// Read memory from the current process using process_vm_readv.
    ///
    /// This uses pid=0 which refers to the calling process itself.
    /// According to Linux man page: "The caller must have the CAP_SYS_PTRACE
    /// capability, OR the real, effective, and saved-set user ID of the caller
    /// must match the real user ID of the target process."
    ///
    /// For pid=0 (current process), the user IDs always match, so no special
    /// privileges are required. This works in most environments including
    /// containers (unless seccomp filters explicitly block process_vm_readv).
    ///
    /// The function is named `_local` to clarify it's for self-reading,
    /// not for reading remote processes.
    pub fn safe_read_linux_local(
        remote_ptr: *const libc::c_void,
        local_ptr: *mut libc::c_void,
        len: usize,
    ) -> isize {
        let local_iov = iovec {
            iov_base: local_ptr,
            iov_len: len,
        };
        let remote_iov = iovec {
            iov_base: remote_ptr as *mut libc::c_void,
            iov_len: len,
        };

        // pid=0 means the calling process reads its own memory
        // No CAP_SYS_PTRACE required for self-reading
        unsafe { process_vm_readv(0, &local_iov, 1, &remote_iov, 1, 0) }
    }
}

#[cfg(target_os = "linux")]
fn safe_read_linux(ptr: usize, buf: &mut [u8]) -> bool {
    use linux_read::safe_read_linux_local;

    let len = buf.len();

    let result = safe_read_linux_local(
        ptr as *const libc::c_void,
        buf.as_mut_ptr() as *mut libc::c_void,
        len,
    );

    result == len as isize
}

#[cfg(not(target_os = "linux"))]
#[allow(dead_code)] // Stub for non-Linux platforms; used when building on macOS/Windows
fn safe_read_linux(_ptr: usize, _buf: &mut [u8]) -> bool {
    false
}

#[cfg(not(target_os = "linux"))]
fn read_bytes_volatile(ptr: usize, buf: &mut [u8]) -> bool {
    // Pre-check: verify the entire range is valid before reading
    if !are_pages_valid(ptr, buf.len()) {
        return false;
    }

    // Use a safer approach: try-catch with signal handling would be ideal,
    // but Rust doesn't have that. Instead, we rely on pre-validation.
    // On macOS, direct volatile reads should work if pages are valid.
    unsafe {
        let src = ptr as *const u8;
        for (i, byte) in buf.iter_mut().enumerate() {
            *byte = std::ptr::read_volatile(src.add(i));
        }
    }
    true
}

/// Check that every page in [ptr, ptr + size) is in a valid region.
fn are_pages_valid(ptr: usize, size: usize) -> bool {
    let page_start = ptr & !(PAGE_SIZE - 1);
    let page_end = (ptr + size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

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
