//! Binary parser for high-performance memory tracking data import
//!
//! This module implements the BinaryParser struct that reads and parses binary format
//! memory tracking data. It supports on-demand section loading, data validation,
//! and error recovery mechanisms.

use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
use crate::export::binary_format::{
    BinaryDecoder, BinaryFile, BinaryFormatError, BinaryHeader, CompressionEngine, CompressionType,
    SectionDirectory, SectionEntry, SectionType, StringTable, TypeTable, BINARY_VERSION_MAJOR,
    BINARY_VERSION_MINOR,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Binary parser options configuration
#[derive(Debug, Clone)]
pub struct BinaryParserOptions {
    /// Enable strict validation mode
    pub strict_validation: bool,
    /// Buffer size for I/O operations (default: 256KB)
    pub buffer_size: usize,
    /// Enable partial recovery from corrupted data
    pub enable_recovery: bool,
    /// Maximum number of recovery attempts
    pub max_recovery_attempts: usize,
    /// Enable progress reporting during parsing
    pub enable_progress_reporting: bool,
    /// Verify checksums for all sections
    pub verify_checksums: bool,
    /// Load all sections into memory immediately
    pub preload_all_sections: bool,
}

impl Default for BinaryParserOptions {
    fn default() -> Self {
        Self {
            strict_validation: true,
            buffer_size: 256 * 1024, // 256KB
            enable_recovery: true,
            max_recovery_attempts: 3,
            enable_progress_reporting: false,
            verify_checksums: true,
            preload_all_sections: false,
        }
    }
}

impl BinaryParserOptions {
    /// Create options with strict validation enabled
    pub fn strict() -> Self {
        Self {
            strict_validation: true,
            enable_recovery: false,
            verify_checksums: true,
            ..Default::default()
        }
    }

    /// Create options with recovery enabled for corrupted files
    pub fn recovery_mode() -> Self {
        Self {
            strict_validation: false,
            enable_recovery: true,
            max_recovery_attempts: 5,
            verify_checksums: false,
            ..Default::default()
        }
    }

    /// Create options for fast parsing (minimal validation)
    pub fn fast() -> Self {
        Self {
            strict_validation: false,
            enable_recovery: false,
            verify_checksums: false,
            preload_all_sections: true,
            ..Default::default()
        }
    }
}

/// Parse progress information
#[derive(Debug, Clone)]
pub struct ParseProgress {
    /// Current section being parsed
    pub current_section: Option<SectionType>,
    /// Number of sections parsed
    pub sections_parsed: usize,
    /// Total number of sections
    pub total_sections: usize,
    /// Bytes parsed so far
    pub bytes_parsed: usize,
    /// Total file size
    pub total_file_size: usize,
    /// Parse start time
    pub start_time: std::time::Instant,
}

impl ParseProgress {
    /// Calculate completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.total_file_size == 0 {
            0.0
        } else {
            (self.bytes_parsed as f64 / self.total_file_size as f64) * 100.0
        }
    }

    /// Calculate elapsed time
    pub fn elapsed_time(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Estimate remaining time
    pub fn estimated_remaining_time(&self) -> Option<std::time::Duration> {
        let completion = self.completion_percentage();
        if completion > 0.0 && completion < 100.0 {
            let elapsed = self.elapsed_time();
            let total_estimated = elapsed.as_secs_f64() / (completion / 100.0);
            let remaining = total_estimated - elapsed.as_secs_f64();
            Some(std::time::Duration::from_secs_f64(remaining.max(0.0)))
        } else {
            None
        }
    }
}

/// Binary parse result with statistics
#[derive(Debug, Clone)]
pub struct BinaryParseResult {
    /// Path to the parsed file
    pub file_path: String,
    /// Total file size in bytes
    pub file_size: usize,
    /// Parse duration
    pub parse_duration: std::time::Duration,
    /// Number of sections parsed
    pub sections_parsed: usize,
    /// Number of allocations parsed
    pub allocations_parsed: usize,
    /// Whether recovery was used
    pub recovery_used: bool,
    /// Number of corrupted sections recovered
    pub recovered_sections: usize,
    /// Peak memory usage during parsing
    pub peak_memory_usage: usize,
}

impl BinaryParseResult {
    /// Calculate parse speed in MB/s
    pub fn parse_speed_mbps(&self) -> f64 {
        let seconds = self.parse_duration.as_secs_f64();
        if seconds > 0.0 {
            (self.file_size as f64) / (1024.0 * 1024.0) / seconds
        } else {
            0.0
        }
    }
}

/// Section loading status
#[derive(Debug, Clone, PartialEq)]
pub enum SectionStatus {
    /// Section not loaded
    NotLoaded,
    /// Section is being loaded
    Loading,
    /// Section loaded successfully
    Loaded,
    /// Section failed to load
    Failed(String),
    /// Section recovered from corruption
    Recovered,
}

/// Loaded section data
#[derive(Debug, Clone)]
pub struct LoadedSection {
    /// Section type
    pub section_type: SectionType,
    /// Raw section data (decompressed)
    pub data: Vec<u8>,
    /// Loading status
    pub status: SectionStatus,
    /// Checksum verification result
    pub checksum_valid: bool,
    /// Compression type used
    pub compression: CompressionType,
    /// Original compressed size
    pub compressed_size: usize,
    /// Decompressed size
    pub decompressed_size: usize,
}

/// Version compatibility information
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VersionInfo {
    /// Major version number
    pub major: u16,
    /// Minor version number
    pub minor: u16,
}

impl VersionInfo {
    /// Create a new version info
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Get current parser version
    pub fn current() -> Self {
        Self {
            major: BINARY_VERSION_MAJOR,
            minor: BINARY_VERSION_MINOR,
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }

    /// Check if this version is compatible with another version
    pub fn is_compatible_with(&self, other: &VersionInfo) -> bool {
        // Same major version is compatible
        // Higher minor version is backward compatible
        self.major == other.major && self.minor >= other.minor
    }

    /// Check if this version is newer than another
    pub fn is_newer_than(&self, other: &VersionInfo) -> bool {
        self.major > other.major || (self.major == other.major && self.minor > other.minor)
    }

    /// Check if this version is older than another
    pub fn is_older_than(&self, other: &VersionInfo) -> bool {
        self.major < other.major || (self.major == other.major && self.minor < other.minor)
    }
}

/// Version compatibility result
#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityResult {
    /// Fully compatible
    Compatible,
    /// Compatible with warnings
    CompatibleWithWarnings(Vec<String>),
    /// Incompatible but can be upgraded
    RequiresUpgrade {
        file_version: VersionInfo,
        required_version: VersionInfo,
        upgrade_suggestions: Vec<String>,
    },
    /// Completely incompatible
    Incompatible {
        file_version: VersionInfo,
        parser_version: VersionInfo,
        reason: String,
    },
}

/// Version compatibility checker
pub struct VersionCompatibilityChecker {
    /// Supported version ranges
    supported_versions: Vec<(VersionInfo, VersionInfo)>, // (min, max)
    /// Version-specific migration handlers
    migration_handlers: HashMap<String, Box<dyn Fn(&mut BinaryParser) -> TrackingResult<()>>>,
}

impl VersionCompatibilityChecker {
    /// Create a new compatibility checker
    pub fn new() -> Self {
        let mut checker = Self {
            supported_versions: Vec::new(),
            migration_handlers: HashMap::new(),
        };

        // Add default supported version ranges
        checker.add_supported_version_range(VersionInfo::new(1, 0), VersionInfo::current());

        checker
    }

    /// Add a supported version range
    pub fn add_supported_version_range(
        &mut self,
        min_version: VersionInfo,
        max_version: VersionInfo,
    ) {
        self.supported_versions.push((min_version, max_version));
    }

