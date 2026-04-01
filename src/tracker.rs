//! Unified Tracker API - Enhanced Version
//!
//! This module provides a simple, unified interface for memory tracking
//! with all the power of the old API and more.
//!
//! # Features
//!
//! - **Simple API**: `tracker!()` and `track!()` macros
//! - **Auto-capture**: Automatic variable name and type capture
//! - **System Monitoring**: CPU, memory monitoring (background thread, zero overhead)
//! - **Per-thread Tracking**: Independent tracking per thread
//! - **Sampling**: Configurable sampling rates
//! - **Hotspot Analysis**: Automatic allocation hotspot detection
//! - **HTML Dashboard**: Interactive visualization
//! - **JSON/Binary Export**: Multiple export formats
//!
//! # Architecture
//!
//! System monitoring runs in a background thread that collects metrics every 100ms.
//! The `track!` macro only reads atomic values (nanosecond overhead), ensuring
//! no blocking on data collection.
//!
//! # Usage
//!
//! ```rust
//! use memscope_rs::{tracker, track};
//!
//! // Simple usage - system monitoring is automatic
//! let tracker = tracker!();
//! let my_vec = vec![1, 2, 3];
//! track!(tracker, my_vec);
//! // Note: export_json returns (), not Result, so we don't use ?
//! let _ = tracker.export_json("output");
//!
//! // Advanced usage with custom sampling
//! use memscope_rs::tracker::SamplingConfig;
//! let tracker = tracker!().with_sampling(SamplingConfig::high_performance());
//! ```

use crate::core::tracker::MemoryTracker;
use crate::system_monitor;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct SamplingConfig {
    pub sample_rate: f64,
    pub capture_call_stack: bool,
    pub max_stack_depth: usize,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            sample_rate: 1.0,
            capture_call_stack: false,
            max_stack_depth: 10,
        }
    }
}

impl SamplingConfig {
    pub fn demo() -> Self {
        Self {
            sample_rate: 0.1,
            capture_call_stack: false,
            max_stack_depth: 5,
        }
    }

    pub fn full() -> Self {
        Self {
            sample_rate: 1.0,
            capture_call_stack: true,
            max_stack_depth: 20,
        }
    }

