//! Binary file indexing system for fast allocation record lookup and filtering
//!
//! This module provides indexing capabilities for binary files to enable:
//! - O(1) record location by index
//! - Fast pre-filtering using bloom filters and range queries
//! - Compressed index storage to minimize memory usage
//! - Cache-friendly index structures

use crate::export::binary::error::BinaryExportError;
use crate::export::binary::format::FileHeader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Main binary file index containing all metadata needed for fast access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryIndex {
    /// Version of the index format for compatibility
    pub version: u32,
    
    /// Hash of the binary file content for cache validation
    pub file_hash: u64,
    
    /// Path to the original binary file
    pub file_path: PathBuf,
    
    /// File header information
    pub header: FileHeader,
    
    /// String table index information
    pub string_table: StringTableIndex,
    
    /// Compressed allocation records index
    pub allocations: CompactAllocationIndex,
    
    /// Advanced metrics segment information (optional)
    pub advanced_metrics: Option<AdvancedMetricsIndex>,
    
    /// When this index was created
    pub created_at: SystemTime,
    
    /// Size of the original binary file
    pub file_size: u64,
}

impl BinaryIndex {
    /// Current index format version
    pub const INDEX_VERSION: u32 = 1;
    
    /// Create a new binary index
    pub fn new(
        file_path: PathBuf,
        file_hash: u64,
        file_size: u64,
        header: FileHeader,
    ) -> Self {
        Self {
            version: Self::INDEX_VERSION,
            file_hash,
            file_path,
            header,
            string_table: StringTableIndex::default(),
            allocations: CompactAllocationIndex::default(),
            advanced_metrics: None,
            created_at: SystemTime::now(),
            file_size,
        }
    }
    
    /// Check if this index is valid for the given file
    pub fn is_valid_for_file(&self, file_path: &std::path::Path, file_hash: u64) -> bool {
        self.file_path == file_path && self.file_hash == file_hash
    }
    
    /// Get the number of allocation records
    pub fn record_count(&self) -> u32 {
        self.allocations.count
    }
    
    /// Get the file offset for a specific record index
    pub fn get_record_offset(&self, record_index: usize) -> Option<u64> {
        if record_index >= self.allocations.count as usize {
            return None;
        }
        
        let relative_offset = self.allocations.relative_offsets.get(record_index)?;
        Some(self.allocations.records_start_offset + *relative_offset as u64)
    }
    
    /// Get the record size for a specific record index
    pub fn get_record_size(&self, record_index: usize) -> Option<u16> {
        self.allocations.record_sizes.get(record_index).copied()
    }
    
    /// Check if quick filtering data is available
    pub fn has_quick_filter_data(&self) -> bool {
        self.allocations.quick_filter_data.is_some()
    }
}

/// String table index information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StringTableIndex {
    /// Offset of the string table in the file
    pub offset: u64,
    
    /// Size of the string table in bytes
    pub size: u64,
    
    /// Number of strings in the table
    pub string_count: u32,
    
    /// Whether the string table uses compression
    pub uses_compression: bool,
}

/// Compact allocation index using relative offsets and compressed data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompactAllocationIndex {
    /// Total number of allocation records
    pub count: u32,
    
    /// Absolute offset where allocation records start
    pub records_start_offset: u64,
    
    /// Relative offsets from records_start_offset (saves memory)
    pub relative_offsets: Vec<u32>,
    
    /// Size of each record in bytes (most records are < 64KB)
    pub record_sizes: Vec<u16>,
    
    /// Quick filtering data for large files (optional)
    pub quick_filter_data: Option<QuickFilterData>,
}

impl CompactAllocationIndex {
    /// Create a new compact allocation index
    pub fn new(records_start_offset: u64) -> Self {
        Self {
            count: 0,
            records_start_offset,
            relative_offsets: Vec::new(),
            record_sizes: Vec::new(),
            quick_filter_data: None,
        }
    }
    
    /// Add a record to the index
    pub fn add_record(&mut self, absolute_offset: u64, size: u16) -> Result<(), BinaryExportError> {
        if absolute_offset < self.records_start_offset {
            return Err(BinaryExportError::CorruptedData(
                "Record offset is before records start".to_string()
            ));
        }
        
        let relative_offset = absolute_offset - self.records_start_offset;
        if relative_offset > u32::MAX as u64 {
            return Err(BinaryExportError::CorruptedData(
                "Record offset too large for relative addressing".to_string()
            ));
        }
        
        self.relative_offsets.push(relative_offset as u32);
        self.record_sizes.push(size);
        self.count += 1;
        
        Ok(())
    }
    
    /// Reserve capacity for the expected number of records
    pub fn reserve(&mut self, capacity: usize) {
        self.relative_offsets.reserve(capacity);
        self.record_sizes.reserve(capacity);
    }
    
    /// Get memory usage of this index in bytes
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() +
        self.relative_offsets.capacity() * std::mem::size_of::<u32>() +
        self.record_sizes.capacity() * std::mem::size_of::<u16>() +
        self.quick_filter_data.as_ref().map_or(0, |qfd| qfd.memory_usage())
    }
}

