//! data localizer - reduce global state access overhead
//!
//! This module implements data localization functionality,
//! fetching all export data at once to avoid repeated access to global state,
//! thus significantly improving export performance.

use crate::analysis::unsafe_ffi_tracker::{
    get_global_unsafe_ffi_tracker, EnhancedAllocationInfo, UnsafeFFIStats,
};
use crate::core::scope_tracker::get_global_scope_tracker;
use crate::core::tracker::get_global_tracker;
use crate::core::types::MemoryStats;
use crate::core::types::ScopeInfo;
use crate::core::types::{AllocationInfo, TrackingError, TrackingResult};
use std::time::{Duration, Instant};

/// data localizer - fetch all export data at once to avoid repeated access to global state
pub struct DataLocalizer {
    /// cached basic allocation data
    cached_allocations: Option<Vec<AllocationInfo>>,
    /// cached FFI enhanced data
    cached_ffi_data: Option<Vec<EnhancedAllocationInfo>>,
    /// cached stats
    cached_stats: Option<MemoryStats>,
    /// cached FFI stats
    cached_ffi_stats: Option<UnsafeFFIStats>,
    /// cached scope info
    cached_scope_info: Option<Vec<ScopeInfo>>,
    /// last update time
    last_update: Instant,
    /// cache ttl
    cache_ttl: Duration,
}

/// localized export data, containing all necessary information
#[derive(Debug, Clone)]
pub struct LocalizedExportData {
    /// basic memory allocation info
    pub allocations: Vec<AllocationInfo>,
    /// FFI enhanced allocation info
    pub enhanced_allocations: Vec<EnhancedAllocationInfo>,
    /// stats
    pub stats: MemoryStats,
    /// FFI stats
    pub ffi_stats: UnsafeFFIStats,
    /// scope info
    pub scope_info: Vec<ScopeInfo>,
    /// timestamp
    pub timestamp: Instant,
}

/// data gathering stats
#[derive(Debug, Clone)]
pub struct DataGatheringStats {
    /// total time ms
    pub total_time_ms: u64,
    /// basic data time ms
    pub basic_data_time_ms: u64,
    /// FFI data time ms
    pub ffi_data_time_ms: u64,
    /// scope data time ms
    pub scope_data_time_ms: u64,
    /// allocation count
    pub allocation_count: usize,
    /// ffi allocation count
    pub ffi_allocation_count: usize,
    /// scope count
    pub scope_count: usize,
}

impl DataLocalizer {
    /// create new data localizer
    pub fn new() -> Self {
        Self {
            cached_allocations: None,
            cached_ffi_data: None,
            cached_stats: None,
            cached_ffi_stats: None,
            cached_scope_info: None,
            last_update: Instant::now(),
            cache_ttl: Duration::from_millis(100), // 100ms cache, avoid too frequent data fetching
        }
    }

    /// create data localizer with custom cache ttl
    pub fn with_cache_ttl(cache_ttl: Duration) -> Self {
        Self {
            cached_allocations: None,
            cached_ffi_data: None,
            cached_stats: None,
            cached_ffi_stats: None,
            cached_scope_info: None,
            last_update: Instant::now(),
            cache_ttl,
        }
    }

