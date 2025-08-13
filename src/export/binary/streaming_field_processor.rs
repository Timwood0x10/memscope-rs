//! Streaming field processor for constant memory field processing
//!
//! This module provides streaming field processing capabilities that maintain
//! constant memory usage by processing records one at a time and immediately
//! discarding processed data. It includes LRU field caching and optimized
//! record structures for high-performance processing.

use crate::export::binary::error::BinaryExportError;
use crate::export::binary::field_parser::PartialAllocationInfo;
use crate::export::binary::selective_reader::AllocationField;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// Configuration for streaming field processor
#[derive(Debug, Clone)]
pub struct StreamingFieldProcessorConfig {
    /// Maximum size of the LRU field cache
    pub max_cache_size: usize,

    /// Enable field value caching
    pub enable_field_caching: bool,

    /// Enable pre-formatted field optimization
    pub enable_preformatted_fields: bool,

    /// Maximum memory usage before forcing cache eviction (bytes)
    pub max_memory_usage: usize,

    /// Enable streaming statistics collection
    pub enable_statistics: bool,

    /// Batch size for processing multiple records
    pub batch_size: usize,
}

impl Default for StreamingFieldProcessorConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 1000,
            enable_field_caching: true,
            enable_preformatted_fields: true,
            max_memory_usage: 16 * 1024 * 1024, // 16MB
            enable_statistics: true,
            batch_size: 100,
        }
    }
}

/// Statistics for streaming field processing
#[derive(Debug, Clone, Default)]
pub struct StreamingFieldProcessorStats {
    /// Total records processed
    pub records_processed: u64,

    /// Total fields processed
    pub fields_processed: u64,

    /// Number of cache hits
    pub cache_hits: u64,

    /// Number of cache misses
    pub cache_misses: u64,

    /// Number of cache evictions
    pub cache_evictions: u64,

    /// Total processing time in microseconds
    pub total_processing_time_us: u64,

    /// Time spent on field formatting (microseconds)
    pub field_formatting_time_us: u64,

    /// Memory usage in bytes
    pub current_memory_usage: usize,

    /// Peak memory usage in bytes
    pub peak_memory_usage: usize,

    /// Number of preformatted fields used
    pub preformatted_fields_used: u64,

    /// Number of records immediately discarded
    pub records_discarded: u64,
}

impl StreamingFieldProcessorStats {
    /// Calculate cache hit rate as percentage
    pub fn cache_hit_rate(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total_requests as f64) * 100.0
        }
    }

    /// Calculate processing throughput (records per second)
    pub fn processing_throughput(&self) -> f64 {
        if self.total_processing_time_us == 0 {
            0.0
        } else {
            (self.records_processed as f64 * 1_000_000.0) / self.total_processing_time_us as f64
        }
    }

    /// Calculate field processing efficiency
    pub fn field_processing_efficiency(&self) -> f64 {
        if self.fields_processed == 0 {
            0.0
        } else {
            (self.preformatted_fields_used as f64 / self.fields_processed as f64) * 100.0
        }
    }

    /// Calculate memory efficiency (bytes per record)
    pub fn memory_efficiency(&self) -> f64 {
        if self.records_processed == 0 {
            0.0
        } else {
            self.current_memory_usage as f64 / self.records_processed as f64
        }
    }
}

/// Cached field value with LRU metadata
#[derive(Debug, Clone)]
struct CachedFieldValue {
    /// The formatted field value
    value: String,

    /// Last access time for LRU eviction
    last_accessed: Instant,

    /// Access count for popularity tracking
    access_count: u32,

    /// Estimated memory usage of this cache entry
    memory_usage: usize,
}

impl CachedFieldValue {
    fn new(value: String) -> Self {
        let memory_usage = value.len() + std::mem::size_of::<Self>();
        Self {
            value,
            last_accessed: Instant::now(),
            access_count: 1,
            memory_usage,
        }
    }

    fn access(&mut self) -> &str {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.value
    }
}

/// Optimized record structure with pre-formatted fields
#[derive(Debug, Clone)]
pub struct OptimizedRecord {
    /// Original allocation info
    pub allocation: PartialAllocationInfo,

    /// Pre-formatted field values for common fields
    pub preformatted_fields: HashMap<AllocationField, String>,

    /// Timestamp when this record was created
    pub created_at: Instant,

    /// Estimated memory usage of this record
    pub memory_usage: usize,
}

impl OptimizedRecord {
    /// Create a new optimized record
    pub fn new(allocation: PartialAllocationInfo) -> Self {
        let mut record = Self {
            allocation,
            preformatted_fields: HashMap::new(),
            created_at: Instant::now(),
            memory_usage: 0,
        };

        // Estimate memory usage
        record.memory_usage = record.estimate_memory_usage();
        record
    }

