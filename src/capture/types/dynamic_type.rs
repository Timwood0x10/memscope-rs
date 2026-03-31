//! Dynamic type tracking types.
//!
//! This module contains types for tracking dynamic types (trait objects),
//! virtual function tables, and type erasure.

use serde::{Deserialize, Serialize};

use super::generic::SourceLocation;

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
}
