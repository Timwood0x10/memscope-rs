//! Targeted optimizations for actual performance bottlenecks
//! 
//! Based on profiling data, these optimizations target real performance issues
//! rather than theoretical ones.

use crate::core::smart_optimization::{SmartMutex, SafeUnwrap, SmartStats};
use crate::core::atomic_stats::SimpleMemoryStats;
use crate::core::simple_mutex::SimpleMutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::time::Instant;

/// Optimized statistics collector that avoids lock contention
pub struct FastStatsCollector {
    // Core memory statistics using cache-line optimized structure
    memory_stats: SimpleMemoryStats,
    
    // Additional counters for deallocations
    pub deallocation_count: AtomicU64,
    pub total_deallocated: AtomicU64,
    
    // Low-frequency detailed data uses simple mutex
    detailed_data: SimpleMutex<DetailedStatsData>,
}

#[derive(Default)]
struct DetailedStatsData {
    allocation_sizes: Vec<usize>,
    peak_memory: usize,
    allocation_histogram: HashMap<usize, u64>,
}

impl FastStatsCollector {
    pub fn new() -> Self {
        Self {
            memory_stats: SimpleMemoryStats::new(),
            deallocation_count: AtomicU64::new(0),
            total_deallocated: AtomicU64::new(0),
            detailed_data: SimpleMutex::new(DetailedStatsData::default()),
        }
    }

    /// Fast path: just increment counters (no locks)
    pub fn record_allocation_fast(&self, size: usize) {
        self.memory_stats.record_allocation_fast(size as u64);
    }

    /// Slow path: record detailed data (uses lock, but only when needed)
    pub fn record_allocation_detailed(&self, size: usize) {
        // Use the optimized detailed recording
        self.memory_stats.record_allocation_detailed(size as u64);
        
        // Only do histogram tracking if we can get the lock quickly
        #[cfg(feature = "parking-lot")]
        if let Some(mut data) = self.detailed_data.try_lock() {
            data.allocation_sizes.push(size);
            *data.allocation_histogram.entry(size).or_insert(0) += 1;
        }
        
        #[cfg(not(feature = "parking-lot"))]
        if let Ok(mut data) = self.detailed_data.try_lock() {
            data.allocation_sizes.push(size);
            *data.allocation_histogram.entry(size).or_insert(0) += 1;
        }
        // If we can't get the lock, we just skip histogram tracking
    }

    pub fn record_deallocation(&self, size: usize) {
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
        self.total_deallocated.fetch_add(size as u64, Ordering::Relaxed);
        self.memory_stats.record_deallocation(size as u64);
    }

    /// Get basic stats without any locks
    pub fn get_basic_stats(&self) -> BasicStats {
        let snapshot = self.memory_stats.snapshot();
        BasicStats {
            allocation_count: snapshot.total_allocations,
            deallocation_count: self.deallocation_count.load(Ordering::Relaxed),
            total_allocated: snapshot.total_allocated,
            total_deallocated: self.total_deallocated.load(Ordering::Relaxed),
        }
    }

    /// Get detailed stats (may use lock, but with timeout)
    pub fn get_detailed_stats(&self) -> Option<DetailedStats> {
        let basic = self.get_basic_stats();
        let snapshot = self.memory_stats.snapshot();
        
        #[cfg(feature = "parking-lot")]
        if let Some(data) = self.detailed_data.try_lock() {
            Some(DetailedStats {
                basic,
                peak_memory: snapshot.peak_memory as usize,
                avg_allocation_size: if !data.allocation_sizes.is_empty() {
                    data.allocation_sizes.iter().sum::<usize>() / data.allocation_sizes.len()
                } else {
                    0
                },
                allocation_count_by_size: data.allocation_histogram.clone(),
            })
        } else {
            None
        }
        
        #[cfg(not(feature = "parking-lot"))]
        if let Ok(data) = self.detailed_data.try_lock() {
            Some(DetailedStats {
                basic,
                peak_memory: snapshot.peak_memory as usize,
                avg_allocation_size: if !data.allocation_sizes.is_empty() {
                    data.allocation_sizes.iter().sum::<usize>() / data.allocation_sizes.len()
                } else {
                    0
                },
                allocation_count_by_size: data.allocation_histogram.clone(),
            })
        } else {
            // If we can't get detailed stats, return basic stats only
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct BasicStats {
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub total_allocated: u64,
    pub total_deallocated: u64,
}

#[derive(Debug, Clone)]
pub struct DetailedStats {
    pub basic: BasicStats,
    pub peak_memory: usize,
    pub avg_allocation_size: usize,
    pub allocation_count_by_size: HashMap<usize, u64>,
}

/// Optimized unwrap replacement for hot paths
pub fn fast_unwrap_or_default<T: Default>(option: Option<T>) -> T {
    match option {
        Some(value) => value,
        None => {
            // In hot paths, just return default without logging
            T::default()
        }
    }
}

/// Optimized unwrap for results in hot paths
pub fn fast_unwrap_result_or_default<T: Default, E>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(_) => T::default(),
    }
}

/// Batch operations to reduce lock frequency
pub struct BatchProcessor<T> {
    batch: SimpleMutex<Vec<T>>,
    batch_size: usize,
    processor: Box<dyn Fn(&[T]) + Send + Sync>,
}

impl<T> BatchProcessor<T> {
    pub fn new<F>(batch_size: usize, processor: F) -> Self 
    where 
        F: Fn(&[T]) + Send + Sync + 'static,
    {
        Self {
            batch: SimpleMutex::new(Vec::with_capacity(batch_size)),
            batch_size,
            processor: Box::new(processor),
        }
    }

