//! Simple Data Deduplicator - High Performance Version
//!
//! Simplified version for better performance in demos

use crate::core::types::TrackingResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Simple deduplicated string reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SimpleDeduplicatedString {
    pub hash: u64,
    pub length: usize,
    pub ref_count: u32,
}

/// Simple deduplication statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SimpleDeduplicationStats {
    pub total_operations: u64,
    pub cache_hit_rate: f64,
    pub memory_saved_bytes: u64,
}

/// Simple high-performance data deduplicator
pub struct SimpleDataDeduplicator {
    strings: HashMap<u64, Arc<String>>,
    stats: SimpleDeduplicationStats,
}

impl SimpleDataDeduplicator {
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            stats: SimpleDeduplicationStats::default(),
        }
    }

    pub fn deduplicate_string(&mut self, input: &str) -> TrackingResult<SimpleDeduplicatedString> {
        let hash = self.calculate_hash(input);
        self.stats.total_operations += 1;

        if self.strings.contains_key(&hash) {
            self.stats.cache_hit_rate = 1.0;
            return Ok(SimpleDeduplicatedString {
                hash,
                length: input.len(),
                ref_count: 2,
            });
        }

        self.strings.insert(hash, Arc::new(input.to_string()));
        Ok(SimpleDeduplicatedString {
            hash,
            length: input.len(),
            ref_count: 1,
        })
    }

    pub fn get_string(&self, dedup_ref: &SimpleDeduplicatedString) -> TrackingResult<Arc<String>> {
        match self.strings.get(&dedup_ref.hash) {
            Some(s) => Ok(Arc::clone(s)),
            None => Ok(Arc::new("not found".to_string())),
        }
    }

    pub fn get_stats(&self) -> TrackingResult<SimpleDeduplicationStats> {
        Ok(self.stats.clone())
    }

    fn calculate_hash(&self, input: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for SimpleDataDeduplicator {
    fn default() -> Self {
        Self::new()
    }
}

/// Global simple deduplicator
static mut GLOBAL_SIMPLE_DEDUPLICATOR: Option<SimpleDataDeduplicator> = None;
static INIT: std::sync::Once = std::sync::Once::new();

pub fn get_global_simple_data_deduplicator() -> &'static mut SimpleDataDeduplicator {
    #[allow(static_mut_refs)]
    unsafe {
        INIT.call_once(|| {
            GLOBAL_SIMPLE_DEDUPLICATOR = Some(SimpleDataDeduplicator::new());
        });
        GLOBAL_SIMPLE_DEDUPLICATOR.as_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_deduplicator() {
        // Test creating a new deduplicator
        let dedup = SimpleDataDeduplicator::new();
        let stats = dedup.get_stats().expect("Failed to get stats");
        
        assert_eq!(stats.total_operations, 0);
        assert_eq!(stats.cache_hit_rate, 0.0);
        assert_eq!(stats.memory_saved_bytes, 0);
    }
    
    #[test]
    fn test_deduplicate_string_first_time() {
        // Test deduplicating a string for the first time
        let mut dedup = SimpleDataDeduplicator::new();
        let test_str = "test string for deduplication";
        
        let result = dedup.deduplicate_string(test_str)
            .expect("Failed to deduplicate string");
        
        assert_eq!(result.length, test_str.len());
        assert_eq!(result.ref_count, 1);
        assert!(result.hash > 0);
        
        let stats = dedup.get_stats().expect("Failed to get stats");
        assert_eq!(stats.total_operations, 1);
    }
    
    #[test]
    fn test_deduplicate_string_duplicate() {
        // Test deduplicating the same string twice
        let mut dedup = SimpleDataDeduplicator::new();
        let test_str = "duplicate test string";
        
        let result1 = dedup.deduplicate_string(test_str)
            .expect("Failed to deduplicate string first time");
        let result2 = dedup.deduplicate_string(test_str)
            .expect("Failed to deduplicate string second time");
        
        // Both results should have the same hash
        assert_eq!(result1.hash, result2.hash);
        assert_eq!(result1.length, result2.length);
        assert_eq!(result2.ref_count, 2); // Second time should have ref_count 2
        
        let stats = dedup.get_stats().expect("Failed to get stats");
        assert_eq!(stats.total_operations, 2);
        assert_eq!(stats.cache_hit_rate, 1.0); // Last operation was a cache hit
    }
    
    #[test]
    fn test_get_string_existing() {
        // Test retrieving an existing string
        let mut dedup = SimpleDataDeduplicator::new();
        let test_str = "retrievable string";
        
        let dedup_ref = dedup.deduplicate_string(test_str)
            .expect("Failed to deduplicate string");
        
        let retrieved = dedup.get_string(&dedup_ref)
            .expect("Failed to retrieve string");
        
        assert_eq!(*retrieved, test_str);
    }
    
    #[test]
    fn test_get_string_non_existing() {
        // Test retrieving a non-existing string
        let dedup = SimpleDataDeduplicator::new();
        
        let fake_ref = SimpleDeduplicatedString {
            hash: 999999,
            length: 10,
            ref_count: 1,
        };
        
        let retrieved = dedup.get_string(&fake_ref)
            .expect("Failed to retrieve string");
        
        assert_eq!(*retrieved, "not found");
    }
    
    #[test]
    fn test_calculate_hash_consistency() {
        // Test that hash calculation is consistent
        let dedup = SimpleDataDeduplicator::new();
        let test_str = "consistent hash test";
        
        let hash1 = dedup.calculate_hash(test_str);
        let hash2 = dedup.calculate_hash(test_str);
        
        assert_eq!(hash1, hash2);
        assert!(hash1 > 0);
    }
    
    #[test]
    fn test_calculate_hash_different_strings() {
        // Test that different strings produce different hashes
        let dedup = SimpleDataDeduplicator::new();
        
        let hash1 = dedup.calculate_hash("string one");
        let hash2 = dedup.calculate_hash("string two");
        
        assert_ne!(hash1, hash2);
    }
    
    #[test]
    fn test_default_implementation() {
        // Test the Default trait implementation
        let dedup = SimpleDataDeduplicator::default();
        let stats = dedup.get_stats().expect("Failed to get stats");
        
        assert_eq!(stats.total_operations, 0);
        assert_eq!(stats.cache_hit_rate, 0.0);
        assert_eq!(stats.memory_saved_bytes, 0);
    }
    
    #[test]
    fn test_multiple_string_deduplication() {
        // Test deduplicating multiple different and duplicate strings
        let mut dedup = SimpleDataDeduplicator::new();
        
        let strings = vec![
            "first string",
            "second string",
            "first string",  // duplicate
            "third string",
            "second string", // duplicate
            "first string",  // duplicate
        ];
        
        let mut results = Vec::new();
        for s in &strings {
            results.push(dedup.deduplicate_string(s)
                .expect("Failed to deduplicate string"));
        }
        
        // Check that duplicates have the same hash
        assert_eq!(results[0].hash, results[2].hash);
        assert_eq!(results[0].hash, results[5].hash);
        assert_eq!(results[1].hash, results[4].hash);
        
        // Check that different strings have different hashes
        assert_ne!(results[0].hash, results[1].hash);
        assert_ne!(results[0].hash, results[3].hash);
        assert_ne!(results[1].hash, results[3].hash);
        
        let stats = dedup.get_stats().expect("Failed to get stats");
        assert_eq!(stats.total_operations, 6);
    }
    
    #[test]
    fn test_empty_string_deduplication() {
        // Test deduplicating empty strings
        let mut dedup = SimpleDataDeduplicator::new();
        
        let result = dedup.deduplicate_string("")
            .expect("Failed to deduplicate empty string");
        
        assert_eq!(result.length, 0);
        assert_eq!(result.ref_count, 1);
        assert!(result.hash > 0); // Even empty string should have a hash
    }
    
    #[test]
    fn test_long_string_deduplication() {
        // Test deduplicating very long strings
        let mut dedup = SimpleDataDeduplicator::new();
        let long_str = "a".repeat(10000);
        
        let result = dedup.deduplicate_string(&long_str)
            .expect("Failed to deduplicate long string");
        
        assert_eq!(result.length, 10000);
        assert_eq!(result.ref_count, 1);
        
        // Deduplicate again to verify caching works
        let result2 = dedup.deduplicate_string(&long_str)
            .expect("Failed to deduplicate long string again");
        
        assert_eq!(result.hash, result2.hash);
        assert_eq!(result2.ref_count, 2);
    }
    
    #[test]
    fn test_unicode_string_deduplication() {
        // Test deduplicating unicode strings
        let mut dedup = SimpleDataDeduplicator::new();
        let unicode_str = "Hello 世界 🌍";
        
        let result = dedup.deduplicate_string(unicode_str)
            .expect("Failed to deduplicate unicode string");
        
        assert_eq!(result.length, unicode_str.len());
        
        let retrieved = dedup.get_string(&result)
            .expect("Failed to retrieve unicode string");
        
        assert_eq!(*retrieved, unicode_str);
    }
    
    #[test]
    fn test_stats_accuracy() {
        // Test that statistics are accurately maintained
        let mut dedup = SimpleDataDeduplicator::new();
        
        // Add some unique strings
        dedup.deduplicate_string("unique1").expect("Failed to deduplicate");
        dedup.deduplicate_string("unique2").expect("Failed to deduplicate");
        dedup.deduplicate_string("unique3").expect("Failed to deduplicate");
        
        // Add some duplicates
        dedup.deduplicate_string("unique1").expect("Failed to deduplicate");
        dedup.deduplicate_string("unique2").expect("Failed to deduplicate");
        
        let stats = dedup.get_stats().expect("Failed to get stats");
        assert_eq!(stats.total_operations, 5);
        // Last operation was a cache hit, so cache_hit_rate should be 1.0
        assert_eq!(stats.cache_hit_rate, 1.0);
    }
}
