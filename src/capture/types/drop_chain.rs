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

// Implement From trait for converting from core::types to capture::types
impl From<crate::core::types::DropChainAnalysis> for DropChainAnalysis {
    fn from(old: crate::core::types::DropChainAnalysis) -> Self {
        fn convert_drop_node(old: crate::core::types::DropChainNode) -> DropChainNode {
            DropChainNode {
                object_id: old.object_id,
                type_name: old.type_name,
                drop_timestamp: old.drop_timestamp,
                drop_duration_ns: old.drop_duration_ns,
                children: old.children.into_iter().map(convert_drop_node).collect(),
                drop_impl_type: match old.drop_impl_type {
                    crate::core::types::DropImplementationType::Automatic => {
                        DropImplementationType::Automatic
                    }
                    crate::core::types::DropImplementationType::Custom => {
                        DropImplementationType::Custom
                    }
                    crate::core::types::DropImplementationType::SmartPointer => {
                        DropImplementationType::SmartPointer
                    }
                    crate::core::types::DropImplementationType::Collection => {
                        DropImplementationType::Collection
                    }
                    crate::core::types::DropImplementationType::ResourceHandle => {
                        DropImplementationType::ResourceHandle
                    }
                    crate::core::types::DropImplementationType::NoOp => {
                        DropImplementationType::NoOp
                    }
                },
                cleanup_actions: old
                    .cleanup_actions
                    .into_iter()
                    .map(|a| CleanupAction {
                        action_type: match a.action_type {
                            crate::core::types::CleanupActionType::MemoryDeallocation => {
                                CleanupActionType::MemoryDeallocation
                            }
                            crate::core::types::CleanupActionType::FileHandleClosure => {
                                CleanupActionType::FileHandleClosure
                            }
                            crate::core::types::CleanupActionType::NetworkConnectionClosure => {
                                CleanupActionType::NetworkConnectionClosure
                            }
                            crate::core::types::CleanupActionType::LockRelease => {
                                CleanupActionType::LockRelease
                            }
                            crate::core::types::CleanupActionType::ThreadCleanup => {
                                CleanupActionType::ThreadCleanup
                            }
                            crate::core::types::CleanupActionType::ReferenceCountDecrement => {
                                CleanupActionType::ReferenceCountDecrement
                            }
                            crate::core::types::CleanupActionType::CustomCleanup => {
                                CleanupActionType::CustomCleanup
                            }
                        },
                        timestamp: a.timestamp,
                        duration_ns: a.duration_ns,
                        resource_description: a.resource_description,
                        success: a.success,
                    })
                    .collect(),
                performance_characteristics: DropPerformanceCharacteristics {
                    execution_time_ns: old.performance_characteristics.execution_time_ns,
                    cpu_usage_percent: old.performance_characteristics.cpu_usage_percent,
                    memory_operations: old.performance_characteristics.memory_operations,
                    io_operations: old.performance_characteristics.io_operations,
                    system_calls: old.performance_characteristics.system_calls,
                    impact_level: match old.performance_characteristics.impact_level {
                        crate::core::types::ImpactLevel::Low => ImpactLevel::Low,
                        crate::core::types::ImpactLevel::Medium => ImpactLevel::Medium,
                        crate::core::types::ImpactLevel::High => ImpactLevel::High,
                        crate::core::types::ImpactLevel::Critical => ImpactLevel::Critical,
                    },
                },
            }
        }

        Self {
            root_object: convert_drop_node(old.root_object),
            drop_sequence: old
                .drop_sequence
                .into_iter()
                .map(convert_drop_node)
                .collect(),
            total_duration_ns: old.total_duration_ns,
            performance_metrics: DropChainPerformanceMetrics {
                total_objects: old.performance_metrics.total_objects,
                max_depth: old.performance_metrics.max_depth,
                avg_drop_time_ns: old.performance_metrics.avg_drop_time_ns,
                slowest_drop_ns: old.performance_metrics.slowest_drop_ns,
                efficiency_score: old.performance_metrics.efficiency_score,
                bottlenecks: old
                    .performance_metrics
                    .bottlenecks
                    .into_iter()
                    .map(|b| DropPerformanceBottleneck {
                        object_id: b.object_id,
                        bottleneck_type: match b.bottleneck_type {
                            crate::core::types::DropBottleneckType::SlowCustomDrop => {
                                DropBottleneckType::SlowCustomDrop
                            }
                            crate::core::types::DropBottleneckType::DeepOwnershipHierarchy => {
                                DropBottleneckType::DeepOwnershipHierarchy
                            }
                            crate::core::types::DropBottleneckType::LargeCollectionCleanup => {
                                DropBottleneckType::LargeCollectionCleanup
                            }
                            crate::core::types::DropBottleneckType::ResourceHandleDelay => {
                                DropBottleneckType::ResourceHandleDelay
                            }
                            crate::core::types::DropBottleneckType::LockContention => {
                                DropBottleneckType::LockContention
                            }
                            crate::core::types::DropBottleneckType::MemoryFragmentation => {
                                DropBottleneckType::MemoryFragmentation
                            }
                        },
                        severity: match b.severity {
                            crate::core::types::ImpactLevel::Low => ImpactLevel::Low,
                            crate::core::types::ImpactLevel::Medium => ImpactLevel::Medium,
                            crate::core::types::ImpactLevel::High => ImpactLevel::High,
                            crate::core::types::ImpactLevel::Critical => ImpactLevel::Critical,
                        },
                        description: b.description,
                        optimization_suggestion: b.optimization_suggestion,
                    })
                    .collect(),
            },
            ownership_hierarchy: old.ownership_hierarchy.into(),
            leak_detection: old.leak_detection.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify DropChainNode creation with all fields
    /// Invariants: All fields should be properly initialized
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

        assert_eq!(node.object_id, 1, "Object ID should match");
        assert_eq!(node.type_name, "String", "Type name should match");
        assert_eq!(
            node.drop_impl_type,
            DropImplementationType::Automatic,
            "Drop impl type should be Automatic"
        );
    }

    /// Objective: Verify DropImplementationType variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_drop_implementation_type() {
        let auto = DropImplementationType::Automatic;
        let custom = DropImplementationType::Custom;
        assert!(
            matches!(auto, DropImplementationType::Automatic),
            "Should match Automatic"
        );
        assert!(
            matches!(custom, DropImplementationType::Custom),
            "Should match Custom"
        );
    }

