use std::time::{Duration, Instant};

/// Platform-specific memory information collector
pub struct PlatformMemoryInfo {
    /// Last collected statistics
    last_stats: Option<MemoryStats>,
    /// Collection interval
    collection_interval: Duration,
    /// Last collection time
    last_collection: Option<Instant>,
    /// Platform-specific context
    platform_context: MemoryContext,
}

/// Comprehensive memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Virtual memory statistics
    pub virtual_memory: VirtualMemoryStats,
    /// Physical memory statistics
    pub physical_memory: PhysicalMemoryStats,
    /// Process-specific memory statistics
    pub process_memory: ProcessMemoryStats,
    /// System-wide memory statistics
    pub system_memory: SystemMemoryStats,
    /// Memory pressure indicators
    pub pressure_indicators: PressureIndicators,
    /// Collection timestamp
    pub timestamp: Instant,
}

impl Default for MemoryStats {
    fn default() -> Self {
        MemoryStats {
            virtual_memory: VirtualMemoryStats::default(),
            physical_memory: PhysicalMemoryStats::default(),
            process_memory: ProcessMemoryStats::default(),
            system_memory: SystemMemoryStats::default(),
            pressure_indicators: PressureIndicators::default(),
            timestamp: Instant::now(),
        }
    }
}

/// Virtual memory statistics
#[derive(Debug, Clone, Default)]
pub struct VirtualMemoryStats {
    /// Total virtual address space
    pub total_virtual: u64,
    /// Available virtual address space
    pub available_virtual: u64,
    /// Used virtual address space
    pub used_virtual: u64,
    /// Reserved but uncommitted memory
    pub reserved: u64,
    /// Committed memory
    pub committed: u64,
}

/// Physical memory statistics
#[derive(Debug, Clone, Default)]
pub struct PhysicalMemoryStats {
    /// Total physical memory (RAM)
    pub total_physical: u64,
    /// Available physical memory
    pub available_physical: u64,
    /// Used physical memory
    pub used_physical: u64,
    /// Memory used by OS cache
    pub cached: u64,
    /// Memory used by OS buffers
    pub buffers: u64,
    /// Swap/page file statistics
    pub swap: SwapStats,
}

/// Swap/page file statistics
#[derive(Debug, Clone)]
pub struct SwapStats {
    /// Total swap/page file size
    pub total_swap: u64,
    /// Used swap/page file
    pub used_swap: u64,
    /// Available swap/page file
    pub available_swap: u64,
    /// Swap-in rate (pages per second)
    pub swap_in_rate: f64,
    /// Swap-out rate (pages per second)
    pub swap_out_rate: f64,
}

impl Default for SwapStats {
    fn default() -> Self {
        SwapStats {
            total_swap: 0,
            used_swap: 0,
            available_swap: 0,
            swap_in_rate: 0.0,
            swap_out_rate: 0.0,
        }
    }
}

/// Process-specific memory statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessMemoryStats {
    /// Process virtual memory size
    pub virtual_size: u64,
    /// Process resident set size (RSS)
    pub resident_size: u64,
    /// Process shared memory
    pub shared_size: u64,
    /// Process private memory
    pub private_size: u64,
    /// Heap memory usage
    pub heap_size: u64,
    /// Stack memory usage
    pub stack_size: u64,
    /// Memory-mapped files
    pub mapped_files: u64,
    /// Process memory peak usage
    pub peak_usage: u64,
}

/// System-wide memory statistics
#[derive(Debug, Clone)]
pub struct SystemMemoryStats {
    /// Number of memory allocations
    pub allocation_count: u64,
    /// Number of memory deallocations
    pub deallocation_count: u64,
    /// Current active allocations
    pub active_allocations: u64,
    /// Total bytes allocated
    pub total_allocated: u64,
    /// Total bytes deallocated
    pub total_deallocated: u64,
    /// Memory fragmentation level
    pub fragmentation_level: f64,
    /// Large page usage
    pub large_pages: LargePageStats,
}

impl Default for SystemMemoryStats {
    fn default() -> Self {
        SystemMemoryStats {
            allocation_count: 0,
            deallocation_count: 0,
            active_allocations: 0,
            total_allocated: 0,
            total_deallocated: 0,
            fragmentation_level: 0.0,
            large_pages: LargePageStats::default(),
        }
    }
}

/// Large page usage statistics
#[derive(Debug, Clone, Default)]
pub struct LargePageStats {
    /// Whether large pages are supported
    pub supported: bool,
    /// Total large page memory
    pub total_large_pages: u64,
    /// Used large page memory
    pub used_large_pages: u64,
    /// Large page size
    pub page_size: u64,
}

