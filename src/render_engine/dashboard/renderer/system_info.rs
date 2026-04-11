//! System information gathering for dashboard.

use super::helpers::format_bytes;
use super::types::SystemResources;

/// Get system information
pub fn get_system_info() -> SystemResources {
    #[cfg(target_os = "macos")]
    {
        get_macos_system_info()
    }
    #[cfg(not(target_os = "macos"))]
    {
        SystemResources {
            os_name: "Unknown".to_string(),
            os_version: "Unknown".to_string(),
            architecture: "unknown".to_string(),
            cpu_cores: 1,
            total_physical: "16.00 GB".to_string(),
            available_physical: "8.00 GB".to_string(),
            used_physical: "8.00 GB".to_string(),
            page_size: 4096,
        }
    }
}

#[cfg(target_os = "macos")]
fn get_macos_system_info() -> SystemResources {
    let os_version = unsafe {
        let mut size: libc::size_t = 256;
        let mut buf = [0u8; 256];
        if libc::sysctlbyname(
            c("kern.osrelease"),
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        ) == 0
        {
            String::from_utf8_lossy(&buf[..size - 1]).to_string()
        } else {
            "Unknown".to_string()
        }
    };

    let architecture = unsafe {
        let mut size: libc::size_t = 256;
        let mut buf = [0u8; 256];
        if libc::sysctlbyname(
            c("hw.machine"),
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        ) == 0
        {
            let arch_str = String::from_utf8_lossy(&buf[..size - 1]).to_string();
            if arch_str.contains("arm64") || arch_str.contains("arm") {
                "arm64".to_string()
            } else {
                arch_str
            }
        } else {
            "unknown".to_string()
        }
    };

    let mut cpu_cores: u32 = 1;
    unsafe {
        let mut size = std::mem::size_of::<u32>();
        let mut mib: [libc::c_int; 2] = [libc::CTL_HW, libc::HW_NCPU];
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as libc::c_uint,
            &mut cpu_cores as *mut u32 as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        );
    }

    let mut page_size: u64 = 4096;
    unsafe {
        let mut size = std::mem::size_of::<u64>();
        if libc::sysctlbyname(
            c("hw.pagesize"),
            &mut page_size as *mut u64 as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        ) != 0
        {
            page_size = 4096;
        }
    }

    let mut total: u64 = 0;
    unsafe {
        let mut size = std::mem::size_of::<u64>();
        let mut mib: [libc::c_int; 2] = [libc::CTL_HW, libc::HW_MEMSIZE];
        if libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as libc::c_uint,
            &mut total as *mut u64 as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        ) != 0
        {
            total = 16 * 1024 * 1024 * 1024;
        }
    }

    let (available_physical, used_physical) = unsafe {
        let mut vm_stats: libc::vm_statistics64 = std::mem::zeroed();
        let mut count = libc::HOST_VM_INFO64_COUNT;
        if libc::host_statistics64(
            mach2::mach_init::mach_host_self(),
            libc::HOST_VM_INFO64,
            &mut vm_stats as *mut _ as libc::host_info64_t,
            &mut count,
        ) != 0
        {
            (total / 2, total / 2)
        } else {
            let free = vm_stats.free_count as u64 * page_size;
            let active = vm_stats.active_count as u64 * page_size;
            let inactive = vm_stats.inactive_count as u64 * page_size;
            let wired = vm_stats.wire_count as u64 * page_size;
            let used = active + wired;
            let available = free + inactive;
            (available, used)
        }
    };

    SystemResources {
        os_name: "macOS".to_string(),
        os_version,
        architecture,
        cpu_cores,
        total_physical: format_bytes(total as usize),
        available_physical: format_bytes(available_physical as usize),
        used_physical: format_bytes(used_physical as usize),
        page_size,
    }
}

/// Helper to convert string to C string (cached for sysctl calls)
#[cfg(target_os = "macos")]
fn c(s: &str) -> *const libc::c_char {
    use std::ffi::CString;
    thread_local! {
        static KERN_OSRELEASE: CString = CString::new("kern.osrelease").unwrap();
        static HW_MACHINE: CString = CString::new("hw.machine").unwrap();
        static HW_PAGESIZE: CString = CString::new("hw.pagesize").unwrap();
    }
    match s {
        "kern.osrelease" => KERN_OSRELEASE.with(|c| c.as_ptr()),
        "hw.machine" => HW_MACHINE.with(|c| c.as_ptr()),
        "hw.pagesize" => HW_PAGESIZE.with(|c| c.as_ptr()),
        _ => std::ptr::null(),
    }
}
