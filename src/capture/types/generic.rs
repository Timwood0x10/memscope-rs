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

// Implement From trait for converting from core::types to capture::types
impl From<crate::core::types::GenericTypeInfo> for GenericTypeInfo {
    fn from(old: crate::core::types::GenericTypeInfo) -> Self {
        Self {
            base_type: old.base_type,
            type_parameters: old
                .type_parameters
                .into_iter()
                .map(|p| TypeParameter {
                    name: p.name,
                    concrete_type: p.concrete_type,
                    size: p.size,
                    alignment: p.alignment,
                    is_lifetime: p.is_lifetime,
                })
                .collect(),
            monomorphization_info: MonomorphizationInfo {
                instance_count: old.monomorphization_info.instance_count,
                per_instance_memory: old.monomorphization_info.per_instance_memory,
                total_memory_usage: old.monomorphization_info.total_memory_usage,
                code_bloat_assessment: match old.monomorphization_info.code_bloat_assessment {
                    crate::core::types::CodeBloatLevel::Low => CodeBloatLevel::Low,
                    crate::core::types::CodeBloatLevel::Moderate => CodeBloatLevel::Moderate,
                    crate::core::types::CodeBloatLevel::High => CodeBloatLevel::High,
                    crate::core::types::CodeBloatLevel::Excessive => CodeBloatLevel::Excessive,
                },
            },
            constraints: old
                .constraints
                .into_iter()
                .map(|c| GenericConstraint {
                    constraint_type: match c.constraint_type {
                        crate::core::types::ConstraintType::Trait(s) => ConstraintType::Trait(s),
                        crate::core::types::ConstraintType::Lifetime(s) => {
                            ConstraintType::Lifetime(s)
                        }
                        crate::core::types::ConstraintType::Associated(s) => {
                            ConstraintType::Associated(s)
                        }
                        crate::core::types::ConstraintType::Where(s) => ConstraintType::Where(s),
                    },
                    description: c.description,
                    memory_impact: match c.memory_impact {
                        crate::core::types::MemoryImpact::None => MemoryImpact::None,
                        crate::core::types::MemoryImpact::SizeIncrease(s) => {
                            MemoryImpact::SizeIncrease(s)
                        }
                        crate::core::types::MemoryImpact::AlignmentChange(s) => {
                            MemoryImpact::AlignmentChange(s)
                        }
                        crate::core::types::MemoryImpact::LayoutChange(s) => {
                            MemoryImpact::LayoutChange(s)
                        }
                    },
                })
                .collect(),
        }
    }
}

impl From<crate::core::types::GenericInstantiationInfo> for GenericInstantiationInfo {
    fn from(old: crate::core::types::GenericInstantiationInfo) -> Self {
        Self {
            base_type: old.base_type,
            concrete_parameters: old
                .concrete_parameters
                .into_iter()
                .map(|p| ConcreteTypeParameter {
                    name: p.name,
                    concrete_type: p.concrete_type,
                    complexity_score: p.complexity_score,
                    memory_footprint: p.memory_footprint,
                    alignment: p.alignment,
                    trait_implementations: p.trait_implementations,
                    type_category: match p.type_category {
                        crate::core::types::TypeCategory::Primitive => TypeCategory::Primitive,
                        crate::core::types::TypeCategory::Struct => TypeCategory::Struct,
                        crate::core::types::TypeCategory::Enum => TypeCategory::Enum,
                        crate::core::types::TypeCategory::Union => TypeCategory::Union,
                        crate::core::types::TypeCategory::Tuple => TypeCategory::Tuple,
                        crate::core::types::TypeCategory::Slice => TypeCategory::Slice,
                        crate::core::types::TypeCategory::Array => TypeCategory::Array,
                        crate::core::types::TypeCategory::Pointer => TypeCategory::Pointer,
                        crate::core::types::TypeCategory::Reference => TypeCategory::Reference,
                        crate::core::types::TypeCategory::Function => TypeCategory::Function,
                        crate::core::types::TypeCategory::TraitObject => TypeCategory::TraitObject,
                        crate::core::types::TypeCategory::Generic => TypeCategory::Generic,
                        crate::core::types::TypeCategory::Associated => TypeCategory::Associated,
                    },
                })
                .collect(),
            instantiation_location: SourceLocation {
                file: old.instantiation_location.file,
                line: old.instantiation_location.line,
                column: old.instantiation_location.column,
            },
            instantiation_count: old.instantiation_count,
            memory_per_instance: old.memory_per_instance,
            total_memory_usage: old.total_memory_usage,
            compilation_impact: CompilationImpact {
                compilation_time_ms: old.compilation_impact.compilation_time_ms,
                code_size_increase: old.compilation_impact.code_size_increase,
                ir_complexity_score: old.compilation_impact.ir_complexity_score,
                optimization_difficulty: match old.compilation_impact.optimization_difficulty {
                    crate::core::types::OptimizationDifficulty::Easy => {
                        OptimizationDifficulty::Easy
                    }
                    crate::core::types::OptimizationDifficulty::Moderate => {
                        OptimizationDifficulty::Moderate
                    }
                    crate::core::types::OptimizationDifficulty::Hard => {
                        OptimizationDifficulty::Hard
                    }
                    crate::core::types::OptimizationDifficulty::VeryHard => {
                        OptimizationDifficulty::VeryHard
                    }
                },
            },
            performance_characteristics: PerformanceCharacteristics {
                avg_allocation_time_ns: old.performance_characteristics.avg_allocation_time_ns,
                avg_deallocation_time_ns: old.performance_characteristics.avg_deallocation_time_ns,
                access_pattern: match old.performance_characteristics.access_pattern {
                    crate::core::types::MemoryAccessPattern::Sequential => {
                        MemoryAccessPattern::Sequential
                    }
                    crate::core::types::MemoryAccessPattern::Random => MemoryAccessPattern::Random,
                    crate::core::types::MemoryAccessPattern::Strided { stride } => {
                        MemoryAccessPattern::Strided { stride }
                    }
                    crate::core::types::MemoryAccessPattern::Clustered => {
                        MemoryAccessPattern::Clustered
                    }
                    crate::core::types::MemoryAccessPattern::Mixed => MemoryAccessPattern::Mixed,
                },
                cache_impact: CacheImpact {
                    l1_impact_score: old.performance_characteristics.cache_impact.l1_impact_score,
                    l2_impact_score: old.performance_characteristics.cache_impact.l2_impact_score,
                    l3_impact_score: old.performance_characteristics.cache_impact.l3_impact_score,
                    cache_line_efficiency: old
                        .performance_characteristics
                        .cache_impact
                        .cache_line_efficiency,
                },
                branch_prediction_impact: BranchPredictionImpact {
                    misprediction_rate: old
                        .performance_characteristics
                        .branch_prediction_impact
                        .misprediction_rate,
                    pipeline_stall_impact: old
                        .performance_characteristics
                        .branch_prediction_impact
                        .pipeline_stall_impact,
                    predictability_score: old
                        .performance_characteristics
                        .branch_prediction_impact
                        .predictability_score,
                },
            },
        }
    }
}
