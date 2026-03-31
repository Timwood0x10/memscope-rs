//! Memory access tracking types.
//!
//! This module contains types for tracking memory access patterns,
//! including access events, cache performance, and bandwidth utilization.

use serde::{Deserialize, Serialize};

use super::allocation::{ImpactLevel, Priority};
use super::generic::SourceLocation;

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
}
