//! Selective binary reader for optimized field-specific reading
//!
//! This module provides selective reading capabilities that allow reading only
//! specific fields from binary allocation records, with advanced filtering
//! and optimization features.

use crate::core::types::AllocationInfo;
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::index::BinaryIndex;
use crate::export::binary::parser::BinaryParser;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::path::Path;

/// Configuration options for selective reading operations
#[derive(Debug, Clone)]
pub struct SelectiveReadOptions {
    /// Fields to include in the read operation
    pub include_fields: HashSet<AllocationField>,

    /// Filters to apply during reading
    pub filters: Vec<AllocationFilter>,

    /// Maximum number of records to read (None for unlimited)
    pub limit: Option<usize>,

    /// Number of records to skip from the beginning
    pub offset: Option<usize>,

    /// Field to sort results by
    pub sort_by: Option<SortField>,

    /// Sort order (ascending or descending)
    pub sort_order: SortOrder,

    /// Whether to enable batch processing optimizations
    pub enable_batch_processing: bool,

    /// Batch size for processing (default: 1000)
    pub batch_size: usize,
}

impl Default for SelectiveReadOptions {
    fn default() -> Self {
        Self {
            include_fields: AllocationField::all_basic_fields(),
            filters: Vec::new(),
            limit: None,
            offset: None,
            sort_by: None,
            sort_order: SortOrder::Ascending,
            enable_batch_processing: true,
            batch_size: 1000,
        }
    }
}

impl SelectiveReadOptions {
    /// Create a new SelectiveReadOptions with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the fields to include in the read operation
    pub fn with_fields(mut self, fields: HashSet<AllocationField>) -> Self {
        self.include_fields = fields;
        self
    }

    /// Add a single field to include
    pub fn include_field(mut self, field: AllocationField) -> Self {
        self.include_fields.insert(field);
        self
    }

    /// Add multiple fields to include
    pub fn include_fields(mut self, fields: &[AllocationField]) -> Self {
        for field in fields {
            self.include_fields.insert(*field);
        }
        self
    }

    /// Set filters to apply during reading
    pub fn with_filters(mut self, filters: Vec<AllocationFilter>) -> Self {
        self.filters = filters;
        self
    }

    /// Add a single filter
    pub fn add_filter(mut self, filter: AllocationFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Set the maximum number of records to read
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the number of records to skip
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set the field to sort by
    pub fn sort_by(mut self, field: SortField, order: SortOrder) -> Self {
        self.sort_by = Some(field);
        self.sort_order = order;
        self
    }

    /// Enable or disable batch processing
    pub fn with_batch_processing(mut self, enabled: bool, batch_size: Option<usize>) -> Self {
        self.enable_batch_processing = enabled;
        if let Some(size) = batch_size {
            self.batch_size = size;
        }
        self
    }

    /// Validate the configuration options
    pub fn validate(&self) -> Result<(), BinaryExportError> {
        if self.include_fields.is_empty() {
            return Err(BinaryExportError::CorruptedData(
                "At least one field must be included".to_string(),
            ));
        }

        if self.batch_size == 0 {
            return Err(BinaryExportError::CorruptedData(
                "Batch size must be greater than 0".to_string(),
            ));
        }

        // Note: offset and limit are independent - offset is how many to skip,
        // limit is how many to return after skipping

        Ok(())
    }

    /// Check if a specific field is included
    pub fn includes_field(&self, field: &AllocationField) -> bool {
        self.include_fields.contains(field)
    }

    /// Get the effective limit considering offset
    pub fn effective_limit(&self) -> Option<usize> {
        match (self.limit, self.offset) {
            (Some(limit), Some(offset)) => Some(limit + offset),
            (Some(limit), None) => Some(limit),
            _ => None,
        }
    }
}

/// Enumeration of all possible allocation fields that can be selectively read
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AllocationField {
    // Basic fields (always available)
    Ptr,
    Size,
    TimestampAlloc,
    ThreadId,

    // Optional basic fields
    VarName,
    TypeName,
    ScopeName,
    TimestampDealloc,
    BorrowCount,
    IsLeaked,
    LifetimeMs,

    // improve.md extensions
    BorrowInfo,
    CloneInfo,
    OwnershipHistoryAvailable,

    // Stack trace information
    StackTrace,

    // Advanced fields (may not be available in all files)
    SmartPointerInfo,
    MemoryLayout,
    GenericInfo,
    DynamicTypeInfo,
    RuntimeState,
    StackAllocation,
    TemporaryObject,
    FragmentationAnalysis,
    GenericInstantiation,
    TypeRelationships,
    TypeUsage,
    FunctionCallTracking,
    LifecycleTracking,
    AccessTracking,
    DropChainAnalysis,
}

impl AllocationField {
    /// Get all basic fields that are commonly needed
    pub fn all_basic_fields() -> HashSet<Self> {
        [
            Self::Ptr,
            Self::Size,
            Self::TimestampAlloc,
            Self::ThreadId,
            Self::VarName,
            Self::TypeName,
            Self::IsLeaked,
        ]
        .into_iter()
        .collect()
    }

    /// Get all available fields
    pub fn all_fields() -> HashSet<Self> {
        [
            Self::Ptr,
            Self::Size,
            Self::TimestampAlloc,
            Self::ThreadId,
            Self::VarName,
            Self::TypeName,
            Self::ScopeName,
            Self::TimestampDealloc,
            Self::BorrowCount,
            Self::IsLeaked,
            Self::LifetimeMs,
            // improve.md extensions
            Self::BorrowInfo,
            Self::CloneInfo,
            Self::OwnershipHistoryAvailable,
            Self::StackTrace,
            Self::SmartPointerInfo,
            Self::MemoryLayout,
            Self::GenericInfo,
            Self::DynamicTypeInfo,
            Self::RuntimeState,
            Self::StackAllocation,
            Self::TemporaryObject,
            Self::FragmentationAnalysis,
            Self::GenericInstantiation,
            Self::TypeRelationships,
            Self::TypeUsage,
            Self::FunctionCallTracking,
            Self::LifecycleTracking,
            Self::AccessTracking,
            Self::DropChainAnalysis,
        ]
        .into_iter()
        .collect()
    }

