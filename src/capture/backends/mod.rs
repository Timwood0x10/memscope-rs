//! Capture backends for different tracking strategies.
//!
//! This module provides the CaptureBackend trait and implementations
//! for different tracking strategies (core, lockfree, async, unified).
//!
//! All files are maintained under 1000 lines per coding standards.

// Core tracking modules (split for 1000-line limit)
pub mod core_tracker;
pub mod core_tracking;
pub mod core_types;

// Lockfree tracking modules (split for 1000-line limit)
pub mod lockfree_tracker;
pub mod lockfree_types;

// Async tracking modules (split for 1000-line limit)
pub mod async_tracker;
pub mod async_types;

// Task profile modules (task-level memory profiling)
pub mod task_profile;

// Efficiency scoring modules
pub mod efficiency_scoring;

// Unsafe/FFI tracking modules
pub mod unsafe_tracking;

// Unified tracking modules (split for 1000-line limit)
pub mod unified_tracker;

use crate::event_store::{MemoryEvent, MemoryEventType};

// Re-export core tracker types
pub use core_tracker::{
    collect_all_trackers_local, configure_tracking_strategy, get_cached_thread_data_local,
    get_registry_stats_local, get_tracker, MemoryTracker, ThreadRegistryStats,
};

// Re-export async tracker types
pub use async_tracker::{
    create_tracked, get_memory_snapshot, initialize, is_tracking_active, shutdown, spawn_tracked,
    track_current_allocation, track_current_deallocation, AsyncTracker,
};
pub use async_types::{
    AsyncAllocation, AsyncError, AsyncMemorySnapshot, AsyncResult, AsyncSnapshot, AsyncStats,
    ExtendedTaskInfo, TaskId, TaskInfo, TrackedFuture,
};

// Re-export task profile types
pub use task_profile::{AggregatedTaskStats, TaskMemoryProfile, TaskProfileManager, TaskType};

// Re-export efficiency scoring types
pub use efficiency_scoring::{
    ComponentScores, EfficiencyConfig, EfficiencyScorer, EfficiencyWeights,
};

// Re-export unsafe/FFI tracking types
pub use unsafe_tracking::{
    AllocationInfo, AllocationOrigin, AllocationSource, MemoryPassport, OwnershipInfo,
    PassportStamp, SafetyViolation, SecurityClearance, UnsafeTracker, UnsafeTrackingConfig,
    UnsafeTrackingStats, ValidityStatus, ViolationSeverity,
};

// Re-export lockfree tracker types
pub use lockfree_tracker::{
    finalize_thread_tracker, get_current_tracker, init_thread_tracker, is_tracking,
    memory_snapshot, quick_trace, stop_tracing, trace_all, trace_thread, track_allocation_lockfree,
    track_deallocation_lockfree, ThreadLocalTracker,
};
pub use lockfree_types::{
    AllocationCategory, AnalysisSummary, BottleneckType, Event, EventType, FrequencyData,
    FrequencyPattern, HotCallStack, InteractionType, LockfreeAnalysis, MemoryPeak, MemorySnapshot,
    MemoryStats, PerformanceBottleneck, SamplingConfig, SystemMetrics, ThreadInteraction,
    ThreadStats,
};

// Re-export unified tracker types
pub use unified_tracker::{
    detect_environment, get_backend, initialize as initialize_unified, AsyncRuntimeType,
    BackendConfig, BackendError, DetectionConfig, DispatcherConfig, DispatcherMetrics,
    EnvironmentDetection, EnvironmentDetector, MemoryAnalysisData, MemoryStatistics,
    MemoryTracker as UnifiedMemoryTracker, RuntimeEnvironment, SessionMetadata, TrackerConfig,
    TrackerError, TrackerStatistics, TrackerType, TrackingDispatcher, TrackingOperation,
    TrackingSession, TrackingStrategy, UnifiedBackend,
};

/// Capture Backend trait
///
/// All capture backends must implement this trait to provide
/// a unified interface for capturing memory events.
pub trait CaptureBackend: Send + Sync {
    /// Capture an allocation event
    fn capture_alloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent;

    /// Capture a deallocation event
    fn capture_dealloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent;

