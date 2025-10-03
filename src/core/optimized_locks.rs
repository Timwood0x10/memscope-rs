//! Optimized lock implementations using parking_lot
//!
//! This module provides drop-in replacements for standard library locks
//! using parking_lot for better performance and features.

use crate::core::atomic_stats::AtomicPerformanceCounters;
use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant};

/// Optimized mutex with performance monitoring
#[derive(Debug)]
pub struct OptimizedMutex<T> {
    inner: Mutex<T>,
    counters: AtomicPerformanceCounters,
}

impl<T> OptimizedMutex<T> {
    /// Create a new optimized mutex
    pub fn new(data: T) -> Self {
        Self {
            inner: Mutex::new(data),
            counters: AtomicPerformanceCounters::new(),
        }
    }

    /// Lock the mutex with performance monitoring
    pub fn lock(&self) -> OptimizedMutexGuard<'_, T> {
        let start = Instant::now();

        // Try to acquire the lock without blocking first
        if let Some(guard) = self.inner.try_lock() {
            let wait_time = start.elapsed();
            self.counters.record_lock_acquisition(wait_time);
            return OptimizedMutexGuard {
                guard,
                _counters: &self.counters,
            };
        }

        // If we couldn't acquire immediately, record contention
        self.counters.record_lock_contention();

        // Now block until we can acquire the lock
        let guard = self.inner.lock();
        let wait_time = start.elapsed();
        self.counters.record_lock_acquisition(wait_time);

        OptimizedMutexGuard {
            guard,
            _counters: &self.counters,
        }
    }

    /// Try to lock the mutex without blocking
    pub fn try_lock(&self) -> Option<OptimizedMutexGuard<'_, T>> {
        let start = Instant::now();

        if let Some(guard) = self.inner.try_lock() {
            let wait_time = start.elapsed();
            self.counters.record_lock_acquisition(wait_time);
            Some(OptimizedMutexGuard {
                guard,
                _counters: &self.counters,
            })
        } else {
            self.counters.record_lock_contention();
            None
        }
    }

    /// Try to lock with a timeout
    pub fn try_lock_for(&self, timeout: Duration) -> Option<OptimizedMutexGuard<'_, T>> {
        let start = Instant::now();

        if let Some(guard) = self.inner.try_lock_for(timeout) {
            let wait_time = start.elapsed();
            self.counters.record_lock_acquisition(wait_time);
            Some(OptimizedMutexGuard {
                guard,
                _counters: &self.counters,
            })
        } else {
            self.counters.record_lock_contention();
            None
        }
    }

    /// Get performance statistics for this mutex
    pub fn performance_stats(&self) -> crate::core::atomic_stats::PerformanceSnapshot {
        self.counters.snapshot()
    }
}

/// Guard for optimized mutex
pub struct OptimizedMutexGuard<'a, T> {
    guard: MutexGuard<'a, T>,
    _counters: &'a AtomicPerformanceCounters,
}

impl<'a, T> std::ops::Deref for OptimizedMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T> std::ops::DerefMut for OptimizedMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

/// Optimized RwLock with performance monitoring
#[derive(Debug)]
pub struct OptimizedRwLock<T> {
    inner: RwLock<T>,
    counters: AtomicPerformanceCounters,
}

impl<T> OptimizedRwLock<T> {
    /// Create a new optimized RwLock
    pub fn new(data: T) -> Self {
        Self {
            inner: RwLock::new(data),
            counters: AtomicPerformanceCounters::new(),
        }
    }

    /// Acquire a read lock with performance monitoring
    pub fn read(&self) -> OptimizedRwLockReadGuard<'_, T> {
        let start = Instant::now();

        // Try to acquire the read lock without blocking first
        if let Some(guard) = self.inner.try_read() {
            let wait_time = start.elapsed();
            self.counters.record_lock_acquisition(wait_time);
            return OptimizedRwLockReadGuard {
                guard,
                _counters: &self.counters,
            };
        }

