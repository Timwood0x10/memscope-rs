//! Object lifecycle tracking types.
//!
//! This module contains types for tracking object lifecycles,
//! including lifecycle events, patterns, and efficiency metrics.

use serde::{Deserialize, Serialize};

use super::generic::SourceLocation;

/// Object lifecycle event tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectLifecycleInfo {
    /// Object identifier.
    pub object_id: usize,
    /// Object type name.
    pub type_name: String,
    /// Lifecycle events.
    pub lifecycle_events: Vec<LifecycleEvent>,
    /// Total lifetime duration.
    pub total_lifetime_ns: Option<u64>,
    /// Lifecycle stage durations.
    pub stage_durations: LifecycleStageDurations,
    /// Lifecycle efficiency metrics.
    pub efficiency_metrics: LifecycleEfficiencyMetrics,
    /// Lifecycle patterns.
    pub lifecycle_patterns: Vec<LifecyclePattern>,
}

/// Lifecycle event information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecycleEvent {
    /// Event type.
    pub event_type: LifecycleEventType,
    /// Timestamp when event occurred.
    pub timestamp: u64,
    /// Location where event occurred.
    pub location: SourceLocation,
    /// Memory state at event time.
    pub memory_state: MemoryState,
    /// Performance metrics at event time.
    pub performance_metrics: EventPerformanceMetrics,
    /// Call stack at event time.
    pub call_stack: Vec<String>,
}

/// Types of lifecycle events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LifecycleEventType {
    /// Object creation.
    Creation,
    /// Object initialization.
    Initialization,
    /// Object first use.
    FirstUse,
    /// Object move.
    Move,
    /// Object copy.
    Copy,
    /// Object clone.
    Clone,
    /// Object borrow.
    Borrow,
    /// Object mutable borrow.
    MutableBorrow,
    /// Object borrow release.
    BorrowRelease,
    /// Object modification.
    Modification,
    /// Object last use.
    LastUse,
    /// Object drop.
    Drop,
    /// Object destruction.
    Destruction,
    /// Object memory reclaim.
    MemoryReclaim,
}

/// Memory state at event time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryState {
    /// Memory location.
    pub memory_location: MemoryLocationType,
    /// Memory address.
    pub memory_address: usize,
    /// Object size.
    pub object_size: usize,
    /// Reference count (if applicable).
    pub reference_count: Option<u32>,
    /// Borrow state.
    pub borrow_state: BorrowState,
}

/// Borrow state information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BorrowState {
    /// Object is not borrowed.
    NotBorrowed,
    /// Object is shared borrowed.
    SharedBorrow {
        /// Number of shared borrows.
        count: u32,
    },
    /// Object is mutably borrowed.
    MutableBorrow,
    /// Object has been moved out.
    MovedOut,
}

/// Memory location type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryLocationType {
    /// Stack memory location.
    Stack,
    /// Heap memory location.
    Heap,
    /// Register memory location.
    Register,
    /// Static memory location.
    Static,
    /// Thread-local memory location.
    ThreadLocal,
}

/// Performance metrics at event time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventPerformanceMetrics {
    /// CPU cycles consumed by event.
    pub cpu_cycles: u64,
    /// Memory bandwidth used.
    pub memory_bandwidth_bytes: usize,
    /// Cache misses caused by event.
    pub cache_misses: u32,
    /// Event processing time.
    pub processing_time_ns: u64,
}

/// Lifecycle stage durations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecycleStageDurations {
    /// Time from creation to first use.
    pub creation_to_first_use_ns: Option<u64>,
    /// Time spent in active use.
    pub active_use_duration_ns: Option<u64>,
    /// Time from last use to destruction.
    pub last_use_to_destruction_ns: Option<u64>,
    /// Time spent borrowed.
    pub borrowed_duration_ns: u64,
    /// Time spent idle.
    pub idle_duration_ns: u64,
}

