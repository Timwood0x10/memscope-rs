//! Core adapter for bridging old MemoryTracker to new CoreTracker
//!
//! This adapter maintains backward compatibility with the old MemoryTracker API
//! while internally using the new unified tracking system.

use crate::core::types::{AllocationInfo, TrackingError, TrackingResult};
use crate::core::types::{BorrowInfo as CoreBorrowInfo, CloneInfo as CoreCloneInfo};
use crate::data::{AllocationRecord, BorrowInfo as NewBorrowInfo, CloneInfo as NewCloneInfo};
use crate::manager::TrackingManager;
use crate::tracker::base::TrackBase;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

/// Core adapter that bridges old MemoryTracker to new CoreTracker
///
/// This adapter implements the old MemoryTracker interface while internally
/// using the new unified tracking system. It provides compatibility for all
/// legacy features including variable association, smart pointer tracking, etc.
pub struct CoreAdapter {
    /// New unified tracking manager
    manager: Arc<TrackingManager>,
    
    /// Extended state for legacy features not yet migrated
    extended_state: Arc<Mutex<CoreAdapterState>>,
}

/// Extended state for legacy features
#[derive(Debug)]
struct CoreAdapterState {
    /// Variable name mapping (ptr -> var_name)
    variable_names: HashMap<usize, String>,
    
    /// Type name mapping (ptr -> type_name)
    type_names: HashMap<usize, String>,
    
    /// Smart pointer relationships
    smart_pointer_info: HashMap<usize, SmartPointerInfo>,
    
    /// Borrow tracking data
    borrow_tracking: HashMap<usize, BorrowTrackingData>,
    
    /// Clone relationships
    clone_relationships: HashMap<usize, CloneRelationship>,
}

/// Smart pointer information (compatibility with old system)
#[derive(Debug, Clone)]
struct SmartPointerInfo {
    ptr_type: SmartPointerType,
    ref_count: u32,
    original_ptr: Option<usize>,
}

/// Smart pointer type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SmartPointerType {
    None,
    Rc,
    Arc,
    Box,
    Weak,
}

/// Borrow tracking data
#[derive(Debug, Clone)]
struct BorrowTrackingData {
    immutable_borrows: usize,
    mutable_borrows: usize,
    max_concurrent: usize,
    last_borrow_timestamp: Option<u64>,
}

/// Clone relationship information
#[derive(Debug, Clone)]
struct CloneRelationship {
    clone_count: usize,
    original_ptr: Option<usize>,
    is_clone: bool,
}

impl CoreAdapter {
    /// Create a new core adapter
    pub fn new() -> Self {
        Self {
            manager: Arc::new(TrackingManager::new_core()),
            extended_state: Arc::new(Mutex::new(CoreAdapterState {
                variable_names: HashMap::new(),
                type_names: HashMap::new(),
                smart_pointer_info: HashMap::new(),
                borrow_tracking: HashMap::new(),
                clone_relationships: HashMap::new(),
            })),
        }
    }

    /// Get the underlying tracking manager
    pub fn manager(&self) -> &Arc<TrackingManager> {
        &self.manager
    }

    /// Track an allocation with variable and type information
    pub fn track_allocation_with_metadata(
        &self,
        ptr: usize,
        size: usize,
        var_name: Option<String>,
        type_name: Option<String>,
    ) -> TrackingResult<()> {
        // Use new system for basic tracking
        self.manager.track_alloc(ptr, size);

        // Store extended metadata
        let mut state = self.extended_state.lock().unwrap();
        if let Some(name) = var_name {
            state.variable_names.insert(ptr, name);
        }
        if let Some(name) = type_name {
            state.type_names.insert(ptr, name);
        }

        Ok(())
    }

    /// Track an allocation (simplified interface for tests)
    pub fn track_alloc(&self, ptr: usize, size: usize) {
        self.manager.track_alloc(ptr, size);
    }

    /// Track a deallocation (simplified interface for tests)
    pub fn track_dealloc(&self, ptr: usize) {
        self.manager.track_dealloc(ptr);
    }

    /// Associate a variable with a memory allocation
    pub fn associate_var(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        let mut state = self.extended_state.lock().unwrap();
        state.variable_names.insert(ptr, var_name);
        state.type_names.insert(ptr, type_name);
        Ok(())
    }

    /// Track smart pointer allocation
    pub fn track_smart_pointer_allocation(
        &self,
        ptr: usize,
        size: usize,
        ptr_type: SmartPointerType,
        ref_count: u32,
    ) -> TrackingResult<()> {
        // Use new system for basic tracking
        self.manager.track_alloc(ptr, size);

        // Store smart pointer info
        let mut state = self.extended_state.lock().unwrap();
        state.smart_pointer_info.insert(ptr, SmartPointerInfo {
            ptr_type,
            ref_count,
            original_ptr: None,
        });

        Ok(())
    }

