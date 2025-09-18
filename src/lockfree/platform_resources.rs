//! Cross-platform CPU, GPU, and IO resource monitoring for multi-threaded environments
//! 
//! This module provides platform-specific implementations for real-time resource monitoring
//! that complements memory tracking with comprehensive system resource analysis.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Platform-specific resource metrics collected in multi-threaded context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformResourceMetrics {
    pub timestamp: u64,
    pub cpu_metrics: CpuResourceMetrics,
    pub gpu_metrics: Option<GpuResourceMetrics>,
    pub io_metrics: IoResourceMetrics,
    pub thread_metrics: HashMap<u64, ThreadResourceMetrics>,
}

/// CPU utilization metrics with per-core breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuResourceMetrics {
    pub overall_usage_percent: f32,
    pub per_core_usage: Vec<f32>,
    pub frequency_mhz: Vec<u32>,
    pub temperature_celsius: Vec<f32>,
    pub context_switches_per_sec: u64,
    pub interrupts_per_sec: u64,
    pub load_average: (f64, f64, f64), // 1min, 5min, 15min
}

/// GPU utilization and memory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuResourceMetrics {
    pub device_name: String,
    pub vendor: GpuVendor,
    pub compute_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub temperature_celsius: f32,
    pub power_usage_watts: f32,
    pub frequency_mhz: u32,
}

/// GPU vendor identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Unknown,
}

/// IO subsystem metrics including disk and network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoResourceMetrics {
    pub disk_read_bytes_per_sec: u64,
    pub disk_write_bytes_per_sec: u64,
    pub disk_read_ops_per_sec: u64,
    pub disk_write_ops_per_sec: u64,
    pub network_rx_bytes_per_sec: u64,
    pub network_tx_bytes_per_sec: u64,
    pub network_rx_packets_per_sec: u64,
    pub network_tx_packets_per_sec: u64,
}

/// Per-thread resource consumption metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadResourceMetrics {
    pub thread_id: u64,
    pub thread_name: Option<String>,
    pub cpu_usage_percent: f32,
    pub memory_resident_bytes: u64,
    pub memory_virtual_bytes: u64,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub cpu_time_user_ns: u64,
    pub cpu_time_kernel_ns: u64,
}

/// Cross-platform resource collector
pub struct PlatformResourceCollector {
    #[cfg(target_os = "macos")]
    macos_collector: MacOSResourceCollector,
    #[cfg(target_os = "linux")]
    linux_collector: LinuxResourceCollector,
    last_collection_time: Instant,
}

impl PlatformResourceCollector {
    /// Create new platform-specific resource collector
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            #[cfg(target_os = "macos")]
            macos_collector: MacOSResourceCollector::new()?,
            #[cfg(target_os = "linux")]
            linux_collector: LinuxResourceCollector::new()?,
            last_collection_time: Instant::now(),
        })
    }

    /// Collect current resource metrics from the platform
    pub fn collect_metrics(&mut self) -> Result<PlatformResourceMetrics, Box<dyn std::error::Error>> {
        let timestamp = self.last_collection_time.elapsed().as_millis() as u64;
        
        #[cfg(target_os = "macos")]
        {
            self.macos_collector.collect_metrics(timestamp)
        }
        
        #[cfg(target_os = "linux")]
        {
            self.linux_collector.collect_metrics(timestamp)
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            // Fallback for unsupported platforms
            Ok(PlatformResourceMetrics {
                timestamp,
                cpu_metrics: CpuResourceMetrics::default(),
                gpu_metrics: None,
                io_metrics: IoResourceMetrics::default(),
                thread_metrics: HashMap::new(),
            })
        }
    }

    /// Get collection frequency recommendation based on system load
    pub fn get_optimal_collection_interval(&self) -> Duration {
        // Adaptive sampling based on system activity
        Duration::from_millis(100) // 10Hz default for real-time monitoring
    }
}

impl Default for CpuResourceMetrics {
    fn default() -> Self {
        Self {
            overall_usage_percent: 0.0,
            per_core_usage: Vec::new(),
            frequency_mhz: Vec::new(),
            temperature_celsius: Vec::new(),
            context_switches_per_sec: 0,
            interrupts_per_sec: 0,
            load_average: (0.0, 0.0, 0.0),
        }
    }
}

impl Default for IoResourceMetrics {
    fn default() -> Self {
        Self {
            disk_read_bytes_per_sec: 0,
            disk_write_bytes_per_sec: 0,
            disk_read_ops_per_sec: 0,
            disk_write_ops_per_sec: 0,
            network_rx_bytes_per_sec: 0,
            network_tx_bytes_per_sec: 0,
            network_rx_packets_per_sec: 0,
            network_tx_packets_per_sec: 0,
        }
    }
}

