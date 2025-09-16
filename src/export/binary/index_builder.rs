//! Binary index builder for creating optimized indexes from binary files
//!
//! This module provides functionality to scan binary files and build indexes
//! that enable fast record lookup and filtering operations.

use crate::export::binary::error::BinaryExportError;
use crate::export::binary::format::{FileHeader, ALLOCATION_RECORD_TYPE, HEADER_SIZE};
use crate::export::binary::index::{
    BinaryIndex, BloomFilterParams, CompactAllocationIndex, QuickFilterData, RecordMetadata,
    StringTableIndex,
};
use crate::export::binary::serializable::primitives;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// Builder for creating binary file indexes
pub struct BinaryIndexBuilder {
    /// Threshold for enabling quick filter data (number of records)
    quick_filter_threshold: u32,

    /// Batch size for quick filter data
    quick_filter_batch_size: usize,

    /// Bloom filter parameters
    bloom_filter_params: BloomFilterParams,
}

impl BinaryIndexBuilder {
    /// Create a new index builder with default settings
    pub fn new() -> Self {
        Self {
            quick_filter_threshold: 1000,
            quick_filter_batch_size: 1000,
            bloom_filter_params: BloomFilterParams::default(),
        }
    }

    /// Set the threshold for enabling quick filter data
    pub fn with_quick_filter_threshold(mut self, threshold: u32) -> Self {
        self.quick_filter_threshold = threshold;
        self
    }

    /// Set the batch size for quick filter data
    pub fn with_quick_filter_batch_size(mut self, batch_size: usize) -> Self {
        self.quick_filter_batch_size = batch_size;
        self
    }

    /// Set bloom filter parameters
    pub fn with_bloom_filter_params(mut self, params: BloomFilterParams) -> Self {
        self.bloom_filter_params = params;
        self
    }

    /// Build an index for the specified binary file
    pub fn build_index<P: AsRef<Path>>(
        &self,
        binary_path: P,
    ) -> Result<BinaryIndex, BinaryExportError> {
        let path = binary_path.as_ref();
        let file = File::open(path)?;
        let file_size = file.metadata()?.len();
        let file_hash = self.compute_file_hash(path)?;

        let mut reader = BufReader::new(file);

        // Read and validate header
        let header = self.read_header(&mut reader)?;

        // Read string table information
        let string_table_info = self.read_string_table_info(&mut reader)?;

        // Scan allocation records and build index
        let allocation_index = self.scan_allocation_records(&mut reader, &header)?;

        // Create the index
        let mut index = BinaryIndex::new(path.to_path_buf(), file_hash, file_size, header);

        index.string_table = string_table_info;
        index.allocations = allocation_index;

        Ok(index)
    }

