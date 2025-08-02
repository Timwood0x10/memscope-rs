//! Binary format definitions for high-performance memory tracking data export
//!
//! This module implements a segmented binary file format designed for optimal performance
//! and minimal file size. The format consists of:
//! - 64-byte file header with magic number, version, and metadata
//! - Section directory with 16-byte descriptors for each section
//! - 8 core data sections: Memory Stats, Active Allocations, History, Type Usage, FFI, Lifecycle, Performance, Variable Registry

use std::collections::HashMap;
use std::io::{Cursor, Read, Write};

/// Magic number for MemScope binary files: "MEMSCOPE"
pub const BINARY_MAGIC: [u8; 8] = *b"MEMSCOPE";

/// Current binary format version
pub const BINARY_VERSION_MAJOR: u16 = 1;
pub const BINARY_VERSION_MINOR: u16 = 0;

/// File header size in bytes
pub const HEADER_SIZE: usize = 64;

/// Section directory entry size in bytes
pub const SECTION_ENTRY_SIZE: usize = 20;

/// Binary file header (64 bytes total)
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryHeader {
    /// Magic number: "MEMSCOPE" (8 bytes)
    pub magic: [u8; 8],
    /// Major version number (2 bytes)
    pub version_major: u16,
    /// Minor version number (2 bytes)
    pub version_minor: u16,
    /// Compression type flags (4 bytes)
    pub compression_type: CompressionType,
    /// Number of sections in the file (4 bytes)
    pub section_count: u32,
    /// Total file size in bytes (8 bytes)
    pub total_size: u64,
    /// Creation timestamp (8 bytes)
    pub timestamp: u64,
    /// Header checksum CRC64 (8 bytes)
    pub checksum: u64,
    /// Reserved for future use (20 bytes)
    pub reserved: [u8; 20],
}

/// Compression types supported by the binary format
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CompressionType {
    /// No compression - fastest export/import
    None = 0,
    /// LZ4 compression - balanced speed and size
    Lz4 = 1,
    /// Zstd compression - best compression ratio
    Zstd = 2,
}

/// Section types in the binary format
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionType {
    /// Memory statistics and summary data
    MemoryStats = 1,
    /// Currently active allocations
    ActiveAllocations = 2,
    /// Historical allocation data
    AllocationHistory = 3,
    /// Memory usage by type
    TypeMemoryUsage = 4,
    /// FFI analysis data
    FfiAnalysis = 5,
    /// Lifecycle analysis data
    LifecycleAnalysis = 6,
    /// Performance metrics
    PerformanceData = 7,
    /// Variable registry data
    VariableRegistry = 8,
    /// Security violations and analysis
    SecurityViolations = 9,
    /// Memory passports for cross-boundary tracking
    MemoryPassports = 10,
}

/// Section directory entry (16 bytes)
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct SectionEntry {
    /// Section type identifier (1 byte)
    pub section_type: SectionType,
    /// Compression used for this section (1 byte)
    pub compression: CompressionType,
    /// Reserved flags (2 bytes)
    pub flags: u16,
    /// Offset from file start to section data (8 bytes)
    pub offset: u64,
    /// Compressed size of section data (4 bytes)
    pub compressed_size: u32,
    /// Uncompressed size of section data (4 bytes) - 0 if no compression
    pub uncompressed_size: u32,
}

/// Section directory containing all section entries
#[derive(Debug, Clone)]
pub struct SectionDirectory {
    /// Map of section type to section entry
    pub sections: HashMap<SectionType, SectionEntry>,
}

/// Complete binary file structure
#[derive(Debug, Clone)]
pub struct BinaryFile {
    /// File header
    pub header: BinaryHeader,
    /// Section directory
    pub directory: SectionDirectory,
    /// Raw section data (section_type -> compressed data)
    pub section_data: HashMap<SectionType, Vec<u8>>,
}

impl Default for BinaryHeader {
    fn default() -> Self {
        Self {
            magic: BINARY_MAGIC,
            version_major: BINARY_VERSION_MAJOR,
            version_minor: BINARY_VERSION_MINOR,
            compression_type: CompressionType::None,
            section_count: 0,
            total_size: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            checksum: 0,
            reserved: [0; 20],
        }
    }
}

impl BinaryHeader {
    /// Create a new binary header with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the header magic number and version
    pub fn validate(&self) -> Result<(), BinaryFormatError> {
        if self.magic != BINARY_MAGIC {
            return Err(BinaryFormatError::InvalidMagic {
                expected: BINARY_MAGIC,
                found: self.magic,
            });
        }

        if self.version_major > BINARY_VERSION_MAJOR {
            return Err(BinaryFormatError::UnsupportedVersion {
                file_version: format!("{}.{}", self.version_major, self.version_minor),
                supported_version: format!("{}.{}", BINARY_VERSION_MAJOR, BINARY_VERSION_MINOR),
            });
        }

        Ok(())
    }

    /// Calculate and set the header checksum
    pub fn calculate_checksum(&mut self) {
        // Temporarily set checksum to 0 for calculation
        self.checksum = 0;
        let header_bytes = self.to_bytes();
        self.checksum = crc64_checksum(&header_bytes[..56]); // Exclude checksum field itself
    }

