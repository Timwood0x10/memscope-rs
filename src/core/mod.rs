//! Core memory tracking functionality
//!
//! This module contains the fundamental components for memory tracking:
//! - Memory tracker implementation
//! - Custom allocator
//! - Type definitions
//! - Scope tracking

pub mod adaptive_hashmap;
pub mod allocation_adapter;
pub mod allocator;
pub mod atomic_stats;
pub mod clone_monitor;
pub mod clone_optimizer;
pub mod clone_utils;
pub mod error;
pub mod error_adapter;
pub mod optimized_locks;
pub mod optimized_tracker;
pub mod optimized_types;
pub mod scope_tracker;
pub mod sharded_locks;
pub mod shared_types;
pub mod smart_optimization;
pub mod targeted_optimizations;
pub mod string_pool;
pub mod string_pool_monitor;
pub mod test_optimized_locks;
pub mod tracker;
pub mod tracker_optimizations;
/// Type definitions
pub mod types;
pub mod unwrap_safe;

// Re-export key types for easier access
pub use allocator::TrackingAllocator;
pub use tracker::{get_global_tracker, MemoryTracker};
pub use types::{AllocationInfo, TrackingError, TrackingResult};

// Re-export the new unified error system
pub use error::{MemScopeError, Result as MemScopeResult, ErrorRecovery, DefaultErrorRecovery, RecoveryAction, MemoryOperation, SystemErrorType, ErrorSeverity};
pub use error_adapter::{ErrorAdapter, DefaultErrorAdapter, from_tracking_error, to_tracking_error, adapt_result, to_tracking_result};

// Re-export safe unwrap utilities
pub use unwrap_safe::{UnwrapSafe, UnwrapStats, get_unwrap_stats, update_unwrap_stats};

// Re-export string pool functionality
pub use string_pool::{intern_string, get_string_pool_stats, StringPoolStats};

// Re-export string pool monitoring
pub use string_pool_monitor::{
    get_string_pool_monitor_stats, StringPoolMonitorStats, PerformanceMetrics,
    MemoryEfficiencyMetrics, UsagePatterns, OptimizationRecommendation
};

// Re-export optimized types
pub use optimized_types::OptimizedAllocationInfo;

// Re-export atomic statistics
pub use atomic_stats::{AtomicMemoryStats, SimpleMemoryStats, AtomicPerformanceCounters, MemoryStatsSnapshot, PerformanceSnapshot};

// Re-export optimized locks
pub use optimized_locks::{OptimizedMutex, OptimizedRwLock, LockFreeCounter};

// Re-export sharded locks
pub use sharded_locks::{ShardedRwLock, ShardedMutex, ShardStats};

// Re-export adaptive hashmap
pub use adaptive_hashmap::AdaptiveHashMap;

// Re-export smart optimization tools
pub use smart_optimization::{SmartMutex, SafeUnwrap, SmartClone, SmartStats};

// Re-export targeted optimizations
pub use targeted_optimizations::{FastStatsCollector, BatchProcessor, efficient_string_concat};

// Re-export allocation adapter for compatibility
pub use allocation_adapter::{AllocationInfoAdapter, AllocationCollection, CollectionMemoryStats};

// Re-export clone optimization functionality
pub use clone_optimizer::{CloneOptimizer, CloneStats, CloneInfo};
pub use clone_monitor::{get_clone_stats, get_optimization_recommendations, CloneMonitorStats};
pub use clone_utils::{share_allocation_info, clone_shared_allocation, optimized_clone};
pub use optimized_tracker::OptimizedMemoryTracker;
pub use shared_types::{SharedAllocationInfo, SharedAllocationCollection, SharedConfig};

#[cfg(test)]
pub use unwrap_safe::get_unwrap_stats_mut;