    pub fn add(&self, item: T) {
        let should_process = {
            #[cfg(feature = "parking-lot")]
            {
                let mut batch = self.batch.lock();
                batch.push(item);
                batch.len() >= self.batch_size
            }
            #[cfg(not(feature = "parking-lot"))]
            {
                let mut batch = self.batch.lock().expect("Mutex poisoned");
                batch.push(item);
                batch.len() >= self.batch_size
            }
        };

        if should_process {
            self.process_batch();
        }
    }

    fn process_batch(&self) {
        let batch = {
            #[cfg(feature = "parking-lot")]
            {
                let mut batch_guard = self.batch.lock();
                std::mem::take(&mut *batch_guard)
            }
            #[cfg(not(feature = "parking-lot"))]
            {
                let mut batch_guard = self.batch.lock().expect("Mutex poisoned");
                std::mem::take(&mut *batch_guard)
            }
        };

        if !batch.is_empty() {
            (self.processor)(&batch);
        }
    }

    pub fn flush(&self) {
        self.process_batch();
    }
}

/// Performance-aware string handling
pub fn efficient_string_concat(parts: &[&str]) -> String {
    if parts.is_empty() {
        return String::new();
    }
    
    if parts.len() == 1 {
        return parts[0].to_string();
    }

    // Pre-calculate capacity to avoid reallocations
    let total_len: usize = parts.iter().map(|s| s.len()).sum();
    let mut result = String::with_capacity(total_len);
    
    for part in parts {
        result.push_str(part);
    }
    
    result
}

/// Optimized clone avoidance for common patterns
pub fn clone_if_needed<T: Clone>(value: &T, need_owned: bool) -> std::borrow::Cow<T> {
    if need_owned {
        std::borrow::Cow::Owned(value.clone())
    } else {
        std::borrow::Cow::Borrowed(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;

    #[test]
    fn test_fast_stats_collector() {
        let collector = FastStatsCollector::new();
        
        // Test fast path
        for i in 0..1000 {
            collector.record_allocation_fast(64);
        }
        
        let stats = collector.get_basic_stats();
        assert_eq!(stats.allocation_count, 1000);
        assert_eq!(stats.total_allocated, 64000);
    }

    #[test]
    fn test_fast_stats_concurrent() {
        let collector = Arc::new(FastStatsCollector::new());
        let mut handles = vec![];

        // Test concurrent access
        for _ in 0..10 {
            let collector_clone = collector.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    collector_clone.record_allocation_fast(32);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = collector.get_basic_stats();
        assert_eq!(stats.allocation_count, 1000);
    }

    #[test]
    fn test_batch_processor() {
        let processed_items = Arc::new(SimpleMutex::new(Vec::new()));
        let processed_clone = processed_items.clone();
        
        let processor = BatchProcessor::new(3, move |batch: &[i32]| {
            #[cfg(feature = "parking-lot")]
            {
                let mut items = processed_clone.lock();
                items.extend_from_slice(batch);
            }
            #[cfg(not(feature = "parking-lot"))]
            {
                let mut items = processed_clone.lock().expect("Mutex poisoned");
                items.extend_from_slice(batch);
            }
        });

        // Add items one by one
        processor.add(1);
        processor.add(2);
        processor.add(3); // This should trigger processing

        // Give it a moment to process
        std::thread::sleep(std::time::Duration::from_millis(10));

        #[cfg(feature = "parking-lot")]
        {
            let items = processed_items.lock();
            assert_eq!(*items, vec![1, 2, 3]);
        }
        #[cfg(not(feature = "parking-lot"))]
        {
            let items = processed_items.lock().expect("Mutex poisoned");
            assert_eq!(*items, vec![1, 2, 3]);
        }
    }

    #[test]
    fn test_efficient_string_concat() {
        let parts = vec!["Hello", " ", "World", "!"];
        let result = efficient_string_concat(&parts);
        assert_eq!(result, "Hello World!");
        
        // Test empty case
        let empty_result = efficient_string_concat(&[]);
        assert_eq!(empty_result, "");
        
        // Test single item
        let single_result = efficient_string_concat(&["test"]);
        assert_eq!(single_result, "test");
    }
}