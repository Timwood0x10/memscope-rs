//! Multi-level filtering engine for efficient allocation record filtering
//!
//! This module provides a sophisticated filtering system that uses multiple
//! levels of filtering to efficiently narrow down allocation records:
//! 1. Index-based pre-filtering using quick filter data
//! 2. Bloom filter checks for string-based filters
//! 3. Precise filtering on loaded records

use crate::core::types::AllocationInfo;
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::index::{BinaryIndex, QuickFilterData};
use crate::export::binary::selective_reader::AllocationFilter;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Multi-level filtering engine for efficient record filtering
pub struct FilterEngine {
    /// Reference to the binary index for fast lookups
    index: Arc<BinaryIndex>,

    /// Statistics about filtering performance
    stats: FilterStats,
}

/// Statistics about filtering performance
#[derive(Debug, Clone, Default)]
pub struct FilterStats {
    /// Total number of filter operations performed
    pub total_operations: u64,

    /// Number of records eliminated by index pre-filtering
    pub index_filtered_out: u64,

    /// Number of records eliminated by bloom filter checks
    pub bloom_filtered_out: u64,

    /// Number of records eliminated by precise filtering
    pub precise_filtered_out: u64,

    /// Total time spent on filtering (in microseconds)
    pub total_filter_time_us: u64,

    /// Time spent on index pre-filtering (in microseconds)
    pub index_filter_time_us: u64,

    /// Time spent on bloom filter checks (in microseconds)
    pub bloom_filter_time_us: u64,

    /// Time spent on precise filtering (in microseconds)
    pub precise_filter_time_us: u64,
}

impl FilterStats {
    /// Calculate the efficiency of index pre-filtering
    pub fn index_filter_efficiency(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            (self.index_filtered_out as f64 / self.total_operations as f64) * 100.0
        }
    }

    /// Calculate the efficiency of bloom filtering
    pub fn bloom_filter_efficiency(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            (self.bloom_filtered_out as f64 / self.total_operations as f64) * 100.0
        }
    }

    /// Calculate the overall filtering efficiency
    pub fn overall_efficiency(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            let total_filtered =
                self.index_filtered_out + self.bloom_filtered_out + self.precise_filtered_out;
            (total_filtered as f64 / self.total_operations as f64) * 100.0
        }
    }

    /// Get average time per filter operation (in microseconds)
    pub fn avg_filter_time_us(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            self.total_filter_time_us as f64 / self.total_operations as f64
        }
    }
}

impl FilterEngine {
    /// Create a new filter engine with the given index
    pub fn new(index: Arc<BinaryIndex>) -> Self {
        Self {
            index,
            stats: FilterStats::default(),
        }
    }

    /// Perform multi-level filtering to get candidate record indices
    pub fn filter_candidates(
        &mut self,
        filters: &[AllocationFilter],
    ) -> Result<Vec<usize>, BinaryExportError> {
        let start_time = std::time::Instant::now();

        let total_records = self.index.record_count() as usize;
        let mut candidates: Vec<usize> = (0..total_records).collect();

        self.stats.total_operations += 1;

        // Level 1: Index-based pre-filtering using quick filter data
        if let Some(ref quick_filter) = self.index.allocations.quick_filter_data {
            let index_start = std::time::Instant::now();
            candidates = self.apply_index_prefiltering(&candidates, filters, quick_filter)?;
            self.stats.index_filter_time_us += index_start.elapsed().as_micros() as u64;
            self.stats.index_filtered_out += (total_records - candidates.len()) as u64;
        }

        // Level 2: Bloom filter checks for string-based filters
        let bloom_start = std::time::Instant::now();
        let before_bloom = candidates.len();
        candidates = self.apply_bloom_filter_checks(&candidates, filters)?;
        self.stats.bloom_filter_time_us += bloom_start.elapsed().as_micros() as u64;
        self.stats.bloom_filtered_out += (before_bloom - candidates.len()) as u64;

        self.stats.total_filter_time_us += start_time.elapsed().as_micros() as u64;

        Ok(candidates)
    }

    /// Apply precise filtering to loaded allocation records
    pub fn apply_precise_filters(
        &mut self,
        allocations: Vec<AllocationInfo>,
        filters: &[AllocationFilter],
    ) -> Result<Vec<AllocationInfo>, BinaryExportError> {
        if filters.is_empty() {
            return Ok(allocations);
        }

        let start_time = std::time::Instant::now();
        let initial_count = allocations.len();

        let filtered: Vec<AllocationInfo> = allocations
            .into_iter()
            .filter(|allocation| self.matches_all_filters(allocation, filters))
            .collect();

        let filtered_out = initial_count - filtered.len();
        self.stats.precise_filtered_out += filtered_out as u64;
        self.stats.precise_filter_time_us += start_time.elapsed().as_micros() as u64;

        Ok(filtered)
    }

