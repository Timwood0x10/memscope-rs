//! Binary serialization implementations for smart pointer types

use crate::core::types::{RefCountSnapshot, SmartPointerInfo, SmartPointerType};
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::serializable::{primitives, BinarySerializable};
use std::io::{Read, Write};

impl BinarySerializable for SmartPointerType {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let type_id = match self {
            SmartPointerType::Rc => 0u8,
            SmartPointerType::Arc => 1u8,
            SmartPointerType::RcWeak => 2u8,
            SmartPointerType::ArcWeak => 3u8,
            SmartPointerType::Box => 4u8,
        };
        primitives::write_u8(writer, type_id)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let type_id = primitives::read_u8(reader)?;
        match type_id {
            0 => Ok(SmartPointerType::Rc),
            1 => Ok(SmartPointerType::Arc),
            2 => Ok(SmartPointerType::RcWeak),
            3 => Ok(SmartPointerType::ArcWeak),
            4 => Ok(SmartPointerType::Box),
            _ => Err(BinaryExportError::CorruptedData(format!(
                "Invalid smart pointer type ID: {}",
                type_id
            ))),
        }
    }

    fn binary_size(&self) -> usize {
        1 // Single byte for type ID
    }
}

impl BinarySerializable for RefCountSnapshot {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut size = 0;
        size += primitives::write_u64(writer, self.timestamp)?;
        size += primitives::write_usize(writer, self.strong_count)?;
        size += primitives::write_usize(writer, self.weak_count)?;
        Ok(size)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        let timestamp = primitives::read_u64(reader)?;
        let strong_count = primitives::read_usize(reader)?;
        let weak_count = primitives::read_usize(reader)?;

        Ok(RefCountSnapshot {
            timestamp,
            strong_count,
            weak_count,
        })
    }

    fn binary_size(&self) -> usize {
        8 + 8 + 8 // timestamp + strong_count + weak_count (as u64s)
    }
}

impl BinarySerializable for SmartPointerInfo {
    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut size = 0;

        // Write data_ptr
        size += primitives::write_usize(writer, self.data_ptr)?;

        // Write cloned_from (optional usize)
        size += match self.cloned_from {
            Some(ptr) => primitives::write_u8(writer, 1)? + primitives::write_usize(writer, ptr)?,
            None => primitives::write_u8(writer, 0)?,
        };

        // Write clones vector
        size += primitives::write_u32(writer, self.clones.len() as u32)?;
        for clone_ptr in &self.clones {
            size += primitives::write_usize(writer, *clone_ptr)?;
        }

        // Write ref_count_history
        size += primitives::write_vec(writer, &self.ref_count_history)?;

        // Write weak_count (optional usize)
        size += match self.weak_count {
            Some(count) => {
                primitives::write_u8(writer, 1)? + primitives::write_usize(writer, count)?
            }
            None => primitives::write_u8(writer, 0)?,
        };

        // Write boolean flags
        size += primitives::write_u8(writer, if self.is_weak_reference { 1 } else { 0 })?;
        size += primitives::write_u8(writer, if self.is_data_owner { 1 } else { 0 })?;
        size += primitives::write_u8(writer, if self.is_implicitly_deallocated { 1 } else { 0 })?;

        // Write pointer_type
        size += self.pointer_type.write_binary(writer)?;

