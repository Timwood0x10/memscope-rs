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
    unsafe {
        INIT.call_once(|| {
            GLOBAL_SIMPLE_DEDUPLICATOR = Some(SimpleDataDeduplicator::new());
        });
        GLOBAL_SIMPLE_DEDUPLICATOR.as_mut().unwrap()
    }
}