/// Quick filtering data for large files to enable fast pre-filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickFilterData {
    /// Batch size used for range calculations (e.g., 1000 records per batch)
    pub batch_size: usize,
    
    /// Pointer value ranges for each batch of records
    pub ptr_ranges: Vec<(usize, usize)>,
    
    /// Size ranges for each batch of records
    pub size_ranges: Vec<(usize, usize)>,
    
    /// Timestamp ranges for each batch of records
    pub timestamp_ranges: Vec<(u64, u64)>,
    
    /// Bloom filter for thread IDs (serialized as bytes)
    pub thread_bloom_filter: Vec<u8>,
    
    /// Bloom filter for type names (serialized as bytes)
    pub type_bloom_filter: Vec<u8>,
    
    /// Bloom filter parameters
    pub bloom_filter_params: BloomFilterParams,
}

impl QuickFilterData {
    /// Create new quick filter data with the specified batch size
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            ptr_ranges: Vec::new(),
            size_ranges: Vec::new(),
            timestamp_ranges: Vec::new(),
            thread_bloom_filter: Vec::new(),
            type_bloom_filter: Vec::new(),
            bloom_filter_params: BloomFilterParams::default(),
        }
    }
    
    /// Get memory usage of this quick filter data in bytes
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() +
        self.ptr_ranges.capacity() * std::mem::size_of::<(usize, usize)>() +
        self.size_ranges.capacity() * std::mem::size_of::<(usize, usize)>() +
        self.timestamp_ranges.capacity() * std::mem::size_of::<(u64, u64)>() +
        self.thread_bloom_filter.capacity() +
        self.type_bloom_filter.capacity()
    }
    
    /// Check if a pointer value might be in the specified batch
    pub fn ptr_might_be_in_batch(&self, batch_index: usize, ptr: usize) -> bool {
        if let Some(&(min, max)) = self.ptr_ranges.get(batch_index) {
            ptr >= min && ptr <= max
        } else {
            false
        }
    }
    
    /// Check if a size value might be in the specified batch
    pub fn size_might_be_in_batch(&self, batch_index: usize, size: usize) -> bool {
        if let Some(&(min, max)) = self.size_ranges.get(batch_index) {
            size >= min && size <= max
        } else {
            false
        }
    }
    
    /// Check if a timestamp might be in the specified batch
    pub fn timestamp_might_be_in_batch(&self, batch_index: usize, timestamp: u64) -> bool {
        if let Some(&(min, max)) = self.timestamp_ranges.get(batch_index) {
            timestamp >= min && timestamp <= max
        } else {
            false
        }
    }
}

/// Bloom filter parameters for consistent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloomFilterParams {
    /// Number of hash functions used
    pub hash_functions: u32,
    
    /// Size of the bloom filter in bits
    pub filter_size_bits: u32,
    
    /// Expected number of elements
    pub expected_elements: u32,
    
    /// Target false positive rate
    pub false_positive_rate: f64,
}

impl Default for BloomFilterParams {
    fn default() -> Self {
        Self {
            hash_functions: 3,
            filter_size_bits: 8192, // 1KB
            expected_elements: 1000,
            false_positive_rate: 0.01, // 1%
        }
    }
}

/// Advanced metrics segment index information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedMetricsIndex {
    /// Offset of the advanced metrics segment
    pub offset: u64,
    
    /// Size of the advanced metrics segment
    pub size: u32,
    
    /// Bitmap indicating which metrics are present
    pub metrics_bitmap: u32,
    
    /// Index of individual metric sections within the segment
    pub metric_sections: HashMap<String, MetricSectionIndex>,
}

/// Index information for a specific metric section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSectionIndex {
    /// Offset within the advanced metrics segment
    pub relative_offset: u32,
    
    /// Size of this metric section
    pub size: u32,
    
    /// Number of entries in this section
    pub entry_count: u32,
}

/// Record metadata used during index building
#[derive(Debug, Clone)]
pub struct RecordMetadata {
    /// Pointer value
    pub ptr: usize,
    
    /// Allocation size
    pub size: usize,
    
    /// Allocation timestamp
    pub timestamp: u64,
    
    /// Thread ID (if available)
    pub thread_id: Option<String>,
    
    /// Type name (if available)
    pub type_name: Option<String>,
}

impl RecordMetadata {
    /// Create new record metadata
    pub fn new(ptr: usize, size: usize, timestamp: u64) -> Self {
        Self {
            ptr,
            size,
            timestamp,
            thread_id: None,
            type_name: None,
        }
    }
    
    /// Set thread ID
    pub fn with_thread_id(mut self, thread_id: String) -> Self {
        self.thread_id = Some(thread_id);
        self
    }
    
