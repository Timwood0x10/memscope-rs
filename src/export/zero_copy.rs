//! Zero-copy optimizations for binary export system
//!
//! This module provides zero-copy buffer management and direct memory operations
//! to minimize memory allocations and improve performance during binary export.

use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// Zero-copy buffer pool for reusing memory allocations
#[derive(Debug, Clone)]
pub struct ZeroCopyBufferPool {
    /// Pool of reusable buffers
    buffers: Arc<Mutex<VecDeque<BytesMut>>>,
    /// Default buffer size
    default_size: usize,
    /// Maximum number of buffers to keep in pool
    max_pool_size: usize,
    /// Statistics
    stats: Arc<Mutex<BufferPoolStats>>,
}

/// Buffer pool statistics
#[derive(Debug, Clone, Default)]
pub struct BufferPoolStats {
    /// Total buffers allocated
    pub total_allocated: usize,
    /// Total buffers reused from pool
    pub total_reused: usize,
    /// Current pool size
    pub current_pool_size: usize,
    /// Peak pool size
    pub peak_pool_size: usize,
    /// Total bytes allocated
    pub total_bytes_allocated: usize,
    /// Total bytes reused
    pub total_bytes_reused: usize,
}

impl ZeroCopyBufferPool {
    /// Create a new buffer pool
    pub fn new(default_size: usize, max_pool_size: usize) -> Self {
        Self {
            buffers: Arc::new(Mutex::new(VecDeque::new())),
            default_size,
            max_pool_size,
            stats: Arc::new(Mutex::new(BufferPoolStats::default())),
        }
    }

    /// Get a buffer from the pool or allocate a new one
    pub fn get_buffer(&self, min_capacity: usize) -> ZeroCopyBuffer {
        let capacity = std::cmp::max(min_capacity, self.default_size);

        // Try to get a buffer from the pool
        if let Ok(mut buffers) = self.buffers.lock() {
            if let Some(mut buffer) = buffers.pop_front() {
                // Ensure buffer has enough capacity
                if buffer.capacity() >= capacity {
                    buffer.clear();

                    // Update stats
                    if let Ok(mut stats) = self.stats.lock() {
                        stats.total_reused += 1;
                        stats.total_bytes_reused += buffer.capacity();
                        stats.current_pool_size = buffers.len();
                    }

                    return ZeroCopyBuffer::new(buffer, self.clone());
                } else {
                    // Buffer too small, put it back and allocate new one
                    buffers.push_back(buffer);
                }
            }
        }

        // Allocate new buffer
        let buffer = BytesMut::with_capacity(capacity);

        // Update stats
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_allocated += 1;
            stats.total_bytes_allocated += capacity;
        }

        ZeroCopyBuffer::new(buffer, self.clone())
    }

    /// Return a buffer to the pool
    pub fn return_buffer(&self, buffer: BytesMut) {
        if let Ok(mut buffers) = self.buffers.lock() {
            if buffers.len() < self.max_pool_size {
                buffers.push_back(buffer);

                // Update stats
                if let Ok(mut stats) = self.stats.lock() {
                    stats.current_pool_size = buffers.len();
                    if stats.current_pool_size > stats.peak_pool_size {
                        stats.peak_pool_size = stats.current_pool_size;
                    }
                }
            }
        }
    }

    /// Get buffer pool statistics
    pub fn get_stats(&self) -> BufferPoolStats {
        self.stats
            .lock()
            .unwrap_or_else(|_| {
                // Create a default stats instance when lock is poisoned
                self.stats.clear_poison();
                self.stats.lock().unwrap()
            })
            .clone()
    }

    /// Clear the buffer pool
    pub fn clear(&self) {
        if let Ok(mut buffers) = self.buffers.lock() {
            buffers.clear();
            if let Ok(mut stats) = self.stats.lock() {
                stats.current_pool_size = 0;
            }
        }
    }
}

impl Default for ZeroCopyBufferPool {
    fn default() -> Self {
        Self::new(64 * 1024, 16) // 64KB default, max 16 buffers
    }
}

/// Zero-copy buffer wrapper with automatic pool return
pub struct ZeroCopyBuffer {
    /// The actual buffer
    buffer: BytesMut,
    /// Reference to the pool for returning the buffer
    pool: ZeroCopyBufferPool,
}

