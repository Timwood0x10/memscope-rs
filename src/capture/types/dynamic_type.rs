//! Dynamic type tracking types.
//!
//! This module contains types for tracking dynamic types (trait objects),
//! virtual function tables, and type erasure.

use serde::{Deserialize, Serialize};

/// Dynamic type information (trait objects).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicTypeInfo {
    /// Trait name.
    pub trait_name: String,
    /// Virtual function table information.
    pub vtable_info: VTableInfo,
    /// Concrete object type (if determinable).
    pub concrete_type: Option<String>,
    /// Dynamic dispatch overhead.
    pub dispatch_overhead: DispatchOverhead,
    /// Type erasure information.
    pub type_erasure_info: TypeErasureInfo,
}

/// Virtual function table information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VTableInfo {
    /// VTable size.
    pub vtable_size: usize,
    /// Number of methods.
    pub method_count: usize,
    /// VTable pointer offset.
    pub vtable_ptr_offset: usize,
    /// Method list.
    pub methods: Vec<VTableMethod>,
}

/// Virtual function table method.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VTableMethod {
    /// Method name.
    pub name: String,
    /// Method signature.
    pub signature: String,
    /// Offset in vtable.
    pub vtable_offset: usize,
}

/// Dynamic dispatch overhead.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DispatchOverhead {
    /// Indirect call overhead in nanoseconds.
    pub indirect_call_overhead_ns: f64,
    /// Cache miss probability.
    pub cache_miss_probability: f64,
    /// Branch misprediction rate.
    pub branch_misprediction_rate: f64,
    /// Overall performance impact assessment.
    pub performance_impact: PerformanceImpact,
}

/// Performance impact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PerformanceImpact {
    /// Negligible performance impact.
    Negligible,
    /// Minor performance impact.
    Minor,
    /// Moderate performance impact.
    Moderate,
    /// Significant performance impact.
    Significant,
    /// Severe performance impact.
    Severe,
}

/// Type erasure information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeErasureInfo {
    /// Whether original type information is recoverable.
    pub type_info_recoverable: bool,
    /// Type size information.
    pub size_info: Option<usize>,
    /// Alignment information.
    pub alignment_info: Option<usize>,
    /// Destructor information.
    pub destructor_info: Option<String>,
}

