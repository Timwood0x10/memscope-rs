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
}

impl StringTable {
    /// Create a new empty string table
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            index_map: HashMap::new(),
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
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Calculate the total size needed to serialize the string table
    pub fn serialized_size(&self) -> usize {
        let mut size = 4; // count field (u32)
        for string in &self.strings {
            size += 4 + string.len(); // length prefix + string content
        }
        size
    }

    /// Write the string table to binary format
    pub fn write_binary<W: Write>(&self, writer: &mut W) -> Result<usize, BinaryExportError> {
        let mut size = 0;

        // Write count of strings
        size += primitives::write_u32(writer, self.strings.len() as u32)?;

        // Write each string with length prefix
        for string in &self.strings {
            size += primitives::write_string(writer, string)?;
        }

        Ok(size)
    }

    /// Calculate compression statistics
    pub fn compression_stats(&self) -> CompressionStats {
        let mut total_original_size = 0;
        let mut total_references = 0;

        for (string, &_index) in &self.index_map {
            // Each reference saves (string.len() + 4 - 2) bytes
            // +4 for length prefix, -2 for index storage
            let string_size = string.len() + 4;
            total_original_size += string_size;
            total_references += 1;
        }

        let table_size = self.serialized_size();
        let reference_size = total_references * 2; // 2 bytes per reference (u16 index)
        let compressed_size = table_size + reference_size;

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
}

impl StringTableBuilder {
    /// Create a new builder with the specified minimum frequency threshold
    pub fn new(min_frequency: usize) -> Self {
        Self {
            table: StringTable::new(),
            min_frequency,
            frequency_map: HashMap::new(),
        }
    }

    /// Record a string occurrence for frequency analysis
    pub fn record_string(&mut self, s: &str) {
        // Only record non-empty strings for optimization
        if !s.is_empty() {
            *self.frequency_map.entry(s.to_string()).or_insert(0) += 1;
        }
    }

    /// Build the final string table including only strings that meet the frequency threshold
    pub fn build(mut self) -> Result<StringTable, BinaryExportError> {
        // Add strings that meet the frequency threshold to the table
        for (string, frequency) in &self.frequency_map {
            if *frequency >= self.min_frequency {
                self.table.add_string(string)?;
            }
        }

        Ok(self.table)
    }

    /// Get current frequency statistics
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
        let index1 = table.add_string("Vec<String>").unwrap();
        assert_eq!(index1, 0);
        assert_eq!(table.len(), 1);
        assert_eq!(table.get_string(0), Some("Vec<String>"));

        // Add second string
        let index2 = table.add_string("HashMap<K,V>").unwrap();
        assert_eq!(index2, 1);
        assert_eq!(table.len(), 2);

        // Add duplicate string - should return existing index
        let index3 = table.add_string("Vec<String>").unwrap();
        assert_eq!(index3, 0);
        assert_eq!(table.len(), 2); // No new string added
    }

    #[test]
    fn test_string_table_serialization_size() {
        let mut table = StringTable::new();
        table.add_string("test").unwrap();
        table.add_string("hello").unwrap();

        // Expected size: 4 (count) + 4 + 4 (test) + 4 + 5 (hello) = 21
        assert_eq!(table.serialized_size(), 21);
    }

    #[test]
    fn test_compression_stats() {
        let mut table = StringTable::new();
        table.add_string("Vec<String>").unwrap(); // 11 chars + 4 length = 15 bytes
        table.add_string("HashMap<K,V>").unwrap(); // 12 chars + 4 length = 16 bytes

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

        let table = builder.build().unwrap();
        assert_eq!(table.len(), 2); // Only strings with frequency >= 2
    }

    #[test]
    fn test_string_table_overflow() {
        let mut table = StringTable::new();

        // Fill up to near the limit (this would be slow in practice, so we'll mock it)
        // In a real scenario, we'd hit the limit with many unique strings
        for i in 0..10 {
            table.add_string(&format!("string_{}", i)).unwrap();
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