/// Lifecycle efficiency metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecycleEfficiencyMetrics {
    /// Utilization ratio (active time / total time).
    pub utilization_ratio: f64,
    /// Memory efficiency (useful operations / memory usage).
    pub memory_efficiency: f64,
    /// Performance efficiency score.
    pub performance_efficiency: f64,
    /// Resource waste assessment.
    pub resource_waste: ResourceWasteAssessment,
}

impl Default for LifecycleEfficiencyMetrics {
    fn default() -> Self {
        Self {
            utilization_ratio: 0.0,
            memory_efficiency: 0.0,
            performance_efficiency: 0.0,
            resource_waste: ResourceWasteAssessment {
                wasted_memory_percent: 0.0,
                wasted_cpu_percent: 0.0,
                premature_destructions: 0,
                unused_instances: 0,
                optimization_opportunities: Vec::new(),
            },
        }
    }
}

/// Resource waste assessment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceWasteAssessment {
    /// Wasted memory percentage.
    pub wasted_memory_percent: f64,
    /// Wasted CPU cycles percentage.
    pub wasted_cpu_percent: f64,
    /// Premature destruction events.
    pub premature_destructions: u32,
    /// Unused object instances.
    pub unused_instances: u32,
    /// Optimization opportunities.
    pub optimization_opportunities: Vec<String>,
}

/// Lifecycle pattern information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecyclePattern {
    /// Pattern type.
    pub pattern_type: LifecyclePatternType,
    /// Pattern frequency.
    pub frequency: u32,
    /// Pattern efficiency.
    pub efficiency_score: f64,
    /// Associated performance impact.
    pub performance_impact: f64,
    /// Optimization suggestions.
    pub optimization_suggestions: Vec<String>,
}

/// Types of lifecycle patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LifecyclePatternType {
    /// Short-lived objects.
    ShortLived,
    /// Long-lived objects.
    LongLived,
    /// Cyclical objects.
    Cyclical,
    /// On-demand objects.
    OnDemand,
    /// Cached objects.
    Cached,
    /// Pooled objects.
    Pooled,
    /// Singleton objects.
    Singleton,
    /// Factory objects.
    Factory,
    /// RAII objects.
    RAII,
}

/// Simple lifecycle pattern classification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SimpleLifecyclePattern {
    /// Very short-lived (0-1ms).
    Instant,
    /// Short-lived (2-100ms).
    ShortLived,
    /// Medium-lived (101ms-10s).
    MediumLived,
    /// Long-lived (10s-5min).
    LongLived,
    /// Persistent (>5min).
    Persistent,
}