    /// Capture a reallocation event
    fn capture_realloc(
        &self,
        ptr: usize,
        old_size: usize,
        new_size: usize,
        thread_id: u64,
    ) -> MemoryEvent;

    /// Capture a move event
    fn capture_move(
        &self,
        _from_ptr: usize,
        to_ptr: usize,
        size: usize,
        thread_id: u64,
    ) -> MemoryEvent;
}

/// Type of capture backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureBackendType {
    /// Core tracking backend (original implementation)
    Core,
    /// Lockfree tracking backend (lock-free multi-threaded)
    Lockfree,
    /// Async tracking backend (async task tracking)
    Async,
    /// Unified tracking backend (auto-detects best strategy)
    Unified,
}

impl CaptureBackendType {
    /// Create a capture backend instance
    pub fn create_backend(&self) -> Box<dyn CaptureBackend> {
        match self {
            CaptureBackendType::Core => Box::new(CoreBackend),
            CaptureBackendType::Lockfree => Box::new(LockfreeBackend),
            CaptureBackendType::Async => Box::new(AsyncBackend),
            CaptureBackendType::Unified => Box::new(UnifiedCaptureBackend::default()),
        }
    }
}

/// Core tracking backend
///
/// This is the original tracking backend implementation.
#[derive(Debug)]
pub struct CoreBackend;

impl CaptureBackend for CoreBackend {
    fn capture_alloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent {
        MemoryEvent::allocate(ptr, size, thread_id)
    }

    fn capture_dealloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent {
        MemoryEvent::deallocate(ptr, size, thread_id)
    }

    fn capture_realloc(
        &self,
        ptr: usize,
        old_size: usize,
        new_size: usize,
        thread_id: u64,
    ) -> MemoryEvent {
        MemoryEvent::reallocate(ptr, old_size, new_size, thread_id)
    }

    fn capture_move(
        &self,
        _from_ptr: usize,
        to_ptr: usize,
        size: usize,
        thread_id: u64,
    ) -> MemoryEvent {
        MemoryEvent {
            timestamp: MemoryEvent::now(),
            event_type: MemoryEventType::Move,
            ptr: to_ptr,
            size,
            thread_id,
            var_name: None,
            type_name: None,
            call_stack_hash: None,
            thread_name: None,
        }
    }
}

/// Lockfree tracking backend
///
/// This backend uses lock-free data structures for high-performance
/// multi-threaded tracking.
#[derive(Debug)]
pub struct LockfreeBackend;

impl CaptureBackend for LockfreeBackend {
    fn capture_alloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent {
        MemoryEvent::allocate(ptr, size, thread_id).with_call_stack_hash(self.hash_call_stack())
    }

    fn capture_dealloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent {
        MemoryEvent::deallocate(ptr, size, thread_id).with_call_stack_hash(self.hash_call_stack())
    }

    fn capture_realloc(
        &self,
        ptr: usize,
        old_size: usize,
        new_size: usize,
        thread_id: u64,
    ) -> MemoryEvent {
        MemoryEvent::reallocate(ptr, old_size, new_size, thread_id)
            .with_call_stack_hash(self.hash_call_stack())
    }

    fn capture_move(
        &self,
        _from_ptr: usize,
        to_ptr: usize,
        size: usize,
        thread_id: u64,
    ) -> MemoryEvent {
        MemoryEvent {
            timestamp: MemoryEvent::now(),
            event_type: MemoryEventType::Move,
            ptr: to_ptr,
            size,
            thread_id,
            var_name: None,
            type_name: None,
            call_stack_hash: Some(self.hash_call_stack()),
            thread_name: None,
        }
    }
}

impl LockfreeBackend {
    /// Generate a hash of the current call stack
    #[inline]
    fn hash_call_stack(&self) -> u64 {
        // Placeholder: In real implementation, this would capture the call stack
        // and hash it for efficient grouping and analysis
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut hasher = DefaultHasher::new();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        timestamp.hash(&mut hasher);
        hasher.finish()
    }
}

/// Async tracking backend
///
/// This backend is optimized for async task tracking with task ID support.
#[derive(Debug)]
pub struct AsyncBackend;

