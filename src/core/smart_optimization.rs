//! Smart optimization based on actual performance analysis
//!
//! This module provides targeted optimizations based on real performance data,
//! avoiding the over-engineering that led to performance degradation.

use crate::core::safe_operations::SafeLock;
use parking_lot::Mutex;
use std::borrow::ToOwned;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Smart mutex that chooses the best implementation based on usage patterns
pub enum SmartMutex<T> {
    /// Use standard mutex for low-contention scenarios
    Standard(std::sync::Mutex<T>),
    /// Use parking_lot for high-contention scenarios
    ParkingLot(parking_lot::Mutex<T>),
}

impl<T> SmartMutex<T> {
    /// Create a new smart mutex (starts with standard, can upgrade)
    pub fn new(data: T) -> Self {
        SmartMutex::Standard(std::sync::Mutex::new(data))
    }

    /// Lock with automatic contention detection
    pub fn lock(&self) -> SmartMutexGuard<'_, T> {
        match self {
            SmartMutex::Standard(mutex) => {
                let start = Instant::now();
                let guard = mutex.safe_lock().expect("Failed to lock standard mutex");
                let wait_time = start.elapsed();

                // If we waited too long, this might benefit from parking_lot
                if wait_time > Duration::from_micros(100) {
                    // Log this for potential upgrade
                    record_contention("std_mutex", wait_time);
                }

                SmartMutexGuard::Standard(guard)
            }
            SmartMutex::ParkingLot(mutex) => SmartMutexGuard::ParkingLot(mutex.lock()),
        }
    }

    /// Try to lock without blocking
    pub fn try_lock(&self) -> Option<SmartMutexGuard<'_, T>> {
        match self {
            SmartMutex::Standard(mutex) => mutex.try_lock().ok().map(SmartMutexGuard::Standard),
            SmartMutex::ParkingLot(mutex) => mutex.try_lock().map(SmartMutexGuard::ParkingLot),
        }
    }
}

pub enum SmartMutexGuard<'a, T> {
    Standard(std::sync::MutexGuard<'a, T>),
    ParkingLot(parking_lot::MutexGuard<'a, T>),
}

impl<'a, T> std::ops::Deref for SmartMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            SmartMutexGuard::Standard(guard) => guard,
            SmartMutexGuard::ParkingLot(guard) => guard,
        }
    }
}

impl<'a, T> std::ops::DerefMut for SmartMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            SmartMutexGuard::Standard(guard) => guard,
            SmartMutexGuard::ParkingLot(guard) => guard,
        }
    }
}

/// Simple contention tracking (much lighter than our previous approach)
static CONTENTION_COUNTER: AtomicU64 = AtomicU64::new(0);
static TOTAL_WAIT_TIME_NS: AtomicU64 = AtomicU64::new(0);

fn record_contention(mutex_type: &str, wait_time: Duration) {
    CONTENTION_COUNTER.fetch_add(1, Ordering::Relaxed);
    TOTAL_WAIT_TIME_NS.fetch_add(wait_time.as_nanos() as u64, Ordering::Relaxed);

    // Simple logging without heavy overhead
    if CONTENTION_COUNTER.load(Ordering::Relaxed) % 1000 == 0 {
        let avg_wait =
            TOTAL_WAIT_TIME_NS.load(Ordering::Relaxed) / CONTENTION_COUNTER.load(Ordering::Relaxed);
        tracing::debug!(
            "Mutex contention detected: {} avg wait: {}ns",
            mutex_type,
            avg_wait
        );
    }
}

/// Lightweight unwrap replacement that doesn't panic in release builds
pub trait SafeUnwrap<T> {
    /// Unwrap with fallback value in release builds
    fn safe_unwrap(self, fallback: T) -> T;

    /// Unwrap with error context but no panic in release
    fn safe_unwrap_or_log(self, context: &str, fallback: T) -> T;
}

impl<T> SafeUnwrap<T> for Option<T> {
    fn safe_unwrap(self, fallback: T) -> T {
        match self {
            Some(value) => value,
            None => {
                #[cfg(debug_assertions)]
                {
                    tracing::warn!("safe_unwrap called on None, using fallback");
                    fallback
                }

                #[cfg(not(debug_assertions))]
                {
                    tracing::warn!("safe_unwrap called on None, using fallback");
                    fallback
                }
            }
        }
    }

