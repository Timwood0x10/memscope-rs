//! Binary file format definitions using simple TLV (Type-Length-Value) structure

/// File magic bytes for format identification
pub const MAGIC_BYTES: &[u8; 8] = b"MEMSCOPE";

pub const FORMAT_VERSION: u32 = 2; // Updated for Task 6: Advanced metrics support
pub const HEADER_SIZE: usize = 16;
pub const ALLOCATION_RECORD_TYPE: u8 = 1;

// Task 6: New segment types for advanced metrics
pub const ADVANCED_METRICS_SEGMENT_TYPE: u8 = 2;
pub const ADVANCED_METRICS_MAGIC: &[u8; 4] = b"ADVD"; // Advanced Data segment identifier

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

/// Task 6: Advanced metrics segment header
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct AdvancedMetricsHeader {
    pub magic: [u8; 4],        // "ADVD"
    pub segment_size: u32,     // Size of the entire segment including header
    pub metrics_bitmap: u32,   // Bitmap indicating which metrics are present
    pub reserved: u32,         // Reserved for future use
}

impl AdvancedMetricsHeader {
    pub fn new(segment_size: u32, metrics_bitmap: u32) -> Self {
        Self {
            magic: *ADVANCED_METRICS_MAGIC,
            segment_size,
            metrics_bitmap,
            reserved: 0,
        }
    }

    pub fn is_valid_magic(&self) -> bool {
        self.magic == *ADVANCED_METRICS_MAGIC
    }

    /// Convert header to bytes using Little Endian format
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];

        bytes[0..4].copy_from_slice(&self.magic);
        bytes[4..8].copy_from_slice(&self.segment_size.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.metrics_bitmap.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.reserved.to_le_bytes());

        bytes
    }

    /// Create header from bytes using Little Endian format
    pub fn from_bytes(bytes: &[u8; 16]) -> Self {
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&bytes[0..4]);

        let segment_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let metrics_bitmap = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let reserved = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

        Self {
            magic,
            segment_size,
            metrics_bitmap,
            reserved,
        }
    }
}

/// Task 6: Metrics bitmap flags for identifying which advanced metrics are present
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetricsBitmapFlags {
    LifecycleAnalysis = 1 << 0,      // Task 4 lifecycle metrics
    ContainerAnalysis = 1 << 1,      // Task 3 container analysis
    TypeUsageStats = 1 << 2,         // Task 2 type usage statistics
    SourceAnalysis = 1 << 3,         // Source code analysis
    FragmentationAnalysis = 1 << 4,  // Memory fragmentation analysis
    ThreadContext = 1 << 5,          // Thread context information
    DropChainAnalysis = 1 << 6,      // Drop chain analysis
    ZstAnalysis = 1 << 7,            // Zero-sized type analysis
    HealthScoring = 1 << 8,          // Memory health scoring
    PerformanceBenchmarks = 1 << 9,  // Performance benchmark data
    // Bits 10-31 reserved for future metrics
}

impl MetricsBitmapFlags {
    /// Check if a specific metric is enabled in the bitmap
    pub fn is_enabled(bitmap: u32, flag: MetricsBitmapFlags) -> bool {
        (bitmap & flag as u32) != 0
    }

    /// Enable a specific metric in the bitmap
    pub fn enable(bitmap: u32, flag: MetricsBitmapFlags) -> u32 {
        bitmap | (flag as u32)
    }

    /// Disable a specific metric in the bitmap
    pub fn disable(bitmap: u32, flag: MetricsBitmapFlags) -> u32 {
        bitmap & !(flag as u32)
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

    #[test]
    fn test_advanced_metrics_header_creation() {
        let header = AdvancedMetricsHeader::new(1024, 0x12345678);
        assert_eq!(header.magic, *ADVANCED_METRICS_MAGIC);
        assert_eq!(header.segment_size, 1024);
        assert_eq!(header.metrics_bitmap, 0x12345678);
        assert_eq!(header.reserved, 0);
        assert!(header.is_valid_magic());
    }

    #[test]
    fn test_advanced_metrics_header_serialization() {
        let header = AdvancedMetricsHeader::new(2048, 0xABCDEF00);
        let bytes = header.to_bytes();
        let deserialized = AdvancedMetricsHeader::from_bytes(&bytes);

        assert_eq!(header, deserialized);
    }

    #[test]
    fn test_metrics_bitmap_flags() {
        let mut bitmap = 0u32;

        // Test enabling flags
        bitmap = MetricsBitmapFlags::enable(bitmap, MetricsBitmapFlags::LifecycleAnalysis);
        assert!(MetricsBitmapFlags::is_enabled(bitmap, MetricsBitmapFlags::LifecycleAnalysis));
        assert!(!MetricsBitmapFlags::is_enabled(bitmap, MetricsBitmapFlags::ContainerAnalysis));

        bitmap = MetricsBitmapFlags::enable(bitmap, MetricsBitmapFlags::ContainerAnalysis);
        assert!(MetricsBitmapFlags::is_enabled(bitmap, MetricsBitmapFlags::LifecycleAnalysis));
        assert!(MetricsBitmapFlags::is_enabled(bitmap, MetricsBitmapFlags::ContainerAnalysis));

        // Test disabling flags
        bitmap = MetricsBitmapFlags::disable(bitmap, MetricsBitmapFlags::LifecycleAnalysis);
        assert!(!MetricsBitmapFlags::is_enabled(bitmap, MetricsBitmapFlags::LifecycleAnalysis));
        assert!(MetricsBitmapFlags::is_enabled(bitmap, MetricsBitmapFlags::ContainerAnalysis));
    }
}
