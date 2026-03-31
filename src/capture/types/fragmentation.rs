//! Fragmentation analysis types.
//!
//! This module contains types for analyzing memory fragmentation,
//! including block distribution, metrics, and cause analysis.

use serde::{Deserialize, Serialize};

use super::allocation::ImpactLevel;

/// Enhanced memory fragmentation analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnhancedFragmentationAnalysis {
    /// Total heap size.
    pub total_heap_size: usize,
    /// Used heap size.
    pub used_heap_size: usize,
    /// Free heap size.
    pub free_heap_size: usize,
    /// Number of free blocks.
    pub free_block_count: usize,
    /// Free block size distribution.
    pub free_block_distribution: Vec<BlockSizeRange>,
    /// Fragmentation metrics.
    pub fragmentation_metrics: FragmentationMetrics,
    /// Allocation patterns causing fragmentation.
    pub fragmentation_causes: Vec<FragmentationCause>,
}

/// Block size range for distribution analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockSizeRange {
    /// Minimum size in range.
    pub min_size: usize,
    /// Maximum size in range.
    pub max_size: usize,
    /// Number of blocks in this range.
    pub block_count: usize,
    /// Total size of blocks in this range.
    pub total_size: usize,
}

/// Fragmentation metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FragmentationMetrics {
    /// External fragmentation ratio.
    pub external_fragmentation: f64,
    /// Internal fragmentation ratio.
    pub internal_fragmentation: f64,
    /// Largest free block size.
    pub largest_free_block: usize,
    /// Average free block size.
    pub average_free_block_size: f64,
    /// Fragmentation severity level.
    pub severity_level: FragmentationSeverity,
}

/// Fragmentation severity levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FragmentationSeverity {
    /// Low fragmentation severity.
    Low,
    /// Moderate fragmentation severity.
    Moderate,
    /// High fragmentation severity.
    High,
    /// Critical fragmentation severity.
    Critical,
}

/// Fragmentation cause analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FragmentationCause {
    /// Cause type.
    pub cause_type: FragmentationCauseType,
    /// Description of the cause.
    pub description: String,
    /// Impact on fragmentation.
    pub impact_level: ImpactLevel,
    /// Suggested mitigation.
    pub mitigation_suggestion: String,
}

/// Types of fragmentation causes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FragmentationCauseType {
    /// Mixed allocation sizes.
    MixedAllocationSizes,
    /// Frequent allocation/deallocation.
    FrequentAllocDealloc,
    /// Long-lived allocations blocking coalescing.
    LongLivedAllocations,
    /// Poor allocation strategy.
    PoorAllocationStrategy,
    /// Memory leaks.
    MemoryLeaks,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_fragmentation_analysis() {
        let analysis = EnhancedFragmentationAnalysis {
            total_heap_size: 1024 * 1024,
            used_heap_size: 512 * 1024,
            free_heap_size: 512 * 1024,
            free_block_count: 10,
            free_block_distribution: vec![],
            fragmentation_metrics: FragmentationMetrics {
                external_fragmentation: 0.3,
                internal_fragmentation: 0.1,
                largest_free_block: 256 * 1024,
                average_free_block_size: 51200.0,
                severity_level: FragmentationSeverity::Low,
            },
            fragmentation_causes: vec![],
        };

        assert_eq!(analysis.total_heap_size, 1024 * 1024);
        assert_eq!(analysis.free_block_count, 10);
    }

    #[test]
    fn test_fragmentation_severity() {
        let severity = FragmentationSeverity::High;
        assert!(matches!(severity, FragmentationSeverity::High));
    }

    #[test]
    fn test_fragmentation_cause_type() {
        let cause = FragmentationCauseType::MixedAllocationSizes;
        assert!(matches!(
            cause,
            FragmentationCauseType::MixedAllocationSizes
        ));
    }
}
