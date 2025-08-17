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
        primitives::write_u32(&mut buffer, 0x12345678).unwrap();
        let mut cursor = Cursor::new(&buffer);
        let value = primitives::read_u32(&mut cursor).expect("Failed to get test value");
        assert_eq!(value, 0x12345678);

        // Test string
        buffer.clear();
        primitives::write_string(&mut buffer, "test").unwrap();
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
        let read_value: Option<TestStruct> = primitives::read_option(&mut cursor).unwrap();
        assert!(read_value.is_some());
        assert_eq!(read_value.unwrap().0, 42);

        // Test None value
        buffer.clear();
        let none_value: Option<TestStruct> = None;
        primitives::write_option(&mut buffer, &none_value).unwrap();

        let mut cursor = Cursor::new(&buffer);
        let read_value: Option<TestStruct> = primitives::read_option(&mut cursor).unwrap();
        assert!(read_value.is_none());
    }
}

/// Binary serializable wrapper for UnsafeReport
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryUnsafeReport {
    pub report_id: String,
    pub source_type: u8, // 0=UnsafeBlock, 1=FfiFunction, 2=RawPointer, 3=Transmute
    pub source_details: String, // JSON serialized details
    pub risk_level: u8, // 0=Low, 1=Medium, 2=High, 3=Critical
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
        4   // risk_factors_count
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
        4   // lifecycle_events_count
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

/// Binary serializable wrapper for ResolvedFfiFunction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryResolvedFfiFunction {
    pub library_name: String,
    pub function_name: String,
    pub signature: Option<String>,
    pub category: u8, // Enum index
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
        1   // risk_level
    }
}
