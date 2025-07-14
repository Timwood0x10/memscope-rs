//! Core types and error handling for the memscope-rs library.

use serde::{Deserialize, Serialize};

/// Error type for memory tracking operations
#[derive(Debug, thiserror::Error)]
pub enum TrackingError {
    /// Failed to acquire a lock
    #[error("Failed to acquire lock: {0}")]
    LockError(String),

    /// Invalid pointer for association
    #[error("Invalid pointer association: {ptr:?}")]
    InvalidPointer {
        /// The invalid pointer address
        ptr: usize,
    },

    /// Allocation tracking is disabled
    #[error("Allocation tracking disabled")]
    TrackingDisabled,

    /// Memory corruption detected
    #[error("Memory corruption detected")]
    MemoryCorruption,

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// IO error during export
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for tracking operations
pub type TrackingResult<T> = Result<T, TrackingError>;

/// Enhanced information about a memory allocation with lifecycle tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationInfo {
    /// Memory address of the allocation
    pub ptr: usize,
    /// Size of the allocation in bytes
    pub size: usize,
    /// Timestamp when the allocation occurred (milliseconds since UNIX_EPOCH)
    pub timestamp_alloc: u128,
    /// Timestamp when the deallocation occurred (if applicable)
    pub timestamp_dealloc: Option<u128>,
    /// Optional name of the variable associated with this allocation
    pub var_name: Option<String>,
    /// Optional type name of the variable associated with this allocation
    pub type_name: Option<String>,
    /// Thread ID where the allocation occurred
    pub thread_id: String,
    /// Backtrace information (if available)
    #[cfg(feature = "backtrace")]
    pub backtrace: Option<Vec<String>>,

    // Enhanced lifecycle tracking fields
    /// Peak memory size reached during lifetime (for growable types)
    pub peak_size: Option<usize>,
    /// Number of memory growth events (reallocations)
    pub growth_events: usize,
    /// Scope identifier where this allocation occurred
    pub scope_name: Option<String>,
    /// Ownership pattern for this allocation
    pub ownership_pattern: Option<OwnershipPattern>,
    /// Risk classification for this allocation
    pub risk_level: Option<RiskLevel>,
    /// Memory efficiency score (useful_bytes / allocated_bytes)
    pub efficiency_score: Option<f64>,
    /// Borrowing events count (how many times this was borrowed)
    pub borrow_count: usize,
    /// Mutable borrowing events count
    pub mut_borrow_count: usize,
    /// Ownership transfer events
    pub transfer_count: usize,
    /// Custom metadata tags
    pub metadata_tags: Vec<String>,
}

impl AllocationInfo {
    /// Create a new allocation info with enhanced lifecycle tracking
    pub fn new(ptr: usize, size: usize) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let thread_id = format!("{:?}", std::thread::current().id());

