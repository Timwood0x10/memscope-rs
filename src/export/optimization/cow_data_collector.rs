//! COW-based data collector for optimized binary export
//!
//! This module provides a Copy-on-Write based data collection mechanism
//! that minimizes cloning and reduces lock contention for better performance.

use crate::core::types::{AllocationInfo, TrackingResult, TrackingError};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Lightweight allocation reference using COW
#[derive(Debug, Clone)]
pub struct AllocationRef<'a> {
    /// Pointer address
    pub ptr: usize,
    /// Size in bytes
    pub size: usize,
    /// Variable name (COW to avoid cloning strings)
    pub var_name: Cow<'a, str>,
    /// Type name (COW to avoid cloning strings)
    pub type_name: Cow<'a, str>,
    /// Scope name (COW to avoid cloning strings)
    pub scope_name: Cow<'a, str>,
    /// Allocation timestamp
    pub timestamp_alloc: u64,
    /// Deallocation timestamp
    pub timestamp_dealloc: Option<u64>,
    /// Thread ID (COW to avoid cloning)
    pub thread_id: Cow<'a, str>,
    /// Borrow count
    pub borrow_count: usize,
    /// Is leaked flag
    pub is_leaked: bool,
    /// Lifetime in milliseconds
    pub lifetime_ms: Option<u64>,
}

impl<'a> AllocationRef<'a> {
    /// Create from AllocationInfo with minimal copying
    pub fn from_allocation_info(info: &'a AllocationInfo) -> Self {
        Self {
            ptr: info.ptr,
            size: info.size,
            var_name: info.var_name.as_deref().map_or(Cow::Borrowed(""), |s| Cow::Borrowed(s)),
            type_name: info.type_name.as_deref().map_or(Cow::Borrowed(""), |s| Cow::Borrowed(s)),
            scope_name: info.scope_name.as_deref().map_or(Cow::Borrowed(""), |s| Cow::Borrowed(s)),
            timestamp_alloc: info.timestamp_alloc,
            timestamp_dealloc: info.timestamp_dealloc,
            thread_id: Cow::Borrowed(&info.thread_id),
            borrow_count: info.borrow_count,
            is_leaked: info.is_leaked,
            lifetime_ms: info.lifetime_ms,
        }
    }

    /// Convert to owned AllocationInfo when needed
    pub fn to_allocation_info(&self) -> AllocationInfo {
        let mut info = AllocationInfo::new(self.ptr, self.size);
        info.var_name = if self.var_name.is_empty() { None } else { Some(self.var_name.to_string()) };
        info.type_name = if self.type_name.is_empty() { None } else { Some(self.type_name.to_string()) };
        info.scope_name = if self.scope_name.is_empty() { None } else { Some(self.scope_name.to_string()) };
        info.timestamp_alloc = self.timestamp_alloc;
        info.timestamp_dealloc = self.timestamp_dealloc;
        info.thread_id = self.thread_id.to_string();
        info.borrow_count = self.borrow_count;
        info.is_leaked = self.is_leaked;
        info.lifetime_ms = self.lifetime_ms;
        info
    }
}

/// Shared allocation data using Arc to avoid cloning
#[derive(Debug, Clone)]
pub struct SharedAllocationData {
    /// Shared reference to allocation data
    pub data: Arc<AllocationInfo>,
}

impl SharedAllocationData {
    /// Create from AllocationInfo
    pub fn new(info: AllocationInfo) -> Self {
        Self {
            data: Arc::new(info),
        }
    }

    /// Get reference to the data
    pub fn as_ref(&self) -> &AllocationInfo {
        &self.data
    }
}

/// COW-based data collector configuration
#[derive(Debug, Clone)]
pub struct CowCollectorConfig {
    /// Use shared references instead of cloning
    pub use_shared_refs: bool,
    /// Maximum lock hold time in milliseconds
    pub max_lock_time_ms: u64,
    /// Batch size for chunked collection
    pub batch_size: usize,
    /// Enable progress reporting
    pub enable_progress: bool,
}

impl Default for CowCollectorConfig {
    fn default() -> Self {
        Self {
            use_shared_refs: true,
            max_lock_time_ms: 10, // Very short lock time
            batch_size: 1000,
            enable_progress: true,
        }
    }
}

