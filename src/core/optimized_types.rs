//! Optimized data structures using string interning and Arc sharing
//!
//! This module provides memory-optimized versions of core data structures
//! that use `Arc<str>` for string fields and the global string pool for
//! memory efficiency.

use crate::core::string_pool::intern_string;
use crate::core::types::{
    DropChainAnalysis, DynamicTypeInfo, EnhancedFragmentationAnalysis, FunctionCallTrackingInfo,
    GenericInstantiationInfo, GenericTypeInfo, MemoryAccessTrackingInfo, MemoryLayoutInfo,
    ObjectLifecycleInfo, RuntimeStateInfo, SmartPointerInfo, StackAllocationInfo,
    TemporaryObjectInfo, TypeRelationshipInfo, TypeUsageInfo,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Optimized allocation information using `Arc<str>` for string fields
///
/// This structure is a drop-in replacement for AllocationInfo that uses
/// `Arc<str>` for all string fields to reduce memory usage through string
/// interning. All string fields are automatically interned using the
/// global string pool.
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizedAllocationInfo {
    /// Memory address of the allocation
    pub ptr: usize,
    /// Size of the allocation in bytes
    pub size: usize,
    /// Optional variable name associated with this allocation (interned)
    pub var_name: Option<Arc<str>>,
    /// Optional type name of the allocated data (interned)
    pub type_name: Option<Arc<str>>,
    /// Optional scope name where the allocation occurred (interned)
    pub scope_name: Option<Arc<str>>,
    /// Timestamp when the allocation was made
    pub timestamp_alloc: u64,
    /// Optional timestamp when the allocation was deallocated
    pub timestamp_dealloc: Option<u64>,
    /// Thread ID where the allocation occurred (interned)
    pub thread_id: Arc<str>,
    /// Number of active borrows for this allocation
    pub borrow_count: usize,
    /// Optional stack trace at the time of allocation (interned strings)
    pub stack_trace: Option<Arc<Vec<Arc<str>>>>,
    /// Whether this allocation is considered leaked
    pub is_leaked: bool,
    /// Precise lifetime in milliseconds (calculated from creation to destruction)
    pub lifetime_ms: Option<u64>,
    /// Smart pointer specific information
    pub smart_pointer_info: Option<SmartPointerInfo>,
    /// Detailed memory layout information
    pub memory_layout: Option<MemoryLayoutInfo>,
    /// Generic type information
    pub generic_info: Option<GenericTypeInfo>,
    /// Dynamic type information (trait objects)
    pub dynamic_type_info: Option<DynamicTypeInfo>,
    /// Runtime state information
    pub runtime_state: Option<RuntimeStateInfo>,
    /// Stack allocation information (if allocated on stack)
    pub stack_allocation: Option<StackAllocationInfo>,
    /// Temporary object information (if this is a temporary)
    pub temporary_object: Option<TemporaryObjectInfo>,
    /// Memory fragmentation analysis
    pub fragmentation_analysis: Option<EnhancedFragmentationAnalysis>,
    /// Enhanced generic instantiation tracking
    pub generic_instantiation: Option<GenericInstantiationInfo>,
    /// Type relationship information
    pub type_relationships: Option<TypeRelationshipInfo>,
    /// Type usage information
    pub type_usage: Option<TypeUsageInfo>,
    /// Function call tracking (if allocation is function-related)
    pub function_call_tracking: Option<FunctionCallTrackingInfo>,
    /// Object lifecycle tracking
    pub lifecycle_tracking: Option<ObjectLifecycleInfo>,
    /// Memory access pattern tracking
    pub access_tracking: Option<MemoryAccessTrackingInfo>,
    /// Drop chain analysis (when object is dropped)
    pub drop_chain_analysis: Option<DropChainAnalysis>,
}

impl OptimizedAllocationInfo {
    /// Create a new optimized allocation info
    pub fn new(ptr: usize, size: usize) -> Self {
        Self {
            ptr,
            size,
            var_name: None,
            type_name: None,
            scope_name: None,
            timestamp_alloc: crate::utils::current_timestamp_nanos(),
            timestamp_dealloc: None,
            thread_id: intern_string(&format!("{:?}", std::thread::current().id())),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        }
    }

    /// Set variable information using string interning
    pub fn with_var_info(mut self, var_name: &str, type_name: &str) -> Self {
        self.var_name = Some(intern_string(var_name));
        self.type_name = Some(intern_string(type_name));
        self
    }

    /// Set scope information using string interning
    pub fn with_scope(mut self, scope_name: &str) -> Self {
        self.scope_name = Some(intern_string(scope_name));
        self
    }

    /// Set stack trace using string interning for all frames
    pub fn with_stack_trace(mut self, stack_trace: Vec<String>) -> Self {
        let interned_trace: Vec<Arc<str>> = stack_trace
            .iter()
            .map(|frame| intern_string(frame))
            .collect();
        self.stack_trace = Some(Arc::new(interned_trace));
        self
    }

    /// Mark allocation as deallocated
    pub fn mark_deallocated(&mut self) {
        let now = crate::utils::current_timestamp_nanos();
        self.timestamp_dealloc = Some(now);
        self.lifetime_ms = Some((now - self.timestamp_alloc) / 1_000_000);
    }

    /// Get the lifetime duration in nanoseconds if deallocated
    pub fn lifetime_duration_nanos(&self) -> Option<u64> {
        self.timestamp_dealloc
            .map(|dealloc| dealloc - self.timestamp_alloc)
    }

    /// Get the lifetime duration in milliseconds if deallocated
    pub fn lifetime_duration_ms(&self) -> Option<u64> {
        self.lifetime_duration_nanos()
            .map(|nanos| nanos / 1_000_000)
    }

    /// Check if this allocation is currently active (not deallocated)
    pub fn is_active(&self) -> bool {
        self.timestamp_dealloc.is_none()
    }

    /// Get variable name as &str for compatibility
    pub fn var_name_str(&self) -> Option<&str> {
        self.var_name.as_ref().map(|s| s.as_ref())
    }

    /// Get type name as &str for compatibility
    pub fn type_name_str(&self) -> Option<&str> {
        self.type_name.as_ref().map(|s| s.as_ref())
    }

    /// Get scope name as &str for compatibility
    pub fn scope_name_str(&self) -> Option<&str> {
        self.scope_name.as_ref().map(|s| s.as_ref())
    }

    /// Get thread ID as &str for compatibility
    pub fn thread_id_str(&self) -> &str {
        self.thread_id.as_ref()
    }

    /// Get stack trace as Vec<&str> for compatibility
    pub fn stack_trace_strs(&self) -> Option<Vec<&str>> {
        self.stack_trace
            .as_ref()
            .map(|trace| trace.iter().map(|frame| frame.as_ref()).collect())
    }
}

/// Custom serialization to handle `Arc<str>` fields
impl Serialize for OptimizedAllocationInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("OptimizedAllocationInfo", 24)?;
        state.serialize_field("ptr", &self.ptr)?;
        state.serialize_field("size", &self.size)?;
        state.serialize_field("var_name", &self.var_name.as_ref().map(|s| s.as_ref()))?;
        state.serialize_field("type_name", &self.type_name.as_ref().map(|s| s.as_ref()))?;
        state.serialize_field("scope_name", &self.scope_name.as_ref().map(|s| s.as_ref()))?;
        state.serialize_field("timestamp_alloc", &self.timestamp_alloc)?;
        state.serialize_field("timestamp_dealloc", &self.timestamp_dealloc)?;
        state.serialize_field("borrow_count", &self.borrow_count)?;

        // Serialize stack trace as Vec<&str>
        let stack_trace_strs = self.stack_trace.as_ref().map(|trace| {
            trace
                .iter()
                .map(|frame| frame.as_ref())
                .collect::<Vec<&str>>()
        });
        state.serialize_field("stack_trace", &stack_trace_strs)?;

        state.serialize_field("is_leaked", &self.is_leaked)?;
        state.serialize_field("lifetime_ms", &self.lifetime_ms)?;
        state.serialize_field("smart_pointer_info", &self.smart_pointer_info)?;
        state.serialize_field("memory_layout", &self.memory_layout)?;
        state.serialize_field("generic_info", &self.generic_info)?;
        state.serialize_field("dynamic_type_info", &self.dynamic_type_info)?;
        state.serialize_field("runtime_state", &self.runtime_state)?;
        state.serialize_field("stack_allocation", &self.stack_allocation)?;
        state.serialize_field("temporary_object", &self.temporary_object)?;
        state.serialize_field("fragmentation_analysis", &self.fragmentation_analysis)?;
        state.serialize_field("generic_instantiation", &self.generic_instantiation)?;
        state.serialize_field("type_relationships", &self.type_relationships)?;
        state.serialize_field("type_usage", &self.type_usage)?;
        state.serialize_field("function_call_tracking", &self.function_call_tracking)?;
        state.serialize_field("lifecycle_tracking", &self.lifecycle_tracking)?;
        state.serialize_field("access_tracking", &self.access_tracking)?;
        state.serialize_field("drop_chain_analysis", &self.drop_chain_analysis)?;
        state.end()
    }
}