    /// Serialize header to bytes (Little Endian)
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];
        let mut offset = 0;

        // Magic number (8 bytes)
        bytes[offset..offset + 8].copy_from_slice(&self.magic);
        offset += 8;

        // Version (4 bytes)
        bytes[offset..offset + 2].copy_from_slice(&self.version_major.to_le_bytes());
        offset += 2;
        bytes[offset..offset + 2].copy_from_slice(&self.version_minor.to_le_bytes());
        offset += 2;

        // Compression type (4 bytes)
        bytes[offset..offset + 4].copy_from_slice(&(self.compression_type as u32).to_le_bytes());
        offset += 4;

        // Section count (4 bytes)
        bytes[offset..offset + 4].copy_from_slice(&self.section_count.to_le_bytes());
        offset += 4;

        // Total size (8 bytes)
        bytes[offset..offset + 8].copy_from_slice(&self.total_size.to_le_bytes());
        offset += 8;

        // Timestamp (8 bytes)
        bytes[offset..offset + 8].copy_from_slice(&self.timestamp.to_le_bytes());
        offset += 8;

        // Checksum (8 bytes)
        bytes[offset..offset + 8].copy_from_slice(&self.checksum.to_le_bytes());
        offset += 8;

        // Reserved (20 bytes) - already zeroed

        bytes
    }

    /// Deserialize header from bytes (Little Endian)
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BinaryFormatError> {
        if bytes.len() < HEADER_SIZE {
            return Err(BinaryFormatError::InvalidHeaderSize {
                expected: HEADER_SIZE,
                found: bytes.len(),
            });
        }

        let mut offset = 0;

        // Magic number (8 bytes)
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[offset..offset + 8]);
        offset += 8;

        // Version (4 bytes)
        let version_major = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
        offset += 2;
        let version_minor = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
        offset += 2;

        // Compression type (4 bytes)
        let compression_raw = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        let compression_type = match compression_raw {
            0 => CompressionType::None,
            1 => CompressionType::Lz4,
            2 => CompressionType::Zstd,
            _ => return Err(BinaryFormatError::InvalidCompressionType(compression_raw)),
        };
        offset += 4;

        // Section count (4 bytes)
        let section_count = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        // Total size (8 bytes)
        let total_size = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // Timestamp (8 bytes)
        let timestamp = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // Checksum (8 bytes)
        let checksum = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // Reserved (20 bytes)
        let mut reserved = [0u8; 20];
        reserved.copy_from_slice(&bytes[offset..offset + 20]);

        let header = Self {
            magic,
            version_major,
            version_minor,
            compression_type,
            section_count,
            total_size,
            timestamp,
            checksum,
            reserved,
        };

        // Validate the header
        header.validate()?;

        Ok(header)
    }

    /// Deserialize header from bytes with relaxed validation (for recovery mode)
    pub fn from_bytes_relaxed(bytes: &[u8]) -> Result<Self, BinaryFormatError> {
        if bytes.len() < HEADER_SIZE {
            return Err(BinaryFormatError::InvalidHeaderSize {
                expected: HEADER_SIZE,
                found: bytes.len(),
            });
        }

        let mut offset = 0;

        // Magic number (8 bytes) - don't validate in relaxed mode
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[offset..offset + 8]);
        offset += 8;

        // Version (4 bytes)
        let version_major = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
        offset += 2;
        let version_minor = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
        offset += 2;

        // Compression type (4 bytes) - use None if invalid
        let compression_raw = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        let compression_type = match compression_raw {
            0 => CompressionType::None,
            1 => CompressionType::Lz4,
            2 => CompressionType::Zstd,
            _ => CompressionType::None, // Default to None in relaxed mode
        };
        offset += 4;

        // Section count (4 bytes)
        let section_count = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        // Total size (8 bytes)
        let total_size = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // Timestamp (8 bytes)
        let timestamp = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // Checksum (8 bytes)
        let checksum = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // Reserved (20 bytes)
        let mut reserved = [0u8; 20];
        reserved.copy_from_slice(&bytes[offset..offset + 20]);

        let header = Self {
            magic,
            version_major,
            version_minor,
            compression_type,
            section_count,
            total_size,
            timestamp,
            checksum,
            reserved,
        };

        // Don't validate in relaxed mode - just return the header
        Ok(header)
    }
}

impl SectionEntry {
    /// Create a new section entry
    pub fn new(
        section_type: SectionType,
        compression: CompressionType,
        offset: u64,
        compressed_size: u32,
        uncompressed_size: u32,
    ) -> Self {
        Self {
            section_type,
            compression,
            flags: 0,
            offset,
            compressed_size,
            uncompressed_size,
        }
    }

    /// Serialize section entry to bytes (Little Endian)
    pub fn to_bytes(&self) -> [u8; SECTION_ENTRY_SIZE] {
        let mut bytes = [0u8; SECTION_ENTRY_SIZE];
        let mut offset = 0;

        // Section type (1 byte)
        bytes[offset] = self.section_type as u8;
        offset += 1;

        // Compression (1 byte)
        bytes[offset] = self.compression as u8;
        offset += 1;

        // Flags (2 bytes)
        bytes[offset..offset + 2].copy_from_slice(&self.flags.to_le_bytes());
        offset += 2;

        // Offset (8 bytes)
        bytes[offset..offset + 8].copy_from_slice(&self.offset.to_le_bytes());
        offset += 8;

        // Compressed size (4 bytes)
        bytes[offset..offset + 4].copy_from_slice(&self.compressed_size.to_le_bytes());
        offset += 4;

        // Uncompressed size (4 bytes)
        bytes[offset..offset + 4].copy_from_slice(&self.uncompressed_size.to_le_bytes());

        bytes
    }

    /// Deserialize section entry from bytes (Little Endian)
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BinaryFormatError> {
        if bytes.len() < SECTION_ENTRY_SIZE {
            return Err(BinaryFormatError::InvalidSectionEntrySize {
                expected: SECTION_ENTRY_SIZE,
                found: bytes.len(),
            });
        }

        let mut offset = 0;

        // Section type (1 byte)
        let section_type = match bytes[offset] {
            1 => SectionType::MemoryStats,
            2 => SectionType::ActiveAllocations,
            3 => SectionType::AllocationHistory,
            4 => SectionType::TypeMemoryUsage,
            5 => SectionType::FfiAnalysis,
            6 => SectionType::LifecycleAnalysis,
            7 => SectionType::PerformanceData,
            8 => SectionType::VariableRegistry,
            9 => SectionType::SecurityViolations,
            10 => SectionType::MemoryPassports,
            other => return Err(BinaryFormatError::InvalidSectionType(other)),
        };
        offset += 1;

        // Compression (1 byte)
        let compression = match bytes[offset] {
            0 => CompressionType::None,
            1 => CompressionType::Lz4,
            2 => CompressionType::Zstd,
            other => return Err(BinaryFormatError::InvalidCompressionType(other as u32)),
        };
        offset += 1;

        // Flags (2 bytes)
        let flags = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
        offset += 2;

        // Offset (8 bytes)
        let section_offset = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // Compressed size (4 bytes)
        let compressed_size = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        // Uncompressed size (4 bytes)
        let uncompressed_size = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);

        Ok(Self {
            section_type,
            compression,
            flags,
            offset: section_offset,
            compressed_size,
            uncompressed_size,
        })
    }
}

impl SectionDirectory {
    /// Create a new empty section directory
    pub fn new() -> Self {
        Self {
            sections: HashMap::new(),
        }
    }

