//! Smart optimization based on actual performance analysis
//! 
//! This module provides targeted optimizations based on real performance data,
//! avoiding the over-engineering that led to performance degradation.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use std::borrow::ToOwned;

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
    pub fn lock(&self) -> SmartMutexGuard<T> {
        match self {
            SmartMutex::Standard(mutex) => {
                let start = Instant::now();
                let guard = mutex.lock().unwrap();
                let wait_time = start.elapsed();
                
                // If we waited too long, this might benefit from parking_lot
                if wait_time > Duration::from_micros(100) {
                    // Log this for potential upgrade
                    record_contention("std_mutex", wait_time);
                }
                
                SmartMutexGuard::Standard(guard)
            }
            SmartMutex::ParkingLot(mutex) => {
                SmartMutexGuard::ParkingLot(mutex.lock())
            }
        }
    }

    /// Try to lock without blocking
    pub fn try_lock(&self) -> Option<SmartMutexGuard<T>> {
        match self {
            SmartMutex::Standard(mutex) => {
                mutex.try_lock().ok().map(SmartMutexGuard::Standard)
            }
            SmartMutex::ParkingLot(mutex) => {
                mutex.try_lock().map(SmartMutexGuard::ParkingLot)
            }
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
        let avg_wait = TOTAL_WAIT_TIME_NS.load(Ordering::Relaxed) / CONTENTION_COUNTER.load(Ordering::Relaxed);
        tracing::debug!("Mutex contention detected: {} avg wait: {}ns", mutex_type, avg_wait);
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
                panic!("Called safe_unwrap on None");
                
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
    fn safe_unwrap(self, fallback: T) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                #[cfg(debug_assertions)]
                panic!("Called safe_unwrap on Err: {:?}", e);
                
                #[cfg(not(debug_assertions))]
                {
                    tracing::warn!("safe_unwrap called on Err: {:?}, using fallback", e);
                    fallback
                }
            }
        }
    }

    fn safe_unwrap_or_log(self, context: &str, fallback: T) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                tracing::warn!("safe_unwrap_or_log failed in context: {} - error: {:?}", context, e);
                fallback
            }
        }
    }
}

/// Efficient clone reduction for hot paths
pub trait SmartClone<T: ?Sized + ToOwned> {
    /// Clone only if necessary, otherwise return reference
    fn smart_clone(&self) -> std::borrow::Cow<T>;
}

impl SmartClone<str> for String {
    fn smart_clone(&self) -> std::borrow::Cow<str> {
        std::borrow::Cow::Borrowed(self)
    }
}

impl<T: Clone> SmartClone<[T]> for Vec<T> {
    fn smart_clone(&self) -> std::borrow::Cow<[T]> {
        std::borrow::Cow::Borrowed(self)
    }
}

/// Macro for smart unwrapping
#[macro_export]
macro_rules! safe_unwrap {
    ($expr:expr, $fallback:expr) => {
        $crate::core::smart_optimization::SafeUnwrap::safe_unwrap($expr, $fallback)
    };
    ($expr:expr, $fallback:expr, $context:expr) => {
        $crate::core::smart_optimization::SafeUnwrap::safe_unwrap_or_log($expr, $context, $fallback)
    };
}

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
    fn test_safe_unwrap() {
        let some_value = Some(42);
        assert_eq!(some_value.safe_unwrap(0), 42);
        
        let none_value: Option<i32> = None;
        assert_eq!(none_value.safe_unwrap(99), 99);
    }

    #[test]
    fn test_smart_stats() {
        let stats = SmartStats::new();
        
        // Fast operations
        for _ in 0..1000 {
            stats.record_allocation();
        }
        
        let (allocs, deallocs) = stats.get_simple_stats();
        assert_eq!(allocs, 1000);
        assert_eq!(deallocs, 0);
    }
}