//! Generic type tracking types.
//!
//! This module contains types for tracking generic type instantiations,
//! monomorphization, and type parameters.
use serde::{Deserialize, Serialize};

/// Memory access pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryAccessPattern {
    /// Sequential access pattern.
    Sequential,
    /// Random access pattern.
    Random,
    /// Strided access pattern.
    Strided {
        /// Stride size.
        stride: usize,
    },
    /// Clustered access pattern.
    Clustered,
    /// Mixed access pattern.
    Mixed,
}

/// Generic type information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericTypeInfo {
    /// Generic base type name.
    pub base_type: String,
    /// Generic type parameters.
    pub type_parameters: Vec<TypeParameter>,
    /// Monomorphization information.
    pub monomorphization_info: MonomorphizationInfo,
    /// Generic constraint information.
    pub constraints: Vec<GenericConstraint>,
}

/// Generic type parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeParameter {
    /// Parameter name.
    pub name: String,
    /// Concrete type.
    pub concrete_type: String,
    /// Type size.
    pub size: usize,
    /// Type alignment.
    pub alignment: usize,
    /// Whether this is a lifetime parameter.
    pub is_lifetime: bool,
}

/// Monomorphization information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonomorphizationInfo {
    /// Number of monomorphization instances.
    pub instance_count: usize,
    /// Memory usage per instance.
    pub per_instance_memory: usize,
    /// Total memory usage.
    pub total_memory_usage: usize,
    /// Code bloat assessment.
    pub code_bloat_assessment: CodeBloatLevel,
}

/// Code bloat level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CodeBloatLevel {
    /// Low code bloat.
    Low,
    /// Moderate code bloat.
    Moderate,
    /// High code bloat.
    High,
    /// Excessive code bloat.
    Excessive,
}

/// Generic constraint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericConstraint {
    /// Constraint type.
    pub constraint_type: ConstraintType,
    /// Constraint description.
    pub description: String,
    /// Impact on memory layout.
    pub memory_impact: MemoryImpact,
}

/// Constraint type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Trait constraint.
    Trait(String),
    /// Lifetime constraint.
    Lifetime(String),
    /// Associated type constraint.
    Associated(String),
    /// Where clause constraint.
    Where(String),
}

/// Memory impact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryImpact {
    /// No memory impact.
    None,
    /// Size increase.
    SizeIncrease(usize),
    /// Alignment change.
    AlignmentChange(usize),
    /// Layout change.
    LayoutChange(String),
}

/// Enhanced generic instantiation tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericInstantiationInfo {
    /// Base generic type name.
    pub base_type: String,
    /// Concrete type parameters.
    pub concrete_parameters: Vec<ConcreteTypeParameter>,
    /// Instantiation location.
    pub instantiation_location: SourceLocation,
    /// Instantiation frequency.
    pub instantiation_count: usize,
    /// Memory usage per instantiation.
    pub memory_per_instance: usize,
    /// Total memory usage across all instances.
    pub total_memory_usage: usize,
    /// Compilation time impact.
    pub compilation_impact: CompilationImpact,
    /// Runtime performance characteristics.
    pub performance_characteristics: PerformanceCharacteristics,
}

/// Concrete type parameter with detailed information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConcreteTypeParameter {
    /// Parameter name.
    pub name: String,
    /// Concrete type.
    pub concrete_type: String,
    /// Type complexity score.
    pub complexity_score: u32,
    /// Memory footprint.
    pub memory_footprint: usize,
    /// Alignment requirements.
    pub alignment: usize,
    /// Whether type implements common traits.
    pub trait_implementations: Vec<String>,
    /// Type category.
    pub type_category: TypeCategory,
}

/// Type category classification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeCategory {
    /// Primitive type.
    Primitive,
    /// Struct type.
    Struct,
    /// Enum type.
    Enum,
    /// Union type.
    Union,
    /// Tuple type.
    Tuple,
    /// Array type.
    Array,
    /// Slice type.
    Slice,
    /// Reference type.
    Reference,
    /// Pointer type.
    Pointer,
    /// Function type.
    Function,
    /// Closure type.
    TraitObject,
    /// Generic type.
    Generic,
    /// Associated type.
    Associated,
}

/// Source location information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path.
    pub file: String,
    /// Line number.
    pub line: u32,
    /// Column number.
    pub column: u32,
}

/// Compilation impact assessment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompilationImpact {
    /// Estimated compilation time increase (milliseconds).
    pub compilation_time_ms: u64,
    /// Code size increase (bytes).
    pub code_size_increase: usize,
    /// LLVM IR complexity score.
    pub ir_complexity_score: u32,
    /// Optimization difficulty level.
    pub optimization_difficulty: OptimizationDifficulty,
}

/// Optimization difficulty levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationDifficulty {
    /// Easy optimization difficulty.
    Easy,
    /// Moderate optimization difficulty.
    Moderate,
    /// Hard optimization difficulty.
    Hard,
    /// Very hard optimization difficulty.
    VeryHard,
}

/// Performance characteristics of instantiated types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceCharacteristics {
    /// Average allocation time (nanoseconds).
    pub avg_allocation_time_ns: f64,
    /// Average deallocation time (nanoseconds).
    pub avg_deallocation_time_ns: f64,
    /// Memory access pattern.
    pub access_pattern: MemoryAccessPattern,
    /// Cache performance impact.
    pub cache_impact: CacheImpact,
    /// Branch prediction impact.
    pub branch_prediction_impact: BranchPredictionImpact,
}

impl Default for PerformanceCharacteristics {
    fn default() -> Self {
        Self {
            avg_allocation_time_ns: 0.0,
            avg_deallocation_time_ns: 0.0,
            access_pattern: MemoryAccessPattern::Sequential,
            cache_impact: CacheImpact::default(),
            branch_prediction_impact: BranchPredictionImpact::default(),
        }
    }
}

/// Cache impact assessment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheImpact {
    /// L1 cache impact score.
    pub l1_impact_score: f64,
    /// L2 cache impact score.
    pub l2_impact_score: f64,
    /// L3 cache impact score.
    pub l3_impact_score: f64,
    /// Cache line utilization efficiency.
    pub cache_line_efficiency: f64,
}

impl Default for CacheImpact {
    fn default() -> Self {
        Self {
            l1_impact_score: 0.0,
            l2_impact_score: 0.0,
            l3_impact_score: 0.0,
            cache_line_efficiency: 0.0,
        }
    }
}

/// Branch prediction impact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchPredictionImpact {
    /// Branch misprediction rate.
    pub misprediction_rate: f64,
    /// Impact on pipeline stalls.
    pub pipeline_stall_impact: f64,
    /// Predictability score.
    pub predictability_score: f64,
}

impl Default for BranchPredictionImpact {
    fn default() -> Self {
        Self {
            misprediction_rate: 0.0,
            pipeline_stall_impact: 0.0,
            predictability_score: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_type_info() {
        let info = GenericTypeInfo {
            base_type: "Vec".to_string(),
            type_parameters: vec![],
            monomorphization_info: MonomorphizationInfo {
                instance_count: 1,
                per_instance_memory: 24,
                total_memory_usage: 24,
                code_bloat_assessment: CodeBloatLevel::Low,
            },
            constraints: vec![],
        };

        assert_eq!(info.base_type, "Vec");
    }

    #[test]
    fn test_performance_characteristics_default() {
        let chars = PerformanceCharacteristics::default();
        assert_eq!(chars.avg_allocation_time_ns, 0.0);
    }

    #[test]
    fn test_cache_impact_default() {
        let impact = CacheImpact::default();
        assert_eq!(impact.l1_impact_score, 0.0);
    }
}
