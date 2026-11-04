//! Fault-tolerant byte copying. Source addresses are OS addresses, never Rust references.
//! A successful read is a best-effort observation, not proof of allocation identity.

pub(super) fn read(address: usize, output: &mut [u8]) -> bool {
    if address == 0 || output.is_empty() || address.checked_add(output.len()).is_none() {
        return false;
    }
    platform_read(address, output)
}

#[cfg(target_os = "macos")]
fn platform_read(address: usize, output: &mut [u8]) -> bool {
    let mut copied = 0;
    // SAFETY: Mach validates the source address in the current task. The destination
    // is exclusively borrowed, initialized, and writable for exactly output.len().
    // No reference to remote memory is constructed; invalid sources return an error.
    let result = unsafe {
        mach2::vm::mach_vm_read_overwrite(
            mach2::traps::mach_task_self(),
            address as u64,
            output.len() as u64,
            output.as_mut_ptr() as u64,
            &mut copied,
        )
    };
    result == mach2::kern_return::KERN_SUCCESS && copied == output.len() as u64
}

#[cfg(target_os = "linux")]
fn platform_read(address: usize, output: &mut [u8]) -> bool {
    let local = libc::iovec {
        iov_base: output.as_mut_ptr().cast(),
        iov_len: output.len(),
    };
    let remote = libc::iovec {
        iov_base: address as *mut libc::c_void,
        iov_len: output.len(),
    };
    // SAFETY: Both iovec descriptors live through the call; the local buffer is
    // exclusively writable for its advertised length. The kernel validates remote
    // addresses. getpid(), not pid=0, selects the current process for this syscall.
    let copied = unsafe { libc::process_vm_readv(libc::getpid(), &local, 1, &remote, 1, 0) };
    copied >= 0 && copied as usize == output.len()
}

#[cfg(windows)]
fn platform_read(address: usize, output: &mut [u8]) -> bool {
    let mut copied = 0;
    // SAFETY: -1 is the current-process pseudo handle. The kernel validates the
    // remote range; only the exclusively borrowed output range may be written.
    let result = unsafe {
        windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory(
            -1isize as _,
            address as _,
            output.as_mut_ptr().cast(),
            output.len(),
            &mut copied,
        )
    };
    result != 0 && copied == output.len()
}

#[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
fn platform_read(_address: usize, _output: &mut [u8]) -> bool {
    false
}

#[cfg(all(test, not(miri), any(target_os = "macos", target_os = "linux")))]
mod tests {
    use super::*;

    /// Objective: Stale heap bytes remain safely owned through scan cloning/drop.
    /// Invariants: Captured bytes, including free-list metadata, are not modified.
    #[test]
    fn stale_small_blocks_can_be_copied_and_dropped() {
        use crate::analysis::unsafe_inference::{scan_buffer, OwnedMemoryView};
        let _guard = crate::core::allocator::TrackingGuard::acquire();
        for len in 1..=256 {
            let allocation = vec![42u8; len];
            let address = allocation.as_ptr() as usize;
            drop(allocation);
            let mut bytes = scan_buffer(len);
            bytes.resize(len, 0);
            if read(address, &mut bytes) {
                let copy = OwnedMemoryView::copy_scan(&bytes);
                assert_eq!(
                    copy.as_slice(),
                    bytes,
                    "scan copies must preserve all observed bytes"
                );
            }
        }
    }

    /// Objective: Concurrent scans remain byte copies even after source unmapping.
    /// Invariants: 50 workers read live data, reject unmapped/protected ranges, and
    /// never write beyond the destination. No source alignment is required.
    #[test]
    fn fifty_threads_read_then_unmap() {
        std::thread::scope(|scope| {
            for _ in 0..50 {
                scope.spawn(|| {
                    // SAFETY: Each worker owns a fresh anonymous mapping; all writes
                    // precede reads and unmapping, and its pointer is not shared.
                    unsafe {
                        let page = libc::sysconf(libc::_SC_PAGESIZE) as usize;
                        let ptr = libc::mmap(
                            std::ptr::null_mut(),
                            page * 2,
                            libc::PROT_READ | libc::PROT_WRITE,
                            libc::MAP_PRIVATE | libc::MAP_ANON,
                            -1,
                            0,
                        );
                        assert_ne!(ptr, libc::MAP_FAILED, "test mapping must succeed");
                        std::ptr::write_bytes(ptr, 42, page);
                        let mut output = [0u8; 34];
                        assert!(
                            read(ptr as usize + 1, &mut output[1..33]),
                            "unaligned mapped source is readable"
                        );
                        assert_eq!(output[1..33], [42; 32], "copy must preserve bytes");
                        assert_eq!(
                            (output[0], output[33]),
                            (0, 0),
                            "destination canaries must remain intact"
                        );
                        assert_eq!(
                            libc::mprotect(
                                (ptr as *mut u8).add(page).cast(),
                                page,
                                libc::PROT_NONE
                            ),
                            0,
                            "protect second page"
                        );
                        assert!(
                            !read(ptr as usize + page - 16, &mut output),
                            "partial reads are rejected"
                        );
                        assert_eq!(libc::munmap(ptr, page * 2), 0, "unmap owned pages");
                        // Another worker may immediately map this address again.
                        // Either outcome is valid; the stale read must never fault.
                        let _ = read(ptr as usize, &mut output);
                    }
                });
            }
        });
    }
}

#[cfg(test)]
mod bounds_tests {
    use super::*;

    #[test]
    fn invalid_ranges_never_reach_the_os() {
        let mut output = [1u8; 16];
        assert!(!read(0, &mut output), "null is not a readable source");
        assert!(!read(usize::MAX, &mut output), "overflow must be rejected");
        assert!(!read(1, &mut []), "empty reads have no evidence");
        assert_eq!(output, [1; 16], "invalid input must not modify output");
    }
}