    /// Objective: Verify all DropImplementationType variants
    /// Invariants: All variants should have debug representation
    #[test]
    fn test_drop_implementation_type_all_variants() {
        let variants = vec![
            DropImplementationType::Automatic,
            DropImplementationType::Custom,
            DropImplementationType::SmartPointer,
            DropImplementationType::Collection,
            DropImplementationType::ResourceHandle,
            DropImplementationType::NoOp,
        ];

        for variant in &variants {
            let debug_str = format!("{variant:?}");
            assert!(
                !debug_str.is_empty(),
                "Variant should have debug representation"
            );
        }
    }

    /// Objective: Verify DropChainNode with nested children
    /// Invariants: Should handle nested drop chain hierarchy
    #[test]
    fn test_drop_chain_node_nested() {
        let child = DropChainNode {
            object_id: 2,
            type_name: "i32".to_string(),
            drop_timestamp: 1001,
            drop_duration_ns: 100,
            children: vec![],
            drop_impl_type: DropImplementationType::Automatic,
            cleanup_actions: vec![],
            performance_characteristics: DropPerformanceCharacteristics {
                execution_time_ns: 100,
                cpu_usage_percent: 0.0,
                memory_operations: 0,
                io_operations: 0,
                system_calls: 0,
                impact_level: ImpactLevel::Low,
            },
        };

        let parent = DropChainNode {
            object_id: 1,
            type_name: "Vec<i32>".to_string(),
            drop_timestamp: 1000,
            drop_duration_ns: 500,
            children: vec![child],
            drop_impl_type: DropImplementationType::Collection,
            cleanup_actions: vec![],
            performance_characteristics: DropPerformanceCharacteristics {
                execution_time_ns: 500,
                cpu_usage_percent: 0.1,
                memory_operations: 10,
                io_operations: 0,
                system_calls: 0,
                impact_level: ImpactLevel::Low,
            },
        };

        assert_eq!(parent.children.len(), 1, "Parent should have one child");
        assert_eq!(
            parent.children[0].object_id, 2,
            "Child object ID should match"
        );
    }

    /// Objective: Verify CleanupAction creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_cleanup_action() {
        let action = CleanupAction {
            action_type: CleanupActionType::MemoryDeallocation,
            timestamp: 1000,
            duration_ns: 500,
            resource_description: "Heap memory block".to_string(),
            success: true,
        };