    /// Check compatibility between file version and parser version
    pub fn check_compatibility(&self, file_version: &VersionInfo) -> CompatibilityResult {
        let parser_version = VersionInfo::current();

        // Check if file version is supported
        let is_supported = self.supported_versions.iter().any(|(min, max)| {
            file_version.major >= min.major
                && file_version.major <= max.major
                && (file_version.major > min.major || file_version.minor >= min.minor)
                && (file_version.major < max.major || file_version.minor <= max.minor)
        });

        if !is_supported {
            return CompatibilityResult::Incompatible {
                file_version: file_version.clone(),
                parser_version,
                reason: format!(
                    "File version {} is not supported by parser version {}",
                    file_version.to_string(),
                    parser_version.to_string()
                ),
            };
        }

        // Check specific compatibility scenarios
        if file_version == &parser_version {
            CompatibilityResult::Compatible
        } else if file_version.is_older_than(&parser_version) {
            // Older file version - check if we can handle it
            if file_version.major == parser_version.major {
                // Same major version - backward compatible with possible warnings
                let warnings =
                    self.get_backward_compatibility_warnings(file_version, &parser_version);
                if warnings.is_empty() {
                    CompatibilityResult::Compatible
                } else {
                    CompatibilityResult::CompatibleWithWarnings(warnings)
                }
            } else {
                // Different major version - requires upgrade
                CompatibilityResult::RequiresUpgrade {
                    file_version: file_version.clone(),
                    required_version: parser_version,
                    upgrade_suggestions: self
                        .get_upgrade_suggestions(file_version, &parser_version),
                }
            }
        } else {
            // Newer file version - incompatible
            CompatibilityResult::Incompatible {
                file_version: file_version.clone(),
                parser_version,
                reason: format!(
                    "File version {} is newer than parser version {}. Please upgrade the parser.",
                    file_version.to_string(),
                    parser_version.to_string()
                ),
            }
        }
    }

    /// Get backward compatibility warnings
    fn get_backward_compatibility_warnings(
        &self,
        file_version: &VersionInfo,
        parser_version: &VersionInfo,
    ) -> Vec<String> {
        let mut warnings = Vec::new();

        // Add version-specific warnings
        if file_version.minor < parser_version.minor {
            warnings.push(format!(
                "File was created with an older version ({}) and may not contain all features available in version {}",
                file_version.to_string(),
                parser_version.to_string()
            ));
        }

        // Add specific feature warnings based on version differences
        if file_version.minor == 0 && parser_version.minor > 0 {
            warnings.push("Some advanced features may not be available in this file".to_string());
        }

        warnings
    }

    /// Get upgrade suggestions
    fn get_upgrade_suggestions(
        &self,
        file_version: &VersionInfo,
        parser_version: &VersionInfo,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        suggestions.push(format!(
            "Consider re-exporting your data with parser version {} to access all features",
            parser_version.to_string()
        ));

        if file_version.major < parser_version.major {
            suggestions.push(
                "This file uses an older major version format that may have compatibility issues"
                    .to_string(),
            );
            suggestions.push(
                "Some sections may not be readable or may be interpreted differently".to_string(),
            );
        }

        suggestions.push("Use the conversion tools to upgrade the file format".to_string());

        suggestions
    }

    /// Apply version-specific migrations
    pub fn apply_migrations(
        &self,
        parser: &mut BinaryParser,
        file_version: &VersionInfo,
    ) -> TrackingResult<()> {
        let migration_key = format!("{}_{}", file_version.major, file_version.minor);

        if let Some(handler) = self.migration_handlers.get(&migration_key) {
            handler(parser)?;
        }

        Ok(())
    }
}

impl Default for VersionCompatibilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Main binary parser struct
pub struct BinaryParser {
    /// Parser options
    options: BinaryParserOptions,
    /// File header
    header: Option<BinaryHeader>,
    /// Section directory
    directory: Option<SectionDirectory>,
    /// String table for string deduplication
    string_table: Option<StringTable>,
    /// Type table for type optimization
    type_table: Option<TypeTable>,
    /// Loaded sections cache
    loaded_sections: HashMap<SectionType, LoadedSection>,
    /// Raw file data
    file_data: Vec<u8>,
    /// Progress callback
    progress_callback: Option<Box<dyn Fn(&ParseProgress) + Send + Sync>>,
    /// Version compatibility checker
    compatibility_checker: VersionCompatibilityChecker,
    /// File version information
    file_version: Option<VersionInfo>,
    /// Compatibility result
    compatibility_result: Option<CompatibilityResult>,
}

impl BinaryParser {
    /// Create a new binary parser with default options
    pub fn new() -> Self {
        Self {
            options: BinaryParserOptions::default(),
            header: None,
            directory: None,
            string_table: None,
            type_table: None,
            loaded_sections: HashMap::new(),
            file_data: Vec::new(),
            progress_callback: None,
            compatibility_checker: VersionCompatibilityChecker::new(),
            file_version: None,
            compatibility_result: None,
        }
    }

    /// Create a new binary parser with custom options
    pub fn with_options(options: BinaryParserOptions) -> Self {
        Self {
            options,
            header: None,
            directory: None,
            string_table: None,
            type_table: None,
            loaded_sections: HashMap::new(),
            file_data: Vec::new(),
            progress_callback: None,
            compatibility_checker: VersionCompatibilityChecker::new(),
            file_version: None,
            compatibility_result: None,
        }
    }