    /// Check if an allocation matches all filters
    pub fn matches_all_filters(
        &self,
        allocation: &AllocationInfo,
        filters: &[AllocationFilter],
    ) -> bool {
        filters.iter().all(|filter| filter.matches(allocation))
    }

    /// Get filtering statistics
    pub fn get_stats(&self) -> &FilterStats {
        &self.stats
    }

    /// Reset filtering statistics
    pub fn reset_stats(&mut self) {
        self.stats = FilterStats::default();
    }

    // Private helper methods

    /// Apply index-based pre-filtering using quick filter data
    fn apply_index_prefiltering(
        &self,
        candidates: &[usize],
        filters: &[AllocationFilter],
        quick_filter: &QuickFilterData,
    ) -> Result<Vec<usize>, BinaryExportError> {
        let mut filtered_candidates = Vec::new();

        // Group candidates by batch for efficient processing
        let mut batch_candidates: std::collections::HashMap<usize, Vec<usize>> =
            std::collections::HashMap::new();

        for &candidate_index in candidates {
            let batch_index = candidate_index / quick_filter.batch_size;
            batch_candidates
                .entry(batch_index)
                .or_default()
                .push(candidate_index);
        }

        // Process each batch
        for (batch_index, batch_records) in batch_candidates {
            if self.batch_might_contain_matches(batch_index, filters, quick_filter) {
                filtered_candidates.extend(batch_records);
            }
        }

        Ok(filtered_candidates)
    }

    /// Check if a batch might contain records matching the filters
    fn batch_might_contain_matches(
        &self,
        batch_index: usize,
        filters: &[AllocationFilter],
        quick_filter: &QuickFilterData,
    ) -> bool {
        for filter in filters {
            match filter {
                AllocationFilter::PtrRange(min, max) => {
                    // Check if the batch's pointer range overlaps with the filter range
                    if let Some(&(batch_min, batch_max)) = quick_filter.ptr_ranges.get(batch_index)
                    {
                        if *max < batch_min || *min > batch_max {
                            return false; // No overlap, batch can't contain matches
                        }
                    }
                }
                AllocationFilter::SizeRange(min, max) => {
                    // Check if the batch's size range overlaps with the filter range
                    if let Some(&(batch_min, batch_max)) = quick_filter.size_ranges.get(batch_index)
                    {
                        if *max < batch_min || *min > batch_max {
                            return false; // No overlap, batch can't contain matches
                        }
                    }
                }
                AllocationFilter::TimestampRange(min, max) => {
                    // Check if the batch's timestamp range overlaps with the filter range
                    if let Some(&(batch_min, batch_max)) =
                        quick_filter.timestamp_ranges.get(batch_index)
                    {
                        if *max < batch_min || *min > batch_max {
                            return false; // No overlap, batch can't contain matches
                        }
                    }
                }
                _ => {
                    // Other filters can't be pre-filtered using range data
                    continue;
                }
            }
        }

        true // Batch might contain matches
    }

    /// Apply bloom filter checks for string-based filters
    fn apply_bloom_filter_checks(
        &self,
        candidates: &[usize],
        filters: &[AllocationFilter],
    ) -> Result<Vec<usize>, BinaryExportError> {
        // Extract string-based filters
        let string_filters: Vec<&AllocationFilter> = filters
            .iter()
            .filter(|f| self.is_string_based_filter(f))
            .collect();

        if string_filters.is_empty() {
            return Ok(candidates.to_vec());
        }

        let mut filtered_candidates = Vec::new();

        if let Some(ref quick_filter) = self.index.allocations.quick_filter_data {
            for &candidate_index in candidates {
                if self.candidate_might_match_string_filters(
                    candidate_index,
                    &string_filters,
                    quick_filter,
                ) {
                    filtered_candidates.push(candidate_index);
                }
            }
        } else {
            // No bloom filters available, pass through all candidates
            filtered_candidates.extend_from_slice(candidates);
        }

        Ok(filtered_candidates)
    }

    /// Check if a filter is string-based and can use bloom filters
    fn is_string_based_filter(&self, filter: &AllocationFilter) -> bool {
        matches!(
            filter,
            AllocationFilter::ThreadEquals(_)
                | AllocationFilter::ThreadContains(_)
                | AllocationFilter::TypeEquals(_)
                | AllocationFilter::TypeContains(_)
                | AllocationFilter::VarNameContains(_)
                | AllocationFilter::ScopeNameContains(_)
        )
    }