// Implement From trait for converting from core::types to capture::types
impl From<crate::core::types::DynamicTypeInfo> for DynamicTypeInfo {
    fn from(old: crate::core::types::DynamicTypeInfo) -> Self {
        Self {
            trait_name: old.trait_name,
            vtable_info: VTableInfo {
                vtable_size: old.vtable_info.vtable_size,
                method_count: old.vtable_info.method_count,
                vtable_ptr_offset: old.vtable_info.vtable_ptr_offset,
                methods: old
                    .vtable_info
                    .methods
                    .into_iter()
                    .map(|m| VTableMethod {
                        name: m.name,
                        signature: m.signature,
                        vtable_offset: m.vtable_offset,
                    })
                    .collect(),
            },
            concrete_type: old.concrete_type,
            dispatch_overhead: DispatchOverhead {
                indirect_call_overhead_ns: old.dispatch_overhead.indirect_call_overhead_ns,
                cache_miss_probability: old.dispatch_overhead.cache_miss_probability,
                branch_misprediction_rate: old.dispatch_overhead.branch_misprediction_rate,
                performance_impact: match old.dispatch_overhead.performance_impact {
                    crate::core::types::PerformanceImpact::Negligible => {
                        PerformanceImpact::Negligible
                    }
                    crate::core::types::PerformanceImpact::Minor => PerformanceImpact::Minor,
                    crate::core::types::PerformanceImpact::Moderate => PerformanceImpact::Moderate,
                    crate::core::types::PerformanceImpact::Significant => {
                        PerformanceImpact::Significant
                    }
                    crate::core::types::PerformanceImpact::Severe => PerformanceImpact::Severe,
                },
            },
            type_erasure_info: TypeErasureInfo {
                type_info_recoverable: old.type_erasure_info.type_info_recoverable,
                size_info: old.type_erasure_info.size_info,
                alignment_info: old.type_erasure_info.alignment_info,
                destructor_info: old.type_erasure_info.destructor_info,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_type_info() {
        let info = DynamicTypeInfo {
            trait_name: "Display".to_string(),
            vtable_info: VTableInfo {
                vtable_size: 32,
                method_count: 1,
                vtable_ptr_offset: 0,
                methods: vec![],
            },
            concrete_type: Some("String".to_string()),
            dispatch_overhead: DispatchOverhead {
                indirect_call_overhead_ns: 5.0,
                cache_miss_probability: 0.1,
                branch_misprediction_rate: 0.05,
                performance_impact: PerformanceImpact::Minor,
            },
            type_erasure_info: TypeErasureInfo {
                type_info_recoverable: true,
                size_info: Some(24),
                alignment_info: Some(8),
                destructor_info: None,
            },
        };

        assert_eq!(info.trait_name, "Display");
        assert!(info.concrete_type.is_some());
    }

    #[test]
    fn test_vtable_info() {
        let vtable = VTableInfo {
            vtable_size: 64,
            method_count: 3,
            vtable_ptr_offset: 0,
            methods: vec![VTableMethod {
                name: "drop".to_string(),
                signature: "fn(&mut self)".to_string(),
                vtable_offset: 0,
            }],
        };

        assert_eq!(vtable.method_count, 3);
        assert_eq!(vtable.methods.len(), 1);
    }

    #[test]
    fn test_vtable_method_creation() {
        let method = VTableMethod {
            name: "fmt".to_string(),
            signature: "fn(&self, &mut Formatter) -> Result".to_string(),
            vtable_offset: 8,
        };

        assert_eq!(method.name, "fmt");
        assert_eq!(method.vtable_offset, 8);
    }

    #[test]
    fn test_vtable_info_multiple_methods() {
        let vtable = VTableInfo {
            vtable_size: 128,
            method_count: 5,
            vtable_ptr_offset: 0,
            methods: vec![
                VTableMethod {
                    name: "drop".to_string(),
                    signature: "fn(&mut self)".to_string(),
                    vtable_offset: 0,
                },
                VTableMethod {
                    name: "clone".to_string(),
                    signature: "fn(&self) -> Self".to_string(),
                    vtable_offset: 8,
                },
                VTableMethod {
                    name: "eq".to_string(),
                    signature: "fn(&self, &Self) -> bool".to_string(),
                    vtable_offset: 16,
                },
            ],
        };

        assert_eq!(vtable.methods.len(), 3);
        assert_eq!(vtable.methods[1].name, "clone");
    }

    #[test]
    fn test_dispatch_overhead_creation() {
        let overhead = DispatchOverhead {
            indirect_call_overhead_ns: 10.0,
            cache_miss_probability: 0.15,
            branch_misprediction_rate: 0.08,
            performance_impact: PerformanceImpact::Moderate,
        };

        assert!((overhead.indirect_call_overhead_ns - 10.0).abs() < f64::EPSILON);
        assert!((overhead.cache_miss_probability - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn test_performance_impact_variants() {
        let impacts = vec![
            PerformanceImpact::Negligible,
            PerformanceImpact::Minor,
            PerformanceImpact::Moderate,
            PerformanceImpact::Significant,
            PerformanceImpact::Severe,
        ];

        for impact in impacts {
            let overhead = DispatchOverhead {
                indirect_call_overhead_ns: 0.0,
                cache_miss_probability: 0.0,
                branch_misprediction_rate: 0.0,
                performance_impact: impact.clone(),
            };
            assert_eq!(overhead.performance_impact, impact);
        }
    }

    #[test]
    fn test_type_erasure_info_creation() {
        let info = TypeErasureInfo {
            type_info_recoverable: false,
            size_info: None,
            alignment_info: None,
            destructor_info: Some("custom_drop".to_string()),
        };

        assert!(!info.type_info_recoverable);
        assert!(info.size_info.is_none());
        assert!(info.destructor_info.is_some());
    }

    #[test]
    fn test_type_erasure_info_full() {
        let info = TypeErasureInfo {
            type_info_recoverable: true,
            size_info: Some(32),
            alignment_info: Some(8),
            destructor_info: Some("drop_in_place".to_string()),
        };

        assert!(info.type_info_recoverable);
        assert_eq!(info.size_info, Some(32));
        assert_eq!(info.alignment_info, Some(8));
    }

    #[test]
    fn test_dynamic_type_info_no_concrete_type() {
        let info = DynamicTypeInfo {
            trait_name: "Iterator".to_string(),
            vtable_info: VTableInfo {
                vtable_size: 48,
                method_count: 2,
                vtable_ptr_offset: 0,
                methods: vec![],
            },
            concrete_type: None,
            dispatch_overhead: DispatchOverhead {
                indirect_call_overhead_ns: 15.0,
                cache_miss_probability: 0.25,
                branch_misprediction_rate: 0.12,
                performance_impact: PerformanceImpact::Significant,
            },
            type_erasure_info: TypeErasureInfo {
                type_info_recoverable: false,
                size_info: None,
                alignment_info: None,
                destructor_info: None,
            },
        };

        assert!(info.concrete_type.is_none());
        assert!(matches!(
            info.dispatch_overhead.performance_impact,
            PerformanceImpact::Significant
        ));
    }

    #[test]
    fn test_dynamic_type_info_serialization() {
        let info = DynamicTypeInfo {
            trait_name: "Debug".to_string(),
            vtable_info: VTableInfo {
                vtable_size: 16,
                method_count: 1,
                vtable_ptr_offset: 0,
                methods: vec![],
            },
            concrete_type: Some("i32".to_string()),
            dispatch_overhead: DispatchOverhead {
                indirect_call_overhead_ns: 2.0,
                cache_miss_probability: 0.01,
                branch_misprediction_rate: 0.005,
                performance_impact: PerformanceImpact::Negligible,
            },
            type_erasure_info: TypeErasureInfo {
                type_info_recoverable: true,
                size_info: Some(4),
                alignment_info: Some(4),
                destructor_info: None,
            },
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: DynamicTypeInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.trait_name, info.trait_name);
    }

    #[test]
    fn test_vtable_info_serialization() {
        let vtable = VTableInfo {
            vtable_size: 64,
            method_count: 2,
            vtable_ptr_offset: 8,
            methods: vec![VTableMethod {
                name: "test".to_string(),
                signature: "fn()".to_string(),
                vtable_offset: 0,
            }],
        };

        let json = serde_json::to_string(&vtable).unwrap();
        let deserialized: VTableInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.vtable_size, vtable.vtable_size);
    }

    #[test]
    fn test_performance_impact_serialization() {
        let impacts = vec![
            PerformanceImpact::Negligible,
            PerformanceImpact::Minor,
            PerformanceImpact::Moderate,
            PerformanceImpact::Significant,
            PerformanceImpact::Severe,
        ];

        for impact in impacts {
            let json = serde_json::to_string(&impact).unwrap();
            let deserialized: PerformanceImpact = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, impact);
        }
    }

    #[test]
    fn test_dispatch_overhead_serialization() {
        let overhead = DispatchOverhead {
            indirect_call_overhead_ns: 5.5,
            cache_miss_probability: 0.1,
            branch_misprediction_rate: 0.05,
            performance_impact: PerformanceImpact::Minor,
        };

        let json = serde_json::to_string(&overhead).unwrap();
        let deserialized: DispatchOverhead = serde_json::from_str(&json).unwrap();
        assert!(
            (deserialized.indirect_call_overhead_ns - overhead.indirect_call_overhead_ns).abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn test_type_erasure_info_serialization() {
        let info = TypeErasureInfo {
            type_info_recoverable: true,
            size_info: Some(16),
            alignment_info: Some(8),
            destructor_info: Some("drop".to_string()),
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: TypeErasureInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.type_info_recoverable,
            info.type_info_recoverable
        );
    }

    #[test]
    fn test_vtable_method_serialization() {
        let method = VTableMethod {
            name: "execute".to_string(),
            signature: "fn(&mut self, Args) -> Result".to_string(),
            vtable_offset: 24,
        };

        let json = serde_json::to_string(&method).unwrap();
        let deserialized: VTableMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, method.name);
    }

    #[test]
    fn test_dynamic_type_info_clone() {
        let info = DynamicTypeInfo {
            trait_name: "Clone".to_string(),
            vtable_info: VTableInfo {
                vtable_size: 8,
                method_count: 1,
                vtable_ptr_offset: 0,
                methods: vec![],
            },
            concrete_type: None,
            dispatch_overhead: DispatchOverhead {
                indirect_call_overhead_ns: 0.0,
                cache_miss_probability: 0.0,
                branch_misprediction_rate: 0.0,
                performance_impact: PerformanceImpact::Negligible,
            },
            type_erasure_info: TypeErasureInfo {
                type_info_recoverable: true,
                size_info: None,
                alignment_info: None,
                destructor_info: None,
            },
        };

        let cloned = info.clone();
        assert_eq!(cloned.trait_name, info.trait_name);
    }

    #[test]
    fn test_dynamic_type_info_debug() {
        let info = DynamicTypeInfo {
            trait_name: "Debug".to_string(),
            vtable_info: VTableInfo {
                vtable_size: 8,
                method_count: 0,
                vtable_ptr_offset: 0,
                methods: vec![],
            },
            concrete_type: None,
            dispatch_overhead: DispatchOverhead {
                indirect_call_overhead_ns: 0.0,
                cache_miss_probability: 0.0,
                branch_misprediction_rate: 0.0,
                performance_impact: PerformanceImpact::Negligible,
            },
            type_erasure_info: TypeErasureInfo {
                type_info_recoverable: false,
                size_info: None,
                alignment_info: None,
                destructor_info: None,
            },
        };

        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("DynamicTypeInfo"));
        assert!(debug_str.contains("trait_name"));
    }

    #[test]
    fn test_vtable_info_equality() {
        let vtable1 = VTableInfo {
            vtable_size: 32,
            method_count: 2,
            vtable_ptr_offset: 0,
            methods: vec![],
        };
        let vtable2 = VTableInfo {
            vtable_size: 32,
            method_count: 2,
            vtable_ptr_offset: 0,
            methods: vec![],
        };

        assert_eq!(vtable1, vtable2);
    }

    #[test]
    fn test_boundary_values_vtable() {
        let vtable = VTableInfo {
            vtable_size: usize::MAX,
            method_count: usize::MAX,
            vtable_ptr_offset: usize::MAX,
            methods: vec![],
        };

        assert_eq!(vtable.vtable_size, usize::MAX);
        assert_eq!(vtable.method_count, usize::MAX);
    }

    #[test]
    fn test_boundary_values_dispatch_overhead() {
        let overhead = DispatchOverhead {
            indirect_call_overhead_ns: f64::MAX,
            cache_miss_probability: 1.0,
            branch_misprediction_rate: 1.0,
            performance_impact: PerformanceImpact::Severe,
        };

        assert!(overhead.indirect_call_overhead_ns.is_finite());
        assert!((overhead.cache_miss_probability - 1.0).abs() < f64::EPSILON);
    }
}