        Self {
            ptr,
            size,
            timestamp_alloc: timestamp,
            timestamp_dealloc: None,
            var_name: None,
            type_name: None,
            thread_id,
            #[cfg(feature = "backtrace")]
            backtrace: None,

            // Initialize enhanced lifecycle fields
            peak_size: Some(size), // Initially same as size
            growth_events: 0,
            scope_name: None,
            ownership_pattern: None,
            risk_level: None,
            efficiency_score: Some(1.0), // Initially 100% efficient
            borrow_count: 0,
            mut_borrow_count: 0,
            transfer_count: 0,
            metadata_tags: Vec::new(),
        }
    }

    /// Mark this allocation as deallocated
    pub fn mark_deallocated(&mut self) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        self.timestamp_dealloc = Some(timestamp);
    }

    /// Check if this allocation is still active
    pub fn is_active(&self) -> bool {
        self.timestamp_dealloc.is_none()
    }

    /// Get the lifetime of this allocation in milliseconds
    pub fn lifetime_ms(&self) -> Option<u128> {
        self.timestamp_dealloc
            .map(|dealloc| dealloc - self.timestamp_alloc)
    }

    /// Record a memory growth event (reallocation)
    pub fn record_growth(&mut self, new_size: usize) {
        self.growth_events += 1;
        if let Some(peak) = self.peak_size {
            self.peak_size = Some(peak.max(new_size));
        } else {
            self.peak_size = Some(new_size);
        }

        // Update efficiency score
        if let Some(peak) = self.peak_size {
            self.efficiency_score = Some(self.size as f64 / peak as f64);
        }
    }

    /// Record a borrowing event
    pub fn record_borrow(&mut self, is_mutable: bool) {
        if is_mutable {
            self.mut_borrow_count += 1;
        } else {
            self.borrow_count += 1;
        }
    }

    /// Record an ownership transfer
    pub fn record_transfer(&mut self) {
        self.transfer_count += 1;
    }

    /// Add a metadata tag
    pub fn add_metadata_tag(&mut self, tag: String) {
        if !self.metadata_tags.contains(&tag) {
            self.metadata_tags.push(tag);
        }
    }

    /// Calculate memory growth factor
    pub fn memory_growth_factor(&self) -> f64 {
        if let Some(peak) = self.peak_size {
            peak as f64 / self.size.max(1) as f64
        } else {
            1.0
        }
    }

    /// Classify the risk level of this allocation
    pub fn classify_risk(&mut self) {
        let growth_factor = self.memory_growth_factor();
        let lifetime = self.lifetime_ms().unwrap_or(0) as f64;

        self.risk_level = Some(if self.size > 1024 * 1024 || growth_factor > 3.0 {
            RiskLevel::Critical
        } else if self.size > 1024 || growth_factor > 2.0 || lifetime > 10000.0 {
            RiskLevel::High
        } else if self.size > 256 || growth_factor > 1.5 || lifetime > 1000.0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        });
    }

    /// Determine ownership pattern based on type
    pub fn determine_ownership_pattern(&mut self) {
        if let Some(type_name) = &self.type_name {
            self.ownership_pattern =
                Some(if type_name.contains("Rc") || type_name.contains("Arc") {
                    OwnershipPattern::Shared
                } else if type_name.starts_with('&') {
                    OwnershipPattern::Borrowed
                } else if self.transfer_count > 0 && self.borrow_count > 0 {
                    OwnershipPattern::Mixed
                } else {
                    OwnershipPattern::Owned
                });
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryStats {
    /// Total number of allocations tracked
    pub total_allocations: usize,
    /// Total number of deallocations tracked
    pub total_deallocations: usize,
    /// Total bytes allocated
    pub total_allocated: usize,
    /// Total bytes deallocated
    pub total_deallocated: usize,
    /// Current number of active allocations
    pub active_allocations: usize,
    /// Current bytes in active allocations
    pub active_memory: usize,
    /// Peak number of active allocations
    pub peak_allocations: usize,
    /// Peak memory usage in bytes
    pub peak_memory: usize,
    /// Lifecycle statistics
    pub lifecycle_stats: LifecycleStats,
}

/// Memory usage by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMemoryUsage {
    /// The name of the data type
    pub type_name: String,
    /// Total size in bytes for this type
    pub total_size: usize,
    /// Number of allocations for this type
    pub allocation_count: usize,
}

/// Allocation hotspot information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotInfo {
    /// Location identifier (could be function name, file:line, etc.)
    pub location: String,
    /// Number of allocations from this location
    pub count: usize,
    /// Total size of allocations from this location
    pub total_size: usize,
    /// Average allocation size
    pub average_size: f64,
}