    /// gather all export data at once to avoid repeated access to global state
    pub fn gather_all_export_data(
        &mut self,
    ) -> TrackingResult<(LocalizedExportData, DataGatheringStats)> {
        let total_start = Instant::now();

        tracing::info!("🔄 start data localization to reduce global state access...");

        // check if cache is still valid
        if self.is_cache_valid() {
            tracing::info!("✅ using cached data, skipping repeated fetching");
            return self.get_cached_data();
        }

        // step 1: get basic memory tracking data
        let basic_start = Instant::now();
        let tracker = get_global_tracker();
        let allocations = tracker.get_active_allocations().map_err(|e| {
            TrackingError::ExportError(format!("get active allocations failed: {}", e))
        })?;
        let stats = tracker
            .get_stats()
            .map_err(|e| TrackingError::ExportError(format!("get stats failed: {}", e)))?;
        let basic_time = basic_start.elapsed();

        // step 2: get ffi related data
        let ffi_start = Instant::now();
        let ffi_tracker = get_global_unsafe_ffi_tracker();
        let enhanced_allocations = ffi_tracker.get_enhanced_allocations().unwrap_or_else(|e| {
            tracing::error!(
                "s⚠️ get enhanced allocations failed: {}, using empty data",
                e
            );
            Vec::new()
        });
        let ffi_stats = ffi_tracker.get_stats();
        let ffi_time = ffi_start.elapsed();

        // step 3: get scope data
        let scope_start = Instant::now();
        let scope_tracker = get_global_scope_tracker();
        let scope_info = scope_tracker.get_all_scopes();
        let scope_time = scope_start.elapsed();

        let total_time = total_start.elapsed();

        // update cache
        self.cached_allocations = Some(allocations.clone());
        self.cached_ffi_data = Some(enhanced_allocations.clone());
        self.cached_stats = Some(stats.clone());
        self.cached_ffi_stats = Some(ffi_stats.clone());
        self.cached_scope_info = Some(scope_info.clone());
        self.last_update = Instant::now();

        let localized_data = LocalizedExportData {
            allocations: allocations.clone(),
            enhanced_allocations: enhanced_allocations.clone(),
            stats,
            ffi_stats,
            scope_info: scope_info.clone(),
            timestamp: total_start,
        };

        let gathering_stats = DataGatheringStats {
            total_time_ms: total_time.as_millis() as u64,
            basic_data_time_ms: basic_time.as_millis() as u64,
            ffi_data_time_ms: ffi_time.as_millis() as u64,
            scope_data_time_ms: scope_time.as_millis() as u64,
            allocation_count: allocations.len(),
            ffi_allocation_count: enhanced_allocations.len(),
            scope_count: scope_info.len(),
        };

        // print performance stats
        tracing::info!("✅ data localization completed:");
        tracing::info!("   total time: {:?}", total_time);
        tracing::info!(
            "   basic data: {:?} ({} allocations)",
            basic_time,
            gathering_stats.allocation_count
        );
        tracing::info!(
            "   ffi data: {:?} ({} enhanced allocations)",
            ffi_time,
            gathering_stats.ffi_allocation_count
        );
        tracing::info!(
            "   scope data: {:?} ({} scopes)",
            scope_time,
            gathering_stats.scope_count
        );
        tracing::info!(
            "   data localization avoided {} global state accesses",
            self.estimate_avoided_global_accesses(&gathering_stats)
        );

        Ok((localized_data, gathering_stats))
    }

    /// refresh cache and gather all export data
    pub fn refresh_cache(&mut self) -> TrackingResult<(LocalizedExportData, DataGatheringStats)> {
        self.invalidate_cache();
        self.gather_all_export_data()
    }

    /// check if cache is still valid
    fn is_cache_valid(&self) -> bool {
        self.cached_allocations.is_some()
            && self.cached_ffi_data.is_some()
            && self.cached_stats.is_some()
            && self.cached_ffi_stats.is_some()
            && self.cached_scope_info.is_some()
            && self.last_update.elapsed() < self.cache_ttl
    }

    /// get cached data
    fn get_cached_data(&self) -> TrackingResult<(LocalizedExportData, DataGatheringStats)> {
        let localized_data = LocalizedExportData {
            allocations: self.cached_allocations.as_ref().unwrap().clone(),
            enhanced_allocations: self.cached_ffi_data.as_ref().unwrap().clone(),
            stats: self.cached_stats.as_ref().unwrap().clone(),
            ffi_stats: self.cached_ffi_stats.as_ref().unwrap().clone(),
            scope_info: self.cached_scope_info.as_ref().unwrap().clone(),
            timestamp: self.last_update,
        };

        let gathering_stats = DataGatheringStats {
            total_time_ms: 0, // cache hit, no time
            basic_data_time_ms: 0,
            ffi_data_time_ms: 0,
            scope_data_time_ms: 0,
            allocation_count: localized_data.allocations.len(),
            ffi_allocation_count: localized_data.enhanced_allocations.len(),
            scope_count: localized_data.scope_info.len(),
        };

        Ok((localized_data, gathering_stats))
    }

