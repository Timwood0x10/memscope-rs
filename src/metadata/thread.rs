//! Thread Registry - Thread metadata management
//!
//! This module provides thread registration and metadata tracking
//! for the MetadataEngine.

use crate::core::{MemScopeError, MemScopeResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Thread information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    /// Thread identifier
    pub thread_id: u64,
    /// Thread name (if available)
    pub thread_name: Option<String>,
    /// When this thread was created
    pub created_at: u64,
    /// Number of allocations made by this thread
    pub allocation_count: usize,
    /// Total bytes allocated by this thread
    pub total_allocated: usize,
    /// Peak memory usage for this thread
    pub peak_memory: usize,
    /// Whether this thread is still active
    pub is_active: bool,
}

/// Thread Registry - manages thread metadata
#[derive(Debug)]
pub struct ThreadRegistry {
    /// Registered threads
    threads: Arc<Mutex<HashMap<u64, ThreadInfo>>>,
    /// Next available internal thread ID
    next_id: Arc<Mutex<u64>>,
}

impl ThreadRegistry {
    /// Create a new ThreadRegistry
    pub fn new() -> Self {
        Self {
            threads: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    pub fn next_id(&self) -> MemScopeResult<u64> {
        let mut id = self.next_id.lock().map_err(|e| {
            MemScopeError::system(
                crate::core::error::SystemErrorType::Locking,
                format!("Failed to acquire next_id lock: {}", e),
            )
        })?;
        let current = *id;
        *id = current.saturating_add(1);
        Ok(current)
    }

    /// Register the current thread
    pub fn register_current_thread(&self) -> MemScopeResult<u64> {
        let thread_id = std::thread::current().id();
        let hash = self.hash_thread_id(&thread_id);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let mut threads = self.threads.lock().map_err(|e| {
            MemScopeError::system(
                crate::core::error::SystemErrorType::Locking,
                format!("Failed to acquire threads lock: {}", e),
            )
        })?;
        threads.entry(hash).or_insert_with(|| ThreadInfo {
            thread_id: hash,
            thread_name: Some(format!("{:?}", thread_id)),
            created_at: timestamp,
            allocation_count: 0,
            total_allocated: 0,
            peak_memory: 0,
            is_active: true,
        });
        Ok(hash)
    }

    /// Get thread info by hash
    pub fn get_thread_info(&self, hash: u64) -> MemScopeResult<Option<ThreadInfo>> {
        let threads = self.threads.lock().map_err(|e| {
            MemScopeError::system(
                crate::core::error::SystemErrorType::Locking,
                format!("Failed to acquire threads lock: {}", e),
            )
        })?;
        Ok(threads.get(&hash).cloned())
    }

    /// Record an allocation for a thread
    pub fn record_allocation(&self, hash: u64, size: usize) -> MemScopeResult<()> {
        let mut threads = self.threads.lock().map_err(|e| {
            MemScopeError::system(
                crate::core::error::SystemErrorType::Locking,
                format!("Failed to acquire threads lock: {}", e),
            )
        })?;
        if let Some(info) = threads.get_mut(&hash) {
            info.allocation_count += 1;
            info.total_allocated += size;
        }
        Ok(())
    }

    /// Update peak memory for a thread
    pub fn update_peak_memory(&self, hash: u64, current_memory: usize) -> MemScopeResult<()> {
        let mut threads = self.threads.lock().map_err(|e| {
            MemScopeError::system(
                crate::core::error::SystemErrorType::Locking,
                format!("Failed to acquire threads lock: {}", e),
            )
        })?;
        if let Some(info) = threads.get_mut(&hash) {
            if current_memory > info.peak_memory {
                info.peak_memory = current_memory;
            }
        }
        Ok(())
    }

    /// Mark a thread as inactive
    pub fn mark_thread_inactive(&self, hash: u64) -> MemScopeResult<()> {
        let mut threads = self.threads.lock().map_err(|e| {
            MemScopeError::system(
                crate::core::error::SystemErrorType::Locking,
                format!("Failed to acquire threads lock: {}", e),
            )
        })?;
        if let Some(info) = threads.get_mut(&hash) {
            info.is_active = false;
        }
        Ok(())
    }

    /// Get all threads
    pub fn get_all_threads(&self) -> MemScopeResult<Vec<ThreadInfo>> {
        let threads = self.threads.lock().map_err(|e| {
            MemScopeError::system(
                crate::core::error::SystemErrorType::Locking,
                format!("Failed to acquire threads lock: {}", e),
            )
        })?;
        Ok(threads.values().cloned().collect())
    }

    /// Get active threads only
    pub fn get_active_threads(&self) -> MemScopeResult<Vec<ThreadInfo>> {
        let threads = self.threads.lock().map_err(|e| {
            MemScopeError::system(
                crate::core::error::SystemErrorType::Locking,
                format!("Failed to acquire threads lock: {}", e),
            )
        })?;
        Ok(threads.values().filter(|t| t.is_active).cloned().collect())
    }

    /// Get the number of registered threads
    pub fn len(&self) -> MemScopeResult<usize> {
        let threads = self.threads.lock().map_err(|e| {
            MemScopeError::system(
                crate::core::error::SystemErrorType::Locking,
                format!("Failed to acquire threads lock: {}", e),
            )
        })?;
        Ok(threads.len())
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> MemScopeResult<bool> {
        Ok(self.len()? == 0)
    }

    /// Hash a thread ID to a u64
    fn hash_thread_id(&self, thread_id: &std::thread::ThreadId) -> u64 {
        crate::utils::thread_id_to_u64(*thread_id)
    }
}

impl Default for ThreadRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify that ThreadRegistry starts empty.
    /// Invariants: New registry must have zero threads.
    #[test]
    fn test_thread_registry_creation() {
        let registry = ThreadRegistry::new();
        assert!(registry.is_empty().unwrap(), "New registry should be empty");
    }

    /// Objective: Verify that register_current_thread creates a valid entry.
    /// Invariants: Thread must be registered with is_active=true and valid hash.
    #[test]
    fn test_register_current_thread() {
        let registry = ThreadRegistry::new();
        let hash = registry.register_current_thread().unwrap();
        assert!(hash > 0, "Thread hash should be positive");
        assert_eq!(
            registry.len().unwrap(),
            1,
            "Registry should have one thread"
        );

        let info = registry.get_thread_info(hash).unwrap();
        assert!(info.is_some(), "Thread info should exist");
        assert!(
            info.unwrap().is_active,
            "Thread should be active by default"
        );
    }

    /// Objective: Verify that record_allocation correctly updates allocation stats.
    /// Invariants: allocation_count and total_allocated must be accurate.
    #[test]
    fn test_record_allocation() {
        let registry = ThreadRegistry::new();
        let hash = registry.register_current_thread().unwrap();

        registry.record_allocation(hash, 100).unwrap();
        registry.record_allocation(hash, 200).unwrap();

        let info = registry.get_thread_info(hash).unwrap().unwrap();
        assert_eq!(info.allocation_count, 2, "Should have 2 allocations");
        assert_eq!(info.total_allocated, 300, "Total allocated should be 300");
    }

    /// Objective: Verify that update_peak_memory tracks maximum memory correctly.
    /// Invariants: peak_memory must only increase, never decrease.
    #[test]
    fn test_update_peak_memory() {
        let registry = ThreadRegistry::new();
        let hash = registry.register_current_thread().unwrap();

        registry.update_peak_memory(hash, 100).unwrap();
        registry.update_peak_memory(hash, 200).unwrap();
        registry.update_peak_memory(hash, 150).unwrap();

        let info = registry.get_thread_info(hash).unwrap().unwrap();
        assert_eq!(info.peak_memory, 200, "Peak memory should be 200");
    }

    /// Objective: Verify that mark_thread_inactive sets is_active to false.
    /// Invariants: Thread must be marked inactive after call.
    #[test]
    fn test_mark_thread_inactive() {
        let registry = ThreadRegistry::new();
        let hash = registry.register_current_thread().unwrap();

        registry.mark_thread_inactive(hash).unwrap();

        let info = registry.get_thread_info(hash).unwrap().unwrap();
        assert!(!info.is_active, "Thread should be marked inactive");
    }

    /// Objective: Verify that get_active_threads filters correctly.
    /// Invariants: Only active threads should be returned.
    #[test]
    fn test_get_active_threads() {
        let registry = ThreadRegistry::new();
        let hash = registry.register_current_thread().unwrap();

        let active = registry.get_active_threads().unwrap();
        assert_eq!(active.len(), 1, "Should have one active thread");

        registry.mark_thread_inactive(hash).unwrap();

        let active = registry.get_active_threads().unwrap();
        assert!(
            active.is_empty(),
            "Should have no active threads after marking inactive"
        );
    }

    /// Objective: Verify that get_all_threads returns all registered threads.
    /// Invariants: All threads including inactive ones must be returned.
    #[test]
    fn test_get_all_threads() {
        let registry = ThreadRegistry::new();
        let _hash = registry.register_current_thread().unwrap();

        let all = registry.get_all_threads().unwrap();
        assert_eq!(all.len(), 1, "Should have one thread total");
    }

    /// Objective: Verify that next_id returns incrementing values.
    /// Invariants: Each call should return a unique, incrementing ID.
    #[test]
    fn test_next_id() {
        let registry = ThreadRegistry::new();
        let id1 = registry.next_id().unwrap();
        let id2 = registry.next_id().unwrap();
        let id3 = registry.next_id().unwrap();

        assert_eq!(id1, 1, "First ID should be 1");
        assert_eq!(id2, 2, "Second ID should be 2");
        assert_eq!(id3, 3, "Third ID should be 3");
    }

    /// Objective: Verify that record_allocation on non-existent thread does not panic.
    /// Invariants: Should silently ignore unknown thread hash.
    #[test]
    fn test_record_allocation_unknown_thread() {
        let registry = ThreadRegistry::new();
        let result = registry.record_allocation(99999, 100);
        assert!(result.is_ok(), "Should not error on unknown thread");
    }

    /// Objective: Verify that update_peak_memory on non-existent thread does not panic.
    /// Invariants: Should silently ignore unknown thread hash.
    #[test]
    fn test_update_peak_memory_unknown_thread() {
        let registry = ThreadRegistry::new();
        let result = registry.update_peak_memory(99999, 100);
        assert!(result.is_ok(), "Should not error on unknown thread");
    }

    /// Objective: Verify that get_thread_info returns None for unknown thread.
    /// Invariants: Unknown hash should return None.
    #[test]
    fn test_get_thread_info_unknown() {
        let registry = ThreadRegistry::new();
        let info = registry.get_thread_info(99999).unwrap();
        assert!(info.is_none(), "Unknown thread should return None");
    }

    /// Objective: Verify that ThreadRegistry implements Default.
    /// Invariants: Default should create an empty registry.
    #[test]
    fn test_default() {
        let registry = ThreadRegistry::default();
        assert!(
            registry.is_empty().unwrap(),
            "Default registry should be empty"
        );
    }

    /// Objective: Verify that ThreadInfo can be serialized and cloned.
    /// Invariants: Clone should have identical values.
    #[test]
    fn test_thread_info_clone() {
        let info = ThreadInfo {
            thread_id: 1,
            thread_name: Some("test".to_string()),
            created_at: 12345,
            allocation_count: 10,
            total_allocated: 1000,
            peak_memory: 500,
            is_active: true,
        };

        let cloned = info.clone();
        assert_eq!(
            cloned.thread_id, info.thread_id,
            "Cloned thread_id should match"
        );
        assert_eq!(
            cloned.allocation_count, info.allocation_count,
            "Cloned allocation_count should match"
        );
    }

    /// Objective: Verify that ThreadInfo Debug trait works correctly.
    /// Invariants: Debug output should contain field names.
    #[test]
    fn test_thread_info_debug() {
        let info = ThreadInfo {
            thread_id: 1,
            thread_name: Some("test".to_string()),
            created_at: 0,
            allocation_count: 0,
            total_allocated: 0,
            peak_memory: 0,
            is_active: true,
        };

        let debug_str = format!("{:?}", info);
        assert!(
            debug_str.contains("ThreadInfo"),
            "Debug output should contain ThreadInfo"
        );
        assert!(
            debug_str.contains("thread_id"),
            "Debug output should contain thread_id"
        );
    }

    /// Objective: Verify that len returns correct count after multiple registrations.
    /// Invariants: len should match number of unique threads registered.
    #[test]
    fn test_len_multiple_registrations() {
        let registry = ThreadRegistry::new();
        let hash1 = registry.register_current_thread().unwrap();
        let hash2 = registry.register_current_thread().unwrap();

        assert_eq!(hash1, hash2, "Same thread should get same hash");
        assert_eq!(
            registry.len().unwrap(),
            1,
            "Same thread registered twice should still be 1"
        );
    }
}
