//! Data processing system for binary export
//!
//! This module provides high-performance data processing capabilities including
//! batch processing, streaming operations, and parallel processing for large datasets.

use std::collections::HashMap;
use std::io::{Read, Write, BufReader, BufWriter};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use crossbeam_queue::SegQueue;
use crossbeam_utils::thread;
use rand::seq::SliceRandom;

use super::core::UnifiedData;
use super::error::BinaryExportError;
use super::memory::{MemoryManager, SmartBuffer};

/// Configuration for data processing operations
#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    /// Maximum memory usage during processing (bytes)
    pub max_memory_usage: usize,
    /// Chunk size for streaming operations (bytes)
    pub chunk_size: usize,
    /// Number of worker threads for parallel processing
    pub worker_threads: usize,
    /// Enable memory usage monitoring
    pub monitor_memory: bool,
    /// Processing timeout (seconds)
    pub timeout_secs: u64,
    /// Enable data validation during processing
    pub validate_data: bool,
    /// Buffer size for I/O operations
    pub io_buffer_size: usize,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            max_memory_usage: 512 * 1024 * 1024, // 512MB
            chunk_size: 256 * 1024, // 256KB
            worker_threads: num_cpus::get(),
            monitor_memory: true,
            timeout_secs: 300, // 5 minutes
            validate_data: true,
            io_buffer_size: 64 * 1024, // 64KB
        }
    }
}

impl ProcessingConfig {
    /// Fast processing configuration - optimized for speed
    pub fn fast() -> Self {
        Self {
            max_memory_usage: 256 * 1024 * 1024, // 256MB
            chunk_size: 64 * 1024, // 64KB
            worker_threads: num_cpus::get() * 2,
            monitor_memory: false,
            timeout_secs: 60,
            validate_data: false,
            io_buffer_size: 32 * 1024, // 32KB
        }
    }
    
    /// Memory-efficient configuration - optimized for low memory usage
    pub fn memory_efficient() -> Self {
        Self {
            max_memory_usage: 64 * 1024 * 1024, // 64MB
            chunk_size: 16 * 1024, // 16KB
            worker_threads: 2,
            monitor_memory: true,
            timeout_secs: 600, // 10 minutes
            validate_data: true,
            io_buffer_size: 8 * 1024, // 8KB
        }
    }
}

/// Statistics about data processing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStats {
    /// Total bytes processed
    pub bytes_processed: u64,
    /// Processing duration
    pub duration: Duration,
    /// Processing throughput (bytes/second)
    pub throughput: f64,
    /// Peak memory usage during processing
    pub peak_memory_usage: usize,
    /// Number of chunks processed
    pub chunks_processed: u64,
    /// Number of validation errors encountered
    pub validation_errors: u32,
    /// Processing efficiency (0.0 to 1.0)
    pub efficiency: f64,
}

/// Result of data processing operations
pub type ProcessResult<T> = Result<T, BinaryExportError>;

/// Processed data container with metadata
#[derive(Debug, Clone)]
pub struct ProcessedData {
    /// The processed data payload
    pub data: Vec<u8>,
    /// Processing metadata
    pub metadata: ProcessingMetadata,
    /// Data validation results
    pub validation_results: ValidationResults,
}

/// Metadata about the processing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetadata {
    /// Processing timestamp
    pub timestamp: std::time::SystemTime,
    /// Processing method used
    pub method: ProcessingMethod,
    /// Data format after processing
    pub format: DataFormat,
    /// Compression applied (if any)
    pub compression: Option<super::compression::CompressionAlgorithm>,
    /// Processing configuration used
    pub config_hash: u64,
}

/// Processing methods available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessingMethod {
    /// Batch processing (all data in memory)
    Batch,
    /// Streaming processing (constant memory usage)
    Streaming,
    /// Parallel processing (multi-threaded)
    Parallel,
    /// Hybrid processing (combination of methods)
    Hybrid,
}

/// Data formats supported by the processor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataFormat {
    /// Raw binary data
    Raw,
    /// Serialized with bincode
    Bincode,
    /// MessagePack format
    MessagePack,
    /// Custom binary format
    CustomBinary,
}

/// Data validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation errors found
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Data integrity score (0.0 to 1.0)
    pub integrity_score: f64,
}

/// Main data processor implementation
pub struct DataProcessor {
    /// Processing configuration
    config: ProcessingConfig,
    /// Memory manager for efficient memory usage
    memory_manager: Arc<MemoryManager>,
    /// Processing statistics
    stats: Arc<Mutex<Vec<ProcessStats>>>,
    /// Memory usage monitor
    memory_monitor: MemoryMonitor,
}

/// Memory usage monitoring system
struct MemoryMonitor {
    /// Current memory usage
    current_usage: Arc<Mutex<usize>>,
    /// Peak memory usage
    peak_usage: Arc<Mutex<usize>>,
    /// Memory usage history
    usage_history: Arc<Mutex<Vec<(Instant, usize)>>>,
}

impl DataProcessor {
    /// Create a new data processor with the given configuration
    pub fn new(config: ProcessingConfig) -> Self {
        let memory_manager = Arc::new(MemoryManager::new(config.max_memory_usage));
        let memory_monitor = MemoryMonitor::new();
        
        Self {
            config,
            memory_manager,
            stats: Arc::new(Mutex::new(Vec::new())),
            memory_monitor,
        }
    }

    /// Process data in batch mode (all data in memory)
    pub fn process_batch(&self, data: &UnifiedData) -> ProcessResult<ProcessedData> {
        let start_time = Instant::now();
        self.memory_monitor.start_monitoring();
        
        // Validate input data if enabled
        let validation_results = if self.config.validate_data {
            self.validate_unified_data(data)?
        } else {
            ValidationResults::default()
        };
        
        // Serialize the data
        let serialized_data = self.serialize_data(data, DataFormat::Bincode)?;
        
        // Apply any transformations
        let processed_data = self.apply_transformations(&serialized_data)?;
        
        // Create processing metadata
        let metadata = ProcessingMetadata {
            timestamp: std::time::SystemTime::now(),
            method: ProcessingMethod::Batch,
            format: DataFormat::Bincode,
            compression: None,
            config_hash: self.calculate_config_hash(),
        };
        
        let duration = start_time.elapsed();
        let peak_memory = self.memory_monitor.get_peak_usage();
        
        // Record statistics
        let stats = ProcessStats {
            bytes_processed: processed_data.len() as u64,
            duration,
            throughput: processed_data.len() as f64 / duration.as_secs_f64(),
            peak_memory_usage: peak_memory,
            chunks_processed: 1,
            validation_errors: validation_results.errors.len() as u32,
            efficiency: self.calculate_efficiency(&processed_data, duration),
        };
        
        self.record_stats(stats);
        
        Ok(ProcessedData {
            data: processed_data,
            metadata,
            validation_results,
        })
    }

