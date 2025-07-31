//! Memory management for binary export operations
//!
//! This module provides intelligent memory management including zero-copy
//! optimizations, adaptive memory allocation, and backpressure control
//! for efficient handling of large datasets.

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::borrow::Cow;

/// Memory manager for binary export operations
pub struct MemoryManager {
    /// Maximum memory usage allowed (bytes)
    max_memory: usize,
    /// Current memory usage (bytes)
    current_usage: Arc<Mutex<usize>>,
    /// Peak memory usage during operation
    peak_usage: Arc<Mutex<usize>>,
    /// Memory allocation history for analysis
    allocation_history: Arc<Mutex<VecDeque<MemoryAllocation>>>,
    /// Enable memory pressure monitoring
    enable_monitoring: bool,
}

/// Record of a memory allocation
#[derive(Debug, Clone)]
struct MemoryAllocation {
    /// Size of allocation in bytes
    size: usize,
    /// Timestamp of allocation
    timestamp: std::time::Instant,
    /// Type of allocation
    allocation_type: AllocationType,
}

/// Types of memory allocations tracked
#[derive(Debug, Clone)]
enum AllocationType {
    /// Buffer for data collection
    DataBuffer,
    /// Compression working memory
    CompressionBuffer,
    /// Serialization buffer
    SerializationBuffer,
    /// I/O buffer
    IoBuffer,
    /// Temporary working memory
    Temporary,
}

impl MemoryManager {
    /// Create a new memory manager with the specified memory limit
    pub fn new(max_memory: usize) -> Self {
        Self {
            max_memory,
            current_usage: Arc::new(Mutex::new(0)),
            peak_usage: Arc::new(Mutex::new(0)),
            allocation_history: Arc::new(Mutex::new(VecDeque::new())),
            enable_monitoring: true,
        }
    }

    /// Allocate memory with the specified size and type
    pub fn allocate(&self, size: usize, alloc_type: AllocationType) -> Result<MemoryAllocation, crate::export::binary::error::BinaryExportError> {
        let mut current = self.current_usage.lock().unwrap();
        
        // Check if allocation would exceed limit
        if *current + size > self.max_memory {
            return Err(crate::export::binary::error::BinaryExportError::MemoryLimitExceeded {
                limit: self.max_memory,
                usage: *current + size,
            });
        }

        // Update current usage
        *current += size;
        
        // Update peak usage
        let mut peak = self.peak_usage.lock().unwrap();
        if *current > *peak {
            *peak = *current;
        }
        drop(peak);
        drop(current);

        let allocation = MemoryAllocation {
            size,
            timestamp: std::time::Instant::now(),
            allocation_type: alloc_type,
        };

        // Record allocation in history
        if self.enable_monitoring {
            let mut history = self.allocation_history.lock().unwrap();
            history.push_back(allocation.clone());
            
            // Keep history size reasonable
            if history.len() > 1000 {
                history.pop_front();
            }
        }

        Ok(allocation)
    }

    /// Deallocate memory
    pub fn deallocate(&self, allocation: &MemoryAllocation) {
        let mut current = self.current_usage.lock().unwrap();
        *current = current.saturating_sub(allocation.size);
    }

    /// Get current memory usage
    pub fn current_usage(&self) -> usize {
        *self.current_usage.lock().unwrap()
    }

    /// Get peak memory usage
    pub fn peak_usage(&self) -> usize {
        *self.peak_usage.lock().unwrap()
    }

    /// Get memory usage percentage
    pub fn usage_percentage(&self) -> f64 {
        let current = self.current_usage();
        (current as f64 / self.max_memory as f64) * 100.0
    }

    /// Check if memory pressure is high
    pub fn is_memory_pressure_high(&self) -> bool {
        self.usage_percentage() > 80.0
    }

    /// Get available memory
    pub fn available_memory(&self) -> usize {
        self.max_memory.saturating_sub(self.current_usage())
    }

