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
pub mod bounded_memory_stats;
pub mod call_stack_normalizer;
pub mod clone_monitor;
pub mod clone_optimizer;
pub mod clone_utils;
pub mod enhanced_pointer_extractor;
pub mod enhanced_type_inference;
pub mod error;
pub mod error_adapter;
pub mod lifecycle_summary;
pub mod optimized_locks;
pub mod optimized_tracker;
pub mod optimized_types;
pub mod ownership_history;
pub mod safe_operations;
pub mod scope_tracker;
pub mod sharded_locks;
pub mod shared_types;
pub mod simple_mutex;
pub mod smart_optimization;
pub mod string_pool;
pub mod string_pool_monitor;
pub mod targeted_optimizations;
pub mod test_optimized_locks;
pub mod threshold_batch_processor;
pub mod tracker;

/// Type definitions
pub mod types;
pub mod unwrap_safe;

// Re-export key types for easier access
pub use allocator::TrackingAllocator;
pub use tracker::{get_global_tracker, MemoryTracker};
pub use types::{AllocationInfo, TrackingError, TrackingResult};

// Re-export the new unified error system
pub use error::{
    DefaultErrorRecovery, ErrorRecovery, ErrorSeverity, MemScopeError, MemoryOperation,
    RecoveryAction, Result as MemScopeResult, SystemErrorType,
};
pub use error_adapter::{
    adapt_result, from_tracking_error, to_tracking_error, to_tracking_result, DefaultErrorAdapter,
    ErrorAdapter,
};

// Re-export safe unwrap utilities
pub use unwrap_safe::{get_unwrap_stats, update_unwrap_stats, UnwrapSafe, UnwrapStats};

// Re-export string pool functionality
pub use string_pool::{get_string_pool_stats, intern_string, StringPoolStats};

// Re-export string pool monitoring
pub use string_pool_monitor::{
    get_string_pool_monitor_stats, MemoryEfficiencyMetrics, OptimizationRecommendation,
    PerformanceMetrics, StringPoolMonitorStats, UsagePatterns,
};

// Re-export optimized types
pub use optimized_types::OptimizedAllocationInfo;

// Re-export atomic statistics
pub use atomic_stats::{
    AtomicMemoryStats, AtomicPerformanceCounters, MemoryStatsSnapshot, PerformanceSnapshot,
    SimpleMemoryStats,
};

// Re-export optimized locks
pub use optimized_locks::{LockFreeCounter, OptimizedMutex, OptimizedRwLock};

// Re-export sharded locks
pub use sharded_locks::{ShardStats, ShardedMutex, ShardedRwLock};

// Re-export adaptive hashmap
pub use adaptive_hashmap::AdaptiveHashMap;

// Re-export simple mutex
pub use simple_mutex::SimpleMutex;

// Re-export smart optimization tools
pub use smart_optimization::{SmartClone, SmartMutex, SmartStats};

// Re-export targeted optimizations
pub use targeted_optimizations::{efficient_string_concat, BatchProcessor, FastStatsCollector};

// Re-export threshold batch processor
pub use threshold_batch_processor::{BatchConfig, ProcessingStats, ThresholdBatchProcessor};

// Re-export allocation adapter for compatibility
pub use allocation_adapter::{AllocationCollection, AllocationInfoAdapter, CollectionMemoryStats};

// Re-export clone optimization functionality
pub use clone_monitor::{get_clone_stats, get_optimization_recommendations, CloneMonitorStats};
pub use clone_optimizer::{CloneInfo, CloneOptimizer, CloneStats};
pub use clone_utils::{clone_shared_allocation, optimized_clone, share_allocation_info};
pub use optimized_tracker::OptimizedMemoryTracker;
pub use shared_types::{SharedAllocationCollection, SharedAllocationInfo, SharedConfig};

// Re-export bounded memory stats functionality
pub use bounded_memory_stats::{
    AllocationHistoryManager, AllocationSummary, BoundedMemoryStats, BoundedStatsConfig,
    HistoricalSummary, MemoryUsageStats,
};

// Re-export enhanced pointer extractor functionality
pub use enhanced_pointer_extractor::{
    EnhancedPointerExtractor, EnhancedTrackable, PointerInfo, PointerStatistics, SyntheticReason,
    TypeCategory, TypeInfo,
};

// Re-export enhanced type inference functionality
pub use enhanced_type_inference::{
    AllocationContext, InferenceMethod, InferenceStatistics, InferredType, TypeConfidence,
    TypeInferenceEngine, TypeSignature,
};

// Re-export ownership history functionality
pub use ownership_history::{
    ActiveBorrow, BorrowInfo, BorrowType, CloneInfo as OwnershipCloneInfo, HistoryConfig,
    OwnershipEvent, OwnershipEventDetails, OwnershipEventType, OwnershipHistoryExport,
    OwnershipHistoryRecorder, OwnershipStatistics, OwnershipSummary, RefCountInfo,
};

// Re-export lifecycle summary functionality
pub use lifecycle_summary::{
    AllocationLifecycleSummary, ExportMetadata, LifecycleEvent, LifecycleEventSummary,
    LifecycleExportData, LifecyclePattern, LifecycleSummaryGenerator, SummaryConfig, VariableGroup,
};

// Re-export call stack normalizer functionality
pub use call_stack_normalizer::{
    get_global_call_stack_normalizer, initialize_global_call_stack_normalizer, CallStackId,
    CallStackNormalizer, CallStackRef, NormalizedCallStack, NormalizerConfig, NormalizerStats,
};

#[cfg(test)]
pub use unwrap_safe::get_unwrap_stats_mut;