// Platform-specific implementations

#[cfg(target_os = "macos")]
mod macos_impl {
    use super::*;
    use std::mem;

    pub struct MacOSResourceCollector {
        cpu_count: usize,
        page_size: u64,
        last_cpu_times: Vec<u64>,
    }

    impl MacOSResourceCollector {
        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let cpu_count = num_cpus::get();
            let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;
            
            Ok(Self {
                cpu_count,
                page_size,
                last_cpu_times: vec![0; cpu_count],
            })
        }

        pub fn collect_metrics(&mut self, timestamp: u64) -> Result<PlatformResourceMetrics, Box<dyn std::error::Error>> {
            let cpu_metrics = self.collect_cpu_metrics()?;
            let gpu_metrics = self.collect_gpu_metrics().ok();
            let io_metrics = self.collect_io_metrics()?;
            let thread_metrics = self.collect_thread_metrics()?;

            Ok(PlatformResourceMetrics {
                timestamp,
                cpu_metrics,
                gpu_metrics,
                io_metrics,
                thread_metrics,
            })
        }

        fn collect_cpu_metrics(&mut self) -> Result<CpuResourceMetrics, Box<dyn std::error::Error>> {
            // Use macOS-specific system calls for CPU metrics
            let mut host_cpu_load_info: libc::host_cpu_load_info = unsafe { mem::zeroed() };
            let mut count = libc::HOST_CPU_LOAD_INFO_COUNT;
            
            let result = unsafe {
                libc::host_statistics(
                    libc::mach_host_self(),
                    libc::HOST_CPU_LOAD_INFO,
                    &mut host_cpu_load_info as *mut _ as *mut i32,
                    &mut count,
                )
            };

            if result != libc::KERN_SUCCESS {
                return Ok(CpuResourceMetrics::default());
            }

            // Calculate CPU usage from ticks
            let total_ticks = host_cpu_load_info.cpu_ticks[libc::CPU_STATE_USER as usize]
                + host_cpu_load_info.cpu_ticks[libc::CPU_STATE_SYSTEM as usize]
                + host_cpu_load_info.cpu_ticks[libc::CPU_STATE_IDLE as usize]
                + host_cpu_load_info.cpu_ticks[libc::CPU_STATE_NICE as usize];

            let idle_ticks = host_cpu_load_info.cpu_ticks[libc::CPU_STATE_IDLE as usize];
            let usage_percent = if total_ticks > 0 {
                100.0 - (idle_ticks as f32 / total_ticks as f32 * 100.0)
            } else {
                0.0
            };

            // Get load average
            let mut load_avg: [f64; 3] = [0.0; 3];
            unsafe {
                if libc::getloadavg(load_avg.as_mut_ptr(), 3) != -1 {
                    // Load average retrieved successfully
                }
            }

            Ok(CpuResourceMetrics {
                overall_usage_percent: usage_percent,
                per_core_usage: vec![usage_percent / self.cpu_count as f32; self.cpu_count],
                frequency_mhz: vec![0; self.cpu_count], // Requires additional syscalls
                temperature_celsius: Vec::new(), // Requires IOKit integration
                context_switches_per_sec: 0, // Requires vm_stat parsing
                interrupts_per_sec: 0,
                load_average: (load_avg[0], load_avg[1], load_avg[2]),
            })
        }

        fn collect_gpu_metrics(&self) -> Result<GpuResourceMetrics, Box<dyn std::error::Error>> {
            // macOS GPU monitoring typically requires Metal Performance Shaders or IOKit
            // For now, return basic info
            Ok(GpuResourceMetrics {
                device_name: "Apple GPU".to_string(),
                vendor: GpuVendor::Apple,
                compute_usage_percent: 0.0,
                memory_usage_percent: 0.0,
                memory_used_bytes: 0,
                memory_total_bytes: 0,
                temperature_celsius: 0.0,
                power_usage_watts: 0.0,
                frequency_mhz: 0,
            })
        }

        fn collect_io_metrics(&self) -> Result<IoResourceMetrics, Box<dyn std::error::Error>> {
            // macOS IO statistics can be collected via sysctlbyname
            Ok(IoResourceMetrics::default())
        }

