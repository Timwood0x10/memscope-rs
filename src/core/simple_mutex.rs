//! Simple mutex implementation with compile-time optimization selection
//!
//! This module provides a simplified mutex that avoids runtime overhead
//! by using compile-time feature selection.

/// Optimized mutex type selected at compile time
#[cfg(feature = "parking-lot")]
pub type OptimizedMutex<T> = parking_lot::Mutex<T>;

#[cfg(not(feature = "parking-lot"))]
pub type OptimizedMutex<T> = std::sync::Mutex<T>;

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
    pub fn lock(&self) -> parking_lot::MutexGuard<'_, T> {
        #[cfg(debug_assertions)]
        self.access_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        self.inner.lock()
    }

    /// Lock the mutex
    #[cfg(not(feature = "parking-lot"))]
    pub fn lock(
        &self,
    ) -> Result<std::sync::MutexGuard<T>, std::sync::PoisonError<std::sync::MutexGuard<T>>> {
        #[cfg(debug_assertions)]
        self.access_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        self.inner.lock()
    }

    /// Try to lock the mutex
    #[cfg(feature = "parking-lot")]
    pub fn try_lock(&self) -> Option<parking_lot::MutexGuard<'_, T>> {
        #[cfg(debug_assertions)]
        self.access_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        self.inner.try_lock()
    }

    /// Try to lock the mutex
    #[cfg(not(feature = "parking-lot"))]
    pub fn try_lock(
        &self,
    ) -> Result<std::sync::MutexGuard<T>, std::sync::TryLockError<std::sync::MutexGuard<T>>> {
        #[cfg(debug_assertions)]
        self.access_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

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

// SimpleMutex uses std::sync::Mutex internally, so we provide safe_lock methods
impl<T> SimpleMutex<T> {
    /// Safe lock that returns Result for both parking-lot and std
    #[cfg(feature = "parking-lot")]
    pub fn safe_lock(&self) -> crate::core::types::TrackingResult<parking_lot::MutexGuard<'_, T>> {
        // parking-lot's lock() never fails
        Ok(self.lock())
    }

    #[cfg(not(feature = "parking-lot"))]
    pub fn safe_lock(&self) -> crate::core::types::TrackingResult<std::sync::MutexGuard<'_, T>> {
        // std::sync::Mutex's lock() returns Result<MutexGuard, PoisonError>
        self.lock().map_err(|_| {
            crate::core::types::TrackingError::LockError("Failed to acquire mutex lock".to_string())
        })
    }

    /// Safe try_lock that returns consistent Result for both parking-lot and std
    #[cfg(feature = "parking-lot")]
    pub fn try_safe_lock(
        &self,
    ) -> crate::core::types::TrackingResult<Option<parking_lot::MutexGuard<'_, T>>> {
        // parking-lot's try_lock() returns Option<MutexGuard>
        Ok(self.try_lock())
    }

    #[cfg(not(feature = "parking-lot"))]
    pub fn try_safe_lock(
        &self,
    ) -> crate::core::types::TrackingResult<Option<std::sync::MutexGuard<'_, T>>> {
        // std::sync::Mutex's try_lock() returns Result<MutexGuard, TryLockError>
        match self.try_lock() {
            Ok(guard) => Ok(Some(guard)),
            Err(std::sync::TryLockError::WouldBlock) => Ok(None),
            Err(_) => Err(crate::core::types::TrackingError::LockError(
                "Failed to try acquire mutex lock".to_string(),
            )),
        }
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
    fn test_simple_mutex_creation() {
        let mutex = SimpleMutex::new(42);
        assert_eq!(mutex.access_count(), 0);
    }

    #[test]
    fn test_simple_mutex_lock() {
        let mutex = SimpleMutex::new(42);

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

        #[cfg(debug_assertions)]
        assert_eq!(mutex.access_count(), 1);
    }

    #[test]
    fn test_simple_mutex_try_lock() {
        let mutex = SimpleMutex::new(42);

        #[cfg(feature = "parking-lot")]
        {
            let guard = mutex.try_lock();
            assert!(guard.is_some());
            assert_eq!(*guard.unwrap(), 42);
        }

        #[cfg(not(feature = "parking-lot"))]
        {
            let guard = mutex.try_lock();
            assert!(guard.is_ok());
            assert_eq!(*guard.unwrap(), 42);
        }

        #[cfg(debug_assertions)]
        assert_eq!(mutex.access_count(), 1);
    }

    #[test]
    fn test_simple_mutex_concurrent_access() {
        let mutex = Arc::new(SimpleMutex::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let mutex_clone = Arc::clone(&mutex);
            let handle = thread::spawn(move || {
                #[cfg(feature = "parking-lot")]
                {
                    let mut guard = mutex_clone.lock();
                    *guard += 1;
                }

                #[cfg(not(feature = "parking-lot"))]
                {
                    let mut guard = mutex_clone
                        .lock()
                        .expect("Mutex should not be poisoned in thread");
                    *guard += 1;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread should complete successfully");
        }

        #[cfg(feature = "parking-lot")]
        {
            let guard = mutex.lock();
            assert_eq!(*guard, 10);
        }

        #[cfg(not(feature = "parking-lot"))]
        {
            let guard = mutex.lock().unwrap();
            assert_eq!(*guard, 10);
        }
    }

    #[test]
    fn test_simple_mutex_default() {
        let mutex: SimpleMutex<i32> = SimpleMutex::default();

        #[cfg(feature = "parking-lot")]
        {
            let guard = mutex.lock();
            assert_eq!(*guard, 0);
        }

        #[cfg(not(feature = "parking-lot"))]
        {
            let guard = mutex.lock().unwrap();
            assert_eq!(*guard, 0);
        }
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_access_count_tracking() {
        let mutex = SimpleMutex::new(42);
        assert_eq!(mutex.access_count(), 0);

        #[cfg(feature = "parking-lot")]
        {
            let _guard1 = mutex.lock();
            assert_eq!(mutex.access_count(), 1);

            let _guard2 = mutex.try_lock();
            assert_eq!(mutex.access_count(), 2);
        }

        #[cfg(not(feature = "parking-lot"))]
        {
            let _guard1 = mutex.lock().unwrap();
            assert_eq!(mutex.access_count(), 1);

            let _guard2 = mutex.try_lock().unwrap();
            assert_eq!(mutex.access_count(), 2);
        }
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn test_access_count_release_mode() {
        let mutex = SimpleMutex::new(42);

        #[cfg(feature = "parking-lot")]
        {
            let _guard = mutex.lock();
        }

        #[cfg(not(feature = "parking-lot"))]
        {
            let _guard = mutex.lock().unwrap();
        }

        // In release mode, access count should always be 0
        assert_eq!(mutex.access_count(), 0);
    }
}
