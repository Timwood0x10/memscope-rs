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

    #[test]
    fn test_branch_prediction_impact_default() {
        let impact = BranchPredictionImpact::default();
        assert_eq!(impact.misprediction_rate, 0.0);
        assert_eq!(impact.pipeline_stall_impact, 0.0);
        assert_eq!(impact.predictability_score, 0.0);
    }

    #[test]
    fn test_memory_access_pattern_variants() {
        let patterns = vec![
            MemoryAccessPattern::Sequential,
            MemoryAccessPattern::Random,
            MemoryAccessPattern::Strided { stride: 64 },
            MemoryAccessPattern::Clustered,
            MemoryAccessPattern::Mixed,
        ];

        for pattern in patterns {
            let chars = PerformanceCharacteristics {
                avg_allocation_time_ns: 0.0,
                avg_deallocation_time_ns: 0.0,
                access_pattern: pattern.clone(),
                cache_impact: CacheImpact::default(),
                branch_prediction_impact: BranchPredictionImpact::default(),
            };
            assert_eq!(chars.access_pattern, pattern);
        }
    }

    #[test]
    fn test_code_bloat_level_variants() {
        let levels = vec![
            CodeBloatLevel::Low,
            CodeBloatLevel::Moderate,
            CodeBloatLevel::High,
            CodeBloatLevel::Excessive,
        ];

        for level in levels {
            let info = MonomorphizationInfo {
                instance_count: 0,
                per_instance_memory: 0,
                total_memory_usage: 0,
                code_bloat_assessment: level.clone(),
            };
            assert_eq!(info.code_bloat_assessment, level);
        }
    }

    #[test]
    fn test_constraint_type_variants() {
        let constraints = vec![
            ConstraintType::Trait("Clone".to_string()),
            ConstraintType::Lifetime("'static".to_string()),
            ConstraintType::Associated("Item".to_string()),
            ConstraintType::Where("T: Send".to_string()),
        ];

        for constraint_type in constraints {
            let constraint = GenericConstraint {
                constraint_type: constraint_type.clone(),
                description: String::new(),
                memory_impact: MemoryImpact::None,
            };
            assert_eq!(constraint.constraint_type, constraint_type);
        }
    }

    #[test]
    fn test_memory_impact_variants() {
        let impacts = vec![
            MemoryImpact::None,
            MemoryImpact::SizeIncrease(64),
            MemoryImpact::AlignmentChange(8),
            MemoryImpact::LayoutChange("Reordered".to_string()),
        ];

        for impact in impacts {
            let constraint = GenericConstraint {
                constraint_type: ConstraintType::Trait("Clone".to_string()),
                description: String::new(),
                memory_impact: impact.clone(),
            };
            assert_eq!(constraint.memory_impact, impact);
        }
    }

    #[test]
    fn test_type_parameter_creation() {
        let param = TypeParameter {
            name: "T".to_string(),
            concrete_type: "i32".to_string(),
            size: 4,
            alignment: 4,
            is_lifetime: false,
        };

        assert_eq!(param.name, "T");
        assert_eq!(param.concrete_type, "i32");
        assert!(!param.is_lifetime);
    }

    #[test]
    fn test_type_parameter_lifetime() {
        let param = TypeParameter {
            name: "'a".to_string(),
            concrete_type: "".to_string(),
            size: 0,
            alignment: 0,
            is_lifetime: true,
        };

        assert!(param.is_lifetime);
    }

    #[test]
    fn test_monomorphization_info_creation() {
        let info = MonomorphizationInfo {
            instance_count: 10,
            per_instance_memory: 24,
            total_memory_usage: 240,
            code_bloat_assessment: CodeBloatLevel::Moderate,
        };

        assert_eq!(info.instance_count, 10);
        assert_eq!(info.total_memory_usage, 240);
    }

    #[test]
    fn test_generic_constraint_creation() {
        let constraint = GenericConstraint {
            constraint_type: ConstraintType::Trait("Send".to_string()),
            description: "Type must be thread-safe".to_string(),
            memory_impact: MemoryImpact::None,
        };

        assert!(matches!(
            constraint.constraint_type,
            ConstraintType::Trait(_)
        ));
    }

    #[test]
    fn test_generic_type_info_with_parameters() {
        let info = GenericTypeInfo {
            base_type: "HashMap".to_string(),
            type_parameters: vec![
                TypeParameter {
                    name: "K".to_string(),
                    concrete_type: "String".to_string(),
                    size: 24,
                    alignment: 8,
                    is_lifetime: false,
                },
                TypeParameter {
                    name: "V".to_string(),
                    concrete_type: "i32".to_string(),
                    size: 4,
                    alignment: 4,
                    is_lifetime: false,
                },
            ],
            monomorphization_info: MonomorphizationInfo {
                instance_count: 1,
                per_instance_memory: 48,
                total_memory_usage: 48,
                code_bloat_assessment: CodeBloatLevel::Low,
            },
            constraints: vec![GenericConstraint {
                constraint_type: ConstraintType::Trait("Hash".to_string()),
                description: "Key must be hashable".to_string(),
                memory_impact: MemoryImpact::None,
            }],
        };

        assert_eq!(info.type_parameters.len(), 2);
        assert_eq!(info.constraints.len(), 1);
    }

    #[test]
    fn test_generic_instantiation_info_creation() {
        let info = GenericInstantiationInfo {
            base_type: "Vec".to_string(),
            concrete_parameters: vec![],
            instantiation_location: SourceLocation {
                file: "main.rs".to_string(),
                line: 10,
                column: 5,
            },
            instantiation_count: 5,
            memory_per_instance: 24,
            total_memory_usage: 120,
            compilation_impact: CompilationImpact {
                compilation_time_ms: 10,
                code_size_increase: 256,
                ir_complexity_score: 5,
                optimization_difficulty: OptimizationDifficulty::Easy,
            },
            performance_characteristics: PerformanceCharacteristics::default(),
        };

        assert_eq!(info.base_type, "Vec");
        assert_eq!(info.instantiation_count, 5);
    }

    #[test]
    fn test_concrete_type_parameter_creation() {
        let param = ConcreteTypeParameter {
            name: "T".to_string(),
            concrete_type: "String".to_string(),
            complexity_score: 3,
            memory_footprint: 24,
            alignment: 8,
            trait_implementations: vec!["Clone".to_string(), "Debug".to_string()],
            type_category: TypeCategory::Struct,
        };

        assert_eq!(param.name, "T");
        assert_eq!(param.trait_implementations.len(), 2);
    }

    #[test]
    fn test_type_category_all_variants() {
        let categories = vec![
            TypeCategory::Primitive,
            TypeCategory::Struct,
            TypeCategory::Enum,
            TypeCategory::Union,
            TypeCategory::Tuple,
            TypeCategory::Array,
            TypeCategory::Slice,
            TypeCategory::Reference,
            TypeCategory::Pointer,
            TypeCategory::Function,
            TypeCategory::TraitObject,
            TypeCategory::Generic,
            TypeCategory::Associated,
        ];

        for category in categories {
            let param = ConcreteTypeParameter {
                name: String::new(),
                concrete_type: String::new(),
                complexity_score: 0,
                memory_footprint: 0,
                alignment: 0,
                trait_implementations: vec![],
                type_category: category.clone(),
            };
            assert_eq!(param.type_category, category);
        }
    }

    #[test]
    fn test_source_location_creation() {
        let loc = SourceLocation {
            file: "src/main.rs".to_string(),
            line: 42,
            column: 10,
        };

        assert_eq!(loc.file, "src/main.rs");
        assert_eq!(loc.line, 42);
        assert_eq!(loc.column, 10);
    }

    #[test]
    fn test_compilation_impact_creation() {
        let impact = CompilationImpact {
            compilation_time_ms: 100,
            code_size_increase: 1024,
            ir_complexity_score: 50,
            optimization_difficulty: OptimizationDifficulty::Hard,
        };

        assert_eq!(impact.compilation_time_ms, 100);
        assert_eq!(impact.code_size_increase, 1024);
    }

    #[test]
    fn test_optimization_difficulty_variants() {
        let difficulties = vec![
            OptimizationDifficulty::Easy,
            OptimizationDifficulty::Moderate,
            OptimizationDifficulty::Hard,
            OptimizationDifficulty::VeryHard,
        ];

        for difficulty in difficulties {
            let impact = CompilationImpact {
                compilation_time_ms: 0,
                code_size_increase: 0,
                ir_complexity_score: 0,
                optimization_difficulty: difficulty.clone(),
            };
            assert_eq!(impact.optimization_difficulty, difficulty);
        }
    }

    #[test]
    fn test_performance_characteristics_creation() {
        let chars = PerformanceCharacteristics {
            avg_allocation_time_ns: 50.0,
            avg_deallocation_time_ns: 30.0,
            access_pattern: MemoryAccessPattern::Random,
            cache_impact: CacheImpact {
                l1_impact_score: 0.8,
                l2_impact_score: 0.6,
                l3_impact_score: 0.4,
                cache_line_efficiency: 0.75,
            },
            branch_prediction_impact: BranchPredictionImpact {
                misprediction_rate: 0.1,
                pipeline_stall_impact: 0.05,
                predictability_score: 0.9,
            },
        };

        assert!((chars.avg_allocation_time_ns - 50.0).abs() < f64::EPSILON);
        assert!((chars.cache_impact.l1_impact_score - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_impact_creation() {
        let impact = CacheImpact {
            l1_impact_score: 0.9,
            l2_impact_score: 0.7,
            l3_impact_score: 0.5,
            cache_line_efficiency: 0.85,
        };

        assert!((impact.l1_impact_score - 0.9).abs() < f64::EPSILON);
        assert!((impact.cache_line_efficiency - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_branch_prediction_impact_creation() {
        let impact = BranchPredictionImpact {
            misprediction_rate: 0.15,
            pipeline_stall_impact: 0.08,
            predictability_score: 0.92,
        };

        assert!((impact.misprediction_rate - 0.15).abs() < f64::EPSILON);
        assert!((impact.predictability_score - 0.92).abs() < f64::EPSILON);
    }

    #[test]
    fn test_generic_type_info_serialization() {
        let info = GenericTypeInfo {
            base_type: "Option".to_string(),
            type_parameters: vec![TypeParameter {
                name: "T".to_string(),
                concrete_type: "i32".to_string(),
                size: 4,
                alignment: 4,
                is_lifetime: false,
            }],
            monomorphization_info: MonomorphizationInfo {
                instance_count: 1,
                per_instance_memory: 4,
                total_memory_usage: 4,
                code_bloat_assessment: CodeBloatLevel::Low,
            },
            constraints: vec![],
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: GenericTypeInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.base_type, info.base_type);
    }

    #[test]
    fn test_memory_access_pattern_serialization() {
        let patterns = vec![
            MemoryAccessPattern::Sequential,
            MemoryAccessPattern::Random,
            MemoryAccessPattern::Strided { stride: 128 },
            MemoryAccessPattern::Clustered,
            MemoryAccessPattern::Mixed,
        ];

        for pattern in patterns {
            let json = serde_json::to_string(&pattern).unwrap();
            let deserialized: MemoryAccessPattern = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, pattern);
        }
    }

    #[test]
    fn test_code_bloat_level_serialization() {
        let levels = vec![
            CodeBloatLevel::Low,
            CodeBloatLevel::Moderate,
            CodeBloatLevel::High,
            CodeBloatLevel::Excessive,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let deserialized: CodeBloatLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, level);
        }
    }

    #[test]
    fn test_constraint_type_serialization() {
        let constraint_types = vec![
            ConstraintType::Trait("Clone".to_string()),
            ConstraintType::Lifetime("'static".to_string()),
            ConstraintType::Associated("Item".to_string()),
            ConstraintType::Where("T: Send".to_string()),
        ];

        for ct in constraint_types {
            let json = serde_json::to_string(&ct).unwrap();
            let deserialized: ConstraintType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, ct);
        }
    }

    #[test]
    fn test_memory_impact_serialization() {
        let impacts = vec![
            MemoryImpact::None,
            MemoryImpact::SizeIncrease(64),
            MemoryImpact::AlignmentChange(8),
            MemoryImpact::LayoutChange("Reordered".to_string()),
        ];

        for impact in impacts {
            let json = serde_json::to_string(&impact).unwrap();
            let deserialized: MemoryImpact = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, impact);
        }
    }

    #[test]
    fn test_type_category_serialization() {
        let categories = vec![
            TypeCategory::Primitive,
            TypeCategory::Struct,
            TypeCategory::Enum,
            TypeCategory::Union,
            TypeCategory::Tuple,
            TypeCategory::Array,
            TypeCategory::Slice,
            TypeCategory::Reference,
            TypeCategory::Pointer,
            TypeCategory::Function,
            TypeCategory::TraitObject,
            TypeCategory::Generic,
            TypeCategory::Associated,
        ];

        for category in categories {
            let json = serde_json::to_string(&category).unwrap();
            let deserialized: TypeCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, category);
        }
    }

    #[test]
    fn test_optimization_difficulty_serialization() {
        let difficulties = vec![
            OptimizationDifficulty::Easy,
            OptimizationDifficulty::Moderate,
            OptimizationDifficulty::Hard,
            OptimizationDifficulty::VeryHard,
        ];

        for difficulty in difficulties {
            let json = serde_json::to_string(&difficulty).unwrap();
            let deserialized: OptimizationDifficulty = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, difficulty);
        }
    }

    #[test]
    fn test_generic_instantiation_info_serialization() {
        let info = GenericInstantiationInfo {
            base_type: "Vec".to_string(),
            concrete_parameters: vec![ConcreteTypeParameter {
                name: "T".to_string(),
                concrete_type: "u8".to_string(),
                complexity_score: 1,
                memory_footprint: 1,
                alignment: 1,
                trait_implementations: vec![],
                type_category: TypeCategory::Primitive,
            }],
            instantiation_location: SourceLocation {
                file: "lib.rs".to_string(),
                line: 1,
                column: 1,
            },
            instantiation_count: 1,
            memory_per_instance: 24,
            total_memory_usage: 24,
            compilation_impact: CompilationImpact {
                compilation_time_ms: 5,
                code_size_increase: 64,
                ir_complexity_score: 1,
                optimization_difficulty: OptimizationDifficulty::Easy,
            },
            performance_characteristics: PerformanceCharacteristics::default(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: GenericInstantiationInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.base_type, info.base_type);
    }

    #[test]
    fn test_generic_type_info_clone() {
        let info = GenericTypeInfo {
            base_type: "Box".to_string(),
            type_parameters: vec![],
            monomorphization_info: MonomorphizationInfo {
                instance_count: 1,
                per_instance_memory: 8,
                total_memory_usage: 8,
                code_bloat_assessment: CodeBloatLevel::Low,
            },
            constraints: vec![],
        };

        let cloned = info.clone();
        assert_eq!(cloned.base_type, info.base_type);
    }

    #[test]
    fn test_generic_type_info_debug() {
        let info = GenericTypeInfo {
            base_type: "Rc".to_string(),
            type_parameters: vec![],
            monomorphization_info: MonomorphizationInfo {
                instance_count: 0,
                per_instance_memory: 0,
                total_memory_usage: 0,
                code_bloat_assessment: CodeBloatLevel::Low,
            },
            constraints: vec![],
        };

        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("GenericTypeInfo"));
        assert!(debug_str.contains("base_type"));
    }

    #[test]
    fn test_boundary_values_monomorphization() {
        let info = MonomorphizationInfo {
            instance_count: usize::MAX,
            per_instance_memory: usize::MAX,
            total_memory_usage: usize::MAX,
            code_bloat_assessment: CodeBloatLevel::Excessive,
        };

        assert_eq!(info.instance_count, usize::MAX);
        assert_eq!(info.total_memory_usage, usize::MAX);
    }

    #[test]
    fn test_boundary_values_compilation_impact() {
        let impact = CompilationImpact {
            compilation_time_ms: u64::MAX,
            code_size_increase: usize::MAX,
            ir_complexity_score: u32::MAX,
            optimization_difficulty: OptimizationDifficulty::VeryHard,
        };

        assert_eq!(impact.compilation_time_ms, u64::MAX);
        assert_eq!(impact.ir_complexity_score, u32::MAX);
    }

    #[test]
    fn test_boundary_values_performance_chars() {
        let chars = PerformanceCharacteristics {
            avg_allocation_time_ns: f64::MAX,
            avg_deallocation_time_ns: f64::MAX,
            access_pattern: MemoryAccessPattern::Mixed,
            cache_impact: CacheImpact {
                l1_impact_score: f64::MAX,
                l2_impact_score: f64::MAX,
                l3_impact_score: f64::MAX,
                cache_line_efficiency: 1.0,
            },
            branch_prediction_impact: BranchPredictionImpact {
                misprediction_rate: 1.0,
                pipeline_stall_impact: 1.0,
                predictability_score: 1.0,
            },
        };

        assert!(chars.avg_allocation_time_ns.is_finite());
    }
}