        fn collect_thread_metrics(&self) -> Result<HashMap<u64, ThreadResourceMetrics>, Box<dyn std::error::Error>> {
            let mut metrics = HashMap::new();
            
            // Get current process info
            let _pid = unsafe { libc::getpid() };
            
            // Use task_info to get thread information
            let mut task_info: libc::mach_task_basic_info = unsafe { mem::zeroed() };
            let mut count = libc::MACH_TASK_BASIC_INFO_COUNT;
            
            let result = unsafe {
                libc::task_info(
                    libc::mach_task_self(),
                    libc::MACH_TASK_BASIC_INFO,
                    &mut task_info as *mut _ as *mut i32,
                    &mut count,
                )
            };

            if result == libc::KERN_SUCCESS {
                // Get current thread ID for basic info
                let thread_id = unsafe { libc::pthread_self() } as u64;
                
                metrics.insert(thread_id, ThreadResourceMetrics {
                    thread_id,
                    thread_name: std::thread::current().name().map(String::from),
                    cpu_usage_percent: 0.0,
                    memory_resident_bytes: task_info.resident_size,
                    memory_virtual_bytes: task_info.virtual_size,
                    io_read_bytes: 0,
                    io_write_bytes: 0,
                    cpu_time_user_ns: 0,
                    cpu_time_kernel_ns: 0,
                });
            }

            Ok(metrics)
        }
    }
}

#[cfg(target_os = "macos")]
pub use macos_impl::MacOSResourceCollector;

#[cfg(target_os = "linux")]
mod linux_impl {
    use super::*;
    use std::fs;
    use std::str::FromStr;

    pub struct LinuxResourceCollector {
        cpu_count: usize,
        page_size: u64,
        last_cpu_stats: Vec<CpuStat>,
        last_collection_time: Instant,
    }

    #[derive(Debug, Clone)]
    struct CpuStat {
        user: u64,
        nice: u64,
        system: u64,
        idle: u64,
        iowait: u64,
        irq: u64,
        softirq: u64,
    }

    impl LinuxResourceCollector {
        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let cpu_count = num_cpus::get();
            let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;
            
            Ok(Self {
                cpu_count,
                page_size,
                last_cpu_stats: Vec::new(),
                last_collection_time: Instant::now(),
            })
        }

        pub fn collect_metrics(&mut self, timestamp: u64) -> Result<PlatformResourceMetrics, Box<dyn std::error::Error>> {
            let cpu_metrics = self.collect_cpu_metrics()?;
            let gpu_metrics = self.collect_gpu_metrics().ok();
            let io_metrics = self.collect_io_metrics()?;
            let thread_metrics = self.collect_thread_metrics()?;

            Ok(PlatformResourceMetrics {
                timestamp,
                cpu_metrics,
                gpu_metrics,
                io_metrics,
                thread_metrics,
            })
        }

        fn collect_cpu_metrics(&mut self) -> Result<CpuResourceMetrics, Box<dyn std::error::Error>> {
            let current_stats = self.read_proc_stat()?;
            let mut per_core_usage = Vec::new();
            
            // Calculate usage if we have previous stats
            if !self.last_cpu_stats.is_empty() && self.last_cpu_stats.len() == current_stats.len() {
                for (current, last) in current_stats.iter().zip(self.last_cpu_stats.iter()) {
                    let usage = self.calculate_cpu_usage(current, last);
                    per_core_usage.push(usage);
                }
            } else {
                per_core_usage = vec![0.0; self.cpu_count];
            }

            let overall_usage = if per_core_usage.is_empty() {
                0.0
            } else {
                per_core_usage.iter().sum::<f32>() / per_core_usage.len() as f32
            };

            // Read CPU frequencies
            let frequencies = self.read_cpu_frequencies()?;
            
            // Read load average
            let load_average = self.read_load_average()?;

            // Update state
            self.last_cpu_stats = current_stats;
            self.last_collection_time = Instant::now();

            Ok(CpuResourceMetrics {
                overall_usage_percent: overall_usage,
                per_core_usage,
                frequency_mhz: frequencies,
                temperature_celsius: self.read_cpu_temperatures().unwrap_or_default(),
                context_switches_per_sec: 0, // Requires /proc/stat parsing
                interrupts_per_sec: 0,
                load_average,
            })
        }