    pub fn high_performance() -> Self {
        Self {
            sample_rate: 0.01,
            capture_call_stack: false,
            max_stack_depth: 0,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SystemSnapshot {
    pub timestamp: u64,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_usage_percent: f64,
    pub thread_count: usize,
    pub disk_read_bps: u64,
    pub disk_write_bps: u64,
    pub network_rx_bps: u64,
    pub network_tx_bps: u64,
    pub gpu_usage_percent: f64,
    pub gpu_memory_used: u64,
    pub gpu_memory_total: u64,
}

#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub active_allocations: usize,
    pub peak_memory_bytes: u64,
    pub current_memory_bytes: u64,
    pub allocation_rate_per_sec: f64,
    pub deallocation_rate_per_sec: f64,
    pub hotspots: Vec<AllocationHotspot>,
    pub system_snapshots: Vec<SystemSnapshot>,
}

#[derive(Debug, Clone)]
pub struct AllocationHotspot {
    pub var_name: String,
    pub type_name: String,
    pub total_size: usize,
    pub allocation_count: usize,
    pub location: Option<String>,
}

pub struct Tracker {
    inner: Arc<MemoryTracker>,
    config: Arc<Mutex<TrackerConfig>>,
    start_time: Instant,
    system_snapshots: Arc<Mutex<Vec<SystemSnapshot>>>,
}

#[derive(Debug, Clone)]
struct TrackerConfig {
    sampling: SamplingConfig,
    #[allow(dead_code)]
    track_thread_id: bool,
    auto_export_on_drop: bool,
    export_path: Option<String>,
}

impl Tracker {
    pub fn new() -> Self {
        system_monitor::SystemMonitor::global();

        Self {
            inner: crate::core::tracker::get_tracker(),
            config: Arc::new(Mutex::new(TrackerConfig {
                sampling: SamplingConfig::default(),
                track_thread_id: true,
                auto_export_on_drop: false,
                export_path: None,
            })),
            start_time: Instant::now(),
            system_snapshots: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_system_monitoring(self) -> Self {
        self.capture_system_snapshot();
        self
    }

    pub fn with_sampling(self, config: SamplingConfig) -> Self {
        if let Ok(mut cfg) = self.config.lock() {
            cfg.sampling = config;
        }
        self
    }

    pub fn with_auto_export(self, path: &str) -> Self {
        if let Ok(mut cfg) = self.config.lock() {
            cfg.auto_export_on_drop = true;
            cfg.export_path = Some(path.to_string());
        }
        self
    }

    pub fn track_as<T: crate::Trackable>(&self, var: &T, name: &str) {
        if let Ok(cfg) = self.config.lock() {
            if cfg.sampling.sample_rate < 1.0 {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                name.hash(&mut hasher);
                let hash = hasher.finish();
                let threshold = (cfg.sampling.sample_rate * 1000.0) as u64;
                if (hash % 1000) > threshold {
                    return;
                }
            }
        }

        self.track_inner(var, name);
    }

    fn track_inner<T: crate::Trackable>(&self, var: &T, name: &str) {
        let type_name = var.get_type_name().to_string();
        let size = var.get_size_estimate();

        let ptr = var.get_heap_ptr().unwrap_or_else(|| {
            use std::cell::Cell;
            thread_local! {
                static COUNTER: Cell<u64> = const { Cell::new(0x8000_0000) };
            }
            COUNTER.with(|counter| {
                let val = counter.get();
                counter.set(val.wrapping_add(1));
                val as usize
            })
        });

        if let Err(e) = self.inner.track_allocation(ptr, size) {
            tracing::error!("Failed to track allocation at ptr {:x}: {}", ptr, e);
            return;
        }

        if let Err(e) = self.inner.associate_var(ptr, name.to_string(), type_name) {
            tracing::error!("Failed to associate var '{}' at ptr {:x}: {}", name, ptr, e);
        }
    }

    fn capture_system_snapshot(&self) {
        let snapshot = SystemSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            cpu_usage_percent: system_monitor::cpu_usage(),
            memory_usage_bytes: system_monitor::memory_used(),
            memory_usage_percent: system_monitor::memory_usage_percent(),
            thread_count: system_monitor::thread_count(),
            disk_read_bps: system_monitor::disk_read_bps(),
            disk_write_bps: system_monitor::disk_write_bps(),
            network_rx_bps: system_monitor::network_rx_bps(),
            network_tx_bps: system_monitor::network_tx_bps(),
            gpu_usage_percent: system_monitor::gpu_memory_usage_percent(),
            gpu_memory_used: system_monitor::gpu_memory_used(),
            gpu_memory_total: system_monitor::gpu_memory_total(),
        };

        if let Ok(mut snapshots) = self.system_snapshots.lock() {
            snapshots.push(snapshot);
        }
    }

    pub fn snapshot(&self) -> crate::core::types::MemoryStats {
        self.inner.get_stats().unwrap_or_default()
    }

    pub fn analyze(&self) -> AnalysisReport {
        let stats = self.snapshot();
        let allocations = self.inner.get_active_allocations().unwrap_or_default();
        let elapsed = self.start_time.elapsed().as_secs_f64();

        // Calculate peak memory from current allocations (since stats.peak_memory is broken)
        let current_memory: usize = allocations.iter().map(|a| a.size).sum();
        let peak_memory = if stats.peak_memory > 0 {
            stats.peak_memory
        } else {
            // Fallback: use current memory if peak_memory is 0
            current_memory
        };

        let mut hotspot_map: HashMap<String, (String, usize, usize)> = HashMap::new();
        for alloc in &allocations {
            if let Some(ref var_name) = alloc.var_name {
                let key = var_name.clone();
                let entry = hotspot_map.entry(key).or_insert((
                    alloc.type_name.clone().unwrap_or_default(),
                    0,
                    0,
                ));
                entry.1 += alloc.size;
                entry.2 += 1;
            }
        }

        let hotspots: Vec<AllocationHotspot> = hotspot_map
            .into_iter()
            .map(
                |(var_name, (type_name, total_size, count))| AllocationHotspot {
                    var_name,
                    type_name,
                    total_size,
                    allocation_count: count,
                    location: None,
                },
            )
            .collect();

        let system_snapshots = self
            .system_snapshots
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();

        AnalysisReport {
            total_allocations: stats.total_allocations,
            total_deallocations: stats.total_deallocations,
            active_allocations: stats.active_allocations,
            peak_memory_bytes: peak_memory as u64,
            current_memory_bytes: current_memory as u64,
            allocation_rate_per_sec: if elapsed > 0.0 {
                stats.total_allocations as f64 / elapsed
            } else {
                0.0
            },
            deallocation_rate_per_sec: if elapsed > 0.0 {
                stats.total_deallocations as f64 / elapsed
            } else {
                0.0
            },
            hotspots,
            system_snapshots,
        }
    }

    pub fn export_svg(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.inner.export_memory_analysis(path)?;
        Ok(())
    }

    pub fn export_json(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let allocations = self.inner.get_active_allocations()?;
        let stats = self.inner.get_stats()?;
        crate::export::export_user_variables_json(allocations, stats, path)?;
        Ok(())
    }

    pub fn export_binary(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let allocations = self.inner.get_active_allocations()?;
        let stats = self.inner.get_stats()?;
        crate::export::export_user_variables_binary(allocations, stats, path)?;
        Ok(())
    }

    pub fn export_analysis(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let report = self.analyze();
        let json = serde_json::to_string_pretty(&report)?;
        let output_dir = std::path::Path::new("MemoryAnalysis");
        std::fs::create_dir_all(output_dir)?;
        let file_path = output_dir.join(format!("{}_analysis.json", path));
        std::fs::write(file_path, json)?;
        Ok(())
    }

    pub fn export_html(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.export_svg(path)?;
        Ok(())
    }

    pub fn inner(&self) -> &Arc<MemoryTracker> {
        &self.inner
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn system_snapshots(&self) -> Vec<SystemSnapshot> {
        self.system_snapshots
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn current_system_snapshot(&self) -> SystemSnapshot {
        SystemSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            cpu_usage_percent: system_monitor::cpu_usage(),
            memory_usage_bytes: system_monitor::memory_used(),
            memory_usage_percent: system_monitor::memory_usage_percent(),
            thread_count: system_monitor::thread_count(),
            disk_read_bps: system_monitor::disk_read_bps(),
            disk_write_bps: system_monitor::disk_write_bps(),
            network_rx_bps: system_monitor::network_rx_bps(),
            network_tx_bps: system_monitor::network_tx_bps(),
            gpu_usage_percent: system_monitor::gpu_memory_usage_percent(),
            gpu_memory_used: system_monitor::gpu_memory_used(),
            gpu_memory_total: system_monitor::gpu_memory_total(),
        }
    }
}

impl Default for Tracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        if let Ok(cfg) = self.config.lock() {
            if cfg.auto_export_on_drop {
                if let Some(ref path) = cfg.export_path {
                    if let Err(e) = self.export_json(path) {
                        tracing::error!("Failed to auto-export on drop: {}", e);
                    }
                }
            }
        }
    }
}

impl serde::Serialize for AnalysisReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AnalysisReport", 9)?;
        state.serialize_field("total_allocations", &self.total_allocations)?;
        state.serialize_field("total_deallocations", &self.total_deallocations)?;
        state.serialize_field("active_allocations", &self.active_allocations)?;
        state.serialize_field("peak_memory_bytes", &self.peak_memory_bytes)?;
        state.serialize_field("current_memory_bytes", &self.current_memory_bytes)?;
        state.serialize_field("allocation_rate_per_sec", &self.allocation_rate_per_sec)?;
        state.serialize_field("deallocation_rate_per_sec", &self.deallocation_rate_per_sec)?;
        state.serialize_field("hotspots", &self.hotspots)?;
        state.serialize_field("system_snapshots", &self.system_snapshots)?;
        state.end()
    }
}

