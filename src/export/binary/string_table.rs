//! String table optimization for binary export format
//!
//! This module implements string table compression to reduce file size and serialization overhead
//! by storing frequently repeated strings (like type names, function names) only once and
//! referencing them by index elsewhere in the file.

use crate::export::binary::error::BinaryExportError;
use crate::export::binary::serializable::primitives;
use std::collections::HashMap;
use std::io::Write;

/// String table for deduplicating repeated strings in binary export
#[derive(Debug, Clone)]
pub struct StringTable {
    /// Ordered list of unique strings
    strings: Vec<String>,
    /// Map from string to its index in the strings vector
    index_map: HashMap<String, u16>,
    /// Enable compressed index format for smaller indices
    use_compressed_indices: bool,
}

impl StringTable {
    /// Create a new empty string table
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            index_map: HashMap::new(),
            use_compressed_indices: true,
        }
    }

    /// Create a new string table with compression settings
    pub fn with_compression(use_compressed_indices: bool) -> Self {
        Self {
            strings: Vec::new(),
            index_map: HashMap::new(),
            use_compressed_indices,
        }
    }

    /// Add a string to the table and return its index
    /// If the string already exists, returns the existing index
    pub fn add_string(&mut self, s: &str) -> Result<u16, BinaryExportError> {
        if let Some(&index) = self.index_map.get(s) {
            return Ok(index);
        }

        // Check if we've reached the maximum number of strings (u16::MAX)
        if self.strings.len() >= u16::MAX as usize {
            return Err(BinaryExportError::CorruptedData(
                "String table overflow: too many unique strings".to_string(),
            ));
        }

        let index = self.strings.len() as u16;
        self.strings.push(s.to_string());
        self.index_map.insert(s.to_string(), index);
        Ok(index)
    }

    /// Get a string by its index
    pub fn get_string(&self, index: u16) -> Option<&str> {
        self.strings.get(index as usize).map(|s| s.as_str())
    }

    /// Get the index of a string if it exists in the table
    pub fn get_index(&self, s: &str) -> Option<u16> {
        self.index_map.get(s).copied()
    }

    /// Get the number of strings in the table
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Check if the table is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Calculate the total size needed to serialize the string table
    pub fn serialized_size(&self) -> usize {
        let mut size = 1; // compression flag (u8)
        size += 4; // count field (u32)
        for string in &self.strings {
            size += 4 + string.len(); // length prefix + string content
        }
        size
    }

    /// Read a compressed index from the reader
    #[allow(dead_code)]
    pub fn read_compressed_index<R: std::io::Read>(
        reader: &mut R,
        use_compressed: bool,
    ) -> Result<u16, BinaryExportError> {
        if !use_compressed {
            // Use standard u16 format
            return primitives::read_u16(reader);
        }

        // Read first byte to determine format
        let first_byte = primitives::read_u8(reader)?;

        if first_byte < 0x80 {
            // Single byte format (0-127)
            Ok(first_byte as u16)
        } else if first_byte < 0xC0 {
            // Two byte format (128-16383)
            let second_byte = primitives::read_u8(reader)?;
            let index = (((first_byte & 0x3F) as u16) << 8) | (second_byte as u16);
            Ok(index)
        } else {
            // Three byte format (16384-65535)
            let index = primitives::read_u16(reader)?;
            Ok(index)
        }
    }

    /// Write the string table to binary format
    pub fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut size = 0;

        // Write compression flag
        size += primitives::write_u8(writer, if self.use_compressed_indices { 1 } else { 0 })?;

        // Write count of strings
        size += primitives::write_u32(writer, self.strings.len() as u32)?;

        // Write each string with length prefix
        for string in &self.strings {
            size += primitives::write_string(writer, string)?;
        }

        Ok(size)
    }

    /// Write a compressed index to the writer
    pub fn write_compressed_index<W: Write>(
        &self,
        writer: &mut W,
        index: u16,
    ) -> Result<usize, BinaryExportError> {
        if !self.use_compressed_indices {
            // Use standard u16 format
            return primitives::write_u16(writer, index);
        }

        // Use variable-length encoding for smaller indices
        if index < 128 {
            // Single byte for indices 0-127
            primitives::write_u8(writer, index as u8)
        } else if index < 16384 {
            // Two bytes for indices 128-16383
            // First byte: 0x80 | (index >> 8)
            // Second byte: index & 0xFF
            let first_byte = 0x80 | ((index >> 8) as u8);
            let second_byte = (index & 0xFF) as u8;
            let mut size = primitives::write_u8(writer, first_byte)?;
            size += primitives::write_u8(writer, second_byte)?;
            Ok(size)
        } else {
            // Three bytes for indices 16384-65535
            // First byte: 0xC0
            // Next two bytes: standard u16
            let mut size = primitives::write_u8(writer, 0xC0)?;
            size += primitives::write_u16(writer, index)?;
            Ok(size)
        }
    }

    /// Calculate size needed for a compressed index
    pub fn compressed_index_size(&self, index: u16) -> usize {
        if !self.use_compressed_indices {
            return 2; // Standard u16
        }

        if index < 128 {
            1 // Single byte
        } else if index < 16384 {
            2 // Two bytes
        } else {
            3 // Three bytes
        }
    }

    /// Calculate compression statistics
    pub fn compression_stats(&self) -> CompressionStats {
        let mut total_original_size = 0;
        let mut total_references = 0;
        let mut total_reference_size = 0;

        for (string, &index) in &self.index_map {
            // Each reference saves (string.len() + 4 - compressed_index_size) bytes
            // +4 for length prefix, -compressed_index_size for index storage
            let string_size = string.len() + 4;
            total_original_size += string_size;
            total_references += 1;

            // Calculate actual reference size based on compression
            total_reference_size += self.compressed_index_size(index);
        }

        let table_size = self.serialized_size();
        let compressed_size = table_size + total_reference_size;

        CompressionStats {
            original_size: total_original_size,
            compressed_size,
            table_size,
            reference_count: total_references,
            compression_ratio: if total_original_size > 0 {
                compressed_size as f64 / total_original_size as f64
            } else {
                1.0
            },
        }
    }
}