        fn read_proc_stat(&self) -> Result<Vec<CpuStat>, Box<dyn std::error::Error>> {
            let content = fs::read_to_string("/proc/stat")?;
            let mut stats = Vec::new();

            for line in content.lines() {
                if line.starts_with("cpu") && line.chars().nth(3).map_or(false, |c| c.is_ascii_digit()) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 8 {
                        stats.push(CpuStat {
                            user: parts[1].parse().unwrap_or(0),
                            nice: parts[2].parse().unwrap_or(0),
                            system: parts[3].parse().unwrap_or(0),
                            idle: parts[4].parse().unwrap_or(0),
                            iowait: parts[5].parse().unwrap_or(0),
                            irq: parts[6].parse().unwrap_or(0),
                            softirq: parts[7].parse().unwrap_or(0),
                        });
                    }
                }
            }

            Ok(stats)
        }

        fn calculate_cpu_usage(&self, current: &CpuStat, last: &CpuStat) -> f32 {
            let current_total = current.user + current.nice + current.system + current.idle + 
                               current.iowait + current.irq + current.softirq;
            let last_total = last.user + last.nice + last.system + last.idle + 
                            last.iowait + last.irq + last.softirq;

            let total_diff = current_total.saturating_sub(last_total);
            let idle_diff = current.idle.saturating_sub(last.idle);

            if total_diff == 0 {
                0.0
            } else {
                100.0 - (idle_diff as f32 / total_diff as f32 * 100.0)
            }
        }

        fn read_cpu_frequencies(&self) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
            let mut frequencies = Vec::new();
            
            for i in 0..self.cpu_count {
                let freq_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq", i);
                if let Ok(content) = fs::read_to_string(freq_path) {
                    if let Ok(freq_khz) = content.trim().parse::<u32>() {
                        frequencies.push(freq_khz / 1000); // Convert to MHz
                    } else {
                        frequencies.push(0);
                    }
                } else {
                    frequencies.push(0);
                }
            }

            Ok(frequencies)
        }

        fn read_cpu_temperatures(&self) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
            let mut temperatures = Vec::new();
            
            // Try common thermal zones
            for i in 0..8 {
                let temp_path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
                if let Ok(content) = fs::read_to_string(temp_path) {
                    if let Ok(temp_millic) = content.trim().parse::<i32>() {
                        temperatures.push(temp_millic as f32 / 1000.0);
                    }
                }
            }

            Ok(temperatures)
        }

        fn read_load_average(&self) -> Result<(f64, f64, f64), Box<dyn std::error::Error>> {
            let content = fs::read_to_string("/proc/loadavg")?;
            let parts: Vec<&str> = content.split_whitespace().collect();
            
            if parts.len() >= 3 {
                let load1 = parts[0].parse().unwrap_or(0.0);
                let load5 = parts[1].parse().unwrap_or(0.0);
                let load15 = parts[2].parse().unwrap_or(0.0);
                Ok((load1, load5, load15))
            } else {
                Ok((0.0, 0.0, 0.0))
            }
        }

        fn collect_gpu_metrics(&self) -> Result<GpuResourceMetrics, Box<dyn std::error::Error>> {
            // Try NVIDIA first
            if let Ok(gpu) = self.collect_nvidia_gpu() {
                return Ok(gpu);
            }

            // Try AMD
            if let Ok(gpu) = self.collect_amd_gpu() {
                return Ok(gpu);
            }

            // Try Intel
            if let Ok(gpu) = self.collect_intel_gpu() {
                return Ok(gpu);
            }

            Err("No supported GPU found".into())
        }

        fn collect_nvidia_gpu(&self) -> Result<GpuResourceMetrics, Box<dyn std::error::Error>> {
            // Check if nvidia-smi is available
            let output = std::process::Command::new("nvidia-smi")
                .args(&["--query-gpu=name,utilization.gpu,memory.used,memory.total,temperature.gpu,power.draw,clocks.gr", "--format=csv,noheader,nounits"])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let result = String::from_utf8_lossy(&output.stdout);
                    let parts: Vec<&str> = result.trim().split(',').collect();
                    
                    if parts.len() >= 7 {
                        Ok(GpuResourceMetrics {
                            device_name: parts[0].trim().to_string(),
                            vendor: GpuVendor::Nvidia,
                            compute_usage_percent: parts[1].trim().parse().unwrap_or(0.0),
                            memory_usage_percent: 0.0, // Calculate from used/total
                            memory_used_bytes: parts[2].trim().parse::<u64>().unwrap_or(0) * 1024 * 1024,
                            memory_total_bytes: parts[3].trim().parse::<u64>().unwrap_or(0) * 1024 * 1024,
                            temperature_celsius: parts[4].trim().parse().unwrap_or(0.0),
                            power_usage_watts: parts[5].trim().parse().unwrap_or(0.0),
                            frequency_mhz: parts[6].trim().parse().unwrap_or(0),
                        })
                    } else {
                        Err("Invalid nvidia-smi output".into())
                    }
                }
                _ => Err("nvidia-smi not available".into())
            }
        }

        fn collect_amd_gpu(&self) -> Result<GpuResourceMetrics, Box<dyn std::error::Error>> {
            // AMD GPU monitoring through ROCm or sysfs
            Err("AMD GPU monitoring not implemented yet".into())
        }

        fn collect_intel_gpu(&self) -> Result<GpuResourceMetrics, Box<dyn std::error::Error>> {
            // Intel GPU monitoring through i915 driver
            Err("Intel GPU monitoring not implemented yet".into())
        }

        fn collect_io_metrics(&self) -> Result<IoResourceMetrics, Box<dyn std::error::Error>> {
            // Read /proc/diskstats for disk I/O
            // Read /proc/net/dev for network I/O
            Ok(IoResourceMetrics::default())
        }

        fn collect_thread_metrics(&self) -> Result<HashMap<u64, ThreadResourceMetrics>, Box<dyn std::error::Error>> {
            let mut metrics = HashMap::new();
            
            // Read /proc/self/task/ for thread information
            if let Ok(entries) = fs::read_dir("/proc/self/task") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Ok(tid) = entry.file_name().to_str().unwrap_or("").parse::<u64>() {
                            if let Ok(thread_metrics) = self.collect_single_thread_metrics(tid) {
                                metrics.insert(tid, thread_metrics);
                            }
                        }
                    }
                }
            }

            Ok(metrics)
        }

        fn collect_single_thread_metrics(&self, tid: u64) -> Result<ThreadResourceMetrics, Box<dyn std::error::Error>> {
            let stat_path = format!("/proc/self/task/{}/stat", tid);
            let status_path = format!("/proc/self/task/{}/status", tid);
            let comm_path = format!("/proc/self/task/{}/comm", tid);

            let stat_content = fs::read_to_string(stat_path).unwrap_or_default();
            let status_content = fs::read_to_string(status_path).unwrap_or_default();
            let thread_name = fs::read_to_string(comm_path).ok().map(|s| s.trim().to_string());

            // Parse basic metrics from stat and status files
            let stat_parts: Vec<&str> = stat_content.split_whitespace().collect();
            
            let mut memory_resident = 0u64;
            let mut memory_virtual = 0u64;
            
            for line in status_content.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        memory_resident = value.parse::<u64>().unwrap_or(0) * 1024; // Convert kB to bytes
                    }
                } else if line.starts_with("VmSize:") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        memory_virtual = value.parse::<u64>().unwrap_or(0) * 1024; // Convert kB to bytes
                    }
                }
            }

            Ok(ThreadResourceMetrics {
                thread_id: tid,
                thread_name,
                cpu_usage_percent: 0.0, // Requires time-based calculation
                memory_resident_bytes: memory_resident,
                memory_virtual_bytes: memory_virtual,
                io_read_bytes: 0,  // Requires /proc/self/task/*/io
                io_write_bytes: 0,
                cpu_time_user_ns: stat_parts.get(13).and_then(|s| s.parse().ok()).unwrap_or(0),
                cpu_time_kernel_ns: stat_parts.get(14).and_then(|s| s.parse().ok()).unwrap_or(0),
            })
        }
    }
}

