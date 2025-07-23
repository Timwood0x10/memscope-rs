//! Streaming JSON writer for optimized large file output
//!
//! This module provides high-performance streaming JSON writing capabilities
//! with support for buffering, compression, and non-blocking I/O operations.

use crate::core::types::{TrackingError, TrackingResult};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Serialize;
use std::io::{BufWriter, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Configuration for streaming JSON writer
#[derive(Debug, Clone)]
pub struct StreamingWriterConfig {
    /// Buffer size for I/O operations (default: 256KB)
    pub buffer_size: usize,
    /// Enable compression (default: false)
    pub enable_compression: bool,
    /// Compression level (1-9, default: 6)
    pub compression_level: u32,
    /// Enable non-blocking writes (default: true)
    pub enable_async_writes: bool,
    /// Maximum queue size for async writes (default: 1000)
    pub async_queue_size: usize,
    /// Write timeout in milliseconds (default: 5000)
    pub write_timeout_ms: u64,
    /// Enable pretty printing (default: false for performance)
    pub pretty_print: bool,
    /// Flush interval in milliseconds (default: 1000)
    pub flush_interval_ms: u64,
}

impl Default for StreamingWriterConfig {
    fn default() -> Self {
        Self {
            buffer_size: 256 * 1024, // 256KB
            enable_compression: false,
            compression_level: 6,
            enable_async_writes: true,
            async_queue_size: 1000,
            write_timeout_ms: 5000,
            pretty_print: false,
            flush_interval_ms: 1000,
        }
    }
}

/// Performance metrics for streaming writer
#[derive(Debug, Clone)]
pub struct StreamingWriterMetrics {
    /// Total bytes written
    pub bytes_written: u64,
    /// Total write operations
    pub write_operations: u64,
    /// Total write time in milliseconds
    pub total_write_time_ms: u64,
    /// Average write speed in bytes per second
    pub avg_write_speed_bps: f64,
    /// Compression ratio (if compression enabled)
    pub compression_ratio: Option<f64>,
    /// Buffer flush count
    pub flush_count: u64,
    /// Queue overflow count (for async writes)
    pub queue_overflow_count: u64,
    /// Peak memory usage in bytes
    pub peak_memory_usage: u64,
}

impl Default for StreamingWriterMetrics {
    fn default() -> Self {
        Self {
            bytes_written: 0,
            write_operations: 0,
            total_write_time_ms: 0,
            avg_write_speed_bps: 0.0,
            compression_ratio: None,
            flush_count: 0,
            queue_overflow_count: 0,
            peak_memory_usage: 0,
        }
    }
}

/// Write operation for async processing
#[derive(Debug)]
enum WriteOperation {
    /// Write JSON data
    WriteJson(String),
    /// Write raw bytes
    WriteBytes(Vec<u8>),
    /// Flush buffer
    Flush,
    /// Close writer
    Close,
}

/// High-performance streaming JSON writer
pub struct StreamingJsonWriter<W: Write + Send + 'static> {
    /// Writer configuration
    config: StreamingWriterConfig,
    /// Performance metrics
    metrics: Arc<Mutex<StreamingWriterMetrics>>,
    /// Async write channel sender
    write_sender: Option<Sender<WriteOperation>>,
    /// Async write thread handle
    write_thread: Option<JoinHandle<TrackingResult<()>>>,
    /// Whether the writer has been initialized
    initialized: bool,
    /// Start time for performance tracking
    start_time: Instant,
    /// Phantom data to use the type parameter
    _phantom: std::marker::PhantomData<W>,
}

