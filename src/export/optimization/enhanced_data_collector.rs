//! Enhanced data collector for optimized binary export
//!
//! This module provides improved data collection mechanisms that avoid
//! long-term locks and support cancellation, timeouts, and batch processing.

use crate::core::types::{AllocationInfo, TrackingResult, TrackingError};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};

/// Configuration for enhanced data collection
#[derive(Debug, Clone)]
pub struct CollectionConfig {
    /// Maximum number of allocations to collect in one batch
    pub batch_size: usize,
    /// Timeout for each batch collection operation
    pub batch_timeout: Duration,
    /// Maximum total time for complete collection
    pub total_timeout: Duration,
    /// Whether to use lock-free collection when possible
    pub lock_free_enabled: bool,
    /// Minimum delay between batches to reduce lock contention
    pub batch_delay: Duration,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            batch_timeout: Duration::from_millis(100),
            total_timeout: Duration::from_secs(30),
            lock_free_enabled: true,
            batch_delay: Duration::from_millis(1),
        }
    }
}

impl CollectionConfig {
    /// Fast collection configuration - prioritize speed over completeness
    pub fn fast() -> Self {
        Self {
            batch_size: 2000,
            batch_timeout: Duration::from_millis(50),
            total_timeout: Duration::from_secs(5),
            lock_free_enabled: true,
            batch_delay: Duration::from_millis(0),
        }
    }

    /// Safe collection configuration - prioritize completeness over speed
    pub fn safe() -> Self {
        Self {
            batch_size: 500,
            batch_timeout: Duration::from_millis(200),
            total_timeout: Duration::from_secs(60),
            lock_free_enabled: false,
            batch_delay: Duration::from_millis(5),
        }
    }

    /// Streaming configuration for very large datasets
    pub fn streaming() -> Self {
        Self {
            batch_size: 100,
            batch_timeout: Duration::from_millis(20),
            total_timeout: Duration::from_secs(300), // 5 minutes for very large datasets
            lock_free_enabled: true,
            batch_delay: Duration::from_millis(1),
        }
    }
}

/// Data availability status
#[derive(Debug, Clone, PartialEq)]
pub enum DataAvailability {
    /// Data is immediately available
    Available { estimated_count: usize },
    /// Data is available but may require waiting
    Delayed { estimated_wait: Duration, estimated_count: usize },
    /// Data is currently locked or unavailable
    Locked { retry_after: Duration },
    /// Data collection would likely timeout
    Timeout { reason: String },
}

/// Progress callback for data collection
pub type ProgressCallback = Arc<dyn Fn(CollectionProgress) + Send + Sync>;

/// Progress information during data collection
#[derive(Debug, Clone)]
pub struct CollectionProgress {
    pub collected_count: usize,
    pub estimated_total: usize,
    pub elapsed_time: Duration,
    pub current_batch: usize,
    pub total_batches: usize,
    pub throughput: f64, // items per second
}

/// Cancellation token for data collection operations
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced data collector with optimized performance characteristics
pub struct EnhancedDataCollector {
    config: CollectionConfig,
}

