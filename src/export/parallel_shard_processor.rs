//! Parallel shard processor - high performance parallel JSON serialization
//!
//! This module implements parallel shard processing functionality, dividing large allocation data into shards for parallel processing,
//! significantly improving JSON serialization performance, especially on multi-core systems.

use crate::core::types::{AllocationInfo, TrackingError, TrackingResult};
use crate::export::data_localizer::LocalizedExportData;
use rayon::prelude::*;
use serde_json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Parallel shard processor configuration
#[derive(Debug, Clone)]
pub struct ParallelShardConfig {
    /// Size of each shard (allocation count)
    pub shard_size: usize,
    /// Parallel processing threshold (only enable parallel processing if the number of allocations exceeds this value)
    pub parallel_threshold: usize,
    /// Maximum number of threads (None means use system default)
    pub max_threads: Option<usize>,
    /// Whether to enable performance monitoring
    pub enable_monitoring: bool,
    /// Estimated JSON size per allocation (for pre-allocation)
    pub estimated_json_size_per_allocation: usize,
}

impl Default for ParallelShardConfig {
    fn default() -> Self {
        Self {
            shard_size: 1000,                        // Each shard contains 1000 allocations
            parallel_threshold: 2000, // Only enable parallel processing if the number of allocations exceeds 2000
            max_threads: None,        // Use system default number of threads
            enable_monitoring: true,  // Enable performance monitoring
            estimated_json_size_per_allocation: 200, // Estimated JSON size per allocation (for pre-allocation)
        }
    }
}

/// Processed shard data
#[derive(Debug, Clone)]
pub struct ProcessedShard {
    /// Serialized JSON data
    pub data: Vec<u8>,
    /// Number of allocations in the shard
    pub allocation_count: usize,
    /// Shard index
    pub shard_index: usize,
    /// Processing time (milliseconds)
    pub processing_time_ms: u64,
}

/// Parallel processing statistics
#[derive(Debug, Clone)]
pub struct ParallelProcessingStats {
    /// Total number of allocations
    pub total_allocations: usize,
    /// Number of shards
    pub shard_count: usize,
    /// Number of threads used
    pub threads_used: usize,
    /// Total processing time (milliseconds)
    pub total_processing_time_ms: u64,
    /// Average processing time per shard (milliseconds)
    pub avg_shard_processing_time_ms: f64,
    /// Parallel efficiency (acceleration ratio compared to single thread)
    pub parallel_efficiency: f64,
    /// Throughput (allocations per second)
    pub throughput_allocations_per_sec: f64,
    /// Whether parallel processing was used
    pub used_parallel_processing: bool,
    /// Total output size (bytes)
    pub total_output_size_bytes: usize,
}

/// Parallel shard processor
pub struct ParallelShardProcessor {
    /// Configuration
    config: ParallelShardConfig,
    /// Processing counter (for monitoring)
    processed_count: AtomicUsize,
}

impl ParallelShardProcessor {
    /// Create a new parallel shard processor
    pub fn new(config: ParallelShardConfig) -> Self {
        // If a maximum thread count is specified, set the rayon thread pool
        if let Some(max_threads) = config.max_threads {
            rayon::ThreadPoolBuilder::new()
                .num_threads(max_threads)
                .build_global()
                .unwrap_or_else(|e| {
                    tracing::warn!(
                        "⚠️ Failed to set thread pool size to {}: {}",
                        max_threads, e
                    );
                });
        }

        Self {
            config,
            processed_count: AtomicUsize::new(0),
        }
    }

    /// Process allocations in parallel
    pub fn process_allocations_parallel(
        &self,
        data: &LocalizedExportData,
    ) -> TrackingResult<(Vec<ProcessedShard>, ParallelProcessingStats)> {
        let start_time = Instant::now();
        let allocations = &data.allocations;

        tracing::info!(
            "🔄 Starting parallel shard processing for {} allocations...",
            allocations.len()
        );

        // Determine whether to use parallel processing
        let use_parallel = allocations.len() >= self.config.parallel_threshold;
        let actual_threads = if use_parallel {
            rayon::current_num_threads()
        } else {
            1
        };

        tracing::info!(
            "   Parallel mode: {}, threads: {}, shard size: {}",
            if use_parallel { "enabled" } else { "disabled" },
            actual_threads,
            self.config.shard_size
        );

        // Reset counter for monitoring
        self.processed_count.store(0, Ordering::Relaxed);

        // Split data into shards
        let shards: Vec<&[AllocationInfo]> = allocations.chunks(self.config.shard_size).collect();

        tracing::info!("   Shard count: {}", shards.len());

        // Parallel or serial processing of shards
        let processed_shards: TrackingResult<Vec<ProcessedShard>> = if use_parallel {
            shards
                .into_par_iter()
                .enumerate()
                .map(|(index, shard)| self.process_shard_optimized(shard, index))
                .collect()
        } else {
            shards
                .into_iter()
                .enumerate()
                .map(|(index, shard)| self.process_shard_optimized(shard, index))
                .collect()
        };

        let processed_shards = processed_shards?;
        let total_time = start_time.elapsed();

        // Calculate statistics
        let stats = self.calculate_processing_stats(
            &processed_shards,
            allocations.len(),
            actual_threads,
            total_time.as_millis() as u64,
            use_parallel,
        );

        // Print performance statistics
        self.print_performance_stats(&stats);

        Ok((processed_shards, stats))
    }