    /// Process data using parallel processing
    pub fn process_parallel(&self, data: &UnifiedData) -> ProcessResult<ProcessedData> {
        let start_time = Instant::now();
        self.memory_monitor.start_monitoring();
        
        // Validate input data
        let validation_results = if self.config.validate_data {
            self.validate_unified_data(data)?
        } else {
            ValidationResults::default()
        };
        
        // Split data into chunks for parallel processing
        let chunks = self.split_data_for_parallel_processing(data)?;
        
        // Process chunks in parallel using thread pool
        let processed_chunks = self.process_chunks_parallel(chunks)?;
        
        // Merge processed chunks back together
        let merged_data = self.merge_processed_chunks(processed_chunks)?;
        
        // Create processing metadata
        let metadata = ProcessingMetadata {
            timestamp: std::time::SystemTime::now(),
            method: ProcessingMethod::Parallel,
            format: DataFormat::Bincode,
            compression: None,
            config_hash: self.calculate_config_hash(),
        };
        
        let duration = start_time.elapsed();
        let peak_memory = self.memory_monitor.get_peak_usage();
        
        // Record statistics
        let stats = ProcessStats {
            bytes_processed: merged_data.len() as u64,
            duration,
            throughput: merged_data.len() as f64 / duration.as_secs_f64(),
            peak_memory_usage: peak_memory,
            chunks_processed: chunks.len() as u64,
            validation_errors: validation_results.errors.len() as u32,
            efficiency: self.calculate_efficiency(&merged_data, duration),
        };
        
        self.record_stats(stats);
        
        Ok(ProcessedData {
            data: merged_data,
            metadata,
            validation_results,
        })
    }

    /// Add memory usage monitoring and limits
    fn monitor_memory_usage(&self) -> ProcessResult<()> {
        let current_usage = self.memory_monitor.get_current_usage();
        
        if current_usage > self.config.max_memory_usage {
            return Err(BinaryExportError::MemoryLimitExceeded {
                limit: self.config.max_memory_usage,
                usage: current_usage,
            });
        }
        
        Ok(())
    }

    /// Validate unified data structure
    fn validate_unified_data(&self, data: &UnifiedData) -> ProcessResult<ValidationResults> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut integrity_score = 1.0;
        
        // Validate allocation data
        if data.allocations.allocations.is_empty() {
            warnings.push("No allocation data found".to_string());
            integrity_score -= 0.1;
        }
        
        // Validate memory regions
        if data.allocations.regions.is_empty() {
            warnings.push("No memory regions found".to_string());
            integrity_score -= 0.1;
        }
        
        // Validate analysis data consistency
        if let Some(ref lifecycle) = data.analysis.lifecycle {
            if lifecycle.allocation_patterns.is_empty() {
                warnings.push("No lifecycle patterns found".to_string());
                integrity_score -= 0.05;
            }
        }
        
        // Check for data corruption indicators
        if data.metadata.format_version == 0 {
            errors.push("Invalid format version".to_string());
            integrity_score -= 0.3;
        }
        
        integrity_score = integrity_score.max(0.0);
        