/// Memory pressure indicators
#[derive(Debug, Clone)]
pub struct PressureIndicators {
    /// Overall memory pressure level
    pub pressure_level: PressureLevel,
    /// Whether system is in low memory condition
    pub low_memory: bool,
    /// Whether swapping is occurring
    pub swapping_active: bool,
    /// Memory allocation failure rate
    pub allocation_failure_rate: f64,
    /// GC pressure (if applicable)
    pub gc_pressure: Option<f64>,
}

impl Default for PressureIndicators {
    fn default() -> Self {
        PressureIndicators {
            pressure_level: PressureLevel::default(),
            low_memory: false,
            swapping_active: false,
            allocation_failure_rate: 0.0,
            gc_pressure: None,
        }
    }
}

/// Memory pressure levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum PressureLevel {
    /// Normal memory pressure
    #[default]
    Normal,
    /// Moderate memory pressure
    Moderate,
    /// High memory pressure
    High,
    /// Critical memory pressure
    Critical,
}

/// System information
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// Operating system name
    pub os_name: String,
    /// OS version
    pub os_version: String,
    /// System architecture
    pub architecture: String,
    /// Number of CPU cores
    pub cpu_cores: u32,
    /// CPU cache sizes
    pub cpu_cache: CpuCacheInfo,
    /// Page size
    pub page_size: u64,
    /// Large page size if supported
    pub large_page_size: Option<u64>,
    /// Memory management unit info
    pub mmu_info: MmuInfo,
}

/// CPU cache information
#[derive(Debug, Clone)]
pub struct CpuCacheInfo {
    /// L1 cache size per core
    pub l1_cache_size: u64,
    /// L2 cache size per core
    pub l2_cache_size: u64,
    /// L3 cache size (shared)
    pub l3_cache_size: Option<u64>,
    /// Cache line size
    pub cache_line_size: u64,
}

/// Memory Management Unit information
#[derive(Debug, Clone)]
pub struct MmuInfo {
    /// Virtual address space size (bits)
    pub virtual_address_bits: u32,
    /// Physical address space size (bits)
    pub physical_address_bits: u32,
    /// Whether ASLR is enabled
    pub aslr_enabled: bool,
    /// Whether NX/XD bit is supported
    pub nx_bit_supported: bool,
}

/// Platform-specific context
#[derive(Debug)]
struct MemoryContext {
    /// Whether collector is initialized
    initialized: bool,

    #[cfg(target_os = "linux")]
    linux_context: LinuxMemoryContext,

    #[cfg(target_os = "windows")]
    windows_context: WindowsMemoryContext,

    #[cfg(target_os = "macos")]
    macos_context: MacOSMemoryContext,
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct LinuxMemoryContext {
    /// Whether /proc/meminfo is accessible
    proc_meminfo_available: bool,
    /// Whether /proc/self/status is accessible
    proc_status_available: bool,
    /// Whether /proc/self/maps is accessible
    proc_maps_available: bool,
}

#[cfg(target_os = "windows")]
#[derive(Debug)]
struct WindowsMemoryContext {
    /// Whether GlobalMemoryStatusEx is available
    global_memory_api_available: bool,
    /// Whether GetProcessMemoryInfo is available
    process_memory_api_available: bool,
    /// Whether VirtualQueryEx is available
    virtual_query_available: bool,
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
struct MacOSMemoryContext {
    /// Whether vm_stat is available
    vm_stat_available: bool,
    /// Whether task_info is available
    task_info_available: bool,
    /// Whether mach APIs are available
    mach_api_available: bool,
}

impl PlatformMemoryInfo {
    /// Create new memory info collector
    pub fn new() -> Self {
        Self {
            last_stats: None,
            collection_interval: Duration::from_secs(1),
            last_collection: None,
            platform_context: MemoryContext::new(),
        }
    }

    /// Initialize memory info collector
    pub fn initialize(&mut self) -> Result<(), MemoryError> {
        #[cfg(target_os = "linux")]
        {
            self.initialize_linux()
        }

        #[cfg(target_os = "windows")]
        {
            self.initialize_windows()
        }

        #[cfg(target_os = "macos")]
        {
            self.initialize_macos()
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Err(MemoryError::UnsupportedPlatform)
        }
    }

    /// Collect current memory statistics
    pub fn collect_stats(&mut self) -> Result<MemoryStats, MemoryError> {
        if !self.platform_context.initialized {
            return Err(MemoryError::NotInitialized);
        }

        let now = Instant::now();

        // Check if we should collect (rate limiting)
        if let Some(last) = self.last_collection {
            if now.duration_since(last) < self.collection_interval {
                if let Some(ref stats) = self.last_stats {
                    return Ok(stats.clone());
                }
            }
        }

        let stats = self.perform_collection()?;
        self.last_stats = Some(stats.clone());
        self.last_collection = Some(now);

        Ok(stats)
    }

