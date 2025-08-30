//! Call Stack Normalization System
//!
//! This module implements call stack normalization to avoid duplicate call stack information
//! by creating a registry system with ID-based references.

use crate::analysis::unsafe_ffi_tracker::StackFrame;
use crate::core::types::{TrackingError, TrackingResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Unique identifier for normalized call stacks
pub type CallStackId = u32;

/// Normalized call stack entry with unique ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedCallStack {
    /// Unique identifier for this call stack
    pub id: CallStackId,
    /// The actual stack frames
    pub frames: Vec<StackFrame>,
    /// Hash of the call stack for quick comparison
    pub hash: u64,
    /// Reference count for memory management
    pub ref_count: u32,
    /// Creation timestamp
    pub created_at: u64,
}

/// Call stack normalizer registry
pub struct CallStackNormalizer {
    /// Registry mapping call stack hashes to normalized entries
    stack_registry: Arc<Mutex<HashMap<u64, NormalizedCallStack>>>,
    /// ID to hash mapping for quick lookups
    id_to_hash: Arc<Mutex<HashMap<CallStackId, u64>>>,
    /// Next available ID counter
    next_id: Arc<Mutex<CallStackId>>,
    /// Configuration for the normalizer
    config: NormalizerConfig,
    /// Statistics tracking
    stats: Arc<Mutex<NormalizerStats>>,
}

/// Configuration for call stack normalizer
#[derive(Debug, Clone)]
pub struct NormalizerConfig {
    /// Maximum number of call stacks to keep in registry
    pub max_registry_size: usize,
    /// Enable automatic cleanup of unused call stacks
    pub enable_auto_cleanup: bool,
    /// Minimum reference count to keep during cleanup
    pub min_ref_count_for_cleanup: u32,
    /// Enable detailed statistics tracking
    pub enable_statistics: bool,
}

impl Default for NormalizerConfig {
    fn default() -> Self {
        Self {
            max_registry_size: 10000,
            enable_auto_cleanup: true,
            min_ref_count_for_cleanup: 1,
            enable_statistics: true,
        }
    }
}

/// Statistics for call stack normalizer
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NormalizerStats {
    /// Total call stacks processed
    pub total_processed: usize,
    /// Number of unique call stacks
    pub unique_stacks: usize,
    /// Number of duplicate call stacks avoided
    pub duplicates_avoided: usize,
    /// Memory saved by normalization (estimated bytes)
    pub memory_saved_bytes: usize,
    /// Registry cleanup operations performed
    pub cleanup_operations: usize,
    /// Average call stack depth
    pub average_stack_depth: f64,
    /// Statistics collection start time
    pub stats_start_time: u64,
}

impl CallStackNormalizer {
    /// Create new call stack normalizer
    pub fn new(config: NormalizerConfig) -> Self {
        tracing::info!("📚 Initializing Call Stack Normalizer");
        tracing::info!("   • Max registry size: {}", config.max_registry_size);
        tracing::info!("   • Auto cleanup: {}", config.enable_auto_cleanup);
        tracing::info!("   • Statistics: {}", config.enable_statistics);

        Self {
            stack_registry: Arc::new(Mutex::new(HashMap::new())),
            id_to_hash: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            config,
            stats: Arc::new(Mutex::new(NormalizerStats {
                stats_start_time: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                ..Default::default()
            })),
        }
    }

    /// Normalize call stack and return unique ID
    pub fn normalize_call_stack(&self, frames: &[StackFrame]) -> TrackingResult<CallStackId> {
        if frames.is_empty() {
            return Err(TrackingError::InvalidPointer(
                "Empty call stack".to_string(),
            ));
        }

        let hash = self.calculate_call_stack_hash(frames);

        // Check if this call stack already exists
        if let Ok(mut registry) = self.stack_registry.lock() {
            if let Some(existing) = registry.get_mut(&hash) {
                // Increment reference count
                existing.ref_count += 1;

                // Update statistics
                self.update_stats_duplicate_avoided(frames.len());

                tracing::debug!("📚 Reused existing call stack ID: {}", existing.id);
                return Ok(existing.id);
            }

            // Create new normalized call stack
            let id = self.get_next_id()?;
            let normalized = NormalizedCallStack {
                id,
                frames: frames.to_vec(),
                hash,
                ref_count: 1,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };

            // Check registry size limit
            if registry.len() >= self.config.max_registry_size {
                if self.config.enable_auto_cleanup {
                    self.cleanup_registry_internal(&mut registry)?;
                } else {
                    return Err(TrackingError::ResourceExhausted(
                        "Call stack registry full".to_string(),
                    ));
                }
            }

            // Store in registry
            registry.insert(hash, normalized);

            // Update ID mapping
            if let Ok(mut id_map) = self.id_to_hash.lock() {
                id_map.insert(id, hash);
            }

            // Update statistics
            self.update_stats_new_stack(frames.len());

            tracing::debug!("📚 Created new call stack ID: {} (hash: {:x})", id, hash);
            Ok(id)
        } else {
            Err(TrackingError::LockContention(
                "Failed to lock registry".to_string(),
            ))
        }
    }