impl EnhancedDataCollector {
    /// Create a new enhanced data collector with default configuration
    pub fn new() -> Self {
        Self {
            config: CollectionConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: CollectionConfig) -> Self {
        Self { config }
    }

    /// Check data availability without blocking
    pub fn check_availability(
        &self,
        tracker: &crate::core::tracker::MemoryTracker,
    ) -> DataAvailability {
        let start = Instant::now();
        
        // Try to quickly estimate the data size
        match tracker.get_stats() {
            Ok(stats) => {
                let estimated_count = stats.active_allocations;
                let estimated_time = Duration::from_millis(
                    (estimated_count as u64 * 10) / 1000 // Rough estimate: 10μs per allocation
                );

                if estimated_time > self.config.total_timeout {
                    DataAvailability::Timeout {
                        reason: format!(
                            "Estimated collection time ({:?}) exceeds timeout ({:?})",
                            estimated_time, self.config.total_timeout
                        ),
                    }
                } else if estimated_time > Duration::from_millis(100) {
                    DataAvailability::Delayed {
                        estimated_wait: estimated_time,
                        estimated_count,
                    }
                } else {
                    DataAvailability::Available { estimated_count }
                }
            }
            Err(_) => DataAvailability::Locked {
                retry_after: Duration::from_millis(100),
            },
        }
    }

    /// Collect data in batches with timeout and cancellation support
    pub fn collect_batch(
        &self,
        tracker: &crate::core::tracker::MemoryTracker,
        offset: usize,
        limit: usize,
        cancellation: &CancellationToken,
    ) -> TrackingResult<Vec<AllocationInfo>> {
        if cancellation.is_cancelled() {
            return Err(TrackingError::OperationCancelled);
        }

        let start = Instant::now();
        let mut collected = Vec::new();
        let mut current_offset = offset;
        let target_count = limit.min(self.config.batch_size);

        // Use a timeout for the entire batch operation
        while collected.len() < target_count && start.elapsed() < self.config.batch_timeout {
            if cancellation.is_cancelled() {
                return Err(TrackingError::OperationCancelled);
            }

            // Try to get a small chunk of data with minimal lock time
            let chunk_size = (target_count - collected.len()).min(100); // Small chunks
            match self.collect_chunk(tracker, current_offset, chunk_size) {
                Ok(mut chunk) => {
                    current_offset += chunk.len();
                    collected.append(&mut chunk);
                    
                    // Small delay to reduce lock contention
                    if !self.config.batch_delay.is_zero() {
                        std::thread::sleep(self.config.batch_delay);
                    }
                }
                Err(e) => {
                    if collected.is_empty() {
                        return Err(e);
                    } else {
                        // Return partial results if we have some data
                        println!("⚠️  Partial batch collection: got {} items, error: {}", collected.len(), e);
                        break;
                    }
                }
            }
        }

        if collected.is_empty() {
            Err(TrackingError::CollectionTimeout {
                timeout: self.config.batch_timeout,
            })
        } else {
            Ok(collected)
        }
    }

    /// Collect all data with progress reporting and cancellation
    pub fn collect_all(
        &self,
        tracker: &crate::core::tracker::MemoryTracker,
        progress_callback: Option<ProgressCallback>,
        cancellation: &CancellationToken,
    ) -> TrackingResult<Vec<AllocationInfo>> {
        let start_time = Instant::now();
        let mut all_allocations = Vec::new();
        let mut current_offset = 0;
        let mut batch_count = 0;

        // Estimate total count for progress reporting
        let estimated_total = match tracker.get_stats() {
            Ok(stats) => stats.active_allocations,
            Err(_) => 0,
        };

        println!("🚀 Starting enhanced data collection...");
        println!("   - Estimated items: {}", estimated_total);
        println!("   - Batch size: {}", self.config.batch_size);
        println!("   - Total timeout: {:?}", self.config.total_timeout);

        loop {
            if cancellation.is_cancelled() {
                println!("❌ Data collection cancelled by user");
                return Err(TrackingError::OperationCancelled);
            }

            if start_time.elapsed() > self.config.total_timeout {
                println!("⏰ Data collection timeout after {:?}", start_time.elapsed());
                if all_allocations.is_empty() {
                    return Err(TrackingError::CollectionTimeout {
                        timeout: self.config.total_timeout,
                    });
                } else {
                    println!("   - Returning {} partial results", all_allocations.len());
                    break;
                }
            }

            // Collect next batch
            match self.collect_batch(tracker, current_offset, self.config.batch_size, cancellation) {
                Ok(batch) => {
                    if batch.is_empty() {
                        // No more data available
                        break;
                    }

                    current_offset += batch.len();
                    all_allocations.extend(batch);
                    batch_count += 1;

                    // Report progress
                    if let Some(ref callback) = progress_callback {
                        let progress = CollectionProgress {
                            collected_count: all_allocations.len(),
                            estimated_total: estimated_total.max(all_allocations.len()),
                            elapsed_time: start_time.elapsed(),
                            current_batch: batch_count,
                            total_batches: (estimated_total / self.config.batch_size) + 1,
                            throughput: all_allocations.len() as f64 / start_time.elapsed().as_secs_f64(),
                        };
                        callback(progress);
                    }

                    // Log progress periodically
                    if batch_count % 10 == 0 {
                        println!(
                            "📊 Progress: {} items collected in {:?} ({:.1} items/sec)",
                            all_allocations.len(),
                            start_time.elapsed(),
                            all_allocations.len() as f64 / start_time.elapsed().as_secs_f64()
                        );
                    }
                }
                Err(TrackingError::OperationCancelled) => {
                    return Err(TrackingError::OperationCancelled);
                }
                Err(e) => {
                    if all_allocations.is_empty() {
                        return Err(e);
                    } else {
                        println!("⚠️  Collection ended with error: {}", e);
                        println!("   - Returning {} partial results", all_allocations.len());
                        break;
                    }
                }
            }
        }

        let final_duration = start_time.elapsed();
        println!("✅ Enhanced data collection completed!");
        println!("   - Total items: {}", all_allocations.len());
        println!("   - Total time: {:?}", final_duration);
        println!("   - Throughput: {:.1} items/sec", 
                all_allocations.len() as f64 / final_duration.as_secs_f64());
        println!("   - Batches processed: {}", batch_count);

        Ok(all_allocations)
    }

    /// Collect a small chunk of data with minimal lock time
    fn collect_chunk(
        &self,
        tracker: &crate::core::tracker::MemoryTracker,
        offset: usize,
        limit: usize,
    ) -> TrackingResult<Vec<AllocationInfo>> {
        // This is a simplified implementation. In a real scenario, we would need
        // to modify the MemoryTracker to support chunked access.
        // For now, we'll use the existing method but with a timeout wrapper.
        
        let start = Instant::now();
        
        // Try to get all allocations with a timeout
        let result = std::thread::scope(|s| {
            let handle = s.spawn(|| tracker.get_all_active_allocations());
            
            // Wait for the operation to complete or timeout
            loop {
                if start.elapsed() > Duration::from_millis(50) {
                    // If we can't get the data quickly, return an error
                    return Err(TrackingError::CollectionTimeout {
                        timeout: Duration::from_millis(50),
                    });
                }
                
                if handle.is_finished() {
                    return handle.join().unwrap_or_else(|_| {
                        Err(TrackingError::InternalError("Thread panicked".to_string()))
                    });
                }
                
                std::thread::sleep(Duration::from_millis(1));
            }
        });

        match result {
            Ok(all_allocations) => {
                // Simulate chunked access by slicing the results
                let end_offset = (offset + limit).min(all_allocations.len());
                if offset >= all_allocations.len() {
                    Ok(Vec::new())
                } else {
                    Ok(all_allocations[offset..end_offset].to_vec())
                }
            }
            Err(e) => Err(e),
        }
    }
}

impl Default for EnhancedDataCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_config_defaults() {
        let config = CollectionConfig::default();
        assert_eq!(config.batch_size, 1000);
        assert!(config.lock_free_enabled);
    }

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_data_availability_variants() {
        let available = DataAvailability::Available { estimated_count: 100 };
        match available {
            DataAvailability::Available { estimated_count } => {
                assert_eq!(estimated_count, 100);
            }
            _ => panic!("Expected Available variant"),
        }
    }
}