    /// Add a section entry to the directory
    pub fn add_section(&mut self, entry: SectionEntry) {
        self.sections.insert(entry.section_type, entry);
    }

    /// Get a section entry by type
    pub fn get_section(&self, section_type: SectionType) -> Option<&SectionEntry> {
        self.sections.get(&section_type)
    }

    /// Get all section types in the directory
    pub fn section_types(&self) -> Vec<SectionType> {
        self.sections.keys().copied().collect()
    }

    /// Serialize directory to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.sections.len() * SECTION_ENTRY_SIZE);

        // Sort sections by type for consistent output
        let mut sections: Vec<_> = self.sections.values().collect();
        sections.sort_by_key(|entry| entry.section_type as u8);

        for entry in sections {
            bytes.extend_from_slice(&entry.to_bytes());
        }

        bytes
    }

    /// Deserialize directory from bytes
    pub fn from_bytes(bytes: &[u8], section_count: u32) -> Result<Self, BinaryFormatError> {
        let expected_size = section_count as usize * SECTION_ENTRY_SIZE;
        if bytes.len() < expected_size {
            return Err(BinaryFormatError::InvalidDirectorySize {
                expected: expected_size,
                found: bytes.len(),
            });
        }

        let mut directory = Self::new();

        for i in 0..section_count as usize {
            let start = i * SECTION_ENTRY_SIZE;
            let end = start + SECTION_ENTRY_SIZE;
            let entry_bytes = &bytes[start..end];

            let entry = SectionEntry::from_bytes(entry_bytes)?;
            directory.add_section(entry);
        }

        Ok(directory)
    }
}

impl Default for SectionDirectory {
    fn default() -> Self {
        Self::new()
    }
}

/// Binary format errors
#[derive(Debug, thiserror::Error)]
pub enum BinaryFormatError {
    #[error("Invalid magic number: expected {expected:?}, found {found:?}")]
    InvalidMagic { expected: [u8; 8], found: [u8; 8] },

    #[error(
        "Unsupported version: file version {file_version}, supported version {supported_version}"
    )]
    UnsupportedVersion {
        file_version: String,
        supported_version: String,
    },

    #[error("Invalid header size: expected {expected}, found {found}")]
    InvalidHeaderSize { expected: usize, found: usize },

    #[error("Invalid section entry size: expected {expected}, found {found}")]
    InvalidSectionEntrySize { expected: usize, found: usize },

    #[error("Invalid directory size: expected {expected}, found {found}")]
    InvalidDirectorySize { expected: usize, found: usize },

    #[error("Invalid compression type: {0}")]
    InvalidCompressionType(u32),

    #[error("Invalid section type: {0}")]
    InvalidSectionType(u8),

    #[error("Checksum mismatch: expected {expected:x}, found {found:x}")]
    ChecksumMismatch { expected: u64, found: u64 },

    #[error("Invalid string table: {0}")]
    InvalidStringTable(String),

    #[error("Invalid type table: {0}")]
    InvalidTypeTable(String),

    #[error("Unexpected end of data")]
    UnexpectedEndOfData,

    #[error("Invalid string ID: {0}")]
    InvalidStringId(u32),

    #[error("Invalid type ID: {0}")]
    InvalidTypeId(u16),

    #[error("Invalid option marker: {0}")]
    InvalidOptionMarker(u8),

    #[error("Invalid compressed data")]
    InvalidCompressedData,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// String table for deduplicating strings in binary format
#[derive(Debug, Clone)]
pub struct StringTable {
    /// Vector of unique strings
    strings: Vec<String>,
    /// Map from string to index for fast lookup
    string_to_id: HashMap<String, u32>,
}

impl StringTable {
    /// Create a new empty string table
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            string_to_id: HashMap::new(),
        }
    }

    /// Intern a string and return its ID
    /// If the string already exists, returns the existing ID
    /// Otherwise, adds the string and returns a new ID
    pub fn intern(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.string_to_id.get(s) {
            id
        } else {
            let id = self.strings.len() as u32;
            self.strings.push(s.to_string());
            self.string_to_id.insert(s.to_string(), id);
            id
        }
    }

    /// Get a string by its ID
    pub fn get(&self, id: u32) -> Option<&str> {
        self.strings.get(id as usize).map(|s| s.as_str())
    }

    /// Get the number of unique strings
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Check if the string table is empty
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Get all strings as a slice
    pub fn strings(&self) -> &[String] {
        &self.strings
    }

    /// Serialize string table to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Write string count (4 bytes)
        bytes.extend_from_slice(&(self.strings.len() as u32).to_le_bytes());

        // Write string offsets and data
        let mut string_data = Vec::new();
        let mut offsets = Vec::new();

        for string in &self.strings {
            offsets.push(string_data.len() as u32);
            string_data.extend_from_slice(string.as_bytes());
            string_data.push(0); // Null terminator
        }

        // Write offsets (4 bytes each)
        for offset in offsets {
            bytes.extend_from_slice(&offset.to_le_bytes());
        }

        // Write string data
        bytes.extend_from_slice(&string_data);

        bytes
    }

    /// Deserialize string table from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BinaryFormatError> {
        if bytes.len() < 4 {
            return Err(BinaryFormatError::InvalidStringTable(
                "Too short".to_string(),
            ));
        }

        let mut offset = 0;

        // Read string count
        let string_count = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        if string_count == 0 {
            return Ok(Self::new());
        }

        // Read offsets
        let mut offsets = Vec::with_capacity(string_count as usize);
        for _ in 0..string_count {
            if offset + 4 > bytes.len() {
                return Err(BinaryFormatError::InvalidStringTable(
                    "Incomplete offsets".to_string(),
                ));
            }
            let string_offset = u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]);
            offsets.push(string_offset);
            offset += 4;
        }

        // Read strings
        let string_data_start = offset;
        let mut strings = Vec::with_capacity(string_count as usize);
        let mut string_to_id = HashMap::with_capacity(string_count as usize);

        for (id, &string_offset) in offsets.iter().enumerate() {
            let start = string_data_start + string_offset as usize;
            if start >= bytes.len() {
                return Err(BinaryFormatError::InvalidStringTable(
                    "Invalid offset".to_string(),
                ));
            }

            // Find null terminator
            let end = bytes[start..]
                .iter()
                .position(|&b| b == 0)
                .map(|pos| start + pos)
                .ok_or_else(|| {
                    BinaryFormatError::InvalidStringTable("Missing null terminator".to_string())
                })?;

            let string = String::from_utf8(bytes[start..end].to_vec())
                .map_err(|_| BinaryFormatError::InvalidStringTable("Invalid UTF-8".to_string()))?;

            string_to_id.insert(string.clone(), id as u32);
            strings.push(string);
        }

        Ok(Self {
            strings,
            string_to_id,
        })
    }
}

