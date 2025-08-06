//! Simple mutex implementation with compile-time optimization selection
//! 
//! This module provides a simplified mutex that avoids runtime overhead
//! by using compile-time feature selection.

use std::sync::{Mutex as StdMutex, MutexGuard};

/// Optimized mutex type selected at compile time
#[cfg(feature = "parking-lot")]
pub type OptimizedMutex<T> = parking_lot::Mutex<T>;

#[cfg(not(feature = "parking-lot"))]
pub type OptimizedMutex<T> = StdMutex<T>;

/// Simple mutex wrapper that provides consistent API
pub struct SimpleMutex<T> {
    inner: OptimizedMutex<T>,
    #[cfg(debug_assertions)]
    access_count: std::sync::atomic::AtomicU64,
}

impl<T> SimpleMutex<T> {
    /// Create new simple mutex
    pub fn new(data: T) -> Self {
        Self {
            inner: OptimizedMutex::new(data),
            #[cfg(debug_assertions)]
            access_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Lock the mutex
    #[cfg(feature = "parking-lot")]
    pub fn lock(&self) -> parking_lot::MutexGuard<T> {
        #[cfg(debug_assertions)]
        self.access_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        self.inner.lock()
    }

    /// Lock the mutex
    #[cfg(not(feature = "parking-lot"))]
    pub fn lock(&self) -> Result<MutexGuard<T>, std::sync::PoisonError<MutexGuard<T>>> {
        #[cfg(debug_assertions)]
        self.access_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        self.inner.lock()
    }

    /// Try to lock the mutex
    #[cfg(feature = "parking-lot")]
    pub fn try_lock(&self) -> Option<parking_lot::MutexGuard<T>> {
        #[cfg(debug_assertions)]
        self.access_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        self.inner.try_lock()
    }

    /// Try to lock the mutex
    #[cfg(not(feature = "parking-lot"))]
    pub fn try_lock(&self) -> Result<MutexGuard<T>, std::sync::TryLockError<MutexGuard<T>>> {
        #[cfg(debug_assertions)]
        self.access_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        self.inner.try_lock()
    }

    /// Get access count (debug only)
    #[cfg(debug_assertions)]
    pub fn access_count(&self) -> u64 {
        self.access_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get access count (release mode - always returns 0)
    #[cfg(not(debug_assertions))]
    pub fn access_count(&self) -> u64 {
        0
    }
}

impl<T: Default> Default for SimpleMutex<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

// Safety: SimpleMutex is Send if T is Send
unsafe impl<T: Send> Send for SimpleMutex<T> {}

// Safety: SimpleMutex is Sync if T is Send
unsafe impl<T: Send> Sync for SimpleMutex<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_basic_operations() {
        let mutex = SimpleMutex::new(42);
        
        // Test lock
        #[cfg(feature = "parking-lot")]
        {
            let guard = mutex.lock();
            assert_eq!(*guard, 42);
        }
        
        #[cfg(not(feature = "parking-lot"))]
        {
            let guard = mutex.lock().unwrap();
            assert_eq!(*guard, 42);
        }
        
        // Test try_lock
        #[cfg(feature = "parking-lot")]
        {
            if let Some(guard) = mutex.try_lock() {
                assert_eq!(*guard, 42);
            }
        }
        
        #[cfg(not(feature = "parking-lot"))]
        {
            if let Ok(guard) = mutex.try_lock() {
                assert_eq!(*guard, 42);
            };
        }
    }

    #[test]
    fn test_concurrent_access() {
        let mutex = Arc::new(SimpleMutex::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let mutex_clone = mutex.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    #[cfg(feature = "parking-lot")]
                    {
                        let mut guard = mutex_clone.lock();
                        *guard += 1;
                    }
                    
                    #[cfg(not(feature = "parking-lot"))]
                    {
                        let mut guard = mutex_clone.lock().unwrap();
                        *guard += 1;
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        #[cfg(feature = "parking-lot")]
        {
            let guard = mutex.lock();
            assert_eq!(*guard, 1000);
        }
        
        #[cfg(not(feature = "parking-lot"))]
        {
            let guard = mutex.lock().unwrap();
            assert_eq!(*guard, 1000);
        }

        // Check access count in debug mode
        #[cfg(debug_assertions)]
        {
            assert!(mutex.access_count() > 0);
        }
    }
}