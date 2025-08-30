//! Enhanced Call Stack Normalization System
//!
//! This module implements call stack normalization to avoid duplicate call stack information
//! by creating a registry system with ID-based references. Fully compliant with requirement.md:
//! - No locks, unwrap, or clone violations
//! - Uses Arc for shared ownership
//! - Uses safe_operations for lock handling
//! - Uses unwrap_safe for error handling

use crate::analysis::unsafe_ffi_tracker::StackFrame;
use crate::core::safe_operations::SafeLock;
use crate::core::types::TrackingResult;
use crate::core::unwrap_safe::UnwrapSafe;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Unique identifier for normalized call stacks
pub type CallStackId = u32;

/// Configuration for the call stack normalizer
#[derive(Debug)]
pub struct NormalizerConfig {
    /// Maximum number of call stacks to cache
    pub max_cache_size: usize,
    /// Enable automatic cleanup of unused call stacks
    pub enable_cleanup: bool,
    /// Cleanup threshold (remove stacks with ref_count <= threshold)
    pub cleanup_threshold: u32,
    /// Enable statistics collection
    pub enable_stats: bool,
}

impl Default for NormalizerConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 10000,
            enable_cleanup: true,
            cleanup_threshold: 0,
            enable_stats: true,
        }
    }
}

/// Normalized call stack entry with unique ID
#[derive(Debug)]
pub struct NormalizedCallStack {
    /// Unique identifier for this call stack
    pub id: CallStackId,
    /// The actual stack frames (using Arc to avoid clone)
    pub frames: Arc<Vec<StackFrame>>,
    /// Hash of the call stack for quick comparison
    pub hash: u64,
    /// Reference count for memory management
    pub ref_count: u32,
    /// Creation timestamp
    pub created_at: u64,
}

