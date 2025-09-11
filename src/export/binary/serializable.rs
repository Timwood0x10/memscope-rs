//! Binary serialization trait and implementations for efficient data encoding

use crate::export::binary::error::BinaryExportError;
use std::io::{Read, Write};

/// Trait for types that can be serialized to/from binary format efficiently
pub trait BinarySerializable: Sized {
    /// Write the object to binary format
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError>;

    /// Read the object from binary format
    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError>;

    /// Calculate the size needed for binary serialization
    fn binary_size(&self) -> usize;
}

/// Helper functions for writing primitive types
pub mod primitives {
    use super::*;

    /// Write a u8 value
    pub fn write_u8<W: Write>(writer: &mut W, value: u8) -> Result<usize, BinaryExportError> {
        writer.write_all(&[value])?;
        Ok(1)
    }

    /// Read a u8 value
    pub fn read_u8<R: Read>(reader: &mut R) -> Result<u8, BinaryExportError> {
        let mut buffer = [0u8; 1];
        reader.read_exact(&mut buffer)?;
        Ok(buffer[0])
    }

    /// Write a u16 value in little endian
    pub fn write_u16<W: Write>(writer: &mut W, value: u16) -> Result<usize, BinaryExportError> {
        writer.write_all(&value.to_le_bytes())?;
        Ok(2)
    }

    /// Write a u32 value in little endian
    pub fn write_u32<W: Write>(writer: &mut W, value: u32) -> Result<usize, BinaryExportError> {
        writer.write_all(&value.to_le_bytes())?;
        Ok(4)
    }

    /// Read a u16 value in little endian
    pub fn read_u16<R: Read>(reader: &mut R) -> Result<u16, BinaryExportError> {
        let mut buffer = [0u8; 2];
        reader.read_exact(&mut buffer)?;
        Ok(u16::from_le_bytes(buffer))
    }

    /// Read a u32 value in little endian
    pub fn read_u32<R: Read>(reader: &mut R) -> Result<u32, BinaryExportError> {
        let mut buffer = [0u8; 4];
        reader.read_exact(&mut buffer)?;
        Ok(u32::from_le_bytes(buffer))
    }

    /// Write a u64 value in little endian
    pub fn write_u64<W: Write>(writer: &mut W, value: u64) -> Result<usize, BinaryExportError> {
        writer.write_all(&value.to_le_bytes())?;
        Ok(8)
    }

    /// Read a u64 value in little endian
    pub fn read_u64<R: Read>(reader: &mut R) -> Result<u64, BinaryExportError> {
        let mut buffer = [0u8; 8];
        reader.read_exact(&mut buffer)?;
        Ok(u64::from_le_bytes(buffer))
    }

    /// Write a usize value as u64 in little endian
    pub fn write_usize<W: Write>(writer: &mut W, value: usize) -> Result<usize, BinaryExportError> {
        write_u64(writer, value as u64)
    }

    /// Read a usize value from u64 in little endian
    pub fn read_usize<R: Read>(reader: &mut R) -> Result<usize, BinaryExportError> {
        let value = read_u64(reader)?;
        Ok(value as usize)
    }

    /// Write a f32 value in little endian
    pub fn write_f32<W: Write>(writer: &mut W, value: f32) -> Result<usize, BinaryExportError> {
        writer.write_all(&value.to_le_bytes())?;
        Ok(4)
    }

    /// Read a f32 value in little endian
    pub fn read_f32<R: Read>(reader: &mut R) -> Result<f32, BinaryExportError> {
        let mut buffer = [0u8; 4];
        reader.read_exact(&mut buffer)?;
        Ok(f32::from_le_bytes(buffer))
    }

    /// Write a f64 value in little endian
    pub fn write_f64<W: Write>(writer: &mut W, value: f64) -> Result<usize, BinaryExportError> {
        writer.write_all(&value.to_le_bytes())?;
        Ok(8)
    }

    /// Read a f64 value in little endian
    pub fn read_f64<R: Read>(reader: &mut R) -> Result<f64, BinaryExportError> {
        let mut buffer = [0u8; 8];
        reader.read_exact(&mut buffer)?;
        Ok(f64::from_le_bytes(buffer))
    }

    /// Write a string with length prefix
    pub fn write_string<W: Write>(writer: &mut W, value: &str) -> Result<usize, BinaryExportError> {
        let mut size = write_u32(writer, value.len() as u32)?;
        writer.write_all(value.as_bytes())?;
        size += value.len();
        Ok(size)
    }

    /// Read a string with length prefix
    pub fn read_string<R: Read>(reader: &mut R) -> Result<String, BinaryExportError> {
        let length = read_u32(reader)? as usize;
        let mut buffer = vec![0u8; length];
        reader.read_exact(&mut buffer)?;
        String::from_utf8(buffer)
            .map_err(|_| BinaryExportError::CorruptedData("Invalid UTF-8 string".to_string()))
    }