    /// Optimized shard processing method
    fn process_shard_optimized(
        &self,
        shard: &[AllocationInfo],
        shard_index: usize,
    ) -> TrackingResult<ProcessedShard> {
        let shard_start = Instant::now();

        // Estimate output size and preallocate buffers
        let estimated_size = shard.len() * self.config.estimated_json_size_per_allocation;
        let mut output_buffer = Vec::with_capacity(estimated_size);

        // Use serde_json's efficient API to serialize directly to byte vector
        // This is more reliable than manual formatting and performs well
        serde_json::to_writer(&mut output_buffer, shard).map_err(|e| {
            TrackingError::ExportError(format!("Shard {} serialization failed: {}", shard_index, e))
        })?;

        let processing_time = shard_start.elapsed();

        // Update processed counter
        self.processed_count
            .fetch_add(shard.len(), Ordering::Relaxed);

        // If monitoring is enabled, print progress
        if self.config.enable_monitoring && shard_index % 10 == 0 {
            let _processed = self.processed_count.load(Ordering::Relaxed);
            tracing::info!(
                "   Shard {} completed: {} allocations, {} bytes, {:?}",
                shard_index,
                shard.len(),
                output_buffer.len(),
                processing_time
            );
        }

        Ok(ProcessedShard {
            data: output_buffer,
            allocation_count: shard.len(),
            shard_index,
            processing_time_ms: processing_time.as_millis() as u64,
        })
    }

    /// Calculate processing statistics
    fn calculate_processing_stats(
        &self,
        shards: &[ProcessedShard],
        total_allocations: usize,
        threads_used: usize,
        total_time_ms: u64,
        used_parallel: bool,
    ) -> ParallelProcessingStats {
        let total_output_size: usize = shards.iter().map(|s| s.data.len()).sum();
        let avg_shard_time: f64 = if !shards.is_empty() {
            shards
                .iter()
                .map(|s| s.processing_time_ms as f64)
                .sum::<f64>()
                / shards.len() as f64
        } else {
            0.0
        };

        let throughput = if total_time_ms > 0 {
            (total_allocations as f64 * 1000.0) / total_time_ms as f64
        } else {
            0.0
        };

        // Estimate parallel efficiency (simplified calculation)
        let parallel_efficiency = if used_parallel && threads_used > 1 {
            // In an ideal scenario, N threads should provide close to N times the performance
            // Actual efficiency = Actual acceleration ratio / Theoretical acceleration ratio
            let theoretical_speedup = threads_used as f64;
            let estimated_sequential_time = avg_shard_time * shards.len() as f64;
            let actual_speedup = if total_time_ms > 0 {
                estimated_sequential_time / total_time_ms as f64
            } else {
                1.0
            };
            (actual_speedup / theoretical_speedup).min(1.0)
        } else {
            1.0 // Single thread efficiency is 100%
        };

        ParallelProcessingStats {
            total_allocations,
            shard_count: shards.len(),
            threads_used,
            total_processing_time_ms: total_time_ms,
            avg_shard_processing_time_ms: avg_shard_time,
            parallel_efficiency,
            throughput_allocations_per_sec: throughput,
            used_parallel_processing: used_parallel,
            total_output_size_bytes: total_output_size,
        }
    }

    /// Print performance statistics
    fn print_performance_stats(&self, stats: &ParallelProcessingStats) {
        tracing::info!("✅ Parallel shard processing completed:");
        tracing::info!("   Total allocations: {}", stats.total_allocations);
        tracing::info!("   Shard count: {}", stats.shard_count);
        tracing::info!("   Threads used: {}", stats.threads_used);
        tracing::info!("   Total time: {}ms", stats.total_processing_time_ms);
        tracing::info!(
            "   Average shard time: {:.2}ms",
            stats.avg_shard_processing_time_ms
        );
        tracing::info!(
            "   Throughput: {:.0} allocations/sec",
            stats.throughput_allocations_per_sec
        );
        tracing::info!(
            "   Output size: {:.2} MB",
            stats.total_output_size_bytes as f64 / 1024.0 / 1024.0
        );

        if stats.used_parallel_processing {
            tracing::info!(
                "   Parallel efficiency: {:.1}%",
                stats.parallel_efficiency * 100.0
            );
            let speedup = stats.parallel_efficiency * stats.threads_used as f64;
            tracing::info!("   Actual speedup: {:.2}x", speedup);
        }
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ParallelShardConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: ParallelShardConfig) {
        self.config = config;
    }

    /// Get processed count
    pub fn get_processed_count(&self) -> usize {
        self.processed_count.load(Ordering::Relaxed)
    }
}

impl Default for ParallelShardProcessor {
    fn default() -> Self {
        Self::new(ParallelShardConfig::default())
    }
}