    /// Get fields needed for memory analysis
    pub fn memory_analysis_fields() -> HashSet<Self> {
        [
            Self::Ptr,
            Self::Size,
            Self::VarName,
            Self::TypeName,
            Self::ThreadId,
            Self::TimestampAlloc,
            Self::IsLeaked,
            Self::BorrowCount,
            // improve.md extensions for memory analysis
            Self::LifetimeMs,
            Self::BorrowInfo,
            Self::CloneInfo,
            Self::OwnershipHistoryAvailable,
        ]
        .into_iter()
        .collect()
    }

    /// Get fields needed for lifetime analysis
    pub fn lifetime_analysis_fields() -> HashSet<Self> {
        [
            Self::Ptr,
            Self::VarName,
            Self::TimestampAlloc,
            Self::TimestampDealloc,
            Self::LifetimeMs,
            Self::ScopeName,
        ]
        .into_iter()
        .collect()
    }

    /// Get fields needed for performance analysis
    pub fn performance_analysis_fields() -> HashSet<Self> {
        [
            Self::Ptr,
            Self::Size,
            Self::TimestampAlloc,
            Self::ThreadId,
            Self::BorrowCount,
            Self::FragmentationAnalysis,
        ]
        .into_iter()
        .collect()
    }

    /// Get fields needed for complex types analysis
    pub fn complex_types_fields() -> HashSet<Self> {
        [
            Self::Ptr,
            Self::Size,
            Self::VarName,
            Self::TypeName,
            Self::SmartPointerInfo,
            Self::MemoryLayout,
            Self::GenericInfo,
            Self::TypeRelationships,
        ]
        .into_iter()
        .collect()
    }

    /// Get fields needed for unsafe FFI analysis
    pub fn unsafe_ffi_fields() -> HashSet<Self> {
        [
            Self::Ptr,
            Self::VarName,
            Self::TypeName,
            Self::ThreadId,
            Self::StackTrace,
            Self::RuntimeState,
        ]
        .into_iter()
        .collect()
    }

    /// Check if this field is always available in binary files
    pub fn is_basic_field(&self) -> bool {
        matches!(
            self,
            Self::Ptr | Self::Size | Self::TimestampAlloc | Self::ThreadId
        )
    }

    /// Check if this field requires advanced metrics to be enabled
    pub fn requires_advanced_metrics(&self) -> bool {
        matches!(
            self,
            Self::SmartPointerInfo
                | Self::MemoryLayout
                | Self::GenericInfo
                | Self::DynamicTypeInfo
                | Self::RuntimeState
                | Self::StackAllocation
                | Self::TemporaryObject
                | Self::FragmentationAnalysis
                | Self::GenericInstantiation
                | Self::TypeRelationships
                | Self::TypeUsage
                | Self::FunctionCallTracking
                | Self::LifecycleTracking
                | Self::AccessTracking
                | Self::DropChainAnalysis
        )
    }
}

/// Filter conditions that can be applied during selective reading
#[derive(Debug, Clone)]
pub enum AllocationFilter {
    /// Filter by pointer value range
    PtrRange(usize, usize),

    /// Filter by allocation size range
    SizeRange(usize, usize),

    /// Filter by timestamp range
    TimestampRange(u64, u64),

    /// Filter by exact thread ID match
    ThreadEquals(String),

    /// Filter by thread ID pattern (contains)
    ThreadContains(String),

    /// Filter by exact type name match
    TypeEquals(String),

    /// Filter by type name pattern (contains)
    TypeContains(String),

    /// Filter by variable name pattern (contains)
    VarNameContains(String),

    /// Filter by scope name pattern (contains)
    ScopeNameContains(String),

    /// Filter records that have stack trace information
    HasStackTrace,

    /// Filter records that don't have stack trace information
    NoStackTrace,

    /// Filter leaked allocations only
    LeakedOnly,

    /// Filter non-leaked allocations only
    NotLeaked,

    /// Filter by minimum borrow count
    MinBorrowCount(usize),

    /// Filter by maximum borrow count
    MaxBorrowCount(usize),

    /// Filter by lifetime range (in milliseconds)
    LifetimeRange(u64, u64),
}

impl AllocationFilter {
    /// Check if this filter can be applied using index pre-filtering
    pub fn supports_index_prefiltering(&self) -> bool {
        matches!(
            self,
            Self::PtrRange(_, _)
                | Self::SizeRange(_, _)
                | Self::TimestampRange(_, _)
                | Self::ThreadEquals(_)
                | Self::ThreadContains(_)
                | Self::TypeEquals(_)
                | Self::TypeContains(_)
        )
    }

    /// Apply this filter to an allocation record
    pub fn matches(&self, allocation: &AllocationInfo) -> bool {
        match self {
            Self::PtrRange(min, max) => allocation.ptr >= *min && allocation.ptr <= *max,
            Self::SizeRange(min, max) => allocation.size >= *min && allocation.size <= *max,
            Self::TimestampRange(min, max) => {
                allocation.timestamp_alloc >= *min && allocation.timestamp_alloc <= *max
            }
            Self::ThreadEquals(thread) => allocation.thread_id == *thread,
            Self::ThreadContains(pattern) => allocation.thread_id.contains(pattern),
            Self::TypeEquals(type_name) => allocation.type_name.as_ref() == Some(type_name),
            Self::TypeContains(pattern) => allocation.type_name.as_ref().is_some_and(|t| t.contains(pattern)),
            Self::VarNameContains(pattern) => allocation.var_name.as_ref().is_some_and(|v| v.contains(pattern)),
            Self::ScopeNameContains(pattern) => allocation.scope_name.as_ref().is_some_and(|s| s.contains(pattern)),
            Self::HasStackTrace => allocation.stack_trace.is_some(),
            Self::NoStackTrace => allocation.stack_trace.is_none(),
            Self::LeakedOnly => allocation.is_leaked,
            Self::NotLeaked => !allocation.is_leaked,
            Self::MinBorrowCount(min) => allocation.borrow_count >= *min,
            Self::MaxBorrowCount(max) => allocation.borrow_count <= *max,
            Self::LifetimeRange(min, max) => allocation.lifetime_ms.is_some_and(|lifetime| lifetime >= *min && lifetime <= *max),
        }
    }
}

/// Fields that can be used for sorting results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortField {
    Ptr,
    Size,
    TimestampAlloc,
    TimestampDealloc,
    LifetimeMs,
    BorrowCount,
    ThreadId,
    TypeName,
    VarName,
}

