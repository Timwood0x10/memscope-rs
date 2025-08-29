//! Bounded memory statistics to prevent infinite growth
//!
//! This module provides memory statistics structures that use bounded containers
//! to prevent memory leaks during long-running applications.

use crate::core::types::{
    AllocationInfo, ConcurrencyAnalysis, FragmentationAnalysis, ScopeLifecycleMetrics,
    SystemLibraryStats,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Configuration for bounded memory statistics
#[derive(Debug, Clone)]
pub struct BoundedStatsConfig {
    /// Maximum number of recent allocations to keep in memory
    pub max_recent_allocations: usize,
    /// Maximum number of historical summaries to keep
    pub max_historical_summaries: usize,
    /// Enable automatic cleanup when limits are reached
    pub enable_auto_cleanup: bool,
    /// Cleanup threshold (percentage of max before cleanup)
    pub cleanup_threshold: f32,
}

impl Default for BoundedStatsConfig {
    fn default() -> Self {
        Self {
            max_recent_allocations: 10_000,  // Keep last 10k allocations
            max_historical_summaries: 1_000, // Keep 1k historical summaries
            enable_auto_cleanup: true,
            cleanup_threshold: 0.9, // Cleanup when 90% full
        }
    }
}

/// Lightweight allocation summary for bounded storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationSummary {
    pub ptr: usize,
    pub size: usize,
    pub timestamp_alloc: u64,
    pub timestamp_dealloc: Option<u64>,
    pub type_name: Option<String>,
    pub var_name: Option<String>,
    pub is_leaked: bool,
    pub lifetime_ms: Option<u64>,
}

impl From<&AllocationInfo> for AllocationSummary {
    fn from(alloc: &AllocationInfo) -> Self {
        Self {
            ptr: alloc.ptr,
            size: alloc.size,
            timestamp_alloc: alloc.timestamp_alloc,
            timestamp_dealloc: alloc.timestamp_dealloc,
            type_name: alloc.type_name.clone(),
            var_name: alloc.var_name.clone(),
            is_leaked: alloc.is_leaked,
            lifetime_ms: alloc.lifetime_ms,
        }
    }
}

/// Bounded memory statistics that prevent infinite growth
#[derive(Debug, Clone, Serialize)]
pub struct BoundedMemoryStats {
    /// Configuration for this instance
    #[serde(skip)]
    pub config: BoundedStatsConfig,

    /// Basic statistics (these don't grow infinitely)
    pub total_allocations: usize,
    pub total_allocated: usize,
    pub active_allocations: usize,
    pub active_memory: usize,
    pub peak_allocations: usize,
    pub peak_memory: usize,
    pub total_deallocations: usize,
    pub total_deallocated: usize,
    pub leaked_allocations: usize,
    pub leaked_memory: usize,

    /// Analysis data (bounded)
    pub fragmentation_analysis: FragmentationAnalysis,
    pub lifecycle_stats: ScopeLifecycleMetrics,
    pub system_library_stats: SystemLibraryStats,
    pub concurrency_analysis: ConcurrencyAnalysis,

    /// Bounded containers for detailed data
    pub recent_allocations: VecDeque<AllocationSummary>,
    pub historical_summaries: VecDeque<HistoricalSummary>,

    /// Cleanup statistics
    pub cleanup_count: u32,
    pub last_cleanup_timestamp: Option<u64>,
    pub total_allocations_processed: u64,
}

/// Historical summary for long-term trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalSummary {
    pub timestamp: u64,
    pub total_allocations: usize,
    pub total_memory: usize,
    pub active_allocations: usize,
    pub active_memory: usize,
    pub allocation_rate: f64,    // allocations per second
    pub memory_growth_rate: f64, // bytes per second
}

impl BoundedMemoryStats {
    /// Create new bounded memory statistics with default configuration
    pub fn new() -> Self {
        Self::with_config(BoundedStatsConfig::default())
    }

