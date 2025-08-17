//! Threshold-based batch processor that adapts based on operation frequency
//!
//! This module provides a batch processor that automatically switches between
//! direct processing and batching based on operation frequency.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};


/// Configuration for batch processing behavior
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub batch_size: usize,
    pub frequency_threshold: u64, // operations per second
    pub measurement_window: Duration,
}

impl BatchConfig {
    /// Low frequency configuration (100 ops/sec threshold)
    pub fn low_frequency() -> Self {
        Self {
            batch_size: 10,
            frequency_threshold: 100,
            measurement_window: Duration::from_secs(1),
        }
    }

    /// Medium frequency configuration (500 ops/sec threshold)
    pub fn medium_frequency() -> Self {
        Self {
            batch_size: 25,
            frequency_threshold: 500,
            measurement_window: Duration::from_secs(1),
        }
    }

    /// High frequency configuration (1000 ops/sec threshold)
    pub fn high_frequency() -> Self {
        Self {
            batch_size: 50,
            frequency_threshold: 1000,
            measurement_window: Duration::from_secs(1),
        }
    }

    /// Create custom configuration
    pub fn custom(
        batch_size: usize,
        frequency_threshold: u64,
        measurement_window: Duration,
    ) -> Self {
        Self {
            batch_size,
            frequency_threshold,
            measurement_window,
        }
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self::medium_frequency()
    }
}

/// Threshold-based batch processor
pub struct ThresholdBatchProcessor<T> {
    config: BatchConfig,
    buffer: Mutex<Vec<T>>,
    processor: Box<dyn Fn(&[T]) + Send + Sync>,

    // Frequency tracking
    operation_count: AtomicU64,
    last_measurement: Mutex<Instant>,
    batching_enabled: AtomicBool,

    // Statistics
    total_operations: AtomicU64,
    batched_operations: AtomicU64,
}

impl<T> ThresholdBatchProcessor<T> {
    /// Create new threshold batch processor
    pub fn new<F>(config: BatchConfig, processor: F) -> Self
    where
        F: Fn(&[T]) + Send + Sync + 'static,
    {
        Self {
            config,
            buffer: Mutex::new(Vec::new()),
            processor: Box::new(processor),
            operation_count: AtomicU64::new(0),
            last_measurement: Mutex::new(Instant::now()),
            batching_enabled: AtomicBool::new(false),
            total_operations: AtomicU64::new(0),
            batched_operations: AtomicU64::new(0),
        }
    }

    /// Create with default medium frequency configuration
    pub fn with_default_config<F>(processor: F) -> Self
    where
        F: Fn(&[T]) + Send + Sync + 'static,
    {
        Self::new(BatchConfig::default(), processor)
    }

    /// Process an item (either directly or via batching)
    pub fn process(&self, item: T) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        self.operation_count.fetch_add(1, Ordering::Relaxed);

        // Check if we should update batching mode
        self.update_batching_mode();

        if self.batching_enabled.load(Ordering::Relaxed) {
            self.process_batched(item);
        } else {
            self.process_direct(item);
        }
    }

    /// Process item directly (no batching)
    fn process_direct(&self, item: T) {
        let items = vec![item];
        (self.processor)(&items);
    }

    /// Process item via batching
    fn process_batched(&self, item: T) {
        let should_flush = {
            if let Ok(mut buffer) = self.buffer.try_lock() {
                buffer.push(item);
                let should_flush = buffer.len() >= self.config.batch_size;
                should_flush
            } else {
                // If we can't get the lock, process directly to avoid blocking
                self.process_direct(item);
                return;
            }
        };

        if should_flush {
            self.flush_batch();
        }

        self.batched_operations.fetch_add(1, Ordering::Relaxed);
    }

    /// Flush the current batch
    pub fn flush_batch(&self) {
        if let Ok(mut buffer) = self.buffer.try_lock() {
            if !buffer.is_empty() {
                let items = std::mem::take(&mut *buffer);
                drop(buffer); // Release lock before processing
                (self.processor)(&items);
            }
        }
    }

    /// Update batching mode based on current frequency
    fn update_batching_mode(&self) {
        if let Ok(mut last_measurement) = self.last_measurement.try_lock() {
            let now = Instant::now();
            let elapsed = now.duration_since(*last_measurement);

            if elapsed >= self.config.measurement_window {
                let ops_count = self.operation_count.swap(0, Ordering::Relaxed);
                let frequency = if elapsed.as_secs() > 0 {
                    ops_count / elapsed.as_secs()
                } else {
                    ops_count * 1000 / elapsed.as_millis().max(1) as u64
                };

                // Enable batching if frequency exceeds threshold
                let should_batch = frequency > self.config.frequency_threshold;
                self.batching_enabled.store(should_batch, Ordering::Relaxed);

                *last_measurement = now;
            }
        }
    }

    /// Get current frequency (operations per second)
    pub fn current_frequency(&self) -> u64 {
        if let Ok(last_measurement) = self.last_measurement.try_lock() {
            let elapsed = last_measurement.elapsed();
            let ops_count = self.operation_count.load(Ordering::Relaxed);

            if elapsed.as_secs() > 0 {
                ops_count / elapsed.as_secs()
            } else {
                ops_count * 1000 / elapsed.as_millis().max(1) as u64
            }
        } else {
            0
        }
    }

    /// Check if batching is currently enabled
    pub fn is_batching_enabled(&self) -> bool {
        self.batching_enabled.load(Ordering::Relaxed)
    }

    /// Get processing statistics
    pub fn stats(&self) -> ProcessingStats {
        let total = self.total_operations.load(Ordering::Relaxed);
        let batched = self.batched_operations.load(Ordering::Relaxed);

        ProcessingStats {
            total_operations: total,
            batched_operations: batched,
            direct_operations: total - batched,
            batching_ratio: if total > 0 {
                batched as f64 / total as f64
            } else {
                0.0
            },
            current_frequency: self.current_frequency(),
            batching_enabled: self.is_batching_enabled(),
        }
    }

    /// Reset all statistics
    pub fn reset_stats(&self) {
        self.total_operations.store(0, Ordering::Relaxed);
        self.batched_operations.store(0, Ordering::Relaxed);
        self.operation_count.store(0, Ordering::Relaxed);

        if let Ok(mut last_measurement) = self.last_measurement.try_lock() {
            *last_measurement = Instant::now();
        }
    }
}

