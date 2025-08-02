//! Cache optimization for binary export system
//!
//! This module provides cache-friendly data layouts, memory pools, prefetching strategies,
//! and locality optimizations to improve cache hit rates and reduce memory access latency.

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

/// Cache line size for alignment optimization
pub const CACHE_LINE_SIZE: usize = 64;

/// Memory pool configuration
#[derive(Debug, Clone)]
pub struct MemoryPoolConfig {
    /// Block size for allocations
    pub block_size: usize,
    /// Number of blocks to pre-allocate
    pub initial_blocks: usize,
    /// Maximum number of blocks in pool
    pub max_blocks: usize,
    /// Whether to align blocks to cache lines
    pub cache_aligned: bool,
    /// Enable memory prefetching
    pub enable_prefetch: bool,
}

impl Default for MemoryPoolConfig {
    fn default() -> Self {
        Self {
            block_size: 4096, // 4KB blocks
            initial_blocks: 16,
            max_blocks: 256,
            cache_aligned: true,
            enable_prefetch: true,
        }
    }
}

/// Cache-aligned memory block
#[derive(Debug)]
pub struct CacheAlignedBlock {
    /// Pointer to the allocated memory
    ptr: NonNull<u8>,
    /// Size of the block
    size: usize,
    /// Whether the block is currently in use
    in_use: bool,
    /// Layout used for allocation
    layout: Layout,
}

impl CacheAlignedBlock {
    /// Create a new cache-aligned block
    pub fn new(size: usize, cache_aligned: bool) -> Result<Self, CacheOptimizationError> {
        let align = if cache_aligned { CACHE_LINE_SIZE } else { 8 };
        let layout = Layout::from_size_align(size, align)
            .map_err(|_| CacheOptimizationError::InvalidLayout { size, align })?;

        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            return Err(CacheOptimizationError::AllocationFailed { size });
        }

        Ok(Self {
            ptr: NonNull::new(ptr).unwrap(),
            size,
            in_use: false,
            layout,
        })
    }

    /// Get a pointer to the block data
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    /// Get the size of the block
    pub fn size(&self) -> usize {
        self.size
    }

    /// Check if the block is in use
    pub fn is_in_use(&self) -> bool {
        self.in_use
    }

    /// Mark the block as in use
    pub fn mark_in_use(&mut self) {
        self.in_use = true;
    }

    /// Mark the block as free
    pub fn mark_free(&mut self) {
        self.in_use = false;
    }

    /// Get a slice view of the block
    pub unsafe fn as_slice(&self) -> &[u8] {
        std::slice::from_raw_parts(self.ptr.as_ptr(), self.size)
    }

    /// Get a mutable slice view of the block
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.size)
    }
}

impl Drop for CacheAlignedBlock {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

/// Memory pool for cache-optimized allocations
pub struct MemoryPool {
    /// Configuration
    config: MemoryPoolConfig,
    /// Available blocks
    free_blocks: Vec<CacheAlignedBlock>,
    /// Blocks currently in use
    used_blocks: Vec<CacheAlignedBlock>,
    /// Statistics
    stats: MemoryPoolStats,
}

/// Memory pool statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryPoolStats {
    /// Total allocations requested
    pub total_allocations: usize,
    /// Total deallocations
    pub total_deallocations: usize,
    /// Cache hits (reused blocks)
    pub cache_hits: usize,
    /// Cache misses (new allocations)
    pub cache_misses: usize,
    /// Current blocks in use
    pub current_used_blocks: usize,
    /// Peak blocks in use
    pub peak_used_blocks: usize,
    /// Total memory allocated
    pub total_memory_allocated: usize,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new(config: MemoryPoolConfig) -> Result<Self, CacheOptimizationError> {
        let mut pool = Self {
            config: config.clone(),
            free_blocks: Vec::new(),
            used_blocks: Vec::new(),
            stats: MemoryPoolStats::default(),
        };

        // Pre-allocate initial blocks
        for _ in 0..config.initial_blocks {
            let block = CacheAlignedBlock::new(config.block_size, config.cache_aligned)?;
            pool.free_blocks.push(block);
            pool.stats.total_memory_allocated += config.block_size;
        }

        Ok(pool)
    }