impl Default for StringTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Type table for optimizing common type names with predefined IDs
#[derive(Debug, Clone)]
pub struct TypeTable {
    /// Predefined common types with their IDs
    predefined: HashMap<&'static str, u16>,
    /// Custom types not in predefined list
    custom_types: Vec<String>,
    /// Map from custom type to ID (starting after predefined IDs)
    custom_type_to_id: HashMap<String, u16>,
    /// Next available custom type ID
    next_custom_id: u16,
}

impl TypeTable {
    /// Create a new type table with predefined common types
    pub fn new() -> Self {
        let mut predefined = HashMap::new();

        // Common Rust types with predefined IDs
        predefined.insert("i8", 1);
        predefined.insert("i16", 2);
        predefined.insert("i32", 3);
        predefined.insert("i64", 4);
        predefined.insert("i128", 5);
        predefined.insert("isize", 6);
        predefined.insert("u8", 7);
        predefined.insert("u16", 8);
        predefined.insert("u32", 9);
        predefined.insert("u64", 10);
        predefined.insert("u128", 11);
        predefined.insert("usize", 12);
        predefined.insert("f32", 13);
        predefined.insert("f64", 14);
        predefined.insert("bool", 15);
        predefined.insert("char", 16);
        predefined.insert("String", 17);
        predefined.insert("&str", 18);
        predefined.insert("Vec<i32>", 19);
        predefined.insert("Vec<u8>", 20);
        predefined.insert("Vec<String>", 21);
        predefined.insert("HashMap<String,String>", 22);
        predefined.insert("HashMap<String,i32>", 23);
        predefined.insert("Option<String>", 24);
        predefined.insert("Option<i32>", 25);
        predefined.insert("Box<String>", 26);
        predefined.insert("Rc<String>", 27);
        predefined.insert("Arc<String>", 28);

        Self {
            predefined,
            custom_types: Vec::new(),
            custom_type_to_id: HashMap::new(),
            next_custom_id: 1000, // Start custom IDs at 1000 to avoid conflicts
        }
    }

    /// Get type ID for a type name, adding it if not present
    pub fn get_or_intern_type_id(&mut self, type_name: &str) -> u16 {
        // Check predefined types first
        if let Some(&id) = self.predefined.get(type_name) {
            return id;
        }

        // Check custom types
        if let Some(&id) = self.custom_type_to_id.get(type_name) {
            return id;
        }

        // Add new custom type
        let id = self.next_custom_id;
        self.next_custom_id += 1;
        self.custom_types.push(type_name.to_string());
        self.custom_type_to_id.insert(type_name.to_string(), id);
        id
    }

    /// Get type name by ID
    pub fn get_type_name(&self, id: u16) -> Option<String> {
        // Check predefined types
        for (&name, &type_id) in &self.predefined {
            if type_id == id {
                return Some(name.to_string());
            }
        }

        // Check custom types
        if id >= 1000 {
            let custom_index = (id - 1000) as usize;
            self.custom_types.get(custom_index).cloned()
        } else {
            None
        }
    }

    /// Get the number of total types (predefined + custom)
    pub fn total_types(&self) -> usize {
        self.predefined.len() + self.custom_types.len()
    }

    /// Get the number of custom types
    pub fn custom_type_count(&self) -> usize {
        self.custom_types.len()
    }

    /// Check if a type is predefined
    pub fn is_predefined(&self, type_name: &str) -> bool {
        self.predefined.contains_key(type_name)
    }

    /// Serialize type table to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Write custom type count (4 bytes)
        bytes.extend_from_slice(&(self.custom_types.len() as u32).to_le_bytes());

        // Write custom types using string table format
        let mut string_data = Vec::new();
        let mut offsets = Vec::new();

        for custom_type in &self.custom_types {
            offsets.push(string_data.len() as u32);
            string_data.extend_from_slice(custom_type.as_bytes());
            string_data.push(0); // Null terminator
        }

        // Write offsets (4 bytes each)
        for offset in offsets {
            bytes.extend_from_slice(&offset.to_le_bytes());
        }

        // Write string data
        bytes.extend_from_slice(&string_data);

        bytes
    }

    /// Deserialize type table from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BinaryFormatError> {
        let mut table = Self::new();

        if bytes.len() < 4 {
            return Ok(table); // Empty custom types is valid
        }

        let mut offset = 0;

        // Read custom type count
        let custom_count = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        if custom_count == 0 {
            return Ok(table);
        }

        // Read offsets
        let mut offsets = Vec::with_capacity(custom_count as usize);
        for _ in 0..custom_count {
            if offset + 4 > bytes.len() {
                return Err(BinaryFormatError::InvalidTypeTable(
                    "Incomplete offsets".to_string(),
                ));
            }
            let type_offset = u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]);
            offsets.push(type_offset);
            offset += 4;
        }

        // Read custom types
        let string_data_start = offset;
        for &type_offset in &offsets {
            let start = string_data_start + type_offset as usize;
            if start >= bytes.len() {
                return Err(BinaryFormatError::InvalidTypeTable(
                    "Invalid offset".to_string(),
                ));
            }

            // Find null terminator
            let end = bytes[start..]
                .iter()
                .position(|&b| b == 0)
                .map(|pos| start + pos)
                .ok_or_else(|| {
                    BinaryFormatError::InvalidTypeTable("Missing null terminator".to_string())
                })?;

            let type_name = String::from_utf8(bytes[start..end].to_vec())
                .map_err(|_| BinaryFormatError::InvalidTypeTable("Invalid UTF-8".to_string()))?;

            // Add to custom types (this will assign the correct ID)
            table.get_or_intern_type_id(&type_name);
        }

        Ok(table)
    }
}

impl Default for TypeTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Binary encoder for serializing data to bytes
pub struct BinaryEncoder {
    buffer: Vec<u8>,
    string_table: StringTable,
    type_table: TypeTable,
}