    /// Pre-format commonly used fields
    pub fn preformat_fields(&mut self, fields: &HashSet<AllocationField>) {
        for field in fields {
            if let Some(formatted) = self.format_field(field) {
                self.preformatted_fields.insert(*field, formatted);
            }
        }

        // Update memory usage estimate
        self.memory_usage = self.estimate_memory_usage();
    }

    /// Get a formatted field value (from cache or format on demand)
    pub fn get_formatted_field(&self, field: &AllocationField) -> Option<String> {
        // Check preformatted fields first
        if let Some(cached) = self.preformatted_fields.get(field) {
            return Some(cached.clone());
        }

        // Format on demand
        self.format_field(field)
    }

    /// Format a specific field
    fn format_field(&self, field: &AllocationField) -> Option<String> {
        match field {
            AllocationField::Ptr => self.allocation.ptr.map(|ptr| format!("\"0x{:x}\"", ptr)),
            AllocationField::Size => self.allocation.size.map(|size| size.to_string()),
            AllocationField::VarName => {
                self.allocation
                    .var_name
                    .as_ref()
                    .map(|var_name| match var_name {
                        Some(name) => format!("\"{}\"", name.replace('"', "\\\"")),
                        None => "null".to_string(),
                    })
            }
            AllocationField::TypeName => {
                self.allocation
                    .type_name
                    .as_ref()
                    .map(|type_name| match type_name {
                        Some(name) => format!("\"{}\"", name.replace('"', "\\\"")),
                        None => "null".to_string(),
                    })
            }
            AllocationField::ScopeName => {
                self.allocation
                    .scope_name
                    .as_ref()
                    .map(|scope_name| match scope_name {
                        Some(name) => format!("\"{}\"", name.replace('"', "\\\"")),
                        None => "null".to_string(),
                    })
            }
            AllocationField::TimestampAlloc => {
                self.allocation.timestamp_alloc.map(|ts| ts.to_string())
            }
            AllocationField::TimestampDealloc => {
                self.allocation
                    .timestamp_dealloc
                    .as_ref()
                    .map(|ts_opt| match ts_opt {
                        Some(ts) => ts.to_string(),
                        None => "null".to_string(),
                    })
            }
            AllocationField::ThreadId => self
                .allocation
                .thread_id
                .as_ref()
                .map(|thread_id| format!("\"{}\"", thread_id.replace('"', "\\\""))),
            AllocationField::BorrowCount => {
                self.allocation.borrow_count.map(|count| count.to_string())
            }
            AllocationField::IsLeaked => self
                .allocation
                .is_leaked
                .map(|leaked| if leaked { "true" } else { "false" }.to_string()),
            AllocationField::StackTrace => {
                self.allocation
                    .stack_trace
                    .as_ref()
                    .map(|stack_trace_opt| match stack_trace_opt {
                        Some(trace) => {
                            let trace_json: Vec<String> = trace
                                .iter()
                                .map(|s| format!("\"{}\"", s.replace('"', "\\\"")))
                                .collect();
                            format!("[{}]", trace_json.join(", "))
                        }
                        None => "null".to_string(),
                    })
            }
            AllocationField::LifetimeMs => {
                self.allocation
                    .lifetime_ms
                    .as_ref()
                    .map(|lifetime_opt| match lifetime_opt {
                        Some(ms) => ms.to_string(),
                        None => "null".to_string(),
                    })
            }
            _ => None, // Advanced fields not implemented yet
        }
    }

    /// Estimate memory usage of this record
    fn estimate_memory_usage(&self) -> usize {
        let mut usage = std::mem::size_of::<Self>();

        // Add preformatted fields memory
        for (_, value) in &self.preformatted_fields {
            usage += value.len() + std::mem::size_of::<String>();
        }

        // Add allocation info memory (simplified estimation)
        usage += 256; // Approximate size of PartialAllocationInfo

        usage
    }
}

/// Streaming field processor with constant memory usage
pub struct StreamingFieldProcessor {
    /// Configuration
    config: StreamingFieldProcessorConfig,

    /// LRU cache for field values
    field_cache: HashMap<String, CachedFieldValue>,

    /// Statistics
    stats: StreamingFieldProcessorStats,

    /// Current memory usage estimate
    current_memory_usage: usize,
}

impl StreamingFieldProcessor {
    /// Create a new streaming field processor
    pub fn new() -> Self {
        Self::with_config(StreamingFieldProcessorConfig::default())
    }