    /// invalidate cache
    pub fn invalidate_cache(&mut self) {
        self.cached_allocations = None;
        self.cached_ffi_data = None;
        self.cached_stats = None;
        self.cached_ffi_stats = None;
        self.cached_scope_info = None;
    }

    /// estimate avoided global accesses
    fn estimate_avoided_global_accesses(&self, stats: &DataGatheringStats) -> usize {
        // In the traditional export process, each allocation may need multiple accesses to global state
        // Here we estimate how many accesses we avoided through data localization
        let basic_accesses = stats.allocation_count * 2; // Each allocation needs to access tracker 2 times
        let ffi_accesses = stats.ffi_allocation_count * 3; // FFI allocations need more accesses
        let scope_accesses = stats.scope_count * 1; // scope access

        basic_accesses + ffi_accesses + scope_accesses
    }

    /// get cache stats
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            is_cached: self.is_cache_valid(),
            cache_age_ms: self.last_update.elapsed().as_millis() as u64,
            cache_ttl_ms: self.cache_ttl.as_millis() as u64,
            cached_allocation_count: self
                .cached_allocations
                .as_ref()
                .map(|v| v.len())
                .unwrap_or(0),
            cached_ffi_count: self.cached_ffi_data.as_ref().map(|v| v.len()).unwrap_or(0),
            cached_scope_count: self
                .cached_scope_info
                .as_ref()
                .map(|v| v.len())
                .unwrap_or(0),
        }
    }
}

/// cache stats
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// whether there is valid cache
    pub is_cached: bool,
    /// cache age (milliseconds)
    pub cache_age_ms: u64,
    /// cache ttl (milliseconds)
    pub cache_ttl_ms: u64,
    /// cached allocation count
    pub cached_allocation_count: usize,
    /// cached ffi count
    pub cached_ffi_count: usize,
    /// cached scope count
    pub cached_scope_count: usize,
}

impl Default for DataLocalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalizedExportData {
    /// get data age
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }

    /// check if data is still fresh
    pub fn is_fresh(&self, max_age: Duration) -> bool {
        self.age() < max_age
    }

    /// get total allocation count (basic + ffi)
    pub fn total_allocation_count(&self) -> usize {
        self.allocations.len() + self.enhanced_allocations.len()
    }

    /// get data summary
    pub fn get_summary(&self) -> String {
        format!(
            "LocalizedExportData {{ allocations: {}, ffi_allocations: {}, scopes: {}, age: {:?} }}",
            self.allocations.len(),
            self.enhanced_allocations.len(),
            self.scope_info.len(),
            self.age()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_localizer_creation() {
        let localizer = DataLocalizer::new();
        assert!(!localizer.is_cache_valid());

        let cache_stats = localizer.get_cache_stats();
        assert!(!cache_stats.is_cached);
        assert_eq!(cache_stats.cached_allocation_count, 0);
    }

    #[test]
    fn test_cache_ttl() {
        let short_ttl = Duration::from_millis(1);
        let mut localizer = DataLocalizer::with_cache_ttl(short_ttl);

        // simulate cached data
        localizer.cached_allocations = Some(vec![]);
        localizer.cached_ffi_data = Some(vec![]);
        localizer.cached_stats = Some(MemoryStats::default());
        localizer.cached_ffi_stats = Some(UnsafeFFIStats::default());
        localizer.cached_scope_info = Some(vec![]);
        localizer.last_update = Instant::now();

        assert!(localizer.is_cache_valid());

        // wait for cache to expire
        std::thread::sleep(Duration::from_millis(2));
        assert!(!localizer.is_cache_valid());
    }

    #[test]
    fn test_localized_export_data() {
        let data = LocalizedExportData {
            allocations: vec![],
            enhanced_allocations: vec![],
            stats: MemoryStats::default(),
            ffi_stats: UnsafeFFIStats::default(),
            scope_info: vec![],
            timestamp: Instant::now(),
        };

        assert_eq!(data.total_allocation_count(), 0);
        assert!(data.is_fresh(Duration::from_secs(1)));

        let summary = data.get_summary();
        assert!(summary.contains("allocations: 0"));
    }
}