        assert_eq!(
            action.action_type,
            CleanupActionType::MemoryDeallocation,
            "Action type should match"
        );
        assert!(action.success, "Action should be successful");
    }

    /// Objective: Verify CleanupActionType variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_cleanup_action_type_variants() {
        let variants = vec![
            CleanupActionType::MemoryDeallocation,
            CleanupActionType::FileHandleClosure,
            CleanupActionType::NetworkConnectionClosure,
            CleanupActionType::LockRelease,
            CleanupActionType::ThreadCleanup,
            CleanupActionType::ReferenceCountDecrement,
            CleanupActionType::CustomCleanup,
        ];

        for variant in &variants {
            let debug_str = format!("{variant:?}");
            assert!(
                !debug_str.is_empty(),
                "CleanupActionType should have debug representation"
            );
        }
    }

    /// Objective: Verify DropChainPerformanceMetrics creation
    /// Invariants: All fields should be properly initialized
    #[test]
    fn test_drop_chain_performance_metrics() {
        let metrics = DropChainPerformanceMetrics {
            total_objects: 100,
            max_depth: 5,
            avg_drop_time_ns: 1000.0,
            slowest_drop_ns: 5000,
            efficiency_score: 85.0,
            bottlenecks: vec![],
        };

        assert_eq!(metrics.total_objects, 100, "Total objects should match");
        assert_eq!(metrics.max_depth, 5, "Max depth should match");
        assert_eq!(
            metrics.efficiency_score, 85.0,
            "Efficiency score should match"
        );
    }

    /// Objective: Verify DropPerformanceBottleneck creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_drop_performance_bottleneck() {
        let bottleneck = DropPerformanceBottleneck {
            object_id: 42,
            bottleneck_type: DropBottleneckType::SlowCustomDrop,
            severity: ImpactLevel::High,
            description: "Custom drop takes too long".to_string(),
            optimization_suggestion: "Consider async cleanup".to_string(),
        };

        assert_eq!(bottleneck.object_id, 42, "Object ID should match");
        assert_eq!(
            bottleneck.bottleneck_type,
            DropBottleneckType::SlowCustomDrop,
            "Bottleneck type should match"
        );
        assert_eq!(
            bottleneck.severity,
            ImpactLevel::High,
            "Severity should be High"
        );
    }

    /// Objective: Verify DropBottleneckType variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_drop_bottleneck_type_variants() {
        let variants = vec![
            DropBottleneckType::SlowCustomDrop,
            DropBottleneckType::DeepOwnershipHierarchy,
            DropBottleneckType::LargeCollectionCleanup,
            DropBottleneckType::ResourceHandleDelay,
            DropBottleneckType::LockContention,
            DropBottleneckType::MemoryFragmentation,
        ];

        for variant in &variants {
            let debug_str = format!("{variant:?}");
            assert!(
                !debug_str.is_empty(),
                "DropBottleneckType should have debug representation"
            );
        }
    }

    /// Objective: Verify DropPerformanceCharacteristics creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_drop_performance_characteristics() {
        let chars = DropPerformanceCharacteristics {
            execution_time_ns: 1000,
            cpu_usage_percent: 50.0,
            memory_operations: 10,
            io_operations: 2,
            system_calls: 1,
            impact_level: ImpactLevel::Medium,
        };

        assert_eq!(chars.execution_time_ns, 1000, "Execution time should match");
        assert_eq!(chars.cpu_usage_percent, 50.0, "CPU usage should match");
        assert_eq!(
            chars.impact_level,
            ImpactLevel::Medium,
            "Impact level should be Medium"
        );
    }

    /// Objective: Verify ImpactLevel variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_impact_level_variants() {
        assert_eq!(ImpactLevel::Low, ImpactLevel::Low);
        assert_eq!(ImpactLevel::Medium, ImpactLevel::Medium);
        assert_eq!(ImpactLevel::High, ImpactLevel::High);
        assert_eq!(ImpactLevel::Critical, ImpactLevel::Critical);

        assert_ne!(ImpactLevel::Low, ImpactLevel::Critical);
    }

    /// Objective: Verify DropChainAnalysis creation
    /// Invariants: All fields should be properly initialized
    #[test]
    fn test_drop_chain_analysis() {
        let root = DropChainNode {
            object_id: 1,
            type_name: "Root".to_string(),
            drop_timestamp: 1000,
            drop_duration_ns: 500,
            children: vec![],
            drop_impl_type: DropImplementationType::Custom,
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

        let analysis = DropChainAnalysis {
            root_object: root.clone(),
            drop_sequence: vec![root],
            total_duration_ns: 1000,
            performance_metrics: DropChainPerformanceMetrics {
                total_objects: 1,
                max_depth: 1,
                avg_drop_time_ns: 500.0,
                slowest_drop_ns: 500,
                efficiency_score: 100.0,
                bottlenecks: vec![],
            },
            ownership_hierarchy: OwnershipHierarchy {
                root_owners: vec![],
                max_depth: 0,
                total_objects: 0,
                transfer_events: vec![],
                weak_references: vec![],
                circular_references: vec![],
            },
            leak_detection: ResourceLeakAnalysis {
                potential_leaks: vec![],
                detection_confidence: 1.0,
                usage_patterns: vec![],
                prevention_recommendations: vec![],
            },
        };

        assert_eq!(
            analysis.total_duration_ns, 1000,
            "Total duration should match"
        );
        assert_eq!(
            analysis.drop_sequence.len(),
            1,
            "Drop sequence should have one node"
        );
    }

    /// Objective: Verify serialization of DropImplementationType
    /// Invariants: Should serialize and deserialize correctly
    #[test]
    fn test_drop_implementation_type_serialization() {
        let drop_type = DropImplementationType::SmartPointer;
        let json = serde_json::to_string(&drop_type);
        assert!(json.is_ok(), "Should serialize to JSON");

        let deserialized: Result<DropImplementationType, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "Should deserialize from JSON");
        assert_eq!(
            deserialized.unwrap(),
            DropImplementationType::SmartPointer,
            "Should preserve value"
        );
    }

    /// Objective: Verify serialization of CleanupActionType
    /// Invariants: Should serialize and deserialize correctly
    #[test]
    fn test_cleanup_action_type_serialization() {
        let action_type = CleanupActionType::FileHandleClosure;
        let json = serde_json::to_string(&action_type);
        assert!(json.is_ok(), "Should serialize to JSON");

        let deserialized: Result<CleanupActionType, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "Should deserialize from JSON");
        assert_eq!(
            deserialized.unwrap(),
            CleanupActionType::FileHandleClosure,
            "Should preserve value"
        );
    }

    /// Objective: Verify DropChainNode clone functionality
    /// Invariants: Cloned node should have same values
    #[test]
    fn test_drop_chain_node_clone() {
        let original = DropChainNode {
            object_id: 1,
            type_name: "Test".to_string(),
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

        let cloned = original.clone();

        assert_eq!(
            original.object_id, cloned.object_id,
            "Object ID should match"
        );
        assert_eq!(
            original.type_name, cloned.type_name,
            "Type name should match"
        );
    }

    /// Objective: Verify DropChainPerformanceMetrics with bottlenecks
    /// Invariants: Should handle multiple bottlenecks
    #[test]
    fn test_drop_chain_performance_metrics_with_bottlenecks() {
        let bottleneck1 = DropPerformanceBottleneck {
            object_id: 1,
            bottleneck_type: DropBottleneckType::SlowCustomDrop,
            severity: ImpactLevel::High,
            description: "Slow drop".to_string(),
            optimization_suggestion: "Optimize".to_string(),
        };

        let bottleneck2 = DropPerformanceBottleneck {
            object_id: 2,
            bottleneck_type: DropBottleneckType::LargeCollectionCleanup,
            severity: ImpactLevel::Medium,
            description: "Large collection".to_string(),
            optimization_suggestion: "Use smaller batches".to_string(),
        };

        let metrics = DropChainPerformanceMetrics {
            total_objects: 100,
            max_depth: 5,
            avg_drop_time_ns: 1000.0,
            slowest_drop_ns: 5000,
            efficiency_score: 60.0,
            bottlenecks: vec![bottleneck1, bottleneck2],
        };

        assert_eq!(metrics.bottlenecks.len(), 2, "Should have two bottlenecks");
    }

    /// Objective: Verify CleanupAction with failure status
    /// Invariants: Should handle failed cleanup actions
    #[test]
    fn test_cleanup_action_failure() {
        let action = CleanupAction {
            action_type: CleanupActionType::FileHandleClosure,
            timestamp: 1000,
            duration_ns: 500,
            resource_description: "Failed to close file".to_string(),
            success: false,
        };

        assert!(!action.success, "Action should be marked as failed");
    }

    /// Objective: Verify DropPerformanceCharacteristics with high impact
    /// Invariants: Should handle critical impact level
    #[test]
    fn test_drop_performance_characteristics_critical() {
        let chars = DropPerformanceCharacteristics {
            execution_time_ns: 1000000,
            cpu_usage_percent: 100.0,
            memory_operations: 1000,
            io_operations: 100,
            system_calls: 50,
            impact_level: ImpactLevel::Critical,
        };

        assert_eq!(
            chars.impact_level,
            ImpactLevel::Critical,
            "Impact level should be Critical"
        );
        assert_eq!(chars.cpu_usage_percent, 100.0, "CPU usage should be 100%");
    }
}
