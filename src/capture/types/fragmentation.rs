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

// Implement From trait for converting from core::types to capture::types
impl From<crate::core::types::EnhancedFragmentationAnalysis> for EnhancedFragmentationAnalysis {
    fn from(old: crate::core::types::EnhancedFragmentationAnalysis) -> Self {
        Self {
            total_heap_size: old.total_heap_size,
            used_heap_size: old.used_heap_size,
            free_heap_size: old.free_heap_size,
            free_block_count: old.free_block_count,
            free_block_distribution: old
                .free_block_distribution
                .into_iter()
                .map(|b| BlockSizeRange {
                    min_size: b.min_size,
                    max_size: b.max_size,
                    block_count: b.block_count,
                    total_size: b.total_size,
                })
                .collect(),
            fragmentation_metrics: FragmentationMetrics {
                external_fragmentation: old.fragmentation_metrics.external_fragmentation,
                internal_fragmentation: old.fragmentation_metrics.internal_fragmentation,
                largest_free_block: old.fragmentation_metrics.largest_free_block,
                average_free_block_size: old.fragmentation_metrics.average_free_block_size,
                severity_level: match old.fragmentation_metrics.severity_level {
                    crate::core::types::FragmentationSeverity::Low => FragmentationSeverity::Low,
                    crate::core::types::FragmentationSeverity::Moderate => {
                        FragmentationSeverity::Moderate
                    }
                    crate::core::types::FragmentationSeverity::High => FragmentationSeverity::High,
                    crate::core::types::FragmentationSeverity::Critical => {
                        FragmentationSeverity::Critical
                    }
                },
            },
            fragmentation_causes: old
                .fragmentation_causes
                .into_iter()
                .map(|c| FragmentationCause {
                    cause_type: match c.cause_type {
                        crate::core::types::FragmentationCauseType::MixedAllocationSizes => {
                            FragmentationCauseType::MixedAllocationSizes
                        }
                        crate::core::types::FragmentationCauseType::FrequentAllocDealloc => {
                            FragmentationCauseType::FrequentAllocDealloc
                        }
                        crate::core::types::FragmentationCauseType::LongLivedAllocations => {
                            FragmentationCauseType::LongLivedAllocations
                        }
                        crate::core::types::FragmentationCauseType::PoorAllocationStrategy => {
                            FragmentationCauseType::PoorAllocationStrategy
                        }
                        crate::core::types::FragmentationCauseType::MemoryLeaks => {
                            FragmentationCauseType::MemoryLeaks
                        }
                    },
                    description: c.description,
                    impact_level: match c.impact_level {
                        crate::core::types::ImpactLevel::Low => ImpactLevel::Low,
                        crate::core::types::ImpactLevel::Medium => ImpactLevel::Medium,
                        crate::core::types::ImpactLevel::High => ImpactLevel::High,
                        crate::core::types::ImpactLevel::Critical => ImpactLevel::Critical,
                    },
                    mitigation_suggestion: c.mitigation_suggestion,
                })
                .collect(),
        }
    }
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

    #[test]
    fn test_block_size_range_creation() {
        let range = BlockSizeRange {
            min_size: 0,
            max_size: 1024,
            block_count: 50,
            total_size: 25600,
        };

        assert_eq!(range.min_size, 0);
        assert_eq!(range.max_size, 1024);
        assert_eq!(range.block_count, 50);
        assert_eq!(range.total_size, 25600);
    }

