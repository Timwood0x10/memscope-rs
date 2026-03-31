//! Drop chain analysis types.
//!
//! This module contains types for analyzing drop chains,
//! cleanup actions, and drop performance.

use serde::{Deserialize, Serialize};

use super::allocation::ImpactLevel;
use super::leak_detection::ResourceLeakAnalysis;
use super::ownership::OwnershipHierarchy;

/// Drop chain analysis information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DropChainAnalysis {
    /// Root object that initiated the drop chain.
    pub root_object: DropChainNode,
    /// Complete drop chain sequence.
    pub drop_sequence: Vec<DropChainNode>,
    /// Total drop chain duration in nanoseconds.
    pub total_duration_ns: u64,
    /// Drop chain performance metrics.
    pub performance_metrics: DropChainPerformanceMetrics,
    /// Ownership hierarchy analysis.
    pub ownership_hierarchy: OwnershipHierarchy,
    /// Resource leak detection results.
    pub leak_detection: ResourceLeakAnalysis,
}

/// Individual node in the drop chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DropChainNode {
    /// Object identifier.
    pub object_id: usize,
    /// Object type name.
    pub type_name: String,
    /// Drop timestamp.
    pub drop_timestamp: u64,
    /// Drop duration in nanoseconds.
    pub drop_duration_ns: u64,
    /// Children objects dropped as part of this drop.
    pub children: Vec<DropChainNode>,
    /// Drop implementation type.
    pub drop_impl_type: DropImplementationType,
    /// Resource cleanup actions performed.
    pub cleanup_actions: Vec<CleanupAction>,
    /// Drop performance characteristics.
    pub performance_characteristics: DropPerformanceCharacteristics,
}

/// Types of Drop implementations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DropImplementationType {
    /// Automatic drop (compiler generated).
    Automatic,
    /// Custom Drop trait implementation.
    Custom,
    /// Smart pointer drop (Box, Rc, Arc).
    SmartPointer,
    /// Collection drop (Vec, HashMap, etc.).
    Collection,
    /// Resource handle drop (File, Socket, etc.).
    ResourceHandle,
    /// No-op drop (for Copy types).
    NoOp,
}

/// Cleanup action performed during drop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CleanupAction {
    /// Action type.
    pub action_type: CleanupActionType,
    /// Action timestamp.
    pub timestamp: u64,
    /// Action duration in nanoseconds.
    pub duration_ns: u64,
    /// Resource being cleaned up.
    pub resource_description: String,
    /// Success status.
    pub success: bool,
}

/// Types of cleanup actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CleanupActionType {
    /// Memory deallocation.
    MemoryDeallocation,
    /// File handle closure.
    FileHandleClosure,
    /// Network connection closure.
    NetworkConnectionClosure,
    /// Mutex/lock release.
    LockRelease,
    /// Thread join/cleanup.
    ThreadCleanup,
    /// Reference count decrement.
    ReferenceCountDecrement,
    /// Custom cleanup logic.
    CustomCleanup,
}

/// Drop chain performance metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DropChainPerformanceMetrics {
    /// Total objects in chain.
    pub total_objects: usize,
    /// Maximum chain depth.
    pub max_depth: usize,
    /// Average drop time per object.
    pub avg_drop_time_ns: f64,
    /// Slowest drop in chain.
    pub slowest_drop_ns: u64,
    /// Drop chain efficiency score (0-100).
    pub efficiency_score: f64,
    /// Performance bottlenecks identified.
    pub bottlenecks: Vec<DropPerformanceBottleneck>,
}

/// Drop performance bottleneck.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DropPerformanceBottleneck {
    /// Object causing the bottleneck.
    pub object_id: usize,
    /// Bottleneck type.
    pub bottleneck_type: DropBottleneckType,
    /// Impact severity.
    pub severity: ImpactLevel,
    /// Description.
    pub description: String,
    /// Optimization suggestion.
    pub optimization_suggestion: String,
}

/// Types of drop performance bottlenecks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DropBottleneckType {
    /// Slow custom Drop implementation.
    SlowCustomDrop,
    /// Deep ownership hierarchy.
    DeepOwnershipHierarchy,
    /// Large collection cleanup.
    LargeCollectionCleanup,
    /// Resource handle cleanup delay.
    ResourceHandleDelay,
    /// Lock contention during drop.
    LockContention,
    /// Memory fragmentation during cleanup.
    MemoryFragmentation,
}

/// Drop performance characteristics for individual objects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DropPerformanceCharacteristics {
    /// Drop execution time in nanoseconds.
    pub execution_time_ns: u64,
    /// CPU usage during drop.
    pub cpu_usage_percent: f64,
    /// Memory operations performed.
    pub memory_operations: u32,
    /// I/O operations performed.
    pub io_operations: u32,
    /// System calls made.
    pub system_calls: u32,
    /// Performance impact level.
    pub impact_level: ImpactLevel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drop_chain_node() {
        let node = DropChainNode {
            object_id: 1,
            type_name: "String".to_string(),
            drop_timestamp: 1000,
            drop_duration_ns: 500,
            children: vec![],
            drop_impl_type: DropImplementationType::Automatic,
            cleanup_actions: vec![],
            performance_characteristics: DropPerformanceCharacteristics {
                execution_time_ns: 500,
                cpu_usage_percent: 0.1,
                memory_operations: 1,
                io_operations: 0,
                system_calls: 0,
                impact_level: ImpactLevel::Low,
            },
        };

        assert_eq!(node.object_id, 1);
        assert_eq!(node.type_name, "String");
    }

    #[test]
    fn test_drop_implementation_type() {
        let auto = DropImplementationType::Automatic;
        let custom = DropImplementationType::Custom;
        assert!(matches!(auto, DropImplementationType::Automatic));
        assert!(matches!(custom, DropImplementationType::Custom));
    }
}