    /// Reset memory tracking
    pub fn reset(&self) {
        *self.current_usage.lock().unwrap() = 0;
        *self.peak_usage.lock().unwrap() = 0;
        if self.enable_monitoring {
            self.allocation_history.lock().unwrap().clear();
        }
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        let current = self.current_usage();
        let peak = self.peak_usage();
        let available = self.available_memory();
        
        MemoryStats {
            max_memory: self.max_memory,
            current_usage: current,
            peak_usage: peak,
            available_memory: available,
            usage_percentage: self.usage_percentage(),
            allocation_count: if self.enable_monitoring {
                self.allocation_history.lock().unwrap().len()
            } else {
                0
            },
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Maximum memory limit
    pub max_memory: usize,
    /// Current memory usage
    pub current_usage: usize,
    /// Peak memory usage
    pub peak_usage: usize,
    /// Available memory
    pub available_memory: usize,
    /// Usage percentage
    pub usage_percentage: f64,
    /// Number of tracked allocations
    pub allocation_count: usize,
}

/// Zero-copy view for efficient data access
pub struct ZeroCopyView<'a> {
    /// Reference to the underlying data
    data: &'a [u8],
    /// Offset within the data
    offset: usize,
    /// Length of the view
    length: usize,
}

impl<'a> ZeroCopyView<'a> {
    /// Create a new zero-copy view
    pub fn new(data: &'a [u8], offset: usize, length: usize) -> Result<Self, crate::export::binary::error::BinaryExportError> {
        if offset + length > data.len() {
            return Err(crate::export::binary::error::BinaryExportError::InternalError(
                "View bounds exceed data length".to_string()
            ));
        }

        Ok(Self {
            data,
            offset,
            length,
        })
    }

    /// Get the data slice
    pub fn as_slice(&self) -> &[u8] {
        &self.data[self.offset..self.offset + self.length]
    }

    /// Get the length of the view
    pub fn len(&self) -> usize {
        self.length
    }

    /// Check if the view is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Create a sub-view
    pub fn sub_view(&self, offset: usize, length: usize) -> Result<ZeroCopyView<'a>, crate::export::binary::error::BinaryExportError> {
        if offset + length > self.length {
            return Err(crate::export::binary::error::BinaryExportError::InternalError(
                "Sub-view bounds exceed view length".to_string()
            ));
        }

        Ok(ZeroCopyView {
            data: self.data,
            offset: self.offset + offset,
            length,
        })
    }
}

/// Smart buffer that adapts to memory pressure
pub struct SmartBuffer {
    /// Buffer data - uses Cow for efficient memory usage
    data: Cow<'static, [u8]>,
    /// Memory manager reference
    memory_manager: Arc<MemoryManager>,
    /// Allocation record
    allocation: Option<MemoryAllocation>,
}

impl SmartBuffer {
    /// Create a new smart buffer
    pub fn new(capacity: usize, memory_manager: Arc<MemoryManager>) -> Result<Self, crate::export::binary::error::BinaryExportError> {
        let allocation = memory_manager.allocate(capacity, AllocationType::DataBuffer)?;
        
        Ok(Self {
            data: Cow::Owned(vec![0; capacity]),
            memory_manager,
            allocation: Some(allocation),
        })
    }

    /// Create a smart buffer from existing data (zero-copy when possible)
    pub fn from_data(data: Vec<u8>, memory_manager: Arc<MemoryManager>) -> Result<Self, crate::export::binary::error::BinaryExportError> {
        let size = data.len();
        let allocation = memory_manager.allocate(size, AllocationType::DataBuffer)?;
        
        Ok(Self {
            data: Cow::Owned(data),
            memory_manager,
            allocation: Some(allocation),
        })
    }

    /// Get buffer data as slice
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable access to buffer data
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.data.to_mut()
    }

    /// Get buffer length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Resize buffer (may trigger reallocation)
    pub fn resize(&mut self, new_size: usize) -> Result<(), crate::export::binary::error::BinaryExportError> {
        let current_size = self.len();
        
        if new_size > current_size {
            // Check if we can allocate more memory
            let additional = new_size - current_size;
            let _additional_alloc = self.memory_manager.allocate(additional, AllocationType::DataBuffer)?;
            
            // Deallocate old allocation
            if let Some(old_alloc) = &self.allocation {
                self.memory_manager.deallocate(old_alloc);
            }
            
            // Update allocation record
            self.allocation = Some(_additional_alloc);
        }

        // Resize the buffer
        self.data.to_mut().resize(new_size, 0);
        
        Ok(())
    }

    /// Shrink buffer to fit actual data
    pub fn shrink_to_fit(&mut self) {
        if let Cow::Owned(ref mut vec) = self.data {
            vec.shrink_to_fit();
        }
    }
}

