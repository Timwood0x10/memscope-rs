//! Lock-Free Call Stack Normalization System
//!
//! This module implements a lock-free call stack normalization system to avoid
//! duplicate call stack information using atomic operations and lock-free data structures.
//! Replaces the previous lock-based implementation to comply with requirement.md.

use crate::analysis::unsafe_ffi_tracker::StackFrame;
use crate::core::error::TrackingResult;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Configuration for the call stack normalizer
#[derive(Debug, Clone)]
pub struct NormalizerConfig {
    /// Maximum number of normalized call stacks to cache
    pub max_cache_size: usize,
    /// Enable deduplication statistics
    pub enable_stats: bool,
    /// Hash algorithm selection
    pub hash_algorithm: HashAlgorithm,
}

#[derive(Debug, Clone)]
pub enum HashAlgorithm {
    FastHash,
    CryptoHash,
}

impl Default for NormalizerConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 10000,
            enable_stats: true,
            hash_algorithm: HashAlgorithm::FastHash,
        }
    }
}

/// Normalized call stack with unique ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedCallStack {
    pub id: u64,
    pub frames: Vec<StackFrame>,
    pub hash: u64,
    pub depth: usize,
}

/// Statistics for call stack normalization
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NormalizationStats {
    pub total_processed: u64,
    pub duplicates_avoided: u64,
    pub memory_saved_bytes: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hash_collisions: u64,
}

/// Lock-free call stack normalizer
pub struct LockFreeCallStackNormalizer {
    /// Registry of normalized call stacks indexed by ID
    stack_registry: DashMap<u64, NormalizedCallStack>,
    /// Mapping from hash to ID for fast lookup
    hash_to_id: DashMap<u64, u64>,
    /// Next available ID
    next_id: AtomicU64,
    /// Configuration
    config: NormalizerConfig,
    /// Statistics counters
    total_processed: AtomicU64,
    duplicates_avoided: AtomicU64,
    memory_saved_bytes: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    hash_collisions: AtomicU64,
}

impl LockFreeCallStackNormalizer {
    /// Create new lock-free call stack normalizer
    pub fn new(config: NormalizerConfig) -> Self {
        tracing::info!("🔧 Initializing Lock-Free Call Stack Normalizer");
        tracing::info!("   • Max cache size: {}", config.max_cache_size);
        tracing::info!("   • Statistics enabled: {}", config.enable_stats);
        
        Self {
            stack_registry: DashMap::with_capacity(config.max_cache_size),
            hash_to_id: DashMap::with_capacity(config.max_cache_size),
            next_id: AtomicU64::new(1),
            config,
            total_processed: AtomicU64::new(0),
            duplicates_avoided: AtomicU64::new(0),
            memory_saved_bytes: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            hash_collisions: AtomicU64::new(0),
        }
    }

    /// Normalize call stack and return reference ID
    pub fn normalize_call_stack(&self, frames: &[StackFrame]) -> TrackingResult<CallStackRef> {
        if frames.is_empty() {
            return Ok(CallStackRef::empty());
        }

        let hash = self.compute_hash(frames)?;
        
        // Check if we already have this call stack
        if let Some(existing_id) = self.hash_to_id.get(&hash) {
            self.increment_cache_hits();
            self.increment_duplicates_avoided(frames.len());
            
            tracing::debug!("📋 Found existing call stack with ID: {}", *existing_id);
            return Ok(CallStackRef::new(*existing_id, hash, frames.len()));
        }

        // Create new normalized call stack
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let normalized = NormalizedCallStack {
            id,
            frames: frames.to_vec(),
            hash,
            depth: frames.len(),
        };

        // Store in registry
        self.stack_registry.insert(id, normalized);
        self.hash_to_id.insert(hash, id);
        
        self.increment_cache_misses();
        self.increment_total_processed();

        tracing::debug!("📋 Created new normalized call stack with ID: {}", id);
        Ok(CallStackRef::new(id, hash, frames.len()))
    }