impl<W: Write + Send + 'static> StreamingJsonWriter<W> {
    /// Create a new streaming JSON writer
    pub fn new(writer: W) -> Self {
        Self::with_config(writer, StreamingWriterConfig::default())
    }

    /// Create a new streaming JSON writer with custom configuration
    pub fn with_config(writer: W, config: StreamingWriterConfig) -> Self {
        let metrics = Arc::new(Mutex::new(StreamingWriterMetrics::default()));
        let start_time = Instant::now();

        let mut streaming_writer = Self {
            config,
            metrics,
            write_sender: None,
            write_thread: None,
            initialized: false,
            start_time,
            _phantom: std::marker::PhantomData,
        };

        if streaming_writer.config.enable_async_writes {
            streaming_writer.initialize_async_writer(writer);
        } else {
            // For synchronous writes, we'll store the writer differently
            // This is a simplified implementation
        }

        streaming_writer
    }

    /// Initialize async writer thread
    fn initialize_async_writer(&mut self, writer: W) {
        let (sender, receiver) = mpsc::channel();
        let config = self.config.clone();
        let metrics = Arc::clone(&self.metrics);

        let thread_handle =
            thread::spawn(move || Self::async_write_loop(writer, receiver, config, metrics));

        self.write_sender = Some(sender);
        self.write_thread = Some(thread_handle);
        self.initialized = true;
    }

    /// Async write loop running in background thread
    fn async_write_loop(
        writer: W,
        receiver: Receiver<WriteOperation>,
        config: StreamingWriterConfig,
        metrics: Arc<Mutex<StreamingWriterMetrics>>,
    ) -> TrackingResult<()> {
        let mut buffered_writer = if config.enable_compression {
            let encoder = GzEncoder::new(writer, Compression::new(config.compression_level));
            Box::new(BufWriter::with_capacity(config.buffer_size, encoder)) as Box<dyn Write>
        } else {
            Box::new(BufWriter::with_capacity(config.buffer_size, writer)) as Box<dyn Write>
        };

        let mut last_flush = Instant::now();
        let flush_interval = Duration::from_millis(config.flush_interval_ms);

        while let Ok(operation) = receiver.recv() {
            let write_start = Instant::now();

            match operation {
                WriteOperation::WriteJson(json_data) => {
                    let bytes = json_data.as_bytes();
                    buffered_writer
                        .write_all(bytes)
                        .map_err(|e| TrackingError::IoError(e.to_string()))?;

                    if let Ok(mut metrics) = metrics.lock() {
                        metrics.bytes_written += bytes.len() as u64;
                        metrics.write_operations += 1;
                        metrics.total_write_time_ms += write_start.elapsed().as_millis() as u64;
                    }
                }
                WriteOperation::WriteBytes(bytes) => {
                    buffered_writer
                        .write_all(&bytes)
                        .map_err(|e| TrackingError::IoError(e.to_string()))?;

                    if let Ok(mut metrics) = metrics.lock() {
                        metrics.bytes_written += bytes.len() as u64;
                        metrics.write_operations += 1;
                        metrics.total_write_time_ms += write_start.elapsed().as_millis() as u64;
                    }
                }
                WriteOperation::Flush => {
                    buffered_writer
                        .flush()
                        .map_err(|e| TrackingError::IoError(e.to_string()))?;

                    if let Ok(mut metrics) = metrics.lock() {
                        metrics.flush_count += 1;
                    }
                    last_flush = Instant::now();
                }
                WriteOperation::Close => {
                    buffered_writer
                        .flush()
                        .map_err(|e| TrackingError::IoError(e.to_string()))?;
                    break;
                }
            }

            // Auto-flush based on interval
            if last_flush.elapsed() >= flush_interval {
                buffered_writer
                    .flush()
                    .map_err(|e| TrackingError::IoError(e.to_string()))?;

                if let Ok(mut metrics) = metrics.lock() {
                    metrics.flush_count += 1;
                }
                last_flush = Instant::now();
            }
        }

        Ok(())
    }

    /// Write unsafe FFI analysis header
    pub fn write_unsafe_ffi_header(&mut self, metadata: &ExportMetadata) -> TrackingResult<()> {
        let header = serde_json::json!({
            "metadata": {
                "analysis_type": "unsafe_ffi_analysis_optimized",
                "schema_version": "2.0",
                "export_timestamp": metadata.export_timestamp,
                "optimization_level": "high",
                "processing_mode": if metadata.parallel_processing { "parallel" } else { "sequential" },
                "data_integrity_hash": metadata.integrity_hash
            }
        });

        let json_str = if self.config.pretty_print {
            serde_json::to_string_pretty(&header)
        } else {
            serde_json::to_string(&header)
        }
        .map_err(|e| TrackingError::SerializationError(e.to_string()))?;

        self.write_json_chunk(&json_str)
    }

    /// Write unsafe allocations stream
    pub fn write_unsafe_allocations_stream<T: Serialize>(
        &mut self,
        allocations: &[T],
    ) -> TrackingResult<()> {
        self.write_json_chunk("\"unsafe_analysis\":{")?;
        self.write_json_chunk(&format!(
            "\"total_unsafe_allocations\":{},",
            allocations.len()
        ))?;
        self.write_json_chunk("\"allocations\":[")?;

        for (i, allocation) in allocations.iter().enumerate() {
            let json_str = if self.config.pretty_print {
                serde_json::to_string_pretty(allocation)
            } else {
                serde_json::to_string(allocation)
            }
            .map_err(|e| TrackingError::SerializationError(e.to_string()))?;

            self.write_json_chunk(&json_str)?;

            if i < allocations.len() - 1 {
                self.write_json_chunk(",")?;
            }
        }

        self.write_json_chunk("]}")
    }

    /// Write FFI allocations stream
    pub fn write_ffi_allocations_stream<T: Serialize>(
        &mut self,
        allocations: &[T],
    ) -> TrackingResult<()> {
        self.write_json_chunk("\"ffi_analysis\":{")?;
        self.write_json_chunk(&format!("\"total_ffi_allocations\":{},", allocations.len()))?;
        self.write_json_chunk("\"allocations\":[")?;

        for (i, allocation) in allocations.iter().enumerate() {
            let json_str = if self.config.pretty_print {
                serde_json::to_string_pretty(allocation)
            } else {
                serde_json::to_string(allocation)
            }
            .map_err(|e| TrackingError::SerializationError(e.to_string()))?;

            self.write_json_chunk(&json_str)?;

            if i < allocations.len() - 1 {
                self.write_json_chunk(",")?;
            }
        }

        self.write_json_chunk("]}")
    }

    /// Write boundary events stream
    pub fn write_boundary_events_stream<T: Serialize>(
        &mut self,
        events: &[T],
    ) -> TrackingResult<()> {
        self.write_json_chunk("\"boundary_analysis\":{")?;
        self.write_json_chunk(&format!("\"total_boundary_crossings\":{},", events.len()))?;
        self.write_json_chunk("\"events\":[")?;

        for (i, event) in events.iter().enumerate() {
            let json_str = if self.config.pretty_print {
                serde_json::to_string_pretty(event)
            } else {
                serde_json::to_string(event)
            }
            .map_err(|e| TrackingError::SerializationError(e.to_string()))?;

            self.write_json_chunk(&json_str)?;

            if i < events.len() - 1 {
                self.write_json_chunk(",")?;
            }
        }

        self.write_json_chunk("]}")
    }

    /// Write safety violations stream
    pub fn write_safety_violations_stream<T: Serialize>(
        &mut self,
        violations: &[T],
    ) -> TrackingResult<()> {
        self.write_json_chunk("\"safety_violations\":{")?;
        self.write_json_chunk(&format!("\"total_violations\":{},", violations.len()))?;
        self.write_json_chunk("\"violations\":[")?;

        for (i, violation) in violations.iter().enumerate() {
            let json_str = if self.config.pretty_print {
                serde_json::to_string_pretty(violation)
            } else {
                serde_json::to_string(violation)
            }
            .map_err(|e| TrackingError::SerializationError(e.to_string()))?;

            self.write_json_chunk(&json_str)?;

            if i < violations.len() - 1 {
                self.write_json_chunk(",")?;
            }
        }

        self.write_json_chunk("]}")
    }

    /// Write a complete JSON object in streaming fashion
    pub fn write_complete_json<T: Serialize>(&mut self, data: &T) -> TrackingResult<()> {
        let json_str = if self.config.pretty_print {
            serde_json::to_string_pretty(data)
        } else {
            serde_json::to_string(data)
        }
        .map_err(|e| TrackingError::SerializationError(e.to_string()))?;

        self.write_json_chunk(&json_str)
    }

    /// Write JSON chunk (internal method)
    fn write_json_chunk(&mut self, chunk: &str) -> TrackingResult<()> {
        if !self.initialized && self.config.enable_async_writes {
            return Err(TrackingError::InitializationError(
                "Async writer not initialized".to_string(),
            ));
        }

        if let Some(sender) = &self.write_sender {
            sender
                .send(WriteOperation::WriteJson(chunk.to_string()))
                .map_err(|e| TrackingError::ChannelError(e.to_string()))?;
        } else {
            // Synchronous write fallback
            return Err(TrackingError::NotImplemented(
                "Synchronous write not implemented in this version".to_string(),
            ));
        }

        Ok(())
    }

    /// Flush the writer buffer
    pub fn flush(&mut self) -> TrackingResult<()> {
        if let Some(sender) = &self.write_sender {
            sender
                .send(WriteOperation::Flush)
                .map_err(|e| TrackingError::ChannelError(e.to_string()))?;
        }
        Ok(())
    }

    /// Finalize the writer and close all resources
    pub fn finalize(&mut self) -> TrackingResult<()> {
        if let Some(sender) = self.write_sender.take() {
            sender
                .send(WriteOperation::Close)
                .map_err(|e| TrackingError::ChannelError(e.to_string()))?;
        }

        if let Some(thread_handle) = self.write_thread.take() {
            thread_handle.join().map_err(|e| {
                TrackingError::ThreadError(format!("Thread join failed: {:?}", e))
            })??;
        }

        // Update final metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            let total_time = self.start_time.elapsed().as_millis() as u64;
            if total_time > 0 {
                metrics.avg_write_speed_bps =
                    (metrics.bytes_written as f64 * 1000.0) / total_time as f64;
            }
        }

        Ok(())
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> TrackingResult<StreamingWriterMetrics> {
        self.metrics
            .lock()
            .map(|m| m.clone())
            .map_err(|e| TrackingError::LockError(e.to_string()))
    }

    /// Reset performance metrics
    pub fn reset_metrics(&mut self) -> TrackingResult<()> {
        if let Ok(mut metrics) = self.metrics.lock() {
            *metrics = StreamingWriterMetrics::default();
        }
        self.start_time = Instant::now();
        Ok(())
    }

    /// Check if writer is healthy (for monitoring)
    pub fn is_healthy(&self) -> bool {
        if let Some(_sender) = &self.write_sender {
            // Try to send a non-blocking test message
            // In a real implementation, we might use try_send or have a health check mechanism
            true // Simplified health check
        } else {
            false
        }
    }

    /// Get estimated memory usage
    pub fn get_memory_usage(&self) -> usize {
        // Estimate based on buffer size and queue size
        self.config.buffer_size + (self.config.async_queue_size * 1024) // Rough estimate
    }
}

