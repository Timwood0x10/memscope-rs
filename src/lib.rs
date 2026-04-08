//! Memory tracking and visualization tools for Rust applications.
//!
//! This crate provides tools for tracking memory allocations and visualizing
//! memory usage in Rust applications. It includes a custom global allocator
//! that tracks all heap allocations and deallocations, and provides utilities
//! for exporting memory usage data in various formats.

/// Advanced memory analysis functionality
pub mod analysis;
/// Analysis Engine - Memory analysis logic
pub mod analysis_engine;
/// Core memory tracking functionality
pub mod core;

pub mod capture;
/// Event Store Engine - Centralized event storage
pub mod event_store;
/// Facade API - Unified user interface
pub mod facade;
/// Metadata Engine - Centralized metadata management
pub mod metadata;
/// Query Engine - Unified query interface
pub mod query;
/// Render Engine - Output rendering
pub mod render_engine;
/// Snapshot Engine - Snapshot construction and aggregation
pub mod snapshot;
/// Memory management utilities
pub mod memory {
    pub use crate::snapshot::memory::*;
}
/// Timeline Engine - Time-based memory analysis
pub mod timeline;
/// Unified Tracker API - Simple, unified interface for memory tracking
pub mod tracker;

// Export simplified global tracking API
pub use capture::backends::global_tracking::{
    global_tracker, init_global_tracking, init_global_tracking_with_config, is_initialized,
    reset_global_tracking, GlobalTracker, GlobalTrackerConfig, GlobalTrackerStats, TrackerConfig,
};

/// Unified error handling and recovery system
pub mod error;
/// Memory allocation tracking statistics and monitoring
pub mod tracking;
/// Utility functions
pub mod utils;
/// Variable registry for lightweight HashMap-based variable tracking
pub mod variable_registry;

pub use analysis::*;
pub use capture::backends::bottleneck_analysis::{BottleneckKind, PerformanceIssue};
pub use capture::backends::hotspot_analysis::{CallStackHotspot, MemoryUsagePeak};
pub use capture::backends::{
    configure_tracking_strategy, get_tracker as get_capture_tracker, AllocationCategory,
    AnalysisSummary, AsyncAllocation, AsyncBackend, AsyncMemorySnapshot, AsyncSnapshot, AsyncStats,
    AsyncTracker, CoreBackend, Event, EventType, FrequencyData, FrequencyPattern, InteractionType,
    LockfreeAnalysis, LockfreeBackend, RuntimeEnvironment, SamplingConfig, SystemMetrics, TaskInfo,
    TaskMemoryProfile, ThreadInteraction, ThreadLocalTracker, ThreadStats, TrackedFuture,
    TrackingStrategy, UnifiedBackend,
};
pub use capture::backends::{
    is_tracking, memory_snapshot, quick_trace, stop_tracing, trace_all, trace_thread,
};
pub use capture::types::{AllocationInfo, SmartPointerInfo, TrackingError, TrackingResult};
pub use capture::{CaptureBackend, CaptureBackendType, CaptureEngine};
pub use core::allocator::TrackingAllocator;
pub use core::tracker::{get_tracker, MemoryTracker};
pub use core::{ExportMode, ExportOptions};
pub use core::{MemScopeError, MemScopeResult};
#[cfg(feature = "derive")]
pub use memscope_derive::Trackable;
pub use snapshot::engine::SnapshotEngine;
pub use snapshot::memory::{
    BoundedHistory, BoundedHistoryConfig, BoundedHistoryStats, MemoryConfig, TimestampedEntry,
};
pub use snapshot::types::{ActiveAllocation, MemorySnapshot, MemoryStats, ThreadMemoryStats};
/// Global tracking allocator instance - only enabled with tracking-allocator feature
/// for single-threaded or low-concurrency applications.
#[cfg(feature = "tracking-allocator")]
#[global_allocator]
pub static GLOBAL: TrackingAllocator = TrackingAllocator::new();
/// Trait for types that can be tracked by the memory tracker.
pub trait Trackable {
    /// Get the pointer to the heap allocation for this value.
    fn get_heap_ptr(&self) -> Option<usize>;
    /// Get the type name for this value.
    fn get_type_name(&self) -> &'static str;
    /// Get estimated size of the allocation.
    fn get_size_estimate(&self) -> usize;
    /// Get reference count for smart pointers (Arc/Rc).
    fn get_ref_count(&self) -> Option<usize> {
        None
    }
    /// Get data pointer for smart pointers.
    fn get_data_ptr(&self) -> Option<usize>;
    /// Get the size of the pointed-to data.
    fn get_data_size(&self) -> Option<usize>;
}

