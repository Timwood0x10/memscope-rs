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
        // Calculate record size
        let record_size = self.calculate_record_size(alloc);
        
        // Write Type (1 byte)
        self.writer.write_all(&[ALLOCATION_RECORD_TYPE])?;
        
        // Write Length (4 bytes, Little Endian)
        self.writer.write_all(&(record_size as u32).to_le_bytes())?;
        
        // Write Value: basic fields (ptr, size, timestamp)
        self.writer.write_all(&(alloc.ptr as u64).to_le_bytes())?;
        self.writer.write_all(&(alloc.size as u64).to_le_bytes())?;
        self.writer.write_all(&alloc.timestamp_alloc.to_le_bytes())?;
        
        // Write optional variable name
        self.write_optional_string(&alloc.var_name)?;
        
        // Write optional type name
        self.write_optional_string(&alloc.type_name)?;
        
        // Write thread ID
        self.write_string(&alloc.thread_id)?;
        
        Ok(())
    }
    
    /// Finish writing and flush all data to disk
    pub fn finish(mut self) -> Result<(), BinaryExportError> {
        self.writer.flush()?;
        Ok(())
    }
    
    /// Calculate total size needed for allocation record
    fn calculate_record_size(&self, alloc: &AllocationInfo) -> usize {
        let mut size = 8 + 8 + 8; // ptr + size + timestamp
        
        // Variable name: length (4 bytes) + content
        size += 4;
        if let Some(ref name) = alloc.var_name {
            size += name.len();
        }
        
        // Type name: length (4 bytes) + content
        size += 4;
        if let Some(ref name) = alloc.type_name {
            size += name.len();
        }
        
        // Thread ID: length (4 bytes) + content
        size += 4 + alloc.thread_id.len();
        
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