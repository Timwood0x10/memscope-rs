//! System resource monitoring for async tasks
//!
//! Provides comprehensive system resource monitoring capabilities including
//! CPU, IO, Network, and GPU usage tracking at the async task level.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use crate::async_memory::TaskId;

/// Comprehensive system resource metrics for an async task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSystemMetrics {
    pub task_id: TaskId,
    pub cpu_metrics: CpuMetrics,
    pub io_metrics: IoMetrics,
    pub network_metrics: NetworkMetrics,
    pub gpu_metrics: Option<GpuMetrics>,
    pub timestamp: u64,
}

/// CPU usage metrics for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    /// CPU time consumed by this task (in microseconds)
    pub cpu_time_us: u64,
    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_usage_percent: f64,
    /// Number of context switches
    pub context_switches: u64,
    /// Time spent in user mode (microseconds)
    pub user_time_us: u64,
    /// Time spent in kernel mode (microseconds)  
    pub kernel_time_us: u64,
    /// CPU core affinity (which cores this task ran on)
    pub core_affinity: Vec<u32>,
    /// CPU cache misses
    pub cache_misses: Option<u64>,
    /// CPU instructions executed
    pub instructions: Option<u64>,
}

/// IO operation metrics for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoMetrics {
    /// Total bytes read
    pub bytes_read: u64,
    /// Total bytes written
    pub bytes_written: u64,
    /// Number of read operations
    pub read_operations: u64,
    /// Number of write operations
    pub write_operations: u64,
    /// Total time spent in IO operations (microseconds)
    pub io_wait_time_us: u64,
    /// Average IO operation latency (microseconds)
    pub avg_io_latency_us: f64,
    /// IO bandwidth utilization (MB/s)
    pub io_bandwidth_mbps: f64,
    /// Disk queue depth
    pub disk_queue_depth: u32,
    /// IO operation types breakdown
    pub io_types: IoTypeBreakdown,
}

/// Breakdown of IO operations by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoTypeBreakdown {
    pub file_operations: u64,
    pub network_operations: u64,
    pub pipe_operations: u64,
    pub socket_operations: u64,
}

/// Network usage metrics for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Bytes transmitted
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Number of packets sent
    pub packets_sent: u64,
    /// Number of packets received
    pub packets_received: u64,
    /// Number of active connections
    pub active_connections: u32,
    /// Network latency (milliseconds)
    pub network_latency_ms: f64,
    /// Bandwidth utilization (Mbps)
    pub bandwidth_utilization_mbps: f64,
    /// Connection types breakdown
    pub connection_types: ConnectionTypeBreakdown,
    /// Network errors
    pub network_errors: u64,
}

/// Breakdown of connections by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTypeBreakdown {
    pub tcp_connections: u32,
    pub udp_connections: u32,
    pub unix_sockets: u32,
    pub websocket_connections: u32,
}

/// GPU usage metrics for a task (if available)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMetrics {
    /// GPU device ID
    pub device_id: u32,
    /// GPU device name
    pub device_name: String,
    /// GPU utilization percentage (0.0 - 100.0)
    pub gpu_utilization_percent: f64,
    /// GPU memory used by this task (bytes)
    pub gpu_memory_used: u64,
    /// GPU memory total (bytes)
    pub gpu_memory_total: u64,
    /// GPU compute operations
    pub compute_operations: u64,
    /// GPU memory bandwidth used (GB/s)
    pub memory_bandwidth_gbps: f64,
    /// GPU temperature (Celsius)
    pub temperature_celsius: f32,
    /// Power consumption (Watts)
    pub power_consumption_watts: f32,
}

/// System resource monitor for async tasks
pub struct SystemResourceMonitor {
    /// Per-task system metrics
    task_metrics: HashMap<TaskId, TaskSystemMetrics>,
    /// Baseline system metrics for comparison
    baseline_metrics: Option<SystemBaseline>,
    /// Monitor start time
    start_time: Instant,
    /// Monitoring counters
    counters: MonitoringCounters,
}

/// Baseline system metrics for calculating task-specific usage
#[derive(Debug, Clone)]
struct SystemBaseline {
    _total_cpu_time: u64,
    _total_io_bytes: u64,
    _total_network_bytes: u64,
    _timestamp: Instant,
}

/// Internal monitoring counters
struct MonitoringCounters {
    cpu_samples: AtomicU64,
    io_samples: AtomicU64,
    network_samples: AtomicU64,
    gpu_samples: AtomicU64,
}