        Ok(ValidationResults {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            integrity_score,
        })
    }

    /// Serialize data to bytes using the specified format
    fn serialize_data(&self, data: &UnifiedData, format: DataFormat) -> ProcessResult<Vec<u8>> {
        match format {
            DataFormat::Bincode => {
                bincode::serialize(data)
                    .map_err(|e| BinaryExportError::SerializationError(e.to_string()))
            }
            DataFormat::MessagePack => {
                rmp_serde::to_vec(data)
                    .map_err(|e| BinaryExportError::SerializationError(e.to_string()))
            }
            DataFormat::Raw => {
                // For raw format, we'll use bincode as fallback
                bincode::serialize(data)
                    .map_err(|e| BinaryExportError::SerializationError(e.to_string()))
            }
            DataFormat::CustomBinary => {
                // Custom binary format implementation would go here
                // For now, use bincode as fallback
                bincode::serialize(data)
                    .map_err(|e| BinaryExportError::SerializationError(e.to_string()))
            }
        }
    }

    /// Apply data transformations (placeholder for future enhancements)
    fn apply_transformations(&self, data: &[u8]) -> ProcessResult<Vec<u8>> {
        // For now, just return the data as-is
        // Future transformations could include:
        // - Data deduplication
        // - Compression
        // - Encryption
        // - Format conversion
        Ok(data.to_vec())
    }

    /// Split data into chunks for parallel processing
    fn split_data_for_parallel_processing(&self, data: &UnifiedData) -> ProcessResult<Vec<DataChunk>> {
        let mut chunks = Vec::new();
        
        // Split allocations into chunks
        let allocations_per_chunk = (data.allocations.allocations.len() / self.config.worker_threads).max(1);
        
        for (i, chunk_allocations) in data.allocations.allocations.chunks(allocations_per_chunk).enumerate() {
            let chunk = DataChunk {
                id: i as u64,
                chunk_type: ChunkType::Allocations,
                data: bincode::serialize(chunk_allocations)
                    .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?,
            };
            chunks.push(chunk);
        }
        
        Ok(chunks)
    }

    /// Process chunks in parallel using advanced thread pool
    fn process_chunks_parallel(&self, chunks: Vec<DataChunk>) -> ProcessResult<Vec<ProcessedChunk>> {
        use std::sync::Arc;
        use crossbeam_utils::thread;
        
        let chunk_count = chunks.len();
        let worker_count = self.config.worker_threads.min(chunk_count);
        
        // Create lock-free work queue
        let work_queue = Arc::new(crossbeam_queue::SegQueue::new());
        let results = Arc::new(crossbeam_queue::SegQueue::new());
        
        // Populate work queue
        for (index, chunk) in chunks.into_iter().enumerate() {
            work_queue.push((index, chunk));
        }
        
        // Process chunks in parallel using scoped threads
        thread::scope(|s| {
            let mut handles = Vec::new();
            
            for worker_id in 0..worker_count {
                let work_queue = Arc::clone(&work_queue);
                let results = Arc::clone(&results);
                let config = self.config.clone();
                
                let handle = s.spawn(move |_| {
                    let mut worker_stats = WorkerStats::new(worker_id);
                    
                    while let Some((index, chunk)) = work_queue.pop() {
                        let start_time = Instant::now();
                        
                        match Self::process_single_chunk_optimized(chunk, &config) {
                            Ok(processed_chunk) => {
                                results.push(Ok((index, processed_chunk)));
                                worker_stats.successful_chunks += 1;
                            }
                            Err(e) => {
                                results.push(Err((index, e)));
                                worker_stats.failed_chunks += 1;
                            }
                        }
                        
                        worker_stats.total_processing_time += start_time.elapsed();
                    }
                    
                    worker_stats
                });
                
                handles.push(handle);
            }
            
            // Wait for all workers to complete and collect stats
            let mut total_worker_stats = Vec::new();
            for handle in handles {
                match handle.join() {
                    Ok(stats) => total_worker_stats.push(stats),
                    Err(_) => return Err(BinaryExportError::InternalError(
                        "Worker thread panicked".to_string()
                    )),
                }
            }
            
            Ok(total_worker_stats)
        }).map_err(|_| BinaryExportError::InternalError("Thread scope failed".to_string()))?;
        
        // Collect and sort results
        let mut processed_chunks = vec![None; chunk_count];
        let mut error_count = 0;
        
        while let Some(result) = results.pop() {
            match result {
                Ok((index, processed_chunk)) => {
                    processed_chunks[index] = Some(processed_chunk);
                }
                Err((index, error)) => {
                    error_count += 1;
                    eprintln!("Error processing chunk {}: {}", index, error);
                }
            }
        }
        
        if error_count > 0 {
            return Err(BinaryExportError::InternalError(
                format!("Failed to process {} chunks", error_count)
            ));
        }
        
        // Convert to final result
        let final_chunks: Result<Vec<_>, _> = processed_chunks
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| BinaryExportError::InternalError("Missing processed chunks".to_string()));
        
        final_chunks
    }

    /// Enhanced parallel processing with work stealing
    pub fn process_parallel_with_work_stealing(&self, data: &UnifiedData) -> ProcessResult<ProcessedData> {
        let start_time = Instant::now();
        self.memory_monitor.start_monitoring();
        
        // Validate input data
        let validation_results = if self.config.validate_data {
            self.validate_unified_data(data)?
        } else {
            ValidationResults::default()
        };
        
        // Create work-stealing scheduler
        let scheduler = WorkStealingScheduler::new(self.config.worker_threads);
        
        // Split data into fine-grained work items
        let work_items = self.create_work_items(data)?;
        
        // Process work items with work stealing
        let processed_items = scheduler.process_work_items(work_items)?;
        
        // Merge results
        let merged_data = self.merge_work_items(processed_items)?;
        
        // Create processing metadata
        let metadata = ProcessingMetadata {
            timestamp: std::time::SystemTime::now(),
            method: ProcessingMethod::Parallel,
            format: DataFormat::Bincode,
            compression: None,
            config_hash: self.calculate_config_hash(),
        };
        
        let duration = start_time.elapsed();
        let peak_memory = self.memory_monitor.get_peak_usage();
        
        // Record statistics
        let stats = ProcessStats {
            bytes_processed: merged_data.len() as u64,
            duration,
            throughput: merged_data.len() as f64 / duration.as_secs_f64(),
            peak_memory_usage: peak_memory,
            chunks_processed: work_items.len() as u64,
            validation_errors: validation_results.errors.len() as u32,
            efficiency: self.calculate_parallel_efficiency(&merged_data, duration, self.config.worker_threads),
        };
        
        self.record_stats(stats);
        
        Ok(ProcessedData {
            data: merged_data,
            metadata,
            validation_results,
        })
    }

    /// Process a single chunk (static method for thread safety)
    fn process_single_chunk(chunk: DataChunk, config: &ProcessingConfig) -> ProcessResult<ProcessedChunk> {
        // Apply chunk-specific processing
        let processed_data = chunk.data; // Placeholder - would apply actual processing
        
        Ok(ProcessedChunk {
            id: chunk.id,
            chunk_type: chunk.chunk_type,
            processed_data,
            processing_time: Duration::from_millis(1), // Placeholder
        })
    }

    /// Optimized single chunk processing with lock-free operations
    fn process_single_chunk_optimized(chunk: DataChunk, config: &ProcessingConfig) -> ProcessResult<ProcessedChunk> {
        let start_time = Instant::now();
        
        // Apply optimized processing based on chunk type
        let processed_data = match chunk.chunk_type {
            ChunkType::Allocations => {
                Self::process_allocation_chunk(&chunk.data, config)?
            }
            ChunkType::Analysis => {
                Self::process_analysis_chunk(&chunk.data, config)?
            }
            ChunkType::Metadata => {
                Self::process_metadata_chunk(&chunk.data, config)?
            }
        };
        
        let processing_time = start_time.elapsed();
        
        Ok(ProcessedChunk {
            id: chunk.id,
            chunk_type: chunk.chunk_type,
            processed_data,
            processing_time,
        })
    }

    /// Process allocation data chunk
    fn process_allocation_chunk(data: &[u8], config: &ProcessingConfig) -> ProcessResult<Vec<u8>> {
        // Deserialize allocation data
        let allocations: Vec<crate::export::binary::core::AllocationRecord> = 
            bincode::deserialize(data)
                .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;
        
        // Apply allocation-specific optimizations
        let optimized_allocations = allocations.into_iter()
            .filter(|alloc| alloc.size > 0) // Filter out zero-size allocations
            .collect::<Vec<_>>();
        
        // Re-serialize optimized data
        bincode::serialize(&optimized_allocations)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))
    }

    /// Process analysis data chunk
    fn process_analysis_chunk(data: &[u8], _config: &ProcessingConfig) -> ProcessResult<Vec<u8>> {
        // For now, just return the data as-is
        // In a real implementation, this would apply analysis-specific optimizations
        Ok(data.to_vec())
    }

    /// Process metadata chunk
    fn process_metadata_chunk(data: &[u8], _config: &ProcessingConfig) -> ProcessResult<Vec<u8>> {
        // Metadata chunks are typically small and don't need much processing
        Ok(data.to_vec())
    }

    /// Create fine-grained work items for work stealing
    fn create_work_items(&self, data: &UnifiedData) -> ProcessResult<Vec<WorkItem>> {
        let mut work_items = Vec::new();
        let mut item_id = 0;
        
        // Create work items for allocations
        let allocations_per_item = (data.allocations.allocations.len() / (self.config.worker_threads * 4)).max(1);
        for chunk in data.allocations.allocations.chunks(allocations_per_item) {
            let serialized = bincode::serialize(chunk)
                .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;
            
            work_items.push(WorkItem {
                id: item_id,
                item_type: WorkItemType::Allocations,
                data: serialized,
                priority: WorkPriority::High, // Allocations are high priority
            });
            item_id += 1;
        }
        
        // Create work items for analysis data
        if let Some(ref lifecycle) = data.analysis.lifecycle {
            let serialized = bincode::serialize(lifecycle)
                .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;
            
            work_items.push(WorkItem {
                id: item_id,
                item_type: WorkItemType::Analysis,
                data: serialized,
                priority: WorkPriority::Medium,
            });
            item_id += 1;
        }
        
        // Add more analysis data work items as needed...
        
        Ok(work_items)
    }

    /// Merge processed work items back together
    fn merge_work_items(&self, items: Vec<ProcessedWorkItem>) -> ProcessResult<Vec<u8>> {
        let mut merged_data = Vec::new();
        
        // Sort items by ID to maintain order
        let mut sorted_items = items;
        sorted_items.sort_by_key(|item| item.id);
        
        // Merge data from all items
        for item in sorted_items {
            merged_data.extend_from_slice(&item.processed_data);
        }
        
        Ok(merged_data)
    }

    /// Calculate parallel processing efficiency
    fn calculate_parallel_efficiency(&self, data: &[u8], duration: Duration, worker_count: usize) -> f64 {
        let sequential_estimate = Duration::from_secs_f64(data.len() as f64 / 100_000_000.0); // 100MB/s sequential
        let parallel_speedup = sequential_estimate.as_secs_f64() / duration.as_secs_f64();
        let theoretical_max_speedup = worker_count as f64;
        
        (parallel_speedup / theoretical_max_speedup).min(1.0)
    }

    /// Merge processed chunks back together
    fn merge_processed_chunks(&self, chunks: Vec<ProcessedChunk>) -> ProcessResult<Vec<u8>> {
        let mut merged_data = Vec::new();
        
        // Sort chunks by ID to maintain order
        let mut sorted_chunks = chunks;
        sorted_chunks.sort_by_key(|chunk| chunk.id);
        
        // Merge chunk data
        for chunk in sorted_chunks {
            merged_data.extend_from_slice(&chunk.processed_data);
        }
        
        Ok(merged_data)
    }

    /// Calculate processing efficiency metric
    fn calculate_efficiency(&self, data: &[u8], duration: Duration) -> f64 {
        let theoretical_max_throughput = 1_000_000_000.0; // 1GB/s theoretical max
        let actual_throughput = data.len() as f64 / duration.as_secs_f64();
        
        (actual_throughput / theoretical_max_throughput).min(1.0)
    }

    /// Calculate configuration hash for metadata
    fn calculate_config_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.config.max_memory_usage.hash(&mut hasher);
        self.config.chunk_size.hash(&mut hasher);
        self.config.worker_threads.hash(&mut hasher);
        hasher.finish()
    }

    /// Record processing statistics
    fn record_stats(&self, stats: ProcessStats) {
        if let Ok(mut stats_vec) = self.stats.lock() {
            stats_vec.push(stats);
        }
    }

    /// Get processing statistics
    pub fn get_stats(&self) -> Vec<ProcessStats> {
        self.stats.lock().unwrap_or_else(|_| {
            std::sync::PoisonError::into_inner
        }).clone()
    }

    /// Get current memory usage
    pub fn get_memory_usage(&self) -> usize {
        self.memory_monitor.get_current_usage()
    }

    /// Get peak memory usage
    pub fn get_peak_memory_usage(&self) -> usize {
        self.memory_monitor.get_peak_usage()
    }

    /// Process data in streaming mode with constant memory usage
    pub fn process_streaming<R: Read, W: Write>(
        &self,
        mut reader: R,
        mut writer: W,
    ) -> ProcessResult<ProcessStats> {
        let start_time = Instant::now();
        self.memory_monitor.start_monitoring();
        
        let mut total_bytes_processed = 0u64;
        let mut chunks_processed = 0u64;
        let mut validation_errors = 0u32;
        
        // Create streaming buffer
        let mut buffer = vec![0u8; self.config.chunk_size];
        let mut output_buffer = Vec::with_capacity(self.config.chunk_size * 2);
        
        // Process data in chunks to maintain constant memory usage
        loop {
            // Check memory usage and timeout
            self.monitor_memory_usage()?;
            if start_time.elapsed().as_secs() > self.config.timeout_secs {
                return Err(BinaryExportError::Timeout {
                    operation: "streaming_processing".to_string(),
                    timeout_secs: self.config.timeout_secs,
                });
            }
            
            // Read next chunk
            let bytes_read = reader.read(&mut buffer)
                .map_err(|e| BinaryExportError::IoError(e.kind()))?;
            
            if bytes_read == 0 {
                break; // End of stream
            }
            
            // Process chunk with streaming validation
            let chunk_data = &buffer[..bytes_read];
            let processed_chunk = self.process_streaming_chunk(chunk_data)?;
            
            // Validate chunk if enabled
            if self.config.validate_data {
                let chunk_validation = self.validate_streaming_chunk(chunk_data)?;
                if !chunk_validation.is_valid {
                    validation_errors += chunk_validation.errors.len() as u32;
                }
            }
            
            // Write processed chunk to output
            output_buffer.clear();
            output_buffer.extend_from_slice(&processed_chunk);
            
            writer.write_all(&output_buffer)
                .map_err(|e| BinaryExportError::IoError(e.kind()))?;
            
            total_bytes_processed += bytes_read as u64;
            chunks_processed += 1;
            
            // Update memory monitoring
            self.memory_monitor.update_usage(buffer.len() + output_buffer.len());
        }
        
        // Flush any remaining data
        writer.flush()
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;
        
        let duration = start_time.elapsed();
        let peak_memory = self.memory_monitor.get_peak_usage();
        
        // Create processing statistics
        let stats = ProcessStats {
            bytes_processed: total_bytes_processed,
            duration,
            throughput: total_bytes_processed as f64 / duration.as_secs_f64(),
            peak_memory_usage: peak_memory,
            chunks_processed,
            validation_errors,
            efficiency: self.calculate_streaming_efficiency(total_bytes_processed, duration, peak_memory),
        };
        
        self.record_stats(stats.clone());
        Ok(stats)
    }

    /// Process data with streaming and integrity checking
    pub fn process_streaming_with_integrity<R: Read, W: Write>(
        &self,
        reader: R,
        writer: W,
    ) -> ProcessResult<(ProcessStats, StreamingIntegrityReport)> {
        let mut integrity_checker = StreamingIntegrityChecker::new();
        let mut wrapped_reader = IntegrityCheckingReader::new(reader, &mut integrity_checker);
        let mut wrapped_writer = IntegrityCheckingWriter::new(writer, &mut integrity_checker);
        
        let stats = self.process_streaming(wrapped_reader, wrapped_writer)?;
        let integrity_report = integrity_checker.finalize();
        
        Ok((stats, integrity_report))
    }

    /// Process a single streaming chunk
    fn process_streaming_chunk(&self, chunk_data: &[u8]) -> ProcessResult<Vec<u8>> {
        // Apply streaming transformations
        let mut processed = Vec::with_capacity(chunk_data.len());
        
        // For now, just copy the data (placeholder for actual processing)
        // In a real implementation, this would apply:
        // - Data deduplication within the chunk
        // - Compression if enabled
        // - Format conversion
        // - Encryption if needed
        processed.extend_from_slice(chunk_data);
        
        Ok(processed)
    }

    /// Validate a streaming chunk
    fn validate_streaming_chunk(&self, chunk_data: &[u8]) -> ProcessResult<ValidationResults> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut integrity_score = 1.0;
        
        // Basic chunk validation
        if chunk_data.is_empty() {
            warnings.push("Empty chunk encountered".to_string());
            integrity_score -= 0.1;
        }
        
        // Check for data corruption patterns
        if chunk_data.len() < 4 {
            warnings.push("Very small chunk size".to_string());
            integrity_score -= 0.05;
        }
        
        // Validate chunk structure (placeholder)
        // In a real implementation, this would check:
        // - Data format consistency
        // - Expected data patterns
        // - Checksum validation
        
        integrity_score = integrity_score.max(0.0);
        
        Ok(ValidationResults {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            integrity_score,
        })
    }

    /// Calculate streaming processing efficiency
    fn calculate_streaming_efficiency(&self, bytes_processed: u64, duration: Duration, peak_memory: usize) -> f64 {
        let throughput_efficiency = {
            let theoretical_max = 500_000_000.0; // 500MB/s for streaming
            let actual_throughput = bytes_processed as f64 / duration.as_secs_f64();
            (actual_throughput / theoretical_max).min(1.0)
        };
        
        let memory_efficiency = {
            let target_memory = self.config.chunk_size * 3; // Target: 3x chunk size
            if peak_memory <= target_memory {
                1.0
            } else {
                target_memory as f64 / peak_memory as f64
            }
        };
        
        // Weighted average: 70% throughput, 30% memory efficiency
        throughput_efficiency * 0.7 + memory_efficiency * 0.3
    }

    /// Process data with backpressure control
    pub fn process_streaming_with_backpressure<R: Read, W: Write>(
        &self,
        reader: R,
        writer: W,
        backpressure_config: BackpressureConfig,
    ) -> ProcessResult<ProcessStats> {
        let mut backpressure_controller = BackpressureController::new(backpressure_config);
        let controlled_reader = BackpressureReader::new(reader, &mut backpressure_controller);
        let controlled_writer = BackpressureWriter::new(writer, &mut backpressure_controller);
        
        self.process_streaming(controlled_reader, controlled_writer)
    }
}

