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

impl From<crate::core::types::MemoryStats> for MemoryStats {
    fn from(old: crate::core::types::MemoryStats) -> Self {
        Self {
            total_allocations: old.total_allocations,
            total_allocated: old.total_allocated,
            active_allocations: old.active_allocations,
            active_memory: old.active_memory,
            peak_allocations: old.peak_allocations,
            peak_memory: old.peak_memory,
            total_deallocations: old.total_deallocations,
            total_deallocated: old.total_deallocated,
            leaked_allocations: old.leaked_allocations,
            leaked_memory: old.leaked_memory,
            // Convert FragmentationAnalysis
            fragmentation_analysis: FragmentationAnalysis {
                fragmentation_ratio: old.fragmentation_analysis.fragmentation_ratio,
                largest_free_block: old.fragmentation_analysis.largest_free_block,
                smallest_free_block: old.fragmentation_analysis.smallest_free_block,
                free_block_count: old.fragmentation_analysis.free_block_count,
                total_free_memory: old.fragmentation_analysis.total_free_memory,
                external_fragmentation: old.fragmentation_analysis.external_fragmentation,
                internal_fragmentation: old.fragmentation_analysis.internal_fragmentation,
            },
            // Use default values for complex nested types to avoid unsafe conversions
            lifecycle_stats: ScopeLifecycleMetrics::default(),
            allocations: old.allocations.into_iter().map(|a| a.into()).collect(),
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

    #[test]
    fn test_memory_stats_default() {
        let stats = MemoryStats::default();

        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.active_memory, 0);
        assert_eq!(stats.peak_memory, 0);
        assert_eq!(stats.leaked_allocations, 0);
    }

    #[test]
    fn test_memory_stats_with_values() {
        let mut stats = MemoryStats::new();
        stats.total_allocations = 100;
        stats.total_allocated = 1024 * 1024;
        stats.active_allocations = 50;
        stats.active_memory = 512 * 1024;
        stats.peak_allocations = 75;
        stats.peak_memory = 768 * 1024;
        stats.total_deallocations = 50;
        stats.total_deallocated = 512 * 1024;
        stats.leaked_allocations = 5;
        stats.leaked_memory = 10240;

        assert_eq!(stats.total_allocations, 100);
        assert_eq!(stats.active_allocations, 50);
        assert_eq!(stats.leaked_allocations, 5);
    }

    #[test]
    fn test_fragmentation_analysis_with_values() {
        let frag = FragmentationAnalysis {
            fragmentation_ratio: 0.35,
            largest_free_block: 65536,
            smallest_free_block: 16,
            free_block_count: 128,
            total_free_memory: 524288,
            external_fragmentation: 0.25,
            internal_fragmentation: 0.10,
        };

        assert!((frag.fragmentation_ratio - 0.35).abs() < f64::EPSILON);
        assert_eq!(frag.free_block_count, 128);
        assert!((frag.external_fragmentation - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_memory_type_info_creation() {
        let info = MemoryTypeInfo {
            type_name: "Vec<u8>".to_string(),
            total_size: 1024,
            allocation_count: 10,
            average_size: 102,
            largest_allocation: 512,
            smallest_allocation: 16,
            active_instances: 8,
            leaked_instances: 1,
        };

        assert_eq!(info.type_name, "Vec<u8>");
        assert_eq!(info.allocation_count, 10);
        assert_eq!(info.leaked_instances, 1);
    }

    #[test]
    fn test_type_memory_usage_creation() {
        let usage = TypeMemoryUsage {
            type_name: "String".to_string(),
            total_size: 2048,
            allocation_count: 20,
            average_size: 102.4,
            peak_size: 512,
            current_size: 256,
            efficiency_score: 0.85,
        };

        assert_eq!(usage.type_name, "String");
        assert!((usage.average_size - 102.4).abs() < f64::EPSILON);
        assert!((usage.efficiency_score - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_library_usage_with_values() {
        let mut categories = HashMap::new();
        categories.insert("HashMap".to_string(), 1000);
        categories.insert("Vec".to_string(), 2000);

        let usage = LibraryUsage {
            allocation_count: 100,
            total_bytes: 10240,
            peak_bytes: 5120,
            average_size: 102.4,
            categories,
            hotspot_functions: vec!["push".to_string(), "insert".to_string()],
        };

        assert_eq!(usage.allocation_count, 100);
        assert_eq!(usage.categories.len(), 2);
        assert_eq!(usage.hotspot_functions.len(), 2);
    }

    #[test]
    fn test_system_library_stats_with_values() {
        let mut stats = SystemLibraryStats::default();
        stats.std_collections.allocation_count = 500;
        stats.async_runtime.total_bytes = 10240;
        stats.network_io.peak_bytes = 2048;
        stats.file_system.allocation_count = 100;
        stats.serialization.total_bytes = 4096;
        stats.regex_engine.allocation_count = 50;
        stats.crypto_security.total_bytes = 8192;
        stats.database.allocation_count = 200;
        stats.graphics_ui.total_bytes = 16384;
        stats.http_stack.allocation_count = 75;

        assert_eq!(stats.std_collections.allocation_count, 500);
        assert_eq!(stats.async_runtime.total_bytes, 10240);
    }

    #[test]
    fn test_concurrency_analysis_with_values() {
        let analysis = ConcurrencyAnalysis {
            thread_safety_allocations: 100,
            shared_memory_bytes: 10240,
            mutex_protected: 50,
            arc_shared: 30,
            rc_shared: 20,
            channel_buffers: 15,
            thread_local_storage: 10,
            atomic_operations: 200,
            lock_contention_risk: "Low".to_string(),
        };

        assert_eq!(analysis.thread_safety_allocations, 100);
        assert_eq!(analysis.arc_shared, 30);
        assert_eq!(analysis.lock_contention_risk, "Low");
    }

    #[test]
    fn test_memory_stats_serialization() {
        let stats = MemoryStats::new();

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("total_allocations"));
        assert!(json.contains("active_memory"));
    }

    #[test]
    fn test_fragmentation_analysis_serialization() {
        let frag = FragmentationAnalysis {
            fragmentation_ratio: 0.5,
            largest_free_block: 1024,
            smallest_free_block: 16,
            free_block_count: 50,
            total_free_memory: 2048,
            external_fragmentation: 0.3,
            internal_fragmentation: 0.2,
        };

        let json = serde_json::to_string(&frag).unwrap();
        let deserialized: FragmentationAnalysis = serde_json::from_str(&json).unwrap();
        assert!((deserialized.fragmentation_ratio - frag.fragmentation_ratio).abs() < f64::EPSILON);
    }

    #[test]
    fn test_memory_type_info_serialization() {
        let info = MemoryTypeInfo {
            type_name: "HashMap<String, i32>".to_string(),
            total_size: 4096,
            allocation_count: 25,
            average_size: 163,
            largest_allocation: 1024,
            smallest_allocation: 64,
            active_instances: 20,
            leaked_instances: 2,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("HashMap"));
        assert!(json.contains("4096"));
    }

    #[test]
    fn test_type_memory_usage_serialization() {
        let usage = TypeMemoryUsage {
            type_name: "Box<dyn Any>".to_string(),
            total_size: 8192,
            allocation_count: 100,
            average_size: 81.92,
            peak_size: 2048,
            current_size: 1024,
            efficiency_score: 0.75,
        };

        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("Box"));
        assert!(json.contains("8192"));
    }

    #[test]
    fn test_library_usage_serialization() {
        let usage = LibraryUsage {
            allocation_count: 50,
            total_bytes: 5120,
            peak_bytes: 2560,
            average_size: 102.4,
            categories: HashMap::new(),
            hotspot_functions: vec![],
        };

        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("allocation_count"));
    }

    #[test]
    fn test_concurrency_analysis_serialization() {
        let analysis = ConcurrencyAnalysis {
            thread_safety_allocations: 10,
            shared_memory_bytes: 1024,
            mutex_protected: 5,
            arc_shared: 3,
            rc_shared: 2,
            channel_buffers: 1,
            thread_local_storage: 0,
            atomic_operations: 50,
            lock_contention_risk: "Medium".to_string(),
        };

        let json = serde_json::to_string(&analysis).unwrap();
        assert!(json.contains("thread_safety_allocations"));
    }

    #[test]
    fn test_boundary_values_memory_stats() {
        let mut stats = MemoryStats::new();
        stats.total_allocations = usize::MAX;
        stats.total_allocated = usize::MAX;
        stats.active_allocations = usize::MAX;
        stats.active_memory = usize::MAX;
        stats.peak_allocations = usize::MAX;
        stats.peak_memory = usize::MAX;

        assert_eq!(stats.total_allocations, usize::MAX);
        assert_eq!(stats.peak_memory, usize::MAX);
    }

    #[test]
    fn test_boundary_values_fragmentation() {
        let frag = FragmentationAnalysis {
            fragmentation_ratio: f64::MAX,
            largest_free_block: usize::MAX,
            smallest_free_block: usize::MAX,
            free_block_count: usize::MAX,
            total_free_memory: usize::MAX,
            external_fragmentation: f64::MAX,
            internal_fragmentation: f64::MAX,
        };

        assert!(frag.fragmentation_ratio.is_finite());
        assert_eq!(frag.largest_free_block, usize::MAX);
    }

    #[test]
    fn test_boundary_values_concurrency() {
        let analysis = ConcurrencyAnalysis {
            thread_safety_allocations: usize::MAX,
            shared_memory_bytes: usize::MAX,
            mutex_protected: usize::MAX,
            arc_shared: usize::MAX,
            rc_shared: usize::MAX,
            channel_buffers: usize::MAX,
            thread_local_storage: usize::MAX,
            atomic_operations: usize::MAX,
            lock_contention_risk: String::new(),
        };

        assert_eq!(analysis.thread_safety_allocations, usize::MAX);
        assert_eq!(analysis.atomic_operations, usize::MAX);
    }

    #[test]
    fn test_memory_stats_clone() {
        let mut stats = MemoryStats::new();
        stats.total_allocations = 42;

        let cloned = stats.clone();
        assert_eq!(cloned.total_allocations, 42);
    }

    #[test]
    fn test_memory_stats_debug() {
        let stats = MemoryStats::new();
        let debug_str = format!("{:?}", stats);

        assert!(debug_str.contains("MemoryStats"));
        assert!(debug_str.contains("total_allocations"));
    }

    #[test]
    fn test_fragmentation_analysis_clone() {
        let frag = FragmentationAnalysis {
            fragmentation_ratio: 0.75,
            largest_free_block: 2048,
            smallest_free_block: 32,
            free_block_count: 100,
            total_free_memory: 4096,
            external_fragmentation: 0.5,
            internal_fragmentation: 0.25,
        };

        let cloned = frag.clone();
        assert!((cloned.fragmentation_ratio - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_library_usage_clone() {
        let usage = LibraryUsage {
            allocation_count: 25,
            total_bytes: 2560,
            peak_bytes: 1280,
            average_size: 102.4,
            categories: HashMap::new(),
            hotspot_functions: vec!["test".to_string()],
        };

        let cloned = usage.clone();
        assert_eq!(cloned.allocation_count, 25);
    }

    #[test]
    fn test_concurrency_analysis_clone() {
        let analysis = ConcurrencyAnalysis {
            thread_safety_allocations: 100,
            shared_memory_bytes: 2048,
            mutex_protected: 50,
            arc_shared: 25,
            rc_shared: 15,
            channel_buffers: 10,
            thread_local_storage: 5,
            atomic_operations: 200,
            lock_contention_risk: "High".to_string(),
        };

        let cloned = analysis.clone();
        assert_eq!(cloned.thread_safety_allocations, 100);
    }

    #[test]
    fn test_empty_library_usage_categories() {
        let usage = LibraryUsage::default();
        assert!(usage.categories.is_empty());
        assert!(usage.hotspot_functions.is_empty());
    }

    #[test]
    fn test_library_usage_with_categories() {
        let mut categories = HashMap::new();
        categories.insert("String".to_string(), 500);
        categories.insert("Vec".to_string(), 1000);

        let usage = LibraryUsage {
            allocation_count: 10,
            total_bytes: 1500,
            peak_bytes: 750,
            average_size: 150.0,
            categories,
            hotspot_functions: vec![],
        };

        assert_eq!(usage.categories.get("String"), Some(&500));
        assert_eq!(usage.categories.get("Vec"), Some(&1000));
        assert_eq!(usage.categories.get("HashMap"), None);
    }
}
