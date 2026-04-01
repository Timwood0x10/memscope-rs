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

#[cfg(test)]
mod tests {
    use super::*;

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

        assert_eq!(info.object_id, 1);
        assert_eq!(info.type_name, "String");
    }

    #[test]
    fn test_lifecycle_event_type() {
        let events = vec![
            LifecycleEventType::Creation,
            LifecycleEventType::FirstUse,
            LifecycleEventType::Drop,
        ];

        for event in events {
            assert!(!format!("{event:?}").is_empty());
        }
    }
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