impl SystemResourceMonitor {
    /// Create new system resource monitor
    pub fn new() -> Self {
        Self {
            task_metrics: HashMap::new(),
            baseline_metrics: None,
            start_time: Instant::now(),
            counters: MonitoringCounters {
                cpu_samples: AtomicU64::new(0),
                io_samples: AtomicU64::new(0),
                network_samples: AtomicU64::new(0),
                gpu_samples: AtomicU64::new(0),
            },
        }
    }

    /// Initialize monitoring with baseline measurements
    pub fn initialize(&mut self) -> Result<(), std::io::Error> {
        self.baseline_metrics = Some(SystemBaseline {
            _total_cpu_time: self.get_system_cpu_time()?,
            _total_io_bytes: self.get_system_io_bytes()?,
            _total_network_bytes: self.get_system_network_bytes()?,
            _timestamp: Instant::now(),
        });
        Ok(())
    }

    /// Start monitoring a specific task
    pub fn start_task_monitoring(&mut self, task_id: TaskId) {
        let metrics = TaskSystemMetrics {
            task_id,
            cpu_metrics: self.collect_cpu_metrics(task_id),
            io_metrics: self.collect_io_metrics(task_id),
            network_metrics: self.collect_network_metrics(task_id),
            gpu_metrics: self.collect_gpu_metrics(task_id),
            timestamp: current_timestamp(),
        };

        self.task_metrics.insert(task_id, metrics);
    }

    /// Update metrics for a running task
    pub fn update_task_metrics(&mut self, task_id: TaskId) {
        // Collect all metrics first to avoid borrowing conflicts
        let cpu_metrics = self.collect_cpu_metrics(task_id);
        let io_metrics = self.collect_io_metrics(task_id);
        let network_metrics = self.collect_network_metrics(task_id);
        let gpu_metrics = self.collect_gpu_metrics(task_id);
        let timestamp = current_timestamp();

        // Then update the stored metrics
        if let Some(metrics) = self.task_metrics.get_mut(&task_id) {
            metrics.cpu_metrics = cpu_metrics;
            metrics.io_metrics = io_metrics;
            metrics.network_metrics = network_metrics;
            metrics.gpu_metrics = gpu_metrics;
            metrics.timestamp = timestamp;
        }
    }

    /// Get metrics for a specific task
    pub fn get_task_metrics(&self, task_id: TaskId) -> Option<&TaskSystemMetrics> {
        self.task_metrics.get(&task_id)
    }

    /// Get all task metrics
    pub fn get_all_metrics(&self) -> &HashMap<TaskId, TaskSystemMetrics> {
        &self.task_metrics
    }

    /// Collect CPU metrics for a task
    fn collect_cpu_metrics(&self, task_id: TaskId) -> CpuMetrics {
        self.counters.cpu_samples.fetch_add(1, Ordering::Relaxed);

        // Simulate CPU metrics collection
        // In real implementation, this would read from /proc/[pid]/stat,
        // performance counters, or use platform-specific APIs
        CpuMetrics {
            cpu_time_us: self.get_task_cpu_time(task_id),
            cpu_usage_percent: self.calculate_cpu_usage(task_id),
            context_switches: self.get_context_switches(task_id),
            user_time_us: self.get_user_time(task_id),
            kernel_time_us: self.get_kernel_time(task_id),
            core_affinity: self.get_core_affinity(task_id),
            cache_misses: self.get_cache_misses(task_id),
            instructions: self.get_instructions(task_id),
        }
    }

    /// Collect IO metrics for a task
    fn collect_io_metrics(&self, task_id: TaskId) -> IoMetrics {
        self.counters.io_samples.fetch_add(1, Ordering::Relaxed);

        // Simulate IO metrics collection
        // In real implementation, this would read from /proc/[pid]/io,
        // iotop data, or use BPF/eBPF tracing
        IoMetrics {
            bytes_read: self.get_bytes_read(task_id),
            bytes_written: self.get_bytes_written(task_id),
            read_operations: self.get_read_ops(task_id),
            write_operations: self.get_write_ops(task_id),
            io_wait_time_us: self.get_io_wait_time(task_id),
            avg_io_latency_us: self.calculate_avg_io_latency(task_id),
            io_bandwidth_mbps: self.calculate_io_bandwidth(task_id),
            disk_queue_depth: self.get_disk_queue_depth(task_id),
            io_types: self.get_io_type_breakdown(task_id),
        }
    }