        // If we couldn't acquire immediately, record contention
        self.counters.record_lock_contention();

        // Now block until we can acquire the lock
        let guard = self.inner.read();
        let wait_time = start.elapsed();
        self.counters.record_lock_acquisition(wait_time);

        OptimizedRwLockReadGuard {
            guard,
            _counters: &self.counters,
        }
    }

    /// Acquire a write lock with performance monitoring
    pub fn write(&self) -> OptimizedRwLockWriteGuard<'_, T> {
        let start = Instant::now();

        // Try to acquire the write lock without blocking first
        if let Some(guard) = self.inner.try_write() {
            let wait_time = start.elapsed();
            self.counters.record_lock_acquisition(wait_time);
            return OptimizedRwLockWriteGuard {
                guard,
                _counters: &self.counters,
            };
        }

        // If we couldn't acquire immediately, record contention
        self.counters.record_lock_contention();

        // Now block until we can acquire the lock
        let guard = self.inner.write();
        let wait_time = start.elapsed();
        self.counters.record_lock_acquisition(wait_time);

        OptimizedRwLockWriteGuard {
            guard,
            _counters: &self.counters,
        }
    }

    /// Try to acquire a read lock without blocking
    pub fn try_read(&self) -> Option<OptimizedRwLockReadGuard<'_, T>> {
        let start = Instant::now();

        if let Some(guard) = self.inner.try_read() {
            let wait_time = start.elapsed();
            self.counters.record_lock_acquisition(wait_time);
            Some(OptimizedRwLockReadGuard {
                guard,
                _counters: &self.counters,
            })
        } else {
            self.counters.record_lock_contention();
            None
        }
    }

    /// Try to acquire a write lock without blocking
    pub fn try_write(&self) -> Option<OptimizedRwLockWriteGuard<'_, T>> {
        let start = Instant::now();

        if let Some(guard) = self.inner.try_write() {
            let wait_time = start.elapsed();
            self.counters.record_lock_acquisition(wait_time);
            Some(OptimizedRwLockWriteGuard {
                guard,
                _counters: &self.counters,
            })
        } else {
            self.counters.record_lock_contention();
            None
        }
    }

    /// Get performance statistics for this RwLock
    pub fn performance_stats(&self) -> crate::core::atomic_stats::PerformanceSnapshot {
        self.counters.snapshot()
    }
}

/// Read guard for optimized RwLock
pub struct OptimizedRwLockReadGuard<'a, T> {
    guard: RwLockReadGuard<'a, T>,
    _counters: &'a AtomicPerformanceCounters,
}

impl<'a, T> std::ops::Deref for OptimizedRwLockReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

/// Write guard for optimized RwLock
pub struct OptimizedRwLockWriteGuard<'a, T> {
    guard: RwLockWriteGuard<'a, T>,
    _counters: &'a AtomicPerformanceCounters,
}

impl<'a, T> std::ops::Deref for OptimizedRwLockWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T> std::ops::DerefMut for OptimizedRwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

/// Lock-free counter for simple atomic operations
#[derive(Debug)]
pub struct LockFreeCounter {
    value: std::sync::atomic::AtomicU64,
}

impl LockFreeCounter {
    /// Create a new lock-free counter
    pub fn new(initial_value: u64) -> Self {
        Self {
            value: std::sync::atomic::AtomicU64::new(initial_value),
        }
    }

    /// Increment the counter and return the new value
    pub fn increment(&self) -> u64 {
        let old_value = self.value
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        old_value.wrapping_add(1)
    }

    /// Decrement the counter and return the new value
    pub fn decrement(&self) -> u64 {
        let old_value = self.value
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        old_value.wrapping_sub(1)
    }

    /// Add a value to the counter and return the new value
    pub fn add(&self, value: u64) -> u64 {
        let old_value = self.value
            .fetch_add(value, std::sync::atomic::Ordering::Relaxed);
        old_value.wrapping_add(value)
    }

