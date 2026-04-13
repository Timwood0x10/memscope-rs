//! Memory access tracking types.
//!
//! This module contains types for tracking memory access patterns,
//! including access events, cache performance, and bandwidth utilization.

use serde::{Deserialize, Serialize};

use super::allocation::{ImpactLevel, Priority};

/// Memory access pattern tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryAccessTrackingInfo {
    /// Memory region identifier.
    pub region_id: usize,
    /// Memory address range.
    pub address_range: AddressRange,
    /// Access events.
    pub access_events: Vec<MemoryAccessEvent>,
    /// Access statistics.
    pub access_statistics: MemoryAccessStatistics,
    /// Access patterns.
    pub access_patterns: Vec<AccessPattern>,
    /// Performance impact.
    pub performance_impact: MemoryAccessPerformanceImpact,
}

/// Memory address range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddressRange {
    /// Start address.
    pub start_address: usize,
    /// End address.
    pub end_address: usize,
    /// Size in bytes.
    pub size: usize,
}

/// Memory access event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryAccessEvent {
    /// Access type.
    pub access_type: MemoryAccessType,
    /// Timestamp.
    pub timestamp: u64,
    /// Memory address.
    pub address: usize,
    /// Access size.
    pub size: usize,
    /// Function that performed the access.
    pub function_name: String,
    /// Access latency.
    pub latency_ns: u64,
    /// Cache hit/miss information.
    pub cache_info: CacheAccessInfo,
}

/// Types of memory access.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryAccessType {
    /// Read access.
    Read,
    /// Write access.
    Write,
    /// Read-modify-write access.
    ReadModifyWrite,
    /// Prefetch access.
    Prefetch,
    /// Flush access.
    Flush,
}

/// Cache access information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheAccessInfo {
    /// L1 cache hit.
    pub l1_hit: bool,
    /// L2 cache hit.
    pub l2_hit: bool,
    /// L3 cache hit.
    pub l3_hit: bool,
    /// Memory access required.
    pub memory_access: bool,
    /// Access latency breakdown.
    pub latency_breakdown: CacheLatencyBreakdown,
}

/// Cache latency breakdown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheLatencyBreakdown {
    /// L1 cache latency.
    pub l1_latency_ns: f64,
    /// L2 cache latency.
    pub l2_latency_ns: f64,
    /// L3 cache latency.
    pub l3_latency_ns: f64,
    /// Main memory latency.
    pub memory_latency_ns: f64,
}

/// Memory access statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryAccessStatistics {
    /// Total read operations.
    pub total_reads: u64,
    /// Total write operations.
    pub total_writes: u64,
    /// Read/write ratio.
    pub read_write_ratio: f64,
    /// Average access frequency per second.
    pub avg_access_frequency: f64,
    /// Peak access frequency.
    pub peak_access_frequency: f64,
    /// Access locality metrics.
    pub locality_metrics: LocalityMetrics,
    /// Bandwidth utilization.
    pub bandwidth_utilization: BandwidthUtilization,
}

/// Memory locality metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalityMetrics {
    /// Temporal locality score (0-1).
    pub temporal_locality: f64,
    /// Spatial locality score (0-1).
    pub spatial_locality: f64,
    /// Sequential access percentage.
    pub sequential_access_percent: f64,
    /// Random access percentage.
    pub random_access_percent: f64,
    /// Stride pattern detection.
    pub stride_patterns: Vec<StridePattern>,
}

/// Stride pattern information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StridePattern {
    /// Stride size in bytes.
    pub stride_size: usize,
    /// Pattern frequency.
    pub frequency: u32,
    /// Pattern efficiency.
    pub efficiency_score: f64,
    /// Cache friendliness.
    pub cache_friendliness: f64,
}

/// Bandwidth utilization information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BandwidthUtilization {
    /// Peak bandwidth usage (bytes/sec).
    pub peak_bandwidth: f64,
    /// Average bandwidth usage.
    pub avg_bandwidth: f64,
    /// Bandwidth efficiency percentage.
    pub efficiency_percent: f64,
    /// Bottleneck identification.
    pub bottlenecks: Vec<BandwidthBottleneck>,
}

/// Bandwidth bottleneck information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BandwidthBottleneck {
    /// Bottleneck location.
    pub location: BandwidthBottleneckLocation,
    /// Impact severity.
    pub severity: ImpactLevel,
    /// Description.
    pub description: String,
    /// Mitigation suggestions.
    pub mitigation_suggestions: Vec<String>,
}

/// Bandwidth bottleneck locations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BandwidthBottleneckLocation {
    /// L1 cache bottleneck.
    L1Cache,
    /// L2 cache bottleneck.
    L2Cache,
    /// L3 cache bottleneck.
    L3Cache,
    /// Main memory bottleneck.
    MainMemory,
    /// System bus bottleneck.
    SystemBus,
    /// PCIe bottleneck.
    PCIe,
    /// Network bottleneck.
    Network,
}

/// Access pattern information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessPattern {
    /// Pattern type.
    pub pattern_type: AccessPatternType,
    /// Pattern description.
    pub description: String,
    /// Frequency of this pattern.
    pub frequency: u32,
    /// Performance characteristics.
    pub performance_characteristics: AccessPatternPerformance,
    /// Optimization potential.
    pub optimization_potential: f64,
}

/// Types of access patterns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccessPatternType {
    /// Sequential access pattern.
    Sequential,
    /// Random access pattern.
    Random,
    /// Strided access pattern.
    Strided,
    /// Hotspot access pattern.
    Hotspot,
    /// Sparse access pattern.
    Sparse,
    /// Dense access pattern.
    Dense,
    /// Temporal access pattern.
    Temporal,
    /// Spatial access pattern.
    Spatial,
}