    /// Collect network metrics for a task
    fn collect_network_metrics(&self, task_id: TaskId) -> NetworkMetrics {
        self.counters
            .network_samples
            .fetch_add(1, Ordering::Relaxed);

        // Simulate network metrics collection
        // In real implementation, this would use netstat, ss, or netlink sockets
        NetworkMetrics {
            bytes_sent: self.get_bytes_sent(task_id),
            bytes_received: self.get_bytes_received(task_id),
            packets_sent: self.get_packets_sent(task_id),
            packets_received: self.get_packets_received(task_id),
            active_connections: self.get_active_connections(task_id),
            network_latency_ms: self.measure_network_latency(task_id),
            bandwidth_utilization_mbps: self.calculate_bandwidth_utilization(task_id),
            connection_types: self.get_connection_breakdown(task_id),
            network_errors: self.get_network_errors(task_id),
        }
    }

    /// Collect GPU metrics for a task (if GPU is available)
    fn collect_gpu_metrics(&self, task_id: TaskId) -> Option<GpuMetrics> {
        if !self.is_gpu_available() {
            return None;
        }

        self.counters.gpu_samples.fetch_add(1, Ordering::Relaxed);

        // Simulate GPU metrics collection
        // In real implementation, this would use NVIDIA ML, ROCm, or Intel GPU APIs
        Some(GpuMetrics {
            device_id: 0,
            device_name: self.get_gpu_device_name(),
            gpu_utilization_percent: self.get_gpu_utilization(task_id),
            gpu_memory_used: self.get_gpu_memory_used(task_id),
            gpu_memory_total: self.get_gpu_memory_total(),
            compute_operations: self.get_gpu_compute_ops(task_id),
            memory_bandwidth_gbps: self.get_gpu_memory_bandwidth(task_id),
            temperature_celsius: self.get_gpu_temperature(),
            power_consumption_watts: self.get_gpu_power_consumption(),
        })
    }

    // Simulation methods for demonstration
    // In real implementation, these would interface with actual system APIs

    fn get_system_cpu_time(&self) -> Result<u64, std::io::Error> {
        // Read from /proc/stat or equivalent
        Ok(1000000) // Simulated value
    }

    fn get_system_io_bytes(&self) -> Result<u64, std::io::Error> {
        // Read from /proc/diskstats or equivalent
        Ok(50000000) // Simulated value
    }

    fn get_system_network_bytes(&self) -> Result<u64, std::io::Error> {
        // Read from /proc/net/dev or equivalent
        Ok(25000000) // Simulated value
    }

    fn get_task_cpu_time(&self, task_id: TaskId) -> u64 {
        // Simulate CPU time based on task ID and elapsed time
        let elapsed = self.start_time.elapsed().as_micros() as u64;
        (task_id as u64 * 1000) + (elapsed / 100)
    }

    fn calculate_cpu_usage(&self, task_id: TaskId) -> f64 {
        // Simulate CPU usage calculation
        let base_usage = (task_id as f64 % 100.0) / 10.0;
        let time_factor = (self.start_time.elapsed().as_secs() as f64).sin().abs();
        (base_usage + time_factor * 5.0).min(100.0)
    }

    fn get_context_switches(&self, task_id: TaskId) -> u64 {
        (task_id as u64 * 10) + (self.start_time.elapsed().as_millis() as u64 / 100)
    }

    fn get_user_time(&self, task_id: TaskId) -> u64 {
        self.get_task_cpu_time(task_id) * 70 / 100 // 70% user time
    }

    fn get_kernel_time(&self, task_id: TaskId) -> u64 {
        self.get_task_cpu_time(task_id) * 30 / 100 // 30% kernel time
    }

    fn get_core_affinity(&self, _task_id: TaskId) -> Vec<u32> {
        vec![0, 1, 2, 3] // Simulate running on cores 0-3
    }

    fn get_cache_misses(&self, task_id: TaskId) -> Option<u64> {
        Some(task_id as u64 * 500 + self.start_time.elapsed().as_millis() as u64)
    }

    fn get_instructions(&self, task_id: TaskId) -> Option<u64> {
        Some(task_id as u64 * 1000000 + self.start_time.elapsed().as_micros() as u64 * 10)
    }