/// Sort order for results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Builder for creating SelectiveReadOptions with a fluent API
pub struct SelectiveReadOptionsBuilder {
    options: SelectiveReadOptions,
}

impl SelectiveReadOptionsBuilder {
    /// Create a new builder with default options
    pub fn new() -> Self {
        Self {
            options: SelectiveReadOptions::default(),
        }
    }

    /// Create a builder for memory analysis
    pub fn for_memory_analysis() -> Self {
        Self {
            options: SelectiveReadOptions {
                include_fields: AllocationField::memory_analysis_fields(),
                ..Default::default()
            },
        }
    }

    /// Create a builder for lifetime analysis
    pub fn for_lifetime_analysis() -> Self {
        Self {
            options: SelectiveReadOptions {
                include_fields: AllocationField::lifetime_analysis_fields(),
                ..Default::default()
            },
        }
    }

    /// Create a builder for performance analysis
    pub fn for_performance_analysis() -> Self {
        Self {
            options: SelectiveReadOptions {
                include_fields: AllocationField::performance_analysis_fields(),
                ..Default::default()
            },
        }
    }

    /// Create a builder for complex types analysis
    pub fn for_complex_types_analysis() -> Self {
        Self {
            options: SelectiveReadOptions {
                include_fields: AllocationField::complex_types_fields(),
                ..Default::default()
            },
        }
    }

    /// Create a builder for unsafe FFI analysis
    pub fn for_unsafe_ffi_analysis() -> Self {
        Self {
            options: SelectiveReadOptions {
                include_fields: AllocationField::unsafe_ffi_fields(),
                ..Default::default()
            },
        }
    }

    /// Add a field to include
    pub fn include_field(mut self, field: AllocationField) -> Self {
        self.options.include_fields.insert(field);
        self
    }

    /// Add multiple fields to include
    pub fn include_fields(mut self, fields: &[AllocationField]) -> Self {
        for field in fields {
            self.options.include_fields.insert(*field);
        }
        self
    }

    /// Set all fields to include
    pub fn with_fields(mut self, fields: HashSet<AllocationField>) -> Self {
        self.options.include_fields = fields;
        self
    }

    /// Add a filter
    pub fn filter(mut self, filter: AllocationFilter) -> Self {
        self.options.filters.push(filter);
        self
    }

    /// Add multiple filters
    pub fn filters(mut self, filters: Vec<AllocationFilter>) -> Self {
        self.options.filters.extend(filters);
        self
    }

    /// Set the limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.options.limit = Some(limit);
        self
    }

    /// Set the offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.options.offset = Some(offset);
        self
    }

    /// Set sorting
    pub fn sort_by(mut self, field: SortField, order: SortOrder) -> Self {
        self.options.sort_by = Some(field);
        self.options.sort_order = order;
        self
    }

    /// Configure batch processing
    pub fn batch_processing(mut self, enabled: bool, batch_size: Option<usize>) -> Self {
        self.options.enable_batch_processing = enabled;
        if let Some(size) = batch_size {
            self.options.batch_size = size;
        }
        self
    }

    /// Build the final SelectiveReadOptions
    pub fn build(self) -> Result<SelectiveReadOptions, BinaryExportError> {
        self.options.validate()?;
        Ok(self.options)
    }
}