/// Convenience function: Fast parallel processing of allocation data
pub fn process_allocations_fast(
    data: &LocalizedExportData,
) -> TrackingResult<(Vec<ProcessedShard>, ParallelProcessingStats)> {
    let processor = ParallelShardProcessor::default();
    processor.process_allocations_parallel(data)
}

/// Convenience function: Parallel processing with custom configuration
pub fn process_allocations_with_config(
    data: &LocalizedExportData,
    config: ParallelShardConfig,
) -> TrackingResult<(Vec<ProcessedShard>, ParallelProcessingStats)> {
    let processor = ParallelShardProcessor::new(config);
    processor.process_allocations_parallel(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::unsafe_ffi_tracker::UnsafeFFIStats;
    use crate::core::types::{MemoryStats, ScopeInfo};
    use std::time::Instant;

    fn create_test_data(allocation_count: usize) -> LocalizedExportData {
        let mut allocations = Vec::new();
        for i in 0..allocation_count {
            allocations.push(AllocationInfo {
                ptr: 0x1000 + i,
                size: 64 + (i % 100),
                type_name: Some(format!("TestType{}", i % 10)),
                var_name: Some(format!("var_{}", i)),
                scope_name: Some(format!("scope_{}", i % 5)),
                timestamp_alloc: 1000000 + i as u64,
                timestamp_dealloc: None,
                thread_id: format!("test_thread_{}", i % 3),
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
            });
        }

        LocalizedExportData {
            allocations,
            enhanced_allocations: Vec::new(),
            stats: MemoryStats::default(),
            ffi_stats: UnsafeFFIStats::default(),
            scope_info: Vec::<ScopeInfo>::new(),
            timestamp: Instant::now(),
        }
    }

    #[test]
    fn test_parallel_shard_processor_creation() {
        let config = ParallelShardConfig::default();
        let processor = ParallelShardProcessor::new(config);
        assert_eq!(processor.get_config().shard_size, 1000);
    }

    #[test]
    fn test_small_dataset_sequential_processing() {
        let data = create_test_data(100); // Small dataset, should use sequential processing
        let processor = ParallelShardProcessor::default();

        let result = processor.process_allocations_parallel(&data);
        assert!(result.is_ok());

        let (shards, stats) = result.unwrap();
        assert_eq!(stats.total_allocations, 100);
        assert!(!stats.used_parallel_processing); // Should use sequential processing
        assert_eq!(shards.len(), 1); // Only one shard
    }

    #[test]
    fn test_large_dataset_parallel_processing() {
        let data = create_test_data(5000); // Large dataset, should use parallel processing
        let processor = ParallelShardProcessor::default();

        let result = processor.process_allocations_parallel(&data);
        assert!(result.is_ok());

        let (shards, stats) = result.unwrap();
        assert_eq!(stats.total_allocations, 5000);
        assert!(stats.used_parallel_processing); // Should use parallel processing
        assert!(shards.len() > 1); // Should have multiple shards

        // Verify that the total number of allocations processed equals the original data
        let total_processed: usize = shards.iter().map(|s| s.allocation_count).sum();
        assert_eq!(total_processed, 5000);
    }

    #[test]
    fn test_custom_config() {
        let config = ParallelShardConfig {
            shard_size: 500,
            parallel_threshold: 1000,
            max_threads: Some(2),
            enable_monitoring: false,
            estimated_json_size_per_allocation: 150,
        };

        let data = create_test_data(2000);
        let processor = ParallelShardProcessor::new(config);

        let result = processor.process_allocations_parallel(&data);
        assert!(result.is_ok());

        let (shards, stats) = result.unwrap();
        assert_eq!(stats.total_allocations, 2000);
        assert_eq!(shards.len(), 4); // 2000 / 500 = 4 shards
    }

    #[test]
    fn test_convenience_functions() {
        let data = create_test_data(1500);

        // Test fast processing function
        let result = process_allocations_fast(&data);
        assert!(result.is_ok());

        // Test custom configuration function
        let config = ParallelShardConfig {
            shard_size: 300,
            ..Default::default()
        };
        let result = process_allocations_with_config(&data, config);
        assert!(result.is_ok());

        let (shards, _) = result.unwrap();
        assert_eq!(shards.len(), 5); // 1500 / 300 = 5 shards
    }

    #[test]
    fn test_processed_shard_structure() {
        let data = create_test_data(100);
        let processor = ParallelShardProcessor::default();

        let result = processor.process_allocations_parallel(&data);
        assert!(result.is_ok());

        let (shards, _) = result.unwrap();
        assert_eq!(shards.len(), 1);

        let shard = &shards[0];
        assert_eq!(shard.allocation_count, 100);
        assert_eq!(shard.shard_index, 0);
        assert!(!shard.data.is_empty());
        // processing_time_ms is u64, always >= 0, so just check it exists
        assert!(shard.processing_time_ms < u64::MAX);

        // Verify that the JSON data is valid
        let parsed: Result<Vec<AllocationInfo>, _> = serde_json::from_slice(&shard.data);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().len(), 100);
    }
}