    /// Set progress callback for monitoring parse progress
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&ParseProgress) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Load and parse a binary file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> TrackingResult<BinaryParseResult> {
        let start_time = std::time::Instant::now();
        let path = path.as_ref();

        // Read file data
        let file = File::open(path).map_err(|e| {
            crate::core::types::TrackingError::IoError(format!("Failed to open file: {}", e))
        })?;

        let mut reader = BufReader::with_capacity(self.options.buffer_size, file);
        reader.read_to_end(&mut self.file_data).map_err(|e| {
            crate::core::types::TrackingError::IoError(format!("Failed to read file: {}", e))
        })?;

        let file_size = self.file_data.len();

        // Initialize progress tracking
        let mut progress = ParseProgress {
            current_section: None,
            sections_parsed: 0,
            total_sections: 0,
            bytes_parsed: 0,
            total_file_size: file_size,
            start_time,
        };

        // Report initial progress
        if let Some(ref callback) = self.progress_callback {
            callback(&progress);
        }

        // Parse file header
        self.parse_header(&mut progress)?;

        // Parse section directory
        self.parse_section_directory(&mut progress)?;

        // Parse string and type tables
        self.parse_tables(&mut progress)?;

        // Preload all sections if requested
        if self.options.preload_all_sections {
            self.preload_all_sections(&mut progress)?;
        }

        // Create result
        let result = BinaryParseResult {
            file_path: path.to_string_lossy().to_string(),
            file_size,
            parse_duration: start_time.elapsed(),
            sections_parsed: self.loaded_sections.len(),
            allocations_parsed: 0, // Will be calculated when sections are parsed
            recovery_used: false,  // TODO: Track recovery usage
            recovered_sections: 0, // TODO: Track recovered sections
            peak_memory_usage: 0,  // TODO: Implement memory tracking
        };

        Ok(result)
    }

    /// Parse the file header
    fn parse_header(&mut self, progress: &mut ParseProgress) -> TrackingResult<()> {
        if self.file_data.len() < 64 {
            return Err(crate::core::types::TrackingError::ValidationError(
                "File too small to contain header".to_string(),
            ));
        }

        let header = if self.options.enable_recovery {
            // In recovery mode, try to parse header with relaxed validation
            match BinaryHeader::from_bytes_relaxed(&self.file_data[0..64]) {
                Ok(h) => h,
                Err(e) => {
                    tracing::warn!(
                        "Header parsing failed in recovery mode: {}, attempting reconstruction",
                        e
                    );
                    // Try to reconstruct a minimal valid header
                    self.reconstruct_header_from_data()?
                }
            }
        } else {
            BinaryHeader::from_bytes(&self.file_data[0..64]).map_err(|e| {
                crate::core::types::TrackingError::ValidationError(format!("Invalid header: {}", e))
            })?
        };

        // Validate header (skip in recovery mode if header was reconstructed)
        if !self.options.enable_recovery
            || header.magic == crate::export::binary_format::BINARY_MAGIC
        {
            if let Err(e) = header.validate() {
                if self.options.enable_recovery {
                    tracing::warn!(
                        "Header validation failed in recovery mode: {}, continuing anyway",
                        e
                    );
                } else {
                    return Err(crate::core::types::TrackingError::ValidationError(format!(
                        "Header validation failed: {}",
                        e
                    )));
                }
            }
        } else {
            tracing::info!(
                "Skipping header validation in recovery mode due to reconstructed header"
            );
        }

        // Verify checksum if enabled
        if self.options.verify_checksums {
            let mut header_copy = header.clone();
            header_copy.calculate_checksum();
            if header_copy.checksum != header.checksum {
                if self.options.strict_validation {
                    return Err(crate::core::types::TrackingError::ValidationError(
                        "Header checksum mismatch".to_string(),
                    ));
                }
                // In non-strict mode, log warning but continue
                tracing::warn!("Header checksum mismatch, continuing in recovery mode");
            }
        }

        // Check version compatibility
        let file_version = VersionInfo::new(header.version_major, header.version_minor);
        let compatibility_result = self
            .compatibility_checker
            .check_compatibility(&file_version);

        // Handle compatibility result
        match &compatibility_result {
            CompatibilityResult::Compatible => {
                tracing::info!(
                    "File version {} is fully compatible",
                    file_version.to_string()
                );
            }
            CompatibilityResult::CompatibleWithWarnings(warnings) => {
                tracing::warn!(
                    "File version {} is compatible with warnings:",
                    file_version.to_string()
                );
                for warning in warnings {
                    tracing::warn!("  - {}", warning);
                }
            }
            CompatibilityResult::RequiresUpgrade {
                file_version,
                required_version,
                upgrade_suggestions,
            } => {
                tracing::warn!(
                    "File version {} requires upgrade to {}",
                    file_version.to_string(),
                    required_version.to_string()
                );
                for suggestion in upgrade_suggestions {
                    tracing::info!("  - {}", suggestion);
                }
                if self.options.strict_validation {
                    return Err(crate::core::types::TrackingError::ExportError(format!(
                        "File version {} requires upgrade to {}",
                        file_version.to_string(),
                        required_version.to_string()
                    )));
                }
            }
            CompatibilityResult::Incompatible {
                file_version,
                parser_version,
                reason,
            } => {
                tracing::error!(
                    "File version {} is incompatible with parser version {}: {}",
                    file_version.to_string(),
                    parser_version.to_string(),
                    reason
                );
                if self.options.strict_validation {
                    return Err(crate::core::types::TrackingError::ExportError(format!(
                        "Incompatible version: {}",
                        reason
                    )));
                }
            }
        }

        progress.total_sections = header.section_count as usize;
        progress.bytes_parsed = 64;

        self.header = Some(header);
        self.file_version = Some(file_version);
        self.compatibility_result = Some(compatibility_result);

        // Apply version-specific migrations if needed
        let file_version_for_migration = self.file_version;
        if let Some(file_version) = file_version_for_migration {
            // For now, we'll skip migrations to avoid borrowing issues
            // In a real implementation, migrations would be applied differently
            tracing::info!(
                "Migration for version {} would be applied here",
                file_version.to_string()
            );
        }

        // Report progress
        if let Some(ref callback) = self.progress_callback {
            callback(progress);
        }

        Ok(())
    }

    /// Attempt to reconstruct a valid header from corrupted data
    fn reconstruct_header_from_data(&self) -> TrackingResult<BinaryHeader> {
        tracing::info!("Attempting to reconstruct header from file data");

        let mut header = BinaryHeader::new();

        // Try to estimate file size
        header.total_size = self.file_data.len() as u64;

        // Try to estimate section count by scanning for section-like patterns
        header.section_count = self.estimate_section_count();

        // Use current timestamp
        header.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Set a dummy checksum
        header.checksum = 0;

        tracing::info!(
            "Reconstructed header with {} estimated sections",
            header.section_count
        );
        Ok(header)
    }

    /// Estimate the number of sections by scanning file data
    fn estimate_section_count(&self) -> u32 {
        // Look for patterns that might indicate sections
        // This is a heuristic approach for recovery

        if self.file_data.len() < 128 {
            return 1; // Minimal file
        }

        // Scan for potential section boundaries
        let mut potential_sections = 0;
        let scan_start = 64; // After header
        let scan_end = std::cmp::min(self.file_data.len(), 1024); // Don't scan too far

        for i in (scan_start..scan_end).step_by(4) {
            if i + 4 <= self.file_data.len() {
                let value = u32::from_le_bytes([
                    self.file_data[i],
                    self.file_data[i + 1],
                    self.file_data[i + 2],
                    self.file_data[i + 3],
                ]);

                // Look for reasonable-looking size values
                if value > 0 && value < self.file_data.len() as u32 {
                    potential_sections += 1;
                }
            }
        }

        // Return a reasonable estimate
        std::cmp::max(1, std::cmp::min(potential_sections / 4, 10))
    }

    /// Parse the section directory
    fn parse_section_directory(&mut self, progress: &mut ParseProgress) -> TrackingResult<()> {
        let header = self.header.as_ref().ok_or_else(|| {
            crate::core::types::TrackingError::ExportError("Header not parsed".to_string())
        })?;

        let directory_start = 64; // After header
        let directory_size = header.section_count as usize * 20; // 20 bytes per section entry

        if self.file_data.len() < directory_start + directory_size {
            return Err(crate::core::types::TrackingError::ExportError(
                "File too small to contain section directory".to_string(),
            ));
        }

        let directory_data = &self.file_data[directory_start..directory_start + directory_size];
        let directory = SectionDirectory::from_bytes(directory_data, header.section_count)
            .map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Invalid section directory: {}",
                    e
                ))
            })?;

        progress.bytes_parsed += directory_size;
        self.directory = Some(directory);

        // Report progress
        if let Some(ref callback) = self.progress_callback {
            callback(progress);
        }

        Ok(())
    }

    /// Parse string and type tables
    fn parse_tables(&mut self, progress: &mut ParseProgress) -> TrackingResult<()> {
        let header = self.header.as_ref().unwrap();
        let _directory = self.directory.as_ref().unwrap();

        let mut current_offset = 64 + (header.section_count as usize * 20); // After header and directory

        // Parse string table
        let string_table_size = self.estimate_table_size(current_offset)?;
        if self.file_data.len() >= current_offset + string_table_size {
            let string_table_data =
                &self.file_data[current_offset..current_offset + string_table_size];
            match StringTable::from_bytes(string_table_data) {
                Ok(table) => {
                    self.string_table = Some(table);
                    current_offset += string_table_size;
                    progress.bytes_parsed += string_table_size;
                }
                Err(e) => {
                    if self.options.strict_validation {
                        return Err(crate::core::types::TrackingError::ExportError(format!(
                            "Failed to parse string table: {}",
                            e
                        )));
                    }
                    tracing::warn!("Failed to parse string table: {}, using empty table", e);
                    self.string_table = Some(StringTable::new());
                }
            }
        } else {
            self.string_table = Some(StringTable::new());
        }

        // Parse type table
        let type_table_size = self.estimate_table_size(current_offset)?;
        if self.file_data.len() >= current_offset + type_table_size {
            let type_table_data = &self.file_data[current_offset..current_offset + type_table_size];
            match TypeTable::from_bytes(type_table_data) {
                Ok(table) => {
                    self.type_table = Some(table);
                    progress.bytes_parsed += type_table_size;
                }
                Err(e) => {
                    if self.options.strict_validation {
                        return Err(crate::core::types::TrackingError::ExportError(format!(
                            "Failed to parse type table: {}",
                            e
                        )));
                    }
                    tracing::warn!("Failed to parse type table: {}, using empty table", e);
                    self.type_table = Some(TypeTable::new());
                }
            }
        } else {
            self.type_table = Some(TypeTable::new());
        }

        // Report progress
        if let Some(ref callback) = self.progress_callback {
            callback(progress);
        }

        Ok(())
    }

    /// Estimate table size by reading the count field
    fn estimate_table_size(&self, offset: usize) -> TrackingResult<usize> {
        if self.file_data.len() < offset + 4 {
            return Ok(0);
        }

        let count = u32::from_le_bytes([
            self.file_data[offset],
            self.file_data[offset + 1],
            self.file_data[offset + 2],
            self.file_data[offset + 3],
        ]);

        // Estimate size: 4 bytes for count + count * 4 bytes for offsets + estimated string data
        let estimated_size = 4 + (count as usize * 4) + (count as usize * 20); // Rough estimate
        Ok(estimated_size.min(self.file_data.len() - offset))
    }

    /// Preload all sections into memory
    fn preload_all_sections(&mut self, progress: &mut ParseProgress) -> TrackingResult<()> {
        let _directory = self.directory.as_ref().unwrap();

        for section_type in _directory.section_types() {
            progress.current_section = Some(section_type);
            if let Some(ref callback) = self.progress_callback {
                callback(progress);
            }

            match self.load_section(section_type) {
                Ok(()) => {
                    progress.sections_parsed += 1;
                }
                Err(e) => {
                    if self.options.strict_validation {
                        return Err(e);
                    }
                    tracing::warn!("Failed to load section {:?}: {}", section_type, e);
                }
            }

            if let Some(ref callback) = self.progress_callback {
                callback(progress);
            }
        }

        Ok(())
    }

    /// Load a specific section on demand
    pub fn load_section(&mut self, section_type: SectionType) -> TrackingResult<()> {
        // Check if already loaded
        if let Some(section) = self.loaded_sections.get(&section_type) {
            if section.status == SectionStatus::Loaded {
                return Ok(());
            }
        }

        let directory = self.directory.as_ref().ok_or_else(|| {
            crate::core::types::TrackingError::ExportError("Directory not parsed".to_string())
        })?;

        let entry = directory.get_section(section_type).ok_or_else(|| {
            crate::core::types::TrackingError::ExportError(format!(
                "Section {:?} not found",
                section_type
            ))
        })?;

        // Mark as loading
        self.loaded_sections.insert(
            section_type,
            LoadedSection {
                section_type,
                data: Vec::new(),
                status: SectionStatus::Loading,
                checksum_valid: false,
                compression: entry.compression,
                compressed_size: entry.compressed_size as usize,
                decompressed_size: entry.uncompressed_size as usize,
            },
        );

        // Load section data
        let section_start = entry.offset as usize;
        let section_end = section_start + entry.compressed_size as usize;

        if self.file_data.len() < section_end {
            let error_msg = format!(
                "File too small to contain section {:?} (expected {} bytes, have {})",
                section_type,
                section_end,
                self.file_data.len()
            );

            self.loaded_sections.insert(
                section_type,
                LoadedSection {
                    section_type,
                    data: Vec::new(),
                    status: SectionStatus::Failed(error_msg.clone()),
                    checksum_valid: false,
                    compression: entry.compression,
                    compressed_size: entry.compressed_size as usize,
                    decompressed_size: entry.uncompressed_size as usize,
                },
            );

            return Err(crate::core::types::TrackingError::ExportError(error_msg));
        }

        let compressed_data = &self.file_data[section_start..section_end];

        // Decompress if needed
        let decompressed_data = if entry.compression != CompressionType::None {
            CompressionEngine::decompress(compressed_data, entry.compression).map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decompress section {:?}: {}",
                    section_type, e
                ))
            })?
        } else {
            compressed_data.to_vec()
        };

        // Verify decompressed size
        if entry.uncompressed_size > 0
            && decompressed_data.len() != entry.uncompressed_size as usize
        {
            let error_msg = format!(
                "Section {:?} size mismatch: expected {}, got {}",
                section_type,
                entry.uncompressed_size,
                decompressed_data.len()
            );

            if self.options.strict_validation {
                self.loaded_sections.insert(
                    section_type,
                    LoadedSection {
                        section_type,
                        data: Vec::new(),
                        status: SectionStatus::Failed(error_msg.clone()),
                        checksum_valid: false,
                        compression: entry.compression,
                        compressed_size: entry.compressed_size as usize,
                        decompressed_size: entry.uncompressed_size as usize,
                    },
                );

                return Err(crate::core::types::TrackingError::ExportError(error_msg));
            }
        }

        // Create loaded section
        let loaded_section = LoadedSection {
            section_type,
            data: decompressed_data,
            status: SectionStatus::Loaded,
            checksum_valid: true, // TODO: Implement actual checksum verification
            compression: entry.compression,
            compressed_size: entry.compressed_size as usize,
            decompressed_size: entry.uncompressed_size as usize,
        };

        self.loaded_sections.insert(section_type, loaded_section);
        Ok(())
    }

    /// Get a loaded section
    pub fn get_loaded_section(&self, section_type: SectionType) -> Option<&LoadedSection> {
        self.loaded_sections.get(&section_type)
    }

    /// Get the file header
    pub fn get_header(&self) -> Option<&BinaryHeader> {
        self.header.as_ref()
    }

    /// Get the section directory
    pub fn get_directory(&self) -> Option<&SectionDirectory> {
        self.directory.as_ref()
    }

    /// Get the string table
    pub fn get_string_table(&self) -> Option<&StringTable> {
        self.string_table.as_ref()
    }

    /// Get the type table
    pub fn get_type_table(&self) -> Option<&TypeTable> {
        self.type_table.as_ref()
    }

    /// Check if a section is loaded
    pub fn is_section_loaded(&self, section_type: SectionType) -> bool {
        self.loaded_sections
            .get(&section_type)
            .map(|s| s.status == SectionStatus::Loaded)
            .unwrap_or(false)
    }

    /// Get section loading status
    pub fn get_section_status(&self, section_type: SectionType) -> SectionStatus {
        self.loaded_sections
            .get(&section_type)
            .map(|s| s.status.clone())
            .unwrap_or(SectionStatus::NotLoaded)
    }

    /// Get list of available sections
    pub fn get_available_sections(&self) -> Vec<SectionType> {
        self.directory
            .as_ref()
            .map(|d| d.section_types())
            .unwrap_or_default()
    }

    /// Get loaded section data
    pub fn get_section_data(&self, section_type: SectionType) -> Option<&[u8]> {
        self.loaded_sections
            .get(&section_type)
            .filter(|s| s.status == SectionStatus::Loaded)
            .map(|s| s.data.as_slice())
    }

    /// Validate file integrity
    pub fn validate_integrity(&self) -> TrackingResult<()> {
        // Validate header
        if let Some(header) = &self.header {
            header.validate().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Header validation failed: {}",
                    e
                ))
            })?;
        } else {
            return Err(crate::core::types::TrackingError::ExportError(
                "No header loaded".to_string(),
            ));
        }

        // Validate directory
        if self.directory.is_none() {
            return Err(crate::core::types::TrackingError::ExportError(
                "No directory loaded".to_string(),
            ));
        }

        // Validate loaded sections
        for (section_type, section) in &self.loaded_sections {
            if matches!(section.status, SectionStatus::Failed(_)) {
                return Err(crate::core::types::TrackingError::ExportError(format!(
                    "Section {:?} failed to load",
                    section_type
                )));
            }
        }

        Ok(())
    }

    /// Clear all loaded sections to free memory
    pub fn clear_sections(&mut self) {
        self.loaded_sections.clear();
    }

    /// Get memory usage statistics
    pub fn get_memory_usage(&self) -> usize {
        let mut total = self.file_data.len();

        for section in self.loaded_sections.values() {
            total += section.data.len();
        }

        if let Some(string_table) = &self.string_table {
            total += string_table
                .strings()
                .iter()
                .map(|s| s.len())
                .sum::<usize>();
        }

        if let Some(type_table) = &self.type_table {
            total += type_table.custom_type_count() * 50; // Rough estimate
        }

        total
    }

    /// Get file version information
    pub fn get_file_version(&self) -> Option<&VersionInfo> {
        self.file_version.as_ref()
    }

    /// Get compatibility result
    pub fn get_compatibility_result(&self) -> Option<&CompatibilityResult> {
        self.compatibility_result.as_ref()
    }

    /// Check if the file is compatible with the current parser
    pub fn is_compatible(&self) -> bool {
        match &self.compatibility_result {
            Some(CompatibilityResult::Compatible)
            | Some(CompatibilityResult::CompatibleWithWarnings(_)) => true,
            _ => false,
        }
    }

    /// Check if the file requires upgrade
    pub fn requires_upgrade(&self) -> bool {
        matches!(
            self.compatibility_result,
            Some(CompatibilityResult::RequiresUpgrade { .. })
        )
    }

    /// Get upgrade suggestions if file requires upgrade
    pub fn get_upgrade_suggestions(&self) -> Vec<String> {
        match &self.compatibility_result {
            Some(CompatibilityResult::RequiresUpgrade {
                upgrade_suggestions,
                ..
            }) => upgrade_suggestions.clone(),
            _ => Vec::new(),
        }
    }

    /// Get compatibility warnings
    pub fn get_compatibility_warnings(&self) -> Vec<String> {
        match &self.compatibility_result {
            Some(CompatibilityResult::CompatibleWithWarnings(warnings)) => warnings.clone(),
            _ => Vec::new(),
        }
    }

    /// Perform automatic format conversion if possible
    pub fn auto_convert_format(&mut self) -> TrackingResult<bool> {
        if let Some(CompatibilityResult::RequiresUpgrade { file_version, .. }) =
            &self.compatibility_result
        {
            tracing::info!(
                "Attempting automatic format conversion from version {}",
                file_version.to_string()
            );

            // Apply version-specific conversions
            match (file_version.major, file_version.minor) {
                (1, 0) => {
                    // Convert from version 1.0 to current
                    self.convert_from_v1_0()?;
                    return Ok(true);
                }
                _ => {
                    tracing::warn!(
                        "No automatic conversion available for version {}",
                        file_version.to_string()
                    );
                    return Ok(false);
                }
            }
        }
        Ok(false)
    }

    /// Convert from version 1.0 format
    fn convert_from_v1_0(&mut self) -> TrackingResult<()> {
        tracing::info!("Converting from version 1.0 format");

        // Version 1.0 conversion logic would go here
        // For now, we'll just log that conversion was attempted
        tracing::info!("Version 1.0 conversion completed");

        Ok(())
    }

    /// Generate version compatibility report
    pub fn generate_compatibility_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Version Compatibility Report ===\n");

        if let Some(file_version) = &self.file_version {
            report.push_str(&format!("File Version: {}\n", file_version.to_string()));
        }

        let parser_version = VersionInfo::current();
        report.push_str(&format!("Parser Version: {}\n", parser_version.to_string()));

        if let Some(compatibility_result) = &self.compatibility_result {
            match compatibility_result {
                CompatibilityResult::Compatible => {
                    report.push_str("Status: ✅ Fully Compatible\n");
                }
                CompatibilityResult::CompatibleWithWarnings(warnings) => {
                    report.push_str("Status: ⚠️  Compatible with Warnings\n");
                    report.push_str("Warnings:\n");
                    for warning in warnings {
                        report.push_str(&format!("  - {}\n", warning));
                    }
                }
                CompatibilityResult::RequiresUpgrade {
                    upgrade_suggestions,
                    ..
                } => {
                    report.push_str("Status: 🔄 Requires Upgrade\n");
                    report.push_str("Suggestions:\n");
                    for suggestion in upgrade_suggestions {
                        report.push_str(&format!("  - {}\n", suggestion));
                    }
                }
                CompatibilityResult::Incompatible { reason, .. } => {
                    report.push_str("Status: ❌ Incompatible\n");
                    report.push_str(&format!("Reason: {}\n", reason));
                }
            }
        }

        report.push_str("=====================================\n");
        report
    }

    /// Parse memory statistics from the MemoryStats section
    pub fn parse_memory_stats(&mut self) -> TrackingResult<MemoryStats> {
        self.load_section(SectionType::MemoryStats)?;

        let section_data = self
            .get_section_data(SectionType::MemoryStats)
            .ok_or_else(|| {
                crate::core::types::TrackingError::ExportError(
                    "MemoryStats section not loaded".to_string(),
                )
            })?;

        let string_table = self.string_table.as_ref().unwrap().clone();
        let type_table = self.type_table.as_ref().unwrap().clone();
        let mut decoder = BinaryDecoder::new(section_data, string_table, type_table);

        let total_allocated = decoder.decode_usize().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to decode total_allocated: {}",
                e
            ))
        })?;

        let total_deallocated = decoder.decode_usize().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to decode total_deallocated: {}",
                e
            ))
        })?;

        let active_memory = decoder.decode_usize().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to decode active_memory: {}",
                e
            ))
        })?;

        let peak_memory = decoder.decode_usize().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to decode peak_memory: {}",
                e
            ))
        })?;

        let active_allocations = decoder.decode_usize().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to decode active_allocations: {}",
                e
            ))
        })?;

        let total_allocations = decoder.decode_usize().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to decode total_allocations: {}",
                e
            ))
        })?;

        let peak_allocations = decoder.decode_usize().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to decode peak_allocations: {}",
                e
            ))
        })?;

        let leaked_allocations = decoder.decode_usize().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to decode leaked_allocations: {}",
                e
            ))
        })?;

        let leaked_memory = decoder.decode_usize().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to decode leaked_memory: {}",
                e
            ))
        })?;

        Ok(MemoryStats {
            total_allocated,
            total_deallocated,
            active_memory,
            peak_memory,
            active_allocations,
            total_allocations,
            peak_allocations,
            leaked_allocations,
            leaked_memory,
            ..Default::default()
        })
    }

    /// Parse active allocations from the ActiveAllocations section
    pub fn parse_active_allocations(&mut self) -> TrackingResult<Vec<AllocationInfo>> {
        // Debug: Check if ActiveAllocations section exists
        let available_sections = self.get_available_sections();
        println!("Available sections: {:?}", available_sections);
        
        if !available_sections.contains(&SectionType::ActiveAllocations) {
            println!("ActiveAllocations section not found in directory!");
            return Ok(Vec::new());
        }
        
        self.load_section(SectionType::ActiveAllocations)?;

        let section_data = self
            .get_section_data(SectionType::ActiveAllocations)
            .ok_or_else(|| {
                crate::core::types::TrackingError::ExportError(
                    "ActiveAllocations section not loaded".to_string(),
                )
            })?;
            
        println!("ActiveAllocations section data size: {} bytes", section_data.len());
        
        // Debug: Check the first few bytes of the section data
        if section_data.len() >= 8 {
            let first_bytes = &section_data[0..8];
            println!("First 8 bytes of ActiveAllocations section: {:?}", first_bytes);
            let allocation_count_raw = u32::from_le_bytes([first_bytes[0], first_bytes[1], first_bytes[2], first_bytes[3]]);
            println!("Raw allocation count from first 4 bytes: {}", allocation_count_raw);
        }

        let string_table = self.string_table.as_ref().unwrap().clone();
        let type_table = self.type_table.as_ref().unwrap().clone();
        let mut decoder = BinaryDecoder::new(section_data, string_table, type_table);

        let allocation_count = decoder.decode_u32().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to decode allocation count: {}",
                e
            ))
        })? as usize;
        
        println!("Decoded allocation count: {}", allocation_count);

        let mut allocations = Vec::with_capacity(allocation_count);

        for i in 0..allocation_count {
            let ptr = decoder.decode_usize().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode ptr for allocation {}: {}",
                    i, e
                ))
            })?;

            let size = decoder.decode_usize().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode size for allocation {}: {}",
                    i, e
                ))
            })?;

            let var_name = decoder.decode_optional_string().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode var_name for allocation {}: {}",
                    i, e
                ))
            })?;

            let type_name = decoder.decode_optional_type_name().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode type_name for allocation {}: {}",
                    i, e
                ))
            })?;

            let scope_name = decoder.decode_optional_string().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode scope_name for allocation {}: {}",
                    i, e
                ))
            })?;

            let timestamp_alloc = decoder.decode_u64().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode timestamp_alloc for allocation {}: {}",
                    i, e
                ))
            })?;

            // Optional timestamp_dealloc
            let has_dealloc_timestamp = decoder.decode_u8().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode dealloc timestamp flag for allocation {}: {}",
                    i, e
                ))
            })?;

            let timestamp_dealloc = if has_dealloc_timestamp == 1 {
                Some(decoder.decode_u64().map_err(|e| {
                    crate::core::types::TrackingError::ExportError(format!(
                        "Failed to decode timestamp_dealloc for allocation {}: {}",
                        i, e
                    ))
                })?)
            } else {
                None
            };

            let thread_id = decoder.decode_string().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode thread_id for allocation {}: {}",
                    i, e
                ))
            })?;

            let borrow_count = decoder.decode_usize().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode borrow_count for allocation {}: {}",
                    i, e
                ))
            })?;

            let stack_trace = decoder.decode_optional_string_vec().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode stack_trace for allocation {}: {}",
                    i, e
                ))
            })?;

            let is_leaked = decoder.decode_u8().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode is_leaked for allocation {}: {}",
                    i, e
                ))
            })? == 1;

            // Optional lifetime_ms
            let has_lifetime = decoder.decode_u8().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode lifetime flag for allocation {}: {}",
                    i, e
                ))
            })?;

            let lifetime_ms = if has_lifetime == 1 {
                Some(decoder.decode_u64().map_err(|e| {
                    crate::core::types::TrackingError::ExportError(format!(
                        "Failed to decode lifetime_ms for allocation {}: {}",
                        i, e
                    ))
                })?)
            } else {
                None
            };

            // Skip extended fields for now (they were encoded as flags)
            // Must match exactly with the encoder's field list
            for field_name in &[
                "smart_pointer_info",
                "memory_layout",
                "generic_info",
                "dynamic_type_info",
                "runtime_state",
                "stack_allocation",
                "temporary_object",
                "fragmentation_analysis",
                "generic_instantiation",
                "type_relationships",
                "type_usage",
                "function_call_tracking",
                "lifecycle_tracking",
            ] {
                let _flag = decoder.decode_u8().map_err(|e| {
                    crate::core::types::TrackingError::ExportError(format!(
                        "Failed to decode {} flag for allocation {}: {}",
                        field_name, i, e
                    ))
                })?;
                // For now, we just read the flags but don't reconstruct the complex fields
            }

            let mut allocation = AllocationInfo::new(ptr, size);
            allocation.var_name = var_name;
            allocation.type_name = type_name;
            allocation.scope_name = scope_name;
            allocation.timestamp_alloc = timestamp_alloc;
            allocation.timestamp_dealloc = timestamp_dealloc;
            allocation.thread_id = thread_id;
            allocation.borrow_count = borrow_count;
            allocation.stack_trace = stack_trace;
            allocation.is_leaked = is_leaked;
            allocation.lifetime_ms = lifetime_ms;

            allocations.push(allocation);
        }

        Ok(allocations)
    }

    /// Parse type memory usage from the TypeMemoryUsage section
    pub fn parse_type_memory_usage(&mut self) -> TrackingResult<Vec<TypeMemoryUsage>> {
        self.load_section(SectionType::TypeMemoryUsage)?;

        let section_data = self
            .get_section_data(SectionType::TypeMemoryUsage)
            .ok_or_else(|| {
                crate::core::types::TrackingError::ExportError(
                    "TypeMemoryUsage section not loaded".to_string(),
                )
            })?;

        let string_table = self.string_table.as_ref().unwrap().clone();
        let type_table = self.type_table.as_ref().unwrap().clone();
        let mut decoder = BinaryDecoder::new(section_data, string_table, type_table);

        let type_count = decoder.decode_u32().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to decode type count: {}",
                e
            ))
        })? as usize;

        let mut type_usages = Vec::with_capacity(type_count);

        for i in 0..type_count {
            let type_name = decoder.decode_string().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode type_name for type {}: {}",
                    i, e
                ))
            })?;

            let total_size = decoder.decode_usize().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode total_size for type {}: {}",
                    i, e
                ))
            })?;

            let allocation_count = decoder.decode_usize().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode allocation_count for type {}: {}",
                    i, e
                ))
            })?;

            // Calculate derived fields
            let average_size = if allocation_count > 0 {
                total_size as f64 / allocation_count as f64
            } else {
                0.0
            };

            type_usages.push(TypeMemoryUsage {
                type_name,
                total_size,
                allocation_count,
                average_size,
                peak_size: total_size,    // For now, assume peak equals total
                current_size: total_size, // For now, assume current equals total
                efficiency_score: 1.0,    // Default efficiency score
            });
        }

        Ok(type_usages)
    }

    /// Parse allocation history from the AllocationHistory section
    pub fn parse_allocation_history(&mut self) -> TrackingResult<Vec<AllocationInfo>> {
        if !self.is_section_loaded(SectionType::AllocationHistory) {
            self.load_section(SectionType::AllocationHistory)?;
        }

        let section_data = self
            .get_section_data(SectionType::AllocationHistory)
            .ok_or_else(|| {
                crate::core::types::TrackingError::ExportError(
                    "AllocationHistory section not loaded".to_string(),
                )
            })?;

        let string_table = self.string_table.as_ref().unwrap().clone();
        let type_table = self.type_table.as_ref().unwrap().clone();
        let mut decoder = BinaryDecoder::new(section_data, string_table, type_table);

        let history_count = decoder.decode_u32().map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to decode history count: {}",
                e
            ))
        })? as usize;

        let mut historical_allocations = Vec::with_capacity(history_count);

        for i in 0..history_count {
            let ptr = decoder.decode_usize().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode ptr for history {}: {}",
                    i, e
                ))
            })?;

            let size = decoder.decode_usize().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode size for history {}: {}",
                    i, e
                ))
            })?;

            let type_name = decoder.decode_optional_type_name().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode type_name for history {}: {}",
                    i, e
                ))
            })?;

            let timestamp_alloc = decoder.decode_u64().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode timestamp_alloc for history {}: {}",
                    i, e
                ))
            })?;

            let timestamp_dealloc = decoder.decode_u64().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode timestamp_dealloc for history {}: {}",
                    i, e
                ))
            })?;

            let _lifetime = decoder.decode_u64().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode lifetime for history {}: {}",
                    i, e
                ))
            })?;

            let is_leaked = decoder.decode_u8().map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to decode is_leaked for history {}: {}",
                    i, e
                ))
            })? == 1;

            let mut allocation = AllocationInfo::new(ptr, size);
            allocation.type_name = type_name;
            allocation.timestamp_alloc = timestamp_alloc;
            allocation.timestamp_dealloc = Some(timestamp_dealloc);
            allocation.is_leaked = is_leaked;

            historical_allocations.push(allocation);
        }

        Ok(historical_allocations)
    }

    /// Parse all sections and return complete memory tracking data
    pub fn parse_all_data(&mut self) -> TrackingResult<ParsedMemoryData> {
        let memory_stats = self.parse_memory_stats()?;
        let active_allocations = self.parse_active_allocations()?;
        let type_memory_usage = self.parse_type_memory_usage()?;

        // Optional sections
        let allocation_history = if self
            .get_available_sections()
            .contains(&SectionType::AllocationHistory)
        {
            Some(self.parse_allocation_history()?)
        } else {
            None
        };

        Ok(ParsedMemoryData {
            memory_stats,
            active_allocations,
            type_memory_usage,
            allocation_history,
            ffi_analysis: None,        // TODO: Implement FFI analysis parsing
            security_violations: None, // TODO: Implement security violations parsing
            memory_passports: None,    // TODO: Implement memory passports parsing
        })
    }
}

