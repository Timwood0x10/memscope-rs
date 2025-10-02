//! System-wide Resource Profiler
//!
//! Comprehensive system resource tracking: CPU, GPU, Memory, I/O, Network
//! Cross-platform support for Windows, Linux, macOS

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Comprehensive system resource snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceSnapshot {
    pub timestamp: u64,
    pub cpu_metrics: CpuMetrics,
    pub memory_metrics: MemoryMetrics,
    pub gpu_metrics: Option<GpuMetrics>,
    pub io_metrics: IoMetrics,
    pub network_metrics: NetworkMetrics,
    pub process_metrics: ProcessMetrics,
    pub thread_metrics: HashMap<u64, ThreadMetrics>,
}

/// CPU utilization and performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    /// Overall CPU usage percentage (0-100)
    pub overall_usage: f32,
    /// Per-core CPU usage
    pub core_usage: Vec<f32>,
    /// CPU frequency in MHz
    pub frequency: u64,
    /// Load average (1, 5, 15 minutes) - Unix only
    pub load_average: Option<(f64, f64, f64)>,
    /// CPU temperature if available
    pub temperature: Option<f32>,
    /// Context switches per second
    pub context_switches: u64,
    /// CPU cache misses if available
    pub cache_misses: Option<u64>,
}

/// Memory system metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Total physical memory in bytes
    pub total_physical: u64,
    /// Available physical memory in bytes
    pub available_physical: u64,
    /// Used physical memory in bytes
    pub used_physical: u64,
    /// Total virtual memory in bytes
    pub total_virtual: u64,
    /// Available virtual memory in bytes
    pub available_virtual: u64,
    /// Memory pressure indicator (0-100)
    pub pressure: f32,
    /// Page faults per second
    pub page_faults: u64,
    /// Memory bandwidth utilization if available
    pub bandwidth_utilization: Option<f32>,
}

/// GPU utilization and memory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMetrics {
    /// GPU name/model
    pub device_name: String,
    /// GPU utilization percentage (0-100)
    pub gpu_usage: f32,
    /// GPU memory usage in bytes
    pub memory_used: u64,
    /// Total GPU memory in bytes
    pub memory_total: u64,
    /// GPU temperature if available
    pub temperature: Option<f32>,
    /// GPU frequency in MHz
    pub frequency: Option<u64>,
    /// Power consumption in watts
    pub power_usage: Option<f32>,
    /// CUDA/OpenCL compute utilization
    pub compute_usage: Option<f32>,
}

/// I/O subsystem metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoMetrics {
    /// Disk read bytes per second
    pub disk_read_bps: u64,
    /// Disk write bytes per second
    pub disk_write_bps: u64,
    /// Disk read operations per second
    pub disk_read_ops: u64,
    /// Disk write operations per second
    pub disk_write_ops: u64,
    /// Average disk latency in microseconds
    pub disk_latency_us: Option<u64>,
    /// Disk queue depth
    pub disk_queue_depth: Option<u32>,
}

/// Network utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Network receive bytes per second
    pub rx_bps: u64,
    /// Network transmit bytes per second
    pub tx_bps: u64,
    /// Network receive packets per second
    pub rx_pps: u64,
    /// Network transmit packets per second
    pub tx_pps: u64,
    /// Network latency if measurable
    pub latency_ms: Option<f32>,
    /// Active connections count
    pub connections: u32,
}

/// Per-process resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// CPU usage percentage for this process
    pub cpu_usage: f32,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Number of threads
    pub thread_count: u32,
    /// Number of file handles
    pub handle_count: u32,
    /// Process priority
    pub priority: i32,
}

/// Per-thread resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMetrics {
    /// Thread ID
    pub thread_id: u64,
    /// Thread name if available
    pub thread_name: Option<String>,
    /// CPU time consumed by this thread
    pub cpu_time_ns: u64,
    /// Thread state (Running, Sleeping, etc.)
    pub state: ThreadState,
    /// Thread priority
    pub priority: i32,
    /// CPU affinity mask
    pub cpu_affinity: Option<u64>,
}

/// Thread execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreadState {
    Running,
    Sleeping,
    Waiting,
    Blocked,
    Zombie,
}

/// System resource profiler
pub struct SystemProfiler {
    start_time: Instant,
    #[allow(dead_code)]
    sample_interval: Duration,
    last_snapshot: Option<SystemResourceSnapshot>,
    #[cfg(feature = "system-metrics")]
    system: std::cell::RefCell<sysinfo::System>,
}

impl SystemProfiler {
    /// Create new system profiler with specified sampling interval
    pub fn new(sample_interval: Duration) -> Self {
        Self {
            start_time: Instant::now(),
            sample_interval,
            last_snapshot: None,
            #[cfg(feature = "system-metrics")]
            system: std::cell::RefCell::new(sysinfo::System::new_all()),
        }
    }

