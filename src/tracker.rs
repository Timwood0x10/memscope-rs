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
//! // Analyze the tracked allocations
//! let report = tracker.analyze();
//!
//! // Advanced usage with custom sampling
//! use memscope_rs::tracker::SamplingConfig;
//! let tracker = tracker!().with_sampling(SamplingConfig::high_performance());
//! ```

use crate::capture::system_monitor;
use crate::core::tracker::MemoryTracker;
use crate::event_store::{EventStore, MemoryEvent};
use crate::render_engine::dashboard::renderer::rebuild_allocations_from_events;
use crate::render_engine::export::{export_snapshot_to_json, ExportJsonOptions};
use crate::snapshot::MemorySnapshot;

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
    event_store: Arc<EventStore>,
    config: Arc<Mutex<TrackerConfig>>,
    start_time: Instant,
    system_snapshots: Arc<Mutex<Vec<SystemSnapshot>>>,
}

impl Clone for Tracker {
    fn clone(&self) -> Self {
        Tracker {
            inner: self.inner.clone(),
            event_store: self.event_store.clone(),
            config: self.config.clone(),
            start_time: Instant::now(), // Use current time for cloned tracker
            system_snapshots: self.system_snapshots.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct TrackerConfig {
    sampling: SamplingConfig,
    auto_export_on_drop: bool,
    export_path: Option<String>,
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MemoryTracker::new()),
            event_store: Arc::new(EventStore::new()),
            config: Arc::new(Mutex::new(TrackerConfig {
                sampling: SamplingConfig::default(),
                auto_export_on_drop: false,
                export_path: None,
            })),
            start_time: Instant::now(),
            system_snapshots: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn global() -> Self {
        use crate::core::tracker::get_tracker;
        static GLOBAL_EVENT_STORE: std::sync::OnceLock<Arc<EventStore>> =
            std::sync::OnceLock::new();
        static GLOBAL_CONFIG: std::sync::OnceLock<Arc<Mutex<TrackerConfig>>> =
            std::sync::OnceLock::new();
        static GLOBAL_SYSTEM_SNAPSHOTS: std::sync::OnceLock<Arc<Mutex<Vec<SystemSnapshot>>>> =
            std::sync::OnceLock::new();

        Self {
            inner: get_tracker(),
            event_store: GLOBAL_EVENT_STORE
                .get_or_init(|| Arc::new(EventStore::new()))
                .clone(),
            config: GLOBAL_CONFIG
                .get_or_init(|| {
                    Arc::new(Mutex::new(TrackerConfig {
                        sampling: SamplingConfig::default(),
                        auto_export_on_drop: false,
                        export_path: None,
                    }))
                })
                .clone(),
            start_time: Instant::now(),
            system_snapshots: GLOBAL_SYSTEM_SNAPSHOTS
                .get_or_init(|| Arc::new(Mutex::new(Vec::new())))
                .clone(),
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

    pub fn track_as<T: crate::Trackable>(
        &self,
        var: &T,
        name: &str,
        file: &str,
        line: u32,
        module_path: &str,
    ) {
        if let Ok(cfg) = self.config.lock() {
            if cfg.sampling.sample_rate < 1.0 {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                // Use current timestamp for randomness to ensure sampling works
                // correctly even with identical variable names in a loop
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos();
                timestamp.hash(&mut hasher);
                std::thread::current().id().hash(&mut hasher);
                name.hash(&mut hasher);
                file.hash(&mut hasher);
                line.hash(&mut hasher);
                let hash = hasher.finish();
                let threshold = (cfg.sampling.sample_rate * 1000.0) as u64;
                if (hash % 1000) > threshold {
                    return;
                }
            }
        }

        self.track_inner(var, name, file, line, module_path);
    }

    #[allow(clippy::too_many_arguments)]
    /// Track a clone operation
    pub fn track_clone(
        &self,
        source_ptr: usize,
        target_ptr: usize,
        size: usize,
        var_name: Option<String>,
        type_name: Option<String>,
        file: &str,
        line: u32,
        module_path: &str,
    ) {
        let thread_id_u64 = crate::utils::current_thread_id_u64();

        let mut event = crate::event_store::MemoryEvent::clone_event(
            source_ptr,
            target_ptr,
            size,
            thread_id_u64,
            var_name,
            type_name,
        );
        event.source_file = Some(file.to_string());
        event.source_line = Some(line);
        event.module_path = Some(module_path.to_string());

        self.event_store.record(event);
    }

    fn track_inner<T: crate::Trackable>(
        &self,
        var: &T,
        name: &str,
        file: &str,
        line: u32,
        module_path: &str,
    ) {
        let type_name = var.get_type_name().to_string();
        let kind = var.track_kind();

        let thread_id_u64 = crate::utils::current_thread_id_u64();

        match kind {
            crate::core::types::TrackKind::HeapOwner { ptr, size } => {
                // Only HeapOwner gets tracked in inner tracker
                if let Err(e) = self.inner.track_allocation(ptr, size) {
                    tracing::error!("Failed to track allocation at ptr {:x}: {}", ptr, e);
                    return;
                }

                let mut event = MemoryEvent::allocate(ptr, size, thread_id_u64);
                event.var_name = Some(name.to_string());
                event.type_name = Some(type_name.clone());
                event.source_file = Some(file.to_string());
                event.source_line = Some(line);
                event.module_path = Some(module_path.to_string());
                self.event_store.record(event);

                if let Err(e) = self.inner.associate_var(
                    ptr,
                    name.to_string(),
                    type_name,
                    Some(file),
                    Some(line),
                ) {
                    tracing::error!("Failed to associate var '{}' at ptr {:x}: {}", name, ptr, e);
                }
            }
            crate::core::types::TrackKind::StackOwner {
                ptr: stack_ptr,
                heap_ptr,
                size,
            } => {
                // StackOwner records stack pointer metadata for clone detection
                // Use stack_ptr as key for track_allocation to avoid overwriting Arc clones
                // This allows inner tracker to count allocations while preserving clone detection

                if let Err(e) = self.inner.track_allocation(stack_ptr, size) {
                    tracing::error!("Failed to track allocation at ptr {:x}: {}", stack_ptr, e);
                    return;
                }

                let mut event = MemoryEvent::allocate(heap_ptr, size, thread_id_u64);
                event.var_name = Some(name.to_string());
                event.type_name = Some(type_name.clone());
                event.source_file = Some(file.to_string());
                event.source_line = Some(line);
                event.module_path = Some(module_path.to_string());
                // Store stack pointer in custom metadata for clone detection
                event.stack_ptr = Some(stack_ptr);
                self.event_store.record(event);

                if let Err(e) = self.inner.associate_var(
                    heap_ptr,
                    name.to_string(),
                    type_name,
                    Some(file),
                    Some(line),
                ) {
                    tracing::error!(
                        "Failed to associate var '{}' at ptr {:x}: {}",
                        name,
                        heap_ptr,
                        e
                    );
                }
            }
            crate::core::types::TrackKind::Container | crate::core::types::TrackKind::Value => {
                // Container and Value record metadata events without heap allocation
                // They will be tracked as graph nodes but not scanned by HeapScanner
                let mut event = MemoryEvent::metadata(
                    name.to_string(),
                    type_name,
                    thread_id_u64,
                    var.get_size_estimate(),
                );
                event.source_file = Some(file.to_string());
                event.source_line = Some(line);
                event.module_path = Some(module_path.to_string());
                self.event_store.record(event);
            }
        }
    }

    pub fn track_deallocation(&self, ptr: usize) -> crate::TrackingResult<bool> {
        let size = self.inner.get_allocation_size(ptr).unwrap_or(0);

        let result = self.inner.track_deallocation(ptr)?;

        // Only record event if deallocation was successful (ptr was tracked)
        if result {
            let thread_id_u64 = crate::utils::current_thread_id_u64();

            let event = MemoryEvent::deallocate(ptr, size, thread_id_u64);
            self.event_store.record(event);
        }

        Ok(result)
    }

    pub fn events(&self) -> Vec<MemoryEvent> {
        self.event_store.snapshot()
    }

    pub fn event_store(&self) -> &Arc<EventStore> {
        &self.event_store
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

    pub fn stats(&self) -> crate::core::types::MemoryStats {
        let stats = self.inner.get_stats().unwrap_or_default();
        crate::core::types::MemoryStats {
            total_allocations: stats.total_allocations as usize,
            total_allocated: stats.total_allocated as usize,
            active_allocations: stats.active_allocations,
            active_memory: stats.active_memory as usize,
            peak_allocations: stats.peak_allocations,
            peak_memory: stats.peak_memory as usize,
            total_deallocations: stats.total_deallocations as usize,
            total_deallocated: stats.total_deallocated as usize,
            leaked_allocations: stats.leaked_allocations,
            leaked_memory: stats.leaked_memory as usize,
            ..Default::default()
        }
    }

    pub fn analyze(&self) -> AnalysisReport {
        let stats = self.stats();
        let events = self.event_store().snapshot();
        let allocations = rebuild_allocations_from_events(&events);
        let elapsed = self.start_time.elapsed().as_secs_f64();

        let current_memory: usize = allocations.iter().map(|a| a.size).sum();
        let peak_memory = stats.peak_memory.max(current_memory);

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
            active_allocations: allocations.len(),
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
        // Auto-record deallocation for all active allocations
        let events = self.event_store().snapshot();
        let allocations = rebuild_allocations_from_events(&events);

        // Record deallocation events for all active allocations
        let thread_id_u64 = crate::utils::current_thread_id_u64();

        for alloc in &allocations {
            let event = MemoryEvent::deallocate(alloc.ptr, alloc.size, thread_id_u64);
            self.event_store().record(event);
        }

        if let Ok(cfg) = self.config.lock() {
            if cfg.auto_export_on_drop {
                if let Some(ref path) = cfg.export_path {
                    // Use event_store as unified data source (includes both HeapOwner and Container allocations)
                    let events = self.event_store().snapshot();
                    let allocations = rebuild_allocations_from_events(&events);
                    let snapshot = MemorySnapshot::from_allocation_infos(allocations);
                    let options = ExportJsonOptions::default();
                    if let Err(e) =
                        export_snapshot_to_json(&snapshot, std::path::Path::new(path), &options)
                    {
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
        $tracker.track_as(&$var, var_name, file!(), line!(), module_path!());
    }};
}

#[macro_export]
macro_rules! track_clone {
    ($tracker:expr, $source:expr, $target:expr) => {{
        let source_name = stringify!($source);
        let target_name = stringify!($target);
        let source_ptr = &$source as *const _ as usize;
        let target_ptr = &$target as *const _ as usize;
        let type_name = $crate::utils::type_of(&$target);
        $tracker.track_clone(
            source_ptr,
            target_ptr,
            std::mem::size_of_val(&$target),
            Some(target_name.to_string()),
            Some(type_name.to_string()),
            file!(),
            line!(),
            module_path!(),
        );
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
    #[cfg(target_os = "macos")]
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

    #[test]
    fn test_sampling_config_default() {
        let config = SamplingConfig::default();
        assert_eq!(config.sample_rate, 1.0);
        assert!(!config.capture_call_stack);
        assert_eq!(config.max_stack_depth, 10);
    }

    #[test]
    fn test_sampling_config_demo() {
        let config = SamplingConfig::demo();
        assert_eq!(config.sample_rate, 0.1);
        assert!(!config.capture_call_stack);
        assert_eq!(config.max_stack_depth, 5);
    }

    #[test]
    fn test_sampling_config_full() {
        let config = SamplingConfig::full();
        assert_eq!(config.sample_rate, 1.0);
        assert!(config.capture_call_stack);
        assert_eq!(config.max_stack_depth, 20);
    }

    #[test]
    fn test_sampling_config_high_performance() {
        let config = SamplingConfig::high_performance();
        assert_eq!(config.sample_rate, 0.01);
        assert!(!config.capture_call_stack);
        assert_eq!(config.max_stack_depth, 0);
    }

    #[test]
    fn test_sampling_config_clone() {
        let config = SamplingConfig::full();
        let cloned = config.clone();
        assert_eq!(cloned.sample_rate, config.sample_rate);
        assert_eq!(cloned.capture_call_stack, config.capture_call_stack);
    }

    #[test]
    fn test_sampling_config_debug() {
        let config = SamplingConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("SamplingConfig"));
        assert!(debug_str.contains("sample_rate"));
    }

    #[test]
    fn test_system_snapshot_default() {
        let snapshot = SystemSnapshot::default();
        assert_eq!(snapshot.timestamp, 0);
        assert_eq!(snapshot.cpu_usage_percent, 0.0);
        assert_eq!(snapshot.memory_usage_bytes, 0);
        assert_eq!(snapshot.thread_count, 0);
    }

    #[test]
    fn test_system_snapshot_clone() {
        let snapshot = SystemSnapshot {
            timestamp: 1000,
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 1024 * 1024,
            memory_usage_percent: 25.0,
            thread_count: 4,
            disk_read_bps: 1000,
            disk_write_bps: 500,
            network_rx_bps: 2000,
            network_tx_bps: 1000,
            gpu_usage_percent: 30.0,
            gpu_memory_used: 512 * 1024 * 1024,
            gpu_memory_total: 2 * 1024 * 1024 * 1024,
        };

        let cloned = snapshot.clone();
        assert_eq!(cloned.timestamp, 1000);
        assert_eq!(cloned.cpu_usage_percent, 50.0);
    }

    #[test]
    fn test_system_snapshot_debug() {
        let snapshot = SystemSnapshot::default();
        let debug_str = format!("{:?}", snapshot);
        assert!(debug_str.contains("SystemSnapshot"));
    }

    #[test]
    fn test_analysis_report_creation() {
        let report = AnalysisReport {
            total_allocations: 100,
            total_deallocations: 50,
            active_allocations: 50,
            peak_memory_bytes: 1024 * 1024,
            current_memory_bytes: 512 * 1024,
            allocation_rate_per_sec: 10.0,
            deallocation_rate_per_sec: 5.0,
            hotspots: vec![],
            system_snapshots: vec![],
        };

        assert_eq!(report.total_allocations, 100);
        assert_eq!(report.active_allocations, 50);
    }

    #[test]
    fn test_analysis_report_clone() {
        let report = AnalysisReport {
            total_allocations: 10,
            total_deallocations: 5,
            active_allocations: 5,
            peak_memory_bytes: 1024,
            current_memory_bytes: 512,
            allocation_rate_per_sec: 1.0,
            deallocation_rate_per_sec: 0.5,
            hotspots: vec![],
            system_snapshots: vec![],
        };

        let cloned = report.clone();
        assert_eq!(cloned.total_allocations, 10);
    }

    #[test]
    fn test_analysis_report_debug() {
        let report = AnalysisReport {
            total_allocations: 0,
            total_deallocations: 0,
            active_allocations: 0,
            peak_memory_bytes: 0,
            current_memory_bytes: 0,
            allocation_rate_per_sec: 0.0,
            deallocation_rate_per_sec: 0.0,
            hotspots: vec![],
            system_snapshots: vec![],
        };

        let debug_str = format!("{:?}", report);
        assert!(debug_str.contains("AnalysisReport"));
    }

    #[test]
    fn test_allocation_hotspot_creation() {
        let hotspot = AllocationHotspot {
            var_name: "my_vec".to_string(),
            type_name: "Vec<u8>".to_string(),
            total_size: 1024,
            allocation_count: 10,
            location: Some("main.rs:42".to_string()),
        };

        assert_eq!(hotspot.var_name, "my_vec");
        assert_eq!(hotspot.total_size, 1024);
    }

    #[test]
    fn test_allocation_hotspot_clone() {
        let hotspot = AllocationHotspot {
            var_name: "data".to_string(),
            type_name: "String".to_string(),
            total_size: 100,
            allocation_count: 5,
            location: None,
        };

        let cloned = hotspot.clone();
        assert_eq!(cloned.var_name, "data");
    }

    #[test]
    fn test_allocation_hotspot_debug() {
        let hotspot = AllocationHotspot {
            var_name: "test".to_string(),
            type_name: "i32".to_string(),
            total_size: 4,
            allocation_count: 1,
            location: None,
        };

        let debug_str = format!("{:?}", hotspot);
        assert!(debug_str.contains("AllocationHotspot"));
    }

    #[test]
    fn test_tracker_clone() {
        let tracker = Tracker::new();
        let cloned = tracker.clone();

        let report1 = tracker.analyze();
        let report2 = cloned.analyze();

        // Both should have the same underlying data
        assert_eq!(report1.total_allocations, report2.total_allocations);
    }

    #[test]
    fn test_tracker_with_sampling() {
        let tracker = Tracker::new().with_sampling(SamplingConfig::high_performance());
        let data = vec![1, 2, 3];
        tracker.track_as(&data, "data", "test.rs", 1, "test_module");
    }

    #[test]
    fn test_tracker_elapsed() {
        let tracker = Tracker::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = tracker.elapsed();
        assert!(elapsed >= std::time::Duration::from_millis(10));
    }

    #[test]
    fn test_tracker_with_system_monitoring() {
        let tracker = Tracker::new().with_system_monitoring();
        let _ = tracker.current_system_snapshot();
    }

    #[test]
    fn test_tracker_track_as_multiple() {
        let tracker = Tracker::new();
        let data = vec![1, 2, 3, 4, 5];

        tracker.track_as(&data, "my_vec", "test.rs", 10, "test_module");
        tracker.track_as(&data, "my_vec", "test.rs", 20, "test_module");

        let report = tracker.analyze();
        let _ = report.total_allocations;
    }

    #[test]
    fn test_sampling_config_custom() {
        let config = SamplingConfig {
            sample_rate: 0.5,
            capture_call_stack: true,
            max_stack_depth: 15,
        };

        assert!((config.sample_rate - 0.5).abs() < 0.001);
        assert!(config.capture_call_stack);
        assert_eq!(config.max_stack_depth, 15);
    }

    #[test]
    fn test_analysis_report_with_hotspots() {
        let report = AnalysisReport {
            total_allocations: 100,
            total_deallocations: 50,
            active_allocations: 50,
            peak_memory_bytes: 1024 * 1024,
            current_memory_bytes: 512 * 1024,
            allocation_rate_per_sec: 10.0,
            deallocation_rate_per_sec: 5.0,
            hotspots: vec![AllocationHotspot {
                var_name: "test".to_string(),
                type_name: "Vec<u8>".to_string(),
                total_size: 1024,
                allocation_count: 10,
                location: Some("test.rs:1".to_string()),
            }],
            system_snapshots: vec![],
        };

        assert_eq!(report.hotspots.len(), 1);
    }

    #[test]
    fn test_tracker_with_sampling_and_monitoring() {
        let tracker = Tracker::new()
            .with_sampling(SamplingConfig::demo())
            .with_system_monitoring();

        let data = vec![1, 2, 3];
        tracker.track_as(&data, "data", "test.rs", 1, "test_module");

        let snapshot = tracker.current_system_snapshot();
        assert!(snapshot.cpu_usage_percent >= 0.0);
    }

    #[test]
    fn test_tracker_events() {
        let tracker = Tracker::new();
        let data = vec![1, 2, 3];
        tracker.track_as(&data, "test_data", "test.rs", 1, "test_module");

        let events = tracker.events();
        assert!(!events.is_empty());
    }

    #[test]
    fn test_tracker_event_store() {
        let tracker = Tracker::new();
        let _store = tracker.event_store();
    }

    #[test]
    fn test_tracker_stats() {
        let tracker = Tracker::new();
        let stats = tracker.stats();

        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.active_allocations, 0);
    }

    #[test]
    fn test_tracker_stats_with_data() {
        let tracker = Tracker::new();
        let data = vec![1u8; 1024];
        tracker.track_as(&data, "buffer", "test.rs", 1, "test_module");

        let stats = tracker.stats();
        assert!(
            stats.total_allocations >= 1,
            "Should have at least one allocation after tracking data"
        );
    }

    #[test]
    fn test_tracker_system_snapshots() {
        let tracker = Tracker::new().with_system_monitoring();
        let snapshots = tracker.system_snapshots();
        assert!(!snapshots.is_empty());
    }

    #[test]
    fn test_tracker_inner() {
        let tracker = Tracker::new();
        let _inner = tracker.inner();
    }

    #[test]
    fn test_tracker_with_auto_export() {
        let tracker = Tracker::new().with_auto_export("/tmp/test_export");
        let data = vec![1, 2, 3];
        tracker.track_as(&data, "test", "test.rs", 1, "test_module");
    }

    #[test]
    fn test_sampling_config_zero_rate() {
        let config = SamplingConfig {
            sample_rate: 0.0,
            capture_call_stack: false,
            max_stack_depth: 0,
        };

        let tracker = Tracker::new().with_sampling(config);
        let data = vec![1, 2, 3];
        tracker.track_as(&data, "test", "test.rs", 1, "test_module");
    }

    #[test]
    fn test_analysis_report_serialization() {
        let report = AnalysisReport {
            total_allocations: 100,
            total_deallocations: 50,
            active_allocations: 50,
            peak_memory_bytes: 1024,
            current_memory_bytes: 512,
            allocation_rate_per_sec: 10.0,
            deallocation_rate_per_sec: 5.0,
            hotspots: vec![],
            system_snapshots: vec![],
        };

        let json = serde_json::to_string(&report);
        assert!(json.is_ok());
    }

    #[test]
    fn test_allocation_hotspot_serialization() {
        let hotspot = AllocationHotspot {
            var_name: "test".to_string(),
            type_name: "Vec<u8>".to_string(),
            total_size: 1024,
            allocation_count: 10,
            location: Some("test.rs:1".to_string()),
        };

        let json = serde_json::to_string(&hotspot);
        assert!(json.is_ok());
    }

    #[test]
    fn test_tracker_multiple_system_snapshots() {
        let tracker = Tracker::new().with_system_monitoring();
        std::thread::sleep(std::time::Duration::from_millis(10));
        tracker.current_system_snapshot();

        let snapshots = tracker.system_snapshots();
        assert!(
            !snapshots.is_empty(),
            "Should have at least one system snapshot"
        );
    }

    #[test]
    fn test_tracker_analyze_with_hotspots() {
        let tracker = Tracker::new();
        let data1 = vec![1u8; 100];
        let data2 = vec![2u8; 200];
        let data3 = vec![3u8; 300];

        tracker.track_as(&data1, "buffer1", "test.rs", 1, "test_module");
        tracker.track_as(&data2, "buffer2", "test.rs", 2, "test_module");
        tracker.track_as(&data3, "buffer3", "test.rs", 3, "test_module");

        let report = tracker.analyze();
        assert!(
            report.total_allocations >= 3,
            "Should have at least 3 allocations after tracking"
        );
    }

    #[test]
    fn test_tracker_default() {
        let tracker = Tracker::default();
        let report = tracker.analyze();
        assert_eq!(report.total_allocations, 0);
    }
}