    /// Get call stack by reference ID
    pub fn get_call_stack(&self, call_stack_ref: &CallStackRef) -> TrackingResult<Vec<StackFrame>> {
        match self.stack_registry.get(&call_stack_ref.id) {
            Some(normalized) => Ok(normalized.frames.clone()),
            None => {
                tracing::warn!("📋 Call stack not found for ID: {}", call_stack_ref.id);
                Err(crate::core::error::TrackingError::DataNotFound(
                    format!("Call stack with ID {} not found", call_stack_ref.id)
                ))
            }
        }
    }

    /// Get call stack by ID
    pub fn get_call_stack_by_id(&self, id: u64) -> TrackingResult<Vec<StackFrame>> {
        match self.stack_registry.get(&id) {
            Some(normalized) => Ok(normalized.frames.clone()),
            None => {
                tracing::warn!("📋 Call stack not found for ID: {}", id);
                Err(crate::core::error::TrackingError::DataNotFound(
                    format!("Call stack with ID {} not found", id)
                ))
            }
        }
    }

    /// Check if call stack exists
    pub fn has_call_stack(&self, call_stack_ref: &CallStackRef) -> bool {
        self.stack_registry.contains_key(&call_stack_ref.id)
    }

    /// Get normalization statistics
    pub fn get_stats(&self) -> NormalizationStats {
        NormalizationStats {
            total_processed: self.total_processed.load(Ordering::Relaxed),
            duplicates_avoided: self.duplicates_avoided.load(Ordering::Relaxed),
            memory_saved_bytes: self.memory_saved_bytes.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            hash_collisions: self.hash_collisions.load(Ordering::Relaxed),
        }
    }

    /// Get number of normalized call stacks
    pub fn len(&self) -> usize {
        self.stack_registry.len()
    }

    /// Check if normalizer is empty
    pub fn is_empty(&self) -> bool {
        self.stack_registry.is_empty()
    }

    /// Clear all normalized call stacks
    pub fn clear(&self) {
        self.stack_registry.clear();
        self.hash_to_id.clear();
        self.next_id.store(1, Ordering::Relaxed);
        self.reset_stats();
        tracing::info!("📋 Cleared all normalized call stacks");
    }

