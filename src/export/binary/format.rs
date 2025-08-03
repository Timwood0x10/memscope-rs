//! Binary file format definitions using simple TLV (Type-Length-Value) structure

/// File magic bytes for format identification
pub const MAGIC_BYTES: &[u8; 8] = b"MEMSCOPE";

pub const FORMAT_VERSION: u32 = 1;
pub const HEADER_SIZE: usize = 16;
pub const ALLOCATION_RECORD_TYPE: u8 = 1;

/// File header structure (16 bytes fixed size)
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct FileHeader {
    pub magic: [u8; 8],
    pub version: u32,
    pub count: u32,
}

impl FileHeader {
    pub fn new(count: u32) -> Self {
        Self {
            magic: *MAGIC_BYTES,
            version: FORMAT_VERSION,
            count,
        }
    }

    pub fn is_valid_magic(&self) -> bool {
        self.magic == *MAGIC_BYTES
    }

    pub fn is_compatible_version(&self) -> bool {
        self.version == FORMAT_VERSION
    }

    /// Convert header to bytes using Little Endian format
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];

        bytes[0..8].copy_from_slice(&self.magic);
        bytes[8..12].copy_from_slice(&self.version.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.count.to_le_bytes());

        bytes
    }

    /// Create header from bytes using Little Endian format
    pub fn from_bytes(bytes: &[u8; HEADER_SIZE]) -> Self {
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);

        let version = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let count = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

        Self {
            magic,
            version,
            count,
        }
    }
}

/// Allocation record structure for binary serialization
#[derive(Debug, Clone, PartialEq)]
pub struct AllocationRecord {
    pub ptr: u64,
    pub size: u64,
    pub timestamp: u64,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub thread_id: String,
}

impl AllocationRecord {
    /// Calculate serialized size in bytes
    #[allow(dead_code)]
    pub fn serialized_size(&self) -> usize {
        let mut size = 1 + 4; // Type + Length
        size += 8 + 8 + 8; // ptr + size + timestamp

        size += 4; // var_name_len
        if let Some(ref name) = self.var_name {
            size += name.len();
        }

        size += 4; // type_name_len
        if let Some(ref name) = self.type_name {
            size += name.len();
        }

        size += 4; // thread_id_len
        size += self.thread_id.len();

        size
    }
}

/// Endian conversion utilities
pub mod endian {
    #[allow(dead_code)]
    pub fn u32_to_le_bytes(value: u32) -> [u8; 4] {
        value.to_le_bytes()
    }

    #[allow(dead_code)]
    pub fn u32_from_le_bytes(bytes: [u8; 4]) -> u32 {
        u32::from_le_bytes(bytes)
    }

    #[allow(dead_code)]
    pub fn u64_to_le_bytes(value: u64) -> [u8; 8] {
        value.to_le_bytes()
    }

    #[allow(dead_code)]
    pub fn u64_from_le_bytes(bytes: [u8; 8]) -> u64 {
        u64::from_le_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_header_creation() {
        let header = FileHeader::new(100);
        assert_eq!(header.magic, *MAGIC_BYTES);
        assert_eq!(header.version, FORMAT_VERSION);
        assert_eq!(header.count, 100);
        assert!(header.is_valid_magic());
        assert!(header.is_compatible_version());
    }

    #[test]
    fn test_file_header_serialization() {
        let header = FileHeader::new(42);
        let bytes = header.to_bytes();
        let deserialized = FileHeader::from_bytes(&bytes);

        assert_eq!(header, deserialized);
    }

    #[test]
    fn test_allocation_record_size_calculation() {
        let record = AllocationRecord {
            ptr: 0x1000,
            size: 1024,
            timestamp: 1234567890,
            var_name: Some("test_var".to_string()),
            type_name: Some("i32".to_string()),
            thread_id: "main".to_string(),
        };

        let expected_size = 1 + 4 + // Type + Length
                           8 + 8 + 8 + // ptr + size + timestamp
                           4 + 8 + // var_name_len + var_name
                           4 + 3 + // type_name_len + type_name
                           4 + 4; // thread_id_len + thread_id

        assert_eq!(record.serialized_size(), expected_size);
    }

    #[test]
    fn test_endian_conversion() {
        let value = 0x12345678u32;
        let bytes = endian::u32_to_le_bytes(value);
        let converted = endian::u32_from_le_bytes(bytes);
        assert_eq!(value, converted);

        let value64 = 0x123456789ABCDEFu64;
        let bytes64 = endian::u64_to_le_bytes(value64);
        let converted64 = endian::u64_from_le_bytes(bytes64);
        assert_eq!(value64, converted64);
    }
}