    /// Track smart pointer clone
    pub fn track_smart_pointer_clone(
        &self,
        source_ptr: usize,
        new_ptr: usize,
        ref_count: u32,
    ) -> TrackingResult<()> {
        // Track the new allocation
        self.manager.track_alloc(new_ptr, 0); // Size 0 for smart pointer reference

        // Update clone relationships
        let mut state = self.extended_state.lock().unwrap();
        state.clone_relationships.insert(new_ptr, CloneRelationship {
            clone_count: 1,
            original_ptr: Some(source_ptr),
            is_clone: true,
        });

        // Update ref count
        if let Some(info) = state.smart_pointer_info.get_mut(&source_ptr) {
            info.ref_count = ref_count;
        }

        Ok(())
    }

    /// Track borrow operation
    pub fn track_borrow(&self, ptr: usize, is_mutable: bool) -> TrackingResult<()> {
        let mut state = self.extended_state.lock().unwrap();
        let tracking_data = state.borrow_tracking.entry(ptr).or_insert_with(|| BorrowTrackingData {
            immutable_borrows: 0,
            mutable_borrows: 0,
            max_concurrent: 0,
            last_borrow_timestamp: None,
        });

        if is_mutable {
            tracking_data.mutable_borrows += 1;
        } else {
            tracking_data.immutable_borrows += 1;
        }

        let current_concurrent = tracking_data.immutable_borrows + tracking_data.mutable_borrows;
        if current_concurrent > tracking_data.max_concurrent {
            tracking_data.max_concurrent = current_concurrent;
        }

        tracking_data.last_borrow_timestamp = Some(Self::current_timestamp());

        Ok(())
    }