    /// Subtract a value from the counter and return the new value
    pub fn sub(&self, value: u64) -> u64 {
        let old_value = self.value
            .fetch_sub(value, std::sync::atomic::Ordering::Relaxed);
        old_value.wrapping_sub(value)
    }

    /// Get the current value
    pub fn get(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Set the value
    pub fn set(&self, value: u64) {
        self.value
            .store(value, std::sync::atomic::Ordering::Relaxed);
    }

    /// Compare and swap
    pub fn compare_and_swap(&self, current: u64, new: u64) -> Result<u64, u64> {
        self.value.compare_exchange(
            current,
            new,
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
        )
    }
}

impl Default for LockFreeCounter {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_optimized_mutex_basic_functionality() {
        let mutex = OptimizedMutex::new(42);

        {
            let guard = mutex.lock();
            assert_eq!(*guard, 42);
        }

        // Test that lock is released
        {
            let mut guard = mutex.lock();
            *guard = 100;
        }

        let guard = mutex.lock();
        assert_eq!(*guard, 100);
    }

    #[test]
    fn test_optimized_mutex_try_lock() {
        let mutex = OptimizedMutex::new(42);

        // First try_lock should succeed
        let guard1 = mutex.try_lock();
        assert!(guard1.is_some());
        assert_eq!(*guard1.unwrap(), 42);

        // Second try_lock should fail while first is held
        let _guard1 = mutex.lock(); // Hold the lock
        let guard2 = mutex.try_lock();
        assert!(guard2.is_none());
    }

    #[test]
    fn test_optimized_rwlock_basic_functionality() {
        let rwlock = OptimizedRwLock::new(42);

        // Test read access
        {
            let read_guard = rwlock.read();
            assert_eq!(*read_guard, 42);
        }

        // Test write access
        {
            let mut write_guard = rwlock.write();
            *write_guard = 100;
        }

        // Verify write took effect
        let read_guard = rwlock.read();
        assert_eq!(*read_guard, 100);
    }

    #[test]
    fn test_optimized_rwlock_writer_exclusivity() {
        let rwlock = Arc::new(OptimizedRwLock::new(0));
        let rwlock_clone = Arc::clone(&rwlock);

        // Start a writer thread
        let writer_handle = thread::spawn(move || {
            let mut write_guard = rwlock_clone.write();
            *write_guard = 100;
            thread::sleep(Duration::from_millis(50)); // Hold write lock
            *write_guard = 200;
        });

        // Give writer time to acquire lock
        thread::sleep(Duration::from_millis(10));

        // Try to read - should wait for writer to finish
        let read_guard = rwlock.read();

        // Just verify the lock works - remove timing assertions for CI stability
        assert_eq!(*read_guard, 200);

        writer_handle.join().unwrap();
    }

    #[test]
    fn test_optimized_rwlock_try_operations() {
        let rwlock = OptimizedRwLock::new(42);

        // try_read should succeed when unlocked
        let read_guard = rwlock.try_read();
        assert!(read_guard.is_some());
        if let Some(guard) = read_guard {
            assert_eq!(*guard, 42);
        }

        // try_write should succeed when unlocked
        let write_guard = rwlock.try_write();
        assert!(write_guard.is_some());

        // try_read should fail when write locked
        let try_read = rwlock.try_read();
        assert!(try_read.is_none());

        // try_write should fail when already write locked
        let try_write = rwlock.try_write();
        assert!(try_write.is_none());
    }

    #[test]
    fn test_lock_free_counter_basic_operations() {
        let counter = LockFreeCounter::new(0);

        assert_eq!(counter.get(), 0);

        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.get(), 1);

        assert_eq!(counter.add(5), 6);
        assert_eq!(counter.get(), 6);

        assert_eq!(counter.decrement(), 5);
        assert_eq!(counter.get(), 5);

        assert_eq!(counter.sub(3), 2);
        assert_eq!(counter.get(), 2);
    }

