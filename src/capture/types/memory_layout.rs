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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationPotential {
    /// No optimization needed.
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

impl Default for OptimizationPotential {
    fn default() -> Self {
        Self::None
    }
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
}