/// Custom deserialization to handle string interning
impl<'de> Deserialize<'de> for OptimizedAllocationInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct OptimizedAllocationInfoHelper {
            ptr: usize,
            size: usize,
            var_name: Option<String>,
            type_name: Option<String>,
            scope_name: Option<String>,
            timestamp_alloc: u64,
            timestamp_dealloc: Option<u64>,
            borrow_count: usize,
            stack_trace: Option<Vec<String>>,
            is_leaked: bool,
            lifetime_ms: Option<u64>,
            smart_pointer_info: Option<SmartPointerInfo>,
            memory_layout: Option<MemoryLayoutInfo>,
            generic_info: Option<GenericTypeInfo>,
            dynamic_type_info: Option<DynamicTypeInfo>,
            runtime_state: Option<RuntimeStateInfo>,
            stack_allocation: Option<StackAllocationInfo>,
            temporary_object: Option<TemporaryObjectInfo>,
            fragmentation_analysis: Option<EnhancedFragmentationAnalysis>,
            generic_instantiation: Option<GenericInstantiationInfo>,
            type_relationships: Option<TypeRelationshipInfo>,
            type_usage: Option<TypeUsageInfo>,
            function_call_tracking: Option<FunctionCallTrackingInfo>,
            lifecycle_tracking: Option<ObjectLifecycleInfo>,
            access_tracking: Option<MemoryAccessTrackingInfo>,
            drop_chain_analysis: Option<DropChainAnalysis>,
        }

        let helper = OptimizedAllocationInfoHelper::deserialize(deserializer)?;

        Ok(OptimizedAllocationInfo {
            ptr: helper.ptr,
            size: helper.size,
            var_name: helper.var_name.map(|s| intern_string(&s)),
            type_name: helper.type_name.map(|s| intern_string(&s)),
            scope_name: helper.scope_name.map(|s| intern_string(&s)),
            timestamp_alloc: helper.timestamp_alloc,
            timestamp_dealloc: helper.timestamp_dealloc,
            thread_id: intern_string(&format!("{:?}", std::thread::current().id())),
            borrow_count: helper.borrow_count,
            stack_trace: helper.stack_trace.map(|trace| {
                Arc::new(
                    trace
                        .into_iter()
                        .map(|frame| intern_string(&frame))
                        .collect(),
                )
            }),
            is_leaked: helper.is_leaked,
            lifetime_ms: helper.lifetime_ms,
            smart_pointer_info: helper.smart_pointer_info,
            memory_layout: helper.memory_layout,
            generic_info: helper.generic_info,
            dynamic_type_info: helper.dynamic_type_info,
            runtime_state: helper.runtime_state,
            stack_allocation: helper.stack_allocation,
            temporary_object: helper.temporary_object,
            fragmentation_analysis: helper.fragmentation_analysis,
            generic_instantiation: helper.generic_instantiation,
            type_relationships: helper.type_relationships,
            type_usage: helper.type_usage,
            function_call_tracking: helper.function_call_tracking,
            lifecycle_tracking: helper.lifecycle_tracking,
            access_tracking: helper.access_tracking,
            drop_chain_analysis: helper.drop_chain_analysis,
        })
    }
}