    /// Create a new streaming field processor with custom configuration
    pub fn with_config(config: StreamingFieldProcessorConfig) -> Self {
        Self {
            config,
            field_cache: HashMap::new(),
            stats: StreamingFieldProcessorStats::default(),
            current_memory_usage: 0,
        }
    }

    /// Process a single record with streaming (constant memory)
    pub fn process_record_streaming<F>(
        &mut self,
        allocation: PartialAllocationInfo,
        requested_fields: &HashSet<AllocationField>,
        mut processor: F,
    ) -> Result<(), BinaryExportError>
    where
        F: FnMut(&OptimizedRecord) -> Result<(), BinaryExportError>,
    {
        let start_time = Instant::now();

        // Create optimized record
        let mut record = OptimizedRecord::new(allocation);

        // Pre-format requested fields
        if self.config.enable_preformatted_fields {
            record.preformat_fields(requested_fields);
        }

        // Update memory usage
        self.current_memory_usage += record.memory_usage;
        if self.current_memory_usage > self.stats.peak_memory_usage {
            self.stats.peak_memory_usage = self.current_memory_usage;
        }

        // Process the record
        processor(&record)?;

        // Immediately discard the record to maintain constant memory
        self.current_memory_usage -= record.memory_usage;
        self.stats.records_discarded += 1;

        // Update statistics
        self.stats.records_processed += 1;
        self.stats.fields_processed += requested_fields.len() as u64;
        self.stats.total_processing_time_us += start_time.elapsed().as_micros() as u64;

        // Check if we need to evict cache entries
        if self.current_memory_usage > self.config.max_memory_usage {
            self.evict_cache_entries()?;
        }

        Ok(())
    }

    /// Process multiple records in streaming fashion
    pub fn process_records_streaming<F>(
        &mut self,
        allocations: Vec<PartialAllocationInfo>,
        requested_fields: &HashSet<AllocationField>,
        mut processor: F,
    ) -> Result<(), BinaryExportError>
    where
        F: FnMut(&OptimizedRecord) -> Result<(), BinaryExportError>,
    {
        // Process records in batches to maintain constant memory
        for batch in allocations.chunks(self.config.batch_size) {
            for allocation in batch {
                self.process_record_streaming(
                    allocation.clone(),
                    requested_fields,
                    &mut processor,
                )?;
            }

            // Force cache cleanup between batches
            if self.current_memory_usage > self.config.max_memory_usage / 2 {
                self.evict_cache_entries()?;
            }
        }

        Ok(())
    }

    /// Get a cached field value or format and cache it
    pub fn get_or_format_field(
        &mut self,
        cache_key: &str,
        field: &AllocationField,
        allocation: &PartialAllocationInfo,
    ) -> Result<Option<String>, BinaryExportError> {
        if !self.config.enable_field_caching {
            // Format directly without caching
            return Ok(self.format_field_direct(field, allocation));
        }

        // Check cache first
        if let Some(cached) = self.field_cache.get_mut(cache_key) {
            self.stats.cache_hits += 1;
            return Ok(Some(cached.access().to_string()));
        }

        // Format and cache
        if let Some(formatted) = self.format_field_direct(field, allocation) {
            let cached_value = CachedFieldValue::new(formatted.clone());
            self.current_memory_usage += cached_value.memory_usage;

            self.field_cache.insert(cache_key.to_string(), cached_value);
            self.stats.cache_misses += 1;

            // Check if we need to evict
            if self.field_cache.len() > self.config.max_cache_size {
                self.evict_lru_entry()?;
            }

            Ok(Some(formatted))
        } else {
            self.stats.cache_misses += 1;
            Ok(None)
        }
    }

    /// Get current statistics
    pub fn get_stats(&self) -> &StreamingFieldProcessorStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = StreamingFieldProcessorStats::default();
    }

    /// Clear all caches
    pub fn clear_cache(&mut self) {
        self.field_cache.clear();
        self.current_memory_usage = 0;
    }

    /// Get current cache size
    pub fn cache_size(&self) -> usize {
        self.field_cache.len()
    }

    /// Get current memory usage
    pub fn memory_usage(&self) -> usize {
        self.current_memory_usage
    }

    // Private helper methods

    /// Format a field directly without caching
    fn format_field_direct(
        &self,
        field: &AllocationField,
        allocation: &PartialAllocationInfo,
    ) -> Option<String> {
        // Create a temporary OptimizedRecord for formatting
        let temp_record = OptimizedRecord::new(allocation.clone());
        temp_record.format_field(field)
    }

    /// Evict cache entries to reduce memory usage
    fn evict_cache_entries(&mut self) -> Result<(), BinaryExportError> {
        let target_size = self.config.max_cache_size / 2;

        while self.field_cache.len() > target_size {
            self.evict_lru_entry()?;
        }

        Ok(())
    }

    /// Evict the least recently used cache entry
    fn evict_lru_entry(&mut self) -> Result<(), BinaryExportError> {
        if let Some((lru_key, lru_value)) = self
            .field_cache
            .iter()
            .min_by_key(|(_, v)| (v.access_count, v.last_accessed))
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.current_memory_usage -= lru_value.memory_usage;
            self.field_cache.remove(&lru_key);
            self.stats.cache_evictions += 1;
        }

        Ok(())
    }
}