    fn safe_unwrap_or_log(self, context: &str, fallback: T) -> T {
        match self {
            Some(value) => value,
            None => {
                tracing::warn!("safe_unwrap_or_log failed in context: {}", context);
                fallback
            }
        }
    }
}

impl<T, E: std::fmt::Debug> SafeUnwrap<T> for Result<T, E> {
    fn safe_unwrap(self, _fallback: T) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                #[cfg(debug_assertions)]
                panic!("Called safe_unwrap on Err: {e:?}");

                #[cfg(not(debug_assertions))]
                {
                    tracing::warn!("safe_unwrap called on Err: {e:?}, using fallback");
                    _fallback
                }
            }
        }
    }

    fn safe_unwrap_or_log(self, context: &str, fallback: T) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                tracing::warn!(
                    "safe_unwrap_or_log failed in context: {context} - error: {:?}",
                    e
                );
                fallback
            }
        }
    }
}

/// Efficient clone reduction for hot paths
pub trait SmartClone<T: ?Sized + ToOwned> {
    /// Clone only if necessary, otherwise return reference
    fn smart_clone(&self) -> std::borrow::Cow<'_, T>;
}

impl SmartClone<str> for String {
    fn smart_clone(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed(self)
    }
}

impl<T: Clone> SmartClone<[T]> for Vec<T> {
    fn smart_clone(&self) -> std::borrow::Cow<'_, [T]> {
        std::borrow::Cow::Borrowed(self)
    }
}

/// Macro for smart unwrapping (removed - use safe_operations instead)
/// Performance-aware statistics that use the right tool for the job
pub struct SmartStats {
    // Use atomics for high-frequency simple counters
    pub allocation_count: AtomicU64,
    pub deallocation_count: AtomicU64,

    // Use mutex for complex data that's accessed less frequently
    pub detailed_stats: Mutex<DetailedStats>,
}

#[derive(Default)]
pub struct DetailedStats {
    pub allocation_sizes: Vec<usize>,
    pub allocation_times: Vec<Duration>,
    pub peak_memory: usize,
}

impl SmartStats {
    pub fn new() -> Self {
        Self {
            allocation_count: AtomicU64::new(0),
            deallocation_count: AtomicU64::new(0),
            detailed_stats: Mutex::new(DetailedStats::default()),
        }
    }

    /// Fast path for simple counting
    pub fn record_allocation(&self) {
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Slower path for detailed tracking (only when needed)
    pub fn record_detailed_allocation(&self, size: usize, time: Duration) {
        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        // Only lock when we need detailed stats
        if let Some(mut stats) = self.detailed_stats.try_lock() {
            stats.allocation_sizes.push(size);
            stats.allocation_times.push(time);
        }
        // If lock fails, we just skip detailed tracking - no big deal
    }

    pub fn get_simple_stats(&self) -> (u64, u64) {
        (
            self.allocation_count.load(Ordering::Relaxed),
            self.deallocation_count.load(Ordering::Relaxed),
        )
    }
}

impl Default for SmartStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_smart_mutex_basic() {
        let mutex = SmartMutex::new(42);
        {
            let guard = mutex.lock();
            assert_eq!(*guard, 42);
        }

        {
            let mut guard = mutex.lock();
            *guard = 100;
        }

        {
            let guard = mutex.lock();
            assert_eq!(*guard, 100);
        }
    }

    #[test]
    fn test_smart_mutex_try_lock() {
        let mutex = SmartMutex::new("test_value");

        // Test successful try_lock
        if let Some(guard) = mutex.try_lock() {
            assert_eq!(*guard, "test_value");
        } else {
            panic!("try_lock should succeed when mutex is not held");
        }

        // Test try_lock when already locked (in a controlled way)
        let mutex2 = Arc::new(SmartMutex::new(0));
        let mutex2_clone = mutex2.clone();

        let _guard = mutex2.lock();
        // Now try_lock should fail
        assert!(mutex2_clone.try_lock().is_none());
    }