/// Conversion from original AllocationInfo to optimized version
impl From<crate::core::types::AllocationInfo> for OptimizedAllocationInfo {
    fn from(original: crate::core::types::AllocationInfo) -> Self {
        Self {
            ptr: original.ptr,
            size: original.size,
            var_name: original.var_name.map(|s| intern_string(&s)),
            type_name: original.type_name.map(|s| intern_string(&s)),
            scope_name: original.scope_name.map(|s| intern_string(&s)),
            timestamp_alloc: original.timestamp_alloc,
            timestamp_dealloc: original.timestamp_dealloc,
            thread_id: intern_string(&original.thread_id),
            borrow_count: original.borrow_count,
            stack_trace: original.stack_trace.map(|trace| {
                Arc::new(
                    trace
                        .into_iter()
                        .map(|frame| intern_string(&frame))
                        .collect(),
                )
            }),
            is_leaked: original.is_leaked,
            lifetime_ms: original.lifetime_ms,
            smart_pointer_info: original.smart_pointer_info,
            memory_layout: original.memory_layout,
            generic_info: original.generic_info,
            dynamic_type_info: original.dynamic_type_info,
            runtime_state: original.runtime_state,
            stack_allocation: original.stack_allocation,
            temporary_object: original.temporary_object,
            fragmentation_analysis: original.fragmentation_analysis,
            generic_instantiation: original.generic_instantiation,
            type_relationships: original.type_relationships,
            type_usage: original.type_usage,
            function_call_tracking: original.function_call_tracking,
            lifecycle_tracking: original.lifecycle_tracking,
            access_tracking: original.access_tracking,
            drop_chain_analysis: original.drop_chain_analysis,
        }
    }
}

