//! Binary data reader for parsing allocation records from binary files

use crate::core::types::AllocationInfo;
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::format::{
    AdvancedMetricsHeader, FileHeader, MetricsBitmapFlags, ALLOCATION_RECORD_TYPE, HEADER_SIZE,
};
use crate::export::binary::serializable::{primitives, BinarySerializable};
use crate::export::binary::string_table::StringTable;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// Binary reader for allocation records using buffered I/O
pub struct BinaryReader {
    reader: BufReader<File>,
    advanced_metrics: Option<AdvancedMetricsData>,
    string_table: Option<StringTable>,
    file_version: Option<u32>,
}

/// Container for advanced metrics data read from binary file
#[derive(Debug, Clone)]
pub struct AdvancedMetricsData {
    pub lifecycle_metrics: HashMap<u64, LifecycleMetric>,
    pub container_metrics: HashMap<u64, String>, // JSON for now
    pub type_usage_metrics: HashMap<u64, String>, // JSON for now
}

/// Lifecycle metric data
#[derive(Debug, Clone)]
pub struct LifecycleMetric {
    pub lifetime_ms: u64,
    pub lifecycle_tracking: Option<String>, // JSON for now
}

impl BinaryReader {
    /// Create new binary reader for the specified file path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, BinaryExportError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        Ok(Self {
            reader,
            advanced_metrics: None,
            string_table: None,
            file_version: None,
        })
    }

    /// Read and validate file header
    pub fn read_header(&mut self) -> Result<FileHeader, BinaryExportError> {
        let mut header_bytes = [0u8; HEADER_SIZE];
        self.reader.read_exact(&mut header_bytes)?;

        let header = FileHeader::from_bytes(&header_bytes);

        // Validate magic bytes
        if !header.is_valid_magic() {
            let expected = String::from_utf8_lossy(&header.magic);
            let actual = String::from_utf8_lossy(b"MEMSCOPE");
            return Err(BinaryExportError::InvalidMagic {
                expected: expected.to_string(),
                actual: actual.to_string(),
            });
        }

        // Check version compatibility
        if !header.is_compatible_version() {
            return Err(BinaryExportError::UnsupportedVersion(header.version));
        }

        // Store version for version-specific handling
        self.file_version = Some(header.get_version());

        // Read string table after header
        self.read_string_table()?;

        Ok(header)
    }

    /// Read string table from file if present
    fn read_string_table(&mut self) -> Result<(), BinaryExportError> {
        // Read string table marker (4 bytes)
        let mut marker = [0u8; 4];
        self.reader.read_exact(&mut marker)?;

        // Read table size (4 bytes)
        let table_size = primitives::read_u32(&mut self.reader)?;

        if &marker == b"STBL" && table_size > 0 {
            // Read compression flag (new format)
            let use_compressed_indices = primitives::read_u8(&mut self.reader)? != 0;

            // Read string table
            let string_count = primitives::read_u32(&mut self.reader)? as usize;
            let mut strings = Vec::with_capacity(string_count);

            for _ in 0..string_count {
                let string = primitives::read_string(&mut self.reader)?;
                strings.push(string);
            }

            // Create string table from read strings with compression setting
            let mut string_table = StringTable::with_compression(use_compressed_indices);
            for string in strings {
                string_table.add_string(&string)?;
            }
            self.string_table = Some(string_table);
        } else if &marker == b"NONE" {
            // No string table present
            self.string_table = None;
        } else {
            return Err(BinaryExportError::CorruptedData(
                "Invalid string table marker".to_string(),
            ));
        }

        Ok(())
    }

    /// Read single allocation record from current position
    pub fn read_allocation(&mut self) -> Result<AllocationInfo, BinaryExportError> {
        let file_version = self.file_version.unwrap_or(1);

        if file_version == 1 {
            self.read_allocation_v1()
        } else {
            self.read_allocation_v2()
        }
    }

    /// Read allocation record for version 1 format (legacy - basic fields only)
    fn read_allocation_v1(&mut self) -> Result<AllocationInfo, BinaryExportError> {
        // Read Type (1 byte)
        let mut type_byte = [0u8; 1];
        self.reader.read_exact(&mut type_byte)?;

        if type_byte[0] != ALLOCATION_RECORD_TYPE {
            return Err(BinaryExportError::CorruptedData(format!(
                "Invalid record type: {}",
                type_byte[0]
            )));
        }

        // Read Length (4 bytes)
        let mut length_bytes = [0u8; 4];
        self.reader.read_exact(&mut length_bytes)?;
        let _record_length = u32::from_le_bytes(length_bytes);

        // Read basic fields only for v1
        let ptr = self.read_u64()? as usize;
        let size = self.read_u64()? as usize;
        let timestamp_alloc = self.read_u64()?;

        // Read optional timestamp_dealloc
        let timestamp_dealloc = if self.read_u8()? == 1 {
            Some(self.read_u64()?)
        } else {
            None
        };

        // Read basic string fields
        let var_name = self.read_optional_string()?;
        let type_name = self.read_optional_string()?;
        let scope_name = self.read_optional_string()?;
        let thread_id = self.read_string()?;

        // Read stack trace
        let stack_trace = self.read_optional_string_vec()?;

        // Read basic numeric fields
        let borrow_count = self.read_u32()? as usize;
        let is_leaked_byte = self.read_u8()?;
        let is_leaked = is_leaked_byte != 0;

        // Read optional lifetime_ms
        let lifetime_flag = self.read_u8()?;
        let lifetime_ms = if lifetime_flag == 1 {
            Some(self.read_u64()?)
        } else {
            None
        };

        // For v1, set all advanced fields to None
        Ok(AllocationInfo {
            ptr,
            size,
            var_name,
            type_name,
            scope_name,
            timestamp_alloc,
            timestamp_dealloc,
            thread_id,
            borrow_count,
            stack_trace,
            is_leaked,
            lifetime_ms,
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
        })
    }

    /// Read allocation record for version 2 format (current - with advanced metrics)
    fn read_allocation_v2(&mut self) -> Result<AllocationInfo, BinaryExportError> {
        // Read Type (1 byte)
        let mut type_byte = [0u8; 1];
        self.reader.read_exact(&mut type_byte)?;

        if type_byte[0] != ALLOCATION_RECORD_TYPE {
            return Err(BinaryExportError::CorruptedData(format!(
                "Invalid record type: {}",
                type_byte[0]
            )));
        }

        // Read Length (4 bytes)
        let mut length_bytes = [0u8; 4];
        self.reader.read_exact(&mut length_bytes)?;
        let _record_length = u32::from_le_bytes(length_bytes);

        // Read basic fields
        let ptr = self.read_u64()? as usize;
        let size = self.read_u64()? as usize;
        let timestamp_alloc = self.read_u64()?;

        // Read optional timestamp_dealloc
        let timestamp_dealloc = if self.read_u8()? == 1 {
            Some(self.read_u64()?)
        } else {
            None
        };

        // Read string fields
        let var_name = self.read_optional_string()?;
        let type_name = self.read_optional_string()?;
        let scope_name = self.read_optional_string()?;
        let thread_id = self.read_string()?;

        // Read stack trace
        let stack_trace = self.read_optional_string_vec()?;

        // Read numeric fields
        let borrow_count = self.read_u32()? as usize;
        let is_leaked_byte = self.read_u8()?;
        let is_leaked = is_leaked_byte != 0;

        // Read optional lifetime_ms
        let lifetime_flag = self.read_u8()?;
        let lifetime_ms = if lifetime_flag == 1 {
            Some(self.read_u64()?)
        } else {
            None
        };

        // Read advanced fields (v2 only)
        let smart_pointer_info = self.read_optional_binary_field()?;
        let memory_layout = self.read_optional_binary_field()?;

        let generic_info = self.read_optional_json_field()?;
        let dynamic_type_info = self.read_optional_json_field()?;
        let runtime_state = self.read_optional_json_field()?;
        let stack_allocation = self.read_optional_json_field()?;
        let temporary_object = self.read_optional_json_field()?;
        let fragmentation_analysis = self.read_optional_json_field()?;
        let generic_instantiation = self.read_optional_json_field()?;
        let type_relationships = self.read_optional_json_field()?;
        let type_usage = self.read_optional_json_field()?;
        let function_call_tracking = self.read_optional_json_field()?;
        let lifecycle_tracking = self.read_optional_json_field()?;
        let access_tracking = self.read_optional_json_field()?;

        Ok(AllocationInfo {
            ptr,
            size,
            var_name,
            type_name,
            scope_name,
            timestamp_alloc,
            timestamp_dealloc,
            thread_id,
            borrow_count,
            stack_trace,
            is_leaked,
            lifetime_ms,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
            smart_pointer_info,
            memory_layout,
            generic_info,
            dynamic_type_info,
            runtime_state,
            stack_allocation,
            temporary_object,
            fragmentation_analysis,
            generic_instantiation,
            type_relationships,
            type_usage,
            function_call_tracking,
            lifecycle_tracking,
            access_tracking,
            drop_chain_analysis: None,
        })
    }

    /// Read all allocation records from file with improved error handling
    pub fn read_all(&mut self) -> Result<Vec<AllocationInfo>, BinaryExportError> {
        let header = self.read_header()?;
        let mut allocations = Vec::with_capacity(header.total_count as usize);

        // Read allocations with better error handling
        for i in 0..header.total_count {
            match self.read_allocation() {
                Ok(allocation) => allocations.push(allocation),
                Err(BinaryExportError::Io(ref e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    // Handle partial reads gracefully
                    tracing::warn!(
                        "Reached end of file after reading {} of {} allocations",
                        i,
                        header.total_count
                    );
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        // Try to read advanced metrics segment if present (optional)
        if let Err(_) = self.try_read_advanced_metrics_segment() {
            // Advanced metrics segment not present or corrupted, continue without it
            tracing::debug!("No advanced metrics segment found or failed to read");
        }

        tracing::info!("Successfully read {} allocation records", allocations.len());
        Ok(allocations)
    }

    /// Try to read advanced metrics segment (backward compatible)
    fn try_read_advanced_metrics_segment(&mut self) -> Result<(), BinaryExportError> {
        // Try to read advanced metrics header with partial read handling
        let mut header_bytes = [0u8; 16];
        let mut bytes_read = 0;

        // Read as much as possible without failing on partial reads
        while bytes_read < 16 {
            match self.reader.read(&mut header_bytes[bytes_read..]) {
                Ok(0) => {
                    // End of file reached
                    if bytes_read == 0 {
                        // No advanced metrics segment at all
                        return Err(BinaryExportError::CorruptedData(
                            "No advanced metrics segment".to_string(),
                        ));
                    } else {
                        // Partial read, file is truncated
                        tracing::warn!(
                            "File appears to be truncated, only read {} of 16 header bytes",
                            bytes_read
                        );
                        return Err(BinaryExportError::CorruptedData(
                            "Truncated advanced metrics header".to_string(),
                        ));
                    }
                }
                Ok(n) => bytes_read += n,
                Err(e) => {
                    tracing::debug!("Failed to read advanced metrics header: {}", e);
                    return Err(BinaryExportError::Io(e));
                }
            }
        }

        let header = AdvancedMetricsHeader::from_bytes(&header_bytes);

        if header.is_valid_magic() {
            // Valid advanced metrics segment found
            self.read_advanced_metrics_data(header)?;
        } else {
            // Not an advanced metrics segment, seek back if possible
            if let Err(e) = self.reader.seek(SeekFrom::Current(-16)) {
                tracing::debug!("Failed to seek back: {}", e);
            }
            return Err(BinaryExportError::CorruptedData(
                "Invalid advanced metrics magic".to_string(),
            ));
        }

        Ok(())
    }

    /// Read advanced metrics data based on header
    fn read_advanced_metrics_data(
        &mut self,
        header: AdvancedMetricsHeader,
    ) -> Result<(), BinaryExportError> {
        let mut lifecycle_metrics = HashMap::new();
        let mut container_metrics = HashMap::new();
        let mut type_usage_metrics = HashMap::new();

        // Read lifecycle analysis data if enabled
        if MetricsBitmapFlags::is_enabled(
            header.metrics_bitmap,
            MetricsBitmapFlags::LifecycleAnalysis,
        ) {
            lifecycle_metrics = self.read_lifecycle_metrics()?;
        }

        // Read container analysis data if enabled
        if MetricsBitmapFlags::is_enabled(
            header.metrics_bitmap,
            MetricsBitmapFlags::ContainerAnalysis,
        ) {
            container_metrics = self.read_container_metrics()?;
        }

        // Read type usage statistics if enabled
        if MetricsBitmapFlags::is_enabled(header.metrics_bitmap, MetricsBitmapFlags::TypeUsageStats)
        {
            type_usage_metrics = self.read_type_usage_metrics()?;
        }

        // Store advanced metrics data
        self.advanced_metrics = Some(AdvancedMetricsData {
            lifecycle_metrics,
            container_metrics,
            type_usage_metrics,
        });

        Ok(())
    }

    /// Read lifecycle metrics from advanced segment
    fn read_lifecycle_metrics(
        &mut self,
    ) -> Result<HashMap<u64, LifecycleMetric>, BinaryExportError> {
        let count = self.read_u32()? as usize;
        let mut metrics = HashMap::with_capacity(count);

        for _ in 0..count {
            let ptr = self.read_u64()?;
            let lifetime_ms = self.read_u64()?;

            let lifecycle_tracking = if self.read_u8()? == 1 {
                Some(self.read_string()?)
            } else {
                None
            };

            metrics.insert(
                ptr,
                LifecycleMetric {
                    lifetime_ms,
                    lifecycle_tracking,
                },
            );
        }

        Ok(metrics)
    }

    /// Read container metrics from advanced segment
    fn read_container_metrics(&mut self) -> Result<HashMap<u64, String>, BinaryExportError> {
        let count = self.read_u32()? as usize;
        let mut metrics = HashMap::with_capacity(count);

        for _ in 0..count {
            let ptr = self.read_u64()?;
            let json_data = self.read_string()?;
            metrics.insert(ptr, json_data);
        }

        Ok(metrics)
    }

    /// Read type usage metrics from advanced segment
    fn read_type_usage_metrics(&mut self) -> Result<HashMap<u64, String>, BinaryExportError> {
        let count = self.read_u32()? as usize;
        let mut metrics = HashMap::with_capacity(count);

        for _ in 0..count {
            let ptr = self.read_u64()?;
            let json_data = self.read_string()?;
            metrics.insert(ptr, json_data);
        }

        Ok(metrics)
    }

    /// Get advanced metrics data if available
    pub fn get_advanced_metrics(&self) -> Option<&AdvancedMetricsData> {
        self.advanced_metrics.as_ref()
    }

    /// Read u64 value in Little Endian format
    fn read_u64(&mut self) -> Result<u64, BinaryExportError> {
        let mut bytes = [0u8; 8];
        self.reader.read_exact(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }

    /// Read u32 value in Little Endian format
    fn read_u32(&mut self) -> Result<u32, BinaryExportError> {
        let mut bytes = [0u8; 4];
        self.reader.read_exact(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }

    /// Read optional string with length prefix or string table reference
    fn read_optional_string(&mut self) -> Result<Option<String>, BinaryExportError> {
        let mut length_bytes = [0u8; 4];
        self.reader.read_exact(&mut length_bytes)?;
        let length = u32::from_le_bytes(length_bytes);

        if length == 0xFFFFFFFE {
            // None marker
            Ok(None)
        } else if length == 0xFFFF {
            // String table reference
            let index = primitives::read_u16(&mut self.reader)?;
            if let Some(ref string_table) = self.string_table {
                if let Some(string) = string_table.get_string(index) {
                    Ok(Some(string.to_string()))
                } else {
                    Err(BinaryExportError::CorruptedData(format!(
                        "Invalid string table index: {}",
                        index
                    )))
                }
            } else {
                Err(BinaryExportError::CorruptedData(
                    "String table reference found but no string table loaded".to_string(),
                ))
            }
        } else {
            // Inline string (including empty strings with length 0)
            let mut string_bytes = vec![0u8; length as usize];
            self.reader.read_exact(&mut string_bytes)?;

            let string = String::from_utf8(string_bytes).map_err(|_| {
                BinaryExportError::CorruptedData("Invalid UTF-8 string".to_string())
            })?;

            Ok(Some(string))
        }
    }

    /// Read string with length prefix or string table reference
    fn read_string(&mut self) -> Result<String, BinaryExportError> {
        let mut length_bytes = [0u8; 4];
        self.reader.read_exact(&mut length_bytes)?;
        let length = u32::from_le_bytes(length_bytes);

        if length == 0xFFFF {
            // String table reference
            let index = primitives::read_u16(&mut self.reader)?;
            if let Some(ref string_table) = self.string_table {
                if let Some(string) = string_table.get_string(index) {
                    Ok(string.to_string())
                } else {
                    Err(BinaryExportError::CorruptedData(format!(
                        "Invalid string table index: {}",
                        index
                    )))
                }
            } else {
                Err(BinaryExportError::CorruptedData(
                    "String table reference found but no string table loaded".to_string(),
                ))
            }
        } else {
            // Inline string
            let mut string_bytes = vec![0u8; length as usize];
            self.reader.read_exact(&mut string_bytes)?;

            String::from_utf8(string_bytes)
                .map_err(|_| BinaryExportError::CorruptedData("Invalid UTF-8 string".to_string()))
        }
    }

    /// Read an optional vector of strings
    fn read_optional_string_vec(&mut self) -> Result<Option<Vec<String>>, BinaryExportError> {
        let mut count_bytes = [0u8; 4];
        self.reader.read_exact(&mut count_bytes)?;
        let count = u32::from_le_bytes(count_bytes) as usize;

        if count == 0 {
            Ok(None)
        } else {
            let mut strings = Vec::with_capacity(count);
            for _ in 0..count {
                strings.push(self.read_string()?);
            }
            Ok(Some(strings))
        }
    }

    /// Read single byte
    fn read_u8(&mut self) -> Result<u8, BinaryExportError> {
        let mut buffer = [0u8; 1];
        self.reader.read_exact(&mut buffer)?;
        Ok(buffer[0])
    }

    /// Read optional binary field using BinarySerializable trait
    fn read_optional_binary_field<T: BinarySerializable>(
        &mut self,
    ) -> Result<Option<T>, BinaryExportError> {
        if self.read_u8()? == 1 {
            Ok(Some(T::read_binary(&mut self.reader)?))
        } else {
            Ok(None)
        }
    }

    /// Read optional JSON field
    fn read_optional_json_field<T: serde::de::DeserializeOwned>(
        &mut self,
    ) -> Result<Option<T>, BinaryExportError> {
        if self.read_u8()? == 1 {
            let json_str = self.read_string()?;
            let value = serde_json::from_str(&json_str).map_err(|e| {
                BinaryExportError::CorruptedData(format!("JSON deserialization failed: {}", e))
            })?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_reader_creation() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");

        // Create empty file first
        {
            let mut writer =
                BinaryWriter::new(temp_file.path()).expect("Failed to create temp file");
            writer.write_header(0).expect("Failed to write header");
            writer.finish().expect("Failed to finish writing");
        }

        let reader = BinaryReader::new(temp_file.path());
        assert!(reader.is_ok());
    }

    #[test]
    fn test_header_reading() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");

        // Write test data
        {
            let mut writer =
                BinaryWriter::new(temp_file.path()).expect("Failed to create temp file");
            writer.write_header(42).expect("Failed to write header");
            writer.finish().expect("Failed to finish writing");
        }

        // Read and verify
        let mut reader = BinaryReader::new(temp_file.path()).expect("Failed to create temp file");
        let header = reader
            .read_header()
            .expect("Failed to read from binary file");

        assert_eq!(header.total_count, 42);
        assert!(header.is_valid_magic());
        assert!(header.is_compatible_version());
    }

    #[test]
    fn test_allocation_round_trip() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let original_alloc = create_test_allocation();

        // Write test data
        {
            let mut writer =
                BinaryWriter::new(temp_file.path()).expect("Failed to create temp file");
            writer.write_header(1).expect("Failed to write header");
            writer
                .write_allocation(&original_alloc)
                .expect("Failed to write allocation");
            writer.finish().expect("Failed to finish writing");
        }

        // Read and verify
        let mut reader = BinaryReader::new(temp_file.path()).expect("Failed to create temp file");
        let allocations = reader.read_all().expect("Failed to read from binary file");

        assert_eq!(allocations.len(), 1);
        let read_alloc = &allocations[0];

        assert_eq!(read_alloc.ptr, original_alloc.ptr);
        assert_eq!(read_alloc.size, original_alloc.size);
        assert_eq!(read_alloc.timestamp_alloc, original_alloc.timestamp_alloc);
        assert_eq!(read_alloc.var_name, original_alloc.var_name);
        assert_eq!(read_alloc.type_name, original_alloc.type_name);
        assert_eq!(read_alloc.thread_id, original_alloc.thread_id);
    }

    #[test]
    fn test_advanced_metrics_round_trip() {
        use crate::export::binary::config::BinaryExportConfig;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let mut original_alloc = create_test_allocation();
        original_alloc.lifetime_ms = Some(2500); // Add lifecycle data

        // Write test data with advanced metrics
        {
            let config = BinaryExportConfig::debug_comprehensive();
            let mut writer = BinaryWriter::new_with_config(temp_file.path(), &config)
                .expect("Failed to create temp file");
            writer.write_header(1).expect("Failed to write header");
            writer
                .write_allocation(&original_alloc)
                .expect("Failed to write allocation");
            writer
                .write_advanced_metrics_segment(&[original_alloc.clone()])
                .expect("Test operation failed");
            writer.finish().expect("Failed to finish writing");
        }

        // Read and verify
        let mut reader = BinaryReader::new(temp_file.path()).expect("Failed to create temp file");
        let allocations = reader.read_all().expect("Failed to read from binary file");

        assert_eq!(allocations.len(), 1);
        let read_alloc = &allocations[0];

        // Verify basic data
        assert_eq!(read_alloc.ptr, original_alloc.ptr);
        assert_eq!(read_alloc.size, original_alloc.size);
        assert_eq!(read_alloc.lifetime_ms, original_alloc.lifetime_ms);

        // Verify advanced metrics were read
        let advanced_metrics = reader.get_advanced_metrics();
        assert!(advanced_metrics.is_some());

        let metrics = advanced_metrics.expect("Failed to get test value");
        assert!(metrics
            .lifecycle_metrics
            .contains_key(&(original_alloc.ptr as u64)));

        let lifecycle_metric = &metrics.lifecycle_metrics[&(original_alloc.ptr as u64)];
        assert_eq!(lifecycle_metric.lifetime_ms, 2500);
    }

    #[test]
    fn test_backward_compatibility() {
        use crate::export::binary::config::BinaryExportConfig;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let original_alloc = create_test_allocation();

        // Write test data without advanced metrics (old format)
        {
            let config = BinaryExportConfig::minimal();
            let mut writer = BinaryWriter::new_with_config(temp_file.path(), &config)
                .expect("Failed to create temp file");
            writer.write_header(1).expect("Failed to write header");
            writer
                .write_allocation(&original_alloc)
                .expect("Failed to write allocation");
            writer.finish().expect("Failed to finish writing");
        }

        // Read with new reader (should handle missing advanced metrics gracefully)
        let mut reader = BinaryReader::new(temp_file.path()).expect("Failed to create temp file");
        let allocations = reader.read_all().expect("Failed to read from binary file");

        assert_eq!(allocations.len(), 1);
        let read_alloc = &allocations[0];

        // Basic data should still be readable
        assert_eq!(read_alloc.ptr, original_alloc.ptr);
        assert_eq!(read_alloc.size, original_alloc.size);

        // Advanced metrics should be None (backward compatibility)
        let advanced_metrics = reader.get_advanced_metrics();
        assert!(advanced_metrics.is_none());
    }
}