    #[test]
    fn test_lock_free_counter_concurrent_increments() {
        let counter = Arc::new(LockFreeCounter::new(0));
        let mut handles = Vec::new();

        // Spawn multiple threads that increment
        for _ in 0..10 {
            let counter_clone = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    counter_clone.increment();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Should have exactly 10,000 increments
        assert_eq!(counter.get(), 10000);
    }

    #[test]
    fn test_lock_free_counter_mixed_operations() {
        let counter = Arc::new(LockFreeCounter::new(0));
        let mut handles = Vec::new();

        // Incrementing threads
        for _ in 0..5 {
            let counter_clone = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    counter_clone.increment();
                }
            });
            handles.push(handle);
        }

        // Decrementing threads
        for _ in 0..3 {
            let counter_clone = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..500 {
                    counter_clone.decrement();
                }
            });
            handles.push(handle);
        }

        // Adding threads
        for _ in 0..2 {
            let counter_clone = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    counter_clone.add(10);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Expected: 5*1000 - 3*500 + 2*100*10 = 5000 - 1500 + 2000 = 5500
        assert_eq!(counter.get(), 5500);
    }

    #[test]
    fn test_lock_free_counter_compare_and_swap() {
        let counter = LockFreeCounter::new(42);

        // Successful compare and swap
        let result = counter.compare_and_swap(42, 100);
        assert_eq!(result, Ok(42));
        assert_eq!(counter.get(), 100);

        // Failed compare and swap
        let result = counter.compare_and_swap(42, 200);
        assert_eq!(result, Err(100)); // Returns current value
        assert_eq!(counter.get(), 100); // Value unchanged
    }

    #[test]
    fn test_lock_free_counter_set_operation() {
        let counter = LockFreeCounter::new(0);

        counter.set(12345);
        assert_eq!(counter.get(), 12345);

        // Should work with large values
        counter.set(u64::MAX - 1);
        assert_eq!(counter.get(), u64::MAX - 1);
    }

    #[test]
    fn test_lock_free_counter_default() {
        let counter = LockFreeCounter::default();
        assert_eq!(counter.get(), 0);

        counter.increment();
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_lock_contention_performance() {
        let mutex = Arc::new(OptimizedMutex::new(0));
        let mut handles = Vec::new();

        let start = std::time::Instant::now();

        // Create high contention scenario
        for _ in 0..20 {
            let mutex_clone = Arc::clone(&mutex);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let mut guard = mutex_clone.lock();
                    *guard += 1;
                    // Simulate minimal work while holding the lock
                    std::hint::spin_loop();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let duration = start.elapsed();

        // Should handle contention correctly and efficiently
        assert_eq!(*mutex.lock(), 2000);
        assert!(duration < Duration::from_secs(1)); // Should complete reasonably fast
    }

    #[test]
    fn test_rwlock_reader_writer_fairness() {
        let rwlock = Arc::new(OptimizedRwLock::new(0));
        let rwlock_clone = Arc::clone(&rwlock);

        // Start multiple readers
        let mut reader_handles = Vec::new();
        for i in 0..5 {
            let rwlock_clone = Arc::clone(&rwlock);
            let handle = thread::spawn(move || {
                thread::sleep(Duration::from_millis(i * 10)); // Stagger start times
                let _guard = rwlock_clone.read();
                thread::sleep(Duration::from_millis(50)); // Hold read lock
            });
            reader_handles.push(handle);
        }

        // Start a writer after readers have started
        thread::sleep(Duration::from_millis(20));
        let writer_handle = thread::spawn(move || {
            let mut guard = rwlock_clone.write();
            *guard = 42;
        });

        // Wait for all to complete
        for handle in reader_handles {
            handle.join().unwrap();
        }
        writer_handle.join().unwrap();

        // Writer should have eventually acquired the lock
        let guard = rwlock.read();
        assert_eq!(*guard, 42);
    }
}