impl From<crate::core::types::ObjectLifecycleInfo> for ObjectLifecycleInfo {
    fn from(old: crate::core::types::ObjectLifecycleInfo) -> Self {
        Self {
            object_id: old.object_id,
            type_name: old.type_name,
            lifecycle_events: old
                .lifecycle_events
                .into_iter()
                .map(|e| LifecycleEvent {
                    event_type: match e.event_type {
                        crate::core::types::LifecycleEventType::Creation => {
                            LifecycleEventType::Creation
                        }
                        crate::core::types::LifecycleEventType::Initialization => {
                            LifecycleEventType::Initialization
                        }
                        crate::core::types::LifecycleEventType::FirstUse => {
                            LifecycleEventType::FirstUse
                        }
                        crate::core::types::LifecycleEventType::Move => LifecycleEventType::Move,
                        crate::core::types::LifecycleEventType::Copy => LifecycleEventType::Copy,
                        crate::core::types::LifecycleEventType::Clone => LifecycleEventType::Clone,
                        crate::core::types::LifecycleEventType::Borrow => {
                            LifecycleEventType::Borrow
                        }
                        crate::core::types::LifecycleEventType::MutableBorrow => {
                            LifecycleEventType::MutableBorrow
                        }
                        crate::core::types::LifecycleEventType::BorrowRelease => {
                            LifecycleEventType::BorrowRelease
                        }
                        crate::core::types::LifecycleEventType::Modification => {
                            LifecycleEventType::Modification
                        }
                        crate::core::types::LifecycleEventType::LastUse => {
                            LifecycleEventType::LastUse
                        }
                        crate::core::types::LifecycleEventType::Drop => LifecycleEventType::Drop,
                        crate::core::types::LifecycleEventType::Destruction => {
                            LifecycleEventType::Destruction
                        }
                        crate::core::types::LifecycleEventType::MemoryReclaim => {
                            LifecycleEventType::MemoryReclaim
                        }
                    },
                    timestamp: e.timestamp,
                    location: SourceLocation {
                        file: e.location.file,
                        line: e.location.line,
                        column: e.location.column,
                    },
                    memory_state: MemoryState {
                        memory_location: match e.memory_state.memory_location {
                            crate::core::types::MemoryLocationType::Stack => {
                                MemoryLocationType::Stack
                            }
                            crate::core::types::MemoryLocationType::Heap => {
                                MemoryLocationType::Heap
                            }
                            crate::core::types::MemoryLocationType::Register => {
                                MemoryLocationType::Register
                            }
                            crate::core::types::MemoryLocationType::Static => {
                                MemoryLocationType::Static
                            }
                            crate::core::types::MemoryLocationType::ThreadLocal => {
                                MemoryLocationType::ThreadLocal
                            }
                        },
                        memory_address: e.memory_state.memory_address,
                        object_size: e.memory_state.object_size,
                        reference_count: e.memory_state.reference_count,
                        borrow_state: match e.memory_state.borrow_state {
                            crate::core::types::BorrowState::NotBorrowed => {
                                BorrowState::NotBorrowed
                            }
                            crate::core::types::BorrowState::SharedBorrow { count } => {
                                BorrowState::SharedBorrow { count }
                            }
                            crate::core::types::BorrowState::MutableBorrow => {
                                BorrowState::MutableBorrow
                            }
                            crate::core::types::BorrowState::MovedOut => BorrowState::MovedOut,
                        },
                    },
                    performance_metrics: EventPerformanceMetrics {
                        cpu_cycles: e.performance_metrics.cpu_cycles,
                        memory_bandwidth_bytes: e.performance_metrics.memory_bandwidth_bytes,
                        cache_misses: e.performance_metrics.cache_misses,
                        processing_time_ns: e.performance_metrics.processing_time_ns,
                    },
                    call_stack: e.call_stack,
                })
                .collect(),
            total_lifetime_ns: old.total_lifetime_ns,
            stage_durations: LifecycleStageDurations {
                creation_to_first_use_ns: old.stage_durations.creation_to_first_use_ns,
                active_use_duration_ns: old.stage_durations.active_use_duration_ns,
                last_use_to_destruction_ns: old.stage_durations.last_use_to_destruction_ns,
                borrowed_duration_ns: old.stage_durations.borrowed_duration_ns,
                idle_duration_ns: old.stage_durations.idle_duration_ns,
            },
            efficiency_metrics: LifecycleEfficiencyMetrics {
                utilization_ratio: old.efficiency_metrics.utilization_ratio,
                memory_efficiency: old.efficiency_metrics.memory_efficiency,
                performance_efficiency: old.efficiency_metrics.performance_efficiency,
                resource_waste: ResourceWasteAssessment {
                    wasted_memory_percent: old
                        .efficiency_metrics
                        .resource_waste
                        .wasted_memory_percent,
                    wasted_cpu_percent: old.efficiency_metrics.resource_waste.wasted_cpu_percent,
                    premature_destructions: old
                        .efficiency_metrics
                        .resource_waste
                        .premature_destructions,
                    unused_instances: old.efficiency_metrics.resource_waste.unused_instances,
                    optimization_opportunities: old
                        .efficiency_metrics
                        .resource_waste
                        .optimization_opportunities,
                },
            },
            lifecycle_patterns: old
                .lifecycle_patterns
                .into_iter()
                .map(|p| LifecyclePattern {
                    pattern_type: match p.pattern_type {
                        crate::core::types::LifecyclePatternType::ShortLived => {
                            LifecyclePatternType::ShortLived
                        }
                        crate::core::types::LifecyclePatternType::LongLived => {
                            LifecyclePatternType::LongLived
                        }
                        crate::core::types::LifecyclePatternType::Cyclical => {
                            LifecyclePatternType::Cyclical
                        }
                        crate::core::types::LifecyclePatternType::OnDemand => {
                            LifecyclePatternType::OnDemand
                        }
                        crate::core::types::LifecyclePatternType::Cached => {
                            LifecyclePatternType::Cached
                        }
                        crate::core::types::LifecyclePatternType::Pooled => {
                            LifecyclePatternType::Pooled
                        }
                        crate::core::types::LifecyclePatternType::Singleton => {
                            LifecyclePatternType::Singleton
                        }
                        crate::core::types::LifecyclePatternType::Factory => {
                            LifecyclePatternType::Factory
                        }
                        crate::core::types::LifecyclePatternType::RAII => {
                            LifecyclePatternType::RAII
                        }
                    },
                    frequency: p.frequency,
                    efficiency_score: p.efficiency_score,
                    performance_impact: p.performance_impact,
                    optimization_suggestions: p.optimization_suggestions,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify ObjectLifecycleInfo creation with all fields
    /// Invariants: All fields should be properly initialized
    #[test]
    fn test_object_lifecycle_info() {
        let info = ObjectLifecycleInfo {
            object_id: 1,
            type_name: "String".to_string(),
            lifecycle_events: vec![],
            total_lifetime_ns: Some(1000000),
            stage_durations: LifecycleStageDurations {
                creation_to_first_use_ns: Some(100),
                active_use_duration_ns: Some(900000),
                last_use_to_destruction_ns: Some(99900),
                borrowed_duration_ns: 100000,
                idle_duration_ns: 50000,
            },
            efficiency_metrics: LifecycleEfficiencyMetrics::default(),
            lifecycle_patterns: vec![],
        };

        assert_eq!(info.object_id, 1, "Object ID should match");
        assert_eq!(info.type_name, "String", "Type name should match");
        assert_eq!(
            info.total_lifetime_ns,
            Some(1000000),
            "Total lifetime should match"
        );
    }

    /// Objective: Verify LifecycleEventType variants
    /// Invariants: All variants should have debug representation
    #[test]
    fn test_lifecycle_event_type() {
        let events = vec![
            LifecycleEventType::Creation,
            LifecycleEventType::Initialization,
            LifecycleEventType::FirstUse,
            LifecycleEventType::Move,
            LifecycleEventType::Copy,
            LifecycleEventType::Clone,
            LifecycleEventType::Borrow,
            LifecycleEventType::MutableBorrow,
            LifecycleEventType::BorrowRelease,
            LifecycleEventType::Modification,
            LifecycleEventType::LastUse,
            LifecycleEventType::Drop,
            LifecycleEventType::Destruction,
            LifecycleEventType::MemoryReclaim,
        ];

        for event in events {
            let debug_str = format!("{event:?}");
            assert!(
                !debug_str.is_empty(),
                "LifecycleEventType should have debug representation"
            );
        }
    }

    /// Objective: Verify LifecycleEvent creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_lifecycle_event() {
        let event = LifecycleEvent {
            event_type: LifecycleEventType::Creation,
            timestamp: 1000,
            location: SourceLocation {
                file: "test.rs".to_string(),
                line: 10,
                column: 5,
            },
            memory_state: MemoryState {
                memory_location: MemoryLocationType::Heap,
                memory_address: 0x1000,
                object_size: 64,
                reference_count: Some(1),
                borrow_state: BorrowState::NotBorrowed,
            },
            performance_metrics: EventPerformanceMetrics {
                cpu_cycles: 1000,
                memory_bandwidth_bytes: 64,
                cache_misses: 2,
                processing_time_ns: 500,
            },
            call_stack: vec!["main".to_string(), "test".to_string()],
        };

        assert_eq!(
            event.event_type,
            LifecycleEventType::Creation,
            "Event type should be Creation"
        );
        assert_eq!(event.timestamp, 1000, "Timestamp should match");
        assert_eq!(event.call_stack.len(), 2, "Call stack should have 2 frames");
    }

    /// Objective: Verify MemoryState creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_memory_state() {
        let state = MemoryState {
            memory_location: MemoryLocationType::Stack,
            memory_address: 0x7fff0000,
            object_size: 32,
            reference_count: None,
            borrow_state: BorrowState::SharedBorrow { count: 3 },
        };

        assert_eq!(
            state.memory_location,
            MemoryLocationType::Stack,
            "Memory location should be Stack"
        );
        assert_eq!(
            state.borrow_state,
            BorrowState::SharedBorrow { count: 3 },
            "Borrow state should be SharedBorrow with count 3"
        );
    }

    /// Objective: Verify BorrowState variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_borrow_state_variants() {
        let states = vec![
            BorrowState::NotBorrowed,
            BorrowState::SharedBorrow { count: 1 },
            BorrowState::MutableBorrow,
            BorrowState::MovedOut,
        ];

        for state in &states {
            let debug_str = format!("{state:?}");
            assert!(
                !debug_str.is_empty(),
                "BorrowState should have debug representation"
            );
        }

        assert_eq!(BorrowState::NotBorrowed, BorrowState::NotBorrowed);
        assert_ne!(BorrowState::NotBorrowed, BorrowState::MutableBorrow);
    }

    /// Objective: Verify MemoryLocationType variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_memory_location_type_variants() {
        let locations = vec![
            MemoryLocationType::Stack,
            MemoryLocationType::Heap,
            MemoryLocationType::Register,
            MemoryLocationType::Static,
            MemoryLocationType::ThreadLocal,
        ];

        for location in &locations {
            let debug_str = format!("{location:?}");
            assert!(
                !debug_str.is_empty(),
                "MemoryLocationType should have debug representation"
            );
        }
    }