    /// Compute hash of the file content for cache validation
    fn compute_file_hash<P: AsRef<Path>>(&self, path: P) -> Result<u64, BinaryExportError> {
        let mut file = File::open(path)?;
        let mut hasher = DefaultHasher::new();

        // Hash file metadata
        let metadata = file.metadata()?;
        metadata.len().hash(&mut hasher);
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                duration.as_secs().hash(&mut hasher);
            }
        }

        // Hash first 1KB and last 1KB of file for quick validation
        let mut buffer = [0u8; 1024];
        let bytes_read = file.read(&mut buffer)?;
        buffer[..bytes_read].hash(&mut hasher);

        // Hash last 1KB if file is large enough
        if metadata.len() > 2048 {
            file.seek(SeekFrom::End(-1024))?;
            let bytes_read = file.read(&mut buffer)?;
            buffer[..bytes_read].hash(&mut hasher);
        }

        Ok(hasher.finish())
    }

    /// Read and validate file header
    fn read_header(&self, reader: &mut BufReader<File>) -> Result<FileHeader, BinaryExportError> {
        let mut header_bytes = [0u8; HEADER_SIZE];
        reader.read_exact(&mut header_bytes)?;

        let header = FileHeader::from_bytes(&header_bytes);

        // Validate magic bytes
        if !header.is_valid_magic() {
            return Err(BinaryExportError::InvalidMagic {
                expected: "MEMSCOPE".to_string(),
                actual: String::from_utf8_lossy(&header.magic).to_string(),
            });
        }

        // Check version compatibility
        if !header.is_compatible_version() {
            return Err(BinaryExportError::UnsupportedVersion(header.version));
        }

        Ok(header)
    }

    /// Read string table information without loading the actual strings
    fn read_string_table_info(
        &self,
        reader: &mut BufReader<File>,
    ) -> Result<StringTableIndex, BinaryExportError> {
        let table_start_offset = reader.stream_position()?;

        // Read string table marker (4 bytes)
        let mut marker = [0u8; 4];
        reader.read_exact(&mut marker)?;

        // Read table size (4 bytes)
        let table_size = primitives::read_u32(reader)?;

        if &marker == b"STBL" && table_size > 0 {
            // Read compression flag
            let uses_compression = primitives::read_u8(reader)? != 0;

            // Read string count
            let string_count = primitives::read_u32(reader)?;

            // Skip the actual string data
            reader.seek(SeekFrom::Current(table_size as i64 - 9))?; // 9 = 4 + 1 + 4 (size + compression + count)

            Ok(StringTableIndex {
                offset: table_start_offset,
                size: table_size as u64 + 8, // Include marker and size fields
                string_count,
                uses_compression,
            })
        } else if &marker == b"NONE" {
            // table_size was already read above, no need to read it again
            Ok(StringTableIndex::default())
        } else {
            Err(BinaryExportError::CorruptedData(
                "Invalid string table marker".to_string(),
            ))
        }
    }

    /// Scan allocation records and build the allocation index
    fn scan_allocation_records(
        &self,
        reader: &mut BufReader<File>,
        header: &FileHeader,
    ) -> Result<CompactAllocationIndex, BinaryExportError> {
        let records_start_offset = reader.stream_position()?;
        let mut allocation_index = CompactAllocationIndex::new(records_start_offset);
        allocation_index.reserve(header.total_count as usize);

        let mut record_metadata = Vec::new();
        let mut quick_filter_builder = if header.total_count >= self.quick_filter_threshold {
            Some(QuickFilterBuilder::new(
                self.quick_filter_batch_size,
                self.bloom_filter_params.clone(),
            ))
        } else {
            None
        };

        // Get file size for bounds checking
        let file_size = reader
            .get_ref()
            .metadata()
            .map_err(|e| {
                BinaryExportError::CorruptedData(format!("Failed to get file metadata: {e:?}"))
            })?
            .len();

        // Scan each allocation record with error recovery
        for i in 0..header.total_count {
            let record_start_offset = reader.stream_position()?;

            // Check if we're near end of file
            if record_start_offset >= file_size {
                return Err(BinaryExportError::CorruptedData(format!(
                    "Reached end of file while reading record {} of {} at offset {}",
                    i + 1,
                    header.total_count,
                    record_start_offset
                )));
            }

            // Read record metadata with error recovery
            let metadata = match self.read_record_metadata(reader) {
                Ok(metadata) => metadata,
                Err(e) => {
                    tracing::warn!(
                        "Failed to read record {} at offset {}: {e}",
                        i + 1,
                        record_start_offset,
                    );

                    // Try to recover by skipping this record
                    // This is better than failing completely
                    continue;
                }
            };

            let record_end_offset = reader.stream_position()?;

            // Validate record size is reasonable
            let record_size_u64 = record_end_offset - record_start_offset;
            if record_size_u64 > u16::MAX as u64 {
                tracing::warn!(
                    "Record {} has unusually large size: {} bytes, skipping",
                    i + 1,
                    record_size_u64
                );
                continue;
            }

            let record_size = record_size_u64 as u16;

            // Add to allocation index
            if let Err(e) = allocation_index.add_record(record_start_offset, record_size) {
                tracing::warn!("Failed to add record {} to index: {}", i + 1, e);
                continue;
            }

            // Add to quick filter builder if enabled
            if let Some(ref mut builder) = quick_filter_builder {
                builder.add_record(i as usize, &metadata);
            }

            record_metadata.push(metadata);
        }

        // Build quick filter data if we have enough records
        if let Some(builder) = quick_filter_builder {
            allocation_index.quick_filter_data = Some(builder.build());
        }

        tracing::info!(
            "Successfully indexed {} allocation records",
            record_metadata.len()
        );

        Ok(allocation_index)
    }

    /// Read record metadata without fully parsing the allocation record
    fn read_record_metadata(
        &self,
        reader: &mut BufReader<File>,
    ) -> Result<RecordMetadata, BinaryExportError> {
        let current_position = reader.stream_position().unwrap_or(0);

        // Read Type (1 byte)
        let mut type_byte = [0u8; 1];
        reader.read_exact(&mut type_byte).map_err(|e| {
            BinaryExportError::CorruptedData(format!(
                "Failed to read record type at position {current_position}: {e:?}",
            ))
        })?;

        if type_byte[0] != ALLOCATION_RECORD_TYPE {
            return Err(BinaryExportError::CorruptedData(format!(
                "Invalid record type: {} at position {current_position}",
                type_byte[0]
            )));
        }

        // Read Length (4 bytes)
        let mut length_bytes = [0u8; 4];
        reader.read_exact(&mut length_bytes).map_err(|e| {
            BinaryExportError::CorruptedData(format!(
                "Failed to read record length at position {}: {e:?}",
                current_position + 1,
            ))
        })?;
        let record_length = u32::from_le_bytes(length_bytes);

        // Validate record length is reasonable
        if record_length == 0 || record_length > 1024 * 1024 {
            return Err(BinaryExportError::CorruptedData(format!(
                "Invalid record length: {record_length} at position {current_position}",
            )));
        }

        // Read basic fields that we need for indexing
        let ptr = primitives::read_u64(reader).map_err(|e| {
            BinaryExportError::CorruptedData(format!(
                "Failed to read ptr at position {}: {e:?}",
                current_position + 5,
            ))
        })? as usize;

        let size = primitives::read_u64(reader).map_err(|e| {
            BinaryExportError::CorruptedData(format!(
                "Failed to read size at position {}: {e:?}",
                current_position + 13,
            ))
        })? as usize;

        let timestamp = primitives::read_u64(reader).map_err(|e| {
            BinaryExportError::CorruptedData(format!(
                "Failed to read timestamp at position {}: {e:?}",
                current_position + 21,
            ))
        })?;

        // Calculate remaining bytes correctly
        // record_length includes the data after Type+Length fields
        // We've read: ptr(8) + size(8) + timestamp(8) = 24 bytes of data
        let bytes_read_from_data = 24u32;

        if record_length < bytes_read_from_data {
            return Err(BinaryExportError::CorruptedData(format!(
                "Record length {record_length} is smaller than minimum required {bytes_read_from_data} at position {current_position}",
            )));
        }

        let remaining_bytes = record_length - bytes_read_from_data;

        // Skip the rest of the record safely
        if remaining_bytes > 0 {
            reader
                .seek(SeekFrom::Current(remaining_bytes as i64))
                .map_err(|e| {
                    BinaryExportError::CorruptedData(format!(
                        "Failed to skip {remaining_bytes} remaining bytes at position {}: {e:?}",
                        current_position + 29,
                    ))
                })?;
        }

        Ok(RecordMetadata::new(ptr, size, timestamp))
    }
}