    /// Check if a candidate might match string-based filters using bloom filters
    fn candidate_might_match_string_filters(
        &self,
        _candidate_index: usize,
        string_filters: &[&AllocationFilter],
        quick_filter: &QuickFilterData,
    ) -> bool {
        for filter in string_filters {
            match filter {
                AllocationFilter::ThreadEquals(thread_id)
                | AllocationFilter::ThreadContains(thread_id) => {
                    if !self
                        .bloom_filter_might_contain(&quick_filter.thread_bloom_filter, thread_id)
                    {
                        return false;
                    }
                }
                AllocationFilter::TypeEquals(type_name)
                | AllocationFilter::TypeContains(type_name) => {
                    if !self.bloom_filter_might_contain(&quick_filter.type_bloom_filter, type_name)
                    {
                        return false;
                    }
                }
                AllocationFilter::VarNameContains(_) | AllocationFilter::ScopeNameContains(_) => {
                    // These filters don't have dedicated bloom filters yet
                    // In a full implementation, we would add more bloom filters
                    continue;
                }
                _ => continue,
            }
        }

        true // Candidate might match
    }

    /// Check if a bloom filter might contain a string
    fn bloom_filter_might_contain(&self, bloom_filter: &[u8], value: &str) -> bool {
        if bloom_filter.is_empty() {
            return true; // No bloom filter, assume it might contain the value
        }

        // Simple bloom filter check implementation
        let hash_functions = self
            .index
            .allocations
            .quick_filter_data
            .as_ref()
            .map(|qf| qf.bloom_filter_params.hash_functions)
            .unwrap_or(3);

        let filter_size_bits = bloom_filter.len() * 8;

        for i in 0..hash_functions {
            let mut hasher = DefaultHasher::new();
            value.hash(&mut hasher);
            i.hash(&mut hasher); // Add salt for different hash functions
            let hash = hasher.finish();

            let bit_index = (hash % filter_size_bits as u64) as usize;
            let byte_index = bit_index / 8;
            let bit_offset = bit_index % 8;

            if byte_index >= bloom_filter.len() {
                continue;
            }

            if (bloom_filter[byte_index] & (1 << bit_offset)) == 0 {
                return false; // Definitely not in the set
            }
        }

        true // Might be in the set (could be false positive)
    }
}

/// Builder for creating FilterEngine with custom configuration
pub struct FilterEngineBuilder {
    index: Option<Arc<BinaryIndex>>,
}

impl FilterEngineBuilder {
    /// Create a new filter engine builder
    pub fn new() -> Self {
        Self { index: None }
    }

    /// Set the binary index to use
    pub fn with_index(mut self, index: Arc<BinaryIndex>) -> Self {
        self.index = Some(index);
        self
    }

    /// Build the filter engine
    pub fn build(self) -> Result<FilterEngine, BinaryExportError> {
        let index = self.index.ok_or_else(|| {
            BinaryExportError::CorruptedData("Index is required for FilterEngine".to_string())
        })?;

        Ok(FilterEngine::new(index))
    }
}

impl Default for FilterEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for filter optimization
pub struct FilterOptimizer;

