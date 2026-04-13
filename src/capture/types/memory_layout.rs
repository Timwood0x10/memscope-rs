//! Memory layout analysis types.
//!
//! This module contains types for analyzing memory layout,
//! including field layout, padding analysis, and container analysis.

use serde::{Deserialize, Serialize};

/// Detailed memory layout analysis information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryLayoutInfo {
    /// Total size of the type in bytes.
    pub total_size: usize,
    /// Alignment requirement of the type.
    pub alignment: usize,
    /// Field layout information.
    pub field_layout: Vec<FieldLayoutInfo>,
    /// Padding byte information.
    pub padding_info: PaddingAnalysis,
    /// Memory layout efficiency analysis.
    pub layout_efficiency: LayoutEfficiency,
    /// Container-specific analysis (Vec, HashMap, Box, etc.).
    pub container_analysis: Option<ContainerAnalysis>,
}

/// Field layout information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldLayoutInfo {
    /// Field name.
    pub field_name: String,
    /// Field type.
    pub field_type: String,
    /// Field offset within the struct.
    pub offset: usize,
    /// Field size.
    pub size: usize,
    /// Field alignment requirement.
    pub alignment: usize,
    /// Whether this is a padding field.
    pub is_padding: bool,
}

/// Padding byte analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PaddingAnalysis {
    /// Total number of padding bytes.
    pub total_padding_bytes: usize,
    /// Padding byte locations.
    pub padding_locations: Vec<PaddingLocation>,
    /// Padding ratio (padding bytes / total size).
    pub padding_ratio: f64,
    /// Optimization suggestions.
    pub optimization_suggestions: Vec<String>,
}

/// Layout efficiency analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LayoutEfficiency {
    /// Memory utilization (useful data / total size).
    pub memory_utilization: f64,
    /// Cache friendliness score (0-100).
    pub cache_friendliness: f64,
    /// Alignment waste in bytes.
    pub alignment_waste: usize,
    /// Optimization potential assessment.
    pub optimization_potential: OptimizationPotential,
}

/// Padding byte location.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaddingLocation {
    /// Padding start offset.
    pub start_offset: usize,
    /// Padding size.
    pub size: usize,
    /// Padding reason.
    pub reason: PaddingReason,
}

/// Padding reason.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaddingReason {
    /// Field alignment.
    FieldAlignment,
    /// Struct tail alignment.
    StructAlignment,
    /// Enum discriminant alignment.
    EnumDiscriminant,
    /// Other reason.
    Other(String),
}

/// Optimization potential assessment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum OptimizationPotential {
    /// No optimization needed.
    #[default]
    None,
    /// Minor optimization potential.
    Minor {
        /// Potential memory savings in bytes.
        potential_savings: usize,
    },
    /// Moderate optimization.
    Moderate {
        /// Potential savings in bytes.
        potential_savings: usize,
        /// Optimization suggestions.
        suggestions: Vec<String>,
    },
    /// Major optimization.
    Major {
        /// Potential savings in bytes.
        potential_savings: usize,
        /// Optimization suggestions.
        suggestions: Vec<String>,
    },
}

/// Container-specific analysis for Vec, HashMap, Box, etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerAnalysis {
    /// Type of container (Vec, HashMap, Box, etc.).
    pub container_type: ContainerType,
    /// Capacity utilization analysis.
    pub capacity_utilization: CapacityUtilization,
    /// Reallocation pattern detection.
    pub reallocation_patterns: ReallocationPatterns,
    /// Container-specific efficiency metrics.
    pub efficiency_metrics: ContainerEfficiencyMetrics,
}