    /// Get system information
    pub fn get_system_info(&self) -> Result<SystemInfo, MemoryError> {
        if !self.platform_context.initialized {
            return Err(MemoryError::NotInitialized);
        }

        #[cfg(target_os = "linux")]
        return self.get_linux_system_info();

        #[cfg(target_os = "windows")]
        return self.get_windows_system_info();

        #[cfg(target_os = "macos")]
        return self.get_macos_system_info();

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        Err(MemoryError::UnsupportedPlatform)
    }

    /// Set collection interval
    pub fn set_collection_interval(&mut self, interval: Duration) {
        self.collection_interval = interval;
    }

    /// Get last collected statistics
    pub fn get_last_stats(&self) -> Option<&MemoryStats> {
        self.last_stats.as_ref()
    }

    fn perform_collection(&self) -> Result<MemoryStats, MemoryError> {
        #[cfg(target_os = "linux")]
        return self.collect_linux_stats();

        #[cfg(target_os = "windows")]
        return self.collect_windows_stats();

        #[cfg(target_os = "macos")]
        return self.collect_macos_stats();

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        Err(MemoryError::UnsupportedPlatform)
    }

    #[cfg(target_os = "linux")]
    fn initialize_linux(&mut self) -> Result<(), MemoryError> {
        // Check availability of Linux memory information sources
        self.platform_context.linux_context.proc_meminfo_available =
            std::path::Path::new("/proc/meminfo").exists();
        self.platform_context.linux_context.proc_status_available =
            std::path::Path::new("/proc/self/status").exists();
        self.platform_context.linux_context.proc_maps_available =
            std::path::Path::new("/proc/self/maps").exists();

        self.platform_context.initialized = true;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn initialize_windows(&mut self) -> Result<(), MemoryError> {
        // Check availability of Windows memory APIs
        self.platform_context
            .windows_context
            .global_memory_api_available = true; // Simplified
        self.platform_context
            .windows_context
            .process_memory_api_available = true; // Simplified
        self.platform_context
            .windows_context
            .virtual_query_available = true; // Simplified

        self.platform_context.initialized = true;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn initialize_macos(&mut self) -> Result<(), MemoryError> {
        // Check availability of macOS memory APIs
        self.platform_context.macos_context.vm_stat_available = true; // Simplified
        self.platform_context.macos_context.task_info_available = true; // Simplified
        self.platform_context.macos_context.mach_api_available = true; // Simplified

        self.platform_context.initialized = true;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn collect_linux_stats(&self) -> Result<MemoryStats, MemoryError> {
        let mut stats = MemoryStats::default();

        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 2 {
                    continue;
                }
                let value_kb: u64 = match parts[1].parse() {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::warn!(
                            "Failed to parse memory value for '{}': '{}', error: {}",
                            parts[0],
                            parts[1],
                            e
                        );
                        0
                    }
                };
                let value_bytes = value_kb * 1024;

                match parts[0] {
                    "MemTotal:" => stats.physical_memory.total_physical = value_bytes,
                    "MemAvailable:" => stats.physical_memory.available_physical = value_bytes,
                    "Buffers:" => stats.physical_memory.buffers = value_bytes,
                    "Cached:" => stats.physical_memory.cached = value_bytes,
                    "SwapTotal:" => stats.physical_memory.swap.total_swap = value_bytes,
                    "SwapFree:" => stats.physical_memory.swap.available_swap = value_bytes,
                    "SwapUsed:" => stats.physical_memory.swap.used_swap = value_bytes,
                    "Committed_AS:" => stats.virtual_memory.committed = value_bytes,
                    "VmallocTotal:" => stats.virtual_memory.total_virtual = value_bytes,
                    _ => {}
                }
            }
            stats.physical_memory.used_physical = stats
                .physical_memory
                .total_physical
                .saturating_sub(stats.physical_memory.available_physical);
            stats.physical_memory.swap.used_swap = stats
                .physical_memory
                .swap
                .total_swap
                .saturating_sub(stats.physical_memory.swap.available_swap);
            stats.virtual_memory.used_virtual = stats.virtual_memory.committed;
            stats.virtual_memory.available_virtual = stats
                .virtual_memory
                .total_virtual
                .saturating_sub(stats.virtual_memory.used_virtual);
            // Note: Real reserved memory would require reading /proc/iomem
            // Setting to 0 as fallback since it's not currently implemented
            stats.virtual_memory.reserved = 0;
        }

        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 2 {
                    continue;
                }
                let value_kb: u64 = parts[1].parse().unwrap_or(0);
                let value_bytes = value_kb * 1024;

                match parts[0] {
                    "VmSize:" => stats.process_memory.virtual_size = value_bytes,
                    "VmRSS:" => stats.process_memory.resident_size = value_bytes,
                    "RssAnon:" => stats.process_memory.private_size = value_bytes,
                    "RssFile:" => stats.process_memory.mapped_files = value_bytes,
                    "VmData:" => stats.process_memory.heap_size = value_bytes,
                    "VmStk:" => stats.process_memory.stack_size = value_bytes,
                    "VmPeak:" => stats.process_memory.peak_usage = value_bytes,
                    _ => {}
                }
            }
        }