/// Data chunk for parallel processing
#[derive(Debug, Clone)]
struct DataChunk {
    /// Unique chunk identifier
    id: u64,
    /// Type of data in this chunk
    chunk_type: ChunkType,
    /// Serialized chunk data
    data: Vec<u8>,
}

/// Processed data chunk
#[derive(Debug, Clone)]
struct ProcessedChunk {
    /// Unique chunk identifier
    id: u64,
    /// Type of data in this chunk
    chunk_type: ChunkType,
    /// Processed chunk data
    processed_data: Vec<u8>,
    /// Time taken to process this chunk
    processing_time: Duration,
}

/// Types of data chunks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChunkType {
    /// Allocation data chunk
    Allocations,
    /// Analysis data chunk
    Analysis,
    /// Metadata chunk
    Metadata,
}

/// Streaming integrity report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingIntegrityReport {
    /// Total bytes processed
    pub total_bytes: u64,
    /// Number of chunks processed
    pub chunks_processed: u64,
    /// Checksum of all processed data
    pub overall_checksum: String,
    /// Per-chunk checksums
    pub chunk_checksums: Vec<String>,
    /// Integrity violations found
    pub violations: Vec<IntegrityViolation>,
    /// Overall integrity score
    pub integrity_score: f64,
}

/// Integrity violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityViolation {
    /// Chunk index where violation occurred
    pub chunk_index: u64,
    /// Type of violation
    pub violation_type: String,
    /// Severity level
    pub severity: ViolationSeverity,
    /// Description of the violation
    pub description: String,
}