    /// Objective: Verify EventPerformanceMetrics creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_event_performance_metrics() {
        let metrics = EventPerformanceMetrics {
            cpu_cycles: 5000,
            memory_bandwidth_bytes: 1024,
            cache_misses: 10,
            processing_time_ns: 2000,
        };

        assert_eq!(metrics.cpu_cycles, 5000, "CPU cycles should match");
        assert_eq!(
            metrics.memory_bandwidth_bytes, 1024,
            "Memory bandwidth should match"
        );
        assert_eq!(metrics.cache_misses, 10, "Cache misses should match");
    }

    /// Objective: Verify LifecycleStageDurations creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_lifecycle_stage_durations() {
        let durations = LifecycleStageDurations {
            creation_to_first_use_ns: Some(100),
            active_use_duration_ns: Some(1000),
            last_use_to_destruction_ns: Some(50),
            borrowed_duration_ns: 200,
            idle_duration_ns: 100,
        };

        assert_eq!(
            durations.creation_to_first_use_ns,
            Some(100),
            "Creation to first use should match"
        );
        assert_eq!(
            durations.active_use_duration_ns,
            Some(1000),
            "Active use duration should match"
        );
    }

    /// Objective: Verify LifecycleEfficiencyMetrics default
    /// Invariants: Default should have zero values
    #[test]
    fn test_lifecycle_efficiency_metrics_default() {
        let metrics = LifecycleEfficiencyMetrics::default();

        assert_eq!(
            metrics.utilization_ratio, 0.0,
            "Utilization ratio should be 0"
        );
        assert_eq!(
            metrics.memory_efficiency, 0.0,
            "Memory efficiency should be 0"
        );
        assert_eq!(
            metrics.performance_efficiency, 0.0,
            "Performance efficiency should be 0"
        );
        assert_eq!(
            metrics.resource_waste.wasted_memory_percent, 0.0,
            "Wasted memory should be 0"
        );
    }

    /// Objective: Verify LifecycleEfficiencyMetrics with values
    /// Invariants: Should handle populated metrics
    #[test]
    fn test_lifecycle_efficiency_metrics_with_values() {
        let metrics = LifecycleEfficiencyMetrics {
            utilization_ratio: 0.85,
            memory_efficiency: 0.9,
            performance_efficiency: 0.95,
            resource_waste: ResourceWasteAssessment {
                wasted_memory_percent: 5.0,
                wasted_cpu_percent: 3.0,
                premature_destructions: 2,
                unused_instances: 5,
                optimization_opportunities: vec!["Use pool".to_string()],
            },
        };

        assert_eq!(
            metrics.utilization_ratio, 0.85,
            "Utilization ratio should match"
        );
        assert_eq!(
            metrics.resource_waste.premature_destructions, 2,
            "Premature destructions should match"
        );
        assert_eq!(
            metrics.resource_waste.optimization_opportunities.len(),
            1,
            "Should have one optimization opportunity"
        );
    }

    /// Objective: Verify ResourceWasteAssessment creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_resource_waste_assessment() {
        let assessment = ResourceWasteAssessment {
            wasted_memory_percent: 10.0,
            wasted_cpu_percent: 5.0,
            premature_destructions: 3,
            unused_instances: 10,
            optimization_opportunities: vec!["Optimize A".to_string(), "Optimize B".to_string()],
        };

        assert_eq!(
            assessment.wasted_memory_percent, 10.0,
            "Wasted memory percent should match"
        );
        assert_eq!(
            assessment.unused_instances, 10,
            "Unused instances should match"
        );
    }

    /// Objective: Verify LifecyclePattern creation
    /// Invariants: All fields should be accessible
    #[test]
    fn test_lifecycle_pattern() {
        let pattern = LifecyclePattern {
            pattern_type: LifecyclePatternType::ShortLived,
            frequency: 100,
            efficiency_score: 0.8,
            performance_impact: 0.2,
            optimization_suggestions: vec!["Consider pooling".to_string()],
        };

        assert_eq!(
            pattern.pattern_type,
            LifecyclePatternType::ShortLived,
            "Pattern type should match"
        );
        assert_eq!(pattern.frequency, 100, "Frequency should match");
    }

    /// Objective: Verify LifecyclePatternType variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_lifecycle_pattern_type_variants() {
        let patterns = vec![
            LifecyclePatternType::ShortLived,
            LifecyclePatternType::LongLived,
            LifecyclePatternType::Cyclical,
            LifecyclePatternType::OnDemand,
            LifecyclePatternType::Cached,
            LifecyclePatternType::Pooled,
            LifecyclePatternType::Singleton,
            LifecyclePatternType::Factory,
            LifecyclePatternType::RAII,
        ];

        for pattern in &patterns {
            let debug_str = format!("{pattern:?}");
            assert!(
                !debug_str.is_empty(),
                "LifecyclePatternType should have debug representation"
            );
        }
    }

    /// Objective: Verify SimpleLifecyclePattern variants
    /// Invariants: All variants should be distinct
    #[test]
    fn test_simple_lifecycle_pattern_variants() {
        let patterns = vec![
            SimpleLifecyclePattern::Instant,
            SimpleLifecyclePattern::ShortLived,
            SimpleLifecyclePattern::MediumLived,
            SimpleLifecyclePattern::LongLived,
            SimpleLifecyclePattern::Persistent,
        ];

        for pattern in &patterns {
            let debug_str = format!("{pattern:?}");
            assert!(
                !debug_str.is_empty(),
                "SimpleLifecyclePattern should have debug representation"
            );
        }
    }

    /// Objective: Verify serialization of LifecycleEventType
    /// Invariants: Should serialize and deserialize correctly
    #[test]
    fn test_lifecycle_event_type_serialization() {
        let event_type = LifecycleEventType::Clone;
        let json = serde_json::to_string(&event_type);
        assert!(json.is_ok(), "Should serialize to JSON");

        let deserialized: Result<LifecycleEventType, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "Should deserialize from JSON");
        assert_eq!(
            deserialized.unwrap(),
            LifecycleEventType::Clone,
            "Should preserve value"
        );
    }

    /// Objective: Verify serialization of BorrowState
    /// Invariants: Should serialize and deserialize correctly
    #[test]
    fn test_borrow_state_serialization() {
        let state = BorrowState::SharedBorrow { count: 5 };
        let json = serde_json::to_string(&state);
        assert!(json.is_ok(), "Should serialize to JSON");

        let deserialized: Result<BorrowState, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "Should deserialize from JSON");
        assert_eq!(
            deserialized.unwrap(),
            BorrowState::SharedBorrow { count: 5 },
            "Should preserve value"
        );
    }

    /// Objective: Verify serialization of MemoryLocationType
    /// Invariants: Should serialize and deserialize correctly
    #[test]
    fn test_memory_location_type_serialization() {
        let location = MemoryLocationType::Heap;
        let json = serde_json::to_string(&location);
        assert!(json.is_ok(), "Should serialize to JSON");

        let deserialized: Result<MemoryLocationType, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok(), "Should deserialize from JSON");
        assert_eq!(
            deserialized.unwrap(),
            MemoryLocationType::Heap,
            "Should preserve value"
        );
    }

    /// Objective: Verify ObjectLifecycleInfo with events
    /// Invariants: Should handle multiple lifecycle events
    #[test]
    fn test_object_lifecycle_info_with_events() {
        let events = vec![
            LifecycleEvent {
                event_type: LifecycleEventType::Creation,
                timestamp: 0,
                location: SourceLocation {
                    file: "test.rs".to_string(),
                    line: 1,
                    column: 1,
                },
                memory_state: MemoryState {
                    memory_location: MemoryLocationType::Heap,
                    memory_address: 0x1000,
                    object_size: 64,
                    reference_count: None,
                    borrow_state: BorrowState::NotBorrowed,
                },
                performance_metrics: EventPerformanceMetrics {
                    cpu_cycles: 100,
                    memory_bandwidth_bytes: 64,
                    cache_misses: 0,
                    processing_time_ns: 50,
                },
                call_stack: vec![],
            },
            LifecycleEvent {
                event_type: LifecycleEventType::Drop,
                timestamp: 1000000,
                location: SourceLocation {
                    file: "test.rs".to_string(),
                    line: 10,
                    column: 1,
                },
                memory_state: MemoryState {
                    memory_location: MemoryLocationType::Heap,
                    memory_address: 0x1000,
                    object_size: 64,
                    reference_count: None,
                    borrow_state: BorrowState::NotBorrowed,
                },
                performance_metrics: EventPerformanceMetrics {
                    cpu_cycles: 50,
                    memory_bandwidth_bytes: 0,
                    cache_misses: 0,
                    processing_time_ns: 25,
                },
                call_stack: vec![],
            },
        ];

        let info = ObjectLifecycleInfo {
            object_id: 1,
            type_name: "String".to_string(),
            lifecycle_events: events,
            total_lifetime_ns: Some(1000000),
            stage_durations: LifecycleStageDurations {
                creation_to_first_use_ns: None,
                active_use_duration_ns: None,
                last_use_to_destruction_ns: None,
                borrowed_duration_ns: 0,
                idle_duration_ns: 0,
            },
            efficiency_metrics: LifecycleEfficiencyMetrics::default(),
            lifecycle_patterns: vec![],
        };

        assert_eq!(info.lifecycle_events.len(), 2, "Should have 2 events");
        assert_eq!(
            info.lifecycle_events[0].event_type,
            LifecycleEventType::Creation,
            "First event should be Creation"
        );
        assert_eq!(
            info.lifecycle_events[1].event_type,
            LifecycleEventType::Drop,
            "Second event should be Drop"
        );
    }

    /// Objective: Verify Clone implementation for ObjectLifecycleInfo
    /// Invariants: Cloned object should have same values
    #[test]
    fn test_object_lifecycle_info_clone() {
        let original = ObjectLifecycleInfo {
            object_id: 42,
            type_name: "Vec<u8>".to_string(),
            lifecycle_events: vec![],
            total_lifetime_ns: Some(5000),
            stage_durations: LifecycleStageDurations {
                creation_to_first_use_ns: Some(100),
                active_use_duration_ns: Some(4000),
                last_use_to_destruction_ns: Some(900),
                borrowed_duration_ns: 500,
                idle_duration_ns: 200,
            },
            efficiency_metrics: LifecycleEfficiencyMetrics::default(),
            lifecycle_patterns: vec![],
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
}