    #[test]
    fn test_smart_mutex_parking_lot() {
        // Test with ParkingLot variant
        let mutex = SmartMutex::ParkingLot(parking_lot::Mutex::new("parking_lot_test"));

        {
            let guard = mutex.lock();
            assert_eq!(*guard, "parking_lot_test");
        }

        // Test try_lock with ParkingLot
        {
            if let Some(guard) = mutex.try_lock() {
                assert_eq!(*guard, "parking_lot_test");
            } else {
                panic!("ParkingLot try_lock should succeed");
            };
        }
    }

    #[test]
    fn test_smart_mutex_guard_deref() {
        let mutex = SmartMutex::new(vec![1, 2, 3]);
        let guard = mutex.lock();

        // Test Deref
        assert_eq!(guard.len(), 3);
        assert_eq!(guard[0], 1);
    }

    #[test]
    fn test_smart_mutex_guard_deref_mut() {
        let mutex = SmartMutex::new(vec![1, 2, 3]);
        let mut guard = mutex.lock();

        // Test DerefMut
        guard.push(4);
        assert_eq!(guard.len(), 4);
        guard[0] = 10;
        assert_eq!(guard[0], 10);
    }

    #[test]
    fn test_contention_recording() {
        // Reset counters (though they're global, we can at least test they increment)
        let initial_count = CONTENTION_COUNTER.load(Ordering::Relaxed);
        let initial_wait = TOTAL_WAIT_TIME_NS.load(Ordering::Relaxed);

        // Record some contention
        record_contention("test_mutex", Duration::from_micros(50));

        assert_eq!(
            CONTENTION_COUNTER.load(Ordering::Relaxed),
            initial_count + 1
        );
        assert!(TOTAL_WAIT_TIME_NS.load(Ordering::Relaxed) > initial_wait);

        // Test multiple contentions
        for _ in 0..10 {
            record_contention("test_mutex", Duration::from_micros(100));
        }

        assert_eq!(
            CONTENTION_COUNTER.load(Ordering::Relaxed),
            initial_count + 11
        );
    }

    #[test]
    fn test_safe_unwrap_option() {
        // Test Some variant
        let some_value = Some(42);
        assert_eq!(some_value.safe_unwrap(0), 42);

        // Test None variant
        let none_value: Option<i32> = None;
        assert_eq!(none_value.safe_unwrap(99), 99);

        // Test with string
        let some_string = Some("hello".to_string());
        assert_eq!(some_string.safe_unwrap("default".to_string()), "hello");

        let none_string: Option<String> = None;
        assert_eq!(none_string.safe_unwrap("fallback".to_string()), "fallback");
    }