/// Enhanced lifecycle statistics for memory allocations per lifecycle.md specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifecycleStats {
    /// Number of completed allocations (with deallocation timestamps)
    pub completed_allocations: usize,
    /// Average lifetime in milliseconds
    pub average_lifetime_ms: f64,
    /// Median lifetime in milliseconds
    pub median_lifetime_ms: f64,
    /// Lifecycle percentiles
    pub lifetime_percentiles: LifecyclePercentiles,
    /// Shortest lifetime in milliseconds
    pub min_lifetime_ms: u128,
    /// Longest lifetime in milliseconds
    pub max_lifetime_ms: u128,
    /// Number of instant allocations (< 1ms)
    pub instant_allocations: usize,
    /// Number of short-term allocations (1ms - 100ms)
    pub short_term_allocations: usize,
    /// Number of medium-term allocations (100ms - 1s)
    pub medium_term_allocations: usize,
    /// Number of long-term allocations (> 1s)
    pub long_term_allocations: usize,
    /// Number of suspected memory leaks (active > 10s)
    pub suspected_leaks: usize,

    // Enhanced metrics per lifecycle.md
    /// Memory growth events (reallocations, expansions)
    pub memory_growth_events: usize,
    /// Peak concurrent variables at any point in time
    pub peak_concurrent_variables: usize,
    /// Memory efficiency ratio (useful_memory / total_allocated)
    pub memory_efficiency_ratio: f64,
    /// Ownership transfer events detected
    pub ownership_transfer_events: usize,
    /// Borrowing relationship violations
    pub borrowing_violations: usize,
    /// Memory fragmentation score (0.0 = perfect, 1.0 = highly fragmented)
    pub fragmentation_score: f64,
    /// Risk classification distribution
    pub risk_distribution: RiskDistribution,
    /// Scope-based lifecycle metrics
    pub scope_metrics: Vec<ScopeLifecycleMetrics>,
    /// Type-specific lifecycle patterns
    pub type_lifecycle_patterns: Vec<TypeLifecyclePattern>,
}

/// Lifecycle percentile statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifecyclePercentiles {
    /// 50th percentile (median)
    pub p50: f64,
    /// 90th percentile
    pub p90: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
}

/// Lifecycle statistics by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeLifecycleStats {
    /// Type name
    pub type_name: String,
    /// Average lifetime for this type
    pub average_lifetime_ms: f64,
    /// Number of allocations for this type
    pub allocation_count: usize,
    /// Lifecycle category
    pub category: LifecycleCategory,
}

/// Categories for lifecycle duration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleCategory {
    /// Very short-lived (< 1ms)
    Instant,
    /// Short-lived (1ms - 100ms)
    ShortTerm,
    /// Medium-lived (100ms - 1s)
    MediumTerm,
    /// Long-lived (> 1s)
    LongTerm,
}

/// Risk classification distribution for memory allocations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskDistribution {
    /// High memory risk allocations (large size or high growth)
    pub high_memory_risk: usize,
    /// Potential growth risk allocations
    pub potential_growth_risk: usize,
    /// Short lifecycle risk allocations
    pub short_lifecycle_risk: usize,
    /// Low risk allocations
    pub low_risk: usize,
    /// Memory leak risk allocations (long-lived without deallocation)
    pub leak_risk: usize,
}

/// Scope-based lifecycle metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeLifecycleMetrics {
    /// Scope identifier (function name, block, etc.)
    pub scope_name: String,
    /// Number of variables in this scope
    pub variable_count: usize,
    /// Average lifetime of variables in this scope
    pub avg_lifetime_ms: f64,
    /// Total memory usage in this scope
    pub total_memory_bytes: usize,
    /// Peak concurrent variables in this scope
    pub peak_concurrent_vars: usize,
    /// Scope efficiency score (0.0 = poor, 1.0 = excellent)
    pub efficiency_score: f64,
}

/// Type-specific lifecycle patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeLifecyclePattern {
    /// Type name (String, Vec, Box, etc.)
    pub type_name: String,
    /// Average allocation count per variable of this type
    pub avg_allocations_per_var: f64,
    /// Memory growth factor (peak_size / initial_size)
    pub memory_growth_factor: f64,
    /// Typical lifetime range for this type
    pub typical_lifetime_range: (u64, u64), // (min_ms, max_ms)
    /// Ownership pattern (owned, borrowed, shared)
    pub ownership_pattern: OwnershipPattern,
    /// Risk level for this type
    pub risk_level: RiskLevel,
}

/// Ownership patterns for variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnershipPattern {
    /// Exclusively owned (Box, Vec, String)
    Owned,
    /// Shared ownership (Rc, Arc)
    Shared,
    /// Borrowed references (&T, &mut T)
    Borrowed,
    /// Mixed ownership patterns
    Mixed,
}

/// Risk levels for memory allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk - small, predictable allocations
    Low,
    /// Medium risk - moderate size or some growth
    Medium,
    /// High risk - large allocations or significant growth
    High,
    /// Critical risk - potential memory leaks or excessive growth
    Critical,
}