        stats.pressure_indicators = PressureIndicators::default();

        Ok(stats)
    }

    #[cfg(target_os = "windows")]
    fn collect_windows_stats(&self) -> Result<MemoryStats, MemoryError> {
        use windows_sys::Win32::System::SystemInformation::{
            GetSystemInfo, GlobalMemoryStatusEx, MEMORYSTATUSEX, SYSTEM_INFO,
        };

        let mut mem_status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
        mem_status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;

        unsafe {
            if GlobalMemoryStatusEx(&mut mem_status) == 0 {
                return Err(MemoryError::SystemError(
                    "Failed to get memory status".to_string(),
                ));
            }
        }

        let mut sys_info: SYSTEM_INFO = unsafe { std::mem::zeroed() };
        unsafe { GetSystemInfo(&mut sys_info) };

        let total_physical = mem_status.ullTotalPhys;
        let available_physical = mem_status.ullAvailPhys;
        let total_virtual = mem_status.ullTotalVirtual;
        let available_virtual = mem_status.ullAvailVirtual;

        let _page_size = sys_info.dwPageSize as u64;
        let _total_memory_bytes = total_physical;
        let _available_memory_bytes = available_physical;
        let used_memory_bytes = total_physical.saturating_sub(available_physical);
        let _memory_usage_percent = if total_physical > 0 {
            (used_memory_bytes as f64 / total_physical as f64 * 100.0).round() as u32
        } else {
            0
        };

        Ok(MemoryStats {
            virtual_memory: VirtualMemoryStats {
                total_virtual,
                available_virtual,
                used_virtual: total_virtual - available_virtual,
                reserved: total_virtual / 4,
                // Use total page file as an estimate for committed memory
                committed: mem_status.ullTotalPageFile,
            },
            physical_memory: PhysicalMemoryStats {
                total_physical,
                available_physical,
                used_physical: total_physical - available_physical,
                cached: 0,
                buffers: 0,
                swap: SwapStats {
                    total_swap: mem_status.ullTotalPageFile,
                    used_swap: mem_status.ullTotalPageFile - mem_status.ullAvailPageFile,
                    available_swap: mem_status.ullAvailPageFile,
                    swap_in_rate: 0.0,
                    swap_out_rate: 0.0,
                },
            },
            process_memory: ProcessMemoryStats {
                virtual_size: 0,
                resident_size: 0,
                shared_size: 0,
                private_size: 0,
                heap_size: 0,
                stack_size: 0,
                mapped_files: 0,
                peak_usage: 0,
            },
            system_memory: SystemMemoryStats {
                allocation_count: 0,
                deallocation_count: 0,
                active_allocations: 0,
                total_allocated: 0,
                total_deallocated: 0,
                fragmentation_level: 0.0,
                large_pages: LargePageStats {
                    supported: true,
                    total_large_pages: 0,
                    used_large_pages: 0,
                    page_size: sys_info.dwPageSize as u64,
                },
            },
            pressure_indicators: PressureIndicators {
                pressure_level: if mem_status.dwMemoryLoad > 90 {
                    PressureLevel::Critical
                } else if mem_status.dwMemoryLoad > 70 {
                    PressureLevel::High
                } else if mem_status.dwMemoryLoad > 50 {
                    PressureLevel::Moderate
                } else {
                    PressureLevel::Normal
                },
                low_memory: mem_status.dwMemoryLoad > 80,
                swapping_active: mem_status.ullTotalPageFile - mem_status.ullAvailPageFile > 0,
                allocation_failure_rate: 0.0,
                gc_pressure: None,
            },
            timestamp: Instant::now(),
        })
    }