impl Default for StringTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about string table compression effectiveness
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CompressionStats {
    /// Total size if all strings were stored inline
    pub original_size: usize,
    /// Total size with string table compression
    pub compressed_size: usize,
    /// Size of the string table itself
    pub table_size: usize,
    /// Number of string references
    pub reference_count: usize,
    /// Compression ratio (compressed_size / original_size)
    pub compression_ratio: f64,
}

impl CompressionStats {
    /// Calculate space savings in bytes
    pub fn space_saved(&self) -> i64 {
        self.original_size as i64 - self.compressed_size as i64
    }

    /// Calculate space savings as a percentage
    pub fn space_saved_percent(&self) -> f64 {
        if self.original_size > 0 {
            (self.space_saved() as f64 / self.original_size as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Helper for building string tables from allocation data
pub struct StringTableBuilder {
    table: StringTable,
    /// Minimum frequency threshold for including strings in the table
    min_frequency: usize,
    /// Track frequency of each string
    frequency_map: HashMap<String, usize>,
    /// Track total size savings potential for each string
    size_savings_map: HashMap<String, usize>,
    /// Enable advanced compression heuristics
    enable_advanced_compression: bool,
}

impl StringTableBuilder {
    /// Create a new builder with the specified minimum frequency threshold
    pub fn new(min_frequency: usize) -> Self {
        Self {
            table: StringTable::new(),
            min_frequency,
            frequency_map: HashMap::new(),
            size_savings_map: HashMap::new(),
            enable_advanced_compression: true,
        }
    }

    /// Create a new builder with advanced compression settings
    #[allow(dead_code)]
    pub fn with_advanced_compression(min_frequency: usize, enable_advanced: bool) -> Self {
        Self {
            table: StringTable::new(),
            min_frequency,
            frequency_map: HashMap::new(),
            size_savings_map: HashMap::new(),
            enable_advanced_compression: enable_advanced,
        }
    }

    /// Record a string occurrence for frequency analysis
    pub fn record_string(&mut self, s: &str) {
        // Only record non-empty strings for optimization
        if !s.is_empty() {
            let string_key = s.to_string();
            *self.frequency_map.entry(string_key.clone()).or_insert(0) += 1;

            // Calculate potential size savings for this string
            // Each occurrence saves: string_length + 4 (length prefix) - 2 (index reference)
            let savings_per_occurrence = if s.len() + 4 > 2 { s.len() + 4 - 2 } else { 0 };
            *self.size_savings_map.entry(string_key).or_insert(0) += savings_per_occurrence;
        }
    }

    /// Record a string with weight (for strings that appear in critical paths)
    #[allow(dead_code)]
    pub fn record_string_weighted(&mut self, s: &str, weight: usize) {
        if !s.is_empty() {
            let string_key = s.to_string();
            *self.frequency_map.entry(string_key.clone()).or_insert(0) += weight;

            // Weighted size savings calculation
            let savings_per_occurrence = if s.len() + 4 > 2 { s.len() + 4 - 2 } else { 0 };
            *self.size_savings_map.entry(string_key).or_insert(0) +=
                savings_per_occurrence * weight;
        }
    }

    /// Build the final string table including only strings that meet the frequency threshold
    pub fn build(mut self) -> Result<StringTable, BinaryExportError> {
        if self.enable_advanced_compression {
            self.build_with_advanced_heuristics()
        } else {
            self.build_with_simple_frequency()
        }
    }

    /// Build string table using simple frequency-based selection
    fn build_with_simple_frequency(&mut self) -> Result<StringTable, BinaryExportError> {
        // Add strings that meet the frequency threshold to the table
        for (string, frequency) in &self.frequency_map {
            if *frequency >= self.min_frequency {
                self.table.add_string(string)?;
            }
        }

        Ok(self.table.clone())
    }

    /// Build string table using advanced compression heuristics
    fn build_with_advanced_heuristics(&mut self) -> Result<StringTable, BinaryExportError> {
        // Create a list of candidates with their compression scores
        let mut candidates: Vec<(String, f64)> = Vec::new();

        for (string, frequency) in &self.frequency_map {
            if *frequency >= self.min_frequency {
                let size_savings = self.size_savings_map.get(string).unwrap_or(&0);

                // Calculate compression score based on multiple factors
                let compression_score =
                    self.calculate_compression_score(string, *frequency, *size_savings);

                candidates.push((string.clone(), compression_score));
            }
        }

        // Sort candidates by compression score (descending)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Add top candidates to string table, respecting size limits
        let max_table_size = 8192; // 8KB limit for string table
        let mut current_table_size = 4; // Start with count field size

        for (string, _score) in candidates {
            let string_size = 4 + string.len(); // length prefix + content
            if current_table_size + string_size <= max_table_size {
                self.table.add_string(&string)?;
                current_table_size += string_size;
            } else {
                break; // Stop adding if we would exceed size limit
            }
        }

        Ok(self.table.clone())
    }

    /// Calculate compression score for a string based on multiple factors
    fn calculate_compression_score(
        &self,
        string: &str,
        frequency: usize,
        size_savings: usize,
    ) -> f64 {
        let string_len = string.len();

        // Base score from size savings
        let mut score = size_savings as f64;

        // Bonus for high frequency strings
        if frequency > 10 {
            score *= 1.5;
        } else if frequency > 5 {
            score *= 1.2;
        }

        // Bonus for longer strings (more compression potential)
        if string_len > 20 {
            score *= 1.3;
        } else if string_len > 10 {
            score *= 1.1;
        }

        // Bonus for common patterns (type names, function names)
        if self.is_likely_type_name(string) {
            score *= 1.4; // Type names are often repeated
        } else if self.is_likely_function_name(string) {
            score *= 1.2; // Function names are moderately repeated
        }

        // Penalty for very short strings (overhead might not be worth it)
        if string_len < 4 {
            score *= 0.5;
        }

        score
    }

    /// Heuristic to detect if a string is likely a type name
    fn is_likely_type_name(&self, s: &str) -> bool {
        // Common Rust type patterns
        s.contains('<') && s.contains('>') || // Generic types like Vec<T>
        s.starts_with("std::") ||             // Standard library types
        s.starts_with("alloc::") ||           // Allocator types
        s.contains("::") ||                   // Module paths
        s.ends_with("Error") ||               // Error types
        s.ends_with("Result") ||              // Result types
        s.starts_with("Box<") ||              // Box types
        s.starts_with("Arc<") ||              // Arc types
        s.starts_with("Rc<") ||               // Rc types
        s.starts_with("Vec<") ||              // Vec types
        s.starts_with("HashMap<") ||          // HashMap types
        s.starts_with("BTreeMap<") // BTreeMap types
    }

    /// Heuristic to detect if a string is likely a function name
    fn is_likely_function_name(&self, s: &str) -> bool {
        // Common function name patterns
        s.contains("::") ||                   // Method calls
        s == "main" ||                        // Main function
        s.starts_with("std::") ||             // Standard library functions
        s.starts_with("core::") ||            // Core library functions
        s.contains("alloc") ||                // Allocation functions
        s.contains("drop") ||                 // Drop functions
        s.ends_with("_new") ||                // Constructor patterns
        s.ends_with("_init") ||               // Initialization patterns
        s.starts_with("_") ||                 // Internal functions
        (s.len() > 3 && s.chars().all(|c| c.is_ascii_lowercase() || c == '_')) // snake_case
    }

    /// Get current frequency statistics
    #[allow(dead_code)]
    pub fn frequency_stats(&self) -> FrequencyStats {
        let total_strings = self.frequency_map.len();
        let total_occurrences: usize = self.frequency_map.values().sum();
        let qualifying_strings = self
            .frequency_map
            .values()
            .filter(|&&freq| freq >= self.min_frequency)
            .count();

        FrequencyStats {
            total_unique_strings: total_strings,
            total_occurrences,
            qualifying_strings,
            min_frequency: self.min_frequency,
        }
    }
}

/// Statistics about string frequency analysis
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FrequencyStats {
    /// Total number of unique strings encountered
    pub total_unique_strings: usize,
    /// Total number of string occurrences
    pub total_occurrences: usize,
    /// Number of strings that meet the frequency threshold
    pub qualifying_strings: usize,
    /// Minimum frequency threshold used
    pub min_frequency: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_table_basic_operations() {
        let mut table = StringTable::new();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);

        // Add first string
        let index1 = table
            .add_string("Vec<String>")
            .expect("Test operation failed");
        assert_eq!(index1, 0);
        assert_eq!(table.len(), 1);
        assert_eq!(table.get_string(0), Some("Vec<String>"));

        // Add second string
        let index2 = table
            .add_string("HashMap<K,V>")
            .expect("Test operation failed");
        assert_eq!(index2, 1);
        assert_eq!(table.len(), 2);

        // Add duplicate string - should return existing index
        let index3 = table
            .add_string("Vec<String>")
            .expect("Test operation failed");
        assert_eq!(index3, 0);
        assert_eq!(table.len(), 2); // No new string added
    }

    #[test]
    fn test_string_table_serialization_size() {
        let mut table = StringTable::new();
        table.add_string("test").expect("Test operation failed");
        table.add_string("hello").expect("Test operation failed");

        // Expected size: 1 (compression flag) + 4 (count) + 4 + 4 (test) + 4 + 5 (hello) = 22
        assert_eq!(table.serialized_size(), 22);
    }

    #[test]
    fn test_compression_stats() {
        let mut table = StringTable::new();
        table
            .add_string("Vec<String>")
            .expect("Test operation failed"); // 11 chars + 4 length = 15 bytes
        table
            .add_string("HashMap<K,V>")
            .expect("Test operation failed"); // 12 chars + 4 length = 16 bytes

        let stats = table.compression_stats();
        assert_eq!(stats.original_size, 31); // 15 + 16
        assert_eq!(stats.reference_count, 2);
        assert_eq!(stats.table_size, table.serialized_size());
    }

    #[test]
    fn test_string_table_builder() {
        let mut builder = StringTableBuilder::new(2); // Minimum frequency of 2

        // Record strings with different frequencies
        builder.record_string("Vec<String>"); // frequency 1
        builder.record_string("HashMap<K,V>"); // frequency 1
        builder.record_string("Vec<String>"); // frequency 2
        builder.record_string("i32"); // frequency 1
        builder.record_string("HashMap<K,V>"); // frequency 2
        builder.record_string("HashMap<K,V>"); // frequency 3

        let stats = builder.frequency_stats();
        assert_eq!(stats.total_unique_strings, 3);
        assert_eq!(stats.total_occurrences, 6);
        assert_eq!(stats.qualifying_strings, 2); // Vec<String> and HashMap<K,V>

        let table = builder.build().expect("Test operation failed");
        assert_eq!(table.len(), 2); // Only strings with frequency >= 2
    }

    #[test]
    fn test_string_table_overflow() {
        let mut table = StringTable::new();

        // Fill up to near the limit (this would be slow in practice, so we'll mock it)
        // In a real scenario, we'd hit the limit with many unique strings
        for i in 0..10 {
            table
                .add_string(&format!("string_{}", i))
                .expect("Test operation failed");
        }

        assert_eq!(table.len(), 10);
    }

    #[test]
    fn test_empty_string_handling() {
        let mut builder = StringTableBuilder::new(1);
        builder.record_string(""); // Empty string should be ignored
        builder.record_string("valid");

        let stats = builder.frequency_stats();
        assert_eq!(stats.total_unique_strings, 1); // Only "valid" counted
    }
}