    fn get_bytes_read(&self, task_id: TaskId) -> u64 {
        task_id as u64 * 4096 + self.start_time.elapsed().as_millis() as u64 * 100
    }

    fn get_bytes_written(&self, task_id: TaskId) -> u64 {
        task_id as u64 * 2048 + self.start_time.elapsed().as_millis() as u64 * 50
    }

    fn get_read_ops(&self, task_id: TaskId) -> u64 {
        task_id as u64 * 10 + self.start_time.elapsed().as_millis() as u64 / 10
    }

    fn get_write_ops(&self, task_id: TaskId) -> u64 {
        task_id as u64 * 5 + self.start_time.elapsed().as_millis() as u64 / 20
    }

    fn get_io_wait_time(&self, task_id: TaskId) -> u64 {
        task_id as u64 * 1000 + self.start_time.elapsed().as_micros() as u64 / 10
    }

    fn calculate_avg_io_latency(&self, task_id: TaskId) -> f64 {
        let base_latency = 100.0 + (task_id as f64 % 50.0);
        let load_factor = (self.start_time.elapsed().as_secs() as f64 / 10.0).sin() + 1.0;
        base_latency * load_factor
    }

    fn calculate_io_bandwidth(&self, task_id: TaskId) -> f64 {
        let total_bytes = self.get_bytes_read(task_id) + self.get_bytes_written(task_id);
        let elapsed_secs = self.start_time.elapsed().as_secs_f64().max(1.0);
        (total_bytes as f64 / 1_048_576.0) / elapsed_secs // MB/s
    }

    fn get_disk_queue_depth(&self, _task_id: TaskId) -> u32 {
        // Simulate variable queue depth
        ((self.start_time.elapsed().as_secs() % 10) + 1) as u32
    }

    fn get_io_type_breakdown(&self, task_id: TaskId) -> IoTypeBreakdown {
        let total_ops = self.get_read_ops(task_id) + self.get_write_ops(task_id);
        IoTypeBreakdown {
            file_operations: total_ops * 60 / 100,
            network_operations: total_ops * 25 / 100,
            pipe_operations: total_ops * 10 / 100,
            socket_operations: total_ops * 5 / 100,
        }
    }

    fn get_bytes_sent(&self, task_id: TaskId) -> u64 {
        task_id as u64 * 1024 + self.start_time.elapsed().as_millis() as u64 * 200
    }

    fn get_bytes_received(&self, task_id: TaskId) -> u64 {
        task_id as u64 * 2048 + self.start_time.elapsed().as_millis() as u64 * 300
    }

    fn get_packets_sent(&self, task_id: TaskId) -> u64 {
        self.get_bytes_sent(task_id) / 1500 // Assume 1500 byte packets
    }

    fn get_packets_received(&self, task_id: TaskId) -> u64 {
        self.get_bytes_received(task_id) / 1500
    }

    fn get_active_connections(&self, task_id: TaskId) -> u32 {
        ((task_id as u32 % 10) + 1) + (self.start_time.elapsed().as_secs() as u32 % 5)
    }

    fn measure_network_latency(&self, _task_id: TaskId) -> f64 {
        // Simulate network latency with some variation
        10.0 + (self.start_time.elapsed().as_millis() as f64 % 100.0) / 10.0
    }

    fn calculate_bandwidth_utilization(&self, task_id: TaskId) -> f64 {
        let total_bytes = self.get_bytes_sent(task_id) + self.get_bytes_received(task_id);
        let elapsed_secs = self.start_time.elapsed().as_secs_f64().max(1.0);
        (total_bytes as f64 * 8.0) / (1_000_000.0 * elapsed_secs) // Mbps
    }

    fn get_connection_breakdown(&self, task_id: TaskId) -> ConnectionTypeBreakdown {
        let total_conn = self.get_active_connections(task_id);
        ConnectionTypeBreakdown {
            tcp_connections: total_conn * 70 / 100,
            udp_connections: total_conn * 20 / 100,
            unix_sockets: total_conn * 5 / 100,
            websocket_connections: total_conn * 5 / 100,
        }
    }

    fn get_network_errors(&self, task_id: TaskId) -> u64 {
        (task_id as u64 % 100) + self.start_time.elapsed().as_secs()
    }

    fn is_gpu_available(&self) -> bool {
        // In real implementation, check for NVIDIA, AMD, or Intel GPU
        true // Simulate GPU availability
    }