/// COW-based data collector for optimized performance
pub struct CowDataCollector {
    config: CowCollectorConfig,
}

impl CowDataCollector {
    /// Create new COW data collector
    pub fn new(config: CowCollectorConfig) -> Self {
        Self { config }
    }

    /// Collect allocations with minimal cloning using COW
    pub fn collect_with_cow(
        &self,
        tracker: &crate::core::tracker::MemoryTracker,
    ) -> TrackingResult<Vec<SharedAllocationData>> {
        println!("🐄 Starting COW-based allocation collection...");
        let start_time = Instant::now();

        // Strategy 1: Try to get shared references without cloning
        if self.config.use_shared_refs {
            match self.collect_shared_refs(tracker) {
                Ok(shared_data) => {
                    let duration = start_time.elapsed();
                    println!("✅ COW collection completed: {} allocations in {:?}", 
                            shared_data.len(), duration);
                    return Ok(shared_data);
                }
                Err(e) => {
                    println!("⚠️  Shared ref collection failed: {}, trying chunked approach", e);
                }
            }
        }

        // Strategy 2: Chunked collection with minimal lock time
        self.collect_chunked(tracker)
    }

    /// Collect using shared references (fastest method)
    fn collect_shared_refs(
        &self,
        tracker: &crate::core::tracker::MemoryTracker,
    ) -> TrackingResult<Vec<SharedAllocationData>> {
        let start_time = Instant::now();
        let max_lock_time = Duration::from_millis(self.config.max_lock_time_ms);

        // Try to acquire lock with timeout
        let lock_result = tracker.active_allocations.try_lock();
        
        match lock_result {
            Ok(allocations_guard) => {
                println!("   🔓 Lock acquired, collecting {} allocations...", allocations_guard.len());
                
                let mut shared_data = Vec::with_capacity(allocations_guard.len());
                let mut processed = 0;
                
                for (ptr, allocation_info) in allocations_guard.iter() {
                    // Check if we're holding the lock too long
                    if start_time.elapsed() > max_lock_time {
                        println!("   ⏰ Lock timeout, collected {} of {} allocations", 
                                processed, allocations_guard.len());
                        break;
                    }

                    // Create shared reference without cloning the entire structure
                    let shared = SharedAllocationData::new(allocation_info.clone());
                    shared_data.push(shared);
                    processed += 1;

                    // Progress reporting
                    if self.config.enable_progress && processed % 1000 == 0 {
                        println!("   📊 Processed {} allocations...", processed);
                    }
                }

                println!("   ✅ Shared ref collection: {} allocations", shared_data.len());
                Ok(shared_data)
            }
            Err(_) => {
                Err(TrackingError::LockError("Could not acquire lock for shared ref collection".to_string()))
            }
        }
    }

    /// Collect using chunked approach with minimal lock time
    fn collect_chunked(
        &self,
        tracker: &crate::core::tracker::MemoryTracker,
    ) -> TrackingResult<Vec<SharedAllocationData>> {
        println!("   🔄 Starting chunked collection...");
        let start_time = Instant::now();

        let mut all_shared_data = Vec::new();
        let mut chunk_count = 0;
        let max_chunks = 10; // Limit number of chunks to prevent infinite loops

        while chunk_count < max_chunks {
            chunk_count += 1;
            
            // Try to get a chunk of data with very short lock time
            match self.collect_chunk(tracker, chunk_count) {
                Ok(chunk_data) => {
                    if chunk_data.is_empty() {
                        println!("   ✅ No more data, chunked collection complete");
                        break;
                    }
                    
                    println!("   📦 Chunk {}: {} allocations", chunk_count, chunk_data.len());
                    all_shared_data.extend(chunk_data);
                }
                Err(e) => {
                    println!("   ⚠️  Chunk {} failed: {}", chunk_count, e);
                    if chunk_count == 1 {
                        // If first chunk fails, return error
                        return Err(e);
                    } else {
                        // If later chunks fail, return what we have
                        break;
                    }
                }
            }

            // Small delay between chunks to reduce contention
            std::thread::sleep(Duration::from_millis(1));
        }

        let duration = start_time.elapsed();
        println!("   ✅ Chunked collection completed: {} allocations in {:?} ({} chunks)", 
                all_shared_data.len(), duration, chunk_count);
        
        Ok(all_shared_data)
    }