impl Default for BinaryIndexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for quick filter data
struct QuickFilterBuilder {
    batch_size: usize,
    bloom_filter_params: BloomFilterParams,
    current_batch: Vec<RecordMetadata>,
    ptr_ranges: Vec<(usize, usize)>,
    size_ranges: Vec<(usize, usize)>,
    timestamp_ranges: Vec<(u64, u64)>,
    thread_ids: Vec<String>,
    type_names: Vec<String>,
}

impl QuickFilterBuilder {
    fn new(batch_size: usize, bloom_filter_params: BloomFilterParams) -> Self {
        Self {
            batch_size,
            bloom_filter_params,
            current_batch: Vec::with_capacity(batch_size),
            ptr_ranges: Vec::new(),
            size_ranges: Vec::new(),
            timestamp_ranges: Vec::new(),
            thread_ids: Vec::new(),
            type_names: Vec::new(),
        }
    }

    fn add_record(&mut self, _record_index: usize, metadata: &RecordMetadata) {
        self.current_batch.push(metadata.clone());

        // Collect thread IDs and type names for bloom filters
        if let Some(ref thread_id) = metadata.thread_id {
            self.thread_ids.push(thread_id.clone());
        }
        if let Some(ref type_name) = metadata.type_name {
            self.type_names.push(type_name.clone());
        }

        // Process batch when full
        if self.current_batch.len() >= self.batch_size {
            self.process_batch();
        }
    }