    /// Get call stack by ID
    pub fn get_call_stack(&self, id: CallStackId) -> TrackingResult<Vec<StackFrame>> {
        // Get hash from ID
        let hash = if let Ok(id_map) = self.id_to_hash.lock() {
            id_map.get(&id).copied().ok_or_else(|| {
                TrackingError::InvalidPointer(format!("Invalid call stack ID: {id}"))
            })?
        } else {
            return Err(TrackingError::LockContention(
                "Failed to lock ID mapping".to_string(),
            ));
        };

        // Get call stack from registry
        if let Ok(registry) = self.stack_registry.lock() {
            registry
                .get(&hash)
                .map(|normalized| normalized.frames.clone())
                .ok_or_else(|| {
                    TrackingError::InvalidPointer(format!("Call stack not found for ID: {id}"))
                })
        } else {
            Err(TrackingError::LockContention(
                "Failed to lock registry".to_string(),
            ))
        }
    }

    /// Increment reference count for call stack
    pub fn increment_ref_count(&self, id: CallStackId) -> TrackingResult<()> {
        let hash = if let Ok(id_map) = self.id_to_hash.lock() {
            id_map.get(&id).copied().ok_or_else(|| {
                TrackingError::InvalidPointer(format!("Invalid call stack ID: {id}"))
            })?
        } else {
            return Err(TrackingError::LockContention(
                "Failed to lock ID mapping".to_string(),
            ));
        };

        if let Ok(mut registry) = self.stack_registry.lock() {
            if let Some(normalized) = registry.get_mut(&hash) {
                normalized.ref_count += 1;
                tracing::debug!(
                    "📚 Incremented ref count for ID: {} to {}",
                    id,
                    normalized.ref_count
                );
                Ok(())
            } else {
                Err(TrackingError::InvalidPointer(format!(
                    "Call stack not found for ID: {id}",
                )))
            }
        } else {
            Err(TrackingError::LockContention(
                "Failed to lock registry".to_string(),
            ))
        }
    }

    /// Decrement reference count for call stack
    pub fn decrement_ref_count(&self, id: CallStackId) -> TrackingResult<()> {
        let hash = if let Ok(id_map) = self.id_to_hash.lock() {
            id_map.get(&id).copied().ok_or_else(|| {
                TrackingError::InvalidPointer(format!("Invalid call stack ID: {id}"))
            })?
        } else {
            return Err(TrackingError::LockContention(
                "Failed to lock ID mapping".to_string(),
            ));
        };

        if let Ok(mut registry) = self.stack_registry.lock() {
            if let Some(normalized) = registry.get_mut(&hash) {
                if normalized.ref_count > 0 {
                    normalized.ref_count -= 1;
                    tracing::debug!(
                        "📚 Decremented ref count for ID: {} to {}",
                        id,
                        normalized.ref_count
                    );
                }
                Ok(())
            } else {
                Err(TrackingError::InvalidPointer(format!(
                    "Call stack not found for ID: {id}"
                )))
            }
        } else {
            Err(TrackingError::LockContention(
                "Failed to lock registry".to_string(),
            ))
        }
    }

    /// Perform manual cleanup of unused call stacks
    pub fn cleanup_registry(&self) -> TrackingResult<usize> {
        if let Ok(mut registry) = self.stack_registry.lock() {
            self.cleanup_registry_internal(&mut registry)
        } else {
            Err(TrackingError::LockContention(
                "Failed to lock registry".to_string(),
            ))
        }
    }