    #[cfg(target_os = "macos")]
    #[allow(deprecated)] // libc::mach_host_self and mach_task_self are deprecated in favor of mach2 crate
    fn collect_macos_stats(&self) -> Result<MemoryStats, MemoryError> {
        use libc::{c_int, host_statistics64, mach_host_self, vm_statistics64};

        // Get host port
        let host = unsafe { mach_host_self() };

        // Get VM statistics
        let mut vm_stats: vm_statistics64 = unsafe { std::mem::zeroed() };
        let mut count =
            (std::mem::size_of::<vm_statistics64>() / std::mem::size_of::<c_int>()) as u32;

        let result = unsafe {
            host_statistics64(
                host,
                libc::HOST_VM_INFO64,
                &mut vm_stats as *mut vm_statistics64 as *mut c_int,
                &mut count,
            )
        };

        // Get physical memory
        let mut total_physical: u64 = 0;
        unsafe {
            let mut size = std::mem::size_of::<u64>();
            if libc::sysctlbyname(
                c"hw.memsize".as_ptr(),
                &mut total_physical as *mut u64 as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            ) != 0
            {
                // Failed to get physical memory size
                return Err(MemoryError::SystemError(
                    "Failed to get physical memory size via sysctl(hw.memsize)".to_string(),
                ));
            }
        }

        // Get page size
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as u64 };
        let page_size = if page_size == 0 { 4096 } else { page_size };

        // Calculate memory values from VM statistics
        let (physical_memory, available_physical, used_physical, cached, buffers) = if result == 0 {
            let free = vm_stats.free_count as u64 * page_size;
            let inactive = vm_stats.inactive_count as u64 * page_size;
            let wired = vm_stats.wire_count as u64 * page_size;
            let active = vm_stats.active_count as u64 * page_size;
            let speculative = vm_stats.speculative_count as u64 * page_size;

            let used = wired + active;
            let available = free + inactive + speculative;
            let cached_pages = inactive; // On macOS, inactive pages are similar to cache

            (total_physical, available, used, cached_pages, 0)
        } else {
            // Fallback values if host_statistics64 fails
            (total_physical, total_physical / 2, total_physical / 2, 0, 0)
        };

        // Get swap info - estimate from compressed memory
        let compressed = vm_stats.compressor_page_count as u64 * page_size;
        let swap_used_estimated = compressed; // Compressed pages often correlate with swap

        // Get real swap info using sysctl
        let (total_swap, available_swap) = unsafe {
            let mut swap_usage: libc::xsw_usage = std::mem::zeroed();
            let mut size = std::mem::size_of::<libc::xsw_usage>();
            let result = libc::sysctlbyname(
                c"vm.swapusage".as_ptr(),
                &mut swap_usage as *mut libc::xsw_usage as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            );

            if result == 0 {
                (swap_usage.xsu_total, swap_usage.xsu_avail)
            } else {
                // Fallback: use compressed memory as estimate
                (compressed, 0)
            }
        };

        // Get process memory info using task_info
        let process_memory = unsafe {
            let mut task_info: libc::mach_task_basic_info = std::mem::zeroed();
            let mut count = (std::mem::size_of::<libc::mach_task_basic_info>()
                / std::mem::size_of::<libc::natural_t>()) as u32;

            let result = libc::task_info(
                libc::mach_task_self(),
                libc::MACH_TASK_BASIC_INFO,
                &mut task_info as *mut libc::mach_task_basic_info as *mut libc::c_int,
                &mut count,
            );

            if result == 0 {
                ProcessMemoryStats {
                    virtual_size: task_info.virtual_size,
                    resident_size: task_info.resident_size,
                    shared_size: 0,                        // Not directly available
                    private_size: task_info.resident_size, // Approximation
                    heap_size: 0,                          // Not directly available
                    stack_size: 0,                         // Not directly available
                    mapped_files: 0,
                    peak_usage: task_info.resident_size_max,
                }
            } else {
                // Fallback - return zero values when task_info fails
                ProcessMemoryStats {
                    virtual_size: 0,
                    resident_size: 0,
                    shared_size: 0,
                    private_size: 0,
                    heap_size: 0,
                    stack_size: 0,
                    mapped_files: 0,
                    peak_usage: 0,
                }
            }
        };

        // Determine memory pressure
        let pressure_level = if available_physical < total_physical / 10 {
            PressureLevel::Critical
        } else if available_physical < total_physical / 5 {
            PressureLevel::High
        } else if available_physical < total_physical / 3 {
            PressureLevel::Moderate
        } else {
            PressureLevel::Normal
        };