/// Severity levels for integrity violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Low severity - warning only
    Low,
    /// Medium severity - may affect data quality
    Medium,
    /// High severity - significant data corruption
    High,
    /// Critical severity - data unusable
    Critical,
}

/// Configuration for backpressure control
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    /// Maximum buffer size before applying backpressure
    pub max_buffer_size: usize,
    /// Target processing rate (bytes/second)
    pub target_rate: u64,
    /// Backpressure threshold (0.0 to 1.0)
    pub pressure_threshold: f64,
    /// Recovery time after backpressure (milliseconds)
    pub recovery_time_ms: u64,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 1024 * 1024, // 1MB
            target_rate: 100 * 1024 * 1024, // 100MB/s
            pressure_threshold: 0.8,
            recovery_time_ms: 100,
        }
    }
}

/// Streaming integrity checker
struct StreamingIntegrityChecker {
    /// Running checksum
    hasher: sha2::Sha256,
    /// Chunk checksums
    chunk_checksums: Vec<String>,
    /// Violations found
    violations: Vec<IntegrityViolation>,
    /// Current chunk index
    current_chunk: u64,
    /// Total bytes processed
    total_bytes: u64,
}

impl StreamingIntegrityChecker {
    fn new() -> Self {
        use sha2::Digest;
        Self {
            hasher: sha2::Sha256::new(),
            chunk_checksums: Vec::new(),
            violations: Vec::new(),
            current_chunk: 0,
            total_bytes: 0,
        }
    }
    
    fn process_chunk(&mut self, chunk_data: &[u8]) {
        use sha2::Digest;
        
        // Update overall hash
        self.hasher.update(chunk_data);
        
        // Calculate chunk hash
        let mut chunk_hasher = sha2::Sha256::new();
        chunk_hasher.update(chunk_data);
        let chunk_hash = format!("{:x}", chunk_hasher.finalize());
        self.chunk_checksums.push(chunk_hash);
        
        // Basic integrity checks
        if chunk_data.is_empty() {
            self.violations.push(IntegrityViolation {
                chunk_index: self.current_chunk,
                violation_type: "empty_chunk".to_string(),
                severity: ViolationSeverity::Medium,
                description: "Empty chunk encountered in stream".to_string(),
            });
        }
        
        self.current_chunk += 1;
        self.total_bytes += chunk_data.len() as u64;
    }
    