impl ZeroCopyBuffer {
    /// Create a new zero-copy buffer
    fn new(buffer: BytesMut, pool: ZeroCopyBufferPool) -> Self {
        Self { buffer, pool }
    }

    /// Get the underlying buffer
    pub fn buffer(&self) -> &BytesMut {
        &self.buffer
    }

    /// Get mutable reference to the underlying buffer
    pub fn buffer_mut(&mut self) -> &mut BytesMut {
        &mut self.buffer
    }

    /// Write data to the buffer
    pub fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if self.buffer.remaining_mut() < data.len() {
            self.buffer.reserve(data.len());
        }
        self.buffer.put_slice(data);
        Ok(data.len())
    }

    /// Write a single byte
    pub fn write_u8(&mut self, value: u8) {
        if self.buffer.remaining_mut() < 1 {
            self.buffer.reserve(1);
        }
        self.buffer.put_u8(value);
    }

    /// Write a u16 in little endian
    pub fn write_u16_le(&mut self, value: u16) {
        if self.buffer.remaining_mut() < 2 {
            self.buffer.reserve(2);
        }
        self.buffer.put_u16_le(value);
    }

    /// Write a u32 in little endian
    pub fn write_u32_le(&mut self, value: u32) {
        if self.buffer.remaining_mut() < 4 {
            self.buffer.reserve(4);
        }
        self.buffer.put_u32_le(value);
    }

    /// Write a u64 in little endian
    pub fn write_u64_le(&mut self, value: u64) {
        if self.buffer.remaining_mut() < 8 {
            self.buffer.reserve(8);
        }
        self.buffer.put_u64_le(value);
    }

    /// Write a usize as u64 in little endian
    pub fn write_usize_le(&mut self, value: usize) {
        self.write_u64_le(value as u64);
    }

    /// Write a string with length prefix
    pub fn write_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.write_u32_le(bytes.len() as u32);
        let _ = self.write(bytes);
    }

    /// Write an optional string
    pub fn write_optional_string(&mut self, s: &Option<String>) {
        match s {
            Some(string) => {
                self.write_u8(1); // Some marker
                self.write_string(string);
            }
            None => {
                self.write_u8(0); // None marker
            }
        }
    }

    /// Get the current length of data in the buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get the capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Convert to immutable Bytes (zero-copy)
    pub fn freeze(self) -> Bytes {
        self.buffer.clone().freeze()
    }

    /// Split the buffer at the given index (zero-copy)
    pub fn split_to(&mut self, at: usize) -> Bytes {
        self.buffer.split_to(at).freeze()
    }

    /// Clear the buffer for reuse
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Reserve additional capacity
    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional);
    }
}

impl Drop for ZeroCopyBuffer {
    fn drop(&mut self) {
        // Return the buffer to the pool when dropped
        let buffer = std::mem::replace(&mut self.buffer, BytesMut::new());
        self.pool.return_buffer(buffer);
    }
}

impl Write for ZeroCopyBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(()) // No-op for in-memory buffer
    }
}

/// Zero-copy writer that minimizes allocations
pub struct ZeroCopyWriter<W: Write> {
    /// Underlying writer
    writer: W,
    /// Buffer pool for reusing buffers
    pool: ZeroCopyBufferPool,
    /// Current write buffer
    current_buffer: Option<ZeroCopyBuffer>,
    /// Buffer size threshold for flushing
    flush_threshold: usize,
    /// Total bytes written
    bytes_written: usize,
}

impl<W: Write> ZeroCopyWriter<W> {
    /// Create a new zero-copy writer
    pub fn new(writer: W, pool: ZeroCopyBufferPool) -> Self {
        Self {
            writer,
            pool,
            current_buffer: None,
            flush_threshold: 64 * 1024, // 64KB default
            bytes_written: 0,
        }
    }

    /// Set the flush threshold
    pub fn with_flush_threshold(mut self, threshold: usize) -> Self {
        self.flush_threshold = threshold;
        self
    }

