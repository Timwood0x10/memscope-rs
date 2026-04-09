//! Global System Monitor - Background Thread Collection
//!
//! Architecture:
//! - Background thread collects system metrics every 100ms
//! - Atomic variables store current values (lock-free reads)
//! - `track!` only reads atomic values (nanosecond overhead)
//! - No blocking on data collection
//!
//! Features:
//! - CPU monitoring
//! - Memory monitoring
//! - I/O monitoring (basic)
//! - GPU monitoring (platform-specific)
//! - Correlation analysis
//! - Performance scoring

#![allow(warnings, unused)]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use sysinfo::System;

static SYSTEM_MONITOR: std::sync::OnceLock<SystemMonitor> = std::sync::OnceLock::new();

pub struct SystemMonitor {
    cpu_usage: Arc<AtomicU64>,
    memory_available: Arc<AtomicU64>,
    memory_total: Arc<AtomicU64>,
    disk_read_bps: Arc<AtomicU64>,
    disk_write_bps: Arc<AtomicU64>,
    network_rx_bps: Arc<AtomicU64>,
    network_tx_bps: Arc<AtomicU64>,
    gpu_usage: Arc<AtomicU64>,
    gpu_memory_used: Arc<AtomicU64>,
    gpu_memory_total: Arc<AtomicU64>,
    last_update: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl SystemMonitor {
    fn new() -> Self {
        let cpu_usage = Arc::new(AtomicU64::new(0));
        let memory_available = Arc::new(AtomicU64::new(0));
        let memory_total = Arc::new(AtomicU64::new(0));
        let disk_read_bps = Arc::new(AtomicU64::new(0));
        let disk_write_bps = Arc::new(AtomicU64::new(0));
        let network_rx_bps = Arc::new(AtomicU64::new(0));
        let network_tx_bps = Arc::new(AtomicU64::new(0));
        let gpu_usage = Arc::new(AtomicU64::new(0));
        let gpu_memory_used = Arc::new(AtomicU64::new(0));
        let gpu_memory_total = Arc::new(AtomicU64::new(0));
        let last_update = Arc::new(AtomicU64::new(0));
        let running = Arc::new(AtomicBool::new(true));

        let cpu_usage_clone = cpu_usage.clone();
        let memory_available_clone = memory_available.clone();
        let memory_total_clone = memory_total.clone();
        let disk_read_bps_clone = disk_read_bps.clone();
        let disk_write_bps_clone = disk_write_bps.clone();
        let network_rx_bps_clone = network_rx_bps.clone();
        let network_tx_bps_clone = network_tx_bps.clone();
        let gpu_usage_clone = gpu_usage.clone();
        let gpu_memory_used_clone = gpu_memory_used.clone();
        let gpu_memory_total_clone = gpu_memory_total.clone();
        let last_update_clone = last_update.clone();
        let running_clone = running.clone();

        let handle = thread::spawn(move || {
            let mut sys = System::new_all();
            sys.refresh_all();

            let mut last_refresh = Instant::now();
            let last_disk_read = 0u64;
            let last_disk_write = 0u64;
            let last_network_rx = 0u64;
            let last_network_tx = 0u64;

            while running_clone.load(Ordering::Relaxed) {
                let now = Instant::now();

                if now.duration_since(last_refresh).as_millis() >= 100 {
                    sys.refresh_cpu_all();
                    sys.refresh_memory();

                    let cpus = sys.cpus();
                    if !cpus.is_empty() {
                        let total: f64 = cpus.iter().map(|c| c.cpu_usage() as f64).sum();
                        let avg = (total / cpus.len() as f64).min(100.0);
                        cpu_usage_clone.store(avg.to_bits(), Ordering::Release);
                    }

                    memory_available_clone.store(sys.available_memory(), Ordering::Release);
                    memory_total_clone.store(sys.total_memory(), Ordering::Release);

                    #[cfg(target_os = "linux")]
                    {
                        if let Ok(io_stats) = collect_io_stats() {
                            let elapsed_sec = now.duration_since(last_refresh).as_secs_f64();
                            if elapsed_sec > 0.0 {
                                let read_bps = ((io_stats.read_bytes - last_disk_read) as f64
                                    / elapsed_sec)
                                    as u64;
                                let write_bps = ((io_stats.write_bytes - last_disk_write) as f64
                                    / elapsed_sec)
                                    as u64;
                                disk_read_bps_clone.store(read_bps, Ordering::Release);
                                disk_write_bps_clone.store(write_bps, Ordering::Release);
                            }
                            last_disk_read = io_stats.read_bytes;
                            last_disk_write = io_stats.write_bytes;

                            if let Ok(net_stats) = collect_network_stats() {
                                let rx_bps = ((net_stats.rx_bytes - last_network_rx) as f64
                                    / elapsed_sec)
                                    as u64;
                                let tx_bps = ((net_stats.tx_bytes - last_network_tx) as f64
                                    / elapsed_sec)
                                    as u64;
                                network_rx_bps_clone.store(rx_bps, Ordering::Release);
                                network_tx_bps_clone.store(tx_bps, Ordering::Release);
                                last_network_rx = net_stats.rx_bytes;
                                last_network_tx = net_stats.tx_bytes;
                            }
                        }
                    }

                    #[cfg(target_os = "linux")]
                    {
                        if let Ok(gpu_info) = collect_nvidia_gpu() {
                            gpu_usage_clone.store(gpu_info.usage.to_bits(), Ordering::Release);
                            gpu_memory_used_clone.store(gpu_info.memory_used, Ordering::Release);
                            gpu_memory_total_clone.store(gpu_info.memory_total, Ordering::Release);
                        }
                    }

                    last_update_clone.store(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0),
                        Ordering::Release,
                    );

                    last_refresh = now;
                }

                thread::sleep(Duration::from_millis(50));
            }
        });

        Self {
            cpu_usage,
            memory_available,
            memory_total,
            disk_read_bps,
            disk_write_bps,
            network_rx_bps,
            network_tx_bps,
            gpu_usage,
            gpu_memory_used,
            gpu_memory_total,
            last_update,
            running,
            handle: Mutex::new(Some(handle)),
        }
    }