    fn finalize(self) -> StreamingIntegrityReport {
        use sha2::Digest;
        
        let overall_checksum = format!("{:x}", self.hasher.finalize());
        let integrity_score = if self.violations.is_empty() {
            1.0
        } else {
            let critical_count = self.violations.iter()
                .filter(|v| v.severity == ViolationSeverity::Critical)
                .count();
            let high_count = self.violations.iter()
                .filter(|v| v.severity == ViolationSeverity::High)
                .count();
            
            1.0 - (critical_count as f64 * 0.5 + high_count as f64 * 0.2) / self.current_chunk as f64
        };
        
        StreamingIntegrityReport {
            total_bytes: self.total_bytes,
            chunks_processed: self.current_chunk,
            overall_checksum,
            chunk_checksums: self.chunk_checksums,
            violations: self.violations,
            integrity_score: integrity_score.max(0.0),
        }
    }
}

/// Backpressure controller for streaming operations
struct BackpressureController {
    config: BackpressureConfig,
    current_buffer_size: usize,
    last_pressure_time: Option<Instant>,
    processing_rate: f64,
    rate_samples: Vec<(Instant, u64)>,
}

impl BackpressureController {
    fn new(config: BackpressureConfig) -> Self {
        Self {
            config,
            current_buffer_size: 0,
            last_pressure_time: None,
            processing_rate: 0.0,
            rate_samples: Vec::new(),
        }
    }
    
    fn should_apply_backpressure(&mut self, bytes_processed: u64) -> bool {
        // Update processing rate
        let now = Instant::now();
        self.rate_samples.push((now, bytes_processed));
        
        // Keep only recent samples (last 5 seconds)
        let cutoff = now - Duration::from_secs(5);
        self.rate_samples.retain(|(time, _)| *time > cutoff);
        
        // Calculate current rate
        if self.rate_samples.len() >= 2 {
            let first = &self.rate_samples[0];
            let last = &self.rate_samples[self.rate_samples.len() - 1];
            let duration = last.0.duration_since(first.0).as_secs_f64();
            let bytes_diff = last.1 - first.1;
            self.processing_rate = bytes_diff as f64 / duration;
        }
        
        // Check if backpressure should be applied
        let buffer_pressure = self.current_buffer_size as f64 / self.config.max_buffer_size as f64;
        let rate_pressure = self.processing_rate / self.config.target_rate as f64;
        
        let should_apply = buffer_pressure > self.config.pressure_threshold || 
                          rate_pressure > self.config.pressure_threshold;
        
        if should_apply {
            self.last_pressure_time = Some(now);
        }
        
        should_apply
    }
    
    fn get_backpressure_delay(&self) -> Duration {
        if let Some(last_pressure) = self.last_pressure_time {
            let elapsed = last_pressure.elapsed();
            if elapsed < Duration::from_millis(self.config.recovery_time_ms) {
                return Duration::from_millis(self.config.recovery_time_ms) - elapsed;
            }
        }
        Duration::from_millis(0)
    }
    
    fn update_buffer_size(&mut self, size: usize) {
        self.current_buffer_size = size;
    }
}

/// Reader wrapper with integrity checking
struct IntegrityCheckingReader<R: Read> {
    inner: R,
    checker: *mut StreamingIntegrityChecker,
}

impl<R: Read> IntegrityCheckingReader<R> {
    fn new(reader: R, checker: &mut StreamingIntegrityChecker) -> Self {
        Self {
            inner: reader,
            checker: checker as *mut StreamingIntegrityChecker,
        }
    }
}

impl<R: Read> Read for IntegrityCheckingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.inner.read(buf)?;
        if bytes_read > 0 {
            unsafe {
                (*self.checker).process_chunk(&buf[..bytes_read]);
            }
        }
        Ok(bytes_read)
    }
}

/// Writer wrapper with integrity checking
struct IntegrityCheckingWriter<W: Write> {
    inner: W,
    checker: *mut StreamingIntegrityChecker,
}

impl<W: Write> IntegrityCheckingWriter<W> {
    fn new(writer: W, checker: &mut StreamingIntegrityChecker) -> Self {
        Self {
            inner: writer,
            checker: checker as *mut StreamingIntegrityChecker,
        }
    }
}

impl<W: Write> Write for IntegrityCheckingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let bytes_written = self.inner.write(buf)?;
        if bytes_written > 0 {
            unsafe {
                (*self.checker).process_chunk(&buf[..bytes_written]);
            }
        }
        Ok(bytes_written)
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Reader wrapper with backpressure control
struct BackpressureReader<R: Read> {
    inner: R,
    controller: *mut BackpressureController,
    bytes_read_total: u64,
}

impl<R: Read> BackpressureReader<R> {
    fn new(reader: R, controller: &mut BackpressureController) -> Self {
        Self {
            inner: reader,
            controller: controller as *mut BackpressureController,
            bytes_read_total: 0,
        }
    }
}

impl<R: Read> Read for BackpressureReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // Check if backpressure should be applied
        unsafe {
            if (*self.controller).should_apply_backpressure(self.bytes_read_total) {
                let delay = (*self.controller).get_backpressure_delay();
                if !delay.is_zero() {
                    std::thread::sleep(delay);
                }
            }
        }
        
        let bytes_read = self.inner.read(buf)?;
        self.bytes_read_total += bytes_read as u64;
        
        unsafe {
            (*self.controller).update_buffer_size(buf.len());
        }
        
        Ok(bytes_read)
    }
}

/// Writer wrapper with backpressure control
struct BackpressureWriter<W: Write> {
    inner: W,
    controller: *mut BackpressureController,
    bytes_written_total: u64,
}

impl<W: Write> BackpressureWriter<W> {
    fn new(writer: W, controller: &mut BackpressureController) -> Self {
        Self {
            inner: writer,
            controller: controller as *mut BackpressureController,
            bytes_written_total: 0,
        }
    }
}

impl<W: Write> Write for BackpressureWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Check if backpressure should be applied
        unsafe {
            if (*self.controller).should_apply_backpressure(self.bytes_written_total) {
                let delay = (*self.controller).get_backpressure_delay();
                if !delay.is_zero() {
                    std::thread::sleep(delay);
                }
            }
        }
        
        let bytes_written = self.inner.write(buf)?;
        self.bytes_written_total += bytes_written as u64;
        
        unsafe {
            (*self.controller).update_buffer_size(buf.len());
        }
        
        Ok(bytes_written)
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Work item for parallel processing
#[derive(Debug, Clone)]
struct WorkItem {
    /// Unique work item identifier
    id: u64,
    /// Type of work item
    item_type: WorkItemType,
    /// Work item data
    data: Vec<u8>,
    /// Processing priority
    priority: WorkPriority,
}