        Ok(MemoryStats {
            virtual_memory: VirtualMemoryStats {
                // On macOS, user space virtual memory is limited by the architecture
                // For x86_64 and ARM64, user space typically has 128TB virtual address space
                // We use a more accurate estimate based on the process's virtual memory
                total_virtual: process_memory.virtual_size.max(physical_memory * 2),
                available_virtual: physical_memory,
                used_virtual: process_memory.virtual_size,
                reserved: process_memory.virtual_size / 4,
                committed: process_memory.virtual_size / 4,
            },
            physical_memory: PhysicalMemoryStats {
                total_physical: physical_memory,
                available_physical,
                used_physical,
                cached,
                buffers,
                swap: SwapStats {
                    total_swap,
                    used_swap: swap_used_estimated,
                    available_swap,
                    swap_in_rate: 0.0,
                    swap_out_rate: 0.0,
                },
            },
            process_memory,
            system_memory: SystemMemoryStats {
                allocation_count: 0,
                deallocation_count: 0,
                active_allocations: 0,
                total_allocated: 0,
                total_deallocated: 0,
                fragmentation_level: 0.0,
                large_pages: LargePageStats {
                    supported: false,
                    total_large_pages: 0,
                    used_large_pages: 0,
                    page_size,
                },
            },
            pressure_indicators: PressureIndicators {
                pressure_level,
                low_memory: pressure_level >= PressureLevel::High,
                swapping_active: swap_used_estimated > 0,
                allocation_failure_rate: 0.0,
                gc_pressure: None,
            },
            timestamp: Instant::now(),
        })
    }

    #[cfg(target_os = "linux")]
    fn get_linux_system_info(&self) -> Result<SystemInfo, MemoryError> {
        // Get OS version from /proc/sys/kernel/osrelease
        let os_version = std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        // Get architecture from uname
        let architecture = unsafe {
            let mut uname: libc::utsname = std::mem::zeroed();
            if libc::uname(&mut uname) == 0 {
                let machine = std::ffi::CStr::from_ptr(uname.machine.as_ptr())
                    .to_string_lossy()
                    .to_string();
                machine
            } else {
                "unknown".to_string()
            }
        };

        // Get CPU cores from /proc/cpuinfo
        let cpu_cores = if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
            cpuinfo
                .lines()
                .filter(|line| line.starts_with("processor"))
                .count() as u32
        } else {
            1
        };

        // Get page size
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as u64 };
        let page_size = if page_size == 0 { 4096 } else { page_size };

        // Get cache info from /proc/cpuinfo
        let (l1_cache_size, l2_cache_size, l3_cache_size, cache_line_size) =
            if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
                let mut l1 = 0u64;
                let mut l2 = 0u64;
                let mut l3 = 0u64;
                let mut line_size = 64u64;

                for line in cpuinfo.lines() {
                    if line.contains("cache size") {
                        // Format: "cache size : 6144 KB"
                        if let Some(kb_str) = line.split(':').nth(1) {
                            if let Some(kb_val) = kb_str.split_whitespace().next() {
                                if let Ok(kb) = kb_val.parse::<u64>() {
                                    let bytes = kb * 1024;
                                    // Heuristic: L1 < 256KB, L2 < 4MB, L3 >= 4MB
                                    if bytes < 256 * 1024 && l1 == 0 {
                                        l1 = bytes;
                                    } else if bytes < 4 * 1024 * 1024 && l2 == 0 {
                                        l2 = bytes;
                                    } else if bytes >= 4 * 1024 * 1024 && l3 == 0 {
                                        l3 = bytes;
                                    }
                                }
                            }
                        }
                    }
                    if line.contains("cache_alignment") {
                        // Format: "cache_alignment : 64"
                        if let Some(val_str) = line.split(':').nth(1) {
                            if let Ok(val) = val_str.trim().parse::<u64>() {
                                line_size = val;
                            }
                        }
                    }
                }

                (l1, l2, l3, line_size)
            } else {
                (0, 0, 0, 64)
            };

        Ok(SystemInfo {
            os_name: "Linux".to_string(),
            os_version,
            architecture,
            cpu_cores,
            cpu_cache: CpuCacheInfo {
                l1_cache_size,
                l2_cache_size,
                l3_cache_size: if l3_cache_size > 0 {
                    Some(l3_cache_size)
                } else {
                    None
                },
                cache_line_size,
            },
            page_size,
            large_page_size: None, // Not universally supported on Linux
            mmu_info: MmuInfo {
                virtual_address_bits: 48,  // x86_64 typical
                physical_address_bits: 40, // x86_64 typical
                aslr_enabled: true,
                nx_bit_supported: true,
            },
        })
    }

    #[cfg(target_os = "windows")]
    fn get_windows_system_info(&self) -> Result<SystemInfo, MemoryError> {
        use windows_sys::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

        let mut sys_info: SYSTEM_INFO = unsafe { std::mem::zeroed() };
        unsafe { GetSystemInfo(&mut sys_info) };

        let page_size = sys_info.dwPageSize as u64;
        let cpu_cores = sys_info.dwNumberOfProcessors as u32;

        let architecture = match unsafe { sys_info.Anonymous.Anonymous.wProcessorArchitecture } {
            5 => "ARM",
            6 => "ARM64",
            9 => "x64",
            12 => "ARM",
            0 => "x86",
            _ => "Unknown",
        };

        Ok(SystemInfo {
            os_name: "Windows".to_string(),
            os_version: std::env::var("OS").unwrap_or_else(|_| "Unknown".to_string()),
            architecture: architecture.to_string(),
            cpu_cores,
            cpu_cache: CpuCacheInfo {
                l1_cache_size: 0,
                l2_cache_size: 0,
                l3_cache_size: None,
                cache_line_size: page_size,
            },
            page_size,
            large_page_size: Some(sys_info.dwPageSize as u64),
            mmu_info: MmuInfo {
                virtual_address_bits: if unsafe {
                    sys_info.Anonymous.Anonymous.wProcessorArchitecture
                } == 9
                {
                    48
                } else {
                    32
                },
                physical_address_bits: 0,
                aslr_enabled: true,
                nx_bit_supported: true,
            },
        })
    }

    #[cfg(target_os = "macos")]
    fn get_macos_system_info(&self) -> Result<SystemInfo, MemoryError> {
        // Get OS version
        let os_version = unsafe {
            let mut size: libc::size_t = 256;
            let mut buf = [0u8; 256];
            if libc::sysctlbyname(
                c"kern.osrelease".as_ptr(),
                buf.as_mut_ptr() as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            ) == 0
                && size > 0
            {
                String::from_utf8_lossy(&buf[..size.min(buf.len())]).to_string()
            } else {
                "Unknown".to_string()
            }
        };

        // Get architecture
        let architecture = unsafe {
            let mut size: libc::size_t = 256;
            let mut buf = [0u8; 256];
            if libc::sysctlbyname(
                c"hw.machine".as_ptr(),
                buf.as_mut_ptr() as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            ) == 0
                && size > 0
            {
                let arch_str = String::from_utf8_lossy(&buf[..size.min(buf.len())]).to_string();
                // Convert arm64, x86_64 to standard format
                if arch_str.contains("arm64") || arch_str.contains("arm") {
                    "arm64".to_string()
                } else {
                    arch_str
                }
            } else {
                "unknown".to_string()
            }
        };

        // Get CPU cores
        let mut size = std::mem::size_of::<u32>();
        let mut cpu_cores: u32 = 1;
        unsafe {
            let mut mib: [libc::c_int; 2] = [libc::CTL_HW, libc::HW_NCPU];
            if libc::sysctl(
                mib.as_mut_ptr(),
                mib.len() as libc::c_uint,
                &mut cpu_cores as *mut u32 as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            ) == 0
            {
                // Successfully got CPU cores
            }
        }

        // Get page size
        let mut page_size: u64 = 4096;
        unsafe {
            size = std::mem::size_of::<u64>();
            if libc::sysctlbyname(
                c"hw.pagesize".as_ptr(),
                &mut page_size as *mut u64 as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            ) != 0
            {
                page_size = 4096; // Default fallback
            }
        }

        // Get cache line size
        let mut cache_line_size: u64 = 64;
        unsafe {
            size = std::mem::size_of::<u64>();
            if libc::sysctlbyname(
                c"hw.cachelinesize".as_ptr(),
                &mut cache_line_size as *mut u64 as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            ) != 0
            {
                cache_line_size = 64; // Default fallback
            }
        }

        // Get L1 cache size
        let mut l1_cache_size: u64 = 0;
        unsafe {
            size = std::mem::size_of::<u64>();
            if libc::sysctlbyname(
                c"hw.l1dcachesize".as_ptr(),
                &mut l1_cache_size as *mut u64 as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            ) != 0
            {
                // Try alternative
                if libc::sysctlbyname(
                    c"hw.l1icachesize".as_ptr(),
                    &mut l1_cache_size as *mut u64 as *mut libc::c_void,
                    &mut size,
                    std::ptr::null_mut(),
                    0,
                ) != 0
                {
                    l1_cache_size = 0;
                }
            }
        }

        // Get L2 cache size
        let mut l2_cache_size: u64 = 0;
        unsafe {
            size = std::mem::size_of::<u64>();
            if libc::sysctlbyname(
                c"hw.l2cachesize".as_ptr(),
                &mut l2_cache_size as *mut u64 as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            ) != 0
            {
                l2_cache_size = 0;
            }
        }

        // Get L3 cache size (may not exist on Apple Silicon)
        let mut l3_cache_size: u64 = 0;
        unsafe {
            size = std::mem::size_of::<u64>();
            if libc::sysctlbyname(
                c"hw.l3cachesize".as_ptr(),
                &mut l3_cache_size as *mut u64 as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            ) != 0
            {
                l3_cache_size = 0;
            }
        }

        Ok(SystemInfo {
            os_name: "macOS".to_string(),
            os_version,
            architecture,
            cpu_cores,
            cpu_cache: CpuCacheInfo {
                l1_cache_size,
                l2_cache_size,
                l3_cache_size: if l3_cache_size > 0 {
                    Some(l3_cache_size)
                } else {
                    None
                },
                cache_line_size,
            },
            page_size,
            large_page_size: None, // Not supported on Apple Silicon
            mmu_info: MmuInfo {
                virtual_address_bits: 48,
                physical_address_bits: 40,
                aslr_enabled: true,
                nx_bit_supported: true,
            },
        })
    }
}