    pub fn global() -> &'static Self {
        SYSTEM_MONITOR.get_or_init(Self::new)
    }

    pub fn shutdown() {
        if let Some(monitor) = SYSTEM_MONITOR.get() {
            monitor.running.store(false, Ordering::Release);
            if let Ok(mut handle_guard) = monitor.handle.lock() {
                if let Some(handle) = handle_guard.take() {
                    let _ = handle.join();
                }
            }
        }
    }

    #[inline]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    #[inline]
    pub fn cpu_usage(&self) -> f64 {
        let bits = self.cpu_usage.load(Ordering::Acquire);
        let value = f64::from_bits(bits);
        if value.is_nan() || value < 0.0 {
            0.0
        } else {
            value.min(100.0)
        }
    }

    #[inline]
    pub fn memory_available(&self) -> u64 {
        self.memory_available.load(Ordering::Acquire)
    }

    #[inline]
    pub fn memory_total(&self) -> u64 {
        self.memory_total.load(Ordering::Acquire)
    }

    #[inline]
    pub fn memory_used(&self) -> u64 {
        let total = self.memory_total.load(Ordering::Acquire);
        let available = self.memory_available.load(Ordering::Acquire);
        total.saturating_sub(available)
    }

    #[inline]
    pub fn memory_usage_percent(&self) -> f64 {
        let total = self.memory_total.load(Ordering::Acquire);
        let available = self.memory_available.load(Ordering::Acquire);
        if total > 0 {
            ((total - available) as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    #[inline]
    pub fn disk_read_bps(&self) -> u64 {
        self.disk_read_bps.load(Ordering::Acquire)
    }

    #[inline]
    pub fn disk_write_bps(&self) -> u64 {
        self.disk_write_bps.load(Ordering::Acquire)
    }

    #[inline]
    pub fn network_rx_bps(&self) -> u64 {
        self.network_rx_bps.load(Ordering::Acquire)
    }

    #[inline]
    pub fn network_tx_bps(&self) -> u64 {
        self.network_tx_bps.load(Ordering::Acquire)
    }

    #[inline]
    pub fn gpu_usage(&self) -> f64 {
        let bits = self.gpu_usage.load(Ordering::Acquire);
        f64::from_bits(bits)
    }

    #[inline]
    pub fn gpu_memory_used(&self) -> u64 {
        self.gpu_memory_used.load(Ordering::Acquire)
    }

    #[inline]
    pub fn gpu_memory_total(&self) -> u64 {
        self.gpu_memory_total.load(Ordering::Acquire)
    }

    #[inline]
    pub fn gpu_memory_usage_percent(&self) -> f64 {
        let total = self.gpu_memory_total.load(Ordering::Acquire);
        let used = self.gpu_memory_used.load(Ordering::Acquire);
        if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    #[inline]
    pub fn last_update(&self) -> u64 {
        self.last_update.load(Ordering::Acquire)
    }

    #[inline]
    pub fn thread_count(&self) -> usize {
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1)
    }

    pub fn correlation_analysis(&self) -> CorrelationAnalysis {
        let cpu = self.cpu_usage();
        let mem_percent = self.memory_usage_percent();
        let disk_read = self.disk_read_bps();
        let disk_write = self.disk_write_bps();
        let network_rx = self.network_rx_bps();
        let network_tx = self.network_tx_bps();
        let gpu = self.gpu_usage();

        let cpu_mem_correlation = if cpu > 0.0 && mem_percent > 0.0 {
            (cpu / mem_percent).min(2.0)
        } else {
            0.0
        };

        let io_intensity = (disk_read + disk_write) as f64 / 1024.0 / 1024.0;
        let network_intensity = (network_rx + network_tx) as f64 / 1024.0 / 1024.0;

        CorrelationAnalysis {
            cpu_memory_correlation: cpu_mem_correlation,
            io_intensity_mb_per_sec: io_intensity,
            network_intensity_mb_per_sec: network_intensity,
            gpu_cpu_ratio: if cpu > 0.0 { gpu / cpu } else { 0.0 },
            system_load_score: (cpu + mem_percent) / 2.0,
        }
    }

    pub fn performance_score(&self) -> PerformanceScore {
        let cpu = self.cpu_usage();
        let mem_percent = self.memory_usage_percent();
        let disk_read = self.disk_read_bps();
        let disk_write = self.disk_write_bps();
        let network_rx = self.network_rx_bps();
        let network_tx = self.network_tx_bps();

        let cpu_efficiency = if cpu < 80.0 {
            (100.0 - cpu) / 100.0
        } else {
            (100.0 - cpu) / 100.0 * 0.5
        };

        let memory_efficiency = if mem_percent < 80.0 {
            (100.0 - mem_percent) / 100.0
        } else {
            (100.0 - mem_percent) / 100.0 * 0.5
        };

        let io_throughput = (disk_read + disk_write) as f64 / 1024.0 / 1024.0 / 1024.0;
        let io_efficiency = (io_throughput / 100.0).min(1.0);

        let network_throughput = (network_rx + network_tx) as f64 / 1024.0 / 1024.0 / 1024.0;
        let network_efficiency = (network_throughput / 10.0).min(1.0);

        let overall_score = (cpu_efficiency * 0.4
            + memory_efficiency * 0.3
            + io_efficiency * 0.2
            + network_efficiency * 0.1)
            * 100.0;

        PerformanceScore {
            cpu_efficiency: cpu_efficiency * 100.0,
            memory_efficiency: memory_efficiency * 100.0,
            io_efficiency: io_efficiency * 100.0,
            network_efficiency: network_efficiency * 100.0,
            overall_score,
        }
    }
}

impl Drop for SystemMonitor {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);

        if let Ok(mut handle_guard) = self.handle.lock() {
            if let Some(handle) = handle_guard.take() {
                std::thread::spawn(move || {
                    let timeout = std::time::Duration::from_secs(2);
                    let start = std::time::Instant::now();

                    while start.elapsed() < timeout {
                        if handle.is_finished() {
                            let _ = handle.join();
                            return;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                });
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CorrelationAnalysis {
    pub cpu_memory_correlation: f64,
    pub io_intensity_mb_per_sec: f64,
    pub network_intensity_mb_per_sec: f64,
    pub gpu_cpu_ratio: f64,
    pub system_load_score: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceScore {
    pub cpu_efficiency: f64,
    pub memory_efficiency: f64,
    pub io_efficiency: f64,
    pub network_efficiency: f64,
    pub overall_score: f64,
}

#[cfg(target_os = "linux")]
struct IoStats {
    read_bytes: u64,
    write_bytes: u64,
}

#[cfg(target_os = "linux")]
struct NetworkStats {
    rx_bytes: u64,
    tx_bytes: u64,
}

#[cfg(target_os = "linux")]
fn collect_io_stats() -> Result<IoStats, Box<dyn std::error::Error>> {
    use std::fs;

    let content = fs::read_to_string("/proc/diskstats")?;
    let mut total_read = 0u64;
    let mut total_write = 0u64;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            if let Ok(read) = parts[5].parse::<u64>() {
                total_read += read * 512;
            }
            if let Ok(write) = parts[9].parse::<u64>() {
                total_write += write * 512;
            }
        }
    }

    Ok(IoStats {
        read_bytes: total_read,
        write_bytes: total_write,
    })
}

#[cfg(target_os = "linux")]
fn collect_network_stats() -> Result<NetworkStats, Box<dyn std::error::Error>> {
    use std::fs;

    let content = fs::read_to_string("/proc/net/dev")?;
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;

    for line in content.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 10 {
            if let Ok(rx) = parts[1].parse::<u64>() {
                total_rx += rx;
            }
            if let Ok(tx) = parts[9].parse::<u64>() {
                total_tx += tx;
            }
        }
    }

    Ok(NetworkStats {
        rx_bytes: total_rx,
        tx_bytes: total_tx,
    })
}

#[cfg(target_os = "linux")]
fn collect_nvidia_gpu() -> Result<GpuInfo, Box<dyn std::error::Error>> {
    use std::process::Command;

    let output = Command::new("nvidia-smi")
        .args(&[
            "--query-gpu=utilization.gpu,memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = stdout.trim().split(',').collect();
            if parts.len() >= 3 {
                let usage = parts[0].trim().parse::<f64>()?;
                let memory_used = parts[1].trim().parse::<u64>()? * 1024 * 1024;
                let memory_total = parts[2].trim().parse::<u64>()? * 1024 * 1024;
                return Ok(GpuInfo {
                    usage,
                    memory_used,
                    memory_total,
                });
            }
        }
    }

    Err("Failed to collect GPU info".into())
}

#[cfg(target_os = "linux")]
struct GpuInfo {
    usage: f64,
    memory_used: u64,
    memory_total: u64,
}

pub fn cpu_usage() -> f64 {
    SystemMonitor::global().cpu_usage()
}

pub fn memory_available() -> u64 {
    SystemMonitor::global().memory_available()
}

pub fn memory_total() -> u64 {
    SystemMonitor::global().memory_total()
}

pub fn memory_used() -> u64 {
    SystemMonitor::global().memory_used()
}

pub fn memory_usage_percent() -> f64 {
    SystemMonitor::global().memory_usage_percent()
}

pub fn thread_count() -> usize {
    SystemMonitor::global().thread_count()
}

pub fn disk_read_bps() -> u64 {
    SystemMonitor::global().disk_read_bps()
}

pub fn disk_write_bps() -> u64 {
    SystemMonitor::global().disk_write_bps()
}

pub fn network_rx_bps() -> u64 {
    SystemMonitor::global().network_rx_bps()
}

pub fn network_tx_bps() -> u64 {
    SystemMonitor::global().network_tx_bps()
}

pub fn gpu_usage() -> f64 {
    SystemMonitor::global().gpu_usage()
}

pub fn gpu_memory_used() -> u64 {
    SystemMonitor::global().gpu_memory_used()
}

pub fn gpu_memory_total() -> u64 {
    SystemMonitor::global().gpu_memory_total()
}

pub fn gpu_memory_usage_percent() -> f64 {
    SystemMonitor::global().gpu_memory_usage_percent()
}

pub fn correlation_analysis() -> CorrelationAnalysis {
    SystemMonitor::global().correlation_analysis()
}

pub fn performance_score() -> PerformanceScore {
    SystemMonitor::global().performance_score()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_monitor() {
        let monitor = SystemMonitor::global();

        thread::sleep(Duration::from_millis(200));

        let cpu = monitor.cpu_usage();
        println!("CPU usage: {:.2}%", cpu);
        assert!((0.0..=100.0).contains(&cpu));

        let mem = monitor.memory_used();
        println!("Memory used: {} bytes", mem);

        let total = monitor.memory_total();
        println!("Memory total: {} bytes", total);
    }

    #[test]
    fn test_io_monitoring() {
        thread::sleep(Duration::from_millis(200));

        let disk_read = disk_read_bps();
        let disk_write = disk_write_bps();
        let network_rx = network_rx_bps();
        let network_tx = network_tx_bps();

        println!("Disk I/O: {} read/s, {} write/s", disk_read, disk_write);
        println!("Network: {} rx/s, {} tx/s", network_rx, network_tx);
    }

    #[test]
    fn test_correlation_analysis() {
        thread::sleep(Duration::from_millis(200));

        let analysis = correlation_analysis();
        println!(
            "CPU-Memory correlation: {:.2}",
            analysis.cpu_memory_correlation
        );
        println!(
            "I/O intensity: {:.2} MB/s",
            analysis.io_intensity_mb_per_sec
        );
        println!(
            "Network intensity: {:.2} MB/s",
            analysis.network_intensity_mb_per_sec
        );
        println!("System load score: {:.2}", analysis.system_load_score);
    }

    #[test]
    fn test_performance_score() {
        thread::sleep(Duration::from_millis(200));

        let score = performance_score();
        println!("CPU efficiency: {:.2}%", score.cpu_efficiency);
        println!("Memory efficiency: {:.2}%", score.memory_efficiency);
        println!("I/O efficiency: {:.2}%", score.io_efficiency);
        println!("Network efficiency: {:.2}%", score.network_efficiency);
        println!("Overall score: {:.2}%", score.overall_score);
    }

    #[test]
    fn test_convenience_functions() {
        thread::sleep(Duration::from_millis(150));

        let cpu = cpu_usage();
        let mem = memory_used();
        let total = memory_total();
        let percent = memory_usage_percent();
        let threads = thread_count();

        println!("CPU: {:.2}%", cpu);
        println!("Memory: {} / {} ({:.2}%)", mem, total, percent);
        println!("Threads: {}", threads);
    }
}