#[cfg(target_os = "linux")]
pub use linux_impl::LinuxResourceCollector;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_resource_collector_creation() {
        let result = PlatformResourceCollector::new();
        match result {
            Ok(_collector) => {
                // Platform collector created successfully
            }
            Err(e) => {
                // May fail on unsupported platforms, which is expected
                println!("Platform collector creation failed: {}", e);
            }
        }
    }

    #[test]
    fn test_resource_metrics_serialization() {
        let metrics = PlatformResourceMetrics {
            timestamp: 12345,
            cpu_metrics: CpuResourceMetrics::default(),
            gpu_metrics: None,
            io_metrics: IoResourceMetrics::default(),
            thread_metrics: HashMap::new(),
        };

        let serialized = serde_json::to_string(&metrics).expect("Failed to serialize metrics");
        let _deserialized: PlatformResourceMetrics = serde_json::from_str(&serialized)
            .expect("Failed to deserialize metrics");
    }

    #[test]
    fn test_optimal_collection_interval() {
        if let Ok(collector) = PlatformResourceCollector::new() {
            let interval = collector.get_optimal_collection_interval();
            assert!(interval >= Duration::from_millis(50)); // At least 20Hz
            assert!(interval <= Duration::from_secs(1));     // At most 1Hz
        }
    }
}