impl MemoryContext {
    fn new() -> Self {
        Self {
            initialized: false,
            #[cfg(target_os = "linux")]
            linux_context: LinuxMemoryContext {
                proc_meminfo_available: false,
                proc_status_available: false,
                proc_maps_available: false,
            },
            #[cfg(target_os = "windows")]
            windows_context: WindowsMemoryContext {
                global_memory_api_available: false,
                process_memory_api_available: false,
                virtual_query_available: false,
            },
            #[cfg(target_os = "macos")]
            macos_context: MacOSMemoryContext {
                vm_stat_available: false,
                task_info_available: false,
                mach_api_available: false,
            },
        }
    }
}

/// Errors that can occur during memory information collection
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryError {
    /// Platform not supported
    UnsupportedPlatform,
    /// Collector not initialized
    NotInitialized,
    /// Permission denied
    PermissionDenied,
    /// System API error
    SystemError(String),
    /// Parse error
    ParseError(String),
    /// I/O error
    IoError(String),
    /// Feature not implemented
    NotImplemented(String),
}

impl Default for PlatformMemoryInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::UnsupportedPlatform => {
                write!(f, "Platform not supported for memory info collection")
            }
            MemoryError::NotInitialized => write!(f, "Memory info collector not initialized"),
            MemoryError::PermissionDenied => write!(f, "Permission denied for memory info access"),
            MemoryError::SystemError(msg) => write!(f, "System error: {}", msg),
            MemoryError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            MemoryError::IoError(msg) => write!(f, "I/O error: {}", msg),
            MemoryError::NotImplemented(msg) => {
                write!(f, "Feature not implemented: {}", msg)
            }
        }
    }
}