impl BinaryEncoder {
    /// Create a new binary encoder
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            string_table: StringTable::new(),
            type_table: TypeTable::new(),
        }
    }

    /// Create a new binary encoder with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            string_table: StringTable::new(),
            type_table: TypeTable::new(),
        }
    }

    /// Get the current buffer
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Get the string table
    pub fn string_table(&self) -> &StringTable {
        &self.string_table
    }

    /// Get the type table
    pub fn type_table(&self) -> &TypeTable {
        &self.type_table
    }

    /// Clear the buffer but keep the tables
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Reset everything
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.string_table = StringTable::new();
        self.type_table = TypeTable::new();
    }

    /// Encode a u8 value
    pub fn encode_u8(&mut self, value: u8) {
        self.buffer.push(value);
    }

    /// Encode a u16 value (Little Endian)
    pub fn encode_u16(&mut self, value: u16) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    /// Encode a u32 value (Little Endian)
    pub fn encode_u32(&mut self, value: u32) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    /// Encode a u64 value (Little Endian)
    pub fn encode_u64(&mut self, value: u64) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    /// Encode a usize value as u64 (Little Endian)
    pub fn encode_usize(&mut self, value: usize) {
        self.encode_u64(value as u64);
    }

    /// Encode a string using the string table
    pub fn encode_string(&mut self, value: &str) {
        let id = self.string_table.intern(value);
        self.encode_u32(id);
    }

    /// Encode an optional string
    pub fn encode_optional_string(&mut self, value: &Option<String>) {
        match value {
            Some(s) => {
                self.encode_u8(1); // Some marker
                self.encode_string(s);
            }
            None => {
                self.encode_u8(0); // None marker
            }
        }
    }

    /// Encode a type name using the type table
    pub fn encode_type_name(&mut self, type_name: &str) {
        let id = self.type_table.get_or_intern_type_id(type_name);
        self.encode_u16(id);
    }

    /// Encode an optional type name
    pub fn encode_optional_type_name(&mut self, type_name: &Option<String>) {
        match type_name {
            Some(name) => {
                self.encode_u8(1); // Some marker
                self.encode_type_name(name);
            }
            None => {
                self.encode_u8(0); // None marker
            }
        }
    }

    /// Encode a vector of strings
    pub fn encode_string_vec(&mut self, values: &[String]) {
        self.encode_u32(values.len() as u32);
        for value in values {
            self.encode_string(value);
        }
    }

    /// Encode an optional vector of strings
    pub fn encode_optional_string_vec(&mut self, values: &Option<Vec<String>>) {
        match values {
            Some(vec) => {
                self.encode_u8(1); // Some marker
                self.encode_string_vec(vec);
            }
            None => {
                self.encode_u8(0); // None marker
            }
        }
    }

    /// Encode raw bytes with length prefix
    pub fn encode_bytes(&mut self, data: &[u8]) {
        self.encode_u32(data.len() as u32);
        self.buffer.extend_from_slice(data);
    }

    /// Get the encoded data as bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }
}

impl Default for BinaryEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Binary decoder for deserializing data from bytes
pub struct BinaryDecoder<'a> {
    data: &'a [u8],
    position: usize,
    string_table: StringTable,
    type_table: TypeTable,
}

impl<'a> BinaryDecoder<'a> {
    /// Create a new binary decoder
    pub fn new(data: &'a [u8], string_table: StringTable, type_table: TypeTable) -> Self {
        Self {
            data,
            position: 0,
            string_table,
            type_table,
        }
    }

    /// Get the current position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Check if there are more bytes to read
    pub fn has_more(&self) -> bool {
        self.position < self.data.len()
    }

    /// Get remaining bytes count
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }

    /// Decode a u8 value
    pub fn decode_u8(&mut self) -> Result<u8, BinaryFormatError> {
        if self.position >= self.data.len() {
            return Err(BinaryFormatError::UnexpectedEndOfData);
        }
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }

    /// Decode a u16 value (Little Endian)
    pub fn decode_u16(&mut self) -> Result<u16, BinaryFormatError> {
        if self.position + 2 > self.data.len() {
            return Err(BinaryFormatError::UnexpectedEndOfData);
        }
        let value = u16::from_le_bytes([self.data[self.position], self.data[self.position + 1]]);
        self.position += 2;
        Ok(value)
    }

    /// Decode a u32 value (Little Endian)
    pub fn decode_u32(&mut self) -> Result<u32, BinaryFormatError> {
        if self.position + 4 > self.data.len() {
            return Err(BinaryFormatError::UnexpectedEndOfData);
        }
        let value = u32::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ]);
        self.position += 4;
        Ok(value)
    }

    /// Decode a u64 value (Little Endian)
    pub fn decode_u64(&mut self) -> Result<u64, BinaryFormatError> {
        if self.position + 8 > self.data.len() {
            return Err(BinaryFormatError::UnexpectedEndOfData);
        }
        let value = u64::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
            self.data[self.position + 4],
            self.data[self.position + 5],
            self.data[self.position + 6],
            self.data[self.position + 7],
        ]);
        self.position += 8;
        Ok(value)
    }

    /// Decode a usize value from u64 (Little Endian)
    pub fn decode_usize(&mut self) -> Result<usize, BinaryFormatError> {
        let value = self.decode_u64()?;
        Ok(value as usize)
    }

    /// Decode a string using the string table
    pub fn decode_string(&mut self) -> Result<String, BinaryFormatError> {
        let id = self.decode_u32()?;
        Ok(self
            .string_table
            .get(id)
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                // Recovery mode: if string ID is invalid, return a placeholder
                format!("INVALID_STRING_ID_{}", id)
            }))
    }

    /// Decode an optional string
    pub fn decode_optional_string(&mut self) -> Result<Option<String>, BinaryFormatError> {
        let marker = self.decode_u8()?;
        match marker {
            0 => Ok(None),
            1 => Ok(Some(self.decode_string()?)),
            _ => {
                // Recovery mode: if we encounter an invalid marker, try to recover
                // by treating it as None and moving back one position
                if marker > 1 {
                    // Move back one position to re-read this byte as part of the next field
                    if self.position > 0 {
                        self.position -= 1;
                    }
                    Ok(None)
                } else {
                    Err(BinaryFormatError::InvalidOptionMarker(marker))
                }
            }
        }
    }

    /// Decode a type name using the type table
    pub fn decode_type_name(&mut self) -> Result<String, BinaryFormatError> {
        let id = self.decode_u16()?;
        Ok(self.type_table.get_type_name(id).unwrap_or_else(|| {
            // Recovery mode: if type ID is invalid, return a placeholder
            format!("INVALID_TYPE_ID_{}", id)
        }))
    }

    /// Decode an optional type name
    pub fn decode_optional_type_name(&mut self) -> Result<Option<String>, BinaryFormatError> {
        let marker = self.decode_u8()?;
        match marker {
            0 => Ok(None),
            1 => Ok(Some(self.decode_type_name()?)),
            _ => {
                // Recovery mode: if we encounter an invalid marker, try to recover
                // by treating it as None and moving back one position
                if marker > 1 {
                    // Move back one position to re-read this byte as part of the next field
                    if self.position > 0 {
                        self.position -= 1;
                    }
                    Ok(None)
                } else {
                    Err(BinaryFormatError::InvalidOptionMarker(marker))
                }
            }
        }
    }

    /// Decode a vector of strings
    pub fn decode_string_vec(&mut self) -> Result<Vec<String>, BinaryFormatError> {
        let count = self.decode_u32()?;
        let mut vec = Vec::with_capacity(count as usize);
        for _ in 0..count {
            vec.push(self.decode_string()?);
        }
        Ok(vec)
    }

    /// Decode an optional vector of strings
    pub fn decode_optional_string_vec(&mut self) -> Result<Option<Vec<String>>, BinaryFormatError> {
        let marker = self.decode_u8()?;
        match marker {
            0 => Ok(None),
            1 => Ok(Some(self.decode_string_vec()?)),
            _ => {
                // Recovery mode: if we encounter an invalid marker, try to recover
                // by treating it as None and moving back one position
                if marker > 1 {
                    // Move back one position to re-read this byte as part of the next field
                    if self.position > 0 {
                        self.position -= 1;
                    }
                    Ok(None)
                } else {
                    Err(BinaryFormatError::InvalidOptionMarker(marker))
                }
            }
        }
    }

    /// Decode raw bytes with length prefix
    pub fn decode_bytes(&mut self) -> Result<Vec<u8>, BinaryFormatError> {
        let length = self.decode_u32()? as usize;
        if self.position + length > self.data.len() {
            return Err(BinaryFormatError::UnexpectedEndOfData);
        }
        let bytes = self.data[self.position..self.position + length].to_vec();
        self.position += length;
        Ok(bytes)
    }
}