    /// Write an optional value
    pub fn write_option<W: Write, T: BinarySerializable>(
        writer: &mut W,
        value: &Option<T>,
    ) -> Result<usize, BinaryExportError> {
        match value {
            Some(v) => {
                let mut size = write_u8(writer, 1)?; // has value
                size += v.write_binary(writer)?;
                Ok(size)
            }
            None => {
                write_u8(writer, 0) // no value
            }
        }
    }

    /// Read an optional value
    pub fn read_option<R: Read, T: BinarySerializable>(
        reader: &mut R,
    ) -> Result<Option<T>, BinaryExportError> {
        let has_value = read_u8(reader)?;
        if has_value == 1 {
            Ok(Some(T::read_binary(reader)?))
        } else {
            Ok(None)
        }
    }

    /// Write an optional string
    pub fn write_string_option<W: Write>(
        writer: &mut W,
        value: &Option<String>,
    ) -> Result<usize, BinaryExportError> {
        match value {
            Some(s) => {
                let mut size = write_u8(writer, 1)?; // has value
                size += write_string(writer, s)?;
                Ok(size)
            }
            None => {
                write_u8(writer, 0) // no value
            }
        }
    }

    /// Read an optional string
    pub fn read_string_option<R: Read>(
        reader: &mut R,
    ) -> Result<Option<String>, BinaryExportError> {
        let has_value = read_u8(reader)?;
        if has_value == 1 {
            Ok(Some(read_string(reader)?))
        } else {
            Ok(None)
        }
    }

    /// Write a vector with length prefix
    pub fn write_vec<W: Write, T: BinarySerializable>(
        writer: &mut W,
        value: &[T],
    ) -> Result<usize, BinaryExportError> {
        let mut size = write_u32(writer, value.len() as u32)?;
        for item in value {
            size += item.write_binary(writer)?;
        }
        Ok(size)
    }

    /// Read a vector with length prefix
    pub fn read_vec<R: Read, T: BinarySerializable>(
        reader: &mut R,
    ) -> Result<Vec<T>, BinaryExportError> {
        let length = read_u32(reader)? as usize;
        let mut vec = Vec::with_capacity(length);
        for _ in 0..length {
            vec.push(T::read_binary(reader)?);
        }
        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_primitive_serialization() {
        let mut buffer = Vec::new();

        // Test u32
        primitives::write_u32(&mut buffer, 0x12345678).expect("Test operation failed");
        let mut cursor = Cursor::new(&buffer);
        let value = primitives::read_u32(&mut cursor).expect("Failed to get test value");
        assert_eq!(value, 0x12345678);

        // Test string
        buffer.clear();
        primitives::write_string(&mut buffer, "test").expect("Test operation failed");
        let mut cursor = Cursor::new(&buffer);
        let string = primitives::read_string(&mut cursor).expect("Failed to get test value");
        assert_eq!(string, "test");
    }

    #[test]
    fn test_option_serialization() {
        // Mock implementation for testing
        struct TestStruct(u32);

        impl BinarySerializable for TestStruct {
            fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
                primitives::write_u32(writer, self.0)
            }

            fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
                Ok(TestStruct(primitives::read_u32(reader)?))
            }

            fn binary_size(&self) -> usize {
                4
            }
        }

        let mut buffer = Vec::new();

        // Test Some value
        let some_value = Some(TestStruct(42));
        primitives::write_option(&mut buffer, &some_value).expect("Failed to get test value");

        let mut cursor = Cursor::new(&buffer);
        let read_value: Option<TestStruct> =
            primitives::read_option(&mut cursor).expect("Test operation failed");
        assert!(read_value.is_some());
        assert_eq!(read_value.expect("Test operation failed").0, 42);

        // Test None value
        buffer.clear();
        let none_value: Option<TestStruct> = None;
        primitives::write_option(&mut buffer, &none_value).expect("Test operation failed");

