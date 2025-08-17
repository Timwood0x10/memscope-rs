//! Safe Operations - Replaces unwrap() with proper error handling
//!
//! This module provides safe operation utilities that replace
//! dangerous unwrap() calls throughout the codebase.

use std::sync::{Arc, Mutex, RwLock};
use std::fmt;
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

/// Safe Option operations - replaces .unwrap()
pub trait SafeUnwrap<T> {
    /// Safely unwrap with descriptive error
    fn safe_unwrap(self, context: &str) -> TrackingResult<T>;
    
    /// Safely unwrap with default value
    fn safe_unwrap_or(self, default: T) -> T;
    
    /// Safely unwrap with closure
    fn safe_unwrap_or_else<F>(self, f: F) -> T where F: FnOnce() -> T;
}

impl<T> SafeUnwrap<T> for Option<T> {
    fn safe_unwrap(self, context: &str) -> TrackingResult<T> {
        self.ok_or_else(|| {
            crate::core::types::TrackingError::DataError(format!("Failed to unwrap Option: {}", context))
        })
    }
    
    fn safe_unwrap_or(self, default: T) -> T {
        self.unwrap_or(default)
    }
    
    fn safe_unwrap_or_else<F>(self, f: F) -> T 
    where 
        F: FnOnce() -> T 
    {
        self.unwrap_or_else(f)
    }
}

/// Safe Result operations - replaces .unwrap()
impl<T, E: fmt::Display> SafeUnwrap<T> for Result<T, E> {
    fn safe_unwrap(self, context: &str) -> TrackingResult<T> {
        self.map_err(|e| {
            crate::core::types::TrackingError::DataError(format!("Failed to unwrap Result in {}: {}", context, e))
        })
    }
    
    fn safe_unwrap_or(self, default: T) -> T {
        self.unwrap_or(default)
    }
    
    fn safe_unwrap_or_else<F>(self, f: F) -> T 
    where 
        F: FnOnce() -> T 
    {
        self.unwrap_or_else(|_| f())
    }
}

/// Safe Arc operations - replaces clone() with zero-cost sharing
pub trait SafeArc<T> {
    /// Clone Arc reference (zero-cost)
    fn safe_clone(&self) -> Arc<T>;
    
    /// Try to get exclusive access
    fn try_unwrap_arc(self) -> Result<T, Arc<T>>;
}

impl<T> SafeArc<T> for Arc<T> {
    fn safe_clone(&self) -> Arc<T> {
        Arc::clone(self) // Zero-cost reference counting
    }
    
    fn try_unwrap_arc(self) -> Result<T, Arc<T>> {
        Arc::try_unwrap(self)
    }
}

/// Safe I/O operations
pub trait SafeIo {
    /// Safe file operations with proper error handling
    fn safe_read_to_string<P: AsRef<std::path::Path>>(path: P) -> TrackingResult<String>;
    
    /// Safe file write with proper error handling
    fn safe_write<P: AsRef<std::path::Path>>(path: P, contents: &str) -> TrackingResult<()>;
    
    /// Safe directory creation
    fn safe_create_dir_all<P: AsRef<std::path::Path>>(path: P) -> TrackingResult<()>;
}

pub struct FileOps;

impl SafeIo for FileOps {
    fn safe_read_to_string<P: AsRef<std::path::Path>>(path: P) -> TrackingResult<String> {
        std::fs::read_to_string(path.as_ref()).map_err(|e| {
            crate::core::types::TrackingError::IoError(format!(
                "Failed to read file '{}': {}", 
                path.as_ref().display(), 
                e
            ))
        })
    }
    
    fn safe_write<P: AsRef<std::path::Path>>(path: P, contents: &str) -> TrackingResult<()> {
        std::fs::write(path.as_ref(), contents).map_err(|e| {
            crate::core::types::TrackingError::IoError(format!(
                "Failed to write file '{}': {}", 
                path.as_ref().display(), 
                e
            ))
        })
    }
    
    fn safe_create_dir_all<P: AsRef<std::path::Path>>(path: P) -> TrackingResult<()> {
        std::fs::create_dir_all(path.as_ref()).map_err(|e| {
            crate::core::types::TrackingError::IoError(format!(
                "Failed to create directory '{}': {}", 
                path.as_ref().display(), 
                e
            ))
        })
    }
}

/// Safe JSON operations
pub trait SafeJson {
    /// Safe JSON serialization
    fn safe_to_json<T: serde::Serialize>(value: &T) -> TrackingResult<String>;
    
    /// Safe JSON deserialization
    fn safe_from_json<T: serde::de::DeserializeOwned>(json: &str) -> TrackingResult<T>;
    
    /// Safe JSON pretty printing
    fn safe_to_json_pretty<T: serde::Serialize>(value: &T) -> TrackingResult<String>;
}

pub struct JsonOps;

impl SafeJson for JsonOps {
    fn safe_to_json<T: serde::Serialize>(value: &T) -> TrackingResult<String> {
        serde_json::to_string(value).map_err(|e| {
            crate::core::types::TrackingError::SerializationError(format!("JSON serialization failed: {}", e))
        })
    }
    
    fn safe_from_json<T: serde::de::DeserializeOwned>(json: &str) -> TrackingResult<T> {
        serde_json::from_str(json).map_err(|e| {
            crate::core::types::TrackingError::SerializationError(format!("JSON deserialization failed: {}", e))
        })
    }
    
    fn safe_to_json_pretty<T: serde::Serialize>(value: &T) -> TrackingResult<String> {
        serde_json::to_string_pretty(value).map_err(|e| {
            crate::core::types::TrackingError::SerializationError(format!("JSON pretty serialization failed: {}", e))
        })
    }
}

/// Macro for safe unwrapping with context
#[macro_export]
macro_rules! safe_unwrap {
    ($expr:expr, $context:expr) => {
        $expr.safe_unwrap($context)?
    };
}

/// Macro for safe lock acquisition
#[macro_export]
macro_rules! safe_lock {
    ($mutex:expr) => {
        $mutex.safe_lock()?
    };
}

/// Macro for safe Arc cloning
#[macro_export]
macro_rules! safe_arc_clone {
    ($arc:expr) => {
        $arc.safe_clone()
    };
}