impl CaptureBackend for AsyncBackend {
    fn capture_alloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent {
        MemoryEvent::allocate(ptr, size, thread_id)
    }

    fn capture_dealloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent {
        MemoryEvent::deallocate(ptr, size, thread_id)
    }

    fn capture_realloc(
        &self,
        ptr: usize,
        old_size: usize,
        new_size: usize,
        thread_id: u64,
    ) -> MemoryEvent {
        MemoryEvent::reallocate(ptr, old_size, new_size, thread_id)
    }

    fn capture_move(
        &self,
        _from_ptr: usize,
        to_ptr: usize,
        size: usize,
        thread_id: u64,
    ) -> MemoryEvent {
        MemoryEvent {
            timestamp: MemoryEvent::now(),
            event_type: MemoryEventType::Move,
            ptr: to_ptr,
            size,
            thread_id,
            var_name: None,
            type_name: None,
            call_stack_hash: None,
            thread_name: None,
        }
    }
}

/// Unified capture backend
///
/// This backend automatically detects the best tracking strategy
/// based on the runtime environment for capturing events.
pub struct UnifiedCaptureBackend {
    /// The actual backend being used
    inner: Box<dyn CaptureBackend>,
}

impl Default for UnifiedCaptureBackend {
    fn default() -> Self {
        Self {
            inner: Box::new(CoreBackend),
        }
    }
}

impl CaptureBackend for UnifiedCaptureBackend {
    fn capture_alloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent {
        self.inner.capture_alloc(ptr, size, thread_id)
    }

    fn capture_dealloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent {
        self.inner.capture_dealloc(ptr, size, thread_id)
    }

    fn capture_realloc(
        &self,
        ptr: usize,
        old_size: usize,
        new_size: usize,
        thread_id: u64,
    ) -> MemoryEvent {
        self.inner
            .capture_realloc(ptr, old_size, new_size, thread_id)
    }

    fn capture_move(
        &self,
        from_ptr: usize,
        to_ptr: usize,
        size: usize,
        thread_id: u64,
    ) -> MemoryEvent {
        self.inner.capture_move(from_ptr, to_ptr, size, thread_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_backend() {
        let backend = CoreBackend;
        let event = backend.capture_alloc(0x1000, 1024, 1);
        assert_eq!(event.ptr, 0x1000);
        assert_eq!(event.size, 1024);
        assert_eq!(event.thread_id, 1);
        assert!(event.is_allocation());
    }

    #[test]
    fn test_lockfree_backend() {
        let backend = LockfreeBackend;
        let event = backend.capture_alloc(0x1000, 1024, 1);
        assert_eq!(event.ptr, 0x1000);
        assert_eq!(event.size, 1024);
        assert!(event.call_stack_hash.is_some());
    }

    #[test]
    fn test_async_backend() {
        let backend = AsyncBackend;
        let event = backend.capture_alloc(0x1000, 1024, 1);
        assert_eq!(event.ptr, 0x1000);
        assert_eq!(event.size, 1024);
    }

    #[test]
    fn test_unified_backend() {
        let backend = UnifiedCaptureBackend::default();
        let event = backend.capture_alloc(0x1000, 1024, 1);
        assert_eq!(event.ptr, 0x1000);
        assert_eq!(event.size, 1024);
    }

    #[test]
    fn test_backend_type_creation() {
        let core_backend = CaptureBackendType::Core.create_backend();
        let lockfree_backend = CaptureBackendType::Lockfree.create_backend();
        let async_backend = CaptureBackendType::Async.create_backend();
        let unified_backend = CaptureBackendType::Unified.create_backend();

        // Test that all backends can capture events
        let event1 = core_backend.capture_alloc(0x1000, 1024, 1);
        let event2 = lockfree_backend.capture_alloc(0x2000, 2048, 2);
        let event3 = async_backend.capture_alloc(0x3000, 3072, 3);
        let event4 = unified_backend.capture_alloc(0x4000, 4096, 4);

        assert_eq!(event1.ptr, 0x1000);
        assert_eq!(event2.ptr, 0x2000);
        assert_eq!(event3.ptr, 0x3000);
        assert_eq!(event4.ptr, 0x4000);
    }
}