    /// Get normalizer statistics
    pub fn get_stats(&self) -> NormalizerStats {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            tracing::error!("Failed to lock stats");
            NormalizerStats::default()
        }
    }

    /// Get registry size
    pub fn get_registry_size(&self) -> usize {
        self.stack_registry.lock().map(|r| r.len()).unwrap_or(0)
    }

    /// Clear all call stacks (for testing)
    pub fn clear_registry(&self) {
        if let Ok(mut registry) = self.stack_registry.lock() {
            registry.clear();
        }
        if let Ok(mut id_map) = self.id_to_hash.lock() {
            id_map.clear();
        }
        if let Ok(mut next_id) = self.next_id.lock() {
            *next_id = 1;
        }
        if let Ok(mut stats) = self.stats.lock() {
            *stats = NormalizerStats {
                stats_start_time: stats.stats_start_time,
                ..Default::default()
            };
        }
        tracing::info!("🧹 Cleared call stack registry");
    }

    // Private helper methods

    fn calculate_call_stack_hash(&self, frames: &[StackFrame]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash each frame's key components
        for frame in frames {
            frame.function_name.hash(&mut hasher);
            frame.file_name.hash(&mut hasher);
            frame.line_number.hash(&mut hasher);
            frame.is_unsafe.hash(&mut hasher);
        }

        hasher.finish()
    }

    fn get_next_id(&self) -> TrackingResult<CallStackId> {
        if let Ok(mut next_id) = self.next_id.lock() {
            let id = *next_id;
            *next_id = next_id.wrapping_add(1);
            Ok(id)
        } else {
            Err(TrackingError::LockContention(
                "Failed to lock next ID counter".to_string(),
            ))
        }
    }

    fn cleanup_registry_internal(
        &self,
        registry: &mut HashMap<u64, NormalizedCallStack>,
    ) -> TrackingResult<usize> {
        let initial_size = registry.len();
        let min_ref_count = self.config.min_ref_count_for_cleanup;

        // Remove entries with low reference counts
        registry.retain(|_, normalized| normalized.ref_count >= min_ref_count);

        // Update ID mapping
        if let Ok(mut id_map) = self.id_to_hash.lock() {
            id_map.retain(|_, hash| registry.contains_key(hash));
        }

        let removed_count = initial_size - registry.len();

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.cleanup_operations += 1;
            stats.unique_stacks = registry.len();
        }

        tracing::info!(
            "🧹 Cleaned up {} unused call stacks from registry",
            removed_count
        );
        Ok(removed_count)
    }

    fn update_stats_new_stack(&self, stack_depth: usize) {
        if !self.config.enable_statistics {
            return;
        }

        if let Ok(mut stats) = self.stats.lock() {
            stats.total_processed += 1;
            stats.unique_stacks += 1;

            // Update average stack depth
            let total_depth =
                stats.average_stack_depth * (stats.unique_stacks - 1) as f64 + stack_depth as f64;
            stats.average_stack_depth = total_depth / stats.unique_stacks as f64;

            // Estimate memory saved (rough calculation)
            stats.memory_saved_bytes += stack_depth * std::mem::size_of::<StackFrame>();
        }
    }

    fn update_stats_duplicate_avoided(&self, stack_depth: usize) {
        if !self.config.enable_statistics {
            return;
        }

        if let Ok(mut stats) = self.stats.lock() {
            stats.total_processed += 1;
            stats.duplicates_avoided += 1;

            // Estimate memory saved by avoiding duplication
            stats.memory_saved_bytes += stack_depth * std::mem::size_of::<StackFrame>();
        }
    }
}

impl Default for CallStackNormalizer {
    fn default() -> Self {
        Self::new(NormalizerConfig::default())
    }
}

/// Global call stack normalizer instance
static GLOBAL_NORMALIZER: std::sync::OnceLock<Arc<CallStackNormalizer>> =
    std::sync::OnceLock::new();

/// Get global call stack normalizer instance
pub fn get_global_call_stack_normalizer() -> Arc<CallStackNormalizer> {
    GLOBAL_NORMALIZER
        .get_or_init(|| Arc::new(CallStackNormalizer::new(NormalizerConfig::default())))
        .clone()
}

/// Initialize global call stack normalizer with custom config
pub fn initialize_global_call_stack_normalizer(
    config: NormalizerConfig,
) -> Arc<CallStackNormalizer> {
    let normalizer = Arc::new(CallStackNormalizer::new(config));
    if GLOBAL_NORMALIZER.set(normalizer.clone()).is_err() {
        tracing::warn!("Global call stack normalizer already initialized");
    }
    normalizer
}

/// Call stack reference that uses ID instead of storing full frames
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallStackRef {
    /// ID reference to normalized call stack
    pub id: CallStackId,
    /// Optional cached depth for quick access
    pub depth: Option<usize>,
    /// Creation timestamp
    pub created_at: u64,
}