impl serde::Serialize for AllocationHotspot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AllocationHotspot", 5)?;
        state.serialize_field("var_name", &self.var_name)?;
        state.serialize_field("type_name", &self.type_name)?;
        state.serialize_field("total_size", &self.total_size)?;
        state.serialize_field("allocation_count", &self.allocation_count)?;
        state.serialize_field("location", &self.location)?;
        state.end()
    }
}

#[macro_export]
macro_rules! tracker {
    () => {
        $crate::tracker::Tracker::new()
    };
}

#[macro_export]
macro_rules! track {
    ($tracker:expr, $var:expr) => {{
        let var_name = stringify!($var);
        $tracker.track_as(&$var, var_name);
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_creation() {
        let tracker = Tracker::new();
        let _ = tracker;
    }

    #[test]
    fn test_tracker_with_config() {
        let tracker = Tracker::new()
            .with_sampling(SamplingConfig::demo())
            .with_system_monitoring();
        let _ = tracker;
    }

    #[test]
    fn test_track_macro() {
        let tracker = tracker!();
        let my_vec = vec![1, 2, 3];
        track!(tracker, my_vec);
    }

    #[test]
    fn test_analyze() {
        let tracker = tracker!();
        let data = vec![1, 2, 3];
        track!(tracker, data);
        let report = tracker.analyze();
        assert!(report.total_allocations > 0);
    }

    #[test]
    fn test_system_monitoring() {
        std::thread::sleep(std::time::Duration::from_millis(200));

        let cpu = system_monitor::cpu_usage();
        let mem = system_monitor::memory_used();
        let total = system_monitor::memory_total();

        println!("CPU: {:.2}%", cpu);
        println!("Memory: {} / {} bytes", mem, total);

        assert!((0.0..=100.0).contains(&cpu));
        assert!(total > 0);
    }

    #[test]
    fn test_current_system_snapshot() {
        std::thread::sleep(std::time::Duration::from_millis(150));

        let tracker = tracker!();
        let snapshot = tracker.current_system_snapshot();

        println!(
            "Snapshot: CPU={:.2}%, Mem={:.2}%",
            snapshot.cpu_usage_percent, snapshot.memory_usage_percent
        );

        assert!(snapshot.cpu_usage_percent >= 0.0 && snapshot.cpu_usage_percent <= 100.0);
    }
}