        Ok(size)
    }

    fn read_binary<R: Read>(reader: &mut R) -> Result<Self, BinaryExportError> {
        // Read data_ptr
        let data_ptr = primitives::read_usize(reader)?;

        // Read cloned_from (optional usize)
        let cloned_from = if primitives::read_u8(reader)? == 1 {
            Some(primitives::read_usize(reader)?)
        } else {
            None
        };

        // Read clones vector
        let clones_len = primitives::read_u32(reader)? as usize;
        let mut clones = Vec::with_capacity(clones_len);
        for _ in 0..clones_len {
            clones.push(primitives::read_usize(reader)?);
        }

        // Read ref_count_history
        let ref_count_history = primitives::read_vec(reader)?;

        // Read weak_count (optional usize)
        let weak_count = if primitives::read_u8(reader)? == 1 {
            Some(primitives::read_usize(reader)?)
        } else {
            None
        };

        // Read boolean flags
        let is_weak_reference = primitives::read_u8(reader)? == 1;
        let is_data_owner = primitives::read_u8(reader)? == 1;
        let is_implicitly_deallocated = primitives::read_u8(reader)? == 1;

        // Read pointer_type
        let pointer_type = SmartPointerType::read_binary(reader)?;

        Ok(SmartPointerInfo {
            data_ptr,
            cloned_from,
            clones,
            ref_count_history,
            weak_count,
            is_weak_reference,
            is_data_owner,
            is_implicitly_deallocated,
            pointer_type,
        })
    }

    fn binary_size(&self) -> usize {
        let mut size = 0;

        // data_ptr
        size += 8;

        // cloned_from (flag + optional usize)
        size += 1;
        if self.cloned_from.is_some() {
            size += 8;
        }

        // clones vector (length + items)
        size += 4 + (self.clones.len() * 8);

        // ref_count_history (length + items)
        size += 4 + (self.ref_count_history.len() * 24); // Each RefCountSnapshot is 24 bytes

        // weak_count (flag + optional usize)
        size += 1;
        if self.weak_count.is_some() {
            size += 8;
        }

        // boolean flags (3 bytes)
        size += 3;

        // pointer_type
        size += 1;

        size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_smart_pointer_type_serialization() {
        let types = vec![
            SmartPointerType::Rc,
            SmartPointerType::Arc,
            SmartPointerType::RcWeak,
            SmartPointerType::ArcWeak,
            SmartPointerType::Box,
        ];

        for original_type in types {
            let mut buffer = Vec::new();
            original_type.write_binary(&mut buffer).unwrap();

            let mut cursor = Cursor::new(&buffer);
            let read_type = SmartPointerType::read_binary(&mut cursor).unwrap();

            assert_eq!(original_type, read_type);
        }
    }

    #[test]
    fn test_ref_count_snapshot_serialization() {
        let snapshot = RefCountSnapshot {
            timestamp: 1234567890,
            strong_count: 5,
            weak_count: 2,
        };

        let mut buffer = Vec::new();
        snapshot.write_binary(&mut buffer).unwrap();

        let mut cursor = Cursor::new(&buffer);
        let read_snapshot = RefCountSnapshot::read_binary(&mut cursor).unwrap();

        assert_eq!(snapshot, read_snapshot);
    }

    #[test]
    fn test_smart_pointer_info_serialization() {
        let info = SmartPointerInfo {
            data_ptr: 0x1000,
            cloned_from: Some(0x2000),
            clones: vec![0x3000, 0x4000],
            ref_count_history: vec![
                RefCountSnapshot {
                    timestamp: 1000,
                    strong_count: 1,
                    weak_count: 0,
                },
                RefCountSnapshot {
                    timestamp: 2000,
                    strong_count: 2,
                    weak_count: 1,
                },
            ],
            weak_count: Some(1),
            is_weak_reference: false,
            is_data_owner: true,
            is_implicitly_deallocated: false,
            pointer_type: SmartPointerType::Rc,
        };

        let mut buffer = Vec::new();
        let written_size = info.write_binary(&mut buffer).unwrap();

        // Verify size calculation
        assert_eq!(written_size, info.binary_size());

        let mut cursor = Cursor::new(&buffer);
        let read_info = SmartPointerInfo::read_binary(&mut cursor).unwrap();

        assert_eq!(info, read_info);
    }

    #[test]
    fn test_smart_pointer_info_minimal() {
        let info = SmartPointerInfo {
            data_ptr: 0x1000,
            cloned_from: None,
            clones: vec![],
            ref_count_history: vec![],
            weak_count: None,
            is_weak_reference: false,
            is_data_owner: false,
            is_implicitly_deallocated: false,
            pointer_type: SmartPointerType::Box,
        };

        let mut buffer = Vec::new();
        info.write_binary(&mut buffer).unwrap();

        let mut cursor = Cursor::new(&buffer);
        let read_info = SmartPointerInfo::read_binary(&mut cursor).unwrap();

        assert_eq!(info, read_info);
    }
}