impl<T> Trackable for Vec<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        Some(self.as_ptr() as usize)
    }
    fn get_type_name(&self) -> &'static str {
        "Vec<T>"
    }
    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<T>() * self.capacity()
    }
    fn get_data_ptr(&self) -> Option<usize> {
        Some(self.as_ptr() as usize)
    }
    fn get_data_size(&self) -> Option<usize> {
        Some(std::mem::size_of::<T>() * self.len())
    }
}

impl Trackable for String {
    fn get_heap_ptr(&self) -> Option<usize> {
        Some(self.as_ptr() as usize)
    }
    fn get_type_name(&self) -> &'static str {
        "String"
    }
    fn get_size_estimate(&self) -> usize {
        self.capacity()
    }
    fn get_data_ptr(&self) -> Option<usize> {
        Some(self.as_ptr() as usize)
    }
    fn get_data_size(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<K, V> Trackable for std::collections::HashMap<K, V> {
    fn get_heap_ptr(&self) -> Option<usize> {
        None
    }
    fn get_type_name(&self) -> &'static str {
        "HashMap<K, V>"
    }
    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<(K, V)>() * self.capacity()
    }
    fn get_data_ptr(&self) -> Option<usize> {
        None
    }
    fn get_data_size(&self) -> Option<usize> {
        Some(std::mem::size_of::<(K, V)>() * self.len())
    }
}

impl<K, V> Trackable for std::collections::BTreeMap<K, V> {
    fn get_heap_ptr(&self) -> Option<usize> {
        None
    }
    fn get_type_name(&self) -> &'static str {
        "BTreeMap<K, V>"
    }
    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<(K, V)>() * self.len()
    }
    fn get_data_ptr(&self) -> Option<usize> {
        None
    }
    fn get_data_size(&self) -> Option<usize> {
        Some(std::mem::size_of::<(K, V)>() * self.len())
    }
}

impl<T> Trackable for std::collections::VecDeque<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        None
    }
    fn get_type_name(&self) -> &'static str {
        "VecDeque<T>"
    }
    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<T>() * self.capacity()
    }
    fn get_data_ptr(&self) -> Option<usize> {
        None
    }
    fn get_data_size(&self) -> Option<usize> {
        Some(std::mem::size_of::<T>() * self.len())
    }
}

impl<T> Trackable for Box<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        Some(&**self as *const T as usize)
    }
    fn get_type_name(&self) -> &'static str {
        "Box<T>"
    }
    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<T>()
    }
    fn get_data_ptr(&self) -> Option<usize> {
        Some(&**self as *const T as usize)
    }
    fn get_data_size(&self) -> Option<usize> {
        Some(std::mem::size_of::<T>())
    }
}

impl<T> Trackable for std::rc::Rc<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        Some(&**self as *const T as usize)
    }
    fn get_type_name(&self) -> &'static str {
        "Rc<T>"
    }
    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<T>()
    }
    fn get_ref_count(&self) -> Option<usize> {
        Some(std::rc::Rc::strong_count(self))
    }
    fn get_data_ptr(&self) -> Option<usize> {
        Some(&**self as *const T as usize)
    }
    fn get_data_size(&self) -> Option<usize> {
        Some(std::mem::size_of::<T>())
    }
}

impl<T> Trackable for std::sync::Arc<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        Some(&**self as *const T as usize)
    }
    fn get_type_name(&self) -> &'static str {
        "Arc<T>"
    }
    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<T>()
    }
    fn get_ref_count(&self) -> Option<usize> {
        Some(std::sync::Arc::strong_count(self))
    }
    fn get_data_ptr(&self) -> Option<usize> {
        Some(&**self as *const T as usize)
    }
    fn get_data_size(&self) -> Option<usize> {
        Some(std::mem::size_of::<T>())
    }
}

impl<T: Trackable> Trackable for std::cell::RefCell<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        None
    }
    fn get_type_name(&self) -> &'static str {
        "RefCell<T>"
    }
    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<T>()
    }
    fn get_data_ptr(&self) -> Option<usize> {
        None
    }
    fn get_data_size(&self) -> Option<usize> {
        Some(std::mem::size_of::<T>())
    }
}

impl<T: Trackable> Trackable for std::sync::RwLock<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        None
    }
    fn get_type_name(&self) -> &'static str {
        "RwLock<T>"
    }
    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<T>()
    }
    fn get_data_ptr(&self) -> Option<usize> {
        None
    }
    fn get_data_size(&self) -> Option<usize> {
        Some(std::mem::size_of::<T>())
    }
}