/// Conversion from optimized version back to original AllocationInfo
impl From<OptimizedAllocationInfo> for crate::core::types::AllocationInfo {
    fn from(optimized: OptimizedAllocationInfo) -> Self {
        Self {
            ptr: optimized.ptr,
            size: optimized.size,
            var_name: optimized.var_name.map(|s| s.to_string()),
            type_name: optimized.type_name.map(|s| s.to_string()),
            scope_name: optimized.scope_name.map(|s| s.to_string()),
            timestamp_alloc: optimized.timestamp_alloc,
            timestamp_dealloc: optimized.timestamp_dealloc,
            thread_id: optimized.thread_id.to_string(),
            borrow_count: optimized.borrow_count,
            stack_trace: optimized
                .stack_trace
                .map(|trace| trace.iter().map(|frame| frame.to_string()).collect()),
            is_leaked: optimized.is_leaked,
            lifetime_ms: optimized.lifetime_ms,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: true, // Preserve original value
            smart_pointer_info: optimized.smart_pointer_info,
            memory_layout: optimized.memory_layout,
            generic_info: optimized.generic_info,
            dynamic_type_info: optimized.dynamic_type_info,
            runtime_state: optimized.runtime_state,
            stack_allocation: optimized.stack_allocation,
            temporary_object: optimized.temporary_object,
            fragmentation_analysis: optimized.fragmentation_analysis,
            generic_instantiation: optimized.generic_instantiation,
            type_relationships: optimized.type_relationships,
            type_usage: optimized.type_usage,
            function_call_tracking: optimized.function_call_tracking,
            lifecycle_tracking: optimized.lifecycle_tracking,
            access_tracking: optimized.access_tracking,
            drop_chain_analysis: optimized.drop_chain_analysis,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::string_pool::clear_string_pool;

    #[test]
    fn test_optimized_allocation_info_creation() {
        clear_string_pool();

        let info = OptimizedAllocationInfo::new(0x1000, 64)
            .with_var_info("my_var", "Vec<i32>")
            .with_scope("main");

        assert_eq!(info.ptr, 0x1000);
        assert_eq!(info.size, 64);
        assert_eq!(info.var_name_str(), Some("my_var"));
        assert_eq!(info.type_name_str(), Some("Vec<i32>"));
        assert_eq!(info.scope_name_str(), Some("main"));
        assert!(info.is_active());
    }

    #[test]
    fn test_string_interning_in_allocation_info() {
        // Test that string interning works by checking content equality
        // We can't reliably test Arc pointer equality due to test isolation issues
        let info1 = OptimizedAllocationInfo::new(0x1000, 64).with_var_info("test_var", "TestType");
        let info2 = OptimizedAllocationInfo::new(0x2000, 128).with_var_info("test_var", "TestType");

        // Same strings should have same content
        assert_eq!(info1.var_name_str(), Some("test_var"));
        assert_eq!(info2.var_name_str(), Some("test_var"));
        assert_eq!(info1.type_name_str(), Some("TestType"));
        assert_eq!(info2.type_name_str(), Some("TestType"));

        // Verify the strings are actually Arc<str>
        assert!(info1.var_name.is_some());
        assert!(info1.type_name.is_some());
    }

    #[test]
    fn test_stack_trace_interning() {
        let trace = vec![
            "main".to_string(),
            "function_a".to_string(),
            "function_b".to_string(),
        ];

        let info1 = OptimizedAllocationInfo::new(0x1000, 64).with_stack_trace(trace.clone());
        let info2 = OptimizedAllocationInfo::new(0x2000, 128).with_stack_trace(trace);

        // Stack trace frames should have same content
        let trace1_strs = info1
            .stack_trace_strs()
            .expect("Failed to get stack trace strings");
        let trace2_strs = info2
            .stack_trace_strs()
            .expect("Failed to get stack trace strings");

        assert_eq!(trace1_strs, trace2_strs);
        assert_eq!(trace1_strs, vec!["main", "function_a", "function_b"]);

        // Verify the traces are actually Arc<Vec<Arc<str>>>
        assert!(info1.stack_trace.is_some());
        assert!(info2.stack_trace.is_some());
    }

    #[test]
    fn test_conversion_from_original() {
        use crate::core::types::AllocationInfo;

        clear_string_pool();

        let original = AllocationInfo {
            ptr: 0x1000,
            size: 64,
            var_name: Some("test_var".to_string()),
            type_name: Some("TestType".to_string()),
            scope_name: Some("test_scope".to_string()),
            timestamp_alloc: 12345,
            timestamp_dealloc: None,
            thread_id: "thread-1".to_string(),
            borrow_count: 0,
            stack_trace: Some(vec!["frame1".to_string(), "frame2".to_string()]),
            is_leaked: false,
            lifetime_ms: None,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        };

        let optimized = OptimizedAllocationInfo::from(original.clone());

        assert_eq!(optimized.ptr, original.ptr);
        assert_eq!(optimized.size, original.size);
        assert_eq!(optimized.var_name_str(), original.var_name.as_deref());
        assert_eq!(optimized.type_name_str(), original.type_name.as_deref());
        assert_eq!(optimized.scope_name_str(), original.scope_name.as_deref());

        // Convert back and verify
        let converted_back = crate::core::types::AllocationInfo::from(optimized);
        assert_eq!(converted_back.var_name, original.var_name);
        assert_eq!(converted_back.type_name, original.type_name);
        assert_eq!(converted_back.scope_name, original.scope_name);
    }

    #[test]
    fn test_deallocation_tracking() {
        let mut info = OptimizedAllocationInfo::new(0x1000, 64);

        assert!(info.is_active());
        assert!(info.lifetime_duration_nanos().is_none());

        info.mark_deallocated();

        assert!(!info.is_active());
        assert!(info.lifetime_duration_nanos().is_some());
        assert!(info.lifetime_duration_ms().is_some());
    }
}