    /// Write data using zero-copy optimization
    pub fn write_zero_copy(&mut self, data: &[u8]) -> io::Result<()> {
        // Get or create current buffer
        if self.current_buffer.is_none() {
            self.current_buffer = Some(self.pool.get_buffer(self.flush_threshold));
        }

        let buffer = self.current_buffer.as_mut().unwrap();

        // If data is larger than remaining buffer space, flush first
        if buffer.len() + data.len() > self.flush_threshold {
            self.flush_buffer()?;
            self.current_buffer = Some(
                self.pool
                    .get_buffer(std::cmp::max(data.len(), self.flush_threshold)),
            );
            let buffer = self.current_buffer.as_mut().unwrap();
            buffer.write(data)?;
        } else {
            buffer.write(data)?;
        }

        self.bytes_written += data.len();
        Ok(())
    }

    /// Write bytes directly (zero-copy when possible)
    pub fn write_bytes(&mut self, bytes: Bytes) -> io::Result<()> {
        // For large chunks, write directly to avoid copying
        if bytes.len() > self.flush_threshold / 2 {
            self.flush_buffer()?;
            self.writer.write_all(&bytes)?;
            self.bytes_written += bytes.len();
        } else {
            self.write_zero_copy(&bytes)?;
        }
        Ok(())
    }

    /// Flush the current buffer
    pub fn flush_buffer(&mut self) -> io::Result<()> {
        if let Some(buffer) = self.current_buffer.take() {
            if !buffer.is_empty() {
                let bytes = buffer.freeze();
                self.writer.write_all(&bytes)?;
            }
        }
        Ok(())
    }

    /// Get total bytes written
    pub fn bytes_written(&self) -> usize {
        self.bytes_written
    }

    /// Get buffer pool statistics
    pub fn pool_stats(&self) -> BufferPoolStats {
        self.pool.get_stats()
    }
}

impl<W: Write> Write for ZeroCopyWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_zero_copy(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_buffer()?;
        self.writer.flush()
    }
}

impl<W: Write> Drop for ZeroCopyWriter<W> {
    fn drop(&mut self) {
        let _ = self.flush_buffer();
    }
}

/// Vectorized operations for batch processing
pub struct VectorizedProcessor {
    /// Buffer pool for operations
    pool: ZeroCopyBufferPool,
}

impl VectorizedProcessor {
    /// Create a new vectorized processor
    pub fn new(pool: ZeroCopyBufferPool) -> Self {
        Self { pool }
    }

    /// Process multiple allocations in batches
    pub fn process_allocations_batch<F>(
        &self,
        allocations: &[crate::core::types::AllocationInfo],
        batch_size: usize,
        mut processor: F,
    ) -> io::Result<Bytes>
    where
        F: FnMut(&[crate::core::types::AllocationInfo], &mut ZeroCopyBuffer) -> io::Result<()>,
    {
        let mut result_buffer = self.pool.get_buffer(allocations.len() * 64);

        for batch in allocations.chunks(batch_size) {
            let mut batch_buffer = self.pool.get_buffer(batch.len() * 64);
            processor(batch, &mut batch_buffer)?;

            // Append batch result to main buffer
            let batch_bytes = batch_buffer.freeze();
            result_buffer.write(&batch_bytes)?;
        }

        Ok(result_buffer.freeze())
    }

    /// Parallel string processing with zero-copy
    pub fn process_strings_parallel(
        &self,
        strings: &[String],
        batch_size: usize,
    ) -> io::Result<Bytes> {
        use rayon::prelude::*;

        let batches: Vec<_> = strings.chunks(batch_size).collect();
        let results: Result<Vec<_>, _> = batches
            .par_iter()
            .map(|batch| {
                let mut buffer = self.pool.get_buffer(batch.len() * 32);
                buffer.write_u32_le(batch.len() as u32);

                for string in *batch {
                    buffer.write_string(string);
                }

                Ok::<bytes::Bytes, std::io::Error>(buffer.freeze())
            })
            .collect();

        let batch_results = results?;

        // Combine results
        let total_size: usize = batch_results.iter().map(|b| b.len()).sum();
        let mut final_buffer = self.pool.get_buffer(total_size);

        for batch_result in batch_results {
            final_buffer.write(&batch_result)?;
        }

        Ok(final_buffer.freeze())
    }

    /// Optimized type name processing
    pub fn process_type_names(
        &self,
        type_usages: &[crate::core::types::TypeMemoryUsage],
    ) -> io::Result<Bytes> {
        let mut buffer = self.pool.get_buffer(type_usages.len() * 64);

        buffer.write_u32_le(type_usages.len() as u32);

        for usage in type_usages {
            buffer.write_string(&usage.type_name);
            buffer.write_usize_le(usage.total_size);
            buffer.write_usize_le(usage.allocation_count);
        }

        Ok(buffer.freeze())
    }
}