impl CallStackRef {
    /// Create new call stack reference
    pub fn new(id: CallStackId, depth: Option<usize>) -> Self {
        Self {
            id,
            depth,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Get the actual call stack frames
    pub fn get_frames(&self) -> TrackingResult<Vec<StackFrame>> {
        let normalizer = get_global_call_stack_normalizer();
        normalizer.get_call_stack(self.id)
    }

    /// Get call stack depth (cached or calculated)
    pub fn get_depth(&self) -> TrackingResult<usize> {
        match self.depth {
            Some(depth) => Ok(depth),
            None => {
                let frames = self.get_frames()?;
                Ok(frames.len())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_stack_frame(function_name: &str, line: u32) -> StackFrame {
        StackFrame {
            function_name: function_name.to_string(),
            file_name: Some("test.rs".to_string()),
            line_number: Some(line),
            is_unsafe: false,
        }
    }

    #[test]
    fn test_call_stack_normalization() {
        let normalizer = CallStackNormalizer::new(NormalizerConfig::default());

        let frames1 = vec![
            create_test_stack_frame("main", 10),
            create_test_stack_frame("foo", 20),
        ];

        let frames2 = vec![
            create_test_stack_frame("main", 10),
            create_test_stack_frame("foo", 20),
        ];

        // First normalization should create new entry
        let id1 = normalizer
            .normalize_call_stack(&frames1)
            .expect("Failed to normalize call stack");
        assert_eq!(id1, 1);

        // Second normalization with same frames should return same ID
        let _id2 = normalizer
            .normalize_call_stack(&frames2)
            .expect("Failed to normalize call stack");
        assert_eq!(id1, _id2);

        // Verify we can retrieve the frames
        let retrieved_frames = normalizer
            .get_call_stack(id1)
            .expect("Failed to get call stack");
        assert_eq!(retrieved_frames.len(), 2);
        assert_eq!(retrieved_frames[0].function_name, "main");
        assert_eq!(retrieved_frames[1].function_name, "foo");
    }

    #[test]
    fn test_reference_counting() {
        let normalizer = CallStackNormalizer::new(NormalizerConfig::default());

        let frames = vec![create_test_stack_frame("test", 1)];
        let id = normalizer
            .normalize_call_stack(&frames)
            .expect("Test operation failed");

        // Increment reference count
        normalizer
            .increment_ref_count(id)
            .expect("Failed to increment ref count");

        // Decrement reference count
        normalizer
            .decrement_ref_count(id)
            .expect("Failed to decrement ref count");

        // Should still be able to access
        let retrieved = normalizer
            .get_call_stack(id)
            .expect("Failed to get call stack");
        assert_eq!(retrieved.len(), 1);
    }

    #[test]
    fn test_call_stack_ref() {
        // Initialize global normalizer for this test
        let config = NormalizerConfig::default();
        let _global_normalizer = initialize_global_call_stack_normalizer(config);

        let frames = vec![
            create_test_stack_frame("main", 10),
            create_test_stack_frame("test", 20),
        ];

        // Use global normalizer to normalize call stack
        let global_normalizer = get_global_call_stack_normalizer();
        let id = global_normalizer
            .normalize_call_stack(&frames)
            .expect("Failed to normalize call stack");
        let stack_ref = CallStackRef::new(id, Some(2));

        assert_eq!(stack_ref.get_depth().expect("Failed to get depth"), 2);

        let retrieved_frames = stack_ref.get_frames().expect("Failed to get frames");
        assert_eq!(retrieved_frames.len(), 2);
        assert_eq!(retrieved_frames[0].function_name, "main");
    }

    #[test]
    fn test_registry_cleanup() {
        let config = NormalizerConfig {
            max_registry_size: 2,
            min_ref_count_for_cleanup: 2,
            ..Default::default()
        };

        let normalizer = CallStackNormalizer::new(config);

        // Create multiple call stacks
        let frames1 = vec![create_test_stack_frame("func1", 1)];
        let frames2 = vec![create_test_stack_frame("func2", 2)];
        let frames3 = vec![create_test_stack_frame("func3", 3)];

        let id1 = normalizer
            .normalize_call_stack(&frames1)
            .expect("Failed to normalize call stack");
        let _id2 = normalizer
            .normalize_call_stack(&frames2)
            .expect("Failed to normalize call stack");

        // Increment ref count for id1 to keep it during cleanup
        normalizer
            .increment_ref_count(id1)
            .expect("Failed to increment ref count");

        // This should trigger cleanup
        let _id3 = normalizer
            .normalize_call_stack(&frames3)
            .expect("Failed to normalize call stack");

        // id1 should still exist (high ref count)
        assert!(normalizer.get_call_stack(id1).is_ok());

        // id2 might be cleaned up (low ref count)
        let stats = normalizer.get_stats();
        assert!(stats.cleanup_operations > 0);
    }

    #[test]
    fn test_statistics_tracking() {
        let normalizer = CallStackNormalizer::new(NormalizerConfig::default());

        let frames1 = vec![create_test_stack_frame("func1", 1)];
        let frames2 = vec![create_test_stack_frame("func1", 1)]; // Duplicate
        let frames3 = vec![create_test_stack_frame("func2", 2)]; // New

        normalizer
            .normalize_call_stack(&frames1)
            .expect("Failed to normalize call stack");
        normalizer
            .normalize_call_stack(&frames2)
            .expect("Failed to normalize call stack"); // Should be duplicate
        normalizer
            .normalize_call_stack(&frames3)
            .expect("Failed to normalize call stack");

        let stats = normalizer.get_stats();
        assert_eq!(stats.total_processed, 3);
        assert_eq!(stats.unique_stacks, 2);
        assert_eq!(stats.duplicates_avoided, 1);
        assert!(stats.memory_saved_bytes > 0);
    }
}