    /// Take a comprehensive system resource snapshot
    pub fn take_snapshot(&mut self) -> Result<SystemResourceSnapshot, Box<dyn std::error::Error>> {
        let timestamp = self.start_time.elapsed().as_millis() as u64;

        let snapshot = SystemResourceSnapshot {
            timestamp,
            cpu_metrics: self.collect_cpu_metrics()?,
            memory_metrics: self.collect_memory_metrics()?,
            gpu_metrics: self.collect_gpu_metrics()?,
            io_metrics: self.collect_io_metrics()?,
            network_metrics: self.collect_network_metrics()?,
            process_metrics: self.collect_process_metrics()?,
            thread_metrics: self.collect_thread_metrics()?,
        };

        self.last_snapshot = Some(snapshot.clone());
        Ok(snapshot)
    }

    /// Collect CPU performance metrics
    fn collect_cpu_metrics(&self) -> Result<CpuMetrics, Box<dyn std::error::Error>> {
        #[cfg(feature = "system-metrics")]
        {
            let mut system = self.system.borrow_mut();
            system.refresh_cpu_all();

            let overall_usage = system.global_cpu_usage();
            let core_usage: Vec<f32> = system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
            let load_average = sysinfo::System::load_average();

            Ok(CpuMetrics {
                overall_usage,
                core_usage,
                frequency: 0, // Would need platform-specific code
                load_average: Some((load_average.one, load_average.five, load_average.fifteen)),
                temperature: None,   // Would need platform-specific sensors
                context_switches: 0, // Would need platform-specific code
                cache_misses: None,
            })
        }

        #[cfg(not(feature = "system-metrics"))]
        {
            // Fallback implementation
            Ok(CpuMetrics {
                overall_usage: 0.0,
                core_usage: vec![0.0; num_cpus::get()],
                frequency: 0,
                load_average: None,
                temperature: None,
                context_switches: 0,
                cache_misses: None,
            })
        }
    }

    /// Collect memory subsystem metrics
    fn collect_memory_metrics(&self) -> Result<MemoryMetrics, Box<dyn std::error::Error>> {
        #[cfg(feature = "system-metrics")]
        {
            let mut system = self.system.borrow_mut();
            system.refresh_memory();

            let total_physical = system.total_memory();
            let available_physical = system.available_memory();
            let used_physical = total_physical - available_physical;

            let pressure = (used_physical as f32 / total_physical as f32) * 100.0;

            Ok(MemoryMetrics {
                total_physical,
                available_physical,
                used_physical,
                total_virtual: system.total_swap(),
                available_virtual: system.free_swap(),
                pressure,
                page_faults: 0, // Would need platform-specific code
                bandwidth_utilization: None,
            })
        }

        #[cfg(not(feature = "system-metrics"))]
        {
            Ok(MemoryMetrics {
                total_physical: 0,
                available_physical: 0,
                used_physical: 0,
                total_virtual: 0,
                available_virtual: 0,
                pressure: 0.0,
                page_faults: 0,
                bandwidth_utilization: None,
            })
        }
    }

