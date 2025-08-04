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

    /// Write a u32 value in little endian
    pub fn write_u32<W: Write>(writer: &mut W, value: u32) -> Result<usize, BinaryExportError> {
        writer.write_all(&value.to_le_bytes())?;
        Ok(4)
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
        let value = primitives::read_u32(&mut cursor).unwrap();
        assert_eq!(value, 0x12345678);

        // Test string
        buffer.clear();
        primitives::write_string(&mut buffer, "test").unwrap();
        let mut cursor = Cursor::new(&buffer);
        let string = primitives::read_string(&mut cursor).unwrap();
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
        primitives::write_option(&mut buffer, &some_value).unwrap();

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