    fn process_batch(&mut self) {
        if self.current_batch.is_empty() {
            return;
        }

        // Calculate ranges for this batch
        let ptr_min = self.current_batch.iter().map(|r| r.ptr).min().unwrap_or(0);
        let ptr_max = self.current_batch.iter().map(|r| r.ptr).max().unwrap_or(0);
        self.ptr_ranges.push((ptr_min, ptr_max));

        let size_min = self.current_batch.iter().map(|r| r.size).min().unwrap_or(0);
        let size_max = self.current_batch.iter().map(|r| r.size).max().unwrap_or(0);
        self.size_ranges.push((size_min, size_max));

        let timestamp_min = self
            .current_batch
            .iter()
            .map(|r| r.timestamp)
            .min()
            .unwrap_or(0);
        let timestamp_max = self
            .current_batch
            .iter()
            .map(|r| r.timestamp)
            .max()
            .unwrap_or(0);
        self.timestamp_ranges.push((timestamp_min, timestamp_max));

        self.current_batch.clear();
    }

    fn build(mut self) -> QuickFilterData {
        // Process any remaining records
        self.process_batch();

        // Build bloom filters (simplified implementation)
        let thread_bloom_filter = self.build_simple_bloom_filter(&self.thread_ids);
        let type_bloom_filter = self.build_simple_bloom_filter(&self.type_names);

        QuickFilterData {
            batch_size: self.batch_size,
            ptr_ranges: self.ptr_ranges,
            size_ranges: self.size_ranges,
            timestamp_ranges: self.timestamp_ranges,
            thread_bloom_filter,
            type_bloom_filter,
            bloom_filter_params: self.bloom_filter_params,
        }
    }

