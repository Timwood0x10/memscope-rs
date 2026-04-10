//! Memory tracking and visualization tools for Rust applications.
//!
//! This crate provides tools for tracking memory allocations and visualizing
//! memory usage in Rust applications. It includes a custom global allocator
//! that tracks all heap allocations and deallocations, and provides utilities
//! for exporting memory usage data in various formats.

// Import TrackKind for three-layer object model
use crate::core::types::TrackKind;

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
    GlobalTracker, GlobalTrackerConfig, GlobalTrackerStats, TrackerConfig,
};

/// Unified error handling and recovery system
pub mod error;
/// Memory allocation tracking statistics and monitoring
pub mod tracking;
/// Utility functions
pub mod utils;
/// Variable registry for lightweight HashMap-based variable tracking
pub mod variable_registry;

/// Initialize logging system for memscope-rs.
///
/// This function sets up the tracing subscriber with appropriate filtering
/// and formatting for memory tracking operations.
///
/// # Example
///
/// ```ignore
/// use memscope_rs::init_logging;
///
/// init_logging();
/// // Now logging is configured and ready to use
/// ```
///
/// # Environment Variables
///
/// The logging level can be controlled via the `RUST_LOG` environment variable:
///
/// - `RUST_LOG=memscope_rs=error` - Only errors
/// - `RUST_LOG=memscope_rs=warn` - Warnings and errors
/// - `RUST_LOG=memscope_rs=info` - Info, warnings, and errors (default)
/// - `RUST_LOG=memscope_rs=debug` - Debug, info, warnings, and errors
///
/// # Note
///
/// This function can be called multiple times safely; subsequent calls will be ignored.
pub fn init_logging() {
    use tracing_subscriber::{fmt, EnvFilter};

    // Only initialize once
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        // Initialize tracing subscriber with environment variable support
        // Default log level: INFO
        // Can be overridden with RUST_LOG environment variable
        let filter =
            EnvFilter::from_default_env().add_directive("memscope_rs=info".parse().unwrap());

        fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_thread_names(true)
            .init();

        tracing::info!("memscope-rs logging initialized");
    });
}

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
///
/// This trait defines the interface for tracking memory allocations and their
/// semantic roles in the three-layer object model (HeapOwner, Container, Value).
pub trait Trackable {
    /// Get the memory role classification for this value.
    ///
    /// Returns `TrackKind::HeapOwner` for types that own heap memory,
    /// `TrackKind::Container` for types that organize data internally,
    /// and `TrackKind::Value` for types without heap allocation.
    fn track_kind(&self) -> TrackKind;
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
    fn track_kind(&self) -> TrackKind {
        TrackKind::HeapOwner {
            ptr: self.as_ptr() as usize,
            size: self.capacity() * std::mem::size_of::<T>(),
        }
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
    fn track_kind(&self) -> TrackKind {
        TrackKind::HeapOwner {
            ptr: self.as_ptr() as usize,
            size: self.capacity(),
        }
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
    fn track_kind(&self) -> TrackKind {
        TrackKind::Container
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
    fn track_kind(&self) -> TrackKind {
        TrackKind::Container
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
    fn track_kind(&self) -> TrackKind {
        TrackKind::Container
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
    fn track_kind(&self) -> TrackKind {
        TrackKind::HeapOwner {
            ptr: &**self as *const T as usize,
            size: std::mem::size_of_val(&**self),
        }
    }
    fn get_type_name(&self) -> &'static str {
        "Box<T>"
    }
    fn get_size_estimate(&self) -> usize {
        std::mem::size_of_val(&**self)
    }
    fn get_data_ptr(&self) -> Option<usize> {
        Some(&**self as *const T as usize)
    }
    fn get_data_size(&self) -> Option<usize> {
        Some(std::mem::size_of::<T>())
    }
}

impl<T> Trackable for std::rc::Rc<T> {
    fn track_kind(&self) -> TrackKind {
        TrackKind::HeapOwner {
            ptr: &**self as *const T as usize,
            size: std::mem::size_of::<T>(),
        }
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
    fn track_kind(&self) -> TrackKind {
        TrackKind::HeapOwner {
            ptr: &**self as *const T as usize,
            size: std::mem::size_of::<T>(),
        }
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
    fn track_kind(&self) -> TrackKind {
        TrackKind::Container
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
    fn track_kind(&self) -> TrackKind {
        TrackKind::Container
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
