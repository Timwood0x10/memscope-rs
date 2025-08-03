//! Binary data writer for efficient allocation record serialization

use crate::core::types::AllocationInfo;
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::format::{FileHeader, ALLOCATION_RECORD_TYPE};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Binary writer for allocation records using buffered I/O
pub struct BinaryWriter {
    writer: BufWriter<File>,
}

impl BinaryWriter {
    /// Create new binary writer for the specified file path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, BinaryExportError> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        
        Ok(Self { writer })
    }
    
    /// Write file header with allocation count
    pub fn write_header(&mut self, count: u32) -> Result<(), BinaryExportError> {
        let header = FileHeader::new(count);
        let header_bytes = header.to_bytes();
        
        self.writer.write_all(&header_bytes)?;
        Ok(())
    }
    
    /// Write single allocation record in TLV format
    pub fn write_allocation(&mut self, alloc: &AllocationInfo) -> Result<(), BinaryExportError> {
        // Calculate value size (excluding Type and Length fields)
        let value_size = self.calculate_value_size(alloc);
        
        // Write Type (1 byte)
        self.writer.write_all(&[ALLOCATION_RECORD_TYPE])?;
        
        // Write Length (4 bytes, Little Endian) - only the Value part
        self.writer.write_all(&(value_size as u32).to_le_bytes())?;
        
        // Write Value: basic fields (ptr, size, timestamps)
        self.writer.write_all(&(alloc.ptr as u64).to_le_bytes())?;
        self.writer.write_all(&(alloc.size as u64).to_le_bytes())?;
        self.writer.write_all(&alloc.timestamp_alloc.to_le_bytes())?;
        
        // Write optional timestamp_dealloc
        match alloc.timestamp_dealloc {
            Some(ts) => {
                self.writer.write_all(&1u8.to_le_bytes())?; // has value
                self.writer.write_all(&ts.to_le_bytes())?;
            }
            None => {
                self.writer.write_all(&0u8.to_le_bytes())?; // no value
            }
        }
        
        // Write string fields
        self.write_optional_string(&alloc.var_name)?;
        self.write_optional_string(&alloc.type_name)?;
        self.write_optional_string(&alloc.scope_name)?;
        self.write_string(&alloc.thread_id)?;
        
        // Write stack trace
        self.write_optional_string_vec(&alloc.stack_trace)?;
        
        // Write numeric fields
        self.writer.write_all(&(alloc.borrow_count as u32).to_le_bytes())?;
        self.writer.write_all(&(alloc.is_leaked as u8).to_le_bytes())?;
        
        // Write optional lifetime_ms
        match alloc.lifetime_ms {
            Some(ms) => {
                self.writer.write_all(&1u8.to_le_bytes())?; // has value
                self.writer.write_all(&ms.to_le_bytes())?;
            }
            None => {
                self.writer.write_all(&0u8.to_le_bytes())?; // no value
            }
        }
        
        // Write complex JSON fields
        self.write_optional_json_field(&alloc.smart_pointer_info)?;
        self.write_optional_json_field(&alloc.memory_layout)?;
        self.write_optional_json_field(&alloc.generic_info)?;
        self.write_optional_json_field(&alloc.dynamic_type_info)?;
        self.write_optional_json_field(&alloc.runtime_state)?;
        self.write_optional_json_field(&alloc.stack_allocation)?;
        self.write_optional_json_field(&alloc.temporary_object)?;
        self.write_optional_json_field(&alloc.fragmentation_analysis)?;
        self.write_optional_json_field(&alloc.generic_instantiation)?;
        self.write_optional_json_field(&alloc.type_relationships)?;
        self.write_optional_json_field(&alloc.type_usage)?;
        self.write_optional_json_field(&alloc.function_call_tracking)?;
        self.write_optional_json_field(&alloc.lifecycle_tracking)?;
        self.write_optional_json_field(&alloc.access_tracking)?;
        
        Ok(())
    }
    
    /// Finish writing and flush all data to disk
    pub fn finish(mut self) -> Result<(), BinaryExportError> {
        self.writer.flush()?;
        Ok(())
    }
    
    /// Calculate size needed for the Value part of TLV (excluding Type and Length)
    fn calculate_value_size(&self, alloc: &AllocationInfo) -> usize {
        let mut size = 8 + 8 + 8; // ptr + size + timestamp_alloc
        
        // timestamp_dealloc: 1 byte flag + optional 8 bytes
        size += 1;
        if alloc.timestamp_dealloc.is_some() {
            size += 8;
        }
        
        // String fields: length (4 bytes) + content
        size += 4; // var_name length
        if let Some(ref name) = alloc.var_name {
            size += name.len();
        }
        
        size += 4; // type_name length
        if let Some(ref name) = alloc.type_name {
            size += name.len();
        }
        
        size += 4; // scope_name length
        if let Some(ref name) = alloc.scope_name {
            size += name.len();
        }
        
        size += 4 + alloc.thread_id.len(); // thread_id
        
        // Stack trace
        size += 4; // stack_trace count
        if let Some(ref stack_trace) = alloc.stack_trace {
            for frame in stack_trace {
                size += 4 + frame.len(); // length + content for each frame
            }
        }
        
        // Numeric fields
        size += 4; // borrow_count
        size += 1; // is_leaked
        
        // lifetime_ms: 1 byte flag + optional 8 bytes
        size += 1;
        if alloc.lifetime_ms.is_some() {
            size += 8;
        }
        
        // JSON fields
        size += self.calculate_json_field_size(&alloc.smart_pointer_info);
        size += self.calculate_json_field_size(&alloc.memory_layout);
        size += self.calculate_json_field_size(&alloc.generic_info);
        size += self.calculate_json_field_size(&alloc.dynamic_type_info);
        size += self.calculate_json_field_size(&alloc.runtime_state);
        size += self.calculate_json_field_size(&alloc.stack_allocation);
        size += self.calculate_json_field_size(&alloc.temporary_object);
        size += self.calculate_json_field_size(&alloc.fragmentation_analysis);
        size += self.calculate_json_field_size(&alloc.generic_instantiation);
        size += self.calculate_json_field_size(&alloc.type_relationships);
        size += self.calculate_json_field_size(&alloc.type_usage);
        size += self.calculate_json_field_size(&alloc.function_call_tracking);
        size += self.calculate_json_field_size(&alloc.lifecycle_tracking);
        size += self.calculate_json_field_size(&alloc.access_tracking);
        
        size
    }
    
    /// Calculate size needed for optional JSON field
    fn calculate_json_field_size<T: serde::Serialize>(&self, field: &Option<T>) -> usize {
        let mut size = 1; // flag byte
        if let Some(value) = field {
            // Pre-serialize to get exact size
            if let Ok(json_str) = serde_json::to_string(value) {
                size += 4 + json_str.len(); // length + content
            } else {
                // If serialization fails, just account for the flag
                size = 1;
            }
        }
        size
    }
    
    /// Write optional string field with length prefix
    fn write_optional_string(&mut self, opt_str: &Option<String>) -> Result<(), BinaryExportError> {
        match opt_str {
            Some(s) => {
                self.writer.write_all(&(s.len() as u32).to_le_bytes())?;
                self.writer.write_all(s.as_bytes())?;
            }
            None => {
                self.writer.write_all(&0u32.to_le_bytes())?;
            }
        }
        Ok(())
    }
    
    /// Write string field with length prefix
    fn write_string(&mut self, s: &str) -> Result<(), BinaryExportError> {
        self.writer.write_all(&(s.len() as u32).to_le_bytes())?;
        self.writer.write_all(s.as_bytes())?;
        Ok(())
    }
    
    /// Write an optional vector of strings
    fn write_optional_string_vec(&mut self, vec: &Option<Vec<String>>) -> Result<(), BinaryExportError> {
        match vec {
            Some(strings) => {
                // Write count
                self.writer.write_all(&(strings.len() as u32).to_le_bytes())?;
                // Write each string
                for string in strings {
                    self.writer.write_all(&(string.len() as u32).to_le_bytes())?;
                    self.writer.write_all(string.as_bytes())?;
                }
            }
            None => {
                self.writer.write_all(&0u32.to_le_bytes())?; // 0 count indicates None
            }
        }
        Ok(())
    }
    
    /// Write optional JSON field (serialize to JSON string)
    fn write_optional_json_field<T: serde::Serialize>(&mut self, field: &Option<T>) -> Result<(), BinaryExportError> {
        match field {
            Some(value) => {
                let json_str = serde_json::to_string(value)
                    .map_err(|e| BinaryExportError::CorruptedData(format!("JSON serialization failed: {}", e)))?;
                self.writer.write_all(&1u8.to_le_bytes())?; // has value
                self.write_string(&json_str)?;
            }
            None => {
                self.writer.write_all(&0u8.to_le_bytes())?; // no value
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
        }
    }
    
    #[test]
    fn test_writer_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let writer = BinaryWriter::new(temp_file.path());
        assert!(writer.is_ok());
    }
    
    #[test]
    fn test_header_writing() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut writer = BinaryWriter::new(temp_file.path()).unwrap();
        
        let result = writer.write_header(42);
        assert!(result.is_ok());
        
        writer.finish().unwrap();
        
        // Verify file size is at least header size
        let metadata = fs::metadata(temp_file.path()).unwrap();
        assert!(metadata.len() >= 16);
    }
    
    #[test]
    fn test_allocation_writing() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut writer = BinaryWriter::new(temp_file.path()).unwrap();
        
        writer.write_header(1).unwrap();
        
        let alloc = create_test_allocation();
        let result = writer.write_allocation(&alloc);
        assert!(result.is_ok());
        
        writer.finish().unwrap();
        
        // Verify file has content beyond header
        let metadata = fs::metadata(temp_file.path()).unwrap();
        assert!(metadata.len() > 16);
    }
    
    #[test]
    fn test_record_size_calculation() {
        let temp_file = NamedTempFile::new().unwrap();
        let writer = BinaryWriter::new(temp_file.path()).unwrap();
        
        let alloc = create_test_allocation();
        let size = writer.calculate_record_size(&alloc);
        
        // Basic fields: 8 + 8 + 8 = 24
        // var_name: 4 + 8 = 12
        // type_name: 4 + 3 = 7  
        // thread_id: 4 + 4 = 8
        // Total: 24 + 12 + 7 + 8 = 51
        assert_eq!(size, 51);
    }
}