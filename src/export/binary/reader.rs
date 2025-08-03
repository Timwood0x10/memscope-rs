//! Binary data reader for parsing allocation records from binary files

use crate::core::types::AllocationInfo;
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::format::{FileHeader, ALLOCATION_RECORD_TYPE, HEADER_SIZE};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Binary reader for allocation records using buffered I/O
pub struct BinaryReader {
    reader: BufReader<File>,
}

impl BinaryReader {
    /// Create new binary reader for the specified file path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, BinaryExportError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        Ok(Self { reader })
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
        
        Ok(header)
    }
    
    /// Read single allocation record from current position
    pub fn read_allocation(&mut self) -> Result<AllocationInfo, BinaryExportError> {
        // Read Type (1 byte)
        let mut type_byte = [0u8; 1];
        self.reader.read_exact(&mut type_byte)?;
        
        if type_byte[0] != ALLOCATION_RECORD_TYPE {
            return Err(BinaryExportError::CorruptedData(
                format!("Invalid record type: {}", type_byte[0])
            ));
        }
        
        // Read Length (4 bytes)
        let mut length_bytes = [0u8; 4];
        self.reader.read_exact(&mut length_bytes)?;
        let record_length = u32::from_le_bytes(length_bytes);
        
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
        
        // Read JSON fields
        // stack_trace is already read above
        let smart_pointer_info = self.read_optional_json_field()?;
        let memory_layout = self.read_optional_json_field()?;
        

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
        })
    }
    
    /// Read all allocation records from file
    pub fn read_all(&mut self) -> Result<Vec<AllocationInfo>, BinaryExportError> {
        let header = self.read_header()?;
        let mut allocations = Vec::with_capacity(header.count as usize);
        
        for _ in 0..header.count {
            let allocation = self.read_allocation()?;
            allocations.push(allocation);
        }
        
        Ok(allocations)
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
    
    /// Read optional string with length prefix
    fn read_optional_string(&mut self) -> Result<Option<String>, BinaryExportError> {
        let mut length_bytes = [0u8; 4];
        self.reader.read_exact(&mut length_bytes)?;
        let length = u32::from_le_bytes(length_bytes) as usize;
        
        if length == 0 {
            Ok(None)
        } else {
            let mut string_bytes = vec![0u8; length];
            self.reader.read_exact(&mut string_bytes)?;
            
            let string = String::from_utf8(string_bytes)
                .map_err(|_| BinaryExportError::CorruptedData("Invalid UTF-8 string".to_string()))?;
            
            Ok(Some(string))
        }
    }
    
    /// Read string with length prefix
    fn read_string(&mut self) -> Result<String, BinaryExportError> {
        let mut length_bytes = [0u8; 4];
        self.reader.read_exact(&mut length_bytes)?;
        let length = u32::from_le_bytes(length_bytes) as usize;
        
        let mut string_bytes = vec![0u8; length];
        self.reader.read_exact(&mut string_bytes)?;
        
        String::from_utf8(string_bytes)
            .map_err(|_| BinaryExportError::CorruptedData("Invalid UTF-8 string".to_string()))
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
    
    /// Read optional JSON field
    fn read_optional_json_field<T: serde::de::DeserializeOwned>(&mut self) -> Result<Option<T>, BinaryExportError> {
        if self.read_u8()? == 1 {
            let json_str = self.read_string()?;
            let value = serde_json::from_str(&json_str)
                .map_err(|e| BinaryExportError::CorruptedData(format!("JSON deserialization failed: {}", e)))?;
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
    fn test_reader_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        
        // Create empty file first
        {
            let mut writer = BinaryWriter::new(temp_file.path()).unwrap();
            writer.write_header(0).unwrap();
            writer.finish().unwrap();
        }
        
        let reader = BinaryReader::new(temp_file.path());
        assert!(reader.is_ok());
    }
    
    #[test]
    fn test_header_reading() {
        let temp_file = NamedTempFile::new().unwrap();
        
        // Write test data
        {
            let mut writer = BinaryWriter::new(temp_file.path()).unwrap();
            writer.write_header(42).unwrap();
            writer.finish().unwrap();
        }
        
        // Read and verify
        let mut reader = BinaryReader::new(temp_file.path()).unwrap();
        let header = reader.read_header().unwrap();
        
        assert_eq!(header.count, 42);
        assert!(header.is_valid_magic());
        assert!(header.is_compatible_version());
    }
    
    #[test]
    fn test_allocation_round_trip() {
        let temp_file = NamedTempFile::new().unwrap();
        let original_alloc = create_test_allocation();
        
        // Write test data
        {
            let mut writer = BinaryWriter::new(temp_file.path()).unwrap();
            writer.write_header(1).unwrap();
            writer.write_allocation(&original_alloc).unwrap();
            writer.finish().unwrap();
        }
        
        // Read and verify
        let mut reader = BinaryReader::new(temp_file.path()).unwrap();
        let allocations = reader.read_all().unwrap();
        
        assert_eq!(allocations.len(), 1);
        let read_alloc = &allocations[0];
        
        assert_eq!(read_alloc.ptr, original_alloc.ptr);
        assert_eq!(read_alloc.size, original_alloc.size);
        assert_eq!(read_alloc.timestamp_alloc, original_alloc.timestamp_alloc);
        assert_eq!(read_alloc.var_name, original_alloc.var_name);
        assert_eq!(read_alloc.type_name, original_alloc.type_name);
        assert_eq!(read_alloc.thread_id, original_alloc.thread_id);
    }
}