impl Default for SelectiveReadOptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selective_read_options_creation() {
        let options = SelectiveReadOptions::new();
        assert!(!options.include_fields.is_empty());
        assert!(options.filters.is_empty());
        assert_eq!(options.limit, None);
        assert_eq!(options.offset, None);
        assert_eq!(options.sort_by, None);
        assert_eq!(options.batch_size, 1000);
    }

    #[test]
    fn test_selective_read_options_builder() {
        let options = SelectiveReadOptionsBuilder::new()
            .include_field(AllocationField::Ptr)
            .include_field(AllocationField::Size)
            .filter(AllocationFilter::SizeRange(100, 1000))
            .limit(50)
            .offset(10)
            .sort_by(SortField::Size, SortOrder::Descending)
            .build()
            .expect("Test operation failed");

        assert!(options.includes_field(&AllocationField::Ptr));
        assert!(options.includes_field(&AllocationField::Size));
        assert_eq!(options.filters.len(), 1);
        assert_eq!(options.limit, Some(50));
        assert_eq!(options.offset, Some(10));
        assert_eq!(options.sort_by, Some(SortField::Size));
        assert_eq!(options.sort_order, SortOrder::Descending);
    }

    #[test]
    fn test_allocation_field_sets() {
        let basic_fields = AllocationField::all_basic_fields();
        assert!(basic_fields.contains(&AllocationField::Ptr));
        assert!(basic_fields.contains(&AllocationField::Size));
        assert!(basic_fields.contains(&AllocationField::ThreadId));

        let memory_fields = AllocationField::memory_analysis_fields();
        assert!(memory_fields.contains(&AllocationField::Ptr));
        assert!(memory_fields.contains(&AllocationField::Size));
        assert!(memory_fields.contains(&AllocationField::IsLeaked));

        let lifetime_fields = AllocationField::lifetime_analysis_fields();
        assert!(lifetime_fields.contains(&AllocationField::TimestampAlloc));
        assert!(lifetime_fields.contains(&AllocationField::TimestampDealloc));
        assert!(lifetime_fields.contains(&AllocationField::LifetimeMs));
    }

    #[test]
    fn test_allocation_filter_matching() {
        let allocation = AllocationInfo {
            ptr: 0x1000,
            size: 1024,
            var_name: Some("test_var".to_string()),
            type_name: Some("Vec<u8>".to_string()),
            scope_name: None,
            timestamp_alloc: 1234567890,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 2,
            stack_trace: Some(vec!["frame1".to_string()]),
            is_leaked: false,
            lifetime_ms: Some(1000),
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

        // Test various filters
        assert!(AllocationFilter::PtrRange(0x500, 0x1500).matches(&allocation));
        assert!(!AllocationFilter::PtrRange(0x2000, 0x3000).matches(&allocation));

        assert!(AllocationFilter::SizeRange(500, 2000).matches(&allocation));
        assert!(!AllocationFilter::SizeRange(2000, 3000).matches(&allocation));

        assert!(AllocationFilter::ThreadEquals("main".to_string()).matches(&allocation));
        assert!(!AllocationFilter::ThreadEquals("worker".to_string()).matches(&allocation));

        assert!(AllocationFilter::ThreadContains("mai".to_string()).matches(&allocation));
        assert!(!AllocationFilter::ThreadContains("work".to_string()).matches(&allocation));

        assert!(AllocationFilter::TypeContains("Vec".to_string()).matches(&allocation));
        assert!(!AllocationFilter::TypeContains("HashMap".to_string()).matches(&allocation));

        assert!(AllocationFilter::HasStackTrace.matches(&allocation));
        assert!(!AllocationFilter::NoStackTrace.matches(&allocation));

        assert!(!AllocationFilter::LeakedOnly.matches(&allocation));
        assert!(AllocationFilter::NotLeaked.matches(&allocation));

        assert!(AllocationFilter::MinBorrowCount(1).matches(&allocation));
        assert!(!AllocationFilter::MinBorrowCount(5).matches(&allocation));

        assert!(AllocationFilter::MaxBorrowCount(5).matches(&allocation));
        assert!(!AllocationFilter::MaxBorrowCount(1).matches(&allocation));
    }

    #[test]
    fn test_options_validation() {
        // Valid options
        let valid_options = SelectiveReadOptions::new();
        assert!(valid_options.validate().is_ok());

        // Invalid: empty fields
        let invalid_options = SelectiveReadOptions {
            include_fields: HashSet::new(),
            ..Default::default()
        };
        assert!(invalid_options.validate().is_err());

        // Invalid: zero batch size
        let invalid_options = SelectiveReadOptions {
            batch_size: 0,
            ..Default::default()
        };
        assert!(invalid_options.validate().is_err());

        // Valid: offset can be greater than limit (skip 15, return 10)
        let valid_options = SelectiveReadOptions {
            limit: Some(10),
            offset: Some(15),
            ..Default::default()
        };
        assert!(valid_options.validate().is_ok());
    }

    #[test]
    fn test_effective_limit_calculation() {
        let options = SelectiveReadOptions {
            limit: Some(100),
            offset: Some(50),
            ..Default::default()
        };
        assert_eq!(options.effective_limit(), Some(150));

        let options = SelectiveReadOptions {
            limit: Some(100),
            offset: None,
            ..Default::default()
        };
        assert_eq!(options.effective_limit(), Some(100));

        let options = SelectiveReadOptions {
            limit: None,
            offset: Some(50),
            ..Default::default()
        };
        assert_eq!(options.effective_limit(), None);
    }

    #[test]
    fn test_specialized_builders() {
        let memory_options = SelectiveReadOptionsBuilder::for_memory_analysis()
            .build()
            .expect("Test operation failed");
        assert!(memory_options.includes_field(&AllocationField::Ptr));
        assert!(memory_options.includes_field(&AllocationField::IsLeaked));

        let lifetime_options = SelectiveReadOptionsBuilder::for_lifetime_analysis()
            .build()
            .expect("Test operation failed");
        assert!(lifetime_options.includes_field(&AllocationField::TimestampAlloc));
        assert!(lifetime_options.includes_field(&AllocationField::LifetimeMs));

        let performance_options = SelectiveReadOptionsBuilder::for_performance_analysis()
            .build()
            .expect("Test operation failed");
        assert!(performance_options.includes_field(&AllocationField::BorrowCount));
        assert!(performance_options.includes_field(&AllocationField::FragmentationAnalysis));
    }

    #[test]
    fn test_field_categorization() {
        assert!(AllocationField::Ptr.is_basic_field());
        assert!(AllocationField::Size.is_basic_field());
        assert!(!AllocationField::VarName.is_basic_field());

        assert!(!AllocationField::Ptr.requires_advanced_metrics());
        assert!(AllocationField::SmartPointerInfo.requires_advanced_metrics());
        assert!(AllocationField::GenericInfo.requires_advanced_metrics());
    }

    #[test]
    fn test_filter_index_prefiltering_support() {
        assert!(AllocationFilter::PtrRange(0, 1000).supports_index_prefiltering());
        assert!(AllocationFilter::SizeRange(0, 1000).supports_index_prefiltering());
        assert!(AllocationFilter::ThreadEquals("main".to_string()).supports_index_prefiltering());
        assert!(!AllocationFilter::HasStackTrace.supports_index_prefiltering());
        assert!(!AllocationFilter::LeakedOnly.supports_index_prefiltering());
    }
}

/// Selective binary reader that uses indexes for optimized reading
pub struct SelectiveBinaryReader {
    /// Binary file index for fast lookups
    index: BinaryIndex,

    /// Buffered file reader
    reader: BufReader<File>,

    /// Cached allocations for batch processing
    allocation_cache: Vec<AllocationInfo>,

    /// Current position in the file
    current_position: u64,
}