    /// Get active allocations with extended information
    pub fn get_active_allocations(&self) -> TrackingResult<Vec<AllocationInfo>> {
        let snapshot = self.manager.snapshot();
        let state = self.extended_state.lock().unwrap();

        let allocations = snapshot
            .allocations
            .into_iter()
            .map(|record| {
                let var_name = state.variable_names.get(&record.ptr).cloned();
                let type_name = state.type_names.get(&record.ptr).cloned();
                let smart_ptr_info = state.smart_pointer_info.get(&record.ptr);

                // Convert internal SmartPointerInfo to public SmartPointerInfo format
                let public_smart_ptr_info = smart_ptr_info.map(|info| {
                    crate::core::types::SmartPointerInfo {
                        data_ptr: info.original_ptr.unwrap_or(record.ptr),
                        cloned_from: state.clone_relationships.get(&record.ptr)
                            .and_then(|r| r.original_ptr),
                        clones: state.clone_relationships.iter()
                            .filter(|(_, r)| r.original_ptr == Some(record.ptr))
                            .map(|(ptr, _)| *ptr)
                            .collect(),
                        ref_count_history: vec![
                            crate::core::types::RefCountSnapshot {
                                timestamp: record.timestamp,
                                strong_count: info.ref_count as usize,
                                weak_count: 0,
                            }
                        ],
                        weak_count: None,
                        is_weak_reference: info.ptr_type == SmartPointerType::Weak,
                        is_data_owner: info.ref_count > 0,
                        is_implicitly_deallocated: info.ref_count == 0,
                        pointer_type: match info.ptr_type {
                            SmartPointerType::Rc => crate::core::types::SmartPointerType::Rc,
                            SmartPointerType::Arc => crate::core::types::SmartPointerType::Arc,
                            SmartPointerType::Box => crate::core::types::SmartPointerType::Box,
                            SmartPointerType::Weak => crate::core::types::SmartPointerType::RcWeak, // Default to RcWeak
                            SmartPointerType::None => crate::core::types::SmartPointerType::Rc, // Default
                        },
                    }
                });

                // Convert AllocationRecord to AllocationInfo (legacy format)
                AllocationInfo {
                    ptr: record.ptr,
                    size: record.size,
                    var_name: var_name.clone(),
                    type_name: type_name.clone(),
                    timestamp_alloc: record.timestamp,
                    timestamp_dealloc: record.dealloc_timestamp,
                    thread_id: format!("ThreadId({})", record.thread_id),
                    borrow_info: record.borrow_info.as_ref().map(Self::convert_borrow_info),
                    clone_info: record.clone_info.as_ref().map(Self::convert_clone_info),
                    ownership_history_available: record.ownership_history_available,
                    // Set other legacy fields with appropriate defaults
                    scope_name: None,
                    stack_trace: None,
                    is_leaked: record.is_active,
                    lifetime_ms: record.lifetime_ms(),
                    borrow_count: record.borrow_info.as_ref()
                        .map(|b| b.immutable_borrows + b.mutable_borrows)
                        .unwrap_or(0),
                    smart_pointer_info: public_smart_ptr_info,
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
            })
            .collect();

        Ok(allocations)
    }

    /// Convert internal SmartPointerInfo to public SmartPointerInfo format
    fn convert_smart_pointer_info(
        internal_info: &SmartPointerInfo,
        ptr: usize,
        timestamp: u64,
        clone_relationships: &HashMap<usize, CloneRelationship>,
    ) -> crate::core::types::SmartPointerInfo {
        crate::core::types::SmartPointerInfo {
            data_ptr: internal_info.original_ptr.unwrap_or(ptr),
            cloned_from: clone_relationships.get(&ptr)
                .and_then(|r| r.original_ptr),
            clones: clone_relationships.iter()
                .filter(|(_, r)| r.original_ptr == Some(ptr))
                .map(|(clone_ptr, _)| *clone_ptr)
                .collect(),
            ref_count_history: vec![
                crate::core::types::RefCountSnapshot {
                    timestamp,
                    strong_count: internal_info.ref_count as usize,
                    weak_count: 0,
                }
            ],
            weak_count: None,
            is_weak_reference: internal_info.ptr_type == SmartPointerType::Weak,
            is_data_owner: internal_info.ref_count > 0,
            is_implicitly_deallocated: internal_info.ref_count == 0,
            pointer_type: match internal_info.ptr_type {
                SmartPointerType::Rc => crate::core::types::SmartPointerType::Rc,
                SmartPointerType::Arc => crate::core::types::SmartPointerType::Arc,
                SmartPointerType::Box => crate::core::types::SmartPointerType::Box,
                SmartPointerType::Weak => crate::core::types::SmartPointerType::RcWeak, // Default to RcWeak
                SmartPointerType::None => crate::core::types::SmartPointerType::Rc, // Default
            },
        }
    }

    /// Convert new AllocationRecord to legacy AllocationInfo
    pub fn record_to_legacy_info(record: &AllocationRecord) -> AllocationInfo {
        AllocationInfo {
            ptr: record.ptr,
            size: record.size,
            var_name: record.var_name.clone(),
            type_name: record.type_name.clone(),
            timestamp_alloc: record.timestamp,
            timestamp_dealloc: record.dealloc_timestamp,
            thread_id: format!("ThreadId({})", record.thread_id),
            is_leaked: record.is_active,
            lifetime_ms: record.lifetime_ms(),
            borrow_info: record.borrow_info.as_ref().map(Self::convert_borrow_info),
            clone_info: record.clone_info.as_ref().map(Self::convert_clone_info),
            ownership_history_available: record.ownership_history_available,
            borrow_count: record.borrow_info.as_ref()
                .map(|b| b.immutable_borrows + b.mutable_borrows)
                .unwrap_or(0),
            scope_name: None,
            stack_trace: None,
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

    /// Get current timestamp in microseconds
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }

    /// Convert new BorrowInfo to legacy BorrowInfo
    fn convert_borrow_info(new: &NewBorrowInfo) -> CoreBorrowInfo {
        CoreBorrowInfo {
            immutable_borrows: new.immutable_borrows,
            mutable_borrows: new.mutable_borrows,
            max_concurrent_borrows: new.max_concurrent_borrows,
            last_borrow_timestamp: new.last_borrow_timestamp,
        }
    }

    /// Convert new CloneInfo to legacy CloneInfo
    fn convert_clone_info(new: &NewCloneInfo) -> CoreCloneInfo {
        CoreCloneInfo {
            clone_count: new.clone_count,
            is_clone: new.is_clone,
            original_ptr: new.original_ptr,
        }
    }
}

impl Default for CoreAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_adapter_creation() {
        let adapter = CoreAdapter::new();
        assert!(adapter.manager().is_enabled());
    }

    #[test]
    fn test_associate_var() {
        let adapter = CoreAdapter::new();
        let ptr = 0x1000;
        adapter.track_alloc(ptr, 1024);
        
        let result = adapter.associate_var(ptr, "test_var".to_string(), "Vec<i32>".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_track_borrow() {
        let adapter = CoreAdapter::new();
        let ptr = 0x1000;
        adapter.track_alloc(ptr, 1024);
        
        let result = adapter.track_borrow(ptr, false);
        assert!(result.is_ok());
        
        let result = adapter.track_borrow(ptr, true);
        assert!(result.is_ok());
    }
}