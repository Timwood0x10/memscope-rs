//! Optimized lock implementations using parking_lot
//! 
//! This module provides drop-in replacements for standard library locks
//! using parking_lot for better performance and features.

use parking_lot::{Mutex, RwLock, MutexGuard, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant};
use crate::core::atomic_stats::AtomicPerformanceCounters;

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
    pub fn lock(&self) -> OptimizedMutexGuard<T> {
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
    pub fn try_lock(&self) -> Option<OptimizedMutexGuard<T>> {
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
    pub fn try_lock_for(&self, timeout: Duration) -> Option<OptimizedMutexGuard<T>> {
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
    pub fn read(&self) -> OptimizedRwLockReadGuard<T> {
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
    pub fn write(&self) -> OptimizedRwLockWriteGuard<T> {
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
    pub fn try_read(&self) -> Option<OptimizedRwLockReadGuard<T>> {
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
    pub fn try_write(&self) -> Option<OptimizedRwLockWriteGuard<T>> {
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
        self.value.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }

    /// Decrement the counter and return the new value
    pub fn decrement(&self) -> u64 {
        self.value.fetch_sub(1, std::sync::atomic::Ordering::Relaxed) - 1
    }

    /// Add a value to the counter and return the new value
    pub fn add(&self, value: u64) -> u64 {
        self.value.fetch_add(value, std::sync::atomic::Ordering::Relaxed) + value
    }

    /// Subtract a value from the counter and return the new value
    pub fn sub(&self, value: u64) -> u64 {
        self.value.fetch_sub(value, std::sync::atomic::Ordering::Relaxed) - value
    }

    /// Get the current value
    pub fn get(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Set the value
    pub fn set(&self, value: u64) {
        self.value.store(value, std::sync::atomic::Ordering::Relaxed);
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