/// Container type classification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContainerType {
    /// `Vec<T>` container.
    Vec {
        /// Element type.
        element_type: String,
        /// Element size in bytes.
        element_size: usize,
    },
    /// HashMap<K, V> container.
    HashMap {
        /// Key type.
        key_type: String,
        /// Value type.
        value_type: String,
        /// Key size in bytes.
        key_size: usize,
        /// Value size in bytes.
        value_size: usize,
    },
    /// `Box<T>` container.
    Box {
        /// Boxed type.
        boxed_type: String,
        /// Boxed type size in bytes.
        boxed_size: usize,
    },
    /// String container (special case of `Vec<u8>`).
    String,
    /// `Rc<T>` reference counted container.
    Rc {
        /// Referenced type.
        referenced_type: String,
        /// Referenced type size in bytes.
        referenced_size: usize,
    },
    /// `Arc<T>` atomic reference counted container.
    Arc {
        /// Referenced type.
        referenced_type: String,
        /// Referenced type size in bytes.
        referenced_size: usize,
    },
    /// BTreeMap<K, V> container.
    BTreeMap {
        /// Key type.
        key_type: String,
        /// Value type.
        value_type: String,
        /// Key size in bytes.
        key_size: usize,
        /// Value size in bytes.
        value_size: usize,
    },
    /// Other container type.
    Other {
        /// Container type name.
        type_name: String,
    },
}

/// Capacity utilization analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapacityUtilization {
    /// Current capacity (estimated).
    pub current_capacity: usize,
    /// Current length/size (estimated).
    pub current_length: usize,
    /// Utilization ratio (length / capacity).
    pub utilization_ratio: f64,
    /// Wasted space in bytes.
    pub wasted_space: usize,
    /// Efficiency assessment.
    pub efficiency_assessment: UtilizationEfficiency,
}

/// Utilization efficiency assessment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UtilizationEfficiency {
    /// Excellent utilization (>90%).
    Excellent,
    /// Good utilization (70-90%).
    Good,
    /// Fair utilization (50-70%).
    Fair,
    /// Poor utilization (<50%).
    Poor {
        /// Suggested optimization.
        suggestion: String,
    },
}

/// Reallocation pattern detection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReallocationPatterns {
    /// Estimated number of reallocations.
    pub estimated_reallocations: usize,
    /// Growth pattern (exponential, linear, etc.).
    pub growth_pattern: GrowthPattern,
    /// Reallocation frequency assessment.
    pub frequency_assessment: ReallocationFrequency,
    /// Optimization suggestions.
    pub optimization_suggestions: Vec<String>,
}

/// Growth pattern classification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GrowthPattern {
    /// Exponential growth (typical for Vec).
    Exponential,
    /// Linear growth.
    Linear,
    /// Irregular growth.
    Irregular,
    /// Single allocation (no growth).
    SingleAllocation,
    /// Unknown pattern.
    Unknown,
}

/// Reallocation frequency assessment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReallocationFrequency {
    /// No reallocations detected.
    None,
    /// Low frequency (acceptable).
    Low,
    /// Moderate frequency (consider optimization).
    Moderate,
    /// High frequency (optimization recommended).
    High {
        /// Performance impact estimate.
        performance_impact: f64,
    },
}

/// Container-specific efficiency metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerEfficiencyMetrics {
    /// Memory overhead percentage.
    pub memory_overhead: f64,
    /// Cache efficiency score (0-100).
    pub cache_efficiency: f64,
    /// Access pattern efficiency.
    pub access_efficiency: AccessEfficiency,
    /// Overall container health score (0-100).
    pub health_score: f64,
}

/// Access pattern efficiency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccessEfficiency {
    /// Sequential access pattern (cache-friendly).
    Sequential,
    /// Random access pattern (cache-unfriendly).
    Random,
    /// Mixed access pattern.
    Mixed,
    /// Unknown access pattern.
    Unknown,
}