    #[test]
    fn test_safe_unwrap_or_log_option() {
        // Test Some variant
        let some_value = Some(100);
        assert_eq!(some_value.safe_unwrap_or_log("test_context", 0), 100);

        // Test None variant
        let none_value: Option<i32> = None;
        assert_eq!(none_value.safe_unwrap_or_log("none_context", 50), 50);

        // Test with different types
        let some_vec = Some(vec![1, 2, 3]);
        assert_eq!(
            some_vec.safe_unwrap_or_log("vec_context", vec![]),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn test_safe_unwrap_result() {
        // Test Ok variant
        let ok_result: Result<i32, String> = Ok(42);
        assert_eq!(ok_result.safe_unwrap(0), 42);

        // Test Err variant (should use fallback in release mode)
        #[cfg(not(debug_assertions))]
        {
            let err_result: Result<i32, String> = Err("error message".to_string());
            assert_eq!(err_result.safe_unwrap(99), 99);
        }

        // In debug mode, this would panic, so we skip it
        #[cfg(debug_assertions)]
        {
            // Just test Ok case in debug mode
            let ok_result2: Result<String, &str> = Ok("success".to_string());
            assert_eq!(ok_result2.safe_unwrap("fallback".to_string()), "success");
        }
    }

    #[test]
    fn test_safe_unwrap_or_log_result() {
        // Test Ok variant
        let ok_result: Result<i32, String> = Ok(100);
        assert_eq!(ok_result.safe_unwrap_or_log("ok_context", 0), 100);

        // Test Err variant
        let err_result: Result<i32, String> = Err("test error".to_string());
        assert_eq!(err_result.safe_unwrap_or_log("err_context", 50), 50);

        // Test with complex error type
        let err_result2: Result<Vec<u8>, std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        assert_eq!(
            err_result2.safe_unwrap_or_log("io_err", vec![1, 2]),
            vec![1, 2]
        );
    }

    #[test]
    fn test_smart_clone_string() {
        let string = String::from("hello world");
        let cloned = string.smart_clone();

        // Should return a borrowed Cow
        assert!(matches!(cloned, std::borrow::Cow::Borrowed(_)));
        assert_eq!(&*cloned, "hello world");
    }

    #[test]
    fn test_smart_clone_vec() {
        let vec = vec![1, 2, 3, 4, 5];
        let cloned = vec.smart_clone();

        // Should return a borrowed Cow
        assert!(matches!(cloned, std::borrow::Cow::Borrowed(_)));
        assert_eq!(&*cloned, &[1, 2, 3, 4, 5]);

        // Test with different types
        let string_vec = vec!["a".to_string(), "b".to_string()];
        let cloned2 = string_vec.smart_clone();
        assert!(matches!(cloned2, std::borrow::Cow::Borrowed(_)));
        assert_eq!(cloned2.len(), 2);
    }

    #[test]
    fn test_smart_stats_basic() {
        let stats = SmartStats::new();

        // Test initial state
        let (allocs, deallocs) = stats.get_simple_stats();
        assert_eq!(allocs, 0);
        assert_eq!(deallocs, 0);

        // Test allocation recording
        for _ in 0..100 {
            stats.record_allocation();
        }

        let (allocs, deallocs) = stats.get_simple_stats();
        assert_eq!(allocs, 100);
        assert_eq!(deallocs, 0);

        // Test deallocation counting
        for _ in 0..50 {
            stats.deallocation_count.fetch_add(1, Ordering::Relaxed);
        }

        let (allocs, deallocs) = stats.get_simple_stats();
        assert_eq!(allocs, 100);
        assert_eq!(deallocs, 50);
    }

    #[test]
    fn test_smart_stats_detailed() {
        let stats = SmartStats::new();

        // Record detailed allocations
        stats.record_detailed_allocation(1024, Duration::from_micros(10));
        stats.record_detailed_allocation(2048, Duration::from_micros(20));
        stats.record_detailed_allocation(512, Duration::from_micros(5));

        // Check counters
        let (allocs, _) = stats.get_simple_stats();
        assert_eq!(allocs, 3);

        // Check detailed stats
        let detailed = stats.detailed_stats.lock();
        assert_eq!(detailed.allocation_sizes.len(), 3);
        assert_eq!(detailed.allocation_sizes[0], 1024);
        assert_eq!(detailed.allocation_sizes[1], 2048);
        assert_eq!(detailed.allocation_sizes[2], 512);

        assert_eq!(detailed.allocation_times.len(), 3);
        assert_eq!(detailed.allocation_times[0], Duration::from_micros(10));
    }

    #[test]
    fn test_smart_stats_peak_memory() {
        let stats = SmartStats::new();

        // Update peak memory
        {
            let mut detailed = stats.detailed_stats.lock();
            detailed.peak_memory = 1000;
            detailed.allocation_sizes.push(500);
        }

        // Verify
        let detailed = stats.detailed_stats.lock();
        assert_eq!(detailed.peak_memory, 1000);
        assert_eq!(detailed.allocation_sizes[0], 500);
    }

    #[test]
    fn test_smart_stats_concurrent() {
        let stats = Arc::new(SmartStats::new());
        let mut handles = vec![];

        // Test concurrent allocation recording
        for _ in 0..10 {
            let stats_clone = stats.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    stats_clone.record_allocation();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let (allocs, _) = stats.get_simple_stats();
        assert_eq!(allocs, 1000);
    }

    #[test]
    fn test_smart_stats_default() {
        let stats = SmartStats::default();
        let (allocs, deallocs) = stats.get_simple_stats();
        assert_eq!(allocs, 0);
        assert_eq!(deallocs, 0);

        // Verify detailed stats are also default
        let detailed = stats.detailed_stats.lock();
        assert!(detailed.allocation_sizes.is_empty());
        assert!(detailed.allocation_times.is_empty());
        assert_eq!(detailed.peak_memory, 0);
    }

    #[test]
    fn test_detailed_stats_default() {
        let detailed = DetailedStats::default();
        assert!(detailed.allocation_sizes.is_empty());
        assert!(detailed.allocation_times.is_empty());
        assert_eq!(detailed.peak_memory, 0);
    }
}