impl Drop for SmartBuffer {
    fn drop(&mut self) {
        if let Some(allocation) = &self.allocation {
            self.memory_manager.deallocate(allocation);
        }
    }
}

/// Memory pool for efficient buffer reuse
pub struct MemoryPool {
    /// Available buffers by size
    available_buffers: Arc<Mutex<std::collections::HashMap<usize, Vec<Vec<u8>>>>>,
    /// Memory manager
    memory_manager: Arc<MemoryManager>,
    /// Maximum number of buffers to keep per size
    max_buffers_per_size: usize,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new(memory_manager: Arc<MemoryManager>) -> Self {
        Self {
            available_buffers: Arc::new(Mutex::new(std::collections::HashMap::new())),
            memory_manager,
            max_buffers_per_size: 10,
        }
    }

    /// Get a buffer from the pool or allocate a new one
    pub fn get_buffer(&self, size: usize) -> Result<Vec<u8>, crate::export::binary::error::BinaryExportError> {
        let mut buffers = self.available_buffers.lock().unwrap();
        
        if let Some(size_buffers) = buffers.get_mut(&size) {
            if let Some(buffer) = size_buffers.pop() {
                return Ok(buffer);
            }
        }
        
        drop(buffers);
        
        // Allocate new buffer
        let _allocation = self.memory_manager.allocate(size, AllocationType::DataBuffer)?;
        Ok(vec![0; size])
    }

    /// Return a buffer to the pool
    pub fn return_buffer(&self, mut buffer: Vec<u8>) {
        let size = buffer.len();
        buffer.clear();
        
        let mut buffers = self.available_buffers.lock().unwrap();
        let size_buffers = buffers.entry(size).or_insert_with(Vec::new);
        
        if size_buffers.len() < self.max_buffers_per_size {
            size_buffers.push(buffer);
        }
        // If we have too many buffers, just drop this one
    }

    /// Clear all pooled buffers
    pub fn clear(&self) {
        self.available_buffers.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_manager_basic() {
        let manager = MemoryManager::new(1024);
        assert_eq!(manager.current_usage(), 0);
        assert_eq!(manager.available_memory(), 1024);
        
        let alloc = manager.allocate(512, AllocationType::DataBuffer).unwrap();
        assert_eq!(manager.current_usage(), 512);
        assert_eq!(manager.available_memory(), 512);
        
        manager.deallocate(&alloc);
        assert_eq!(manager.current_usage(), 0);
    }

    #[test]
    fn test_memory_limit_exceeded() {
        let manager = MemoryManager::new(100);
        let result = manager.allocate(200, AllocationType::DataBuffer);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_copy_view() {
        let data = vec![1, 2, 3, 4, 5];
        let view = ZeroCopyView::new(&data, 1, 3).unwrap();
        assert_eq!(view.as_slice(), &[2, 3, 4]);
        assert_eq!(view.len(), 3);
        
        let sub_view = view.sub_view(1, 1).unwrap();
        assert_eq!(sub_view.as_slice(), &[3]);
    }

    #[test]
    fn test_smart_buffer() {
        let manager = Arc::new(MemoryManager::new(1024));
        let buffer = SmartBuffer::new(100, manager.clone()).unwrap();
        assert_eq!(buffer.len(), 100);
        assert_eq!(manager.current_usage(), 100);
    }

    #[test]
    fn test_memory_pool() {
        let manager = Arc::new(MemoryManager::new(1024));
        let pool = MemoryPool::new(manager);
        
        let buffer1 = pool.get_buffer(100).unwrap();
        assert_eq!(buffer1.len(), 100);
        
        pool.return_buffer(buffer1);
        let buffer2 = pool.get_buffer(100).unwrap();
        assert_eq!(buffer2.len(), 100);
    }
}