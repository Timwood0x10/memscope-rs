//! Memory mapping optimizations for binary export system
//!
//! This module provides memory-mapped file I/O optimizations for handling large binary files
//! efficiently. It includes platform-specific optimizations and memory usage monitoring.

use std::fs::{File, OpenOptions};
use std::io::{self, Result as IoResult, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[cfg(unix)]
use std::os::unix::fs::FileExt;
#[cfg(windows)]
use std::os::windows::fs::FileExt;

/// Memory mapping configuration and limits
#[derive(Debug, Clone)]
pub struct MemoryMappingConfig {
    /// Maximum memory usage for memory mapping (bytes)
    pub max_memory_usage: usize,
    /// Minimum file size threshold for using memory mapping
    pub min_file_size_for_mmap: usize,
    /// Page size for memory mapping alignment
    pub page_size: usize,
    /// Enable read-ahead optimization
    pub enable_read_ahead: bool,
    /// Read-ahead window size (bytes)
    pub read_ahead_size: usize,
    /// Enable write-behind optimization for output files
    pub enable_write_behind: bool,
    /// Write buffer size for buffered writes
    pub write_buffer_size: usize,
}

impl Default for MemoryMappingConfig {
    fn default() -> Self {
        Self {
            max_memory_usage: 1024 * 1024 * 1024, // 1GB default limit
            min_file_size_for_mmap: 64 * 1024,     // 64KB minimum
            page_size: get_page_size(),
            enable_read_ahead: true,
            read_ahead_size: 256 * 1024, // 256KB read-ahead
            enable_write_behind: true,
            write_buffer_size: 1024 * 1024, // 1MB write buffer
        }
    }
}

/// Memory usage monitor for tracking memory mapping usage
#[derive(Debug, Clone)]
pub struct MemoryUsageMonitor {
    /// Current memory usage in bytes
    current_usage: Arc<AtomicUsize>,
    /// Peak memory usage in bytes
    peak_usage: Arc<AtomicUsize>,
    /// Maximum allowed memory usage
    max_usage: usize,
}

impl MemoryUsageMonitor {
    /// Create a new memory usage monitor
    pub fn new(max_usage: usize) -> Self {
        Self {
            current_usage: Arc::new(AtomicUsize::new(0)),
            peak_usage: Arc::new(AtomicUsize::new(0)),
            max_usage,
        }
    }

    /// Allocate memory and track usage
    pub fn allocate(&self, size: usize) -> Result<(), MemoryMappingError> {
        let current = self.current_usage.fetch_add(size, Ordering::SeqCst) + size;
        
        if current > self.max_usage {
            // Rollback allocation
            self.current_usage.fetch_sub(size, Ordering::SeqCst);
            return Err(MemoryMappingError::MemoryLimitExceeded {
                requested: size,
                current: current - size,
                limit: self.max_usage,
            });
        }

        // Update peak usage
        let mut peak = self.peak_usage.load(Ordering::SeqCst);
        while current > peak {
            match self.peak_usage.compare_exchange_weak(
                peak,
                current,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(new_peak) => peak = new_peak,
            }
        }

        Ok(())
    }

    /// Deallocate memory and update tracking
    pub fn deallocate(&self, size: usize) {
        self.current_usage.fetch_sub(size, Ordering::SeqCst);
    }

    /// Get current memory usage
    pub fn current_usage(&self) -> usize {
        self.current_usage.load(Ordering::SeqCst)
    }

    /// Get peak memory usage
    pub fn peak_usage(&self) -> usize {
        self.peak_usage.load(Ordering::SeqCst)
    }

    /// Get memory usage statistics
    pub fn get_stats(&self) -> MemoryUsageStats {
        MemoryUsageStats {
            current_usage: self.current_usage(),
            peak_usage: self.peak_usage(),
            max_usage: self.max_usage,
            utilization_percent: (self.current_usage() as f64 / self.max_usage as f64) * 100.0,
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryUsageStats {
    pub current_usage: usize,
    pub peak_usage: usize,
    pub max_usage: usize,
    pub utilization_percent: f64,
}

/// Memory-mapped file reader with optimizations
pub struct MemoryMappedReader {
    file: File,
    file_size: u64,
    config: MemoryMappingConfig,
    monitor: MemoryUsageMonitor,
    #[cfg(unix)]
    mmap: Option<memmap2::Mmap>,
    #[cfg(windows)]
    mmap: Option<memmap2::Mmap>,
}

impl MemoryMappedReader {
    /// Create a new memory-mapped reader
    pub fn new<P: AsRef<Path>>(
        path: P,
        config: MemoryMappingConfig,
        monitor: MemoryUsageMonitor,
    ) -> Result<Self, MemoryMappingError> {
        let file = File::open(path.as_ref())?;
        let file_size = file.metadata()?.len();

        let mut reader = Self {
            file,
            file_size,
            config,
            monitor,
            mmap: None,
        };

        // Initialize memory mapping if file is large enough
        if file_size >= reader.config.min_file_size_for_mmap as u64 {
            reader.init_memory_mapping()?;
        }

        Ok(reader)
    }

    /// Initialize memory mapping for the file
    fn init_memory_mapping(&mut self) -> Result<(), MemoryMappingError> {
        // Check if we can allocate memory for mapping
        self.monitor.allocate(self.file_size as usize)?;

        // Create memory mapping
        let mmap = unsafe { memmap2::Mmap::map(&self.file)? };
        
        // Platform-specific optimizations
        #[cfg(unix)]
        self.apply_unix_optimizations(&mmap)?;
        
        #[cfg(windows)]
        self.apply_windows_optimizations(&mmap)?;

        self.mmap = Some(mmap);
        Ok(())
    }

    /// Apply Unix-specific optimizations
    #[cfg(unix)]
    fn apply_unix_optimizations(&self, mmap: &memmap2::Mmap) -> Result<(), MemoryMappingError> {
        use std::os::unix::io::AsRawFd;
        
        // Advise kernel about access patterns
        if self.config.enable_read_ahead {
            unsafe {
                libc::madvise(
                    mmap.as_ptr() as *mut libc::c_void,
                    mmap.len(),
                    libc::MADV_SEQUENTIAL | libc::MADV_WILLNEED,
                );
            }
        }

        // Set read-ahead for the file descriptor
        if self.config.enable_read_ahead {
            unsafe {
                libc::posix_fadvise(
                    self.file.as_raw_fd(),
                    0,
                    self.file_size as i64,
                    libc::POSIX_FADV_SEQUENTIAL,
                );
            }
        }

        Ok(())
    }

    /// Apply Windows-specific optimizations
    #[cfg(windows)]
    fn apply_windows_optimizations(&self, _mmap: &memmap2::Mmap) -> Result<(), MemoryMappingError> {
        // Windows-specific optimizations can be added here
        // For now, we rely on the OS defaults
        Ok(())
    }

    /// Read data from the file using optimal method
    pub fn read_at(&self, offset: u64, buffer: &mut [u8]) -> IoResult<usize> {
        if let Some(ref mmap) = self.mmap {
            // Use memory mapping for reads
            let start = offset as usize;
            let end = std::cmp::min(start + buffer.len(), mmap.len());
            
            if start >= mmap.len() {
                return Ok(0);
            }

            let bytes_to_copy = end - start;
            buffer[..bytes_to_copy].copy_from_slice(&mmap[start..end]);
            Ok(bytes_to_copy)
        } else {
            // Fall back to regular file I/O
            #[cfg(unix)]
            {
                self.file.read_at(buffer, offset)
            }
            #[cfg(windows)]
            {
                self.file.seek_read(buffer, offset)
            }
            #[cfg(not(any(unix, windows)))]
            {
                // Generic implementation for other platforms
                use std::io::{Read, Seek};
                let mut file = &self.file;
                file.seek(SeekFrom::Start(offset))?;
                file.read(buffer)
            }
        }
    }

    /// Read exact amount of data
    pub fn read_exact_at(&self, offset: u64, buffer: &mut [u8]) -> IoResult<()> {
        let mut bytes_read = 0;
        while bytes_read < buffer.len() {
            let n = self.read_at(offset + bytes_read as u64, &mut buffer[bytes_read..])?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Failed to read exact amount of data",
                ));
            }
            bytes_read += n;
        }
        Ok(())
    }

    /// Get file size
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    /// Check if memory mapping is active
    pub fn is_memory_mapped(&self) -> bool {
        self.mmap.is_some()
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryUsageStats {
        self.monitor.get_stats()
    }
}

impl Drop for MemoryMappedReader {
    fn drop(&mut self) {
        if self.mmap.is_some() {
            self.monitor.deallocate(self.file_size as usize);
        }
    }
}

/// Memory-mapped file writer with optimizations
pub struct MemoryMappedWriter {
    file: File,
    config: MemoryMappingConfig,
    monitor: MemoryUsageMonitor,
    write_buffer: Vec<u8>,
    buffer_position: usize,
}

impl MemoryMappedWriter {
    /// Create a new memory-mapped writer
    pub fn new<P: AsRef<Path>>(
        path: P,
        config: MemoryMappingConfig,
        monitor: MemoryUsageMonitor,
    ) -> Result<Self, MemoryMappingError> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path.as_ref())?;

        // Allocate write buffer
        monitor.allocate(config.write_buffer_size)?;

        Ok(Self {
            file,
            config: config.clone(),
            monitor,
            write_buffer: vec![0u8; config.write_buffer_size],
            buffer_position: 0,
        })
    }

    /// Write data with buffering optimization
    pub fn write(&mut self, data: &[u8]) -> IoResult<()> {
        let mut remaining = data;

        while !remaining.is_empty() {
            let buffer_space = self.write_buffer.len() - self.buffer_position;
            
            if buffer_space == 0 {
                // Buffer is full, flush it
                self.flush_buffer()?;
                continue;
            }

            let bytes_to_copy = std::cmp::min(remaining.len(), buffer_space);
            let end_pos = self.buffer_position + bytes_to_copy;
            
            self.write_buffer[self.buffer_position..end_pos]
                .copy_from_slice(&remaining[..bytes_to_copy]);
            
            self.buffer_position += bytes_to_copy;
            remaining = &remaining[bytes_to_copy..];
        }

        Ok(())
    }

    /// Write data at specific offset (bypasses buffer)
    pub fn write_at(&mut self, offset: u64, data: &[u8]) -> IoResult<()> {
        // Flush buffer first to maintain consistency
        self.flush_buffer()?;

        #[cfg(unix)]
        {
            self.file.write_all_at(data, offset)
        }
        #[cfg(windows)]
        {
            self.file.seek_write(data, offset).map(|_| ())
        }
        #[cfg(not(any(unix, windows)))]
        {
            use std::io::{Seek, Write};
            self.file.seek(SeekFrom::Start(offset))?;
            self.file.write_all(data)
        }
    }

    /// Flush the write buffer to disk
    pub fn flush_buffer(&mut self) -> IoResult<()> {
        if self.buffer_position > 0 {
            self.file.write_all(&self.write_buffer[..self.buffer_position])?;
            self.buffer_position = 0;
        }
        Ok(())
    }

    /// Flush all data to disk
    pub fn flush(&mut self) -> IoResult<()> {
        self.flush_buffer()?;
        self.file.flush()
    }

    /// Sync data to disk
    pub fn sync_all(&mut self) -> IoResult<()> {
        self.flush()?;
        self.file.sync_all()
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryUsageStats {
        self.monitor.get_stats()
    }
}

impl Drop for MemoryMappedWriter {
    fn drop(&mut self) {
        let _ = self.flush_buffer();
        self.monitor.deallocate(self.config.write_buffer_size);
    }
}

/// Memory mapping errors
#[derive(Debug, thiserror::Error)]
pub enum MemoryMappingError {
    #[error("Memory limit exceeded: requested {requested} bytes, current usage {current}, limit {limit}")]
    MemoryLimitExceeded {
        requested: usize,
        current: usize,
        limit: usize,
    },

    #[error("Memory mapping failed: {0}")]
    MappingFailed(String),

    #[error("Platform not supported for memory mapping optimizations")]
    PlatformNotSupported,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Memory mapping error: {0}")]
    Mmap(#[from] memmap2::Error),
}

/// Get system page size
fn get_page_size() -> usize {
    #[cfg(unix)]
    {
        unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
    }
    #[cfg(windows)]
    {
        use winapi::um::sysinfoapi::{GetSystemInfo, SYSTEM_INFO};
        let mut system_info: SYSTEM_INFO = unsafe { std::mem::zeroed() };
        unsafe { GetSystemInfo(&mut system_info) };
        system_info.dwPageSize as usize
    }
    #[cfg(not(any(unix, windows)))]
    {
        4096 // Default page size for other platforms
    }
}

/// Batch memory mapping operations for multiple files
pub struct BatchMemoryMapper {
    config: MemoryMappingConfig,
    monitor: MemoryUsageMonitor,
    readers: Vec<MemoryMappedReader>,
}

impl BatchMemoryMapper {
    /// Create a new batch memory mapper
    pub fn new(config: MemoryMappingConfig) -> Self {
        let monitor = MemoryUsageMonitor::new(config.max_memory_usage);
        
        Self {
            config,
            monitor,
            readers: Vec::new(),
        }
    }

    /// Add a file for memory mapping
    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, MemoryMappingError> {
        let reader = MemoryMappedReader::new(path, self.config.clone(), self.monitor.clone())?;
        let index = self.readers.len();
        self.readers.push(reader);
        Ok(index)
    }

    /// Get a reader by index
    pub fn get_reader(&self, index: usize) -> Option<&MemoryMappedReader> {
        self.readers.get(index)
    }

    /// Get memory usage statistics for all files
    pub fn total_memory_stats(&self) -> MemoryUsageStats {
        self.monitor.get_stats()
    }

    /// Get number of memory-mapped files
    pub fn mapped_file_count(&self) -> usize {
        self.readers.iter().filter(|r| r.is_memory_mapped()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_memory_usage_monitor() {
        let monitor = MemoryUsageMonitor::new(1000);
        
        // Test allocation
        assert!(monitor.allocate(500).is_ok());
        assert_eq!(monitor.current_usage(), 500);
        
        // Test over-allocation
        assert!(monitor.allocate(600).is_err());
        assert_eq!(monitor.current_usage(), 500);
        
        // Test deallocation
        monitor.deallocate(200);
        assert_eq!(monitor.current_usage(), 300);
        
        // Test peak usage
        assert_eq!(monitor.peak_usage(), 500);
    }

    #[test]
    fn test_memory_mapped_reader() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"Hello, memory mapping world!";
        temp_file.write_all(test_data)?;
        temp_file.flush()?;

        let config = MemoryMappingConfig::default();
        let monitor = MemoryUsageMonitor::new(config.max_memory_usage);
        
        let reader = MemoryMappedReader::new(temp_file.path(), config, monitor)?;
        
        let mut buffer = vec![0u8; test_data.len()];
        let bytes_read = reader.read_at(0, &mut buffer)?;
        
        assert_eq!(bytes_read, test_data.len());
        assert_eq!(&buffer, test_data);
        
        Ok(())
    }

    #[test]
    fn test_memory_mapped_writer() -> Result<(), Box<dyn std::error::Error>> {
        let temp_file = NamedTempFile::new()?;
        let test_data = b"Hello, memory mapping writer!";

        let config = MemoryMappingConfig::default();
        let monitor = MemoryUsageMonitor::new(config.max_memory_usage);
        
        {
            let mut writer = MemoryMappedWriter::new(temp_file.path(), config, monitor)?;
            writer.write(test_data)?;
            writer.flush()?;
        }

        // Verify written data
        let written_data = std::fs::read(temp_file.path())?;
        assert_eq!(&written_data, test_data);
        
        Ok(())
    }
}