impl Default for StreamingFieldProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for streaming field processor configuration
pub struct StreamingFieldProcessorConfigBuilder {
    config: StreamingFieldProcessorConfig,
}

impl StreamingFieldProcessorConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: StreamingFieldProcessorConfig::default(),
        }
    }

    /// Set maximum cache size
    pub fn max_cache_size(mut self, size: usize) -> Self {
        self.config.max_cache_size = size;
        self
    }

    /// Enable or disable field caching
    pub fn field_caching(mut self, enabled: bool) -> Self {
        self.config.enable_field_caching = enabled;
        self
    }

    /// Enable or disable preformatted fields
    pub fn preformatted_fields(mut self, enabled: bool) -> Self {
        self.config.enable_preformatted_fields = enabled;
        self
    }

    /// Set maximum memory usage
    pub fn max_memory_usage(mut self, bytes: usize) -> Self {
        self.config.max_memory_usage = bytes;
        self
    }

    /// Enable or disable statistics collection
    pub fn statistics(mut self, enabled: bool) -> Self {
        self.config.enable_statistics = enabled;
        self
    }

    /// Set batch size
    pub fn batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }

    /// Build the configuration
    pub fn build(self) -> StreamingFieldProcessorConfig {
        self.config
    }
}

impl Default for StreamingFieldProcessorConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_field_processor_creation() {
        let processor = StreamingFieldProcessor::new();
        assert_eq!(processor.cache_size(), 0);
        assert_eq!(processor.memory_usage(), 0);
    }

    #[test]
    fn test_config_builder() {
        let config = StreamingFieldProcessorConfigBuilder::new()
            .max_cache_size(500)
            .field_caching(false)
            .preformatted_fields(false)
            .max_memory_usage(8 * 1024 * 1024)
            .build();

        assert_eq!(config.max_cache_size, 500);
        assert!(!config.enable_field_caching);
        assert!(!config.enable_preformatted_fields);
        assert_eq!(config.max_memory_usage, 8 * 1024 * 1024);
    }

    #[test]
    fn test_optimized_record_creation() {
        let allocation = PartialAllocationInfo {
            ptr: Some(0x1000),
            size: Some(1024),
            var_name: Some(Some("test_var".to_string())),
            type_name: Some(Some("i32".to_string())),
            scope_name: Some(None),
            timestamp_alloc: Some(1234567890),
            timestamp_dealloc: Some(None),
            thread_id: Some("main".to_string()),
            borrow_count: Some(0),
            stack_trace: Some(None),
            is_leaked: Some(false),
            lifetime_ms: Some(None),
        };

        let record = OptimizedRecord::new(allocation);
        assert!(record.memory_usage > 0);
        assert!(record.preformatted_fields.is_empty());
    }

    #[test]
    fn test_field_formatting() {
        let allocation = PartialAllocationInfo {
            ptr: Some(0x1000),
            size: Some(1024),
            var_name: Some(Some("test_var".to_string())),
            type_name: Some(Some("i32".to_string())),
            scope_name: Some(None),
            timestamp_alloc: Some(1234567890),
            timestamp_dealloc: Some(None),
            thread_id: Some("main".to_string()),
            borrow_count: Some(0),
            stack_trace: Some(None),
            is_leaked: Some(false),
            lifetime_ms: Some(None),
        };

        let record = OptimizedRecord::new(allocation);

        assert_eq!(
            record.get_formatted_field(&AllocationField::Ptr),
            Some("\"0x1000\"".to_string())
        );
        assert_eq!(
            record.get_formatted_field(&AllocationField::Size),
            Some("1024".to_string())
        );
        assert_eq!(
            record.get_formatted_field(&AllocationField::VarName),
            Some("\"test_var\"".to_string())
        );
        assert_eq!(
            record.get_formatted_field(&AllocationField::IsLeaked),
            Some("false".to_string())
        );
    }

    #[test]
    fn test_preformatted_fields() {
        let allocation = PartialAllocationInfo {
            ptr: Some(0x1000),
            size: Some(1024),
            var_name: Some(Some("test".to_string())),
            type_name: Some(Some("i32".to_string())),
            scope_name: Some(None),
            timestamp_alloc: Some(1234567890),
            timestamp_dealloc: Some(None),
            thread_id: Some("main".to_string()),
            borrow_count: Some(0),
            stack_trace: Some(None),
            is_leaked: Some(false),
            lifetime_ms: Some(None),
        };

        let mut record = OptimizedRecord::new(allocation);
        let fields = [AllocationField::Ptr, AllocationField::Size]
            .into_iter()
            .collect();

        record.preformat_fields(&fields);

        assert_eq!(record.preformatted_fields.len(), 2);
        assert!(record
            .preformatted_fields
            .contains_key(&AllocationField::Ptr));
        assert!(record
            .preformatted_fields
            .contains_key(&AllocationField::Size));
    }

    #[test]
    fn test_streaming_processing() {
        let mut processor = StreamingFieldProcessor::new();

        let allocation = PartialAllocationInfo {
            ptr: Some(0x1000),
            size: Some(1024),
            var_name: Some(Some("test".to_string())),
            type_name: Some(Some("i32".to_string())),
            scope_name: Some(None),
            timestamp_alloc: Some(1234567890),
            timestamp_dealloc: Some(None),
            thread_id: Some("main".to_string()),
            borrow_count: Some(0),
            stack_trace: Some(None),
            is_leaked: Some(false),
            lifetime_ms: Some(None),
        };

        let fields = [AllocationField::Ptr, AllocationField::Size]
            .into_iter()
            .collect();

        let mut processed_count = 0;
        processor
            .process_record_streaming(allocation, &fields, |_record| {
                processed_count += 1;
                Ok(())
            })
            .unwrap();

        assert_eq!(processed_count, 1);
        assert_eq!(processor.get_stats().records_processed, 1);
        assert_eq!(processor.get_stats().records_discarded, 1);
        assert_eq!(processor.get_stats().fields_processed, 2);
    }

    #[test]
    fn test_cache_operations() {
        let mut processor = StreamingFieldProcessor::new();

        let allocation = PartialAllocationInfo {
            ptr: Some(0x1000),
            size: Some(1024),
            var_name: Some(Some("test".to_string())),
            type_name: Some(Some("i32".to_string())),
            scope_name: Some(None),
            timestamp_alloc: Some(1234567890),
            timestamp_dealloc: Some(None),
            thread_id: Some("main".to_string()),
            borrow_count: Some(0),
            stack_trace: Some(None),
            is_leaked: Some(false),
            lifetime_ms: Some(None),
        };

        // Test cache miss
        let result1 = processor
            .get_or_format_field("test_key", &AllocationField::Ptr, &allocation)
            .unwrap();
        assert_eq!(result1, Some("\"0x1000\"".to_string()));
        assert_eq!(processor.get_stats().cache_misses, 1);
        assert_eq!(processor.cache_size(), 1);

        // Test cache hit
        let result2 = processor
            .get_or_format_field("test_key", &AllocationField::Ptr, &allocation)
            .unwrap();
        assert_eq!(result2, Some("\"0x1000\"".to_string()));
        assert_eq!(processor.get_stats().cache_hits, 1);
        assert_eq!(processor.cache_size(), 1);
    }

    #[test]
    fn test_statistics() {
        let stats = StreamingFieldProcessorStats {
            cache_hits: 8,
            cache_misses: 2,
            records_processed: 100,
            fields_processed: 500,
            preformatted_fields_used: 300,
            total_processing_time_us: 1_000_000, // 1 second
            current_memory_usage: 1024,
            ..Default::default()
        };

        assert_eq!(stats.cache_hit_rate(), 80.0);
        assert_eq!(stats.processing_throughput(), 100.0); // 100 records per second
        assert_eq!(stats.field_processing_efficiency(), 60.0); // 60% preformatted
        assert_eq!(stats.memory_efficiency(), 10.24); // 10.24 bytes per record
    }

    #[test]
    fn test_memory_management() {
        let config = StreamingFieldProcessorConfigBuilder::new()
            .max_memory_usage(1024) // Very small limit for testing
            .max_cache_size(5)
            .build();

        let mut processor = StreamingFieldProcessor::with_config(config);

        // Test that memory usage is tracked
        let initial_memory = processor.memory_usage();
        assert_eq!(initial_memory, 0);

        // Test cache clearing
        processor.clear_cache();
        assert_eq!(processor.cache_size(), 0);
        assert_eq!(processor.memory_usage(), 0);
    }
}