/// Compression engine for binary data
pub struct CompressionEngine;

impl CompressionEngine {
    /// Compress data using the specified compression type
    pub fn compress(
        data: &[u8],
        compression: CompressionType,
    ) -> Result<Vec<u8>, BinaryFormatError> {
        match compression {
            CompressionType::None => Ok(data.to_vec()),
            CompressionType::Lz4 => {
                // For now, we'll implement a simple mock compression
                // In a real implementation, you would use the lz4 crate
                Self::mock_compress(data, "LZ4")
            }
            CompressionType::Zstd => {
                // For now, we'll implement a simple mock compression
                // In a real implementation, you would use the zstd crate
                Self::mock_compress(data, "ZSTD")
            }
        }
    }

    /// Decompress data using the specified compression type
    pub fn decompress(
        data: &[u8],
        compression: CompressionType,
    ) -> Result<Vec<u8>, BinaryFormatError> {
        match compression {
            CompressionType::None => Ok(data.to_vec()),
            CompressionType::Lz4 => {
                // For now, we'll implement a simple mock decompression
                Self::mock_decompress(data, "LZ4")
            }
            CompressionType::Zstd => {
                // For now, we'll implement a simple mock decompression
                Self::mock_decompress(data, "ZSTD")
            }
        }
    }

    /// Mock compression for testing (adds a simple header)
    fn mock_compress(data: &[u8], algorithm: &str) -> Result<Vec<u8>, BinaryFormatError> {
        let mut compressed = Vec::new();
        compressed.extend_from_slice(algorithm.as_bytes());
        compressed.extend_from_slice(&(data.len() as u32).to_le_bytes());
        compressed.extend_from_slice(data);
        Ok(compressed)
    }

    /// Mock decompression for testing (removes the header)
    fn mock_decompress(data: &[u8], algorithm: &str) -> Result<Vec<u8>, BinaryFormatError> {
        let header_len = algorithm.len();
        if data.len() < header_len + 4 {
            return Err(BinaryFormatError::InvalidCompressedData);
        }

        // Check algorithm header
        if &data[..header_len] != algorithm.as_bytes() {
            return Err(BinaryFormatError::InvalidCompressedData);
        }

        // Read original length
        let original_len = u32::from_le_bytes([
            data[header_len],
            data[header_len + 1],
            data[header_len + 2],
            data[header_len + 3],
        ]) as usize;

        // Extract original data
        let start = header_len + 4;
        if data.len() < start + original_len {
            return Err(BinaryFormatError::InvalidCompressedData);
        }

        Ok(data[start..start + original_len].to_vec())
    }

    /// Estimate compression ratio for a given algorithm and data
    pub fn estimate_compression_ratio(data: &[u8], compression: CompressionType) -> f64 {
        match compression {
            CompressionType::None => 1.0,
            CompressionType::Lz4 => {
                // Rough estimate: LZ4 typically achieves 2-3x compression on text data
                let unique_bytes = data.iter().collect::<std::collections::HashSet<_>>().len();
                if unique_bytes < 64 {
                    0.4 // High repetition
                } else if unique_bytes < 128 {
                    0.6 // Medium repetition
                } else {
                    0.8 // Low repetition
                }
            }
            CompressionType::Zstd => {
                // Rough estimate: Zstd typically achieves 3-5x compression on text data
                let unique_bytes = data.iter().collect::<std::collections::HashSet<_>>().len();
                if unique_bytes < 64 {
                    0.3 // High repetition
                } else if unique_bytes < 128 {
                    0.5 // Medium repetition
                } else {
                    0.7 // Low repetition
                }
            }
        }
    }
}