/// Types of work items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkItemType {
    /// Allocation data processing
    Allocations,
    /// Analysis data processing
    Analysis,
    /// Metadata processing
    Metadata,
}

/// Work item processing priorities
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WorkPriority {
    /// Low priority work
    Low = 1,
    /// Medium priority work
    Medium = 2,
    /// High priority work
    High = 3,
}

/// Processed work item result
#[derive(Debug, Clone)]
struct ProcessedWorkItem {
    /// Original work item ID
    id: u64,
    /// Type of work item
    item_type: WorkItemType,
    /// Processed data
    processed_data: Vec<u8>,
    /// Processing time
    processing_time: Duration,
}

/// Worker thread statistics
#[derive(Debug, Clone)]
struct WorkerStats {
    /// Worker ID
    worker_id: usize,
    /// Number of successfully processed chunks
    successful_chunks: u64,
    /// Number of failed chunks
    failed_chunks: u64,
    /// Total processing time
    total_processing_time: Duration,
    /// Work items stolen from other workers
    stolen_work_items: u64,
    /// Work items stolen by other workers
    work_items_stolen: u64,
}

impl WorkerStats {
    fn new(worker_id: usize) -> Self {
        Self {
            worker_id,
            successful_chunks: 0,
            failed_chunks: 0,
            total_processing_time: Duration::from_millis(0),
            stolen_work_items: 0,
            work_items_stolen: 0,
        }
    }
}

/// Work-stealing scheduler for parallel processing
struct WorkStealingScheduler {
    /// Number of worker threads
    worker_count: usize,
    /// Work queues for each worker (lock-free)
    work_queues: Vec<Arc<crossbeam_queue::SegQueue<WorkItem>>>,
    /// Global work queue for overflow
    global_queue: Arc<crossbeam_queue::SegQueue<WorkItem>>,
}

impl WorkStealingScheduler {
    fn new(worker_count: usize) -> Self {
        let mut work_queues = Vec::new();
        for _ in 0..worker_count {
            work_queues.push(Arc::new(crossbeam_queue::SegQueue::new()));
        }
        
        Self {
            worker_count,
            work_queues,
            global_queue: Arc::new(crossbeam_queue::SegQueue::new()),
        }
    }
    
    fn process_work_items(&self, work_items: Vec<WorkItem>) -> ProcessResult<Vec<ProcessedWorkItem>> {
        use crossbeam_utils::thread;
        
        // Distribute work items to worker queues
        self.distribute_work_items(work_items);
        
        let results = Arc::new(crossbeam_queue::SegQueue::new());
        
        // Spawn worker threads with work stealing
        thread::scope(|s| {
            let mut handles = Vec::new();
            
            for worker_id in 0..self.worker_count {
                let work_queues = self.work_queues.clone();
                let global_queue = Arc::clone(&self.global_queue);
                let results = Arc::clone(&results);
                
                let handle = s.spawn(move |_| {
                    let mut stats = WorkerStats::new(worker_id);
                    
                    loop {
                        // Try to get work from local queue first
                        let work_item = if let Some(item) = work_queues[worker_id].pop() {
                            Some(item)
                        } else if let Some(item) = global_queue.pop() {
                            // Try global queue
                            Some(item)
                        } else {
                            // Try to steal work from other workers
                            self.steal_work(&work_queues, worker_id, &mut stats)
                        };
                        
                        match work_item {
                            Some(item) => {
                                let start_time = Instant::now();
                                
                                match self.process_work_item(item) {
                                    Ok(processed_item) => {
                                        results.push(processed_item);
                                        stats.successful_chunks += 1;
                                    }
                                    Err(_) => {
                                        stats.failed_chunks += 1;
                                    }
                                }
                                
                                stats.total_processing_time += start_time.elapsed();
                            }
                            None => {
                                // No work available, exit
                                break;
                            }
                        }
                    }
                    
                    stats
                });
                
                handles.push(handle);
            }
            
            // Wait for all workers to complete
            for handle in handles {
                handle.join().unwrap();
            }
            
            Ok(())
        }).map_err(|_| BinaryExportError::InternalError("Work stealing failed".to_string()))?;
        
        // Collect results
        let mut processed_items = Vec::new();
        while let Some(item) = results.pop() {
            processed_items.push(item);
        }
        
        Ok(processed_items)
    }
    
    fn distribute_work_items(&self, work_items: Vec<WorkItem>) {
        // Sort work items by priority (high priority first)
        let mut sorted_items = work_items;
        sorted_items.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Distribute items round-robin to worker queues
        for (index, item) in sorted_items.into_iter().enumerate() {
            let worker_id = index % self.worker_count;
            self.work_queues[worker_id].push(item);
        }
    }
    
    fn steal_work(
        &self,
        work_queues: &[Arc<crossbeam_queue::SegQueue<WorkItem>>],
        worker_id: usize,
        stats: &mut WorkerStats,
    ) -> Option<WorkItem> {
        // Try to steal from other workers (random order to avoid contention)
        use rand::seq::SliceRandom;
        let mut other_workers: Vec<usize> = (0..self.worker_count)
            .filter(|&id| id != worker_id)
            .collect();
        
        other_workers.shuffle(&mut rand::thread_rng());
        
        for &other_worker_id in &other_workers {
            if let Some(stolen_item) = work_queues[other_worker_id].pop() {
                stats.stolen_work_items += 1;
                return Some(stolen_item);
            }
        }
        
        None
    }
    
    fn process_work_item(&self, item: WorkItem) -> ProcessResult<ProcessedWorkItem> {
        let start_time = Instant::now();
        
        // Process based on work item type
        let processed_data = match item.item_type {
            WorkItemType::Allocations => {
                // Process allocation data
                item.data // Placeholder - would apply actual processing
            }
            WorkItemType::Analysis => {
                // Process analysis data
                item.data // Placeholder - would apply actual processing
            }
            WorkItemType::Metadata => {
                // Process metadata
                item.data // Placeholder - would apply actual processing
            }
        };
        
        let processing_time = start_time.elapsed();
        
        Ok(ProcessedWorkItem {
            id: item.id,
            item_type: item.item_type,
            processed_data,
            processing_time,
        })
    }
}