/// Access pattern performance characteristics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessPatternPerformance {
    /// Cache hit rate for this pattern.
    pub cache_hit_rate: f64,
    /// Average latency for this pattern.
    pub avg_latency_ns: f64,
    /// Bandwidth efficiency.
    pub bandwidth_efficiency: f64,
    /// Prefetcher effectiveness.
    pub prefetcher_effectiveness: f64,
}

/// Memory access performance impact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryAccessPerformanceImpact {
    /// Overall performance score.
    pub performance_score: f64,
    /// Cache efficiency impact.
    pub cache_efficiency_impact: f64,
    /// Memory bandwidth impact.
    pub bandwidth_impact: f64,
    /// CPU pipeline impact.
    pub pipeline_impact: f64,
    /// Optimization recommendations.
    pub optimization_recommendations: Vec<MemoryOptimizationRecommendation>,
}

/// Memory optimization recommendation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryOptimizationRecommendation {
    /// Recommendation type.
    pub recommendation_type: MemoryOptimizationType,
    /// Priority.
    pub priority: Priority,
    /// Expected improvement.
    pub expected_improvement: f64,
    /// Implementation effort.
    pub implementation_effort: ImplementationDifficulty,
    /// Description.
    pub description: String,
}

/// Types of memory optimizations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryOptimizationType {
    /// Data layout optimization.
    DataLayout,
    /// Access pattern optimization.
    AccessPattern,
    /// Prefetching optimization.
    Prefetching,
    /// Caching optimization.
    Caching,
    /// Memory pooling optimization.
    MemoryPooling,
    /// NUMA optimization.
    NUMA,
    /// Vectorization optimization.
    Vectorization,
    /// Compression optimization.
    Compression,
}

/// Implementation difficulty levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImplementationDifficulty {
    /// Easy implementation difficulty.
    Easy,
    /// Medium implementation difficulty.
    Medium,
    /// Hard implementation difficulty.
    Hard,
    /// Very hard implementation difficulty.
    VeryHard,
}