impl FilterOptimizer {
    /// Optimize a list of filters for better performance
    pub fn optimize_filters(filters: &[AllocationFilter]) -> Vec<AllocationFilter> {
        let mut optimized = filters.to_vec();

        // Sort filters by selectivity (most selective first)
        optimized.sort_by(|a, b| {
            Self::filter_selectivity(a)
                .partial_cmp(&Self::filter_selectivity(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Remove redundant filters
        Self::remove_redundant_filters(&mut optimized);

        optimized
    }

    /// Estimate the selectivity of a filter (lower is more selective)
    fn filter_selectivity(filter: &AllocationFilter) -> f64 {
        match filter {
            // Exact matches are highly selective
            AllocationFilter::ThreadEquals(_) => 0.1,
            AllocationFilter::TypeEquals(_) => 0.2,

            // Range filters depend on the range size (estimated)
            AllocationFilter::PtrRange(min, max) => {
                let range_size = max.saturating_sub(*min) as f64;
                (range_size / (usize::MAX as f64)).min(1.0)
            }
            AllocationFilter::SizeRange(min, max) => {
                let range_size = max.saturating_sub(*min) as f64;
                (range_size / 1_000_000.0).min(1.0) // Assume max reasonable size is 1MB
            }
            AllocationFilter::TimestampRange(min, max) => {
                let range_size = max.saturating_sub(*min) as f64;
                (range_size / 1_000_000_000.0).min(1.0) // Assume reasonable timestamp range
            }

            // Boolean filters are moderately selective
            AllocationFilter::LeakedOnly => 0.3,
            AllocationFilter::NotLeaked => 0.7,
            AllocationFilter::HasStackTrace => 0.4,
            AllocationFilter::NoStackTrace => 0.6,

            // Contains filters are less selective
            AllocationFilter::ThreadContains(_) => 0.5,
            AllocationFilter::TypeContains(_) => 0.6,
            AllocationFilter::VarNameContains(_) => 0.7,
            AllocationFilter::ScopeNameContains(_) => 0.8,

            // Numeric range filters
            AllocationFilter::MinBorrowCount(_) => 0.4,
            AllocationFilter::MaxBorrowCount(_) => 0.6,
            AllocationFilter::LifetimeRange(_, _) => 0.5,
        }
    }

    /// Remove redundant filters from the list
    fn remove_redundant_filters(filters: &mut Vec<AllocationFilter>) {
        // This is a simplified implementation
        // In a full implementation, we would detect overlapping ranges,
        // contradictory filters, etc.

        filters.dedup_by(|a, b| {
            // Remove exact duplicates
            std::mem::discriminant(a) == std::mem::discriminant(b)
                && format!("{:?}", a) == format!("{:?}", b)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::binary::format::FileHeader;
    use crate::export::binary::index::{BloomFilterParams, CompactAllocationIndex};
    use std::path::PathBuf;

    fn create_test_index() -> BinaryIndex {
        let header = FileHeader::new_legacy(100);
        let mut index =
            BinaryIndex::new(PathBuf::from("test.bin"), 0x123456789ABCDEF0, 1024, header);

        // Add some test data to the allocation index
        index.allocations = CompactAllocationIndex::new(1000);

        // Add quick filter data
        let mut quick_filter = QuickFilterData::new(10);
        quick_filter.ptr_ranges = vec![(0x1000, 0x2000), (0x3000, 0x4000)];
        quick_filter.size_ranges = vec![(100, 500), (600, 1000)];
        quick_filter.timestamp_ranges = vec![(1000, 2000), (3000, 4000)];
        quick_filter.bloom_filter_params = BloomFilterParams::default();
        quick_filter.thread_bloom_filter = vec![0xFF, 0x00, 0xFF, 0x00]; // Simple test pattern
        quick_filter.type_bloom_filter = vec![0x0F, 0xF0, 0x0F, 0xF0]; // Simple test pattern

        index.allocations.quick_filter_data = Some(quick_filter);

        index
    }

    #[test]
    fn test_filter_engine_creation() {
        let index = Arc::new(create_test_index());
        let engine = FilterEngine::new(index);

        assert_eq!(engine.get_stats().total_operations, 0);
    }

    #[test]
    fn test_filter_engine_builder() {
        let index = Arc::new(create_test_index());
        let engine = FilterEngineBuilder::new()
            .with_index(index)
            .build()
            .unwrap();

        assert_eq!(engine.get_stats().total_operations, 0);
    }

    #[test]
    fn test_index_prefiltering() {
        let index = Arc::new(create_test_index());
        let mut engine = FilterEngine::new(index);

        let filters = vec![
            AllocationFilter::PtrRange(0x1500, 0x1800), // Should match first batch
            AllocationFilter::SizeRange(200, 300),      // Should match first batch
        ];

        let candidates = engine.filter_candidates(&filters).unwrap();

        // Should have filtered out some candidates
        assert!(candidates.len() <= 100); // Total records in test index
        assert!(engine.get_stats().total_operations > 0);
    }

    #[test]
    fn test_bloom_filter_checks() {
        let index = Arc::new(create_test_index());
        let mut engine = FilterEngine::new(index);

        let filters = vec![
            AllocationFilter::ThreadEquals("main".to_string()),
            AllocationFilter::TypeContains("Vec".to_string()),
        ];

        let _candidates = engine.filter_candidates(&filters).unwrap();

        // Bloom filter should have processed the candidates
        // Note: bloom_filter_time_us is always >= 0 as it's u64
    }

    #[test]
    fn test_precise_filtering() {
        let index = Arc::new(create_test_index());
        let mut engine = FilterEngine::new(index);

        let allocations = vec![
            AllocationInfo {
                ptr: 0x1000,
                size: 1024,
                var_name: Some("test_var".to_string()),
                type_name: Some("Vec<u8>".to_string()),
                scope_name: None,
                timestamp_alloc: 1234567890,
                timestamp_dealloc: None,
                thread_id: "main".to_string(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: None,
                smart_pointer_info: None,
                memory_layout: None,
                generic_info: None,
                dynamic_type_info: None,
                runtime_state: None,
                stack_allocation: None,
                temporary_object: None,
                fragmentation_analysis: None,
                generic_instantiation: None,
                type_relationships: None,
                type_usage: None,
                function_call_tracking: None,
                lifecycle_tracking: None,
                access_tracking: None,
                drop_chain_analysis: None,
            },
            AllocationInfo {
                ptr: 0x2000,
                size: 512,
                var_name: Some("other_var".to_string()),
                type_name: Some("String".to_string()),
                scope_name: None,
                timestamp_alloc: 1234567891,
                timestamp_dealloc: None,
                thread_id: "worker".to_string(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: None,
                smart_pointer_info: None,
                memory_layout: None,
                generic_info: None,
                dynamic_type_info: None,
                runtime_state: None,
                stack_allocation: None,
                temporary_object: None,
                fragmentation_analysis: None,
                generic_instantiation: None,
                type_relationships: None,
                type_usage: None,
                function_call_tracking: None,
                lifecycle_tracking: None,
                access_tracking: None,
                drop_chain_analysis: None,
            },
        ];

        let filters = vec![AllocationFilter::ThreadEquals("main".to_string())];

        let filtered = engine.apply_precise_filters(allocations, &filters).unwrap();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].thread_id, "main");
        assert!(engine.get_stats().precise_filter_time_us > 0);
    }

    #[test]
    fn test_filter_stats() {
        let index = Arc::new(create_test_index());
        let mut engine = FilterEngine::new(index);

        // Perform some filtering operations
        let filters = vec![AllocationFilter::PtrRange(0x1000, 0x2000)];
        let _candidates = engine.filter_candidates(&filters).unwrap();

        let stats = engine.get_stats();
        assert!(stats.total_operations > 0);
        // Note: total_filter_time_us is always >= 0 as it's u64

        // Test stats calculations
        assert!(stats.index_filter_efficiency() >= 0.0);
        assert!(stats.bloom_filter_efficiency() >= 0.0);
        assert!(stats.overall_efficiency() >= 0.0);
        assert!(stats.avg_filter_time_us() >= 0.0);
    }

    #[test]
    fn test_filter_optimizer() {
        let filters = vec![
            AllocationFilter::ThreadContains("test".to_string()),
            AllocationFilter::ThreadEquals("main".to_string()),
            AllocationFilter::PtrRange(0x1000, 0x2000),
            AllocationFilter::LeakedOnly,
        ];

        let optimized = FilterOptimizer::optimize_filters(&filters);

        // Should have the same number of filters (no duplicates in this case)
        assert_eq!(optimized.len(), filters.len());

        // More selective filters should come first
        // ThreadEquals should come before ThreadContains
        let thread_equals_pos = optimized
            .iter()
            .position(|f| matches!(f, AllocationFilter::ThreadEquals(_)));
        let thread_contains_pos = optimized
            .iter()
            .position(|f| matches!(f, AllocationFilter::ThreadContains(_)));

        if let (Some(equals_pos), Some(contains_pos)) = (thread_equals_pos, thread_contains_pos) {
            assert!(equals_pos < contains_pos);
        }
    }

    #[test]
    fn test_matches_all_filters() {
        let index = Arc::new(create_test_index());
        let engine = FilterEngine::new(index);

        let allocation = AllocationInfo {
            ptr: 0x1000,
            size: 1024,
            var_name: Some("test_var".to_string()),
            type_name: Some("Vec<u8>".to_string()),
            scope_name: None,
            timestamp_alloc: 1234567890,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 2,
            stack_trace: Some(vec!["frame1".to_string()]),
            is_leaked: false,
            lifetime_ms: Some(1000),
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        };

        // Test matching filters
        let matching_filters = vec![
            AllocationFilter::ThreadEquals("main".to_string()),
            AllocationFilter::SizeRange(500, 2000),
            AllocationFilter::HasStackTrace,
        ];

        assert!(engine.matches_all_filters(&allocation, &matching_filters));

        // Test non-matching filters
        let non_matching_filters = vec![AllocationFilter::ThreadEquals("worker".to_string())];

        assert!(!engine.matches_all_filters(&allocation, &non_matching_filters));
    }
}
