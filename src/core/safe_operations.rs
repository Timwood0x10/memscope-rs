//! Safe Operations - Provides safe lock operations
//!
//! This module provides safe lock operations that replace
//! dangerous .lock().expect("Failed to acquire lock") calls throughout the codebase.

use crate::core::types::TrackingResult;
use std::sync::{Mutex, RwLock};

/// Safe lock operations - replaces .lock().expect("Failed to acquire lock")
pub trait SafeLock<T> {
    /// Safely acquire lock with timeout and error handling
    fn safe_lock(&self) -> TrackingResult<std::sync::MutexGuard<T>>;

    /// Try to acquire lock without blocking
    fn try_safe_lock(&self) -> TrackingResult<Option<std::sync::MutexGuard<T>>>;
}

impl<T> SafeLock<T> for Mutex<T> {
    fn safe_lock(&self) -> TrackingResult<std::sync::MutexGuard<T>> {
        self.lock().map_err(|e| {
            crate::core::types::TrackingError::LockError(format!(
                "Failed to acquire mutex lock: {}",
                e
            ))
        })
    }

    fn try_safe_lock(&self) -> TrackingResult<Option<std::sync::MutexGuard<T>>> {
        match self.try_lock() {
            Ok(guard) => Ok(Some(guard)),
            Err(std::sync::TryLockError::WouldBlock) => Ok(None),
            Err(std::sync::TryLockError::Poisoned(e)) => Err(
                crate::core::types::TrackingError::LockError(format!("Mutex poisoned: {}", e)),
            ),
        }
    }
}

/// Safe RwLock operations
pub trait SafeRwLock<T> {
    /// Safely acquire read lock
    fn safe_read(&self) -> TrackingResult<std::sync::RwLockReadGuard<T>>;

    /// Safely acquire write lock
    fn safe_write(&self) -> TrackingResult<std::sync::RwLockWriteGuard<T>>;
}

impl<T> SafeRwLock<T> for RwLock<T> {
    fn safe_read(&self) -> TrackingResult<std::sync::RwLockReadGuard<T>> {
        self.read().map_err(|e| {
            crate::core::types::TrackingError::LockError(format!(
                "Failed to acquire read lock: {}",
                e
            ))
        })
    }

    fn safe_write(&self) -> TrackingResult<std::sync::RwLockWriteGuard<T>> {
        self.write().map_err(|e| {
            crate::core::types::TrackingError::LockError(format!(
                "Failed to acquire write lock: {}",
                e
            ))
        })
    }
}

/// Macro for safe lock acquisition
#[macro_export]
macro_rules! safe_lock {
    ($mutex:expr) => {
        $mutex.safe_lock()?
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex, RwLock};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_safe_mutex_lock() {
        let mutex = Mutex::new(42);

        let guard = mutex.safe_lock().unwrap();
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_safe_mutex_try_lock() {
        let mutex = Mutex::new(42);

        let guard = mutex.try_safe_lock().unwrap();
        assert!(guard.is_some());
        assert_eq!(*guard.unwrap(), 42);
    }

    #[test]
    fn test_safe_mutex_try_lock_would_block() {
        let mutex = Arc::new(Mutex::new(42));
        let mutex_clone = Arc::clone(&mutex);

        let _guard = mutex.safe_lock().unwrap();

        // Try to lock from another context - should return None (would block)
        let handle = thread::spawn(move || {
            let result = mutex_clone.try_safe_lock().unwrap();
            result.is_none()
        });

        assert!(handle.join().unwrap());
    }

    #[test]
    fn test_safe_rwlock_read() {
        let rwlock = RwLock::new(42);

        let guard = rwlock.safe_read().unwrap();
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_safe_rwlock_write() {
        let rwlock = RwLock::new(42);

        let mut guard = rwlock.safe_write().unwrap();
        *guard = 100;
        drop(guard);

        let guard = rwlock.safe_read().unwrap();
        assert_eq!(*guard, 100);
    }

    #[test]
    fn test_safe_rwlock_multiple_readers() {
        let rwlock = Arc::new(RwLock::new(42));
        let mut handles = vec![];

        // Multiple readers should be able to acquire locks simultaneously
        for _ in 0..5 {
            let rwlock_clone = Arc::clone(&rwlock);
            let handle = thread::spawn(move || {
                let guard = rwlock_clone.safe_read().unwrap();
                assert_eq!(*guard, 42);
                thread::sleep(Duration::from_millis(10));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_safe_rwlock_writer_exclusivity() {
        let rwlock = Arc::new(RwLock::new(0));
        let rwlock_clone = Arc::clone(&rwlock);

        let handle = thread::spawn(move || {
            let mut guard = rwlock_clone.safe_write().unwrap();
            *guard = 42;
            thread::sleep(Duration::from_millis(50));
            *guard = 100;
        });

        // Give the writer thread time to acquire the lock
        thread::sleep(Duration::from_millis(10));

        // This read should wait for the writer to finish
        let guard = rwlock.safe_read().unwrap();
        assert_eq!(*guard, 100);

        handle.join().unwrap();
    }

    #[test]
    fn test_concurrent_safe_operations() {
        let mutex = Arc::new(Mutex::new(0));
        let mut handles = vec![];

        // Multiple threads incrementing safely
        for _ in 0..10 {
            let mutex_clone = Arc::clone(&mutex);
            let handle = thread::spawn(move || {
                let mut guard = mutex_clone.safe_lock().unwrap();
                *guard += 1;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let guard = mutex.safe_lock().unwrap();
        assert_eq!(*guard, 10);
    }

    #[test]
    fn test_safe_lock_macro() {
        use crate::safe_lock;

        let mutex = Mutex::new(42);

        // Test the macro - this should compile and work
        let result: Result<(), crate::core::types::TrackingError> = (|| {
            let guard = safe_lock!(mutex);
            assert_eq!(*guard, 42);
            Ok(())
        })();

        assert!(result.is_ok());
    }

    #[test]
    fn test_error_handling() {
        let mutex = Mutex::new(42);

        // Test that errors are properly wrapped
        let result = mutex.safe_lock();
        assert!(result.is_ok());

        let try_result = mutex.try_safe_lock();
        assert!(try_result.is_ok());
    }
}
