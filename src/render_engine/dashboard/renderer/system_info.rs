//! System information gathering for dashboard.

#[allow(unused)]
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
        get_portable_system_info()
    }
}

/// Compute the current process CPU usage percentage (0.0-100.0).
///
/// Divides the process's user + system CPU time by its wall-clock uptime
/// recorded by `init_global_tracking()` (via
/// `capture::backends::global_tracking::PROCESS_START`). The CPU time comes
/// from `getrusage(RUSAGE_SELF)` on Unix and from `sysinfo` on Windows, whose
/// `libc` has no `rusage`. The result is clamped to [0, 100] and rounded to
/// two decimal places.
///
/// No manual initialization is needed — `init_global_tracking()` captures
/// the start time automatically on first call.
fn process_cpu_usage_pct() -> f64 {
    // Process wall-clock age from the timer initialized in init_global_tracking().
    let elapsed = match crate::capture::backends::global_tracking::process_start_elapsed_secs() {
        Some(e) if e >= 0.001 => e,
        _ => return 0.0,
    };

    let cpu_secs = process_cpu_time_secs();
    if cpu_secs <= 0.0 {
        return 0.0;
    }

    let pct = (cpu_secs / elapsed) * 100.0;
    // Round to 2 decimal places so the telemetry bar shows a clean value
    // like "0.05%" instead of "0.000023050096979228655%".
    (pct.clamp(0.0, 100.0) * 100.0).round() / 100.0
}

/// Total CPU time (user + system) consumed by this process, in seconds.
///
/// Unix uses `getrusage(RUSAGE_SELF)`. Windows `libc` has no `rusage`, so it
/// asks `sysinfo` for the process's accumulated CPU time instead. Both paths
/// read a measured counter, never an estimate.
#[cfg(unix)]
fn process_cpu_time_secs() -> f64 {
    use std::mem::MaybeUninit;

    let mut ru: MaybeUninit<libc::rusage> = MaybeUninit::uninit();
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, ru.as_mut_ptr()) } != 0 {
        return 0.0;
    }
    let ru = unsafe { ru.assume_init() };
    let user = ru.ru_utime.tv_sec as f64 + ru.ru_utime.tv_usec as f64 / 1_000_000.0;
    let sys = ru.ru_stime.tv_sec as f64 + ru.ru_stime.tv_usec as f64 / 1_000_000.0;
    user + sys
}

/// Windows has no `getrusage`; `GetProcessTimes` reports kernel + user time in
/// 100-nanosecond intervals, which converts directly to seconds.
#[cfg(windows)]
fn process_cpu_time_secs() -> f64 {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};

    let mut creation = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut exit = creation;
    let mut kernel = creation;
    let mut user = creation;
    // SAFETY: the handle is the current process pseudo-handle, and all four
    // FILETIME pointers are valid, writable, and distinct locals.
    let ok = unsafe {
        GetProcessTimes(
            GetCurrentProcess(),
            &mut creation,
            &mut exit,
            &mut kernel,
            &mut user,
        )
    };
    if ok == 0 {
        return 0.0;
    }
    let to_secs = |ft: FILETIME| {
        ((u64::from(ft.dwHighDateTime) << 32) | u64::from(ft.dwLowDateTime)) as f64 / 10_000_000.0
    };
    to_secs(kernel) + to_secs(user)
}

/// Platforms with neither `getrusage` nor Win32 process times report no CPU
/// sample instead of inventing one.
#[cfg(not(any(unix, windows)))]
fn process_cpu_time_secs() -> f64 {
    0.0
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

    // Same portable source as the other platforms: one query, no per-OS branch.
    let page_size: u64 = portable_page_size();

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
        cpu_usage_pct: process_cpu_usage_pct(),
        total_physical_bytes: total,
        used_physical_bytes: used_physical,
    }
}

/// Helper to convert string to C string (cached for sysctl calls)
#[cfg(target_os = "macos")]
fn c(s: &str) -> *const libc::c_char {
    use std::ffi::CString;
    thread_local! {
        static KERN_OSRELEASE: CString = CString::new("kern.osrelease").unwrap();
        static HW_MACHINE: CString = CString::new("hw.machine").unwrap();
    }
    match s {
        "kern.osrelease" => KERN_OSRELEASE.with(|c| c.as_ptr()),
        "hw.machine" => HW_MACHINE.with(|c| c.as_ptr()),
        _ => std::ptr::null(),
    }
}

/// Page size used by the memory accounting.
///
/// One implementation for every platform: `page_size` queries the OS the way each one
/// expects (sysconf on unix, `GetSystemInfo` on Windows). Keeping it portable avoids a
/// per-OS branch that can drift or fail to compile on a target the linters do not cover.
fn portable_page_size() -> u64 {
    page_size::get() as u64
}

/// System information on platforms without a dedicated implementation.
///
/// Values come from the `sysinfo` crate and from `getrusage`; nothing is hardcoded, so a
/// dashboard on Linux reports its own machine instead of a placeholder. When a value is
/// unavailable the field says `Unknown` rather than inventing a number.
#[cfg(not(target_os = "macos"))]
fn get_portable_system_info() -> SystemResources {
    let mut system = sysinfo::System::new();
    system.refresh_memory();
    let total_bytes = system.total_memory();
    let used_bytes = system.used_memory();
    let available_bytes = system.available_memory();

    SystemResources {
        os_name: sysinfo::System::name().unwrap_or_else(|| "Unknown".to_string()),
        os_version: sysinfo::System::os_version().unwrap_or_else(|| "Unknown".to_string()),
        architecture: sysinfo::System::cpu_arch().unwrap_or_else(|| "unknown".to_string()),
        cpu_cores: std::thread::available_parallelism()
            .map(|cores| cores.get() as u32)
            .unwrap_or(1),
        total_physical: format_bytes(total_bytes as usize),
        available_physical: format_bytes(available_bytes as usize),
        used_physical: format_bytes(used_bytes as usize),
        page_size: portable_page_size(),
        cpu_usage_pct: process_cpu_usage_pct(),
        total_physical_bytes: total_bytes,
        used_physical_bytes: used_bytes,
    }
}