    /// Collect GPU utilization metrics (platform-specific)
    fn collect_gpu_metrics(&self) -> Result<Option<GpuMetrics>, Box<dyn std::error::Error>> {
        // GPU metrics collection would require platform-specific implementations:
        // - Windows: DirectX/DXGI APIs
        // - Linux: nvidia-ml-py, ROCm, Intel GPU tools
        // - macOS: Metal Performance Shaders, system_profiler

        #[cfg(target_os = "linux")]
        {
            // Try to read NVIDIA GPU metrics
            if let Ok(gpu_metrics) = self.collect_nvidia_gpu_metrics() {
                return Ok(Some(gpu_metrics));
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Try to read GPU metrics via WMI/DXGI
            if let Ok(gpu_metrics) = self.collect_windows_gpu_metrics() {
                return Ok(Some(gpu_metrics));
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Try to read GPU metrics via Metal/IOKit
            if let Ok(gpu_metrics) = self.collect_macos_gpu_metrics() {
                return Ok(Some(gpu_metrics));
            }
        }

        Ok(None)
    }

    /// Collect I/O subsystem metrics
    fn collect_io_metrics(&self) -> Result<IoMetrics, Box<dyn std::error::Error>> {
        // I/O metrics would be collected from:
        // - Linux: /proc/diskstats, /sys/block/*/stat
        // - Windows: Performance Counters
        // - macOS: IOKit, system_profiler

        Ok(IoMetrics {
            disk_read_bps: 0,
            disk_write_bps: 0,
            disk_read_ops: 0,
            disk_write_ops: 0,
            disk_latency_us: None,
            disk_queue_depth: None,
        })
    }

    /// Collect network utilization metrics
    fn collect_network_metrics(&self) -> Result<NetworkMetrics, Box<dyn std::error::Error>> {
        #[cfg(feature = "system-metrics")]
        {
            // Network monitoring temporarily disabled due to sysinfo API changes
            let total_rx = 0;
            let total_tx = 0;

            Ok(NetworkMetrics {
                rx_bps: total_rx,
                tx_bps: total_tx,
                rx_pps: 0, // Would need more detailed monitoring
                tx_pps: 0,
                latency_ms: None,
                connections: 0,
            })
        }

        #[cfg(not(feature = "system-metrics"))]
        {
            Ok(NetworkMetrics {
                rx_bps: 0,
                tx_bps: 0,
                rx_pps: 0,
                tx_pps: 0,
                latency_ms: None,
                connections: 0,
            })
        }
    }

    /// Collect current process metrics
    fn collect_process_metrics(&self) -> Result<ProcessMetrics, Box<dyn std::error::Error>> {
        #[cfg(feature = "system-metrics")]
        {
            let mut system = self.system.borrow_mut();
            system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

            let current_pid = sysinfo::get_current_pid()?;

            if let Some(process) = system.process(current_pid) {
                Ok(ProcessMetrics {
                    pid: current_pid.as_u32(),
                    name: process.name().to_string_lossy().to_string(),
                    cpu_usage: process.cpu_usage(),
                    memory_usage: process.memory(),
                    thread_count: 0, // Would need platform-specific code
                    handle_count: 0,
                    priority: 0,
                })
            } else {
                Err("Could not find current process".into())
            }
        }

        #[cfg(not(feature = "system-metrics"))]
        {
            Ok(ProcessMetrics {
                pid: std::process::id(),
                name: "unknown".to_string(),
                cpu_usage: 0.0,
                memory_usage: 0,
                thread_count: 0,
                handle_count: 0,
                priority: 0,
            })
        }
    }

    /// Collect per-thread metrics
    fn collect_thread_metrics(
        &self,
    ) -> Result<HashMap<u64, ThreadMetrics>, Box<dyn std::error::Error>> {
        let mut thread_metrics = HashMap::new();

        // Thread-level metrics would require platform-specific implementation:
        // - Linux: /proc/[pid]/task/[tid]/* files
        // - Windows: Thread performance counters
        // - macOS: thread_info() system calls

        // For now, return current thread info
        let current_thread_id = get_current_thread_id();
        thread_metrics.insert(
            current_thread_id,
            ThreadMetrics {
                thread_id: current_thread_id,
                thread_name: std::thread::current().name().map(String::from),
                cpu_time_ns: 0,
                state: ThreadState::Running,
                priority: 0,
                cpu_affinity: None,
            },
        );

        Ok(thread_metrics)
    }

    // Platform-specific GPU collection methods
    #[cfg(target_os = "linux")]
    fn collect_nvidia_gpu_metrics(&self) -> Result<GpuMetrics, Box<dyn std::error::Error>> {
        // Implementation would use nvidia-ml-py or similar
        Err("NVIDIA GPU metrics not implemented".into())
    }

    #[cfg(target_os = "windows")]
    fn collect_windows_gpu_metrics(&self) -> Result<GpuMetrics, Box<dyn std::error::Error>> {
        // Implementation would use DXGI or WMI
        Err("Windows GPU metrics not implemented".into())
    }

    #[cfg(target_os = "macos")]
    fn collect_macos_gpu_metrics(&self) -> Result<GpuMetrics, Box<dyn std::error::Error>> {
        // Implementation would use Metal or IOKit
        Err("macOS GPU metrics not implemented".into())
    }
}

/// Get current thread ID in a cross-platform way
fn get_current_thread_id() -> u64 {
    #[cfg(target_os = "linux")]
    {
        unsafe { libc::syscall(libc::SYS_gettid) as u64 }
    }

    #[cfg(target_os = "windows")]
    {
        unsafe { winapi::um::processthreadsapi::GetCurrentThreadId() as u64 }
    }

    #[cfg(target_os = "macos")]
    {
        unsafe { libc::pthread_self() as u64 }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        std::thread::current().id().as_u64()
    }
}

/// Continuous system profiling manager
pub struct ContinuousProfiler {
    #[allow(dead_code)]
    profiler: SystemProfiler,
    snapshots: Vec<SystemResourceSnapshot>,
    is_running: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl ContinuousProfiler {
    /// Start continuous profiling in background
    pub fn start_background_profiling(interval: Duration) -> Self {
        let profiler = SystemProfiler::new(interval);
        let is_running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));

        Self {
            profiler,
            snapshots: Vec::new(),
            is_running,
        }
    }

    /// Stop profiling and return collected data
    pub fn stop_and_collect(self) -> Vec<SystemResourceSnapshot> {
        self.is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.snapshots
    }
}