    /// Create new bounded memory statistics with custom configuration
    pub fn with_config(config: BoundedStatsConfig) -> Self {
        Self {
            config,
            total_allocations: 0,
            total_allocated: 0,
            active_allocations: 0,
            active_memory: 0,
            peak_allocations: 0,
            peak_memory: 0,
            total_deallocations: 0,
            total_deallocated: 0,
            leaked_allocations: 0,
            leaked_memory: 0,
            fragmentation_analysis: FragmentationAnalysis::default(),
            lifecycle_stats: ScopeLifecycleMetrics {
                scope_name: "global".to_string(),
                variable_count: 0,
                average_lifetime_ms: 0.0,
                total_memory_usage: 0,
                peak_memory_usage: 0,
                allocation_frequency: 0.0,
                deallocation_efficiency: 0.0,
                completed_allocations: 0,
                memory_growth_events: 0,
                peak_concurrent_variables: 0,
                memory_efficiency_ratio: 0.0,
                ownership_transfer_events: 0,
                fragmentation_score: 0.0,
                instant_allocations: 0,
                short_term_allocations: 0,
                medium_term_allocations: 0,
                long_term_allocations: 0,
                suspected_leaks: 0,
                risk_distribution: crate::core::types::RiskDistribution::default(),
                scope_metrics: Vec::new(),
                type_lifecycle_patterns: Vec::new(),
            },
            system_library_stats: SystemLibraryStats::default(),
            concurrency_analysis: ConcurrencyAnalysis::default(),
            recent_allocations: VecDeque::with_capacity(1000),
            historical_summaries: VecDeque::with_capacity(100),
            cleanup_count: 0,
            last_cleanup_timestamp: None,
            total_allocations_processed: 0,
        }
    }

    /// Add a new allocation, automatically managing bounds
    pub fn add_allocation(&mut self, alloc: &AllocationInfo) {
        // CRITICAL FIX: Only process user variables to prevent system allocation overload
        // This is the key fix - only track allocations with variable names (user variables)
        if alloc.var_name.is_none() {
            // Skip system allocations without variable names
            // This prevents the 736 system allocations from overwhelming the bounded stats
            return;
        }

        // Update basic statistics
        self.total_allocations += 1;
        self.total_allocated += alloc.size;
        self.active_allocations += 1;
        self.active_memory += alloc.size;
        self.total_allocations_processed += 1;

        // Update peaks
        if self.active_allocations > self.peak_allocations {
            self.peak_allocations = self.active_allocations;
        }
        if self.active_memory > self.peak_memory {
            self.peak_memory = self.active_memory;
        }

        // Add to recent allocations with bounds checking
        let summary = AllocationSummary::from(alloc);
        self.add_allocation_summary(summary);

        // Check if cleanup is needed
        if self.config.enable_auto_cleanup {
            self.check_and_cleanup();
        }
    }

    /// Record a deallocation
    pub fn record_deallocation(&mut self, ptr: usize, size: usize) {
        self.total_deallocations += 1;
        self.total_deallocated += size;
        self.active_allocations = self.active_allocations.saturating_sub(1);
        self.active_memory = self.active_memory.saturating_sub(size);

        // Update the corresponding allocation summary if found
        let current_timestamp = self.get_current_timestamp();
        if let Some(summary) = self.recent_allocations.iter_mut().find(|s| s.ptr == ptr) {
            summary.timestamp_dealloc = Some(current_timestamp);
            if let Some(alloc_time) = summary.timestamp_dealloc {
                summary.lifetime_ms = Some((alloc_time - summary.timestamp_alloc) / 1_000_000);
            }
        }
    }

    /// Record a memory leak
    pub fn record_leak(&mut self, size: usize) {
        self.leaked_allocations += 1;
        self.leaked_memory += size;
    }

    /// Add allocation summary with bounds management
    fn add_allocation_summary(&mut self, summary: AllocationSummary) {
        // Check if we need to make room
        if self.recent_allocations.len() >= self.config.max_recent_allocations {
            // Remove oldest allocation
            if let Some(old_summary) = self.recent_allocations.pop_front() {
                // Optionally create historical summary from removed data
                self.maybe_create_historical_summary(&old_summary);
            }
        }

        self.recent_allocations.push_back(summary);
    }

    /// Create historical summary from removed allocation data
    fn maybe_create_historical_summary(&mut self, _removed_summary: &AllocationSummary) {
        // Create historical summary periodically (e.g., every 1000 allocations)
        if self.total_allocations % 1000 == 0 {
            let summary = HistoricalSummary {
                timestamp: self.get_current_timestamp(),
                total_allocations: self.total_allocations,
                total_memory: self.total_allocated,
                active_allocations: self.active_allocations,
                active_memory: self.active_memory,
                allocation_rate: self.calculate_allocation_rate(),
                memory_growth_rate: self.calculate_memory_growth_rate(),
            };

            // Add to historical summaries with bounds checking
            if self.historical_summaries.len() >= self.config.max_historical_summaries {
                self.historical_summaries.pop_front();
            }
            self.historical_summaries.push_back(summary);
        }
    }