    #[test]
    fn test_block_size_range_serialization() {
        let range = BlockSizeRange {
            min_size: 1024,
            max_size: 4096,
            block_count: 25,
            total_size: 64000,
        };

        let json = serde_json::to_string(&range).unwrap();
        let deserialized: BlockSizeRange = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, range);
    }

    #[test]
    fn test_fragmentation_metrics_all_severities() {
        let severities = [
            FragmentationSeverity::Low,
            FragmentationSeverity::Moderate,
            FragmentationSeverity::High,
            FragmentationSeverity::Critical,
        ];

        for severity in severities {
            let metrics = FragmentationMetrics {
                external_fragmentation: 0.5,
                internal_fragmentation: 0.2,
                largest_free_block: 1024,
                average_free_block_size: 512.0,
                severity_level: severity.clone(),
            };
            assert_eq!(metrics.severity_level, severity);
        }
    }

    #[test]
    fn test_fragmentation_metrics_boundary_values() {
        let metrics = FragmentationMetrics {
            external_fragmentation: 0.0,
            internal_fragmentation: 1.0,
            largest_free_block: 0,
            average_free_block_size: 0.0,
            severity_level: FragmentationSeverity::Critical,
        };

        assert!((metrics.external_fragmentation - 0.0).abs() < f64::EPSILON);
        assert!((metrics.internal_fragmentation - 1.0).abs() < f64::EPSILON);
        assert_eq!(metrics.largest_free_block, 0);
    }

    #[test]
    fn test_fragmentation_metrics_serialization() {
        let metrics = FragmentationMetrics {
            external_fragmentation: 0.75,
            internal_fragmentation: 0.25,
            largest_free_block: 2048,
            average_free_block_size: 1024.0,
            severity_level: FragmentationSeverity::High,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: FragmentationMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, metrics);
    }

    #[test]
    fn test_fragmentation_cause_creation() {
        let cause = FragmentationCause {
            cause_type: FragmentationCauseType::FrequentAllocDealloc,
            description: "Frequent allocation and deallocation pattern".to_string(),
            impact_level: ImpactLevel::High,
            mitigation_suggestion: "Use object pooling".to_string(),
        };

        assert!(matches!(
            cause.cause_type,
            FragmentationCauseType::FrequentAllocDealloc
        ));
        assert_eq!(
            cause.description,
            "Frequent allocation and deallocation pattern"
        );
        assert!(matches!(cause.impact_level, ImpactLevel::High));
    }

    #[test]
    fn test_all_fragmentation_cause_types() {
        let cause_types = [
            FragmentationCauseType::MixedAllocationSizes,
            FragmentationCauseType::FrequentAllocDealloc,
            FragmentationCauseType::LongLivedAllocations,
            FragmentationCauseType::PoorAllocationStrategy,
            FragmentationCauseType::MemoryLeaks,
        ];

        for cause_type in cause_types {
            let cause = FragmentationCause {
                cause_type: cause_type.clone(),
                description: "Test description".to_string(),
                impact_level: ImpactLevel::Low,
                mitigation_suggestion: "Test mitigation".to_string(),
            };
            assert_eq!(cause.cause_type, cause_type);
        }
    }

    #[test]
    fn test_fragmentation_cause_serialization() {
        let cause = FragmentationCause {
            cause_type: FragmentationCauseType::MemoryLeaks,
            description: "Memory leak detected".to_string(),
            impact_level: ImpactLevel::Critical,
            mitigation_suggestion: "Fix memory leaks".to_string(),
        };

        let json = serde_json::to_string(&cause).unwrap();
        let deserialized: FragmentationCause = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, cause);
    }

    #[test]
    fn test_all_fragmentation_severities_serialization() {
        let severities = vec![
            FragmentationSeverity::Low,
            FragmentationSeverity::Moderate,
            FragmentationSeverity::High,
            FragmentationSeverity::Critical,
        ];

        for severity in severities {
            let json = serde_json::to_string(&severity).unwrap();
            let deserialized: FragmentationSeverity = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, severity);
        }
    }

    #[test]
    fn test_all_cause_types_serialization() {
        let cause_types = vec![
            FragmentationCauseType::MixedAllocationSizes,
            FragmentationCauseType::FrequentAllocDealloc,
            FragmentationCauseType::LongLivedAllocations,
            FragmentationCauseType::PoorAllocationStrategy,
            FragmentationCauseType::MemoryLeaks,
        ];

        for cause_type in cause_types {
            let json = serde_json::to_string(&cause_type).unwrap();
            let deserialized: FragmentationCauseType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, cause_type);
        }
    }

    #[test]
    fn test_enhanced_fragmentation_analysis_with_distribution() {
        let distribution = vec![
            BlockSizeRange {
                min_size: 0,
                max_size: 256,
                block_count: 100,
                total_size: 12800,
            },
            BlockSizeRange {
                min_size: 256,
                max_size: 1024,
                block_count: 50,
                total_size: 32000,
            },
        ];

        let causes = vec![FragmentationCause {
            cause_type: FragmentationCauseType::MixedAllocationSizes,
            description: "Mixed allocation sizes".to_string(),
            impact_level: ImpactLevel::Medium,
            mitigation_suggestion: "Use size classes".to_string(),
        }];

        let analysis = EnhancedFragmentationAnalysis {
            total_heap_size: 1024 * 1024,
            used_heap_size: 768 * 1024,
            free_heap_size: 256 * 1024,
            free_block_count: 150,
            free_block_distribution: distribution.clone(),
            fragmentation_metrics: FragmentationMetrics {
                external_fragmentation: 0.6,
                internal_fragmentation: 0.15,
                largest_free_block: 65536,
                average_free_block_size: 1706.0,
                severity_level: FragmentationSeverity::Moderate,
            },
            fragmentation_causes: causes.clone(),
        };

        assert_eq!(analysis.free_block_distribution.len(), 2);
        assert_eq!(analysis.free_block_distribution[0].block_count, 100);
        assert_eq!(analysis.fragmentation_causes.len(), 1);
    }

    #[test]
    fn test_enhanced_fragmentation_analysis_serialization() {
        let analysis = EnhancedFragmentationAnalysis {
            total_heap_size: 2048 * 1024,
            used_heap_size: 1024 * 1024,
            free_heap_size: 1024 * 1024,
            free_block_count: 25,
            free_block_distribution: vec![BlockSizeRange {
                min_size: 0,
                max_size: 4096,
                block_count: 25,
                total_size: 51200,
            }],
            fragmentation_metrics: FragmentationMetrics {
                external_fragmentation: 0.4,
                internal_fragmentation: 0.1,
                largest_free_block: 32768,
                average_free_block_size: 40960.0,
                severity_level: FragmentationSeverity::Low,
            },
            fragmentation_causes: vec![],
        };

        let json = serde_json::to_string(&analysis).unwrap();
        let deserialized: EnhancedFragmentationAnalysis = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, analysis);
    }

    #[test]
    fn test_enhanced_fragmentation_analysis_clone() {
        let analysis = EnhancedFragmentationAnalysis {
            total_heap_size: 1024,
            used_heap_size: 512,
            free_heap_size: 512,
            free_block_count: 5,
            free_block_distribution: vec![],
            fragmentation_metrics: FragmentationMetrics {
                external_fragmentation: 0.5,
                internal_fragmentation: 0.2,
                largest_free_block: 256,
                average_free_block_size: 102.4,
                severity_level: FragmentationSeverity::Moderate,
            },
            fragmentation_causes: vec![],
        };

        let cloned = analysis.clone();
        assert_eq!(cloned.total_heap_size, analysis.total_heap_size);
        assert_eq!(cloned.free_block_count, analysis.free_block_count);
    }

    #[test]
    fn test_enhanced_fragmentation_analysis_debug() {
        let analysis = EnhancedFragmentationAnalysis {
            total_heap_size: 1024,
            used_heap_size: 512,
            free_heap_size: 512,
            free_block_count: 5,
            free_block_distribution: vec![],
            fragmentation_metrics: FragmentationMetrics {
                external_fragmentation: 0.5,
                internal_fragmentation: 0.2,
                largest_free_block: 256,
                average_free_block_size: 102.4,
                severity_level: FragmentationSeverity::Low,
            },
            fragmentation_causes: vec![],
        };

        let debug_str = format!("{:?}", analysis);
        assert!(debug_str.contains("EnhancedFragmentationAnalysis"));
        assert!(debug_str.contains("total_heap_size"));
    }

    #[test]
    fn test_block_size_range_equality() {
        let range1 = BlockSizeRange {
            min_size: 0,
            max_size: 1024,
            block_count: 10,
            total_size: 5120,
        };
        let range2 = BlockSizeRange {
            min_size: 0,
            max_size: 1024,
            block_count: 10,
            total_size: 5120,
        };
        let range3 = BlockSizeRange {
            min_size: 0,
            max_size: 2048,
            block_count: 10,
            total_size: 5120,
        };

        assert_eq!(range1, range2);
        assert_ne!(range1, range3);
    }

    #[test]
    fn test_fragmentation_severity_equality() {
        assert_eq!(FragmentationSeverity::Low, FragmentationSeverity::Low);
        assert_ne!(FragmentationSeverity::Low, FragmentationSeverity::High);
        assert_ne!(
            FragmentationSeverity::Moderate,
            FragmentationSeverity::Critical
        );
    }

    #[test]
    fn test_fragmentation_cause_type_equality() {
        assert_eq!(
            FragmentationCauseType::MixedAllocationSizes,
            FragmentationCauseType::MixedAllocationSizes
        );
        assert_ne!(
            FragmentationCauseType::MixedAllocationSizes,
            FragmentationCauseType::MemoryLeaks
        );
    }

    #[test]
    fn test_fragmentation_cause_with_all_impact_levels() {
        let impact_levels = [
            ImpactLevel::Low,
            ImpactLevel::Medium,
            ImpactLevel::High,
            ImpactLevel::Critical,
        ];

        for impact in impact_levels {
            let cause = FragmentationCause {
                cause_type: FragmentationCauseType::PoorAllocationStrategy,
                description: "Test".to_string(),
                impact_level: impact.clone(),
                mitigation_suggestion: "Fix it".to_string(),
            };
            assert_eq!(cause.impact_level, impact);
        }
    }

    #[test]
    fn test_empty_free_block_distribution() {
        let analysis = EnhancedFragmentationAnalysis {
            total_heap_size: 1024,
            used_heap_size: 0,
            free_heap_size: 1024,
            free_block_count: 0,
            free_block_distribution: vec![],
            fragmentation_metrics: FragmentationMetrics {
                external_fragmentation: 0.0,
                internal_fragmentation: 0.0,
                largest_free_block: 1024,
                average_free_block_size: 0.0,
                severity_level: FragmentationSeverity::Low,
            },
            fragmentation_causes: vec![],
        };

        assert!(analysis.free_block_distribution.is_empty());
        assert_eq!(analysis.free_block_count, 0);
    }

    #[test]
    fn test_large_heap_analysis() {
        let analysis = EnhancedFragmentationAnalysis {
            total_heap_size: usize::MAX / 2,
            used_heap_size: usize::MAX / 4,
            free_heap_size: usize::MAX / 4,
            free_block_count: 1000000,
            free_block_distribution: vec![],
            fragmentation_metrics: FragmentationMetrics {
                external_fragmentation: f64::MAX,
                internal_fragmentation: f64::MIN,
                largest_free_block: usize::MAX,
                average_free_block_size: f64::INFINITY,
                severity_level: FragmentationSeverity::Critical,
            },
            fragmentation_causes: vec![],
        };

        assert_eq!(analysis.total_heap_size, usize::MAX / 2);
        assert!(
            analysis
                .fragmentation_metrics
                .external_fragmentation
                .is_finite()
                || analysis
                    .fragmentation_metrics
                    .external_fragmentation
                    .is_infinite()
        );
    }
}