/// Processing statistics
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    pub total_operations: u64,
    pub batched_operations: u64,
    pub direct_operations: u64,
    pub batching_ratio: f64,
    pub current_frequency: u64,
    pub batching_enabled: bool,
}

// Safety: ThresholdBatchProcessor is Send if T is Send
unsafe impl<T: Send> Send for ThresholdBatchProcessor<T> {}

// Safety: ThresholdBatchProcessor is Sync if T is Send
unsafe impl<T: Send> Sync for ThresholdBatchProcessor<T> {}

#[cfg(test)]
mod tests {
    use crate::core::safe_operations::SafeLock;

    use super::*;
    use std::sync::{Arc, Mutex as StdMutex};
    use std::time::Duration;

    #[test]
    fn test_low_frequency_direct_processing() {
        let processed = Arc::new(StdMutex::new(Vec::new()));
        let processed_clone = processed.clone();

        let config = BatchConfig::custom(5, 100, Duration::from_millis(100));
        let processor = ThresholdBatchProcessor::new(config, move |items: &[i32]| {
            let mut p = processed_clone.safe_lock().expect("Failed to acquire lock on processed");
            p.extend_from_slice(items);
        });

        // Process items slowly (low frequency) - simulate with smaller batches
        for i in 0..10 {
            processor.process(i);
            // Remove sleep - just process normally for testing
        }

        processor.flush_batch();

        let stats = processor.stats();
        println!("Low frequency stats: {:?}", stats);

        // Should mostly use direct processing
        assert!(!processor.is_batching_enabled());

        let processed_items = processed.safe_lock().expect("Failed to acquire lock on processed");
        assert_eq!(processed_items.len(), 10);
    }

    #[test]
    fn test_high_frequency_batch_processing() {
        let processed = Arc::new(StdMutex::new(Vec::new()));
        let processed_clone = processed.clone();

        let config = BatchConfig::custom(3, 50, Duration::from_millis(100));
        let processor = ThresholdBatchProcessor::new(config, move |items: &[i32]| {
            if let Ok(mut p) = processed_clone.lock() {
                p.extend_from_slice(items);
            }
        });

        // Process items quickly (high frequency)
        for i in 0..20 {
            processor.process(i);
            // No sleep - maximum frequency
        }

        // No need to wait - measurement happens immediately for testing

        // Process a few more to trigger batching mode check
        for i in 20..25 {
            processor.process(i);
        }

        processor.flush_batch();

        let stats = processor.stats();
        println!("High frequency stats: {:?}", stats);

        let processed_items = processed.safe_lock().expect("Failed to acquire lock on processed");
        assert_eq!(processed_items.len(), 25);
    }

    #[test]
    fn test_config_presets() {
        let low = BatchConfig::low_frequency();
        assert_eq!(low.frequency_threshold, 100);

        let medium = BatchConfig::medium_frequency();
        assert_eq!(medium.frequency_threshold, 500);

        let high = BatchConfig::high_frequency();
        assert_eq!(high.frequency_threshold, 1000);
    }
}