    /// Allocate a block from the pool
    pub fn allocate(&mut self, size: usize) -> Result<*mut u8, CacheOptimizationError> {
        self.stats.total_allocations += 1;

        // Try to find a suitable free block
        if let Some(index) = self
            .free_blocks
            .iter()
            .position(|block| block.size() >= size)
        {
            let mut block = self.free_blocks.remove(index);
            block.mark_in_use();
            let ptr = block.as_ptr();

            // Prefetch the memory if enabled
            if self.config.enable_prefetch {
                self.prefetch_memory(ptr, size);
            }

            self.used_blocks.push(block);
            self.stats.cache_hits += 1;
            self.stats.current_used_blocks += 1;

            if self.stats.current_used_blocks > self.stats.peak_used_blocks {
                self.stats.peak_used_blocks = self.stats.current_used_blocks;
            }

            return Ok(ptr);
        }

        // No suitable block found, allocate a new one if under limit
        if self.free_blocks.len() + self.used_blocks.len() < self.config.max_blocks {
            let block_size = std::cmp::max(size, self.config.block_size);
            let mut block = CacheAlignedBlock::new(block_size, self.config.cache_aligned)?;
            block.mark_in_use();
            let ptr = block.as_ptr();

            // Prefetch the memory if enabled
            if self.config.enable_prefetch {
                self.prefetch_memory(ptr, size);
            }

            self.used_blocks.push(block);
            self.stats.cache_misses += 1;
            self.stats.current_used_blocks += 1;
            self.stats.total_memory_allocated += block_size;

            if self.stats.current_used_blocks > self.stats.peak_used_blocks {
                self.stats.peak_used_blocks = self.stats.current_used_blocks;
            }

            Ok(ptr)
        } else {
            Err(CacheOptimizationError::PoolExhausted {
                max_blocks: self.config.max_blocks,
            })
        }
    }

    /// Deallocate a block back to the pool
    pub fn deallocate(&mut self, ptr: *mut u8) -> Result<(), CacheOptimizationError> {
        self.stats.total_deallocations += 1;

        // Find the block in used_blocks
        if let Some(index) = self
            .used_blocks
            .iter()
            .position(|block| block.as_ptr() == ptr)
        {
            let mut block = self.used_blocks.remove(index);
            block.mark_free();
            self.free_blocks.push(block);
            self.stats.current_used_blocks -= 1;
            Ok(())
        } else {
            Err(CacheOptimizationError::InvalidPointer)
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> &MemoryPoolStats {
        &self.stats
    }

    /// Get cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        if self.stats.total_allocations == 0 {
            0.0
        } else {
            self.stats.cache_hits as f64 / self.stats.total_allocations as f64
        }
    }

    /// Prefetch memory to improve cache performance
    fn prefetch_memory(&self, ptr: *mut u8, size: usize) {
        // Prefetch memory in cache line chunks
        let mut current = ptr as usize;
        let end = current + size;

        while current < end {
            unsafe {
                // Use platform-specific prefetch instructions
                #[cfg(target_arch = "x86_64")]
                {
                    std::arch::x86_64::_mm_prefetch(
                        current as *const i8,
                        std::arch::x86_64::_MM_HINT_T0,
                    );
                }
                #[cfg(target_arch = "aarch64")]
                {
                    // Use compiler hint for prefetching instead of unstable intrinsic
                    // This provides a stable alternative that still gives the compiler
                    // optimization hints without requiring unstable features
                    std::hint::black_box(unsafe { std::ptr::read_volatile(current as *const u8) });
                }
            }
            current += CACHE_LINE_SIZE;
        }
    }