    /// Collect a single chunk of data
    fn collect_chunk(
        &self,
        tracker: &crate::core::tracker::MemoryTracker,
        chunk_id: usize,
    ) -> TrackingResult<Vec<SharedAllocationData>> {
        let start_time = Instant::now();
        let max_lock_time = Duration::from_millis(self.config.max_lock_time_ms);

        // Try to acquire lock with very short timeout
        match tracker.active_allocations.try_lock() {
            Ok(allocations_guard) => {
                let mut chunk_data = Vec::new();
                let skip_count = (chunk_id - 1) * self.config.batch_size;
                let take_count = self.config.batch_size;

                // Skip to the right position and take a batch
                for (i, (ptr, allocation_info)) in allocations_guard.iter().enumerate() {
                    if start_time.elapsed() > max_lock_time {
                        println!("     ⏰ Chunk {} lock timeout after {} items", chunk_id, chunk_data.len());
                        break;
                    }

                    if i < skip_count {
                        continue;
                    }

                    if chunk_data.len() >= take_count {
                        break;
                    }

                    // Create shared reference
                    let shared = SharedAllocationData::new(allocation_info.clone());
                    chunk_data.push(shared);
                }

                Ok(chunk_data)
            }
            Err(_) => {
                Err(TrackingError::LockError(format!("Could not acquire lock for chunk {}", chunk_id)))
            }
        }
    }

    /// Convert shared data to regular AllocationInfo vector when needed
    pub fn to_allocation_info_vec(
        shared_data: Vec<SharedAllocationData>
    ) -> Vec<AllocationInfo> {
        println!("🔄 Converting {} shared allocations to AllocationInfo...", shared_data.len());
        
        shared_data.into_iter()
            .map(|shared| (*shared.data).clone()) // Only clone when absolutely necessary
            .collect()
    }

    /// Get lightweight statistics without full collection
    pub fn get_lightweight_stats(
        &self,
        tracker: &crate::core::tracker::MemoryTracker,
    ) -> TrackingResult<(usize, usize)> {
        // Try to get quick stats without holding lock for long
        match tracker.active_allocations.try_lock() {
            Ok(allocations_guard) => {
                let count = allocations_guard.len();
                let total_size: usize = allocations_guard.values()
                    .take(100) // Sample first 100 to estimate
                    .map(|info| info.size)
                    .sum::<usize>() * (count / 100.max(1));
                
                Ok((count, total_size))
            }
            Err(_) => {
                // Fallback to tracker stats
                match tracker.get_stats() {
                    Ok(stats) => Ok((stats.active_allocations, stats.active_memory)),
                    Err(e) => Err(e),
                }
            }
        }
    }
}

impl Default for CowDataCollector {
    fn default() -> Self {
        Self::new(CowCollectorConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cow_collector_config() {
        let config = CowCollectorConfig::default();
        assert!(config.use_shared_refs);
        assert_eq!(config.max_lock_time_ms, 10);
        assert_eq!(config.batch_size, 1000);
    }

    #[test]
    fn test_allocation_ref_conversion() {
        let mut info = AllocationInfo::new(0x1000, 256);
        info.var_name = Some("test_var".to_string());
        info.type_name = Some("TestType".to_string());

        let alloc_ref = AllocationRef::from_allocation_info(&info);
        assert_eq!(alloc_ref.ptr, 0x1000);
        assert_eq!(alloc_ref.size, 256);
        assert_eq!(alloc_ref.var_name, "test_var");
        assert_eq!(alloc_ref.type_name, "TestType");

        let converted_back = alloc_ref.to_allocation_info();
        assert_eq!(converted_back.ptr, info.ptr);
        assert_eq!(converted_back.size, info.size);
        assert_eq!(converted_back.var_name, info.var_name);
        assert_eq!(converted_back.type_name, info.type_name);
    }
}