    /// Set type name
    pub fn with_type_name(mut self, type_name: String) -> Self {
        self.type_name = Some(type_name);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_binary_index_creation() {
        let file_path = PathBuf::from("test.memscope");
        let file_hash = 0x123456789ABCDEF0;
        let file_size = 1024;
        let header = FileHeader::new_legacy(100);
        
        let index = BinaryIndex::new(file_path.clone(), file_hash, file_size, header.clone());
        
        assert_eq!(index.version, BinaryIndex::INDEX_VERSION);
        assert_eq!(index.file_hash, file_hash);
        assert_eq!(index.file_path, file_path);
        assert_eq!(index.header, header);
        assert_eq!(index.file_size, file_size);
        assert_eq!(index.record_count(), 0);
    }
    
    #[test]
    fn test_index_validation() {
        let file_path = PathBuf::from("test.memscope");
        let file_hash = 0x123456789ABCDEF0;
        let header = FileHeader::new_legacy(100);
        
        let index = BinaryIndex::new(file_path.clone(), file_hash, 1024, header);
        
        // Valid file
        assert!(index.is_valid_for_file(&file_path, file_hash));
        
        // Invalid hash
        assert!(!index.is_valid_for_file(&file_path, 0xDEADBEEF));
        
        // Invalid path
        assert!(!index.is_valid_for_file(Path::new("other.memscope"), file_hash));
    }
    
    #[test]
    fn test_compact_allocation_index() {
        let mut index = CompactAllocationIndex::new(1000);
        
        // Add some records
        assert!(index.add_record(1000, 100).is_ok());
        assert!(index.add_record(1200, 200).is_ok());
        assert!(index.add_record(1500, 150).is_ok());
        
        assert_eq!(index.count, 3);
        assert_eq!(index.relative_offsets, vec![0, 200, 500]);
        assert_eq!(index.record_sizes, vec![100, 200, 150]);
        
        // Test error conditions
        assert!(index.add_record(500, 100).is_err()); // Before start offset
    }
    
    #[test]
    fn test_record_offset_calculation() {
        let file_path = PathBuf::from("test.memscope");
        let header = FileHeader::new_legacy(3);
        let mut index = BinaryIndex::new(file_path, 0x123, 1024, header);
        
        // Add some records to the allocation index
        index.allocations.records_start_offset = 1000;
        index.allocations.add_record(1000, 100).unwrap();
        index.allocations.add_record(1200, 200).unwrap();
        index.allocations.add_record(1500, 150).unwrap();
        
        // Test offset calculation
        assert_eq!(index.get_record_offset(0), Some(1000));
        assert_eq!(index.get_record_offset(1), Some(1200));
        assert_eq!(index.get_record_offset(2), Some(1500));
        assert_eq!(index.get_record_offset(3), None); // Out of bounds
        
        // Test size retrieval
        assert_eq!(index.get_record_size(0), Some(100));
        assert_eq!(index.get_record_size(1), Some(200));
        assert_eq!(index.get_record_size(2), Some(150));
        assert_eq!(index.get_record_size(3), None); // Out of bounds
    }
    
    #[test]
    fn test_quick_filter_data() {
        let mut qfd = QuickFilterData::new(1000);
        
        // Add some range data
        qfd.ptr_ranges.push((0x1000, 0x2000));
        qfd.ptr_ranges.push((0x3000, 0x4000));
        qfd.size_ranges.push((100, 500));
        qfd.size_ranges.push((600, 1000));
        qfd.timestamp_ranges.push((1000, 2000));
        qfd.timestamp_ranges.push((3000, 4000));
        
        // Test range checks
        assert!(qfd.ptr_might_be_in_batch(0, 0x1500));
        assert!(!qfd.ptr_might_be_in_batch(0, 0x2500));
        assert!(qfd.ptr_might_be_in_batch(1, 0x3500));
        
        assert!(qfd.size_might_be_in_batch(0, 300));
        assert!(!qfd.size_might_be_in_batch(0, 550));
        assert!(qfd.size_might_be_in_batch(1, 800));
        
        assert!(qfd.timestamp_might_be_in_batch(0, 1500));
        assert!(!qfd.timestamp_might_be_in_batch(0, 2500));
        assert!(qfd.timestamp_might_be_in_batch(1, 3500));
        
        // Test out of bounds
        assert!(!qfd.ptr_might_be_in_batch(2, 0x1500));
    }
    
    #[test]
    fn test_record_metadata() {
        let metadata = RecordMetadata::new(0x1000, 1024, 1234567890)
            .with_thread_id("main".to_string())
            .with_type_name("Vec<u8>".to_string());
        
        assert_eq!(metadata.ptr, 0x1000);
        assert_eq!(metadata.size, 1024);
        assert_eq!(metadata.timestamp, 1234567890);
        assert_eq!(metadata.thread_id, Some("main".to_string()));
        assert_eq!(metadata.type_name, Some("Vec<u8>".to_string()));
    }
    
    #[test]
    fn test_memory_usage_calculation() {
        let mut index = CompactAllocationIndex::new(1000);
        
        // Add some records
        for i in 0..100 {
            index.add_record(1000 + i * 100, 100).unwrap();
        }
        
        let memory_usage = index.memory_usage();
        
        // Should include base struct size plus vector capacities
        let expected_min = std::mem::size_of::<CompactAllocationIndex>() +
                          100 * std::mem::size_of::<u32>() +
                          100 * std::mem::size_of::<u16>();
        
        assert!(memory_usage >= expected_min);
    }
}