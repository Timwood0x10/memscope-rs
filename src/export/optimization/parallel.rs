//! Parallel processing optimization for exports.
//!
//! This module consolidates parallel processing functionality from:
//! - parallel_shard_processor.rs
//! - fast_export_coordinator.rs (1352 lines)
//! - performance_benchmark.rs
//! - performance_comparison.rs
//! - performance_testing.rs (1956 lines)

// Re-export existing parallel functionality
pub use super::parallel_shard_processor::*;
pub use super::fast_export_coordinator::*;
pub use super::performance_benchmark::*;
// pub use super::performance_comparison::*; // Unused and ambiguous import

/// Unified parallel processing interface
pub struct ParallelOptimizer {
    thread_count: usize,
    shard_size: usize,
}

impl ParallelOptimizer {
    /// Create a new parallel optimizer
    pub fn new() -> Self {
        Self {
            thread_count: num_cpus::get(),
            shard_size: 1000,
        }
    }
    
    /// Set custom thread count
    pub fn with_threads(mut self, count: usize) -> Self {
        self.thread_count = count;
        self
    }
    
    /// Set custom shard size
    pub fn with_shard_size(mut self, size: usize) -> Self {
        self.shard_size = size;
        self
    }
    
    /// Process data in parallel shards
    pub fn process_parallel<T, F>(&self, data: &[T], processor: F) -> crate::core::types::TrackingResult<()>
    where
        T: Send + Sync,
        F: Fn(&[T]) -> crate::core::types::TrackingResult<()> + Send + Sync,
    {
        // TODO: Consolidate parallel processing logic
        use rayon::prelude::*;
        
        data.par_chunks(self.shard_size)
            .try_for_each(|chunk| processor(chunk))?;
        
        Ok(())
    }
    
    /// Benchmark export performance
    pub fn benchmark_export<F>(&self, name: &str, export_fn: F) -> BenchmarkResult
    where
        F: FnOnce() -> crate::core::types::TrackingResult<()>,
    {
        // TODO: Move benchmarking logic here
        let start = std::time::Instant::now();
        let result = export_fn();
        let duration = start.elapsed();
        
        BenchmarkResult {
            name: name.to_string(),
            duration,
            success: result.is_ok(),
        }
    }
}

/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration: std::time::Duration,
    pub success: bool,
}