    /// Clear all free blocks to reduce memory usage
    pub fn trim(&mut self) {
        let initial_count = self.free_blocks.len();
        self.free_blocks.clear();
        self.stats.total_memory_allocated -= initial_count * self.config.block_size;
    }
}

/// Cache-friendly data structure for storing allocation information
#[repr(C, align(64))] // Align to cache line
pub struct CacheFriendlyAllocation {
    /// Pointer value (8 bytes)
    pub ptr: usize,
    /// Size (8 bytes)
    pub size: usize,
    /// Timestamp (8 bytes)
    pub timestamp: u64,
    /// Type ID (4 bytes)
    pub type_id: u32,
    /// Flags (4 bytes)
    pub flags: u32,
    /// Padding to fill cache line (24 bytes)
    _padding: [u8; 24],
}

impl CacheFriendlyAllocation {
    /// Create a new cache-friendly allocation record
    pub fn new(ptr: usize, size: usize, timestamp: u64, type_id: u32, flags: u32) -> Self {
        Self {
            ptr,
            size,
            timestamp,
            type_id,
            flags,
            _padding: [0; 24],
        }
    }

    /// Convert from AllocationInfo
    pub fn from_allocation_info(alloc: &crate::core::types::AllocationInfo, type_id: u32) -> Self {
        let flags = if alloc.is_leaked { 1 } else { 0 };
        Self::new(alloc.ptr, alloc.size, alloc.timestamp_alloc, type_id, flags)
    }
}

/// Locality optimizer for improving data access patterns
pub struct LocalityOptimizer {
    /// Memory pool for allocations
    pool: Arc<Mutex<MemoryPool>>,
    /// Cache for frequently accessed data
    data_cache: HashMap<u64, Vec<u8>>,
    /// Access pattern tracking
    access_patterns: HashMap<usize, AccessPattern>,
}

/// Access pattern information
#[derive(Debug, Clone)]
pub struct AccessPattern {
    /// Number of accesses
    pub access_count: usize,
    /// Last access time
    pub last_access: std::time::Instant,
    /// Average access interval
    pub avg_interval: std::time::Duration,
    /// Sequential access likelihood
    pub sequential_likelihood: f64,
}

impl LocalityOptimizer {
    /// Create a new locality optimizer
    pub fn new(pool_config: MemoryPoolConfig) -> Result<Self, CacheOptimizationError> {
        let pool = Arc::new(Mutex::new(MemoryPool::new(pool_config)?));

        Ok(Self {
            pool,
            data_cache: HashMap::new(),
            access_patterns: HashMap::new(),
        })
    }

    /// Optimize data layout for cache efficiency
    pub fn optimize_allocation_layout(
        &mut self,
        allocations: &[crate::core::types::AllocationInfo],
    ) -> Result<Vec<CacheFriendlyAllocation>, CacheOptimizationError> {
        let mut optimized = Vec::with_capacity(allocations.len());

        // Sort allocations by access patterns for better locality
        let mut sorted_allocations: Vec<_> = allocations.iter().enumerate().collect();
        sorted_allocations.sort_by(|a, b| {
            // Sort by size first (larger allocations first)
            b.1.size
                .cmp(&a.1.size)
                // Then by timestamp for temporal locality
                .then(a.1.timestamp_alloc.cmp(&b.1.timestamp_alloc))
        });

        // Convert to cache-friendly format
        for (original_index, alloc) in sorted_allocations {
            let type_id = self.get_or_assign_type_id(&alloc.type_name);
            let cache_friendly = CacheFriendlyAllocation::from_allocation_info(alloc, type_id);
            optimized.push(cache_friendly);

            // Update access patterns
            self.update_access_pattern(alloc.ptr);
        }

        Ok(optimized)
    }