    /// Build a simple bloom filter from strings (simplified implementation)
    fn build_simple_bloom_filter(&self, strings: &[String]) -> Vec<u8> {
        // This is a simplified bloom filter implementation
        // In a real implementation, you would use a proper bloom filter library
        let filter_size = (self.bloom_filter_params.filter_size_bits / 8) as usize;
        let mut filter = vec![0u8; filter_size];

        for string in strings {
            let mut hasher = DefaultHasher::new();
            string.hash(&mut hasher);
            let hash = hasher.finish();

            // Set multiple bits based on hash functions
            for i in 0..self.bloom_filter_params.hash_functions {
                let bit_index = ((hash.wrapping_add(i as u64)) % (filter_size * 8) as u64) as usize;
                let byte_index = bit_index / 8;
                let bit_offset = bit_index % 8;

                if byte_index < filter.len() {
                    filter[byte_index] |= 1 << bit_offset;
                }
            }
        }

        filter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AllocationInfo;
    use crate::export::binary::writer::BinaryWriter;
    use tempfile::NamedTempFile;

    fn create_test_allocation() -> AllocationInfo {
        AllocationInfo {
            ptr: 0x1000,
            size: 1024,
            var_name: Some("test_var".to_string()),
            type_name: Some("i32".to_string()),
            scope_name: None,
            timestamp_alloc: 1234567890,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
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
        }
    }

    #[test]
    fn test_index_builder_creation() {
        let builder = BinaryIndexBuilder::new();
        assert_eq!(builder.quick_filter_threshold, 1000);
        assert_eq!(builder.quick_filter_batch_size, 1000);
    }

    #[test]
    fn test_index_builder_configuration() {
        let builder = BinaryIndexBuilder::new()
            .with_quick_filter_threshold(500)
            .with_quick_filter_batch_size(200);

        assert_eq!(builder.quick_filter_threshold, 500);
        assert_eq!(builder.quick_filter_batch_size, 200);
    }

    #[test]
    fn test_build_index_from_binary_file() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let test_allocations = vec![create_test_allocation(), {
            let mut alloc = create_test_allocation();
            alloc.ptr = 0x2000;
            alloc.size = 2048;
            alloc.timestamp_alloc = 1234567891;
            alloc
        }];

        // Write test data to binary file
        {
            let mut writer =
                BinaryWriter::new(temp_file.path()).expect("Failed to create temp file");
            writer
                .write_header(test_allocations.len() as u32)
                .expect("Failed to write header");
            for alloc in &test_allocations {
                writer
                    .write_allocation(alloc)
                    .expect("Failed to write allocation");
            }
            writer.finish().expect("Failed to finish writing");
        }

        // Build index from the binary file
        let builder = BinaryIndexBuilder::new();
        let index = builder
            .build_index(temp_file.path())
            .expect("Failed to create temp file");

        // Verify index properties
        assert_eq!(index.record_count(), 2);
        assert_eq!(index.header.total_count, 2);
        assert!(index.file_size > 0);
        assert!(index.file_hash != 0);

        // Verify record offsets
        assert!(index.get_record_offset(0).is_some());
        assert!(index.get_record_offset(1).is_some());
        assert!(index.get_record_offset(2).is_none());

        // Verify record sizes
        assert!(index.get_record_size(0).is_some());
        assert!(index.get_record_size(1).is_some());
        assert!(index.get_record_size(2).is_none());
    }

    #[test]
    fn test_file_hash_computation() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");

        // Write some test data
        std::fs::write(temp_file.path(), b"test data for hashing")
            .expect("Failed to create temp file");

        let builder = BinaryIndexBuilder::new();
        let hash1 = builder
            .compute_file_hash(temp_file.path())
            .expect("Failed to create temp file");
        let hash2 = builder
            .compute_file_hash(temp_file.path())
            .expect("Failed to create temp file");

        // Same file should produce same hash
        assert_eq!(hash1, hash2);

        // Different content should produce different hash
        std::fs::write(temp_file.path(), b"different test data")
            .expect("Failed to create temp file");
        let hash3 = builder
            .compute_file_hash(temp_file.path())
            .expect("Failed to create temp file");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_quick_filter_builder() {
        let mut builder = QuickFilterBuilder::new(2, BloomFilterParams::default());

        let metadata1 = RecordMetadata::new(0x1000, 1024, 1000)
            .with_thread_id("main".to_string())
            .with_type_name("Vec<u8>".to_string());

        let metadata2 = RecordMetadata::new(0x2000, 2048, 2000)
            .with_thread_id("worker".to_string())
            .with_type_name("String".to_string());

        builder.add_record(0, &metadata1);
        builder.add_record(1, &metadata2);

        let quick_filter = builder.build();

        assert_eq!(quick_filter.batch_size, 2);
        assert_eq!(quick_filter.ptr_ranges.len(), 1);
        assert_eq!(quick_filter.size_ranges.len(), 1);
        assert_eq!(quick_filter.timestamp_ranges.len(), 1);

        // Check ranges
        assert_eq!(quick_filter.ptr_ranges[0], (0x1000, 0x2000));
        assert_eq!(quick_filter.size_ranges[0], (1024, 2048));
        assert_eq!(quick_filter.timestamp_ranges[0], (1000, 2000));

        // Check bloom filters are created
        assert!(!quick_filter.thread_bloom_filter.is_empty());
        assert!(!quick_filter.type_bloom_filter.is_empty());
    }
}