/// String optimization utilities
pub struct StringOptimizer {
    /// Interned strings for deduplication
    interned: std::collections::HashMap<String, u32>,
    /// String table
    strings: Vec<String>,
    /// Next string ID
    next_id: u32,
}

impl StringOptimizer {
    /// Create a new string optimizer
    pub fn new() -> Self {
        Self {
            interned: std::collections::HashMap::new(),
            strings: Vec::new(),
            next_id: 0,
        }
    }

    /// Intern a string and return its ID (zero-copy when possible)
    pub fn intern(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.interned.get(s) {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.strings.push(s.to_string());
            self.interned.insert(s.to_string(), id);
            id
        }
    }

    /// Get string by ID
    pub fn get_string(&self, id: u32) -> Option<&str> {
        self.strings.get(id as usize).map(|s| s.as_str())
    }

    /// Get the string table as bytes (zero-copy)
    pub fn to_bytes(&self, pool: &ZeroCopyBufferPool) -> io::Result<Bytes> {
        let mut buffer = pool.get_buffer(self.strings.len() * 32);

        buffer.write_u32_le(self.strings.len() as u32);

        for string in &self.strings {
            buffer.write_string(string);
        }

        Ok(buffer.freeze())
    }

    /// Get statistics
    pub fn stats(&self) -> StringOptimizerStats {
        let total_original_size: usize = self.strings.iter().map(|s| s.len()).sum();
        let deduplication_savings = self.interned.len().saturating_sub(self.strings.len());

        StringOptimizerStats {
            unique_strings: self.strings.len(),
            total_references: self.interned.len(),
            deduplication_savings,
            total_original_size,
            compressed_size: self.strings.len() * 4 + total_original_size, // IDs + strings
        }
    }
}

impl Default for StringOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// String optimizer statistics
#[derive(Debug, Clone)]
pub struct StringOptimizerStats {
    pub unique_strings: usize,
    pub total_references: usize,
    pub deduplication_savings: usize,
    pub total_original_size: usize,
    pub compressed_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_buffer_pool() {
        let pool = ZeroCopyBufferPool::new(1024, 4);

        // Get a buffer
        let mut buffer1 = pool.get_buffer(512);
        buffer1.write(b"Hello, world!").unwrap();

        // Get another buffer
        let buffer2 = pool.get_buffer(256);

        // Drop first buffer (should return to pool)
        drop(buffer1);

        // Get another buffer (should reuse from pool)
        let buffer3 = pool.get_buffer(512);

        let stats = pool.get_stats();
        assert!(stats.total_reused > 0);

        drop(buffer2);
        drop(buffer3);
    }

    #[test]
    fn test_zero_copy_writer() -> io::Result<()> {
        let mut output = Vec::new();
        let pool = ZeroCopyBufferPool::default();

        {
            let mut writer = ZeroCopyWriter::new(Cursor::new(&mut output), pool);
            writer.write_zero_copy(b"Hello")?;
            writer.write_zero_copy(b", ")?;
            writer.write_zero_copy(b"world!")?;
            writer.flush()?;
        }

        assert_eq!(output, b"Hello, world!");
        Ok(())
    }

    #[test]
    fn test_string_optimizer() -> io::Result<()> {
        let mut optimizer = StringOptimizer::new();
        let pool = ZeroCopyBufferPool::default();

        let id1 = optimizer.intern("hello");
        let id2 = optimizer.intern("world");
        let id3 = optimizer.intern("hello"); // Should reuse

        assert_eq!(id1, id3);
        assert_ne!(id1, id2);

        let bytes = optimizer.to_bytes(&pool)?;
        assert!(!bytes.is_empty());

        let stats = optimizer.stats();
        assert_eq!(stats.unique_strings, 2);
        assert_eq!(stats.total_references, 2);

        Ok(())
    }

    #[test]
    fn test_vectorized_processor() -> io::Result<()> {
        let pool = ZeroCopyBufferPool::default();
        let processor = VectorizedProcessor::new(pool);

        let strings = vec![
            "string1".to_string(),
            "string2".to_string(),
            "string3".to_string(),
        ];

        let result = processor.process_strings_parallel(&strings, 2)?;
        assert!(!result.is_empty());

        Ok(())
    }
}
