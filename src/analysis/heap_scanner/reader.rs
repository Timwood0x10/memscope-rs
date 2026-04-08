//! Safe heap memory reader with page-wise validation.
//!
//! Provides the `HeapScanner` which reads allocation memory content
//! while preventing segfaults through ValidRegions checks and
//! atomic system calls for fault tolerance.

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
    /// Returns a `ScanResult` for each allocation. Allocations whose pointers
    /// fall outside valid regions will have `memory: None`.
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
        allocations
            .iter()
            .map(|alloc| {
                let memory = safe_read_memory(alloc.ptr, alloc.size);
                ScanResult {
                    ptr: alloc.ptr,
                    size: alloc.size,
                    memory,
                }
            })
            .collect()
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
    let mut addr = ptr;
    for byte in buf.iter_mut() {
        if !is_valid_ptr(addr) {
            return false;
        }
        *byte = unsafe { std::ptr::read_volatile(addr as *const u8) };
        addr = addr.saturating_add(1);
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

    #[test]
    fn test_safe_read_memory_zero_size() {
        assert!(safe_read_memory(0x10000, 0).is_none());
    }

    #[test]
    fn test_safe_read_memory_null_ptr() {
        assert!(safe_read_memory(0, 100).is_none());
    }

    #[test]
    fn test_are_pages_valid_single_page() {
        assert!(are_pages_valid(0x10000, 100));
    }

    #[test]
    fn test_are_pages_valid_cross_page() {
        let ptr = 0x10000 - 100;
        let size = 200;
        // 0xFF00..0x10100 spans two pages, both should be mapped.
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
    fn test_heap_scanner_scan_real_allocations() {
        // Scan actual heap allocations to verify end-to-end behavior.
        let alloc1 = [42u8; 64];
        let alloc2 = [99u8; 128];
        let allocations = vec![
            ActiveAllocation {
                ptr: alloc1.as_ptr() as usize,
                size: alloc1.len(),
                allocated_at: 1000,
                var_name: None,
                type_name: None,
                thread_id: 0,
                call_stack_hash: None,
            },
            ActiveAllocation {
                ptr: alloc2.as_ptr() as usize,
                size: alloc2.len(),
                allocated_at: 2000,
                var_name: None,
                type_name: None,
                thread_id: 0,
                call_stack_hash: None,
            },
        ];

        let results = HeapScanner::scan(&allocations);
        assert_eq!(results.len(), 2);

        // Both allocations should have readable memory.
        assert!(results[0].memory.is_some());
        assert!(results[1].memory.is_some());

        // Verify memory content matches original.
        let mem0 = results[0].memory.as_ref().unwrap();
        let read_size = 64.min(mem0.len());
        assert_eq!(mem0.as_slice()[..read_size], alloc1[..read_size]);
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
            ptr: 0x10000,
            size: 0,
            allocated_at: 1000,
            var_name: None,
            type_name: None,
            thread_id: 0,
            call_stack_hash: None,
        }];

        let results = HeapScanner::scan(&allocations);
        assert_eq!(results.len(), 1);
        // Zero-size allocation should return None for memory (nothing to read).
        assert!(results[0].memory.is_none());
    }

    #[test]
    fn test_heap_scanner_content_preserved_after_scan() {
        // Verify that scanning a Vec with known pattern preserves the content.
        let data = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let alloc = ActiveAllocation {
            ptr: data.as_ptr() as usize,
            size: data.len(),
            allocated_at: 1000,
            var_name: None,
            type_name: None,
            thread_id: 0,
            call_stack_hash: None,
        };

        let results = HeapScanner::scan(&[alloc]);
        assert_eq!(results.len(), 1);

        let mem = results[0].memory.as_ref().unwrap();
        assert_eq!(mem.as_slice()[..8], data[..]);
    }
}