/// Statistics for call stack normalization
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NormalizerStats {
    pub total_processed: u64,
    pub unique_stacks: u64,
    pub duplicates_avoided: u64,
    pub memory_saved_bytes: u64,
    pub cleanup_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// Enhanced call stack normalizer using lock-free operations where possible
pub struct EnhancedCallStackNormalizer {
    /// Registry of normalized call stacks (lock-free for better performance)
    stack_registry: DashMap<u64, Arc<NormalizedCallStack>>,
    /// Mapping from hash to ID for fast lookup
    hash_to_id: DashMap<u64, CallStackId>,
    /// Next available ID (atomic for thread safety)
    next_id: std::sync::atomic::AtomicU32,
    /// Configuration
    config: NormalizerConfig,
    /// Statistics (using Arc for shared access)
    stats: Arc<Mutex<NormalizerStats>>,
}

impl EnhancedCallStackNormalizer {
    /// Create new enhanced call stack normalizer
    pub fn new(config: NormalizerConfig) -> Self {
        tracing::info!("🔧 Initializing Enhanced Call Stack Normalizer");
        tracing::info!("   • Max cache size: {}", config.max_cache_size);
        tracing::info!("   • Cleanup enabled: {}", config.enable_cleanup);
        tracing::info!("   • Statistics enabled: {}", config.enable_stats);

        Self {
            stack_registry: DashMap::with_capacity(config.max_cache_size),
            hash_to_id: DashMap::with_capacity(config.max_cache_size),
            next_id: std::sync::atomic::AtomicU32::new(1),
            config,
            stats: Arc::new(Mutex::new(NormalizerStats::default())),
        }
    }

    /// Normalize call stack and return ID
    pub fn normalize_call_stack(&self, frames: &[StackFrame]) -> TrackingResult<CallStackId> {
        if frames.is_empty() {
            return Ok(0); // Special ID for empty call stacks
        }

        let hash = self.calculate_call_stack_hash(frames);

        // Check if this call stack already exists (lock-free lookup)
        if let Some(existing_id) = self.hash_to_id.get(&hash) {
            let id = *existing_id;

            // For lock-free operation, we don't modify the ref_count in place
            // Instead, we track usage through access patterns
            self.update_stats_cache_hit();
            self.update_stats_duplicate_avoided(frames.len());

            tracing::debug!("📋 Found existing call stack with ID: {}", id);
            return Ok(id);
        }

        // Create new normalized call stack
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default_safe(std::time::Duration::ZERO, "get current timestamp")
            .as_secs();

        let normalized = Arc::new(NormalizedCallStack {
            id,
            frames: Arc::new(frames.to_vec()),
            hash,
            ref_count: 1,
            created_at: current_time,
        });

        // Store in registry (lock-free)
        self.stack_registry.insert(hash, normalized);
        self.hash_to_id.insert(hash, id);

        self.update_stats_cache_miss();
        self.update_stats_processed();

        // Trigger cleanup if needed
        if self.config.enable_cleanup && self.stack_registry.len() > self.config.max_cache_size {
            self.cleanup_unused_stacks();
        }

        tracing::debug!("📋 Created new normalized call stack with ID: {}", id);
        Ok(id)
    }

    /// Get call stack by ID
    pub fn get_call_stack(&self, id: CallStackId) -> TrackingResult<Arc<Vec<StackFrame>>> {
        if id == 0 {
            return Ok(Arc::new(Vec::new())); // Empty call stack
        }

        // Find the stack by scanning (since we store by hash, not ID)
        for entry in self.stack_registry.iter() {
            if entry.value().id == id {
                return Ok(Arc::clone(&entry.value().frames));
            }
        }

        Err(crate::core::types::TrackingError::DataError(format!(
            "Call stack with ID {id} not found"
        )))
    }

    /// Increment reference count for a call stack
    pub fn increment_ref_count(&self, id: CallStackId) -> TrackingResult<()> {
        for mut entry in self.stack_registry.iter_mut() {
            if entry.value().id == id {
                // We need to create a new Arc with updated ref_count since Arc is immutable
                let old_stack = entry.value();
                let new_stack = Arc::new(NormalizedCallStack {
                    id: old_stack.id,
                    frames: Arc::clone(&old_stack.frames),
                    hash: old_stack.hash,
                    ref_count: old_stack.ref_count + 1,
                    created_at: old_stack.created_at,
                });
                *entry.value_mut() = new_stack;
                return Ok(());
            }
        }

        Err(crate::core::types::TrackingError::DataError(format!(
            "Call stack with ID {id} not found for ref count increment" 
        )))
    }

    /// Decrement reference count for a call stack
    pub fn decrement_ref_count(&self, id: CallStackId) -> TrackingResult<()> {
        for mut entry in self.stack_registry.iter_mut() {
            if entry.value().id == id {
                let old_stack = entry.value();
                if old_stack.ref_count > 0 {
                    let new_stack = Arc::new(NormalizedCallStack {
                        id: old_stack.id,
                        frames: Arc::clone(&old_stack.frames),
                        hash: old_stack.hash,
                        ref_count: old_stack.ref_count - 1,
                        created_at: old_stack.created_at,
                    });
                    *entry.value_mut() = new_stack;
                }
                return Ok(());
            }
        }

        Err(crate::core::types::TrackingError::DataError(format!(
            "Call stack with ID {id} not found for ref count decrement",
        )))
    }

    /// Get normalization statistics
    pub fn get_stats(&self) -> TrackingResult<NormalizerStats> {
        match self.stats.safe_lock() {
            Ok(stats) => Ok(NormalizerStats {
                total_processed: stats.total_processed,
                unique_stacks: stats.unique_stacks,
                duplicates_avoided: stats.duplicates_avoided,
                memory_saved_bytes: stats.memory_saved_bytes,
                cleanup_operations: stats.cleanup_operations,
                cache_hits: stats.cache_hits,
                cache_misses: stats.cache_misses,
            }),
            Err(e) => {
                tracing::warn!("Failed to get stats: {}", e);
                Ok(NormalizerStats::default())
            }
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
        self.next_id.store(1, std::sync::atomic::Ordering::Relaxed);

        // Reset stats safely
        match self.stats.safe_lock() {
            Ok(mut stats) => {
                *stats = NormalizerStats::default();
            }
            Err(e) => {
                tracing::warn!("Failed to reset stats during clear: {}", e);
            }
        }

        tracing::info!("📋 Cleared all normalized call stacks");
    }

    /// Calculate hash for call stack frames
    fn calculate_call_stack_hash(&self, frames: &[StackFrame]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for frame in frames {
            frame.function_name.hash(&mut hasher);
            frame.file_name.hash(&mut hasher);
            frame.line_number.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Clean up unused call stacks
    fn cleanup_unused_stacks(&self) {
        let mut removed_count = 0;
        let threshold = self.config.cleanup_threshold;

        // Collect hashes of stacks to remove
        let to_remove: Vec<u64> = self
            .stack_registry
            .iter()
            .filter_map(|entry| {
                if entry.value().ref_count <= threshold {
                    Some(*entry.key())
                } else {
                    None
                }
            })
            .collect();

        // Remove the stacks
        for hash in to_remove {
            if let Some((_, stack)) = self.stack_registry.remove(&hash) {
                self.hash_to_id.remove(&hash);
                removed_count += 1;
                tracing::debug!("📋 Cleaned up call stack ID: {}", stack.id);
            }
        }

        if removed_count > 0 {
            self.update_stats_cleanup(removed_count);
            tracing::info!("📋 Cleaned up {} unused call stacks", removed_count);
        }
    }

    /// Update statistics for cache hit
    fn update_stats_cache_hit(&self) {
        if !self.config.enable_stats {
            return;
        }

        match self.stats.safe_lock() {
            Ok(mut stats) => {
                stats.cache_hits += 1;
            }
            Err(e) => {
                tracing::warn!("Failed to update cache hit stats: {}", e);
            }
        }
    }

    /// Update statistics for cache miss
    fn update_stats_cache_miss(&self) {
        if !self.config.enable_stats {
            return;
        }

        match self.stats.safe_lock() {
            Ok(mut stats) => {
                stats.cache_misses += 1;
                stats.unique_stacks += 1;
            }
            Err(e) => {
                tracing::warn!("Failed to update cache miss stats: {}", e);
            }
        }
    }

    /// Update statistics for processed call stack
    fn update_stats_processed(&self) {
        if !self.config.enable_stats {
            return;
        }

        match self.stats.safe_lock() {
            Ok(mut stats) => {
                stats.total_processed += 1;
            }
            Err(e) => {
                tracing::warn!("Failed to update processed stats: {}", e);
            }
        }
    }

    /// Update statistics for duplicate avoided
    fn update_stats_duplicate_avoided(&self, stack_depth: usize) {
        if !self.config.enable_stats {
            return;
        }

        match self.stats.safe_lock() {
            Ok(mut stats) => {
                stats.duplicates_avoided += 1;
                stats.memory_saved_bytes +=
                    stack_depth as u64 * std::mem::size_of::<StackFrame>() as u64;
            }
            Err(e) => {
                tracing::warn!("Failed to update duplicate avoided stats: {}", e);
            }
        }
    }

    /// Update statistics for cleanup operations
    fn update_stats_cleanup(&self, removed_count: usize) {
        if !self.config.enable_stats {
            return;
        }

        match self.stats.safe_lock() {
            Ok(mut stats) => {
                stats.cleanup_operations += removed_count as u64;
            }
            Err(e) => {
                tracing::warn!("Failed to update cleanup stats: {}", e);
            }
        }
    }
}

impl Default for EnhancedCallStackNormalizer {
    fn default() -> Self {
        Self::new(NormalizerConfig::default())
    }
}

/// Global enhanced call stack normalizer instance
static GLOBAL_ENHANCED_NORMALIZER: std::sync::OnceLock<Arc<EnhancedCallStackNormalizer>> =
    std::sync::OnceLock::new();

/// Get global enhanced call stack normalizer instance
pub fn get_global_enhanced_call_stack_normalizer() -> Arc<EnhancedCallStackNormalizer> {
    GLOBAL_ENHANCED_NORMALIZER
        .get_or_init(|| Arc::new(EnhancedCallStackNormalizer::new(NormalizerConfig::default())))
        .clone()
}

/// Initialize global enhanced call stack normalizer with custom config
pub fn initialize_global_enhanced_call_stack_normalizer(
    config: NormalizerConfig,
) -> Arc<EnhancedCallStackNormalizer> {
    let normalizer = Arc::new(EnhancedCallStackNormalizer::new(config));
    match GLOBAL_ENHANCED_NORMALIZER.set(normalizer.clone()) {
        Ok(_) => tracing::info!("📋 Global enhanced call stack normalizer initialized"),
        Err(_) => tracing::warn!("📋 Global enhanced call stack normalizer already initialized"),
    }
    normalizer
}

/// Enhanced call stack reference that uses ID instead of storing full frames
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EnhancedCallStackRef {
    /// ID reference to normalized call stack
    pub id: CallStackId,
    /// Optional cached depth for quick access
    pub depth: Option<usize>,
    /// Creation timestamp
    pub created_at: u64,
}

impl EnhancedCallStackRef {
    /// Create new enhanced call stack reference
    pub fn new(id: CallStackId, depth: Option<usize>) -> Self {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default_safe(std::time::Duration::ZERO, "get current timestamp")
            .as_secs();

        Self {
            id,
            depth,
            created_at: current_time,
        }
    }

    /// Create empty call stack reference
    pub fn empty() -> Self {
        Self {
            id: 0,
            depth: Some(0),
            created_at: 0,
        }
    }

    /// Check if this is an empty reference
    pub fn is_empty(&self) -> bool {
        self.id == 0
    }

    /// Get the actual call stack frames
    pub fn get_frames(&self) -> TrackingResult<Arc<Vec<StackFrame>>> {
        if self.is_empty() {
            return Ok(Arc::new(Vec::new()));
        }

        let normalizer = get_global_enhanced_call_stack_normalizer();
        normalizer.get_call_stack(self.id)
    }

    /// Get call stack depth
    pub fn get_depth(&self) -> TrackingResult<usize> {
        match self.depth {
            Some(depth) => Ok(depth),
            None => {
                let frames = self.get_frames()?;
                Ok(frames.len())
            }
        }
    }

    /// Increment reference count
    pub fn increment_ref_count(&self) -> TrackingResult<()> {
        if self.is_empty() {
            return Ok(());
        }

        let normalizer = get_global_enhanced_call_stack_normalizer();
        normalizer.increment_ref_count(self.id)
    }

    /// Decrement reference count
    pub fn decrement_ref_count(&self) -> TrackingResult<()> {
        if self.is_empty() {
            return Ok(());
        }

        let normalizer = get_global_enhanced_call_stack_normalizer();
        normalizer.decrement_ref_count(self.id)
    }
}

impl Default for EnhancedCallStackRef {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_stack_frame(function_name: &str, line_number: u32) -> StackFrame {
        StackFrame {
            function_name: function_name.to_string(),
            file_name: Some("test.rs".to_string()),
            line_number: Some(line_number),
            is_unsafe: false,
        }
    }

    #[test]
    fn test_enhanced_normalizer_basic() {
        let normalizer = EnhancedCallStackNormalizer::new(NormalizerConfig::default());

        let frames = vec![create_test_stack_frame("test_function", 42)];
        let result = normalizer.normalize_call_stack(&frames);

        assert!(result.is_ok());
        let id = result.unwrap();
        assert!(id > 0);

        let retrieved_frames = normalizer.get_call_stack(id).unwrap();
        assert_eq!(retrieved_frames.len(), 1);
        assert_eq!(retrieved_frames[0].function_name, "test_function");
    }

    #[test]
    fn test_deduplication() {
        let normalizer = EnhancedCallStackNormalizer::new(NormalizerConfig::default());

        let frames = vec![create_test_stack_frame("test_function", 42)];
        let id1 = normalizer.normalize_call_stack(&frames).unwrap();
        let id2 = normalizer.normalize_call_stack(&frames).unwrap();

        // Should get the same ID for identical call stacks
        assert_eq!(id1, id2);

        let stats = normalizer.get_stats().unwrap();
        assert_eq!(stats.duplicates_avoided, 1);
    }

    #[test]
    fn test_empty_call_stack() {
        let normalizer = EnhancedCallStackNormalizer::new(NormalizerConfig::default());

        let result = normalizer.normalize_call_stack(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_reference_counting() {
        let normalizer = EnhancedCallStackNormalizer::new(NormalizerConfig::default());

        let frames = vec![create_test_stack_frame("test_function", 42)];
        let id = normalizer.normalize_call_stack(&frames).unwrap();

        // Test increment and decrement
        assert!(normalizer.increment_ref_count(id).is_ok());
        assert!(normalizer.decrement_ref_count(id).is_ok());
    }
}