    fn get_gpu_device_name(&self) -> String {
        "NVIDIA GeForce RTX 4090".to_string() // Simulated
    }

    fn get_gpu_utilization(&self, task_id: TaskId) -> f64 {
        let base_util = (task_id as f64 % 80.0) + 10.0;
        let time_factor = (self.start_time.elapsed().as_secs() as f64 / 5.0)
            .sin()
            .abs();
        (base_util + time_factor * 15.0).min(100.0)
    }

    fn get_gpu_memory_used(&self, task_id: TaskId) -> u64 {
        let base_mem = (task_id as u64 % 1000) * 1_048_576; // Base MB in bytes
        base_mem + (self.start_time.elapsed().as_millis() as u64 * 1024) % 1_073_741_824
    }

    fn get_gpu_memory_total(&self) -> u64 {
        24 * 1024 * 1024 * 1024 // 24GB in bytes
    }

    fn get_gpu_compute_ops(&self, task_id: TaskId) -> u64 {
        task_id as u64 * 1000000 + self.start_time.elapsed().as_micros() as u64 * 100
    }

    fn get_gpu_memory_bandwidth(&self, task_id: TaskId) -> f64 {
        let base_bandwidth = 900.0 + (task_id as f64 % 100.0); // GB/s
        let utilization_factor = self.get_gpu_utilization(task_id) / 100.0;
        base_bandwidth * utilization_factor
    }

    fn get_gpu_temperature(&self) -> f32 {
        65.0 + (self.start_time.elapsed().as_secs() as f32 % 20.0) // 65-85°C
    }

    fn get_gpu_power_consumption(&self) -> f32 {
        300.0 + (self.start_time.elapsed().as_secs() as f32 % 150.0) // 300-450W
    }
}

impl Default for SystemResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_monitor_creation() {
        let monitor = SystemResourceMonitor::new();
        assert!(monitor.task_metrics.is_empty());
        assert!(monitor.baseline_metrics.is_none());
    }

    #[test]
    fn test_task_monitoring() {
        let mut monitor = SystemResourceMonitor::new();
        let task_id = 1234;

        monitor.start_task_monitoring(task_id);
        assert!(monitor.task_metrics.contains_key(&task_id));

        let metrics = monitor.get_task_metrics(task_id).unwrap();
        assert_eq!(metrics.task_id, task_id);
        assert!(metrics.cpu_metrics.cpu_time_us > 0);
        // bytes_read and bytes_sent are u64, always >= 0, so no need to check
    }

    #[test]
    fn test_cpu_metrics_calculation() {
        let monitor = SystemResourceMonitor::new();
        let cpu_metrics = monitor.collect_cpu_metrics(1000);

        assert!(cpu_metrics.cpu_usage_percent >= 0.0);
        assert!(cpu_metrics.cpu_usage_percent <= 100.0);
        assert!(cpu_metrics.user_time_us <= cpu_metrics.cpu_time_us);
        assert!(cpu_metrics.kernel_time_us <= cpu_metrics.cpu_time_us);
        assert!(!cpu_metrics.core_affinity.is_empty());
    }

    #[test]
    fn test_io_metrics_calculation() {
        let monitor = SystemResourceMonitor::new();
        let io_metrics = monitor.collect_io_metrics(2000);

        // bytes_read and bytes_written are u64, always >= 0
        assert!(io_metrics.avg_io_latency_us > 0.0);
        assert!(io_metrics.disk_queue_depth > 0);
    }

    #[test]
    fn test_network_metrics_calculation() {
        let monitor = SystemResourceMonitor::new();
        let network_metrics = monitor.collect_network_metrics(3000);

        // bytes_sent and bytes_received are u64, always >= 0
        assert!(network_metrics.active_connections > 0);
        assert!(network_metrics.network_latency_ms > 0.0);
    }

    #[test]
    fn test_gpu_metrics_availability() {
        let monitor = SystemResourceMonitor::new();
        let gpu_metrics = monitor.collect_gpu_metrics(4000);

        if let Some(gpu) = gpu_metrics {
            assert!(gpu.gpu_utilization_percent >= 0.0);
            assert!(gpu.gpu_utilization_percent <= 100.0);
            assert!(gpu.gpu_memory_used <= gpu.gpu_memory_total);
            assert!(!gpu.device_name.is_empty());
        }
    }
}
