//! Memory statistics types.
//!
//! This module contains types for tracking memory usage statistics,
//! including allocation counts, memory sizes, and fragmentation analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::allocation::AllocationInfo;
use super::scope::ScopeLifecycleMetrics;

/// Memory statistics.
///
/// Comprehensive statistics about memory allocations, deallocations,
/// leaks, and fragmentation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct MemoryStats {
    /// Total number of allocations made.
    pub total_allocations: usize,
    /// Total bytes allocated.
    pub total_allocated: usize,
    /// Number of currently active allocations.
    pub active_allocations: usize,
    /// Total bytes in active allocations.
    pub active_memory: usize,
    /// Peak number of concurrent allocations.
    pub peak_allocations: usize,
    /// Peak memory usage in bytes.
    pub peak_memory: usize,
    /// Total number of deallocations performed.
    pub total_deallocations: usize,
    /// Total bytes deallocated.
    pub total_deallocated: usize,
    /// Number of leaked allocations.
    pub leaked_allocations: usize,
    /// Total bytes in leaked allocations.
    pub leaked_memory: usize,
    /// Analysis of memory fragmentation.
    pub fragmentation_analysis: FragmentationAnalysis,
    /// Lifecycle statistics for scopes.
    pub lifecycle_stats: ScopeLifecycleMetrics,
    /// List of all allocation information.
    pub allocations: Vec<AllocationInfo>,
    /// Statistics for system library allocations.
    pub system_library_stats: SystemLibraryStats,
    /// Analysis of concurrent memory operations.
    pub concurrency_analysis: ConcurrencyAnalysis,
}

impl MemoryStats {
    /// Create a new empty MemoryStats.
    pub fn new() -> Self {
        Self {
            total_allocations: 0,
            total_allocated: 0,
            active_allocations: 0,
            active_memory: 0,
            peak_allocations: 0,
            peak_memory: 0,
            total_deallocations: 0,
            total_deallocated: 0,
            leaked_allocations: 0,
            leaked_memory: 0,
            fragmentation_analysis: FragmentationAnalysis::default(),
            lifecycle_stats: ScopeLifecycleMetrics::default(),
            allocations: Vec::new(),
            system_library_stats: SystemLibraryStats::default(),
            concurrency_analysis: ConcurrencyAnalysis::default(),
        }
    }
}

/// Memory type analysis.
#[derive(Debug, Clone, Serialize)]
pub struct MemoryTypeInfo {
    /// Name of the memory type.
    pub type_name: String,
    /// Total size in bytes for this type.
    pub total_size: usize,
    /// Number of allocations of this type.
    pub allocation_count: usize,
    /// Average size of allocations for this type.
    pub average_size: usize,
    /// Size of the largest allocation for this type.
    pub largest_allocation: usize,
    /// Size of the smallest allocation for this type.
    pub smallest_allocation: usize,
    /// Number of currently active instances.
    pub active_instances: usize,
    /// Number of leaked instances.
    pub leaked_instances: usize,
}

/// Type memory usage information.
#[derive(Debug, Clone, Serialize)]
pub struct TypeMemoryUsage {
    /// Name of the type.
    pub type_name: String,
    /// Total size allocated for this type.
    pub total_size: usize,
    /// Number of allocations for this type.
    pub allocation_count: usize,
    /// Average allocation size for this type.
    pub average_size: f64,
    /// Peak memory usage for this type.
    pub peak_size: usize,
    /// Current memory usage for this type.
    pub current_size: usize,
    /// Memory efficiency score for this type.
    pub efficiency_score: f64,
}

/// Fragmentation analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FragmentationAnalysis {
    /// Ratio of fragmented to total memory.
    pub fragmentation_ratio: f64,
    /// Size of the largest free memory block.
    pub largest_free_block: usize,
    /// Size of the smallest free memory block.
    pub smallest_free_block: usize,
    /// Total number of free memory blocks.
    pub free_block_count: usize,
    /// Total amount of free memory.
    pub total_free_memory: usize,
    /// External fragmentation percentage.
    pub external_fragmentation: f64,
    /// Internal fragmentation percentage.
    pub internal_fragmentation: f64,
}

/// System library usage statistics.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SystemLibraryStats {
    /// Usage statistics for standard collections.
    pub std_collections: LibraryUsage,
    /// Usage statistics for async runtime.
    pub async_runtime: LibraryUsage,
    /// Usage statistics for network I/O.
    pub network_io: LibraryUsage,
    /// Usage statistics for file system operations.
    pub file_system: LibraryUsage,
    /// Usage statistics for serialization.
    pub serialization: LibraryUsage,
    /// Usage statistics for regex operations.
    pub regex_engine: LibraryUsage,
    /// Usage statistics for cryptographic operations.
    pub crypto_security: LibraryUsage,
    /// Usage statistics for database operations.
    pub database: LibraryUsage,
    /// Usage statistics for graphics and UI.
    pub graphics_ui: LibraryUsage,
    /// Usage statistics for HTTP operations.
    pub http_stack: LibraryUsage,
}

/// Library usage information.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LibraryUsage {
    /// Number of allocations.
    pub allocation_count: usize,
    /// Total bytes allocated.
    pub total_bytes: usize,
    /// Peak memory usage in bytes.
    pub peak_bytes: usize,
    /// Average allocation size.
    pub average_size: f64,
    /// Categorized usage statistics.
    pub categories: HashMap<String, usize>,
    /// Functions with high allocation activity.
    pub hotspot_functions: Vec<String>,
}

/// Concurrency safety analysis.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ConcurrencyAnalysis {
    /// Thread Safety Allocations.
    pub thread_safety_allocations: usize,
    /// Shared Memory Bytes.
    pub shared_memory_bytes: usize,
    /// Mutex Protected.
    pub mutex_protected: usize,
    /// Arc Shared.
    pub arc_shared: usize,
    /// Rc Shared.
    pub rc_shared: usize,
    /// Channel Buffers.
    pub channel_buffers: usize,
    /// Thread Local Storage.
    pub thread_local_storage: usize,
    /// Atomic Operations.
    pub atomic_operations: usize,
    /// Lock Contention Risk.
    pub lock_contention_risk: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats_creation() {
        let stats = MemoryStats::new();

        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.total_allocated, 0);
        assert_eq!(stats.active_allocations, 0);
    }

    #[test]
    fn test_fragmentation_analysis_default() {
        let frag = FragmentationAnalysis::default();

        assert_eq!(frag.fragmentation_ratio, 0.0);
        assert_eq!(frag.largest_free_block, 0);
        assert_eq!(frag.free_block_count, 0);
    }

    #[test]
    fn test_system_library_stats_default() {
        let stats = SystemLibraryStats::default();

        assert_eq!(stats.std_collections.allocation_count, 0);
        assert_eq!(stats.async_runtime.total_bytes, 0);
        assert_eq!(stats.network_io.peak_bytes, 0);
    }

    #[test]
    fn test_library_usage_default() {
        let usage = LibraryUsage::default();

        assert_eq!(usage.allocation_count, 0);
        assert_eq!(usage.total_bytes, 0);
        assert_eq!(usage.average_size, 0.0);
        assert!(usage.categories.is_empty());
        assert!(usage.hotspot_functions.is_empty());
    }

    #[test]
    fn test_concurrency_analysis_default() {
        let analysis = ConcurrencyAnalysis::default();

        assert_eq!(analysis.thread_safety_allocations, 0);
        assert_eq!(analysis.shared_memory_bytes, 0);
        assert_eq!(analysis.mutex_protected, 0);
    }
}