    /// Prefetch data based on access patterns
    pub fn prefetch_data(&self, ptr: usize, size: usize) {
        if let Some(pattern) = self.access_patterns.get(&ptr) {
            // If this is likely to be accessed sequentially, prefetch more
            if pattern.sequential_likelihood > 0.7 {
                let prefetch_size = std::cmp::min(size * 2, 4096); // Prefetch up to 4KB
                self.prefetch_memory_range(ptr, prefetch_size);
            } else {
                self.prefetch_memory_range(ptr, size);
            }
        }
    }

    /// Get memory pool statistics
    pub fn pool_stats(&self) -> Result<MemoryPoolStats, CacheOptimizationError> {
        Ok(self
            .pool
            .lock()
            .map_err(|_| CacheOptimizationError::LockError)?
            .stats()
            .clone())
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            cache_entries: self.data_cache.len(),
            total_cached_bytes: self.data_cache.values().map(|v| v.len()).sum(),
            access_patterns_tracked: self.access_patterns.len(),
        }
    }

    /// Update access pattern for a memory location
    fn update_access_pattern(&mut self, ptr: usize) {
        let now = std::time::Instant::now();

        if let Some(pattern) = self.access_patterns.get_mut(&ptr) {
            let interval = now.duration_since(pattern.last_access);
            pattern.access_count += 1;
            pattern.last_access = now;

            // Update average interval
            let alpha = 0.1; // Exponential moving average factor
            pattern.avg_interval = std::time::Duration::from_nanos(
                (pattern.avg_interval.as_nanos() as f64 * (1.0 - alpha)
                    + interval.as_nanos() as f64 * alpha) as u64,
            );
        } else {
            self.access_patterns.insert(
                ptr,
                AccessPattern {
                    access_count: 1,
                    last_access: now,
                    avg_interval: std::time::Duration::from_millis(100), // Default
                    sequential_likelihood: 0.5,                          // Neutral
                },
            );
        }
    }

    /// Get or assign a type ID for caching
    fn get_or_assign_type_id(&self, type_name: &Option<String>) -> u32 {
        match type_name {
            Some(name) => {
                // Simple hash-based type ID assignment
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut hasher = DefaultHasher::new();
                name.hash(&mut hasher);
                hasher.finish() as u32
            }
            None => 0, // Unknown type
        }
    }

    /// Prefetch memory range
    fn prefetch_memory_range(&self, ptr: usize, size: usize) {
        let mut current = ptr;
        let end = ptr + size;

        while current < end {
            unsafe {
                #[cfg(target_arch = "x86_64")]
                {
                    std::arch::x86_64::_mm_prefetch(
                        current as *const i8,
                        std::arch::x86_64::_MM_HINT_T0,
                    );
                }
                #[cfg(target_arch = "aarch64")]
                {
                    // Use compiler hint for prefetching instead of unstable intrinsic
                    // This provides a stable alternative that still gives the compiler
                    // optimization hints without requiring unstable features
                    std::hint::black_box(unsafe { std::ptr::read_volatile(current as *const u8) });
                }
            }
            current += CACHE_LINE_SIZE;
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub cache_entries: usize,
    pub total_cached_bytes: usize,
    pub access_patterns_tracked: usize,
}

/// Cache optimization errors
#[derive(Debug, thiserror::Error)]
pub enum CacheOptimizationError {
    #[error("Invalid layout: size {size}, align {align}")]
    InvalidLayout { size: usize, align: usize },

    #[error("Allocation failed for size {size}")]
    AllocationFailed { size: usize },

    #[error("Memory pool exhausted: max blocks {max_blocks}")]
    PoolExhausted { max_blocks: usize },

    #[error("Invalid pointer for deallocation")]
    InvalidPointer,

    #[error("Lock error")]
    LockError,
}

/// Batch processor with cache optimization
pub struct CacheOptimizedBatchProcessor {
    /// Locality optimizer
    optimizer: LocalityOptimizer,
    /// Batch size for processing
    batch_size: usize,
}

impl CacheOptimizedBatchProcessor {
    /// Create a new cache-optimized batch processor
    pub fn new(
        pool_config: MemoryPoolConfig,
        batch_size: usize,
    ) -> Result<Self, CacheOptimizationError> {
        Ok(Self {
            optimizer: LocalityOptimizer::new(pool_config)?,
            batch_size,
        })
    }

    /// Process allocations in cache-friendly batches
    pub fn process_allocations_batched<F, R>(
        &mut self,
        allocations: &[crate::core::types::AllocationInfo],
        mut processor: F,
    ) -> Result<Vec<R>, CacheOptimizationError>
    where
        F: FnMut(&[CacheFriendlyAllocation]) -> Result<R, CacheOptimizationError>,
    {
        // Optimize layout first
        let optimized = self.optimizer.optimize_allocation_layout(allocations)?;
        let mut results = Vec::new();

        // Process in batches for better cache locality
        for batch in optimized.chunks(self.batch_size) {
            // Prefetch the batch data
            for alloc in batch {
                self.optimizer.prefetch_data(alloc.ptr, alloc.size);
            }

            let result = processor(batch)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Get optimization statistics
    pub fn get_stats(&self) -> Result<(MemoryPoolStats, CacheStats), CacheOptimizationError> {
        let pool_stats = self.optimizer.pool_stats()?;
        let cache_stats = self.optimizer.cache_stats();
        Ok((pool_stats, cache_stats))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_aligned_block() -> Result<(), CacheOptimizationError> {
        let block = CacheAlignedBlock::new(1024, true)?;

        // Check alignment
        assert_eq!(block.as_ptr() as usize % CACHE_LINE_SIZE, 0);
        assert_eq!(block.size(), 1024);
        assert!(!block.is_in_use());

        Ok(())
    }

    #[test]
    fn test_memory_pool() -> Result<(), CacheOptimizationError> {
        let config = MemoryPoolConfig::default();
        let mut pool = MemoryPool::new(config)?;

        // Allocate some blocks
        let ptr1 = pool.allocate(1024)?;
        let ptr2 = pool.allocate(2048)?;

        assert_ne!(ptr1, ptr2);
        assert_eq!(pool.stats().current_used_blocks, 2);

        // Deallocate
        pool.deallocate(ptr1)?;
        assert_eq!(pool.stats().current_used_blocks, 1);

        // Reallocate should reuse
        let ptr3 = pool.allocate(512)?;
        assert_eq!(pool.stats().cache_hits, 1);

        Ok(())
    }

    #[test]
    fn test_cache_friendly_allocation() {
        let alloc = CacheFriendlyAllocation::new(0x1000, 256, 12345, 42, 1);

        assert_eq!(alloc.ptr, 0x1000);
        assert_eq!(alloc.size, 256);
        assert_eq!(alloc.timestamp, 12345);
        assert_eq!(alloc.type_id, 42);
        assert_eq!(alloc.flags, 1);

        // Check size is cache line aligned
        assert_eq!(
            std::mem::size_of::<CacheFriendlyAllocation>(),
            CACHE_LINE_SIZE
        );
    }

    #[test]
    fn test_locality_optimizer() -> Result<(), CacheOptimizationError> {
        let config = MemoryPoolConfig::default();
        let mut optimizer = LocalityOptimizer::new(config)?;

        // Create some test allocations
        let allocations = vec![
            crate::core::types::AllocationInfo::new(0x1000, 1024),
            crate::core::types::AllocationInfo::new(0x2000, 512),
        ];

        let optimized = optimizer.optimize_allocation_layout(&allocations)?;
        assert_eq!(optimized.len(), 2);

        // Larger allocation should come first
        assert_eq!(optimized[0].size, 1024);
        assert_eq!(optimized[1].size, 512);

        Ok(())
    }
}