impl From<crate::core::types::MemoryLayoutInfo> for MemoryLayoutInfo {
    fn from(old: crate::core::types::MemoryLayoutInfo) -> Self {
        Self {
            total_size: old.total_size,
            alignment: old.alignment,
            field_layout: old
                .field_layout
                .into_iter()
                .map(|f| FieldLayoutInfo {
                    field_name: f.field_name,
                    field_type: f.field_type,
                    offset: f.offset,
                    size: f.size,
                    alignment: f.alignment,
                    is_padding: f.is_padding,
                })
                .collect(),
            padding_info: PaddingAnalysis {
                total_padding_bytes: old.padding_info.total_padding_bytes,
                padding_locations: old
                    .padding_info
                    .padding_locations
                    .into_iter()
                    .map(|l| PaddingLocation {
                        start_offset: l.start_offset,
                        size: l.size,
                        reason: match l.reason {
                            crate::core::types::PaddingReason::FieldAlignment => {
                                PaddingReason::FieldAlignment
                            }
                            crate::core::types::PaddingReason::StructAlignment => {
                                PaddingReason::StructAlignment
                            }
                            crate::core::types::PaddingReason::EnumDiscriminant => {
                                PaddingReason::EnumDiscriminant
                            }
                            crate::core::types::PaddingReason::Other(s) => PaddingReason::Other(s),
                        },
                    })
                    .collect(),
                padding_ratio: old.padding_info.padding_ratio,
                optimization_suggestions: old.padding_info.optimization_suggestions,
            },
            layout_efficiency: LayoutEfficiency {
                memory_utilization: old.layout_efficiency.memory_utilization,
                cache_friendliness: old.layout_efficiency.cache_friendliness,
                alignment_waste: old.layout_efficiency.alignment_waste,
                optimization_potential: match old.layout_efficiency.optimization_potential {
                    crate::core::types::OptimizationPotential::None => OptimizationPotential::None,
                    crate::core::types::OptimizationPotential::Minor { potential_savings } => {
                        OptimizationPotential::Minor { potential_savings }
                    }
                    crate::core::types::OptimizationPotential::Moderate {
                        potential_savings,
                        suggestions,
                    } => OptimizationPotential::Moderate {
                        potential_savings,
                        suggestions,
                    },
                    crate::core::types::OptimizationPotential::Major {
                        potential_savings,
                        suggestions,
                    } => OptimizationPotential::Major {
                        potential_savings,
                        suggestions,
                    },
                },
            },
            container_analysis: old.container_analysis.map(|c| ContainerAnalysis {
                container_type: match c.container_type {
                    crate::core::types::ContainerType::Vec {
                        element_type,
                        element_size,
                    } => ContainerType::Vec {
                        element_type,
                        element_size,
                    },
                    crate::core::types::ContainerType::HashMap {
                        key_type,
                        value_type,
                        key_size,
                        value_size,
                    } => ContainerType::HashMap {
                        key_type,
                        value_type,
                        key_size,
                        value_size,
                    },
                    crate::core::types::ContainerType::Box {
                        boxed_type,
                        boxed_size,
                    } => ContainerType::Box {
                        boxed_type,
                        boxed_size,
                    },
                    crate::core::types::ContainerType::String => ContainerType::String,
                    crate::core::types::ContainerType::Rc {
                        referenced_type,
                        referenced_size,
                    } => ContainerType::Rc {
                        referenced_type,
                        referenced_size,
                    },
                    crate::core::types::ContainerType::Arc {
                        referenced_type,
                        referenced_size,
                    } => ContainerType::Arc {
                        referenced_type,
                        referenced_size,
                    },
                    crate::core::types::ContainerType::BTreeMap {
                        key_type,
                        value_type,
                        key_size,
                        value_size,
                    } => ContainerType::BTreeMap {
                        key_type,
                        value_type,
                        key_size,
                        value_size,
                    },
                    crate::core::types::ContainerType::Other { type_name } => {
                        ContainerType::Other { type_name }
                    }
                },
                capacity_utilization: CapacityUtilization {
                    current_capacity: c.capacity_utilization.current_capacity,
                    current_length: c.capacity_utilization.current_length,
                    utilization_ratio: c.capacity_utilization.utilization_ratio,
                    wasted_space: c.capacity_utilization.wasted_space,
                    efficiency_assessment: match c.capacity_utilization.efficiency_assessment {
                        crate::core::types::UtilizationEfficiency::Excellent => {
                            UtilizationEfficiency::Excellent
                        }
                        crate::core::types::UtilizationEfficiency::Good => {
                            UtilizationEfficiency::Good
                        }
                        crate::core::types::UtilizationEfficiency::Fair => {
                            UtilizationEfficiency::Fair
                        }
                        crate::core::types::UtilizationEfficiency::Poor { suggestion } => {
                            UtilizationEfficiency::Poor { suggestion }
                        }
                    },
                },
                reallocation_patterns: ReallocationPatterns {
                    estimated_reallocations: c.reallocation_patterns.estimated_reallocations,
                    growth_pattern: match c.reallocation_patterns.growth_pattern {
                        crate::core::types::GrowthPattern::Exponential => {
                            GrowthPattern::Exponential
                        }
                        crate::core::types::GrowthPattern::Linear => GrowthPattern::Linear,
                        crate::core::types::GrowthPattern::Irregular => GrowthPattern::Irregular,
                        crate::core::types::GrowthPattern::SingleAllocation => {
                            GrowthPattern::SingleAllocation
                        }
                        crate::core::types::GrowthPattern::Unknown => GrowthPattern::Unknown,
                    },
                    frequency_assessment: match c.reallocation_patterns.frequency_assessment {
                        crate::core::types::ReallocationFrequency::None => {
                            ReallocationFrequency::None
                        }
                        crate::core::types::ReallocationFrequency::Low => {
                            ReallocationFrequency::Low
                        }
                        crate::core::types::ReallocationFrequency::Moderate => {
                            ReallocationFrequency::Moderate
                        }
                        crate::core::types::ReallocationFrequency::High { performance_impact } => {
                            ReallocationFrequency::High { performance_impact }
                        }
                    },
                    optimization_suggestions: c.reallocation_patterns.optimization_suggestions,
                },
                efficiency_metrics: ContainerEfficiencyMetrics {
                    memory_overhead: c.efficiency_metrics.memory_overhead,
                    cache_efficiency: c.efficiency_metrics.cache_efficiency,
                    access_efficiency: match c.efficiency_metrics.access_efficiency {
                        crate::core::types::AccessEfficiency::Sequential => {
                            AccessEfficiency::Sequential
                        }
                        crate::core::types::AccessEfficiency::Random => AccessEfficiency::Random,
                        crate::core::types::AccessEfficiency::Mixed => AccessEfficiency::Mixed,
                        crate::core::types::AccessEfficiency::Unknown => AccessEfficiency::Unknown,
                    },
                    health_score: c.efficiency_metrics.health_score,
                },
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_layout_info() {
        let layout = MemoryLayoutInfo {
            total_size: 64,
            alignment: 8,
            field_layout: vec![],
            padding_info: PaddingAnalysis::default(),
            layout_efficiency: LayoutEfficiency::default(),
            container_analysis: None,
        };

        assert_eq!(layout.total_size, 64);
        assert_eq!(layout.alignment, 8);
    }

    #[test]
    fn test_padding_analysis_default() {
        let padding = PaddingAnalysis::default();

        assert_eq!(padding.total_padding_bytes, 0);
        assert_eq!(padding.padding_ratio, 0.0);
        assert!(padding.padding_locations.is_empty());
    }

    #[test]
    fn test_container_type_vec() {
        let container = ContainerType::Vec {
            element_type: "i32".to_string(),
            element_size: 4,
        };

        assert!(matches!(container, ContainerType::Vec { .. }));
    }

    #[test]
    fn test_field_layout_info_creation() {
        let field = FieldLayoutInfo {
            field_name: "x".to_string(),
            field_type: "i32".to_string(),
            offset: 0,
            size: 4,
            alignment: 4,
            is_padding: false,
        };

        assert_eq!(field.field_name, "x");
        assert_eq!(field.size, 4);
        assert!(!field.is_padding);
    }

    #[test]
    fn test_field_layout_info_padding() {
        let padding_field = FieldLayoutInfo {
            field_name: "_padding".to_string(),
            field_type: "[u8; 4]".to_string(),
            offset: 4,
            size: 4,
            alignment: 1,
            is_padding: true,
        };

        assert!(padding_field.is_padding);
    }

    #[test]
    fn test_padding_analysis_with_locations() {
        let padding = PaddingAnalysis {
            total_padding_bytes: 8,
            padding_locations: vec![PaddingLocation {
                start_offset: 4,
                size: 4,
                reason: PaddingReason::FieldAlignment,
            }],
            padding_ratio: 0.125,
            optimization_suggestions: vec!["Reorder fields".to_string()],
        };

        assert_eq!(padding.total_padding_bytes, 8);
        assert_eq!(padding.padding_locations.len(), 1);
    }

    #[test]
    fn test_padding_reason_variants() {
        let reasons = vec![
            PaddingReason::FieldAlignment,
            PaddingReason::StructAlignment,
            PaddingReason::EnumDiscriminant,
            PaddingReason::Other("custom reason".to_string()),
        ];

        for reason in reasons {
            let loc = PaddingLocation {
                start_offset: 0,
                size: 4,
                reason: reason.clone(),
            };
            assert_eq!(loc.reason, reason);
        }
    }

    #[test]
    fn test_layout_efficiency_default() {
        let efficiency = LayoutEfficiency::default();

        assert!((efficiency.memory_utilization - 0.0).abs() < f64::EPSILON);
        assert!((efficiency.cache_friendliness - 0.0).abs() < f64::EPSILON);
        assert_eq!(efficiency.alignment_waste, 0);
    }

    #[test]
    fn test_layout_efficiency_with_values() {
        let efficiency = LayoutEfficiency {
            memory_utilization: 0.875,
            cache_friendliness: 75.0,
            alignment_waste: 4,
            optimization_potential: OptimizationPotential::Minor {
                potential_savings: 8,
            },
        };

        assert!((efficiency.memory_utilization - 0.875).abs() < f64::EPSILON);
        assert!((efficiency.cache_friendliness - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_optimization_potential_variants() {
        let none = OptimizationPotential::None;
        let minor = OptimizationPotential::Minor {
            potential_savings: 8,
        };
        let moderate = OptimizationPotential::Moderate {
            potential_savings: 16,
            suggestions: vec!["Reorder fields".to_string()],
        };
        let major = OptimizationPotential::Major {
            potential_savings: 32,
            suggestions: vec!["Use packed struct".to_string()],
        };

        assert!(matches!(none, OptimizationPotential::None));
        assert!(matches!(minor, OptimizationPotential::Minor { .. }));
        assert!(matches!(moderate, OptimizationPotential::Moderate { .. }));
        assert!(matches!(major, OptimizationPotential::Major { .. }));
    }

    #[test]
    fn test_container_type_hashmap() {
        let container = ContainerType::HashMap {
            key_type: "String".to_string(),
            value_type: "i32".to_string(),
            key_size: 24,
            value_size: 4,
        };

        assert!(matches!(container, ContainerType::HashMap { .. }));
    }

    #[test]
    fn test_container_type_box() {
        let container = ContainerType::Box {
            boxed_type: "MyStruct".to_string(),
            boxed_size: 64,
        };

        assert!(matches!(container, ContainerType::Box { .. }));
    }

    #[test]
    fn test_container_type_string() {
        let container = ContainerType::String;
        assert!(matches!(container, ContainerType::String));
    }

    #[test]
    fn test_container_type_rc() {
        let container = ContainerType::Rc {
            referenced_type: "Vec<i32>".to_string(),
            referenced_size: 24,
        };

        assert!(matches!(container, ContainerType::Rc { .. }));
    }

    #[test]
    fn test_container_type_arc() {
        let container = ContainerType::Arc {
            referenced_type: "HashMap<String, i32>".to_string(),
            referenced_size: 48,
        };

        assert!(matches!(container, ContainerType::Arc { .. }));
    }

    #[test]
    fn test_container_type_btree_map() {
        let container = ContainerType::BTreeMap {
            key_type: "String".to_string(),
            value_type: "Vec<u8>".to_string(),
            key_size: 24,
            value_size: 24,
        };

        assert!(matches!(container, ContainerType::BTreeMap { .. }));
    }

    #[test]
    fn test_container_type_other() {
        let container = ContainerType::Other {
            type_name: "CustomContainer".to_string(),
        };

        assert!(matches!(container, ContainerType::Other { .. }));
    }

    #[test]
    fn test_capacity_utilization() {
        let utilization = CapacityUtilization {
            current_capacity: 16,
            current_length: 10,
            utilization_ratio: 0.625,
            wasted_space: 24,
            efficiency_assessment: UtilizationEfficiency::Good,
        };

        assert_eq!(utilization.current_capacity, 16);
        assert_eq!(utilization.current_length, 10);
    }

    #[test]
    fn test_utilization_efficiency_variants() {
        let excellent = UtilizationEfficiency::Excellent;
        let good = UtilizationEfficiency::Good;
        let fair = UtilizationEfficiency::Fair;
        let poor = UtilizationEfficiency::Poor {
            suggestion: "Shrink container".to_string(),
        };

        assert!(matches!(excellent, UtilizationEfficiency::Excellent));
        assert!(matches!(good, UtilizationEfficiency::Good));
        assert!(matches!(fair, UtilizationEfficiency::Fair));
        assert!(matches!(poor, UtilizationEfficiency::Poor { .. }));
    }

    #[test]
    fn test_reallocation_patterns() {
        let patterns = ReallocationPatterns {
            estimated_reallocations: 5,
            growth_pattern: GrowthPattern::Exponential,
            frequency_assessment: ReallocationFrequency::Low,
            optimization_suggestions: vec![],
        };

        assert_eq!(patterns.estimated_reallocations, 5);
    }

    #[test]
    fn test_growth_pattern_variants() {
        let patterns = vec![
            GrowthPattern::Exponential,
            GrowthPattern::Linear,
            GrowthPattern::Irregular,
            GrowthPattern::SingleAllocation,
            GrowthPattern::Unknown,
        ];

        for pattern in patterns {
            let rp = ReallocationPatterns {
                estimated_reallocations: 0,
                growth_pattern: pattern.clone(),
                frequency_assessment: ReallocationFrequency::None,
                optimization_suggestions: vec![],
            };
            assert_eq!(rp.growth_pattern, pattern);
        }
    }

    #[test]
    fn test_reallocation_frequency_variants() {
        let frequencies = vec![
            ReallocationFrequency::None,
            ReallocationFrequency::Low,
            ReallocationFrequency::Moderate,
            ReallocationFrequency::High {
                performance_impact: 0.5,
            },
        ];

        for freq in frequencies {
            let rp = ReallocationPatterns {
                estimated_reallocations: 0,
                growth_pattern: GrowthPattern::Unknown,
                frequency_assessment: freq.clone(),
                optimization_suggestions: vec![],
            };
            assert_eq!(rp.frequency_assessment, freq);
        }
    }

    #[test]
    fn test_container_efficiency_metrics() {
        let metrics = ContainerEfficiencyMetrics {
            memory_overhead: 12.5,
            cache_efficiency: 85.0,
            access_efficiency: AccessEfficiency::Sequential,
            health_score: 90.0,
        };

        assert!((metrics.memory_overhead - 12.5).abs() < f64::EPSILON);
        assert!((metrics.health_score - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_access_efficiency_variants() {
        let variants = vec![
            AccessEfficiency::Sequential,
            AccessEfficiency::Random,
            AccessEfficiency::Mixed,
            AccessEfficiency::Unknown,
        ];

        for variant in variants {
            let metrics = ContainerEfficiencyMetrics {
                memory_overhead: 0.0,
                cache_efficiency: 0.0,
                access_efficiency: variant.clone(),
                health_score: 0.0,
            };
            assert_eq!(metrics.access_efficiency, variant);
        }
    }

    #[test]
    fn test_container_analysis_full() {
        let analysis = ContainerAnalysis {
            container_type: ContainerType::Vec {
                element_type: "u8".to_string(),
                element_size: 1,
            },
            capacity_utilization: CapacityUtilization {
                current_capacity: 32,
                current_length: 20,
                utilization_ratio: 0.625,
                wasted_space: 12,
                efficiency_assessment: UtilizationEfficiency::Good,
            },
            reallocation_patterns: ReallocationPatterns {
                estimated_reallocations: 3,
                growth_pattern: GrowthPattern::Exponential,
                frequency_assessment: ReallocationFrequency::Low,
                optimization_suggestions: vec![],
            },
            efficiency_metrics: ContainerEfficiencyMetrics {
                memory_overhead: 0.0,
                cache_efficiency: 95.0,
                access_efficiency: AccessEfficiency::Sequential,
                health_score: 95.0,
            },
        };

        assert!(matches!(analysis.container_type, ContainerType::Vec { .. }));
    }

    #[test]
    fn test_memory_layout_info_with_fields() {
        let layout = MemoryLayoutInfo {
            total_size: 16,
            alignment: 8,
            field_layout: vec![
                FieldLayoutInfo {
                    field_name: "a".to_string(),
                    field_type: "u64".to_string(),
                    offset: 0,
                    size: 8,
                    alignment: 8,
                    is_padding: false,
                },
                FieldLayoutInfo {
                    field_name: "b".to_string(),
                    field_type: "u32".to_string(),
                    offset: 8,
                    size: 4,
                    alignment: 4,
                    is_padding: false,
                },
            ],
            padding_info: PaddingAnalysis {
                total_padding_bytes: 4,
                padding_locations: vec![PaddingLocation {
                    start_offset: 12,
                    size: 4,
                    reason: PaddingReason::StructAlignment,
                }],
                padding_ratio: 0.25,
                optimization_suggestions: vec![],
            },
            layout_efficiency: LayoutEfficiency {
                memory_utilization: 0.75,
                cache_friendliness: 80.0,
                alignment_waste: 4,
                optimization_potential: OptimizationPotential::Minor {
                    potential_savings: 4,
                },
            },
            container_analysis: None,
        };

        assert_eq!(layout.field_layout.len(), 2);
        assert_eq!(layout.padding_info.total_padding_bytes, 4);
    }

    #[test]
    fn test_memory_layout_info_serialization() {
        let layout = MemoryLayoutInfo {
            total_size: 8,
            alignment: 8,
            field_layout: vec![],
            padding_info: PaddingAnalysis::default(),
            layout_efficiency: LayoutEfficiency::default(),
            container_analysis: None,
        };

        let json = serde_json::to_string(&layout).unwrap();
        let deserialized: MemoryLayoutInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_size, layout.total_size);
    }

    #[test]
    fn test_container_type_serialization() {
        let container = ContainerType::Vec {
            element_type: "i32".to_string(),
            element_size: 4,
        };

        let json = serde_json::to_string(&container).unwrap();
        let deserialized: ContainerType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, container);
    }

    #[test]
    fn test_optimization_potential_default() {
        let potential = OptimizationPotential::default();
        assert!(matches!(potential, OptimizationPotential::None));
    }

    #[test]
    fn test_padding_analysis_serialization() {
        let padding = PaddingAnalysis {
            total_padding_bytes: 16,
            padding_locations: vec![],
            padding_ratio: 0.2,
            optimization_suggestions: vec!["Reorder fields".to_string()],
        };

        let json = serde_json::to_string(&padding).unwrap();
        let deserialized: PaddingAnalysis = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.total_padding_bytes,
            padding.total_padding_bytes
        );
    }
}