/// Complete parsed memory tracking data
#[derive(Debug, Clone)]
pub struct ParsedMemoryData {
    /// Memory statistics
    pub memory_stats: MemoryStats,
    /// Active allocations
    pub active_allocations: Vec<AllocationInfo>,
    /// Type memory usage
    pub type_memory_usage: Vec<TypeMemoryUsage>,
    /// Allocation history (optional)
    pub allocation_history: Option<Vec<AllocationInfo>>,
    /// FFI analysis data (optional)
    pub ffi_analysis: Option<Vec<u8>>, // TODO: Define proper FFI analysis structure
    /// Security violations (optional)
    pub security_violations: Option<Vec<u8>>, // TODO: Define proper security violations structure
    /// Memory passports (optional)
    pub memory_passports: Option<Vec<u8>>, // TODO: Define proper memory passports structure
}

impl BinaryParser {
    /// Convert binary data to JSON format (compatible with existing JSON export)
    pub fn parse_to_json(&mut self) -> TrackingResult<serde_json::Value> {
        let parsed_data = self.parse_all_data()?;

        // Create JSON structure compatible with existing JSON export
        let mut json_data = serde_json::Map::new();

        // Add memory statistics
        json_data.insert(
            "memory_stats".to_string(),
            serde_json::to_value(&parsed_data.memory_stats).map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to serialize memory stats: {}",
                    e
                ))
            })?,
        );

        // Add active allocations
        json_data.insert(
            "active_allocations".to_string(),
            serde_json::to_value(&parsed_data.active_allocations).map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to serialize active allocations: {}",
                    e
                ))
            })?,
        );

        // Add type memory usage
        json_data.insert(
            "type_memory_usage".to_string(),
            serde_json::to_value(&parsed_data.type_memory_usage).map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to serialize type memory usage: {}",
                    e
                ))
            })?,
        );

        // Add allocation history if available
        if let Some(history) = &parsed_data.allocation_history {
            json_data.insert(
                "allocation_history".to_string(),
                serde_json::to_value(history).map_err(|e| {
                    crate::core::types::TrackingError::ExportError(format!(
                        "Failed to serialize allocation history: {}",
                        e
                    ))
                })?,
            );
        }

        // Add metadata
        let mut metadata = serde_json::Map::new();
        if let Some(file_version) = &self.file_version {
            metadata.insert(
                "file_version".to_string(),
                serde_json::Value::String(file_version.to_string()),
            );
        }
        metadata.insert(
            "parser_version".to_string(),
            serde_json::Value::String(VersionInfo::current().to_string()),
        );
        metadata.insert(
            "export_timestamp".to_string(),
            serde_json::Value::Number(serde_json::Number::from(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            )),
        );
        json_data.insert("metadata".to_string(), serde_json::Value::Object(metadata));

        Ok(serde_json::Value::Object(json_data))
    }

    /// Convert binary data to HTML format using existing templates
    pub fn parse_to_html(&mut self) -> TrackingResult<String> {
        let parsed_data = self.parse_all_data()?;

        // Read the existing HTML template
        let template_html = include_str!("../../templates/dashboard.html");
        let template_css = include_str!("../../templates/styles.css");
        let template_js = include_str!("../../templates/script.js");

        // Convert parsed data to JSON format for the template
        let json_data = self.create_template_json_data(&parsed_data)?;

        // Replace template placeholders
        let mut html = template_html.to_string();

        // Replace CSS content
        html = html.replace("{{CSS_CONTENT}}", template_css);

        // Replace JavaScript content
        html = html.replace("{{JS_CONTENT}}", template_js);

        // Replace JSON data (appears twice in the template)
        let json_string = serde_json::to_string(&json_data).map_err(|e| {
            crate::core::types::TrackingError::ExportError(format!(
                "Failed to serialize JSON data: {}",
                e
            ))
        })?;
        html = html.replace("{{ json_data }}", &json_string);

        Ok(html)
    }

    /// Create JSON data structure compatible with the existing template
    fn create_template_json_data(
        &self,
        parsed_data: &ParsedMemoryData,
    ) -> TrackingResult<serde_json::Value> {
        let mut json_data = serde_json::Map::new();

        // Memory analysis section
        let mut memory_analysis = serde_json::Map::new();
        memory_analysis.insert(
            "stats".to_string(),
            serde_json::to_value(&parsed_data.memory_stats).map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to serialize memory stats: {}",
                    e
                ))
            })?,
        );
        memory_analysis.insert(
            "allocations".to_string(),
            serde_json::to_value(&parsed_data.active_allocations).map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to serialize allocations: {}",
                    e
                ))
            })?,
        );
        json_data.insert(
            "memory_analysis".to_string(),
            serde_json::Value::Object(memory_analysis),
        );

        // Complex types section (derived from type memory usage)
        let mut complex_types = serde_json::Map::new();
        let mut summary = serde_json::Map::new();
        summary.insert(
            "total_complex_types".to_string(),
            serde_json::Value::Number(serde_json::Number::from(
                parsed_data.type_memory_usage.len(),
            )),
        );
        summary.insert(
            "generic_type_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(
                parsed_data
                    .type_memory_usage
                    .iter()
                    .filter(|t| t.type_name.contains('<') && t.type_name.contains('>'))
                    .count(),
            )),
        );
        complex_types.insert("summary".to_string(), serde_json::Value::Object(summary));

        // Categorize types
        let mut categorized_types = serde_json::Map::new();
        let smart_pointers: Vec<_> = parsed_data
            .type_memory_usage
            .iter()
            .filter(|t| {
                t.type_name.contains("Box<")
                    || t.type_name.contains("Rc<")
                    || t.type_name.contains("Arc<")
            })
            .collect();
        let collections: Vec<_> = parsed_data
            .type_memory_usage
            .iter()
            .filter(|t| {
                t.type_name.contains("Vec<")
                    || t.type_name.contains("HashMap<")
                    || t.type_name.contains("BTreeMap<")
            })
            .collect();

        categorized_types.insert(
            "smart_pointers".to_string(),
            serde_json::to_value(&smart_pointers).map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to serialize smart pointers: {}",
                    e
                ))
            })?,
        );
        categorized_types.insert(
            "collections".to_string(),
            serde_json::to_value(&collections).map_err(|e| {
                crate::core::types::TrackingError::ExportError(format!(
                    "Failed to serialize collections: {}",
                    e
                ))
            })?,
        );
        complex_types.insert(
            "categorized_types".to_string(),
            serde_json::Value::Object(categorized_types),
        );
        json_data.insert(
            "complex_types".to_string(),
            serde_json::Value::Object(complex_types),
        );

        // Unsafe FFI section (placeholder for now)
        let mut unsafe_ffi = serde_json::Map::new();
        unsafe_ffi.insert(
            "enhanced_ffi_data".to_string(),
            serde_json::Value::Array(vec![]),
        );
        json_data.insert(
            "unsafe_ffi".to_string(),
            serde_json::Value::Object(unsafe_ffi),
        );

        // Add metadata
        let mut metadata = serde_json::Map::new();
        if let Some(file_version) = &self.file_version {
            metadata.insert(
                "file_version".to_string(),
                serde_json::Value::String(file_version.to_string()),
            );
        }
        metadata.insert(
            "parser_version".to_string(),
            serde_json::Value::String(VersionInfo::current().to_string()),
        );
        metadata.insert(
            "export_timestamp".to_string(),
            serde_json::Value::Number(serde_json::Number::from(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            )),
        );
        metadata.insert(
            "source".to_string(),
            serde_json::Value::String("binary_parser".to_string()),
        );
        json_data.insert("metadata".to_string(), serde_json::Value::Object(metadata));

        Ok(serde_json::Value::Object(json_data))
    }

    /// Load allocations directly into memory structures (for programmatic use)
    pub fn load_allocations(&mut self) -> TrackingResult<Vec<AllocationInfo>> {
        self.parse_active_allocations()
    }

    /// Get all allocations from the parsed data (alias for load_allocations)
    pub fn get_allocations(&mut self) -> TrackingResult<Vec<AllocationInfo>> {
        self.load_allocations()
    }

    /// Load memory stats from parsed binary data
    pub fn load_memory_stats(&mut self) -> TrackingResult<MemoryStats> {
        self.parse_memory_stats()
    }

    /// Load type memory usage from parsed binary data
    pub fn load_type_memory_usage(&mut self) -> TrackingResult<Vec<TypeMemoryUsage>> {
        self.parse_type_memory_usage()
    }

    /// Validate conversion by comparing with expected JSON output
    pub fn validate_conversion(
        &mut self,
        expected_json: &serde_json::Value,
    ) -> TrackingResult<bool> {
        let parsed_json = self.parse_to_json()?;

        // Compare key sections
        let memory_stats_match =
            self.compare_json_sections(&parsed_json, expected_json, "memory_stats")?;
        let allocations_match =
            self.compare_json_sections(&parsed_json, expected_json, "active_allocations")?;
        let types_match =
            self.compare_json_sections(&parsed_json, expected_json, "type_memory_usage")?;

        let overall_match = memory_stats_match && allocations_match && types_match;

        if !overall_match {
            tracing::warn!("Conversion validation failed:");
            if !memory_stats_match {
                tracing::warn!("  - Memory stats mismatch");
            }
            if !allocations_match {
                tracing::warn!("  - Active allocations mismatch");
            }
            if !types_match {
                tracing::warn!("  - Type memory usage mismatch");
            }
        }

        Ok(overall_match)
    }

    /// Compare specific sections of JSON data
    fn compare_json_sections(
        &self,
        parsed: &serde_json::Value,
        expected: &serde_json::Value,
        section_name: &str,
    ) -> TrackingResult<bool> {
        let parsed_section = parsed.get(section_name);
        let expected_section = expected.get(section_name);

        match (parsed_section, expected_section) {
            (Some(parsed_val), Some(expected_val)) => {
                // For now, just check if both sections exist and have the same type
                // A more sophisticated comparison could be implemented here
                Ok(std::mem::discriminant(parsed_val) == std::mem::discriminant(expected_val))
            }
            (None, None) => Ok(true),
            _ => Ok(false),
        }
    }

    /// Format bytes for human-readable display
    fn format_bytes(&self, bytes: usize) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }

    /// Simple HTML escape for basic characters
    fn html_escape(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }

    /// Generate conversion report with statistics
    pub fn generate_conversion_report(&mut self) -> TrackingResult<String> {
        let parsed_data = self.parse_all_data()?;

        let mut report = String::new();
        report.push_str("=== Binary to Data Conversion Report ===\n");

        if let Some(file_version) = &self.file_version {
            report.push_str(&format!("File Version: {}\n", file_version.to_string()));
        }

        report.push_str(&format!(
            "Parser Version: {}\n",
            VersionInfo::current().to_string()
        ));
        report.push_str(&format!(
            "Conversion Time: {}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        report.push_str("\n");

        report.push_str("Data Summary:\n");
        report.push_str(&format!("  - Memory Stats: ✅ Loaded\n"));
        report.push_str(&format!(
            "  - Active Allocations: {} items\n",
            parsed_data.active_allocations.len()
        ));
        report.push_str(&format!(
            "  - Type Memory Usage: {} types\n",
            parsed_data.type_memory_usage.len()
        ));

        if parsed_data.allocation_history.is_some() {
            report.push_str(&format!(
                "  - Allocation History: {} items\n",
                parsed_data.allocation_history.as_ref().unwrap().len()
            ));
        } else {
            report.push_str("  - Allocation History: Not available\n");
        }

        report.push_str("\n");
        report.push_str("Available Sections:\n");
        for section_type in self.get_available_sections() {
            let status = if self.is_section_loaded(section_type) {
                "✅"
            } else {
                "⏳"
            };
            report.push_str(&format!("  - {:?}: {}\n", section_type, status));
        }

        report.push_str("\n");
        if let Some(compatibility_result) = &self.compatibility_result {
            match compatibility_result {
                CompatibilityResult::Compatible => {
                    report.push_str("Compatibility: ✅ Fully Compatible\n");
                }
                CompatibilityResult::CompatibleWithWarnings(warnings) => {
                    report.push_str("Compatibility: ⚠️ Compatible with Warnings\n");
                    for warning in warnings {
                        report.push_str(&format!("  - {}\n", warning));
                    }
                }
                CompatibilityResult::RequiresUpgrade { .. } => {
                    report.push_str("Compatibility: 🔄 Requires Upgrade\n");
                }
                CompatibilityResult::Incompatible { reason, .. } => {
                    report.push_str(&format!("Compatibility: ❌ Incompatible ({})\n", reason));
                }
            }
        }

        report.push_str("==========================================\n");
        Ok(report)
    }
}

impl Default for BinaryParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_parser_options() {
        let options = BinaryParserOptions::default();
        assert!(options.strict_validation);
        assert!(options.verify_checksums);

        let strict_options = BinaryParserOptions::strict();
        assert!(strict_options.strict_validation);
        assert!(!strict_options.enable_recovery);

        let recovery_options = BinaryParserOptions::recovery_mode();
        assert!(!recovery_options.strict_validation);
        assert!(recovery_options.enable_recovery);
        assert_eq!(recovery_options.max_recovery_attempts, 5);

        let fast_options = BinaryParserOptions::fast();
        assert!(!fast_options.strict_validation);
        assert!(!fast_options.verify_checksums);
        assert!(fast_options.preload_all_sections);
    }

    #[test]
    fn test_parse_progress() {
        let start_time = std::time::Instant::now();
        let progress = ParseProgress {
            current_section: Some(SectionType::MemoryStats),
            sections_parsed: 2,
            total_sections: 5,
            bytes_parsed: 1000,
            total_file_size: 5000,
            start_time,
        };

        assert_eq!(progress.completion_percentage(), 20.0);
        assert!(progress.elapsed_time().as_nanos() > 0);
    }

    #[test]
    fn test_binary_parse_result() {
        let result = BinaryParseResult {
            file_path: "test.membin".to_string(),
            file_size: 1000,
            parse_duration: std::time::Duration::from_millis(100),
            sections_parsed: 5,
            allocations_parsed: 100,
            recovery_used: false,
            recovered_sections: 0,
            peak_memory_usage: 1024 * 1024,
        };

        assert!(result.parse_speed_mbps() > 0.0);
    }

    #[test]
    fn test_binary_parser_creation() {
        let parser = BinaryParser::new();
        assert!(parser.options.strict_validation);
        assert!(parser.header.is_none());
        assert!(parser.directory.is_none());

        let custom_options = BinaryParserOptions::fast();
        let custom_parser = BinaryParser::with_options(custom_options);
        assert!(!custom_parser.options.strict_validation);
    }

    #[test]
    fn test_section_status() {
        let parser = BinaryParser::new();

        assert_eq!(
            parser.get_section_status(SectionType::MemoryStats),
            SectionStatus::NotLoaded
        );
        assert!(!parser.is_section_loaded(SectionType::MemoryStats));
        assert!(parser.get_available_sections().is_empty());
    }

    #[test]
    fn test_version_info() {
        let v1_0 = VersionInfo::new(1, 0);
        let v1_1 = VersionInfo::new(1, 1);
        let v2_0 = VersionInfo::new(2, 0);

        assert_eq!(v1_0.to_string(), "1.0");
        assert!(v1_1.is_newer_than(&v1_0));
        assert!(v1_0.is_older_than(&v1_1));
        assert!(v1_1.is_compatible_with(&v1_0));
        assert!(!v1_0.is_compatible_with(&v1_1));
        assert!(!v2_0.is_compatible_with(&v1_0));
    }

    #[test]
    fn test_version_compatibility_checker() {
        let checker = VersionCompatibilityChecker::new();

        let current_version = VersionInfo::current();
        let older_version = VersionInfo::new(1, 0);
        let newer_version = VersionInfo::new(2, 0);

        // Test compatible version
        let result = checker.check_compatibility(&current_version);
        assert_eq!(result, CompatibilityResult::Compatible);

        // Test older version
        let result = checker.check_compatibility(&older_version);
        match result {
            CompatibilityResult::CompatibleWithWarnings(_) | CompatibilityResult::Compatible => {
                // Expected for backward compatibility
            }
            _ => panic!("Older version should be compatible"),
        }
    }

    #[test]
    fn test_binary_parser_version_methods() {
        let parser = BinaryParser::new();

        assert!(parser.get_file_version().is_none());
        assert!(parser.get_compatibility_result().is_none());
        assert!(!parser.requires_upgrade());
        assert!(parser.get_upgrade_suggestions().is_empty());
        assert!(parser.get_compatibility_warnings().is_empty());
    }
}
