//! Safe Operations - Provides safe lock operations
//!
//! This module provides safe lock operations that replace
//! dangerous .lock().unwrap() calls throughout the codebase.

use std::sync::{Mutex, RwLock};
use crate::core::types::TrackingResult;

/// Safe lock operations - replaces .lock().unwrap()
pub trait SafeLock<T> {
    /// Safely acquire lock with timeout and error handling
    fn safe_lock(&self) -> TrackingResult<std::sync::MutexGuard<T>>;
    
    /// Try to acquire lock without blocking
    fn try_safe_lock(&self) -> TrackingResult<Option<std::sync::MutexGuard<T>>>;
}

impl<T> SafeLock<T> for Mutex<T> {
    fn safe_lock(&self) -> TrackingResult<std::sync::MutexGuard<T>> {
        self.lock().map_err(|e| {
            crate::core::types::TrackingError::LockError(format!("Failed to acquire mutex lock: {}", e))
        })
    }
    
    fn try_safe_lock(&self) -> TrackingResult<Option<std::sync::MutexGuard<T>>> {
        match self.try_lock() {
            Ok(guard) => Ok(Some(guard)),
            Err(std::sync::TryLockError::WouldBlock) => Ok(None),
            Err(std::sync::TryLockError::Poisoned(e)) => {
                Err(crate::core::types::TrackingError::LockError(format!("Mutex poisoned: {}", e)))
            }
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
            crate::core::types::TrackingError::LockError(format!("Failed to acquire read lock: {}", e))
        })
    }
    
    fn safe_write(&self) -> TrackingResult<std::sync::RwLockWriteGuard<T>> {
        self.write().map_err(|e| {
            crate::core::types::TrackingError::LockError(format!("Failed to acquire write lock: {}", e))
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