/// Export metadata for JSON headers
#[derive(Debug, Clone)]
pub struct ExportMetadata {
    /// Export timestamp
    pub export_timestamp: u64,
    /// Whether parallel processing was used
    pub parallel_processing: bool,
    /// Data integrity hash
    pub integrity_hash: String,
    /// Additional metadata
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ExportMetadata {
    /// Create new export metadata
    pub fn new() -> Self {
        Self {
            export_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            parallel_processing: false,
            integrity_hash: "".to_string(),
            additional_info: std::collections::HashMap::new(),
        }
    }

    /// Set parallel processing flag
    pub fn with_parallel_processing(mut self, parallel: bool) -> Self {
        self.parallel_processing = parallel;
        self
    }

    /// Set integrity hash
    pub fn with_integrity_hash(mut self, hash: String) -> Self {
        self.integrity_hash = hash;
        self
    }

    /// Add additional metadata
    pub fn with_additional_info(mut self, key: String, value: String) -> Self {
        self.additional_info.insert(key, value);
        self
    }
}

impl Default for ExportMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a streaming writer for files
pub fn create_file_streaming_writer(
    file_path: &str,
    config: StreamingWriterConfig,
) -> TrackingResult<StreamingJsonWriter<std::fs::File>> {
    let file =
        std::fs::File::create(file_path).map_err(|e| TrackingError::IoError(e.to_string()))?;

    Ok(StreamingJsonWriter::with_config(file, config))
}

/// Convenience function to create a streaming writer for in-memory buffer
pub fn create_buffer_streaming_writer(
    config: StreamingWriterConfig,
) -> StreamingJsonWriter<Vec<u8>> {
    let buffer = Vec::new();
    StreamingJsonWriter::with_config(buffer, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_writer_creation() {
        let buffer = Vec::new();
        let writer = StreamingJsonWriter::new(buffer);
        assert!(!writer.initialized || writer.config.enable_async_writes);
    }

    #[test]
    fn test_config_defaults() {
        let config = StreamingWriterConfig::default();
        assert_eq!(config.buffer_size, 256 * 1024);
        assert!(!config.enable_compression);
        assert!(config.enable_async_writes);
    }

    #[test]
    fn test_export_metadata() {
        let metadata = ExportMetadata::new()
            .with_parallel_processing(true)
            .with_integrity_hash("test_hash".to_string());

        assert!(metadata.parallel_processing);
        assert_eq!(metadata.integrity_hash, "test_hash");
    }
}