    /// Compute hash for call stack frames
    fn compute_hash(&self, frames: &[StackFrame]) -> TrackingResult<u64> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        match self.config.hash_algorithm {
            HashAlgorithm::FastHash => {
                let mut hasher = DefaultHasher::new();
                for frame in frames {
                    frame.function_name.hash(&mut hasher);
                    frame.file_name.hash(&mut hasher);
                    frame.line_number.hash(&mut hasher);
                }
                Ok(hasher.finish())
            }
            HashAlgorithm::CryptoHash => {
                // For crypto hash, we would use a cryptographic hash function
                // For now, fall back to fast hash
                let mut hasher = DefaultHasher::new();
                for frame in frames {
                    frame.function_name.hash(&mut hasher);
                    frame.file_name.hash(&mut hasher);
                    frame.line_number.hash(&mut hasher);
                }
                Ok(hasher.finish())
            }
        }
    }

    /// Increment total processed counter
    fn increment_total_processed(&self) {
        if self.config.enable_stats {
            self.total_processed.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Increment cache hits counter
    fn increment_cache_hits(&self) {
        if self.config.enable_stats {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Increment cache misses counter
    fn increment_cache_misses(&self) {
        if self.config.enable_stats {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Increment duplicates avoided counter
    fn increment_duplicates_avoided(&self, stack_depth: usize) {
        if self.config.enable_stats {
            self.duplicates_avoided.fetch_add(1, Ordering::Relaxed);
            let memory_saved = stack_depth * std::mem::size_of::<StackFrame>();
            self.memory_saved_bytes.fetch_add(memory_saved as u64, Ordering::Relaxed);
        }
    }

    /// Reset all statistics
    fn reset_stats(&self) {
        self.total_processed.store(0, Ordering::Relaxed);
        self.duplicates_avoided.store(0, Ordering::Relaxed);
        self.memory_saved_bytes.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.hash_collisions.store(0, Ordering::Relaxed);
    }
}

impl Default for LockFreeCallStackNormalizer {
    fn default() -> Self {
        Self::new(NormalizerConfig::default())
    }
}

/// Global lock-free call stack normalizer instance
static GLOBAL_NORMALIZER: std::sync::OnceLock<Arc<LockFreeCallStackNormalizer>> =
    std::sync::OnceLock::new();

/// Get global lock-free call stack normalizer instance
pub fn get_global_call_stack_normalizer() -> Arc<LockFreeCallStackNormalizer> {
    GLOBAL_NORMALIZER
        .get_or_init(|| Arc::new(LockFreeCallStackNormalizer::new(NormalizerConfig::default())))
        .clone()
}

/// Initialize global call stack normalizer with custom config
pub fn initialize_global_call_stack_normalizer(
    config: NormalizerConfig,
) -> Arc<LockFreeCallStackNormalizer> {
    let normalizer = Arc::new(LockFreeCallStackNormalizer::new(config));
    match GLOBAL_NORMALIZER.set(normalizer.clone()) {
        Ok(_) => tracing::info!("📋 Global call stack normalizer initialized"),
        Err(_) => tracing::warn!("📋 Global call stack normalizer already initialized"),
    }
    normalizer
}

/// Call stack reference that uses ID instead of storing full frames
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CallStackRef {
    /// Unique ID for the normalized call stack
    pub id: u64,
    /// Hash of the original call stack for verification
    pub hash: u64,
    /// Depth of the call stack
    pub depth: usize,
}

impl CallStackRef {
    /// Create new call stack reference
    pub fn new(id: u64, hash: u64, depth: usize) -> Self {
        Self { id, hash, depth }
    }

    /// Create empty call stack reference
    pub fn empty() -> Self {
        Self {
            id: 0,
            hash: 0,
            depth: 0,
        }
    }

    /// Check if this is an empty reference
    pub fn is_empty(&self) -> bool {
        self.id == 0 && self.depth == 0
    }

    /// Get the actual call stack frames
    pub fn resolve(&self) -> TrackingResult<Vec<StackFrame>> {
        if self.is_empty() {
            return Ok(Vec::new());
        }
        
        let normalizer = get_global_call_stack_normalizer();
        normalizer.get_call_stack(self)
    }

    /// Verify hash matches expected value
    pub fn verify_hash(&self, expected_hash: u64) -> bool {
        self.hash == expected_hash
    }
}

impl Default for CallStackRef {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_free_normalizer_basic() {
        let normalizer = LockFreeCallStackNormalizer::new(NormalizerConfig::default());
        
        let frames = vec![
            StackFrame {
                function_name: "test_function".to_string(),
                file_name: "test.rs".to_string(),
                line_number: 42,
                column_number: Some(10),
                module_path: Some("test::module".to_string()),
            }
        ];

        let result = normalizer.normalize_call_stack(&frames);
        assert!(result.is_ok());
        
        let call_stack_ref = result.unwrap();
        assert!(!call_stack_ref.is_empty());
        assert_eq!(call_stack_ref.depth, 1);
    }

    #[test]
    fn test_deduplication() {
        let normalizer = LockFreeCallStackNormalizer::new(NormalizerConfig::default());
        
        let frames = vec![
            StackFrame {
                function_name: "test_function".to_string(),
                file_name: "test.rs".to_string(),
                line_number: 42,
                column_number: Some(10),
                module_path: Some("test::module".to_string()),
            }
        ];

        let ref1 = normalizer.normalize_call_stack(&frames).unwrap();
        let ref2 = normalizer.normalize_call_stack(&frames).unwrap();
        
        // Should get the same ID for identical call stacks
        assert_eq!(ref1.id, ref2.id);
        assert_eq!(ref1.hash, ref2.hash);
        
        let stats = normalizer.get_stats();
        assert_eq!(stats.duplicates_avoided, 1);
    }

    #[test]
    fn test_empty_call_stack() {
        let normalizer = LockFreeCallStackNormalizer::new(NormalizerConfig::default());
        
        let result = normalizer.normalize_call_stack(&[]);
        assert!(result.is_ok());
        
        let call_stack_ref = result.unwrap();
        assert!(call_stack_ref.is_empty());
    }
}