        let mut cursor = Cursor::new(&buffer);
        let read_value: Option<TestStruct> =
            primitives::read_option(&mut cursor).expect("Test operation failed");
        assert!(read_value.is_none());
    }

    #[test]
    fn test_all_primitive_types() {
        let mut buffer = Vec::new();

        // Test u8
        primitives::write_u8(&mut buffer, 255).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(primitives::read_u8(&mut cursor).unwrap(), 255);

        // Test u16
        buffer.clear();
        primitives::write_u16(&mut buffer, 0xABCD).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(primitives::read_u16(&mut cursor).unwrap(), 0xABCD);

        // Test u64
        buffer.clear();
        primitives::write_u64(&mut buffer, 0x123456789ABCDEF0).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(
            primitives::read_u64(&mut cursor).unwrap(),
            0x123456789ABCDEF0
        );

        // Test usize
        buffer.clear();
        primitives::write_usize(&mut buffer, 12345).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(primitives::read_usize(&mut cursor).unwrap(), 12345);

        // Test f32
        buffer.clear();
        primitives::write_f32(&mut buffer, 3.14159).unwrap();
        let mut cursor = Cursor::new(&buffer);
        let read_f32 = primitives::read_f32(&mut cursor).unwrap();
        assert!((read_f32 - 3.14159).abs() < 0.0001);

        // Test f64
        buffer.clear();
        primitives::write_f64(&mut buffer, 2.718281828459045).unwrap();
        let mut cursor = Cursor::new(&buffer);
        let read_f64 = primitives::read_f64(&mut cursor).unwrap();
        assert!((read_f64 - 2.718281828459045).abs() < 0.000000000001);
    }

    #[test]
    fn test_string_serialization() {
        let mut buffer = Vec::new();

        // Test empty string
        primitives::write_string(&mut buffer, "").unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(primitives::read_string(&mut cursor).unwrap(), "");

        // Test unicode string
        buffer.clear();
        let unicode_str = "Hello 世界 🦀";
        primitives::write_string(&mut buffer, unicode_str).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(primitives::read_string(&mut cursor).unwrap(), unicode_str);

        // Test long string
        buffer.clear();
        let long_str = "a".repeat(1000);
        primitives::write_string(&mut buffer, &long_str).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(primitives::read_string(&mut cursor).unwrap(), long_str);
    }

    #[test]
    fn test_string_option_serialization() {
        let mut buffer = Vec::new();

        // Test Some string
        let some_str = Some("test string".to_string());
        primitives::write_string_option(&mut buffer, &some_str).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(
            primitives::read_string_option(&mut cursor).unwrap(),
            some_str
        );

        // Test None string
        buffer.clear();
        let none_str: Option<String> = None;
        primitives::write_string_option(&mut buffer, &none_str).unwrap();
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(
            primitives::read_string_option(&mut cursor).unwrap(),
            none_str
        );
    }

    #[test]
    fn test_vec_serialization() {
        struct TestItem(u32);

        impl BinarySerializable for TestItem {
            fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
                primitives::write_u32(writer, self.0)
            }

            fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
                Ok(TestItem(primitives::read_u32(reader)?))
            }

            fn binary_size(&self) -> usize {
                4
            }
        }

        let mut buffer = Vec::new();

        // Test empty vector
        let empty_vec: Vec<TestItem> = vec![];
        primitives::write_vec(&mut buffer, &empty_vec).unwrap();
        let mut cursor = Cursor::new(&buffer);
        let read_vec: Vec<TestItem> = primitives::read_vec(&mut cursor).unwrap();
        assert_eq!(read_vec.len(), 0);

        // Test vector with items
        buffer.clear();
        let test_vec = vec![TestItem(1), TestItem(2), TestItem(3)];
        primitives::write_vec(&mut buffer, &test_vec).unwrap();
        let mut cursor = Cursor::new(&buffer);
        let read_vec: Vec<TestItem> = primitives::read_vec(&mut cursor).unwrap();
        assert_eq!(read_vec.len(), 3);
        assert_eq!(read_vec[0].0, 1);
        assert_eq!(read_vec[1].0, 2);
        assert_eq!(read_vec[2].0, 3);
    }

    #[test]
    fn test_binary_unsafe_report() {
        let report = BinaryUnsafeReport {
            report_id: "test_report_123".to_string(),
            source_type: 1,
            source_details: "FFI function call".to_string(),
            risk_level: 2,
            risk_score: 7.5,
            confidence_score: 0.85,
            generated_at: 1234567890,
            dynamic_violations_count: 3,
            risk_factors_count: 5,
        };

        let mut buffer = Vec::new();
        let written_size = report.write_binary(&mut buffer).unwrap();
        assert_eq!(written_size, report.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_report = BinaryUnsafeReport::read_binary(&mut cursor).unwrap();

        assert_eq!(read_report.report_id, report.report_id);
        assert_eq!(read_report.source_type, report.source_type);
        assert_eq!(read_report.source_details, report.source_details);
        assert_eq!(read_report.risk_level, report.risk_level);
        assert!((read_report.risk_score - report.risk_score).abs() < 0.001);
        assert!((read_report.confidence_score - report.confidence_score).abs() < 0.001);
        assert_eq!(read_report.generated_at, report.generated_at);
        assert_eq!(
            read_report.dynamic_violations_count,
            report.dynamic_violations_count
        );
        assert_eq!(read_report.risk_factors_count, report.risk_factors_count);
    }

    #[test]
    fn test_binary_memory_passport() {
        let passport = BinaryMemoryPassport {
            passport_id: "passport_456".to_string(),
            memory_address: 0x7fff12345678,
            size_bytes: 1024,
            status_at_shutdown: 2,
            created_at: 1000000000,
            updated_at: 1000000100,
            lifecycle_events_count: 7,
        };

        let mut buffer = Vec::new();
        let written_size = passport.write_binary(&mut buffer).unwrap();
        assert_eq!(written_size, passport.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_passport = BinaryMemoryPassport::read_binary(&mut cursor).unwrap();

        assert_eq!(read_passport.passport_id, passport.passport_id);
        assert_eq!(read_passport.memory_address, passport.memory_address);
        assert_eq!(read_passport.size_bytes, passport.size_bytes);
        assert_eq!(
            read_passport.status_at_shutdown,
            passport.status_at_shutdown
        );
        assert_eq!(read_passport.created_at, passport.created_at);
        assert_eq!(read_passport.updated_at, passport.updated_at);
        assert_eq!(
            read_passport.lifecycle_events_count,
            passport.lifecycle_events_count
        );
    }

    #[test]
    fn test_binary_call_stack_ref() {
        let call_stack = BinaryCallStackRef {
            id: 12345,
            depth: 42,
            created_at: 9876543210,
        };

        let mut buffer = Vec::new();
        let written_size = call_stack.write_binary(&mut buffer).unwrap();
        assert_eq!(written_size, call_stack.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_call_stack = BinaryCallStackRef::read_binary(&mut cursor).unwrap();

        assert_eq!(read_call_stack.id, call_stack.id);
        assert_eq!(read_call_stack.depth, call_stack.depth);
        assert_eq!(read_call_stack.created_at, call_stack.created_at);
    }

    #[test]
    fn test_binary_borrow_info() {
        // Test with timestamp
        let borrow_info_with_timestamp = BinaryBorrowInfo {
            immutable_borrows: 5,
            mutable_borrows: 2,
            max_concurrent_borrows: 7,
            last_borrow_timestamp: Some(1234567890),
        };

        let mut buffer = Vec::new();
        let written_size = borrow_info_with_timestamp
            .write_binary(&mut buffer)
            .unwrap();
        assert_eq!(written_size, borrow_info_with_timestamp.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_borrow_info = BinaryBorrowInfo::read_binary(&mut cursor).unwrap();

        assert_eq!(
            read_borrow_info.immutable_borrows,
            borrow_info_with_timestamp.immutable_borrows
        );
        assert_eq!(
            read_borrow_info.mutable_borrows,
            borrow_info_with_timestamp.mutable_borrows
        );
        assert_eq!(
            read_borrow_info.max_concurrent_borrows,
            borrow_info_with_timestamp.max_concurrent_borrows
        );
        assert_eq!(
            read_borrow_info.last_borrow_timestamp,
            borrow_info_with_timestamp.last_borrow_timestamp
        );

        // Test without timestamp
        buffer.clear();
        let borrow_info_no_timestamp = BinaryBorrowInfo {
            immutable_borrows: 3,
            mutable_borrows: 1,
            max_concurrent_borrows: 4,
            last_borrow_timestamp: None,
        };

        let written_size = borrow_info_no_timestamp.write_binary(&mut buffer).unwrap();
        assert_eq!(written_size, borrow_info_no_timestamp.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_borrow_info = BinaryBorrowInfo::read_binary(&mut cursor).unwrap();

        assert_eq!(
            read_borrow_info.immutable_borrows,
            borrow_info_no_timestamp.immutable_borrows
        );
        assert_eq!(
            read_borrow_info.mutable_borrows,
            borrow_info_no_timestamp.mutable_borrows
        );
        assert_eq!(
            read_borrow_info.max_concurrent_borrows,
            borrow_info_no_timestamp.max_concurrent_borrows
        );
        assert_eq!(
            read_borrow_info.last_borrow_timestamp,
            borrow_info_no_timestamp.last_borrow_timestamp
        );
    }

    #[test]
    fn test_binary_clone_info() {
        // Test with original pointer
        let clone_info_with_ptr = BinaryCloneInfo {
            clone_count: 3,
            is_clone: true,
            original_ptr: Some(0x7fff87654321),
        };

        let mut buffer = Vec::new();
        let written_size = clone_info_with_ptr.write_binary(&mut buffer).unwrap();
        assert_eq!(written_size, clone_info_with_ptr.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_clone_info = BinaryCloneInfo::read_binary(&mut cursor).unwrap();

        assert_eq!(read_clone_info.clone_count, clone_info_with_ptr.clone_count);
        assert_eq!(read_clone_info.is_clone, clone_info_with_ptr.is_clone);
        assert_eq!(
            read_clone_info.original_ptr,
            clone_info_with_ptr.original_ptr
        );

        // Test without original pointer
        buffer.clear();
        let clone_info_no_ptr = BinaryCloneInfo {
            clone_count: 1,
            is_clone: false,
            original_ptr: None,
        };

        let written_size = clone_info_no_ptr.write_binary(&mut buffer).unwrap();
        assert_eq!(written_size, clone_info_no_ptr.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_clone_info = BinaryCloneInfo::read_binary(&mut cursor).unwrap();

        assert_eq!(read_clone_info.clone_count, clone_info_no_ptr.clone_count);
        assert_eq!(read_clone_info.is_clone, clone_info_no_ptr.is_clone);
        assert_eq!(read_clone_info.original_ptr, clone_info_no_ptr.original_ptr);
    }

    #[test]
    fn test_binary_ownership_event() {
        // Test with all optional fields
        let event_full = BinaryOwnershipEvent {
            timestamp: 1234567890,
            event_type: 3,
            source_stack_id: 42,
            clone_source_ptr: Some(0x7fff11111111),
            transfer_target_var: Some("target_variable".to_string()),
            borrower_scope: Some("function_scope".to_string()),
        };

        let mut buffer = Vec::new();
        let written_size = event_full.write_binary(&mut buffer).unwrap();
        assert_eq!(written_size, event_full.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_event = BinaryOwnershipEvent::read_binary(&mut cursor).unwrap();

        assert_eq!(read_event.timestamp, event_full.timestamp);
        assert_eq!(read_event.event_type, event_full.event_type);
        assert_eq!(read_event.source_stack_id, event_full.source_stack_id);
        assert_eq!(read_event.clone_source_ptr, event_full.clone_source_ptr);
        assert_eq!(
            read_event.transfer_target_var,
            event_full.transfer_target_var
        );
        assert_eq!(read_event.borrower_scope, event_full.borrower_scope);

        // Test with no optional fields
        buffer.clear();
        let event_minimal = BinaryOwnershipEvent {
            timestamp: 9876543210,
            event_type: 0,
            source_stack_id: 1,
            clone_source_ptr: None,
            transfer_target_var: None,
            borrower_scope: None,
        };

        let written_size = event_minimal.write_binary(&mut buffer).unwrap();
        assert_eq!(written_size, event_minimal.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_event = BinaryOwnershipEvent::read_binary(&mut cursor).unwrap();

        assert_eq!(read_event.timestamp, event_minimal.timestamp);
        assert_eq!(read_event.event_type, event_minimal.event_type);
        assert_eq!(read_event.source_stack_id, event_minimal.source_stack_id);
        assert_eq!(read_event.clone_source_ptr, event_minimal.clone_source_ptr);
        assert_eq!(
            read_event.transfer_target_var,
            event_minimal.transfer_target_var
        );
        assert_eq!(read_event.borrower_scope, event_minimal.borrower_scope);
    }

    #[test]
    fn test_binary_resolved_ffi_function() {
        // Test with signature
        let ffi_func_with_sig = BinaryResolvedFfiFunction {
            library_name: "libc.so.6".to_string(),
            function_name: "malloc".to_string(),
            signature: Some("fn(size: usize) -> *mut c_void".to_string()),
            category: 1,
            risk_level: 2,
        };

        let mut buffer = Vec::new();
        let written_size = ffi_func_with_sig.write_binary(&mut buffer).unwrap();
        assert_eq!(written_size, ffi_func_with_sig.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_ffi_func = BinaryResolvedFfiFunction::read_binary(&mut cursor).unwrap();

        assert_eq!(read_ffi_func.library_name, ffi_func_with_sig.library_name);
        assert_eq!(read_ffi_func.function_name, ffi_func_with_sig.function_name);
        assert_eq!(read_ffi_func.signature, ffi_func_with_sig.signature);
        assert_eq!(read_ffi_func.category, ffi_func_with_sig.category);
        assert_eq!(read_ffi_func.risk_level, ffi_func_with_sig.risk_level);

        // Test without signature
        buffer.clear();
        let ffi_func_no_sig = BinaryResolvedFfiFunction {
            library_name: "unknown".to_string(),
            function_name: "unknown_func".to_string(),
            signature: None,
            category: 0,
            risk_level: 3,
        };

        let written_size = ffi_func_no_sig.write_binary(&mut buffer).unwrap();
        assert_eq!(written_size, ffi_func_no_sig.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_ffi_func = BinaryResolvedFfiFunction::read_binary(&mut cursor).unwrap();

        assert_eq!(read_ffi_func.library_name, ffi_func_no_sig.library_name);
        assert_eq!(read_ffi_func.function_name, ffi_func_no_sig.function_name);
        assert_eq!(read_ffi_func.signature, ffi_func_no_sig.signature);
        assert_eq!(read_ffi_func.category, ffi_func_no_sig.category);
        assert_eq!(read_ffi_func.risk_level, ffi_func_no_sig.risk_level);
    }

    #[test]
    fn test_binary_size_calculations() {
        // Test that binary_size() matches actual written size for all types
        let unsafe_report = BinaryUnsafeReport {
            report_id: "test".to_string(),
            source_type: 1,
            source_details: "details".to_string(),
            risk_level: 2,
            risk_score: 5.0,
            confidence_score: 0.8,
            generated_at: 123456,
            dynamic_violations_count: 1,
            risk_factors_count: 2,
        };

        let mut buffer = Vec::new();
        let written = unsafe_report.write_binary(&mut buffer).unwrap();
        assert_eq!(written, unsafe_report.binary_size());
        assert_eq!(buffer.len(), unsafe_report.binary_size());

        // Test other types similarly
        let memory_passport = BinaryMemoryPassport {
            passport_id: "passport".to_string(),
            memory_address: 0x1000,
            size_bytes: 64,
            status_at_shutdown: 1,
            created_at: 100,
            updated_at: 200,
            lifecycle_events_count: 3,
        };

        buffer.clear();
        let written = memory_passport.write_binary(&mut buffer).unwrap();
        assert_eq!(written, memory_passport.binary_size());
        assert_eq!(buffer.len(), memory_passport.binary_size());
    }

    #[test]
    fn test_error_handling_corrupted_data() {
        // Test reading from empty buffer
        let mut empty_cursor = Cursor::new(Vec::<u8>::new());
        assert!(primitives::read_u32(&mut empty_cursor).is_err());
        assert!(primitives::read_string(&mut empty_cursor).is_err());

        // Test reading string with invalid UTF-8
        let mut buffer = Vec::new();
        primitives::write_u32(&mut buffer, 4).unwrap(); // length = 4
        buffer.extend_from_slice(&[0xFF, 0xFE, 0xFD, 0xFC]); // invalid UTF-8

        let mut cursor = Cursor::new(&buffer);
        let result = primitives::read_string(&mut cursor);
        assert!(result.is_err());
        if let Err(BinaryExportError::CorruptedData(msg)) = result {
            assert!(msg.contains("Invalid UTF-8"));
        } else {
            panic!("Expected CorruptedData error");
        }
    }

    #[test]
    fn test_roundtrip_consistency() {
        // Test that write -> read -> write produces identical results
        let original_event = BinaryOwnershipEvent {
            timestamp: 1234567890,
            event_type: 2,
            source_stack_id: 99,
            clone_source_ptr: Some(0x7fff99999999),
            transfer_target_var: Some("test_var".to_string()),
            borrower_scope: None,
        };

        // First serialization
        let mut buffer1 = Vec::new();
        original_event.write_binary(&mut buffer1).unwrap();

        // Deserialize
        let mut cursor = Cursor::new(&buffer1);
        let deserialized_event = BinaryOwnershipEvent::read_binary(&mut cursor).unwrap();

        // Second serialization
        let mut buffer2 = Vec::new();
        deserialized_event.write_binary(&mut buffer2).unwrap();

        // Buffers should be identical
        assert_eq!(buffer1, buffer2);
    }
}

/// Binary serializable wrapper for UnsafeReport
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryUnsafeReport {
    pub report_id: String,
    pub source_type: u8, // 0=UnsafeBlock, 1=FfiFunction, 2=RawPointer, 3=Transmute
    pub source_details: String, // JSON serialized details
    pub risk_level: u8,  // 0=Low, 1=Medium, 2=High, 3=Critical
    pub risk_score: f32,
    pub confidence_score: f32,
    pub generated_at: u64,
    pub dynamic_violations_count: u32,
    pub risk_factors_count: u32,
}

impl BinarySerializable for BinaryUnsafeReport {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut bytes_written = 0;
        bytes_written += primitives::write_string(writer, &self.report_id)?;
        bytes_written += primitives::write_u8(writer, self.source_type)?;
        bytes_written += primitives::write_string(writer, &self.source_details)?;
        bytes_written += primitives::write_u8(writer, self.risk_level)?;
        bytes_written += primitives::write_f32(writer, self.risk_score)?;
        bytes_written += primitives::write_f32(writer, self.confidence_score)?;
        bytes_written += primitives::write_u64(writer, self.generated_at)?;
        bytes_written += primitives::write_u32(writer, self.dynamic_violations_count)?;
        bytes_written += primitives::write_u32(writer, self.risk_factors_count)?;
        Ok(bytes_written)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        Ok(BinaryUnsafeReport {
            report_id: primitives::read_string(reader)?,
            source_type: primitives::read_u8(reader)?,
            source_details: primitives::read_string(reader)?,
            risk_level: primitives::read_u8(reader)?,
            risk_score: primitives::read_f32(reader)?,
            confidence_score: primitives::read_f32(reader)?,
            generated_at: primitives::read_u64(reader)?,
            dynamic_violations_count: primitives::read_u32(reader)?,
            risk_factors_count: primitives::read_u32(reader)?,
        })
    }

    fn binary_size(&self) -> usize {
        4 + self.report_id.len() + // string length + content
        1 + // source_type
        4 + self.source_details.len() + // string length + content
        1 + // risk_level
        4 + // risk_score (f32)
        4 + // confidence_score (f32)
        8 + // generated_at (u64)
        4 + // dynamic_violations_count
        4 // risk_factors_count
    }
}

/// Binary serializable wrapper for MemoryPassport
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryMemoryPassport {
    pub passport_id: String,
    pub memory_address: u64,
    pub size_bytes: u32,
    pub status_at_shutdown: u8, // 0=FreedByRust, 1=HandoverToFfi, 2=FreedByForeign, 3=ReclaimedByRust, 4=InForeignCustody, 5=Unknown
    pub created_at: u64,
    pub updated_at: u64,
    pub lifecycle_events_count: u32,
}

impl BinarySerializable for BinaryMemoryPassport {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut bytes_written = 0;
        bytes_written += primitives::write_string(writer, &self.passport_id)?;
        bytes_written += primitives::write_u64(writer, self.memory_address)?;
        bytes_written += primitives::write_u32(writer, self.size_bytes)?;
        bytes_written += primitives::write_u8(writer, self.status_at_shutdown)?;
        bytes_written += primitives::write_u64(writer, self.created_at)?;
        bytes_written += primitives::write_u64(writer, self.updated_at)?;
        bytes_written += primitives::write_u32(writer, self.lifecycle_events_count)?;
        Ok(bytes_written)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        Ok(BinaryMemoryPassport {
            passport_id: primitives::read_string(reader)?,
            memory_address: primitives::read_u64(reader)?,
            size_bytes: primitives::read_u32(reader)?,
            status_at_shutdown: primitives::read_u8(reader)?,
            created_at: primitives::read_u64(reader)?,
            updated_at: primitives::read_u64(reader)?,
            lifecycle_events_count: primitives::read_u32(reader)?,
        })
    }

    fn binary_size(&self) -> usize {
        4 + self.passport_id.len() + // string length + content
        8 + // memory_address
        4 + // size_bytes
        1 + // status_at_shutdown
        8 + // created_at
        8 + // updated_at
        4 // lifecycle_events_count
    }
}

/// Binary serializable wrapper for CallStackRef
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryCallStackRef {
    pub id: u32,
    pub depth: u16,
    pub created_at: u64,
}

impl BinarySerializable for BinaryCallStackRef {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut bytes_written = 0;
        bytes_written += primitives::write_u32(writer, self.id)?;
        bytes_written += primitives::write_u16(writer, self.depth)?;
        bytes_written += primitives::write_u64(writer, self.created_at)?;
        Ok(bytes_written)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        Ok(BinaryCallStackRef {
            id: primitives::read_u32(reader)?,
            depth: primitives::read_u16(reader)?,
            created_at: primitives::read_u64(reader)?,
        })
    }

    fn binary_size(&self) -> usize {
        4 + 2 + 8 // id + depth + created_at
    }
}

/// Binary serializable wrapper for BorrowInfo
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryBorrowInfo {
    pub immutable_borrows: u32,
    pub mutable_borrows: u32,
    pub max_concurrent_borrows: u32,
    pub last_borrow_timestamp: Option<u64>,
}

impl BinarySerializable for BinaryBorrowInfo {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut bytes_written = 0;
        bytes_written += primitives::write_u32(writer, self.immutable_borrows)?;
        bytes_written += primitives::write_u32(writer, self.mutable_borrows)?;
        bytes_written += primitives::write_u32(writer, self.max_concurrent_borrows)?;
        bytes_written += match self.last_borrow_timestamp {
            Some(timestamp) => {
                primitives::write_u8(writer, 1)? + primitives::write_u64(writer, timestamp)?
            }
            None => primitives::write_u8(writer, 0)?,
        };
        Ok(bytes_written)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let immutable_borrows = primitives::read_u32(reader)?;
        let mutable_borrows = primitives::read_u32(reader)?;
        let max_concurrent_borrows = primitives::read_u32(reader)?;
        let has_timestamp = primitives::read_u8(reader)?;
        let last_borrow_timestamp = if has_timestamp == 1 {
            Some(primitives::read_u64(reader)?)
        } else {
            None
        };

        Ok(BinaryBorrowInfo {
            immutable_borrows,
            mutable_borrows,
            max_concurrent_borrows,
            last_borrow_timestamp,
        })
    }

    fn binary_size(&self) -> usize {
        4 + 4
            + 4
            + 1
            + if self.last_borrow_timestamp.is_some() {
                8
            } else {
                0
            }
    }
}

/// Binary serializable wrapper for CloneInfo
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryCloneInfo {
    pub clone_count: u32,
    pub is_clone: bool,
    pub original_ptr: Option<u64>,
}

impl BinarySerializable for BinaryCloneInfo {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut bytes_written = 0;
        bytes_written += primitives::write_u32(writer, self.clone_count)?;
        bytes_written += primitives::write_u8(writer, if self.is_clone { 1 } else { 0 })?;
        bytes_written += match self.original_ptr {
            Some(ptr) => primitives::write_u8(writer, 1)? + primitives::write_u64(writer, ptr)?,
            None => primitives::write_u8(writer, 0)?,
        };
        Ok(bytes_written)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let clone_count = primitives::read_u32(reader)?;
        let is_clone = primitives::read_u8(reader)? == 1;
        let has_original_ptr = primitives::read_u8(reader)?;
        let original_ptr = if has_original_ptr == 1 {
            Some(primitives::read_u64(reader)?)
        } else {
            None
        };

        Ok(BinaryCloneInfo {
            clone_count,
            is_clone,
            original_ptr,
        })
    }

    fn binary_size(&self) -> usize {
        4 + 1 + 1 + if self.original_ptr.is_some() { 8 } else { 0 }
    }
}

/// Binary serializable wrapper for OwnershipEvent
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryOwnershipEvent {
    pub timestamp: u64,
    pub event_type: u8, // 0=Allocated, 1=Cloned, 2=Dropped, 3=OwnershipTransferred, 4=Borrowed, 5=MutablyBorrowed
    pub source_stack_id: u32,
    pub clone_source_ptr: Option<u64>,
    pub transfer_target_var: Option<String>,
    pub borrower_scope: Option<String>,
}

impl BinarySerializable for BinaryOwnershipEvent {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut bytes_written = 0;
        bytes_written += primitives::write_u64(writer, self.timestamp)?;
        bytes_written += primitives::write_u8(writer, self.event_type)?;
        bytes_written += primitives::write_u32(writer, self.source_stack_id)?;

        // Write optional clone_source_ptr
        bytes_written += match self.clone_source_ptr {
            Some(ptr) => primitives::write_u8(writer, 1)? + primitives::write_u64(writer, ptr)?,
            None => primitives::write_u8(writer, 0)?,
        };

        // Write optional transfer_target_var
        bytes_written += primitives::write_string_option(writer, &self.transfer_target_var)?;

        // Write optional borrower_scope
        bytes_written += primitives::write_string_option(writer, &self.borrower_scope)?;

        Ok(bytes_written)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let timestamp = primitives::read_u64(reader)?;
        let event_type = primitives::read_u8(reader)?;
        let source_stack_id = primitives::read_u32(reader)?;

        let has_clone_ptr = primitives::read_u8(reader)?;
        let clone_source_ptr = if has_clone_ptr == 1 {
            Some(primitives::read_u64(reader)?)
        } else {
            None
        };

        let transfer_target_var = primitives::read_string_option(reader)?;
        let borrower_scope = primitives::read_string_option(reader)?;

        Ok(BinaryOwnershipEvent {
            timestamp,
            event_type,
            source_stack_id,
            clone_source_ptr,
            transfer_target_var,
            borrower_scope,
        })
    }

    fn binary_size(&self) -> usize {
        8 + // timestamp
        1 + // event_type
        4 + // source_stack_id
        1 + if self.clone_source_ptr.is_some() { 8 } else { 0 } + // optional clone_source_ptr
        1 + self.transfer_target_var.as_ref().map_or(0, |s| 4 + s.len()) + // optional transfer_target_var
        1 + self.borrower_scope.as_ref().map_or(0, |s| 4 + s.len()) // optional borrower_scope
    }
}

/// Binary serializable wrapper for ResolvedFfiFunction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryResolvedFfiFunction {
    pub library_name: String,
    pub function_name: String,
    pub signature: Option<String>,
    pub category: u8,   // Enum index
    pub risk_level: u8, // Enum index
}

impl BinarySerializable for BinaryResolvedFfiFunction {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut bytes_written = 0;
        bytes_written += primitives::write_string(writer, &self.library_name)?;
        bytes_written += primitives::write_string(writer, &self.function_name)?;
        bytes_written += primitives::write_string_option(writer, &self.signature)?;
        bytes_written += primitives::write_u8(writer, self.category)?;
        bytes_written += primitives::write_u8(writer, self.risk_level)?;
        Ok(bytes_written)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        Ok(BinaryResolvedFfiFunction {
            library_name: primitives::read_string(reader)?,
            function_name: primitives::read_string(reader)?,
            signature: primitives::read_string_option(reader)?,
            category: primitives::read_u8(reader)?,
            risk_level: primitives::read_u8(reader)?,
        })
    }

    fn binary_size(&self) -> usize {
        4 + self.library_name.len() + // string length + content
        4 + self.function_name.len() + // string length + content
        1 + self.signature.as_ref().map_or(0, |s| 4 + s.len()) + // option flag + optional string
        1 + // category
        1 // risk_level
    }
}