    /// Check if cleanup is needed and perform it
    fn check_and_cleanup(&mut self) {
        let recent_threshold =
            (self.config.max_recent_allocations as f32 * self.config.cleanup_threshold) as usize;
        let historical_threshold =
            (self.config.max_historical_summaries as f32 * self.config.cleanup_threshold) as usize;

        let mut cleaned = false;

        // Cleanup recent allocations if needed
        if self.recent_allocations.len() >= recent_threshold {
            let remove_count = self.recent_allocations.len() / 4; // Remove 25%
            for _ in 0..remove_count {
                if let Some(old_summary) = self.recent_allocations.pop_front() {
                    self.maybe_create_historical_summary(&old_summary);
                }
            }
            cleaned = true;
        }

        // Cleanup historical summaries if needed
        if self.historical_summaries.len() >= historical_threshold {
            let remove_count = self.historical_summaries.len() / 4; // Remove 25%
            for _ in 0..remove_count {
                self.historical_summaries.pop_front();
            }
            cleaned = true;
        }

        if cleaned {
            self.cleanup_count += 1;
            self.last_cleanup_timestamp = Some(self.get_current_timestamp());
        }
    }

    /// Get current timestamp in nanoseconds
    fn get_current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    /// Calculate allocation rate (allocations per second)
    fn calculate_allocation_rate(&self) -> f64 {
        if let Some(oldest) = self.recent_allocations.front() {
            let time_span = self.get_current_timestamp() - oldest.timestamp_alloc;
            if time_span > 0 {
                let seconds = time_span as f64 / 1_000_000_000.0;
                return self.recent_allocations.len() as f64 / seconds;
            }
        }
        0.0
    }

    /// Calculate memory growth rate (bytes per second)
    fn calculate_memory_growth_rate(&self) -> f64 {
        if let Some(oldest) = self.historical_summaries.front() {
            let time_span = self.get_current_timestamp() - oldest.timestamp;
            if time_span > 0 {
                let seconds = time_span as f64 / 1_000_000_000.0;
                let memory_growth = self.active_memory as i64 - oldest.active_memory as i64;
                return memory_growth as f64 / seconds;
            }
        }
        0.0
    }

    /// Get memory usage statistics for this stats instance
    pub fn get_memory_usage(&self) -> MemoryUsageStats {
        let recent_allocations_size =
            self.recent_allocations.len() * std::mem::size_of::<AllocationSummary>();
        let historical_summaries_size =
            self.historical_summaries.len() * std::mem::size_of::<HistoricalSummary>();
        let base_size = std::mem::size_of::<Self>();

        MemoryUsageStats {
            total_size: base_size + recent_allocations_size + historical_summaries_size,
            recent_allocations_size,
            historical_summaries_size,
            base_size,
            recent_allocations_count: self.recent_allocations.len(),
            historical_summaries_count: self.historical_summaries.len(),
        }
    }

    /// Force cleanup of old data
    pub fn force_cleanup(&mut self) {
        let old_cleanup_threshold = self.config.cleanup_threshold;
        self.config.cleanup_threshold = 0.5; // Force more aggressive cleanup
        self.check_and_cleanup();
        self.config.cleanup_threshold = old_cleanup_threshold;
    }

    /// Get all allocations as a Vec (for compatibility with existing code)
    pub fn get_all_allocations(&self) -> Vec<AllocationInfo> {
        self.recent_allocations
            .iter()
            .map(|summary| {
                // Convert summary back to AllocationInfo for compatibility
                AllocationInfo {
                    ptr: summary.ptr,
                    size: summary.size,
                    var_name: summary.var_name.clone(),
                    type_name: summary.type_name.clone(),
                    scope_name: Some("tracked".to_string()),
                    timestamp_alloc: summary.timestamp_alloc,
                    timestamp_dealloc: summary.timestamp_dealloc,
                    thread_id: "main".to_string(), // Default thread ID
                    borrow_count: 0,
                    stack_trace: None,
                    is_leaked: summary.is_leaked,
                    lifetime_ms: summary.lifetime_ms,
                    borrow_info: None,
                    clone_info: None,
                    ownership_history_available: false,
                    // Set other fields to default/None for compatibility
                    smart_pointer_info: None,
                    memory_layout: None,
                    generic_info: None,
                    dynamic_type_info: None,
                    runtime_state: None,
                    stack_allocation: None,
                    temporary_object: None,
                    fragmentation_analysis: None,
                    generic_instantiation: None,
                    type_relationships: None,
                    type_usage: None,
                    function_call_tracking: None,
                    lifecycle_tracking: None,
                    access_tracking: None,
                    drop_chain_analysis: None,
                }
            })
            .collect()
    }
}