// Implement From trait for converting from core::types to capture::types
impl From<crate::core::types::MemoryAccessTrackingInfo> for MemoryAccessTrackingInfo {
    fn from(old: crate::core::types::MemoryAccessTrackingInfo) -> Self {
        Self {
            region_id: old.region_id,
            address_range: AddressRange {
                start_address: old.address_range.start_address,
                end_address: old.address_range.end_address,
                size: old.address_range.size,
            },
            access_events: old
                .access_events
                .into_iter()
                .map(|e| MemoryAccessEvent {
                    access_type: match e.access_type {
                        crate::core::types::MemoryAccessType::Read => MemoryAccessType::Read,
                        crate::core::types::MemoryAccessType::Write => MemoryAccessType::Write,
                        crate::core::types::MemoryAccessType::ReadModifyWrite => {
                            MemoryAccessType::ReadModifyWrite
                        }
                        crate::core::types::MemoryAccessType::Prefetch => {
                            MemoryAccessType::Prefetch
                        }
                        crate::core::types::MemoryAccessType::Flush => MemoryAccessType::Flush,
                    },
                    timestamp: e.timestamp,
                    address: e.address,
                    size: e.size,
                    function_name: e.function_name,
                    latency_ns: e.latency_ns,
                    cache_info: CacheAccessInfo {
                        l1_hit: e.cache_info.l1_hit,
                        l2_hit: e.cache_info.l2_hit,
                        l3_hit: e.cache_info.l3_hit,
                        memory_access: e.cache_info.memory_access,
                        latency_breakdown: CacheLatencyBreakdown {
                            l1_latency_ns: e.cache_info.latency_breakdown.l1_latency_ns,
                            l2_latency_ns: e.cache_info.latency_breakdown.l2_latency_ns,
                            l3_latency_ns: e.cache_info.latency_breakdown.l3_latency_ns,
                            memory_latency_ns: e.cache_info.latency_breakdown.memory_latency_ns,
                        },
                    },
                })
                .collect(),
            access_statistics: MemoryAccessStatistics {
                total_reads: old.access_statistics.total_reads,
                total_writes: old.access_statistics.total_writes,
                read_write_ratio: old.access_statistics.read_write_ratio,
                avg_access_frequency: old.access_statistics.avg_access_frequency,
                peak_access_frequency: old.access_statistics.peak_access_frequency,
                locality_metrics: LocalityMetrics {
                    temporal_locality: old.access_statistics.locality_metrics.temporal_locality,
                    spatial_locality: old.access_statistics.locality_metrics.spatial_locality,
                    sequential_access_percent: old
                        .access_statistics
                        .locality_metrics
                        .sequential_access_percent,
                    random_access_percent: old
                        .access_statistics
                        .locality_metrics
                        .random_access_percent,
                    stride_patterns: old
                        .access_statistics
                        .locality_metrics
                        .stride_patterns
                        .into_iter()
                        .map(|s| StridePattern {
                            stride_size: s.stride_size,
                            frequency: s.frequency,
                            efficiency_score: s.efficiency_score,
                            cache_friendliness: s.cache_friendliness,
                        })
                        .collect(),
                },
                bandwidth_utilization: BandwidthUtilization {
                    peak_bandwidth: old.access_statistics.bandwidth_utilization.peak_bandwidth,
                    avg_bandwidth: old.access_statistics.bandwidth_utilization.avg_bandwidth,
                    efficiency_percent: old
                        .access_statistics
                        .bandwidth_utilization
                        .efficiency_percent,
                    bottlenecks: old
                        .access_statistics
                        .bandwidth_utilization
                        .bottlenecks
                        .into_iter()
                        .map(|b| BandwidthBottleneck {
                            location: match b.location {
                                crate::core::types::BandwidthBottleneckLocation::L1Cache => {
                                    BandwidthBottleneckLocation::L1Cache
                                }
                                crate::core::types::BandwidthBottleneckLocation::L2Cache => {
                                    BandwidthBottleneckLocation::L2Cache
                                }
                                crate::core::types::BandwidthBottleneckLocation::L3Cache => {
                                    BandwidthBottleneckLocation::L3Cache
                                }
                                crate::core::types::BandwidthBottleneckLocation::MainMemory => {
                                    BandwidthBottleneckLocation::MainMemory
                                }
                                crate::core::types::BandwidthBottleneckLocation::SystemBus => {
                                    BandwidthBottleneckLocation::SystemBus
                                }
                                crate::core::types::BandwidthBottleneckLocation::PCIe => {
                                    BandwidthBottleneckLocation::PCIe
                                }
                                crate::core::types::BandwidthBottleneckLocation::Network => {
                                    BandwidthBottleneckLocation::Network
                                }
                            },
                            severity: match b.severity {
                                crate::core::types::ImpactLevel::Low => ImpactLevel::Low,
                                crate::core::types::ImpactLevel::Medium => ImpactLevel::Medium,
                                crate::core::types::ImpactLevel::High => ImpactLevel::High,
                                crate::core::types::ImpactLevel::Critical => ImpactLevel::Critical,
                            },
                            description: b.description,
                            mitigation_suggestions: b.mitigation_suggestions,
                        })
                        .collect(),
                },
            },
            access_patterns: old
                .access_patterns
                .into_iter()
                .map(|p| AccessPattern {
                    pattern_type: match p.pattern_type {
                        crate::core::types::AccessPatternType::Sequential => {
                            AccessPatternType::Sequential
                        }
                        crate::core::types::AccessPatternType::Random => AccessPatternType::Random,
                        crate::core::types::AccessPatternType::Strided => {
                            AccessPatternType::Strided
                        }
                        crate::core::types::AccessPatternType::Hotspot => {
                            AccessPatternType::Hotspot
                        }
                        crate::core::types::AccessPatternType::Sparse => AccessPatternType::Sparse,
                        crate::core::types::AccessPatternType::Dense => AccessPatternType::Dense,
                        crate::core::types::AccessPatternType::Temporal => {
                            AccessPatternType::Temporal
                        }
                        crate::core::types::AccessPatternType::Spatial => {
                            AccessPatternType::Spatial
                        }
                    },
                    description: p.description,
                    frequency: p.frequency,
                    performance_characteristics: AccessPatternPerformance {
                        cache_hit_rate: p.performance_characteristics.cache_hit_rate,
                        avg_latency_ns: p.performance_characteristics.avg_latency_ns,
                        bandwidth_efficiency: p.performance_characteristics.bandwidth_efficiency,
                        prefetcher_effectiveness: p
                            .performance_characteristics
                            .prefetcher_effectiveness,
                    },
                    optimization_potential: p.optimization_potential,
                })
                .collect(),
            performance_impact: MemoryAccessPerformanceImpact {
                performance_score: old.performance_impact.performance_score,
                cache_efficiency_impact: old.performance_impact.cache_efficiency_impact,
                bandwidth_impact: old.performance_impact.bandwidth_impact,
                pipeline_impact: old.performance_impact.pipeline_impact,
                optimization_recommendations: old
                    .performance_impact
                    .optimization_recommendations
                    .into_iter()
                    .map(|r| MemoryOptimizationRecommendation {
                        recommendation_type: match r.recommendation_type {
                            crate::core::types::MemoryOptimizationType::DataLayout => {
                                MemoryOptimizationType::DataLayout
                            }
                            crate::core::types::MemoryOptimizationType::AccessPattern => {
                                MemoryOptimizationType::AccessPattern
                            }
                            crate::core::types::MemoryOptimizationType::Prefetching => {
                                MemoryOptimizationType::Prefetching
                            }
                            crate::core::types::MemoryOptimizationType::Caching => {
                                MemoryOptimizationType::Caching
                            }
                            crate::core::types::MemoryOptimizationType::MemoryPooling => {
                                MemoryOptimizationType::MemoryPooling
                            }
                            crate::core::types::MemoryOptimizationType::NUMA => {
                                MemoryOptimizationType::NUMA
                            }
                            crate::core::types::MemoryOptimizationType::Vectorization => {
                                MemoryOptimizationType::Vectorization
                            }
                            crate::core::types::MemoryOptimizationType::Compression => {
                                MemoryOptimizationType::Compression
                            }
                        },
                        priority: match r.priority {
                            crate::core::types::Priority::Low => Priority::Low,
                            crate::core::types::Priority::Medium => Priority::Medium,
                            crate::core::types::Priority::High => Priority::High,
                            crate::core::types::Priority::Critical => Priority::Critical,
                        },
                        expected_improvement: r.expected_improvement,
                        implementation_effort: match r.implementation_effort {
                            crate::core::types::ImplementationDifficulty::Easy => {
                                ImplementationDifficulty::Easy
                            }
                            crate::core::types::ImplementationDifficulty::Medium => {
                                ImplementationDifficulty::Medium
                            }
                            crate::core::types::ImplementationDifficulty::Hard => {
                                ImplementationDifficulty::Hard
                            }
                            crate::core::types::ImplementationDifficulty::VeryHard => {
                                ImplementationDifficulty::VeryHard
                            }
                        },
                        description: r.description,
                    })
                    .collect(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_access_tracking_info() {
        let info = MemoryAccessTrackingInfo {
            region_id: 1,
            address_range: AddressRange {
                start_address: 0x1000,
                end_address: 0x2000,
                size: 0x1000,
            },
            access_events: vec![],
            access_statistics: MemoryAccessStatistics {
                total_reads: 100,
                total_writes: 50,
                read_write_ratio: 2.0,
                avg_access_frequency: 1000.0,
                peak_access_frequency: 5000.0,
                locality_metrics: LocalityMetrics {
                    temporal_locality: 0.8,
                    spatial_locality: 0.9,
                    sequential_access_percent: 70.0,
                    random_access_percent: 30.0,
                    stride_patterns: vec![],
                },
                bandwidth_utilization: BandwidthUtilization {
                    peak_bandwidth: 1000000.0,
                    avg_bandwidth: 500000.0,
                    efficiency_percent: 50.0,
                    bottlenecks: vec![],
                },
            },
            access_patterns: vec![],
            performance_impact: MemoryAccessPerformanceImpact {
                performance_score: 0.8,
                cache_efficiency_impact: 0.1,
                bandwidth_impact: 0.1,
                pipeline_impact: 0.05,
                optimization_recommendations: vec![],
            },
        };

        assert_eq!(info.region_id, 1);
        assert_eq!(info.access_statistics.total_reads, 100);
    }

    #[test]
    fn test_access_pattern_type() {
        let patterns = vec![
            AccessPatternType::Sequential,
            AccessPatternType::Random,
            AccessPatternType::Strided,
        ];

        for pattern in patterns {
            assert!(!format!("{pattern:?}").is_empty());
        }
    }

    #[test]
    fn test_all_memory_access_type_variants() {
        let access_types = [
            MemoryAccessType::Read,
            MemoryAccessType::Write,
            MemoryAccessType::ReadModifyWrite,
            MemoryAccessType::Prefetch,
            MemoryAccessType::Flush,
        ];

        for access_type in access_types {
            let event = MemoryAccessEvent {
                access_type: access_type.clone(),
                timestamp: 1000,
                address: 0x1000,
                size: 64,
                function_name: "test".to_string(),
                latency_ns: 50,
                cache_info: CacheAccessInfo {
                    l1_hit: true,
                    l2_hit: false,
                    l3_hit: false,
                    memory_access: false,
                    latency_breakdown: CacheLatencyBreakdown {
                        l1_latency_ns: 4.0,
                        l2_latency_ns: 0.0,
                        l3_latency_ns: 0.0,
                        memory_latency_ns: 0.0,
                    },
                },
            };
            assert_eq!(event.access_type, access_type);
        }
    }

    #[test]
    fn test_all_bandwidth_bottleneck_location_variants() {
        let locations = [
            BandwidthBottleneckLocation::L1Cache,
            BandwidthBottleneckLocation::L2Cache,
            BandwidthBottleneckLocation::L3Cache,
            BandwidthBottleneckLocation::MainMemory,
            BandwidthBottleneckLocation::SystemBus,
            BandwidthBottleneckLocation::PCIe,
            BandwidthBottleneckLocation::Network,
        ];

        for location in locations {
            let bottleneck = BandwidthBottleneck {
                location: location.clone(),
                severity: ImpactLevel::Medium,
                description: "Test bottleneck".to_string(),
                mitigation_suggestions: vec!["Optimize access pattern".to_string()],
            };
            assert_eq!(bottleneck.location, location);
        }
    }

    #[test]
    fn test_all_access_pattern_type_variants() {
        let pattern_types = [
            AccessPatternType::Sequential,
            AccessPatternType::Random,
            AccessPatternType::Strided,
            AccessPatternType::Hotspot,
            AccessPatternType::Sparse,
            AccessPatternType::Dense,
            AccessPatternType::Temporal,
            AccessPatternType::Spatial,
        ];

        for pattern_type in pattern_types {
            let pattern = AccessPattern {
                pattern_type: pattern_type.clone(),
                description: "Test pattern".to_string(),
                frequency: 100,
                performance_characteristics: AccessPatternPerformance {
                    cache_hit_rate: 0.9,
                    avg_latency_ns: 50.0,
                    bandwidth_efficiency: 0.8,
                    prefetcher_effectiveness: 0.7,
                },
                optimization_potential: 0.5,
            };
            assert_eq!(pattern.pattern_type, pattern_type);
        }
    }

    #[test]
    fn test_all_memory_optimization_type_variants() {
        let optimization_types = [
            MemoryOptimizationType::DataLayout,
            MemoryOptimizationType::AccessPattern,
            MemoryOptimizationType::Prefetching,
            MemoryOptimizationType::Caching,
            MemoryOptimizationType::MemoryPooling,
            MemoryOptimizationType::NUMA,
            MemoryOptimizationType::Vectorization,
            MemoryOptimizationType::Compression,
        ];

        for opt_type in optimization_types {
            let recommendation = MemoryOptimizationRecommendation {
                recommendation_type: opt_type.clone(),
                priority: Priority::Medium,
                expected_improvement: 0.3,
                implementation_effort: ImplementationDifficulty::Medium,
                description: "Test recommendation".to_string(),
            };
            assert_eq!(recommendation.recommendation_type, opt_type);
        }
    }

    #[test]
    fn test_all_implementation_difficulty_variants() {
        let difficulties = [
            ImplementationDifficulty::Easy,
            ImplementationDifficulty::Medium,
            ImplementationDifficulty::Hard,
            ImplementationDifficulty::VeryHard,
        ];

        for difficulty in difficulties {
            let recommendation = MemoryOptimizationRecommendation {
                recommendation_type: MemoryOptimizationType::DataLayout,
                priority: Priority::Low,
                expected_improvement: 0.1,
                implementation_effort: difficulty.clone(),
                description: "Test".to_string(),
            };
            assert_eq!(recommendation.implementation_effort, difficulty);
        }
    }

    #[test]
    fn test_address_range_creation() {
        let range = AddressRange {
            start_address: 0x1000,
            end_address: 0x1FFF,
            size: 0x1000,
        };

        assert_eq!(range.start_address, 0x1000);
        assert_eq!(range.end_address, 0x1FFF);
        assert_eq!(range.size, 0x1000);
    }

    #[test]
    fn test_memory_access_event_creation() {
        let event = MemoryAccessEvent {
            access_type: MemoryAccessType::Read,
            timestamp: 1234567890,
            address: 0xDEADBEEF,
            size: 128,
            function_name: "process_data".to_string(),
            latency_ns: 75,
            cache_info: CacheAccessInfo {
                l1_hit: false,
                l2_hit: true,
                l3_hit: false,
                memory_access: false,
                latency_breakdown: CacheLatencyBreakdown {
                    l1_latency_ns: 0.0,
                    l2_latency_ns: 12.0,
                    l3_latency_ns: 0.0,
                    memory_latency_ns: 0.0,
                },
            },
        };

        assert!(matches!(event.access_type, MemoryAccessType::Read));
        assert_eq!(event.size, 128);
        assert!(!event.cache_info.l1_hit);
        assert!(event.cache_info.l2_hit);
    }

    #[test]
    fn test_cache_access_info_creation() {
        let info = CacheAccessInfo {
            l1_hit: true,
            l2_hit: false,
            l3_hit: false,
            memory_access: false,
            latency_breakdown: CacheLatencyBreakdown {
                l1_latency_ns: 4.0,
                l2_latency_ns: 0.0,
                l3_latency_ns: 0.0,
                memory_latency_ns: 0.0,
            },
        };

        assert!(info.l1_hit);
        assert!(!info.memory_access);
    }

    #[test]
    fn test_cache_latency_breakdown_creation() {
        let breakdown = CacheLatencyBreakdown {
            l1_latency_ns: 4.0,
            l2_latency_ns: 12.0,
            l3_latency_ns: 40.0,
            memory_latency_ns: 100.0,
        };

        assert!((breakdown.l1_latency_ns - 4.0).abs() < f64::EPSILON);
        assert!((breakdown.memory_latency_ns - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_memory_access_statistics_creation() {
        let stats = MemoryAccessStatistics {
            total_reads: 10000,
            total_writes: 5000,
            read_write_ratio: 2.0,
            avg_access_frequency: 5000.0,
            peak_access_frequency: 20000.0,
            locality_metrics: LocalityMetrics {
                temporal_locality: 0.85,
                spatial_locality: 0.75,
                sequential_access_percent: 60.0,
                random_access_percent: 40.0,
                stride_patterns: vec![],
            },
            bandwidth_utilization: BandwidthUtilization {
                peak_bandwidth: 5000000.0,
                avg_bandwidth: 2500000.0,
                efficiency_percent: 75.0,
                bottlenecks: vec![],
            },
        };

        assert_eq!(stats.total_reads, 10000);
        assert!((stats.read_write_ratio - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_locality_metrics_creation() {
        let metrics = LocalityMetrics {
            temporal_locality: 0.9,
            spatial_locality: 0.8,
            sequential_access_percent: 80.0,
            random_access_percent: 20.0,
            stride_patterns: vec![StridePattern {
                stride_size: 64,
                frequency: 100,
                efficiency_score: 0.85,
                cache_friendliness: 0.9,
            }],
        };

        assert!((metrics.temporal_locality - 0.9).abs() < f64::EPSILON);
        assert_eq!(metrics.stride_patterns.len(), 1);
    }

    #[test]
    fn test_stride_pattern_creation() {
        let pattern = StridePattern {
            stride_size: 128,
            frequency: 50,
            efficiency_score: 0.75,
            cache_friendliness: 0.8,
        };

        assert_eq!(pattern.stride_size, 128);
        assert_eq!(pattern.frequency, 50);
    }

    #[test]
    fn test_bandwidth_utilization_creation() {
        let utilization = BandwidthUtilization {
            peak_bandwidth: 10000000.0,
            avg_bandwidth: 5000000.0,
            efficiency_percent: 85.0,
            bottlenecks: vec![BandwidthBottleneck {
                location: BandwidthBottleneckLocation::MainMemory,
                severity: ImpactLevel::High,
                description: "Memory bandwidth saturation".to_string(),
                mitigation_suggestions: vec!["Reduce memory access".to_string()],
            }],
        };

        assert_eq!(utilization.bottlenecks.len(), 1);
    }

    #[test]
    fn test_bandwidth_bottleneck_creation() {
        let bottleneck = BandwidthBottleneck {
            location: BandwidthBottleneckLocation::L3Cache,
            severity: ImpactLevel::Critical,
            description: "L3 cache thrashing".to_string(),
            mitigation_suggestions: vec![
                "Improve data locality".to_string(),
                "Reduce working set size".to_string(),
            ],
        };

        assert!(matches!(
            bottleneck.location,
            BandwidthBottleneckLocation::L3Cache
        ));
        assert_eq!(bottleneck.mitigation_suggestions.len(), 2);
    }

    #[test]
    fn test_access_pattern_creation() {
        let pattern = AccessPattern {
            pattern_type: AccessPatternType::Hotspot,
            description: "Hot memory region access".to_string(),
            frequency: 500,
            performance_characteristics: AccessPatternPerformance {
                cache_hit_rate: 0.95,
                avg_latency_ns: 30.0,
                bandwidth_efficiency: 0.9,
                prefetcher_effectiveness: 0.85,
            },
            optimization_potential: 0.6,
        };

        assert!(matches!(pattern.pattern_type, AccessPatternType::Hotspot));
        assert!((pattern.performance_characteristics.cache_hit_rate - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_access_pattern_performance_creation() {
        let perf = AccessPatternPerformance {
            cache_hit_rate: 0.88,
            avg_latency_ns: 45.0,
            bandwidth_efficiency: 0.75,
            prefetcher_effectiveness: 0.6,
        };

        assert!((perf.cache_hit_rate - 0.88).abs() < f64::EPSILON);
        assert!((perf.avg_latency_ns - 45.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_memory_access_performance_impact_creation() {
        let impact = MemoryAccessPerformanceImpact {
            performance_score: 0.72,
            cache_efficiency_impact: 0.15,
            bandwidth_impact: 0.08,
            pipeline_impact: 0.05,
            optimization_recommendations: vec![MemoryOptimizationRecommendation {
                recommendation_type: MemoryOptimizationType::Prefetching,
                priority: Priority::High,
                expected_improvement: 0.25,
                implementation_effort: ImplementationDifficulty::Easy,
                description: "Add software prefetching".to_string(),
            }],
        };

        assert_eq!(impact.optimization_recommendations.len(), 1);
        assert!((impact.performance_score - 0.72).abs() < f64::EPSILON);
    }

    #[test]
    fn test_memory_optimization_recommendation_creation() {
        let recommendation = MemoryOptimizationRecommendation {
            recommendation_type: MemoryOptimizationType::Vectorization,
            priority: Priority::Critical,
            expected_improvement: 0.4,
            implementation_effort: ImplementationDifficulty::Hard,
            description: "Vectorize inner loop".to_string(),
        };

        assert!(matches!(
            recommendation.recommendation_type,
            MemoryOptimizationType::Vectorization
        ));
        assert!(matches!(recommendation.priority, Priority::Critical));
    }

    #[test]
    fn test_memory_access_tracking_info_serialization() {
        let info = MemoryAccessTrackingInfo {
            region_id: 42,
            address_range: AddressRange {
                start_address: 0x8000,
                end_address: 0x9000,
                size: 0x1000,
            },
            access_events: vec![],
            access_statistics: MemoryAccessStatistics {
                total_reads: 500,
                total_writes: 250,
                read_write_ratio: 2.0,
                avg_access_frequency: 1000.0,
                peak_access_frequency: 3000.0,
                locality_metrics: LocalityMetrics {
                    temporal_locality: 0.7,
                    spatial_locality: 0.8,
                    sequential_access_percent: 65.0,
                    random_access_percent: 35.0,
                    stride_patterns: vec![],
                },
                bandwidth_utilization: BandwidthUtilization {
                    peak_bandwidth: 8000000.0,
                    avg_bandwidth: 4000000.0,
                    efficiency_percent: 70.0,
                    bottlenecks: vec![],
                },
            },
            access_patterns: vec![],
            performance_impact: MemoryAccessPerformanceImpact {
                performance_score: 0.75,
                cache_efficiency_impact: 0.12,
                bandwidth_impact: 0.08,
                pipeline_impact: 0.05,
                optimization_recommendations: vec![],
            },
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: MemoryAccessTrackingInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.region_id, info.region_id);
        assert_eq!(
            deserialized.address_range.start_address,
            info.address_range.start_address
        );
    }

    #[test]
    fn test_memory_access_type_serialization() {
        let access_types = vec![
            MemoryAccessType::Read,
            MemoryAccessType::Write,
            MemoryAccessType::ReadModifyWrite,
            MemoryAccessType::Prefetch,
            MemoryAccessType::Flush,
        ];

        for access_type in access_types {
            let json = serde_json::to_string(&access_type).unwrap();
            let deserialized: MemoryAccessType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, access_type);
        }
    }

    #[test]
    fn test_bandwidth_bottleneck_location_serialization() {
        let locations = vec![
            BandwidthBottleneckLocation::L1Cache,
            BandwidthBottleneckLocation::L2Cache,
            BandwidthBottleneckLocation::L3Cache,
            BandwidthBottleneckLocation::MainMemory,
            BandwidthBottleneckLocation::SystemBus,
            BandwidthBottleneckLocation::PCIe,
            BandwidthBottleneckLocation::Network,
        ];

        for location in locations {
            let json = serde_json::to_string(&location).unwrap();
            let deserialized: BandwidthBottleneckLocation = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, location);
        }
    }

    #[test]
    fn test_access_pattern_type_serialization() {
        let pattern_types = vec![
            AccessPatternType::Sequential,
            AccessPatternType::Random,
            AccessPatternType::Strided,
            AccessPatternType::Hotspot,
            AccessPatternType::Sparse,
            AccessPatternType::Dense,
            AccessPatternType::Temporal,
            AccessPatternType::Spatial,
        ];

        for pattern_type in pattern_types {
            let json = serde_json::to_string(&pattern_type).unwrap();
            let deserialized: AccessPatternType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, pattern_type);
        }
    }

    #[test]
    fn test_memory_optimization_type_serialization() {
        let optimization_types = vec![
            MemoryOptimizationType::DataLayout,
            MemoryOptimizationType::AccessPattern,
            MemoryOptimizationType::Prefetching,
            MemoryOptimizationType::Caching,
            MemoryOptimizationType::MemoryPooling,
            MemoryOptimizationType::NUMA,
            MemoryOptimizationType::Vectorization,
            MemoryOptimizationType::Compression,
        ];

        for opt_type in optimization_types {
            let json = serde_json::to_string(&opt_type).unwrap();
            let deserialized: MemoryOptimizationType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, opt_type);
        }
    }

    #[test]
    fn test_implementation_difficulty_serialization() {
        let difficulties = vec![
            ImplementationDifficulty::Easy,
            ImplementationDifficulty::Medium,
            ImplementationDifficulty::Hard,
            ImplementationDifficulty::VeryHard,
        ];

        for difficulty in difficulties {
            let json = serde_json::to_string(&difficulty).unwrap();
            let deserialized: ImplementationDifficulty = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, difficulty);
        }
    }

    #[test]
    fn test_memory_access_tracking_info_clone() {
        let info = MemoryAccessTrackingInfo {
            region_id: 1,
            address_range: AddressRange {
                start_address: 0,
                end_address: 1024,
                size: 1024,
            },
            access_events: vec![],
            access_statistics: MemoryAccessStatistics {
                total_reads: 0,
                total_writes: 0,
                read_write_ratio: 0.0,
                avg_access_frequency: 0.0,
                peak_access_frequency: 0.0,
                locality_metrics: LocalityMetrics {
                    temporal_locality: 0.0,
                    spatial_locality: 0.0,
                    sequential_access_percent: 0.0,
                    random_access_percent: 0.0,
                    stride_patterns: vec![],
                },
                bandwidth_utilization: BandwidthUtilization {
                    peak_bandwidth: 0.0,
                    avg_bandwidth: 0.0,
                    efficiency_percent: 0.0,
                    bottlenecks: vec![],
                },
            },
            access_patterns: vec![],
            performance_impact: MemoryAccessPerformanceImpact {
                performance_score: 0.0,
                cache_efficiency_impact: 0.0,
                bandwidth_impact: 0.0,
                pipeline_impact: 0.0,
                optimization_recommendations: vec![],
            },
        };

        let cloned = info.clone();
        assert_eq!(cloned.region_id, info.region_id);
    }

    #[test]
    fn test_memory_access_tracking_info_debug() {
        let info = MemoryAccessTrackingInfo {
            region_id: 99,
            address_range: AddressRange {
                start_address: 0x1000,
                end_address: 0x2000,
                size: 0x1000,
            },
            access_events: vec![],
            access_statistics: MemoryAccessStatistics {
                total_reads: 0,
                total_writes: 0,
                read_write_ratio: 0.0,
                avg_access_frequency: 0.0,
                peak_access_frequency: 0.0,
                locality_metrics: LocalityMetrics {
                    temporal_locality: 0.0,
                    spatial_locality: 0.0,
                    sequential_access_percent: 0.0,
                    random_access_percent: 0.0,
                    stride_patterns: vec![],
                },
                bandwidth_utilization: BandwidthUtilization {
                    peak_bandwidth: 0.0,
                    avg_bandwidth: 0.0,
                    efficiency_percent: 0.0,
                    bottlenecks: vec![],
                },
            },
            access_patterns: vec![],
            performance_impact: MemoryAccessPerformanceImpact {
                performance_score: 0.0,
                cache_efficiency_impact: 0.0,
                bandwidth_impact: 0.0,
                pipeline_impact: 0.0,
                optimization_recommendations: vec![],
            },
        };

        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("MemoryAccessTrackingInfo"));
        assert!(debug_str.contains("region_id"));
    }

    #[test]
    fn test_address_range_equality() {
        let range1 = AddressRange {
            start_address: 0x1000,
            end_address: 0x2000,
            size: 0x1000,
        };
        let range2 = AddressRange {
            start_address: 0x1000,
            end_address: 0x2000,
            size: 0x1000,
        };
        let range3 = AddressRange {
            start_address: 0x2000,
            end_address: 0x3000,
            size: 0x1000,
        };

        assert_eq!(range1, range2);
        assert_ne!(range1, range3);
    }

    #[test]
    fn test_memory_access_event_with_all_cache_states() {
        let cache_states = [
            (true, false, false, false),
            (false, true, false, false),
            (false, false, true, false),
            (false, false, false, true),
        ];

        for (l1, l2, l3, mem) in cache_states {
            let event = MemoryAccessEvent {
                access_type: MemoryAccessType::Read,
                timestamp: 0,
                address: 0,
                size: 64,
                function_name: "test".to_string(),
                latency_ns: 0,
                cache_info: CacheAccessInfo {
                    l1_hit: l1,
                    l2_hit: l2,
                    l3_hit: l3,
                    memory_access: mem,
                    latency_breakdown: CacheLatencyBreakdown {
                        l1_latency_ns: if l1 { 4.0 } else { 0.0 },
                        l2_latency_ns: if l2 { 12.0 } else { 0.0 },
                        l3_latency_ns: if l3 { 40.0 } else { 0.0 },
                        memory_latency_ns: if mem { 100.0 } else { 0.0 },
                    },
                },
            };
            assert_eq!(event.cache_info.l1_hit, l1);
            assert_eq!(event.cache_info.l2_hit, l2);
            assert_eq!(event.cache_info.l3_hit, l3);
            assert_eq!(event.cache_info.memory_access, mem);
        }
    }

    #[test]
    fn test_priority_variants_in_recommendation() {
        let priorities = [
            Priority::Low,
            Priority::Medium,
            Priority::High,
            Priority::Critical,
        ];

        for priority in priorities {
            let rec = MemoryOptimizationRecommendation {
                recommendation_type: MemoryOptimizationType::Caching,
                priority: priority.clone(),
                expected_improvement: 0.2,
                implementation_effort: ImplementationDifficulty::Medium,
                description: "Test".to_string(),
            };
            assert_eq!(rec.priority, priority);
        }
    }

    #[test]
    fn test_impact_level_variants_in_bottleneck() {
        let impact_levels = [
            ImpactLevel::Low,
            ImpactLevel::Medium,
            ImpactLevel::High,
            ImpactLevel::Critical,
        ];

        for impact in impact_levels {
            let bottleneck = BandwidthBottleneck {
                location: BandwidthBottleneckLocation::SystemBus,
                severity: impact.clone(),
                description: "Test".to_string(),
                mitigation_suggestions: vec![],
            };
            assert_eq!(bottleneck.severity, impact);
        }
    }

    #[test]
    fn test_boundary_values_memory_access() {
        let info = MemoryAccessTrackingInfo {
            region_id: usize::MAX,
            address_range: AddressRange {
                start_address: usize::MAX,
                end_address: usize::MAX,
                size: usize::MAX,
            },
            access_events: vec![MemoryAccessEvent {
                access_type: MemoryAccessType::Read,
                timestamp: u64::MAX,
                address: usize::MAX,
                size: usize::MAX,
                function_name: String::new(),
                latency_ns: u64::MAX,
                cache_info: CacheAccessInfo {
                    l1_hit: true,
                    l2_hit: true,
                    l3_hit: true,
                    memory_access: false,
                    latency_breakdown: CacheLatencyBreakdown {
                        l1_latency_ns: f64::MAX,
                        l2_latency_ns: f64::MAX,
                        l3_latency_ns: f64::MAX,
                        memory_latency_ns: f64::MIN,
                    },
                },
            }],
            access_statistics: MemoryAccessStatistics {
                total_reads: u64::MAX,
                total_writes: u64::MAX,
                read_write_ratio: f64::MAX,
                avg_access_frequency: f64::MAX,
                peak_access_frequency: f64::MAX,
                locality_metrics: LocalityMetrics {
                    temporal_locality: 1.0,
                    spatial_locality: 1.0,
                    sequential_access_percent: 100.0,
                    random_access_percent: 0.0,
                    stride_patterns: vec![],
                },
                bandwidth_utilization: BandwidthUtilization {
                    peak_bandwidth: f64::MAX,
                    avg_bandwidth: f64::MAX,
                    efficiency_percent: 100.0,
                    bottlenecks: vec![],
                },
            },
            access_patterns: vec![],
            performance_impact: MemoryAccessPerformanceImpact {
                performance_score: 1.0,
                cache_efficiency_impact: 1.0,
                bandwidth_impact: 1.0,
                pipeline_impact: 1.0,
                optimization_recommendations: vec![],
            },
        };

        assert_eq!(info.region_id, usize::MAX);
        assert_eq!(info.access_events[0].timestamp, u64::MAX);
    }

    #[test]
    fn test_multiple_access_events() {
        let events = vec![
            MemoryAccessEvent {
                access_type: MemoryAccessType::Read,
                timestamp: 100,
                address: 0x1000,
                size: 64,
                function_name: "read_data".to_string(),
                latency_ns: 10,
                cache_info: CacheAccessInfo {
                    l1_hit: true,
                    l2_hit: false,
                    l3_hit: false,
                    memory_access: false,
                    latency_breakdown: CacheLatencyBreakdown {
                        l1_latency_ns: 4.0,
                        l2_latency_ns: 0.0,
                        l3_latency_ns: 0.0,
                        memory_latency_ns: 0.0,
                    },
                },
            },
            MemoryAccessEvent {
                access_type: MemoryAccessType::Write,
                timestamp: 200,
                address: 0x2000,
                size: 128,
                function_name: "write_data".to_string(),
                latency_ns: 50,
                cache_info: CacheAccessInfo {
                    l1_hit: false,
                    l2_hit: false,
                    l3_hit: false,
                    memory_access: true,
                    latency_breakdown: CacheLatencyBreakdown {
                        l1_latency_ns: 0.0,
                        l2_latency_ns: 0.0,
                        l3_latency_ns: 0.0,
                        memory_latency_ns: 100.0,
                    },
                },
            },
        ];

        let info = MemoryAccessTrackingInfo {
            region_id: 1,
            address_range: AddressRange {
                start_address: 0x1000,
                end_address: 0x3000,
                size: 0x2000,
            },
            access_events: events.clone(),
            access_statistics: MemoryAccessStatistics {
                total_reads: 1,
                total_writes: 1,
                read_write_ratio: 1.0,
                avg_access_frequency: 0.0,
                peak_access_frequency: 0.0,
                locality_metrics: LocalityMetrics {
                    temporal_locality: 0.0,
                    spatial_locality: 0.0,
                    sequential_access_percent: 0.0,
                    random_access_percent: 0.0,
                    stride_patterns: vec![],
                },
                bandwidth_utilization: BandwidthUtilization {
                    peak_bandwidth: 0.0,
                    avg_bandwidth: 0.0,
                    efficiency_percent: 0.0,
                    bottlenecks: vec![],
                },
            },
            access_patterns: vec![],
            performance_impact: MemoryAccessPerformanceImpact {
                performance_score: 0.0,
                cache_efficiency_impact: 0.0,
                bandwidth_impact: 0.0,
                pipeline_impact: 0.0,
                optimization_recommendations: vec![],
            },
        };

        assert_eq!(info.access_events.len(), 2);
        assert!(matches!(
            info.access_events[0].access_type,
            MemoryAccessType::Read
        ));
        assert!(matches!(
            info.access_events[1].access_type,
            MemoryAccessType::Write
        ));
    }
}