impl std::error::Error for MemoryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_info_creation() {
        let info = PlatformMemoryInfo::new();
        assert!(!info.platform_context.initialized);
        assert!(info.last_stats.is_none());
    }

    #[test]
    fn test_initialization() {
        let mut info = PlatformMemoryInfo::new();
        let result = info.initialize();

        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        assert!(result.is_ok());

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        assert_eq!(result, Err(MemoryError::UnsupportedPlatform));
    }

    #[test]
    fn test_stats_collection() {
        let mut info = PlatformMemoryInfo::new();
        let _ = info.initialize();

        let result = info.collect_stats();

        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        {
            if info.platform_context.initialized {
                assert!(result.is_ok());
                let stats = result.expect("Stats should be collected");
                assert!(stats.physical_memory.total_physical > 0);
                assert!(stats.virtual_memory.total_virtual > 0);
            }
        }
    }

    #[test]
    fn test_system_info() {
        let mut info = PlatformMemoryInfo::new();
        let _ = info.initialize();

        let result = info.get_system_info();

        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        {
            if info.platform_context.initialized {
                assert!(result.is_ok());
                let sys_info = result.expect("System info should be available");
                assert!(!sys_info.os_name.is_empty());
                assert!(sys_info.cpu_cores > 0);
                assert!(sys_info.page_size > 0);
            }
        }
    }

    #[test]
    fn test_pressure_level_ordering() {
        assert!(PressureLevel::Critical > PressureLevel::High);
        assert!(PressureLevel::High > PressureLevel::Moderate);
        assert!(PressureLevel::Moderate > PressureLevel::Normal);
    }

    #[test]
    fn test_collection_interval() {
        let mut info = PlatformMemoryInfo::new();
        info.set_collection_interval(Duration::from_millis(500));
        assert_eq!(info.collection_interval, Duration::from_millis(500));
    }
}