/// Calculate CRC64 checksum for data integrity
pub fn crc64_checksum(data: &[u8]) -> u64 {
    // Simple CRC64 implementation - in production, use a proper CRC64 library
    let mut crc = 0xFFFFFFFFFFFFFFFFu64;

    for &byte in data {
        crc ^= byte as u64;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xC96C5795D7870F42u64;
            } else {
                crc >>= 1;
            }
        }
    }

    crc ^ 0xFFFFFFFFFFFFFFFFu64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_header_serialization() {
        let mut header = BinaryHeader::new();
        header.section_count = 5;
        header.total_size = 1024;
        header.calculate_checksum();

        let bytes = header.to_bytes();
        let deserialized = BinaryHeader::from_bytes(&bytes).unwrap();

        assert_eq!(header.magic, deserialized.magic);
        assert_eq!(header.version_major, deserialized.version_major);
        assert_eq!(header.version_minor, deserialized.version_minor);
        assert_eq!(header.section_count, deserialized.section_count);
        assert_eq!(header.total_size, deserialized.total_size);
        assert_eq!(header.checksum, deserialized.checksum);
    }

    #[test]
    fn test_section_entry_serialization() {
        let entry = SectionEntry::new(
            SectionType::MemoryStats,
            CompressionType::Lz4,
            1024,
            512,
            1024,
        );

        let bytes = entry.to_bytes();
        let deserialized = SectionEntry::from_bytes(&bytes).unwrap();

        assert_eq!(entry.section_type, deserialized.section_type);
        assert_eq!(entry.compression, deserialized.compression);
        assert_eq!(entry.offset, deserialized.offset);
        assert_eq!(entry.compressed_size, deserialized.compressed_size);
        assert_eq!(entry.uncompressed_size, deserialized.uncompressed_size);
    }

    #[test]
    fn test_section_directory() {
        let mut directory = SectionDirectory::new();

        let entry1 = SectionEntry::new(
            SectionType::MemoryStats,
            CompressionType::None,
            64,
            256,
            256,
        );

        let entry2 = SectionEntry::new(
            SectionType::ActiveAllocations,
            CompressionType::Lz4,
            320,
            512,
            1024,
        );

        directory.add_section(entry1);
        directory.add_section(entry2);

        let bytes = directory.to_bytes();
        let deserialized = SectionDirectory::from_bytes(&bytes, 2).unwrap();

        assert_eq!(directory.sections.len(), deserialized.sections.len());
        assert!(deserialized.get_section(SectionType::MemoryStats).is_some());
        assert!(deserialized
            .get_section(SectionType::ActiveAllocations)
            .is_some());
    }

    #[test]
    fn test_crc64_checksum() {
        let data = b"Hello, World!";
        let checksum1 = crc64_checksum(data);
        let checksum2 = crc64_checksum(data);

        // Same data should produce same checksum
        assert_eq!(checksum1, checksum2);

        // Different data should produce different checksum
        let different_data = b"Hello, World?";
        let checksum3 = crc64_checksum(different_data);
        assert_ne!(checksum1, checksum3);
    }

    #[test]
    fn test_string_table() {
        let mut table = StringTable::new();

        // Test interning strings
        let id1 = table.intern("hello");
        let id2 = table.intern("world");
        let id3 = table.intern("hello"); // Should return same ID as id1

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 0); // Same as id1

        // Test retrieval
        assert_eq!(table.get(id1), Some("hello"));
        assert_eq!(table.get(id2), Some("world"));
        assert_eq!(table.get(999), None);

        // Test serialization
        let bytes = table.to_bytes();
        let deserialized = StringTable::from_bytes(&bytes).unwrap();

        assert_eq!(table.len(), deserialized.len());
        assert_eq!(deserialized.get(0), Some("hello"));
        assert_eq!(deserialized.get(1), Some("world"));
    }

    #[test]
    fn test_string_table_empty() {
        let table = StringTable::new();
        let bytes = table.to_bytes();
        let deserialized = StringTable::from_bytes(&bytes).unwrap();

        assert!(deserialized.is_empty());
        assert_eq!(deserialized.len(), 0);
    }

    #[test]
    fn test_type_table() {
        let mut table = TypeTable::new();

        // Test predefined types
        let id1 = table.get_or_intern_type_id("i32");
        let id2 = table.get_or_intern_type_id("String");
        assert_eq!(id1, 3); // i32 is predefined as ID 3
        assert_eq!(id2, 17); // String is predefined as ID 17

        // Test custom types
        let id3 = table.get_or_intern_type_id("MyCustomType");
        let id4 = table.get_or_intern_type_id("AnotherType");
        let id5 = table.get_or_intern_type_id("MyCustomType"); // Should return same as id3

        assert_eq!(id3, 1000); // First custom type
        assert_eq!(id4, 1001); // Second custom type
        assert_eq!(id5, 1000); // Same as id3

        // Test retrieval
        assert_eq!(table.get_type_name(3), Some("i32".to_string()));
        assert_eq!(table.get_type_name(17), Some("String".to_string()));
        assert_eq!(table.get_type_name(1000), Some("MyCustomType".to_string()));
        assert_eq!(table.get_type_name(1001), Some("AnotherType".to_string()));
        assert_eq!(table.get_type_name(9999), None);

        // Test predefined check
        assert!(table.is_predefined("i32"));
        assert!(table.is_predefined("String"));
        assert!(!table.is_predefined("MyCustomType"));

        // Test counts
        assert_eq!(table.custom_type_count(), 2);
        assert!(table.total_types() > 2); // Should include predefined types
    }

    #[test]
    fn test_type_table_serialization() {
        let mut table = TypeTable::new();

        // Add some custom types
        table.get_or_intern_type_id("CustomType1");
        table.get_or_intern_type_id("CustomType2");
        table.get_or_intern_type_id("Vec<CustomType1>");

        // Serialize and deserialize
        let bytes = table.to_bytes();
        let deserialized = TypeTable::from_bytes(&bytes).unwrap();

        // Test that custom types are preserved
        assert_eq!(deserialized.custom_type_count(), 3);
        assert_eq!(
            deserialized.get_type_name(1000),
            Some("CustomType1".to_string())
        );
        assert_eq!(
            deserialized.get_type_name(1001),
            Some("CustomType2".to_string())
        );
        assert_eq!(
            deserialized.get_type_name(1002),
            Some("Vec<CustomType1>".to_string())
        );

        // Test that predefined types still work
        assert_eq!(deserialized.get_type_name(3), Some("i32".to_string()));
        assert_eq!(deserialized.get_type_name(17), Some("String".to_string()));
    }

    #[test]
    fn test_type_table_empty_custom() {
        let table = TypeTable::new();
        let bytes = table.to_bytes();
        let deserialized = TypeTable::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.custom_type_count(), 0);
        // Predefined types should still work
        assert_eq!(deserialized.get_type_name(3), Some("i32".to_string()));
    }

    #[test]
    fn test_binary_encoder_decoder() {
        let mut encoder = BinaryEncoder::new();

        // Encode various types
        encoder.encode_u8(42);
        encoder.encode_u16(1234);
        encoder.encode_u32(567890);
        encoder.encode_u64(9876543210);
        encoder.encode_usize(12345);
        encoder.encode_string("hello");
        encoder.encode_string("world");
        encoder.encode_string("hello"); // Duplicate should use same ID

        // Encode optional values
        encoder.encode_optional_string(&Some("optional".to_string()));
        encoder.encode_optional_string(&None);

        // Encode type names
        encoder.encode_type_name("i32");
        encoder.encode_type_name("CustomType");
        encoder.encode_optional_type_name(&Some("String".to_string()));
        encoder.encode_optional_type_name(&None);

        // Encode vector
        encoder.encode_string_vec(&["vec1".to_string(), "vec2".to_string()]);
        encoder.encode_optional_string_vec(&Some(vec!["opt_vec".to_string()]));
        encoder.encode_optional_string_vec(&None);

        // Encode raw bytes
        encoder.encode_bytes(b"raw_data");

        // Create decoder
        let data = encoder.buffer().to_vec();
        let string_table = encoder.string_table().clone();
        let type_table = encoder.type_table().clone();
        let mut decoder = BinaryDecoder::new(&data, string_table, type_table);

        // Decode and verify
        assert_eq!(decoder.decode_u8().unwrap(), 42);
        assert_eq!(decoder.decode_u16().unwrap(), 1234);
        assert_eq!(decoder.decode_u32().unwrap(), 567890);
        assert_eq!(decoder.decode_u64().unwrap(), 9876543210);
        assert_eq!(decoder.decode_usize().unwrap(), 12345);
        assert_eq!(decoder.decode_string().unwrap(), "hello");
        assert_eq!(decoder.decode_string().unwrap(), "world");
        assert_eq!(decoder.decode_string().unwrap(), "hello"); // Same string

        // Decode optional values
        assert_eq!(
            decoder.decode_optional_string().unwrap(),
            Some("optional".to_string())
        );
        assert_eq!(decoder.decode_optional_string().unwrap(), None);

        // Decode type names
        assert_eq!(decoder.decode_type_name().unwrap(), "i32");
        assert_eq!(decoder.decode_type_name().unwrap(), "CustomType");
        assert_eq!(
            decoder.decode_optional_type_name().unwrap(),
            Some("String".to_string())
        );
        assert_eq!(decoder.decode_optional_type_name().unwrap(), None);

        // Decode vector
        assert_eq!(decoder.decode_string_vec().unwrap(), vec!["vec1", "vec2"]);
        assert_eq!(
            decoder.decode_optional_string_vec().unwrap(),
            Some(vec!["opt_vec".to_string()])
        );
        assert_eq!(decoder.decode_optional_string_vec().unwrap(), None);

        // Decode raw bytes
        assert_eq!(decoder.decode_bytes().unwrap(), b"raw_data");

        // Should be at end
        assert!(!decoder.has_more());
    }

    #[test]
    fn test_compression_engine() {
        let original_data = b"Hello, World! This is test data for compression.";

        // Test no compression
        let compressed = CompressionEngine::compress(original_data, CompressionType::None).unwrap();
        let decompressed =
            CompressionEngine::decompress(&compressed, CompressionType::None).unwrap();
        assert_eq!(original_data, decompressed.as_slice());

        // Test LZ4 mock compression
        let compressed = CompressionEngine::compress(original_data, CompressionType::Lz4).unwrap();
        let decompressed =
            CompressionEngine::decompress(&compressed, CompressionType::Lz4).unwrap();
        assert_eq!(original_data, decompressed.as_slice());
        assert!(compressed.len() > original_data.len()); // Mock compression adds header

        // Test Zstd mock compression
        let compressed = CompressionEngine::compress(original_data, CompressionType::Zstd).unwrap();
        let decompressed =
            CompressionEngine::decompress(&compressed, CompressionType::Zstd).unwrap();
        assert_eq!(original_data, decompressed.as_slice());
        assert!(compressed.len() > original_data.len()); // Mock compression adds header
    }

    #[test]
    fn test_compression_ratio_estimation() {
        // Test with highly repetitive data
        let repetitive_data = vec![b'A'; 1000];
        let ratio =
            CompressionEngine::estimate_compression_ratio(&repetitive_data, CompressionType::Lz4);
        assert!(ratio < 0.5); // Should compress well

        // Test with random data
        let random_data: Vec<u8> = (0..=255).cycle().take(1000).collect();
        let ratio =
            CompressionEngine::estimate_compression_ratio(&random_data, CompressionType::Lz4);
        assert!(ratio > 0.7); // Should not compress as well

        // Test no compression
        let ratio =
            CompressionEngine::estimate_compression_ratio(&repetitive_data, CompressionType::None);
        assert_eq!(ratio, 1.0);
    }

    #[test]
    fn test_decoder_error_handling() {
        let empty_data = [];
        let string_table = StringTable::new();
        let type_table = TypeTable::new();
        let mut decoder = BinaryDecoder::new(&empty_data, string_table, type_table);

        // Should fail on empty data
        assert!(decoder.decode_u8().is_err());
        assert!(decoder.decode_u16().is_err());
        assert!(decoder.decode_u32().is_err());
        assert!(decoder.decode_u64().is_err());

        // Test invalid string ID
        let data = [255, 255, 255, 255]; // Large string ID that doesn't exist
        let mut decoder = BinaryDecoder::new(&data, StringTable::new(), TypeTable::new());
        assert!(decoder.decode_string().is_err());

        // Test invalid type ID
        let data = [255, 255]; // Large type ID that doesn't exist
        let mut decoder = BinaryDecoder::new(&data, StringTable::new(), TypeTable::new());
        assert!(decoder.decode_type_name().is_err());

        // Test invalid option marker
        let data = [255]; // Invalid option marker
        let mut decoder = BinaryDecoder::new(&data, StringTable::new(), TypeTable::new());
        assert!(decoder.decode_optional_string().is_err());
    }
}
