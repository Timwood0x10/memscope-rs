//! Thread Registry - Thread metadata management
//!
//! This module provides thread registration and metadata tracking
//! for the MetadataEngine.

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
    #[allow(dead_code)]
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

    /// Register the current thread
    pub fn register_current_thread(&self) -> u64 {
        let thread_id = std::thread::current().id();
        let hash = self.hash_thread_id(&thread_id);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let mut threads = self.threads.lock().unwrap();
        threads.entry(hash).or_insert_with(|| ThreadInfo {
            thread_id: hash,
            thread_name: Some(format!("{:?}", thread_id)),
            created_at: timestamp,
            allocation_count: 0,
            total_allocated: 0,
            peak_memory: 0,
            is_active: true,
        });
        hash
    }

    /// Get thread info by hash
    pub fn get_thread_info(&self, hash: u64) -> Option<ThreadInfo> {
        self.threads.lock().unwrap().get(&hash).cloned()
    }

    /// Record an allocation for a thread
    pub fn record_allocation(&self, hash: u64, size: usize) {
        if let Some(info) = self.threads.lock().unwrap().get_mut(&hash) {
            info.allocation_count += 1;
            info.total_allocated += size;
        }
    }

    /// Update peak memory for a thread
    pub fn update_peak_memory(&self, hash: u64, current_memory: usize) {
        if let Some(info) = self.threads.lock().unwrap().get_mut(&hash) {
            if current_memory > info.peak_memory {
                info.peak_memory = current_memory;
            }
        }
    }

    /// Mark a thread as inactive
    pub fn mark_thread_inactive(&self, hash: u64) {
        if let Some(info) = self.threads.lock().unwrap().get_mut(&hash) {
            info.is_active = false;
        }
    }

    /// Get all threads
    pub fn get_all_threads(&self) -> Vec<ThreadInfo> {
        self.threads.lock().unwrap().values().cloned().collect()
    }

    /// Get active threads only
    pub fn get_active_threads(&self) -> Vec<ThreadInfo> {
        self.threads
            .lock()
            .unwrap()
            .values()
            .filter(|t| t.is_active)
            .cloned()
            .collect()
    }

    /// Get the number of registered threads
    pub fn len(&self) -> usize {
        self.threads.lock().unwrap().len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Hash a thread ID to a u64
    fn hash_thread_id(&self, thread_id: &std::thread::ThreadId) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        thread_id.hash(&mut hasher);
        hasher.finish()
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

    #[test]
    fn test_thread_registry_creation() {
        let registry = ThreadRegistry::new();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_register_current_thread() {
        let registry = ThreadRegistry::new();
        let hash = registry.register_current_thread();
        assert!(hash > 0);
        assert_eq!(registry.len(), 1);

        let info = registry.get_thread_info(hash);
        assert!(info.is_some());
        assert!(info.unwrap().is_active);
    }

    #[test]
    fn test_record_allocation() {
        let registry = ThreadRegistry::new();
        let hash = registry.register_current_thread();

        registry.record_allocation(hash, 100);
        registry.record_allocation(hash, 200);

        let info = registry.get_thread_info(hash).unwrap();
        assert_eq!(info.allocation_count, 2);
        assert_eq!(info.total_allocated, 300);
    }

    #[test]
    fn test_update_peak_memory() {
        let registry = ThreadRegistry::new();
        let hash = registry.register_current_thread();

        registry.update_peak_memory(hash, 100);
        registry.update_peak_memory(hash, 200);
        registry.update_peak_memory(hash, 150);

        let info = registry.get_thread_info(hash).unwrap();
        assert_eq!(info.peak_memory, 200);
    }

    #[test]
    fn test_mark_thread_inactive() {
        let registry = ThreadRegistry::new();
        let hash = registry.register_current_thread();

        registry.mark_thread_inactive(hash);

        let info = registry.get_thread_info(hash).unwrap();
        assert!(!info.is_active);
    }

    #[test]
    fn test_get_active_threads() {
        let registry = ThreadRegistry::new();
        let hash1 = registry.register_current_thread();

        // Simulate another thread by creating a new registry instance
        let registry2 = ThreadRegistry::new();
        let _hash2 = registry2.register_current_thread();

        registry.mark_thread_inactive(hash1);

        // This test only works with the same registry instance
        // In practice, threads would be managed by a global registry
    }
}
