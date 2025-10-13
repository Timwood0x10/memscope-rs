//! Performance monitoring and metrics collection
//!
//! This module provides tools for monitoring the performance of the memory tracking
//! system itself, helping to identify bottlenecks and ensure optimal performance.

pub mod metrics;
pub mod monitor;
pub mod profiler;

pub use metrics::{MetricType, MetricValue, PerformanceMetrics};
pub use monitor::{Alert, AlertLevel, MonitorConfig, PerformanceMonitor};
pub use profiler::{ProfileResult, Profiler, ProfilerConfig};

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Global performance counter for quick metrics
pub struct GlobalCounter {
    allocations_tracked: AtomicU64,
    deallocations_tracked: AtomicU64,
    classifications_performed: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    contention_events: AtomicU64,
    start_time: Instant,
}

impl GlobalCounter {
    fn new() -> Self {
        Self {
            allocations_tracked: AtomicU64::new(0),
            deallocations_tracked: AtomicU64::new(0),
            classifications_performed: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            contention_events: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    /// Record an allocation tracking event
    pub fn record_allocation(&self) {
        self.allocations_tracked.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a deallocation tracking event
    pub fn record_deallocation(&self) {
        self.deallocations_tracked.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a type classification event
    pub fn record_classification(&self) {
        self.classifications_performed
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a lock contention event
    pub fn record_contention(&self) {
        self.contention_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current statistics
    pub fn get_stats(&self) -> GlobalStats {
        let now = Instant::now();
        let uptime = now.duration_since(self.start_time);

        GlobalStats {
            allocations_tracked: self.allocations_tracked.load(Ordering::Relaxed),
            deallocations_tracked: self.deallocations_tracked.load(Ordering::Relaxed),
            classifications_performed: self.classifications_performed.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            contention_events: self.contention_events.load(Ordering::Relaxed),
            uptime,
        }
    }

    /// Reset all counters
    pub fn reset(&self) {
        self.allocations_tracked.store(0, Ordering::Relaxed);
        self.deallocations_tracked.store(0, Ordering::Relaxed);
        self.classifications_performed.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.contention_events.store(0, Ordering::Relaxed);
    }
}

/// Global statistics snapshot
#[derive(Debug, Clone)]
pub struct GlobalStats {
    pub allocations_tracked: u64,
    pub deallocations_tracked: u64,
    pub classifications_performed: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub contention_events: u64,
    pub uptime: Duration,
}

impl GlobalStats {
    /// Calculate cache hit ratio
    pub fn cache_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// Calculate tracking rate (events per second)
    pub fn tracking_rate(&self) -> f64 {
        let total_events = self.allocations_tracked + self.deallocations_tracked;
        let seconds = self.uptime.as_secs_f64();
        if seconds == 0.0 {
            0.0
        } else {
            total_events as f64 / seconds
        }
    }

    /// Calculate classification rate (classifications per second)
    pub fn classification_rate(&self) -> f64 {
        let seconds = self.uptime.as_secs_f64();
        if seconds == 0.0 {
            0.0
        } else {
            self.classifications_performed as f64 / seconds
        }
    }

    /// Calculate contention ratio
    pub fn contention_ratio(&self) -> f64 {
        let total_tracking = self.allocations_tracked + self.deallocations_tracked;
        if total_tracking == 0 {
            0.0
        } else {
            self.contention_events as f64 / total_tracking as f64
        }
    }

    /// Generate a performance report
    pub fn generate_report(&self) -> String {
        format!(
            "Performance Statistics:\n\
             Runtime: {:.2?}\n\
             Allocations tracked: {}\n\
             Deallocations tracked: {}\n\
             Classifications performed: {}\n\
             Cache hit ratio: {:.1}%\n\
             Tracking rate: {:.1} events/sec\n\
             Classification rate: {:.1} classifications/sec\n\
             Contention ratio: {:.2}%\n\
             Performance status: {}",
            self.uptime,
            self.allocations_tracked,
            self.deallocations_tracked,
            self.classifications_performed,
            self.cache_hit_ratio() * 100.0,
            self.tracking_rate(),
            self.classification_rate(),
            self.contention_ratio() * 100.0,
            self.performance_status()
        )
    }

    /// Get overall performance status
    pub fn performance_status(&self) -> &str {
        let cache_ratio = self.cache_hit_ratio();
        let contention_ratio = self.contention_ratio();

        if cache_ratio >= 0.9 && contention_ratio < 0.05 {
            "Excellent"
        } else if cache_ratio >= 0.8 && contention_ratio < 0.1 {
            "Good"
        } else if cache_ratio >= 0.6 && contention_ratio < 0.2 {
            "Fair"
        } else {
            "Needs Attention"
        }
    }
}

/// Global performance counter instance
static GLOBAL_COUNTER: std::sync::OnceLock<GlobalCounter> = std::sync::OnceLock::new();

/// Get the global performance counter
pub fn global_counter() -> &'static GlobalCounter {
    GLOBAL_COUNTER.get_or_init(GlobalCounter::new)
}

/// Convenience function to record allocation
pub fn record_allocation() {
    global_counter().record_allocation();
}

/// Convenience function to record deallocation
pub fn record_deallocation() {
    global_counter().record_deallocation();
}

/// Convenience function to record classification
pub fn record_classification() {
    global_counter().record_classification();
}

/// Convenience function to record cache hit
pub fn record_cache_hit() {
    global_counter().record_cache_hit();
}

/// Convenience function to record cache miss
pub fn record_cache_miss() {
    global_counter().record_cache_miss();
}

/// Convenience function to record contention
pub fn record_contention() {
    global_counter().record_contention();
}

/// Get current global statistics
pub fn get_global_stats() -> GlobalStats {
    global_counter().get_stats()
}

/// Simple timer for measuring operation duration
pub struct Timer {
    start: Instant,
    name: String,
}

impl Timer {
    pub fn new(name: &str) -> Self {
        Self {
            start: Instant::now(),
            name: name.to_string(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = self.elapsed_ms();
        if elapsed > 10.0 {
            // Only log operations that take more than 10ms
            tracing::debug!("Operation '{}' took {:.2}ms", self.name, elapsed);
        }
    }
}

/// Macro for timing operations
#[macro_export]
macro_rules! time_operation {
    ($name:expr, $block:block) => {{
        let _timer = $crate::performance::Timer::new($name);
        $block
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_global_counter() {
        let counter = GlobalCounter::new();

        counter.record_allocation();
        counter.record_deallocation();
        counter.record_classification();
        counter.record_cache_hit();
        counter.record_cache_miss();
        counter.record_contention();

        let stats = counter.get_stats();
        assert_eq!(stats.allocations_tracked, 1);
        assert_eq!(stats.deallocations_tracked, 1);
        assert_eq!(stats.classifications_performed, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.contention_events, 1);
    }

    #[test]
    fn test_cache_hit_ratio() {
        let counter = GlobalCounter::new();

        // Record 3 hits and 1 miss
        counter.record_cache_hit();
        counter.record_cache_hit();
        counter.record_cache_hit();
        counter.record_cache_miss();

        let stats = counter.get_stats();
        assert_eq!(stats.cache_hit_ratio(), 0.75);
    }

    #[test]
    fn test_timer() {
        let timer = Timer::new("test_operation");
        thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed() >= Duration::from_millis(10));
        assert!(timer.elapsed_ms() >= 10.0);
    }

    #[test]
    fn test_global_functions() {
        // Reset counters first
        global_counter().reset();

        record_allocation();
        record_deallocation();
        record_classification();
        record_cache_hit();
        record_cache_miss();
        record_contention();

        let stats = get_global_stats();
        assert_eq!(stats.allocations_tracked, 1);
        assert_eq!(stats.deallocations_tracked, 1);
        assert_eq!(stats.classifications_performed, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.contention_events, 1);
    }

    #[test]
    fn test_performance_status() {
        let mut stats = GlobalStats {
            allocations_tracked: 1000,
            deallocations_tracked: 1000,
            classifications_performed: 500,
            cache_hits: 450, // 90% hit ratio
            cache_misses: 50,
            contention_events: 20, // 1% contention
            uptime: Duration::from_secs(60),
        };

        assert_eq!(stats.performance_status(), "Excellent");

        stats.cache_hits = 400; // 80% hit ratio
        stats.contention_events = 150; // 7.5% contention
        assert_eq!(stats.performance_status(), "Good");

        stats.cache_hits = 300; // 60% hit ratio
        stats.contention_events = 300; // 15% contention
        assert_eq!(stats.performance_status(), "Fair");

        stats.cache_hits = 200; // 40% hit ratio
        stats.contention_events = 500; // 25% contention
        assert_eq!(stats.performance_status(), "Needs Attention");
    }
}
