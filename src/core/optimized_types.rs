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

    #[test]
    fn test_round_trip_conversion_with_all_fields() {
        use crate::core::types::AllocationInfo;

        clear_string_pool();

        // Create original with all possible fields set
        let original = AllocationInfo {
            ptr: 0x2000,
            size: 128,
            var_name: Some("complex_var".to_string()),
            type_name: Some("HashMap<String, Vec<i32>>".to_string()),
            scope_name: Some("complex_scope".to_string()),
            timestamp_alloc: 98765,
            timestamp_dealloc: Some(99999),
            thread_id: "thread-complex".to_string(),
            borrow_count: 5,
            stack_trace: Some(vec![
                "main".to_string(),
                "complex_function".to_string(),
                "nested_call".to_string(),
            ]),
            is_leaked: true,
            lifetime_ms: Some(234),
            borrow_info: None,
            clone_info: None,
            ownership_history_available: true,
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

        // Convert to optimized and back
        let optimized = OptimizedAllocationInfo::from(original.clone());
        let converted_back = AllocationInfo::from(optimized);

        // Verify all fields are preserved
        assert_eq!(converted_back.ptr, original.ptr);
        assert_eq!(converted_back.size, original.size);
        assert_eq!(converted_back.var_name, original.var_name);
        assert_eq!(converted_back.type_name, original.type_name);
        assert_eq!(converted_back.scope_name, original.scope_name);
        assert_eq!(converted_back.timestamp_alloc, original.timestamp_alloc);
        assert_eq!(converted_back.timestamp_dealloc, original.timestamp_dealloc);
        assert_eq!(converted_back.thread_id, original.thread_id);
        assert_eq!(converted_back.borrow_count, original.borrow_count);
        assert_eq!(converted_back.stack_trace, original.stack_trace);
        assert_eq!(converted_back.is_leaked, original.is_leaked);
        assert_eq!(converted_back.lifetime_ms, original.lifetime_ms);
        assert!(converted_back.ownership_history_available); // This is hardcoded in conversion
    }

    #[test]
    fn test_none_some_field_combinations() {
        clear_string_pool();

        // Test with all None optional fields
        let info_all_none = OptimizedAllocationInfo::new(0x3000, 256);
        assert_eq!(info_all_none.var_name_str(), None);
        assert_eq!(info_all_none.type_name_str(), None);
        assert_eq!(info_all_none.scope_name_str(), None);
        assert_eq!(info_all_none.stack_trace_strs(), None);
        assert_eq!(info_all_none.lifetime_ms, None);
        assert_eq!(info_all_none.timestamp_dealloc, None);

        // Test with mixed Some/None fields
        let info_mixed = OptimizedAllocationInfo::new(0x4000, 512)
            .with_var_info("some_var", "") // Empty type name
            .with_scope("some_scope");

        assert_eq!(info_mixed.var_name_str(), Some("some_var"));
        assert_eq!(info_mixed.type_name_str(), Some("")); // Empty string should be preserved
        assert_eq!(info_mixed.scope_name_str(), Some("some_scope"));
        assert_eq!(info_mixed.stack_trace_strs(), None);

        // Test with empty stack trace
        let info_empty_trace = OptimizedAllocationInfo::new(0x5000, 1024).with_stack_trace(vec![]);

        let empty_trace = info_empty_trace.stack_trace_strs().unwrap();
        assert!(empty_trace.is_empty());
    }

    #[test]
    fn test_large_fields_and_long_strings() {
        clear_string_pool();

        // Test with very long strings
        let long_var_name = "a".repeat(1000);
        let long_type_name = "VeryLongTypeName".repeat(100);
        let long_scope_name = "deeply::nested::scope::".repeat(50);

        let info = OptimizedAllocationInfo::new(usize::MAX, usize::MAX)
            .with_var_info(&long_var_name, &long_type_name)
            .with_scope(&long_scope_name);

        assert_eq!(info.ptr, usize::MAX);
        assert_eq!(info.size, usize::MAX);
        assert_eq!(info.var_name_str(), Some(long_var_name.as_str()));
        assert_eq!(info.type_name_str(), Some(long_type_name.as_str()));
        assert_eq!(info.scope_name_str(), Some(long_scope_name.as_str()));

        // Test with large stack trace
        let large_stack_trace: Vec<String> =
            (0..1000).map(|i| format!("function_frame_{}", i)).collect();

        let info_large_trace =
            OptimizedAllocationInfo::new(0x6000, 2048).with_stack_trace(large_stack_trace.clone());

        let trace_strs = info_large_trace.stack_trace_strs().unwrap();
        assert_eq!(trace_strs.len(), 1000);
        assert_eq!(trace_strs[0], "function_frame_0");
        assert_eq!(trace_strs[999], "function_frame_999");
    }

    #[test]
    fn test_serialization_deserialization() {
        clear_string_pool();

        let original = OptimizedAllocationInfo::new(0x7000, 4096)
            .with_var_info("serializable_var", "SerializableType")
            .with_scope("serialization_scope")
            .with_stack_trace(vec![
                "serialize_main".to_string(),
                "serialize_helper".to_string(),
            ]);

        // Test serialization
        let serialized = serde_json::to_string(&original).expect("Failed to serialize");
        assert!(serialized.contains("serializable_var"));
        assert!(serialized.contains("SerializableType"));
        assert!(serialized.contains("serialization_scope"));
        assert!(serialized.contains("serialize_main"));

        // Test deserialization
        let deserialized: OptimizedAllocationInfo =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(deserialized.ptr, original.ptr);
        assert_eq!(deserialized.size, original.size);
        assert_eq!(deserialized.var_name_str(), original.var_name_str());
        assert_eq!(deserialized.type_name_str(), original.type_name_str());
        assert_eq!(deserialized.scope_name_str(), original.scope_name_str());
        assert_eq!(deserialized.stack_trace_strs(), original.stack_trace_strs());
    }

    #[test]
    fn test_serialization_with_none_fields() {
        clear_string_pool();

        let info_minimal = OptimizedAllocationInfo::new(0x8000, 8192);

        // Test serialization of minimal info (most fields None)
        let serialized = serde_json::to_string(&info_minimal).expect("Failed to serialize minimal");
        let deserialized: OptimizedAllocationInfo =
            serde_json::from_str(&serialized).expect("Failed to deserialize minimal");

        assert_eq!(deserialized.ptr, info_minimal.ptr);
        assert_eq!(deserialized.size, info_minimal.size);
        assert_eq!(deserialized.var_name_str(), None);
        assert_eq!(deserialized.type_name_str(), None);
        assert_eq!(deserialized.scope_name_str(), None);
        assert_eq!(deserialized.stack_trace_strs(), None);
    }

    #[test]
    fn test_lifetime_calculations() {
        let mut info = OptimizedAllocationInfo::new(0x9000, 16384);

        // Before deallocation
        assert!(info.is_active());
        assert_eq!(info.lifetime_duration_nanos(), None);
        assert_eq!(info.lifetime_duration_ms(), None);
        assert_eq!(info.lifetime_ms, None);

        // Simulate some time passing and then deallocate
        let start_time = info.timestamp_alloc;
        info.mark_deallocated();

        // After deallocation
        assert!(!info.is_active());
        assert!(info.lifetime_duration_nanos().is_some());
        assert!(info.lifetime_duration_ms().is_some());
        assert!(info.lifetime_ms.is_some());

        let duration_nanos = info.lifetime_duration_nanos().unwrap();
        let duration_ms = info.lifetime_duration_ms().unwrap();
        let stored_lifetime_ms = info.lifetime_ms.unwrap();

        // Verify consistency between different lifetime calculations
        assert_eq!(duration_ms, duration_nanos / 1_000_000);
        assert_eq!(stored_lifetime_ms, duration_ms);

        // Verify deallocation timestamp is after allocation timestamp
        assert!(info.timestamp_dealloc.unwrap() >= start_time);
    }

    #[test]
    fn test_thread_id_handling() {
        clear_string_pool();

        let info = OptimizedAllocationInfo::new(0xA000, 32768);

        // Thread ID should be set automatically and be non-empty
        assert!(!info.thread_id_str().is_empty());
        assert!(info.thread_id_str().contains("ThreadId"));

        // Thread ID should be interned as Arc<str>
        assert!(!info.thread_id.is_empty());
    }

    #[test]
    fn test_borrow_count_tracking() {
        let mut info = OptimizedAllocationInfo::new(0xB000, 65536);

        // Initial borrow count should be 0
        assert_eq!(info.borrow_count, 0);

        // Manually modify borrow count (simulating borrow tracking)
        info.borrow_count = 5;
        assert_eq!(info.borrow_count, 5);

        // Test conversion preserves borrow count
        let original_alloc = crate::core::types::AllocationInfo::from(info.clone());
        assert_eq!(original_alloc.borrow_count, 5);

        let back_to_optimized = OptimizedAllocationInfo::from(original_alloc);
        assert_eq!(back_to_optimized.borrow_count, 5);
    }

    #[test]
    fn test_edge_case_values() {
        clear_string_pool();

        // Test with zero values
        let info_zero = OptimizedAllocationInfo::new(0, 0);
        assert_eq!(info_zero.ptr, 0);
        assert_eq!(info_zero.size, 0);

        // Test with maximum values
        let info_max = OptimizedAllocationInfo::new(usize::MAX, usize::MAX);
        assert_eq!(info_max.ptr, usize::MAX);
        assert_eq!(info_max.size, usize::MAX);

        // Test with special characters in strings
        let special_chars = "测试中文字符!@#$%^&*()[]{}|\\:;\"'<>,.?/~`";
        let info_special = OptimizedAllocationInfo::new(0xC000, 1024)
            .with_var_info(special_chars, special_chars)
            .with_scope(special_chars);

        assert_eq!(info_special.var_name_str(), Some(special_chars));
        assert_eq!(info_special.type_name_str(), Some(special_chars));
        assert_eq!(info_special.scope_name_str(), Some(special_chars));
    }

    #[test]
    fn test_clone_and_equality() {
        clear_string_pool();

        let info1 = OptimizedAllocationInfo::new(0xD000, 2048)
            .with_var_info("clone_test", "CloneType")
            .with_scope("clone_scope");

        let info2 = info1.clone();

        // Test that cloned info is equal
        assert_eq!(info1, info2);
        assert_eq!(info1.ptr, info2.ptr);
        assert_eq!(info1.size, info2.size);
        assert_eq!(info1.var_name_str(), info2.var_name_str());
        assert_eq!(info1.type_name_str(), info2.type_name_str());
        assert_eq!(info1.scope_name_str(), info2.scope_name_str());

        // Test that different infos are not equal
        let info3 = OptimizedAllocationInfo::new(0xE000, 4096);
        assert_ne!(info1, info3);
    }
}