impl MemoryMonitor {
    /// Create a new memory monitor
    fn new() -> Self {
        Self {
            current_usage: Arc::new(Mutex::new(0)),
            peak_usage: Arc::new(Mutex::new(0)),
            usage_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start memory monitoring
    fn start_monitoring(&self) {
        // Reset monitoring state
        if let Ok(mut current) = self.current_usage.lock() {
            *current = 0;
        }
        if let Ok(mut peak) = self.peak_usage.lock() {
            *peak = 0;
        }
        if let Ok(mut history) = self.usage_history.lock() {
            history.clear();
        }
    }

    /// Update current memory usage
    fn update_usage(&self, usage: usize) {
        if let Ok(mut current) = self.current_usage.lock() {
            *current = usage;
        }
        
        if let Ok(mut peak) = self.peak_usage.lock() {
            if usage > *peak {
                *peak = usage;
            }
        }
        
        if let Ok(mut history) = self.usage_history.lock() {
            history.push((Instant::now(), usage));
            
            // Keep only recent history (last 1000 entries)
            if history.len() > 1000 {
                history.drain(0..500);
            }
        }
    }

    /// Get current memory usage
    fn get_current_usage(&self) -> usize {
        self.current_usage.lock().unwrap_or_else(|_| {
            std::sync::PoisonError::into_inner
        })
    }

    /// Get peak memory usage
    fn get_peak_usage(&self) -> usize {
        self.peak_usage.lock().unwrap_or_else(|_| {
            std::sync::PoisonError::into_inner
        })
    }
}

impl Default for ValidationResults {
    fn default() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            integrity_score: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::binary::core::UnifiedData;

    #[test]
    fn test_processing_config_presets() {
        let fast = ProcessingConfig::fast();
        assert_eq!(fast.chunk_size, 64 * 1024);
        assert!(!fast.validate_data);
        
        let memory_efficient = ProcessingConfig::memory_efficient();
        assert_eq!(memory_efficient.max_memory_usage, 64 * 1024 * 1024);
        assert!(memory_efficient.validate_data);
    }

    #[test]
    fn test_data_processor_creation() {
        let config = ProcessingConfig::default();
        let processor = DataProcessor::new(config);
        assert_eq!(processor.get_memory_usage(), 0);
    }

    #[test]
    fn test_batch_processing() {
        let config = ProcessingConfig::default();
        let processor = DataProcessor::new(config);
        let test_data = UnifiedData::new();
        
        let result = processor.process_batch(&test_data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert!(processed.validation_results.is_valid);
        assert_eq!(processed.metadata.method, ProcessingMethod::Batch);
    }

    #[test]
    fn test_memory_monitoring() {
        let monitor = MemoryMonitor::new();
        monitor.start_monitoring();
        
        monitor.update_usage(1000);
        assert_eq!(monitor.get_current_usage(), 1000);
        assert_eq!(monitor.get_peak_usage(), 1000);
        
        monitor.update_usage(500);
        assert_eq!(monitor.get_current_usage(), 500);
        assert_eq!(monitor.get_peak_usage(), 1000); // Peak should remain
    }

    #[test]
    fn test_streaming_processing() {
        let config = ProcessingConfig::default();
        let processor = DataProcessor::new(config);
        
        // Create test data
        let test_data = b"Hello, world! This is test data for streaming processing.";
        let mut reader = std::io::Cursor::new(test_data);
        let mut writer = Vec::new();
        
        let result = processor.process_streaming(&mut reader, &mut writer);
        assert!(result.is_ok());
        
        let stats = result.unwrap();
        assert_eq!(stats.bytes_processed, test_data.len() as u64);
        assert!(stats.chunks_processed > 0);
        assert!(stats.throughput > 0.0);
    }

    #[test]
    fn test_streaming_integrity_checker() {
        let mut checker = StreamingIntegrityChecker::new();
        
        let chunk1 = b"Hello, ";
        let chunk2 = b"world!";
        
        checker.process_chunk(chunk1);
        checker.process_chunk(chunk2);
        
        let report = checker.finalize();
        assert_eq!(report.chunks_processed, 2);
        assert_eq!(report.total_bytes, 13);
        assert_eq!(report.chunk_checksums.len(), 2);
        assert!(report.integrity_score > 0.9);
    }

    #[test]
    fn test_backpressure_controller() {
        let config = BackpressureConfig::default();
        let mut controller = BackpressureController::new(config);
        
        // Initially should not apply backpressure
        assert!(!controller.should_apply_backpressure(1000));
        
        // Update buffer size to trigger backpressure
        controller.update_buffer_size(2 * 1024 * 1024); // 2MB > 1MB limit
        assert!(controller.should_apply_backpressure(1000));
    }

    #[test]
    fn test_streaming_with_integrity() {
        let config = ProcessingConfig::default();
        let processor = DataProcessor::new(config);
        
        let test_data = b"Test data for integrity checking during streaming.";
        let mut reader = std::io::Cursor::new(test_data);
        let mut writer = Vec::new();
        
        let result = processor.process_streaming_with_integrity(&mut reader, &mut writer);
        assert!(result.is_ok());
        
        let (stats, integrity_report) = result.unwrap();
        assert_eq!(stats.bytes_processed, test_data.len() as u64);
        assert!(integrity_report.integrity_score > 0.9);
        assert!(!integrity_report.overall_checksum.is_empty());
    }

    #[test]
    fn test_parallel_processing() {
        let config = ProcessingConfig::default();
        let processor = DataProcessor::new(config);
        let test_data = UnifiedData::new();
        
        let result = processor.process_parallel(&test_data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert!(processed.validation_results.is_valid);
        assert_eq!(processed.metadata.method, ProcessingMethod::Parallel);
    }

    #[test]
    fn test_work_stealing_scheduler() {
        let scheduler = WorkStealingScheduler::new(4);
        
        // Create test work items
        let work_items = vec![
            WorkItem {
                id: 0,
                item_type: WorkItemType::Allocations,
                data: vec![1, 2, 3, 4],
                priority: WorkPriority::High,
            },
            WorkItem {
                id: 1,
                item_type: WorkItemType::Analysis,
                data: vec![5, 6, 7, 8],
                priority: WorkPriority::Medium,
            },
        ];
        
        let result = scheduler.process_work_items(work_items);
        assert!(result.is_ok());
        
        let processed_items = result.unwrap();
        assert_eq!(processed_items.len(), 2);
    }

    #[test]
    fn test_work_item_priority_ordering() {
        let mut items = vec![
            WorkItem {
                id: 0,
                item_type: WorkItemType::Allocations,
                data: vec![],
                priority: WorkPriority::Low,
            },
            WorkItem {
                id: 1,
                item_type: WorkItemType::Analysis,
                data: vec![],
                priority: WorkPriority::High,
            },
            WorkItem {
                id: 2,
                item_type: WorkItemType::Metadata,
                data: vec![],
                priority: WorkPriority::Medium,
            },
        ];
        
        items.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        assert_eq!(items[0].priority, WorkPriority::High);
        assert_eq!(items[1].priority, WorkPriority::Medium);
        assert_eq!(items[2].priority, WorkPriority::Low);
    }
}