impl Default for BoundedMemoryStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory usage statistics for the stats instance itself
#[derive(Debug, Clone, Serialize)]
pub struct MemoryUsageStats {
    pub total_size: usize,
    pub recent_allocations_size: usize,
    pub historical_summaries_size: usize,
    pub base_size: usize,
    pub recent_allocations_count: usize,
    pub historical_summaries_count: usize,
}

/// Allocation history manager for separate storage of detailed history
pub struct AllocationHistoryManager {
    /// Configuration
    config: BoundedStatsConfig,
    /// Detailed allocation history (bounded)
    history: VecDeque<AllocationInfo>,
    /// History cleanup statistics
    cleanup_count: u32,
    last_cleanup_timestamp: Option<u64>,
}

impl AllocationHistoryManager {
    /// Create new history manager
    pub fn new() -> Self {
        Self::with_config(BoundedStatsConfig::default())
    }

    /// Create new history manager with custom configuration
    pub fn with_config(config: BoundedStatsConfig) -> Self {
        Self {
            config,
            history: VecDeque::with_capacity(1000),
            cleanup_count: 0,
            last_cleanup_timestamp: None,
        }
    }

    /// Add allocation to history
    pub fn add_allocation(&mut self, alloc: AllocationInfo) {
        // Check bounds and cleanup if needed
        if self.history.len() >= self.config.max_recent_allocations {
            let remove_count = self.history.len() / 4; // Remove 25%
            for _ in 0..remove_count {
                self.history.pop_front();
            }
            self.cleanup_count += 1;
            self.last_cleanup_timestamp = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
            );
        }

        self.history.push_back(alloc);
    }

    /// Get all history entries
    pub fn get_history(&self) -> &VecDeque<AllocationInfo> {
        &self.history
    }

    /// Get history as Vec for compatibility
    pub fn get_history_vec(&self) -> Vec<AllocationInfo> {
        self.history.iter().cloned().collect()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// Get memory usage of this history manager
    pub fn get_memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + self.history.len() * std::mem::size_of::<AllocationInfo>()
    }
}

impl Default for AllocationHistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AllocationInfo;

    fn create_test_allocation(id: usize) -> AllocationInfo {
        AllocationInfo {
            ptr: 0x1000 + id,
            size: 64 + (id % 100),
            var_name: Some(format!("var_{}", id)),
            type_name: Some("TestType".to_string()),
            scope_name: Some("test".to_string()),
            timestamp_alloc: id as u64 * 1000,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        }
    }

    #[test]
    fn test_bounded_memory_stats_no_overflow() {
        let mut stats = BoundedMemoryStats::with_config(BoundedStatsConfig {
            max_recent_allocations: 100,
            max_historical_summaries: 10,
            enable_auto_cleanup: true,
            cleanup_threshold: 0.9,
        });

        // Add 150 allocations
        for i in 0..150 {
            let alloc = create_test_allocation(i);
            stats.add_allocation(&alloc);
        }

        // Verify bounds are respected
        assert!(stats.recent_allocations.len() <= 100);
        assert_eq!(stats.total_allocations, 150);
        assert!(stats.cleanup_count > 0);
    }

    #[test]
    fn test_allocation_history_manager_bounds() {
        let mut manager = AllocationHistoryManager::with_config(BoundedStatsConfig {
            max_recent_allocations: 50,
            ..Default::default()
        });

        // Add 100 allocations
        for i in 0..100 {
            let alloc = create_test_allocation(i);
            manager.add_allocation(alloc);
        }

        // Verify bounds are respected
        assert!(manager.history.len() <= 50);
        assert!(manager.cleanup_count > 0);
    }

    #[test]
    fn test_memory_usage_calculation() {
        let stats = BoundedMemoryStats::new();
        let usage = stats.get_memory_usage();

        assert!(usage.total_size > 0);
        assert_eq!(usage.recent_allocations_count, 0);
        assert_eq!(usage.historical_summaries_count, 0);
    }

    #[test]
    fn test_deallocation_tracking() {
        let mut stats = BoundedMemoryStats::new();
        let alloc = create_test_allocation(1);

        stats.add_allocation(&alloc);
        assert_eq!(stats.active_allocations, 1);
        assert_eq!(stats.active_memory, alloc.size);

        stats.record_deallocation(alloc.ptr, alloc.size);
        assert_eq!(stats.active_allocations, 0);
        assert_eq!(stats.active_memory, 0);
        assert_eq!(stats.total_deallocations, 1);
    }

    #[test]
    fn test_leak_tracking() {
        let mut stats = BoundedMemoryStats::new();

        stats.record_leak(1024);
        assert_eq!(stats.leaked_allocations, 1);
        assert_eq!(stats.leaked_memory, 1024);
    }
}