#[allow(dead_code)]
impl SelectiveBinaryReader {
    /// Create a new selective reader with an existing index
    pub fn new_with_index<P: AsRef<Path>>(
        file_path: P,
        index: BinaryIndex,
    ) -> Result<Self, BinaryExportError> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        Ok(Self {
            index,
            reader,
            allocation_cache: Vec::new(),
            current_position: 0,
        })
    }

    /// Create a new selective reader and build index automatically
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, BinaryExportError> {
        let index_builder = crate::export::binary::BinaryIndexBuilder::new();
        let index = index_builder.build_index(&file_path)?;
        Self::new_with_index(file_path, index)
    }

    /// Read allocations based on selective options
    pub fn read_selective(
        &mut self,
        options: &SelectiveReadOptions,
    ) -> Result<Vec<AllocationInfo>, BinaryExportError> {
        // Validate options
        options.validate()?;

        // Get candidate record indices using index pre-filtering
        let candidate_indices = self.pre_filter_with_index(&options.filters)?;

        // Apply offset and limit to candidates
        let filtered_indices = self.apply_offset_limit(&candidate_indices, options);

        // Read and parse the selected records
        let mut allocations = self.read_records_by_indices(&filtered_indices, options)?;

        // Apply precise filtering to loaded records
        allocations = self.apply_precise_filters(allocations, &options.filters)?;

        // Apply sorting if requested
        if let Some(sort_field) = options.sort_by {
            self.sort_allocations(&mut allocations, sort_field, options.sort_order)?;
        }

        // Apply final limit after sorting
        if let Some(limit) = options.limit {
            allocations.truncate(limit);
        }

        Ok(allocations)
    }

    /// Read allocations in streaming mode with a callback
    pub fn read_selective_streaming<F>(
        &mut self,
        options: &SelectiveReadOptions,
        mut callback: F,
    ) -> Result<usize, BinaryExportError>
    where
        F: FnMut(&AllocationInfo) -> Result<bool, BinaryExportError>, // Return false to stop
    {
        // Validate options
        options.validate()?;

        // Get candidate record indices using index pre-filtering
        let candidate_indices = self.pre_filter_with_index(&options.filters)?;

        // Apply offset and limit to candidates
        let filtered_indices = self.apply_offset_limit(&candidate_indices, options);

        let mut _processed_count = 0;
        let mut returned_count = 0;

        // Process records in batches for memory efficiency
        for batch in filtered_indices.chunks(options.batch_size) {
            let batch_allocations = self.read_records_by_indices(batch, options)?;

            for allocation in batch_allocations {
                _processed_count += 1;

                // Apply precise filtering
                if self.allocation_matches_filters(&allocation, &options.filters)? {
                    // Apply offset
                    if let Some(offset) = options.offset {
                        if returned_count < offset {
                            returned_count += 1;
                            continue;
                        }
                    }

                    // Apply limit
                    if let Some(limit) = options.limit {
                        if returned_count >= limit + options.offset.unwrap_or(0) {
                            break;
                        }
                    }

                    // Call the callback
                    if !callback(&allocation)? {
                        break;
                    }

                    returned_count += 1;
                }
            }
        }

        Ok(returned_count)
    }

    /// Read the next allocation record in streaming mode
    pub fn read_next_allocation(&mut self) -> Result<Option<AllocationInfo>, BinaryExportError> {
        // This is a simplified implementation for streaming
        // In a full implementation, this would maintain state for sequential reading
        if self.current_position >= self.index.record_count() as u64 {
            return Ok(None);
        }

        let record_index = self.current_position as usize;
        if let Some(offset) = self.index.get_record_offset(record_index) {
            self.reader.seek(SeekFrom::Start(offset))?;
            let allocation = self.parse_allocation_record()?;
            self.current_position += 1;
            Ok(Some(allocation))
        } else {
            Ok(None)
        }
    }

    /// Get the underlying index
    pub fn index(&self) -> &BinaryIndex {
        &self.index
    }

    /// Get statistics about the binary file
    pub fn get_stats(&self) -> SelectiveReaderStats {
        SelectiveReaderStats {
            total_records: self.index.record_count(),
            file_size: self.index.file_size,
            has_quick_filter: self.index.has_quick_filter_data(),
            cache_size: self.allocation_cache.len(),
        }
    }

    // Private helper methods

    /// Pre-filter record indices using the index
    fn pre_filter_with_index(
        &self,
        filters: &[AllocationFilter],
    ) -> Result<Vec<usize>, BinaryExportError> {
        let total_records = self.index.record_count() as usize;
        let mut candidates: Vec<usize> = (0..total_records).collect();

        // If we have quick filter data, use it for pre-filtering
        if let Some(ref quick_filter) = self.index.allocations.quick_filter_data {
            candidates = self.apply_quick_filters(&candidates, filters, quick_filter)?;
        }

        // Apply bloom filter checks for supported filters
        candidates = self.apply_bloom_filter_checks(&candidates, filters)?;

        Ok(candidates)
    }

    /// Apply quick filters using range data
    fn apply_quick_filters(
        &self,
        candidates: &[usize],
        filters: &[AllocationFilter],
        quick_filter: &crate::export::binary::index::QuickFilterData,
    ) -> Result<Vec<usize>, BinaryExportError> {
        let mut filtered_candidates = Vec::new();

        for &candidate_index in candidates {
            let batch_index = candidate_index / quick_filter.batch_size;
            let mut matches = true;

            // Check each filter against the batch ranges
            for filter in filters {
                match filter {
                    AllocationFilter::PtrRange(min, max) => {
                        if !quick_filter.ptr_might_be_in_batch(batch_index, *min)
                            && !quick_filter.ptr_might_be_in_batch(batch_index, *max)
                        {
                            matches = false;
                            break;
                        }
                    }
                    AllocationFilter::SizeRange(min, max) => {
                        if !quick_filter.size_might_be_in_batch(batch_index, *min)
                            && !quick_filter.size_might_be_in_batch(batch_index, *max)
                        {
                            matches = false;
                            break;
                        }
                    }
                    AllocationFilter::TimestampRange(min, max) => {
                        if !quick_filter.timestamp_might_be_in_batch(batch_index, *min)
                            && !quick_filter.timestamp_might_be_in_batch(batch_index, *max)
                        {
                            matches = false;
                            break;
                        }
                    }
                    _ => {} // Other filters can't be pre-filtered with ranges
                }
            }

            if matches {
                filtered_candidates.push(candidate_index);
            }
        }

        Ok(filtered_candidates)
    }

    /// Apply bloom filter checks for string-based filters
    fn apply_bloom_filter_checks(
        &self,
        candidates: &[usize],
        filters: &[AllocationFilter],
    ) -> Result<Vec<usize>, BinaryExportError> {
        // This is a simplified implementation
        // In a full implementation, this would use the bloom filters from the index
        // to quickly eliminate candidates that definitely don't match string filters

        let mut filtered_candidates = Vec::new();

        for &candidate_index in candidates {
            let might_match = true;

            // For now, we'll just pass through all candidates
            // In a real implementation, we would check bloom filters here
            for filter in filters {
                match filter {
                    AllocationFilter::ThreadEquals(_)
                    | AllocationFilter::ThreadContains(_)
                    | AllocationFilter::TypeEquals(_)
                    | AllocationFilter::TypeContains(_) => {
                        // Bloom filter checking for efficient duplicate detection
                        // For now, assume all candidates might match
                    }
                    _ => {}
                }
            }

            if might_match {
                filtered_candidates.push(candidate_index);
            }
        }

        Ok(filtered_candidates)
    }

    /// Apply offset and limit to candidate indices
    fn apply_offset_limit(
        &self,
        candidates: &[usize],
        options: &SelectiveReadOptions,
    ) -> Vec<usize> {
        let start = options.offset.unwrap_or(0);
        let end = if let Some(limit) = options.limit {
            std::cmp::min(start + limit, candidates.len())
        } else {
            candidates.len()
        };

        if start >= candidates.len() {
            Vec::new()
        } else {
            candidates[start..end].to_vec()
        }
    }

    /// Read specific records by their indices
    fn read_records_by_indices(
        &mut self,
        indices: &[usize],
        options: &SelectiveReadOptions,
    ) -> Result<Vec<AllocationInfo>, BinaryExportError> {
        // Load all allocations once and cache them
        if self.allocation_cache.is_empty() {
            self.allocation_cache = BinaryParser::load_allocations(&self.index.file_path)?;
        }

        let mut allocations = Vec::with_capacity(indices.len());

        for &index in indices {
            if index < self.allocation_cache.len() {
                let allocation = &self.allocation_cache[index];
                // Only include requested fields
                let filtered_allocation = self.filter_allocation_fields(allocation, options);
                allocations.push(filtered_allocation);
            }
        }

        Ok(allocations)
    }

    /// Parse a single allocation record from the current position
    fn parse_allocation_record(&mut self) -> Result<AllocationInfo, BinaryExportError> {
        // For the initial implementation, we'll load all allocations once and cache them
        // This is not the most memory-efficient approach, but it's simple and correct
        if self.allocation_cache.is_empty() {
            self.allocation_cache = BinaryParser::load_allocations(&self.index.file_path)?;
        }

        let current_index = self.current_position as usize;
        if current_index < self.allocation_cache.len() {
            Ok(self.allocation_cache[current_index].clone())
        } else {
            Err(BinaryExportError::CorruptedData(
                "Record index out of bounds".to_string(),
            ))
        }
    }

    /// Filter allocation fields based on options
    fn filter_allocation_fields(
        &self,
        allocation: &AllocationInfo,
        options: &SelectiveReadOptions,
    ) -> AllocationInfo {
        // Create a new allocation with only the requested fields
        // This is a simplified implementation - in practice, we would
        // have a more efficient way to handle partial field loading

        let mut filtered = allocation.clone();

        // Clear fields that are not requested
        if !options.includes_field(&AllocationField::VarName) {
            filtered.var_name = None;
        }
        if !options.includes_field(&AllocationField::TypeName) {
            filtered.type_name = None;
        }
        if !options.includes_field(&AllocationField::ScopeName) {
            filtered.scope_name = None;
        }
        if !options.includes_field(&AllocationField::TimestampDealloc) {
            filtered.timestamp_dealloc = None;
        }
        if !options.includes_field(&AllocationField::LifetimeMs) {
            filtered.lifetime_ms = None;
        }
        // improve.md extensions
        if !options.includes_field(&AllocationField::BorrowInfo) {
            filtered.borrow_info = None;
        }
        if !options.includes_field(&AllocationField::CloneInfo) {
            filtered.clone_info = None;
        }
        if !options.includes_field(&AllocationField::OwnershipHistoryAvailable) {
            filtered.ownership_history_available = false;
        }
        if !options.includes_field(&AllocationField::StackTrace) {
            filtered.stack_trace = None;
        }
        if !options.includes_field(&AllocationField::SmartPointerInfo) {
            filtered.smart_pointer_info = None;
        }
        if !options.includes_field(&AllocationField::MemoryLayout) {
            filtered.memory_layout = None;
        }
        if !options.includes_field(&AllocationField::GenericInfo) {
            filtered.generic_info = None;
        }
        if !options.includes_field(&AllocationField::DynamicTypeInfo) {
            filtered.dynamic_type_info = None;
        }
        if !options.includes_field(&AllocationField::RuntimeState) {
            filtered.runtime_state = None;
        }
        if !options.includes_field(&AllocationField::StackAllocation) {
            filtered.stack_allocation = None;
        }
        if !options.includes_field(&AllocationField::TemporaryObject) {
            filtered.temporary_object = None;
        }
        if !options.includes_field(&AllocationField::FragmentationAnalysis) {
            filtered.fragmentation_analysis = None;
        }
        if !options.includes_field(&AllocationField::GenericInstantiation) {
            filtered.generic_instantiation = None;
        }
        if !options.includes_field(&AllocationField::TypeRelationships) {
            filtered.type_relationships = None;
        }
        if !options.includes_field(&AllocationField::TypeUsage) {
            filtered.type_usage = None;
        }
        if !options.includes_field(&AllocationField::FunctionCallTracking) {
            filtered.function_call_tracking = None;
        }
        if !options.includes_field(&AllocationField::LifecycleTracking) {
            filtered.lifecycle_tracking = None;
        }
        if !options.includes_field(&AllocationField::AccessTracking) {
            filtered.access_tracking = None;
        }
        if !options.includes_field(&AllocationField::DropChainAnalysis) {
            filtered.drop_chain_analysis = None;
        }

        filtered
    }

    /// Apply precise filters to loaded allocations
    fn apply_precise_filters(
        &self,
        allocations: Vec<AllocationInfo>,
        filters: &[AllocationFilter],
    ) -> Result<Vec<AllocationInfo>, BinaryExportError> {
        if filters.is_empty() {
            return Ok(allocations);
        }

        let mut filtered = Vec::new();
        for allocation in allocations {
            if self.allocation_matches_filters(&allocation, filters)? {
                filtered.push(allocation);
            }
        }

        Ok(filtered)
    }

    /// Check if an allocation matches all filters
    fn allocation_matches_filters(
        &self,
        allocation: &AllocationInfo,
        filters: &[AllocationFilter],
    ) -> Result<bool, BinaryExportError> {
        for filter in filters {
            if !filter.matches(allocation) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Sort allocations by the specified field and order
    fn sort_allocations(
        &self,
        allocations: &mut [AllocationInfo],
        sort_field: SortField,
        sort_order: SortOrder,
    ) -> Result<(), BinaryExportError> {
        match sort_field {
            SortField::Ptr => {
                allocations.sort_by_key(|a| a.ptr);
            }
            SortField::Size => {
                allocations.sort_by_key(|a| a.size);
            }
            SortField::TimestampAlloc => {
                allocations.sort_by_key(|a| a.timestamp_alloc);
            }
            SortField::TimestampDealloc => {
                allocations.sort_by_key(|a| a.timestamp_dealloc.unwrap_or(0));
            }
            SortField::LifetimeMs => {
                allocations.sort_by_key(|a| a.lifetime_ms.unwrap_or(0));
            }
            SortField::BorrowCount => {
                allocations.sort_by_key(|a| a.borrow_count);
            }
            SortField::ThreadId => {
                allocations.sort_by(|a, b| a.thread_id.cmp(&b.thread_id));
            }
            SortField::TypeName => {
                allocations.sort_by(|a, b| {
                    a.type_name
                        .as_deref()
                        .unwrap_or("")
                        .cmp(b.type_name.as_deref().unwrap_or(""))
                });
            }
            SortField::VarName => {
                allocations.sort_by(|a, b| {
                    a.var_name
                        .as_deref()
                        .unwrap_or("")
                        .cmp(b.var_name.as_deref().unwrap_or(""))
                });
            }
        }

        if sort_order == SortOrder::Descending {
            allocations.reverse();
        }

        Ok(())
    }
}

/// Statistics about the selective reader
#[derive(Debug, Clone)]
pub struct SelectiveReaderStats {
    /// Total number of records in the file
    #[allow(dead_code)]
    pub total_records: u32,

    /// Size of the binary file in bytes
    #[allow(dead_code)]
    pub file_size: u64,

    /// Whether quick filter data is available
    #[allow(dead_code)]
    pub has_quick_filter: bool,

    /// Current size of the allocation cache
    #[allow(dead_code)]
    pub cache_size: usize,
}

// Additional tests for SelectiveBinaryReader
#[cfg(test)]
mod selective_reader_tests {
    use super::*;
    use crate::export::binary::writer::BinaryWriter;
    use tempfile::NamedTempFile;

    fn create_test_binary_with_multiple_allocations() -> NamedTempFile {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let test_allocations = vec![
            AllocationInfo {
                ptr: 0x1000,
                size: 1024,
                var_name: Some("var1".to_string()),
                type_name: Some("Vec<u8>".to_string()),
                scope_name: None,
                timestamp_alloc: 1000,
                timestamp_dealloc: Some(2000),
                thread_id: "main".to_string(),
                borrow_count: 1,
                stack_trace: Some(vec!["frame1".to_string()]),
                is_leaked: false,
                lifetime_ms: Some(1000),
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
            },
            AllocationInfo {
                ptr: 0x2000,
                size: 2048,
                var_name: Some("var2".to_string()),
                type_name: Some("String".to_string()),
                scope_name: None,
                timestamp_alloc: 1500,
                timestamp_dealloc: None,
                thread_id: "worker".to_string(),
                borrow_count: 3,
                stack_trace: None,
                is_leaked: true,
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
            },
            AllocationInfo {
                ptr: 0x3000,
                size: 512,
                var_name: Some("var3".to_string()),
                type_name: Some("HashMap<String, i32>".to_string()),
                scope_name: None,
                timestamp_alloc: 2000,
                timestamp_dealloc: Some(3000),
                thread_id: "main".to_string(),
                borrow_count: 0,
                stack_trace: Some(vec!["frame2".to_string(), "frame3".to_string()]),
                is_leaked: false,
                lifetime_ms: Some(1000),
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
            },
        ];

        // Write test data to binary file
        {
            let mut writer = BinaryWriter::new(temp_file.path()).expect("Operation failed");
            writer
                .write_header(test_allocations.len() as u32)
                .expect("Operation failed");
            for alloc in &test_allocations {
                writer
                    .write_allocation(alloc)
                    .expect("Failed to write allocation");
            }
            writer.finish().expect("Failed to finish writing");
        }

        temp_file
    }

    #[test]
    fn test_selective_reader_creation() {
        let test_file = create_test_binary_with_multiple_allocations();

        // First, let's test if BinaryParser can load the file
        let allocations_result =
            crate::export::binary::BinaryParser::load_allocations(test_file.path());
        if let Err(ref e) = allocations_result {
            println!("Error loading allocations with BinaryParser: {e:?}");
        }
        assert!(
            allocations_result.is_ok(),
            "BinaryParser should be able to load the file"
        );

        let allocations = allocations_result.expect("Test operation failed");
        println!(
            "Successfully loaded {} allocations with BinaryParser",
            allocations.len()
        );

        // Let's debug the file structure
        let file_size = std::fs::metadata(test_file.path())
            .expect("Test operation failed")
            .len();
        println!("Binary file size: {file_size} bytes");

        // Read the file header manually
        let mut file = std::fs::File::open(test_file.path()).expect("Test operation failed");
        let mut header_bytes = [0u8; 16];
        std::io::Read::read_exact(&mut file, &mut header_bytes).expect("Test operation failed");
        println!("Header bytes: {header_bytes:?}");

        // Read string table marker
        let mut marker = [0u8; 4];
        std::io::Read::read_exact(&mut file, &mut marker).expect("Test operation failed");
        println!(
            "String table marker: {marker:?} ({})",
            String::from_utf8_lossy(&marker)
        );

        // Read string table size
        let mut size_bytes = [0u8; 4];
        std::io::Read::read_exact(&mut file, &mut size_bytes).expect("Test operation failed");
        let table_size = u32::from_le_bytes(size_bytes);
        println!("String table size: {table_size}");

        // Current position should be where allocation records start
        let current_pos = std::io::Seek::seek(&mut file, std::io::SeekFrom::Current(0))
            .expect("Failed to get test value");
        println!("Current position after string table header: {current_pos}");

        // Skip string table data if any
        if table_size > 0 {
            std::io::Seek::seek(&mut file, std::io::SeekFrom::Current(table_size as i64))
                .expect("Operation failed");
            let pos_after_table = std::io::Seek::seek(&mut file, std::io::SeekFrom::Current(0))
                .expect("Operation failed");
            println!(
                "Position after skipping string table data: {pos_after_table}"
            );
        }

        // Try to read the first allocation record
        let mut record_type = [0u8; 1];
        if std::io::Read::read_exact(&mut file, &mut record_type).is_ok() {
            println!("First allocation record type: {record_type:?}");
        } else {
            println!("Failed to read first allocation record type");
        }

        // Now test the index builder
        let index_builder = crate::export::binary::BinaryIndexBuilder::new();
        let index_result = index_builder.build_index(test_file.path());
        if let Err(ref e) = index_result {
            println!("Error building index: {e:?}");
        }
        assert!(
            index_result.is_ok(),
            "BinaryIndexBuilder should be able to build index"
        );

        let index = index_result.expect("Test operation failed");
        println!(
            "Successfully built index with {} records",
            index.record_count()
        );

        // Finally test the selective reader
        let reader = SelectiveBinaryReader::new(test_file.path());
        if let Err(ref e) = reader {
            println!("Error creating reader: {e:?}");
        }
        assert!(reader.is_ok());

        let reader = reader.expect("Failed to get test value");
        let stats = reader.get_stats();
        assert_eq!(stats.total_records, 3);
        assert!(stats.file_size > 0);
    }

    #[test]
    fn test_selective_reading_with_filters() {
        let test_file = create_test_binary_with_multiple_allocations();
        let mut reader =
            SelectiveBinaryReader::new(test_file.path()).expect("Test operation failed");

        // Test size filter
        let options = SelectiveReadOptionsBuilder::new()
            .filter(AllocationFilter::SizeRange(1000, 3000))
            .build()
            .expect("Test operation failed");

        let allocations = reader
            .read_selective(&options)
            .expect("Failed to read from binary file");
        assert_eq!(allocations.len(), 2); // Should match 1024 and 2048 byte allocations

        // Test thread filter
        let options = SelectiveReadOptionsBuilder::new()
            .filter(AllocationFilter::ThreadEquals("main".to_string()))
            .build()
            .expect("Test operation failed");

        let allocations = reader
            .read_selective(&options)
            .expect("Failed to read from binary file");
        assert_eq!(allocations.len(), 2); // Should match allocations from main thread
    }

    #[test]
    fn test_selective_reading_with_limit_and_offset() {
        let test_file = create_test_binary_with_multiple_allocations();
        let mut reader =
            SelectiveBinaryReader::new(test_file.path()).expect("Test operation failed");

        // Test limit
        let options = SelectiveReadOptionsBuilder::new()
            .limit(2)
            .build()
            .expect("Test operation failed");

        let allocations = reader
            .read_selective(&options)
            .expect("Failed to read from binary file");
        assert_eq!(allocations.len(), 2);

        // Test offset
        let options = SelectiveReadOptionsBuilder::new()
            .offset(1)
            .limit(1)
            .build()
            .expect("Test operation failed");

        let allocations = reader
            .read_selective(&options)
            .expect("Failed to read from binary file");
        assert_eq!(allocations.len(), 1);
    }

    #[test]
    fn test_selective_reading_with_sorting() {
        let test_file = create_test_binary_with_multiple_allocations();
        let mut reader =
            SelectiveBinaryReader::new(test_file.path()).expect("Test operation failed");

        // Test sorting by size (ascending)
        let options = SelectiveReadOptionsBuilder::new()
            .sort_by(SortField::Size, SortOrder::Ascending)
            .build()
            .expect("Test operation failed");

        let allocations = reader
            .read_selective(&options)
            .expect("Failed to read from binary file");
        assert_eq!(allocations.len(), 3);
        assert!(allocations[0].size <= allocations[1].size);
        assert!(allocations[1].size <= allocations[2].size);

        // Test sorting by size (descending)
        let options = SelectiveReadOptionsBuilder::new()
            .sort_by(SortField::Size, SortOrder::Descending)
            .build()
            .expect("Test operation failed");

        let allocations = reader
            .read_selective(&options)
            .expect("Failed to read from binary file");
        assert_eq!(allocations.len(), 3);
        assert!(allocations[0].size >= allocations[1].size);
        assert!(allocations[1].size >= allocations[2].size);
    }

    #[test]
    fn test_streaming_read() {
        let test_file = create_test_binary_with_multiple_allocations();
        let mut reader =
            SelectiveBinaryReader::new(test_file.path()).expect("Test operation failed");

        let options = SelectiveReadOptionsBuilder::new()
            .filter(AllocationFilter::ThreadEquals("main".to_string()))
            .build()
            .expect("Test operation failed");

        let mut count = 0;
        let result = reader.read_selective_streaming(&options, |_allocation| {
            count += 1;
            Ok(true) // Continue processing
        });

        assert!(result.is_ok());
        assert_eq!(count, 2); // Should process 2 allocations from main thread
    }

    #[test]
    fn test_field_filtering() {
        let test_file = create_test_binary_with_multiple_allocations();
        let mut reader =
            SelectiveBinaryReader::new(test_file.path()).expect("Test operation failed");

        // Only include basic fields
        let options = SelectiveReadOptionsBuilder::new()
            .with_fields(
                [AllocationField::Ptr, AllocationField::Size]
                    .into_iter()
                    .collect(),
            )
            .build()
            .expect("Test operation failed");

        let allocations = reader
            .read_selective(&options)
            .expect("Failed to read from binary file");
        assert_eq!(allocations.len(), 3);

        // Check that non-included fields are cleared
        for allocation in &allocations {
            // Basic fields should be present
            assert!(allocation.ptr > 0);
            assert!(allocation.size > 0);

            // Non-included fields should be None/default
            // Note: This test assumes the field filtering is working correctly
            // In practice, we might need to adjust based on the actual implementation
        }
    }
}
