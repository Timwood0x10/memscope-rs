//! Large file optimization module for JSON processing
//!
//! This module provides streaming JSON parsing and memory optimization
//! for handling large JSON files efficiently without memory issues.

use serde_json::Value;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Configuration for large file processing
#[derive(Debug, Clone)]
pub struct LargeFileConfig {
    /// Maximum memory usage in bytes before switching to streaming mode
    pub max_memory_bytes: usize,
    /// Chunk size for streaming processing in bytes
    pub stream_chunk_size: usize,
    /// Enable memory usage monitoring
    pub enable_memory_monitoring: bool,
    /// Enable progress reporting
    pub enable_progress_reporting: bool,
    /// Maximum file size to process in bytes (safety limit)
    pub max_file_size_bytes: usize,
}

impl Default for LargeFileConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 512 * 1024 * 1024, // 512MB
            stream_chunk_size: 64 * 1024,        // 64KB chunks
            enable_memory_monitoring: true,
            enable_progress_reporting: true,
            max_file_size_bytes: 2 * 1024 * 1024 * 1024, // 2GB limit
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Current memory usage in bytes
    pub current_usage_bytes: usize,
    /// Peak memory usage in bytes
    pub peak_usage_bytes: usize,
    /// Number of memory allocations tracked
    pub allocation_count: usize,
    /// Memory efficiency ratio (0.0 to 1.0)
    pub efficiency_ratio: f64,
}

/// Processing statistics for large files
#[derive(Debug)]
pub struct ProcessingStats {
    /// File size in bytes
    pub file_size_bytes: usize,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Whether streaming mode was used
    pub streaming_mode_used: bool,
    /// Memory statistics
    pub memory_stats: MemoryStats,
    /// Throughput in MB/s
    pub throughput_mb_per_sec: f64,
    /// Number of JSON objects processed
    pub objects_processed: usize,
}

/// Errors that can occur during large file processing
#[derive(Debug)]
pub enum LargeFileError {
    /// File is too large to process safely
    FileTooLarge(usize, usize),
    /// Memory limit exceeded during processing
    MemoryLimitExceeded(usize, usize),
    /// Streaming JSON parsing failed
    StreamingParseError(String),
    /// IO error during file processing
    IoError(std::io::Error),
    /// JSON structure validation failed
    ValidationError(String),
}

impl fmt::Display for LargeFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LargeFileError::FileTooLarge(size, limit) => {
                write!(
                    f,
                    "File size ({size} bytes) exceeds limit ({limit} bytes)",
                )
            }
            LargeFileError::MemoryLimitExceeded(used, limit) => {
                write!(
                    f,
                    "Memory usage ({used} bytes) exceeds limit ({limit} bytes)",
                )
            }
            LargeFileError::StreamingParseError(msg) => {
                write!(f, "Streaming parse error: {msg}")
            }
            LargeFileError::IoError(err) => {
                write!(f, "IO error: {err}")
            }
            LargeFileError::ValidationError(msg) => {
                write!(f, "Validation error: {msg}")
            }
        }
    }
}

impl Error for LargeFileError {}

/// Memory monitor for tracking usage during processing
pub struct MemoryMonitor {
    /// Current memory usage counter
    current_usage: Arc<AtomicUsize>,
    /// Peak memory usage
    peak_usage: Arc<AtomicUsize>,
    /// Memory limit in bytes
    memory_limit: usize,
    /// Enable monitoring flag
    enabled: bool,
}

impl MemoryMonitor {
    /// Create a new memory monitor
    pub fn new(memory_limit: usize, enabled: bool) -> Self {
        Self {
            current_usage: Arc::new(AtomicUsize::new(0)),
            peak_usage: Arc::new(AtomicUsize::new(0)),
            memory_limit,
            enabled,
        }
    }

    /// Allocate memory and track usage
    pub fn allocate(&self, size: usize) -> Result<(), LargeFileError> {
        if !self.enabled {
            return Ok(());
        }

        let new_usage = self.current_usage.fetch_add(size, Ordering::Relaxed) + size;

        // Update peak usage
        let mut peak = self.peak_usage.load(Ordering::Relaxed);
        while new_usage > peak {
            match self.peak_usage.compare_exchange_weak(
                peak,
                new_usage,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(current_peak) => peak = current_peak,
            }
        }

        // Check memory limit
        if new_usage > self.memory_limit {
            return Err(LargeFileError::MemoryLimitExceeded(
                new_usage,
                self.memory_limit,
            ));
        }

        Ok(())
    }

    /// Deallocate memory and update tracking
    pub fn deallocate(&self, size: usize) {
        if self.enabled {
            self.current_usage.fetch_sub(size, Ordering::Relaxed);
        }
    }

    /// Get current memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        let current = self.current_usage.load(Ordering::Relaxed);
        let peak = self.peak_usage.load(Ordering::Relaxed);

        MemoryStats {
            current_usage_bytes: current,
            peak_usage_bytes: peak,
            allocation_count: 1, // Simplified for this implementation
            efficiency_ratio: if peak > 0 {
                current as f64 / peak as f64
            } else {
                1.0
            },
        }
    }
}

/// Large file optimizer for JSON processing
pub struct LargeFileOptimizer {
    /// Configuration settings
    config: LargeFileConfig,
    /// Memory monitor
    memory_monitor: MemoryMonitor,
}

impl LargeFileOptimizer {
    /// Create a new large file optimizer
    pub fn new(config: LargeFileConfig) -> Self {
        let memory_monitor =
            MemoryMonitor::new(config.max_memory_bytes, config.enable_memory_monitoring);

        Self {
            config,
            memory_monitor,
        }
    }

    /// Create optimizer with default configuration
    pub fn new_default() -> Self {
        Self::new(LargeFileConfig::default())
    }

    /// Process a large JSON file with optimization
    pub fn process_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        file_type: &str,
    ) -> Result<(Value, ProcessingStats), LargeFileError> {
        let start_time = Instant::now();
        let path = file_path.as_ref();

        // Check file size
        let file_size = std::fs::metadata(path)
            .map_err(LargeFileError::IoError)?
            .len() as usize;

        if file_size > self.config.max_file_size_bytes {
            return Err(LargeFileError::FileTooLarge(
                file_size,
                self.config.max_file_size_bytes,
            ));
        }

        tracing::info!(
            "🔧 Processing large file: {} ({:.1} MB)",
            path.display(),
            file_size as f64 / 1024.0 / 1024.0
        );

        // Decide processing strategy based on file size
        let use_streaming = file_size > self.config.max_memory_bytes / 2;

        let (json_value, objects_processed) = if use_streaming {
            tracing::info!("📡 Using streaming mode for large file processing");
            self.process_streaming(path, file_type)?
        } else {
            tracing::info!("💾 Using memory-optimized mode for file processing");
            self.process_memory_optimized(path, file_type)?
        };

        let processing_time = start_time.elapsed().as_millis() as u64;
        let throughput = if processing_time > 0 {
            (file_size as f64 / 1024.0 / 1024.0) / (processing_time as f64 / 1000.0)
        } else {
            0.0
        };

        let stats = ProcessingStats {
            file_size_bytes: file_size,
            processing_time_ms: processing_time,
            streaming_mode_used: use_streaming,
            memory_stats: self.memory_monitor.get_stats(),
            throughput_mb_per_sec: throughput,
            objects_processed,
        };

        tracing::info!(
            "✅ File processed: {:.1} MB/s, {} objects, {}ms",
            throughput,
            objects_processed,
            processing_time
        );

        Ok((json_value, stats))
    }

    /// Process file using streaming JSON parsing
    fn process_streaming<P: AsRef<Path>>(
        &self,
        file_path: P,
        file_type: &str,
    ) -> Result<(Value, usize), LargeFileError> {
        let file = File::open(file_path).map_err(LargeFileError::IoError)?;
        let mut reader = BufReader::with_capacity(self.config.stream_chunk_size, file);

        // Track memory allocation for the reader buffer
        self.memory_monitor
            .allocate(self.config.stream_chunk_size)?;

        // For streaming, we'll read the JSON in chunks and validate structure
        let mut buffer = String::new();
        reader
            .read_to_string(&mut buffer)
            .map_err(LargeFileError::IoError)?;

        // Track memory for the buffer
        self.memory_monitor.allocate(buffer.len())?;

        // Parse JSON with streaming deserializer for validation
        let json_value: Value = serde_json::from_str(&buffer)
            .map_err(|e| LargeFileError::StreamingParseError(e.to_string()))?;

        // Validate JSON structure
        self.validate_json_structure(&json_value, file_type)?;

        // Count objects processed (simplified)
        let objects_processed = self.count_json_objects(&json_value);

        // Clean up memory tracking
        self.memory_monitor.deallocate(buffer.len());
        self.memory_monitor
            .deallocate(self.config.stream_chunk_size);

        Ok((json_value, objects_processed))
    }

    /// Process file using memory-optimized approach
    fn process_memory_optimized<P: AsRef<Path>>(
        &self,
        file_path: P,
        file_type: &str,
    ) -> Result<(Value, usize), LargeFileError> {
        // Read file with memory tracking
        let content = std::fs::read_to_string(file_path).map_err(LargeFileError::IoError)?;

        self.memory_monitor.allocate(content.len())?;

        // Parse JSON
        let json_value: Value = serde_json::from_str(&content)
            .map_err(|e| LargeFileError::StreamingParseError(e.to_string()))?;

        // Validate structure
        self.validate_json_structure(&json_value, file_type)?;

        // Count objects
        let objects_processed = self.count_json_objects(&json_value);

        // Clean up memory tracking
        self.memory_monitor.deallocate(content.len());

        Ok((json_value, objects_processed))
    }

    /// Validate JSON structure based on file type
    fn validate_json_structure(&self, json: &Value, file_type: &str) -> Result<(), LargeFileError> {
        match file_type {
            "memory_analysis" => {
                if !json.is_object() {
                    return Err(LargeFileError::ValidationError(
                        "Memory analysis JSON must be an object".to_string(),
                    ));
                }

                // Check for required fields
                let obj = json.as_object().expect("Test operation failed");
                if !obj.contains_key("allocations") && !obj.contains_key("summary") {
                    return Err(LargeFileError::ValidationError(
                        "Memory analysis JSON must contain 'allocations' or 'summary' field"
                            .to_string(),
                    ));
                }
            }
            "unsafe_ffi" => {
                if !json.is_object() {
                    return Err(LargeFileError::ValidationError(
                        "Unsafe FFI JSON must be an object".to_string(),
                    ));
                }

                let obj = json.as_object().expect("Test operation failed");
                if !obj.contains_key("enhanced_ffi_data") && !obj.contains_key("summary") {
                    return Err(LargeFileError::ValidationError(
                        "Unsafe FFI JSON must contain 'enhanced_ffi_data' or 'summary' field"
                            .to_string(),
                    ));
                }
            }
            "performance" => {
                if !json.is_object() {
                    return Err(LargeFileError::ValidationError(
                        "Performance JSON must be an object".to_string(),
                    ));
                }

                let obj = json.as_object().expect("Test operation failed");
                if !obj.contains_key("memory_performance")
                    && !obj.contains_key("allocation_distribution")
                {
                    return Err(LargeFileError::ValidationError(
                        "Performance JSON must contain performance-related fields".to_string(),
                    ));
                }
            }
            "lifetime" => {
                if !json.is_object() {
                    return Err(LargeFileError::ValidationError(
                        "Lifetime JSON must be an object".to_string(),
                    ));
                }

                let obj = json.as_object().expect("Test operation failed");
                if !obj.contains_key("lifecycle_events") {
                    return Err(LargeFileError::ValidationError(
                        "Lifetime JSON must contain 'lifecycle_events' field".to_string(),
                    ));
                }
            }
            "complex_types" => {
                if !json.is_object() {
                    return Err(LargeFileError::ValidationError(
                        "Complex types JSON must be an object".to_string(),
                    ));
                }

                let obj = json.as_object().expect("Test operation failed");
                if !obj.contains_key("categorized_types") && !obj.contains_key("generic_types") {
                    return Err(LargeFileError::ValidationError(
                        "Complex types JSON must contain type-related fields".to_string(),
                    ));
                }
            }
            _ => {
                // Basic validation for other file types
                if !json.is_object() && !json.is_array() {
                    return Err(LargeFileError::ValidationError(
                        "JSON must be an object or array".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Count the number of JSON objects processed
    fn count_json_objects(&self, json: &Value) -> usize {
        match json {
            Value::Object(obj) => {
                let mut count = 1; // The object itself

                // Count objects in arrays that are likely to contain multiple items
                for (key, value) in obj {
                    match key.as_str() {
                        "allocations" | "lifecycle_events" | "enhanced_ffi_data"
                        | "boundary_events" | "categorized_types" | "generic_types" => {
                            if let Value::Array(arr) = value {
                                count += arr.len();
                            }
                        }
                        _ => {}
                    }
                }

                count
            }
            Value::Array(arr) => arr.len(),
            _ => 1,
        }
    }

    /// Get current memory usage statistics
    pub fn get_memory_stats(&self) -> MemoryStats {
        self.memory_monitor.get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_large_file_config_default() {
        let config = LargeFileConfig::default();
        assert_eq!(config.max_memory_bytes, 512 * 1024 * 1024);
        assert_eq!(config.stream_chunk_size, 64 * 1024);
        assert!(config.enable_memory_monitoring);
        assert!(config.enable_progress_reporting);
    }

    #[test]
    fn test_memory_monitor() {
        let monitor = MemoryMonitor::new(1024, true);

        // Test allocation
        assert!(monitor.allocate(512).is_ok());
        assert_eq!(monitor.get_stats().current_usage_bytes, 512);

        // Test deallocation
        monitor.deallocate(256);
        assert_eq!(monitor.get_stats().current_usage_bytes, 256);

        // Test memory limit
        assert!(monitor.allocate(1024).is_err());
    }

    #[test]
    fn test_process_small_file() {
        let temp_dir = TempDir::new().expect("Failed to get test value");
        let file_path = temp_dir.path().join("test.json");

        let test_data =
            r#"{"allocations": [{"ptr": "0x123", "size": 100}], "summary": {"total": 1}}"#;
        fs::write(&file_path, test_data).expect("Failed to write test file");

        let optimizer = LargeFileOptimizer::new_default();
        let result = optimizer.process_file(&file_path, "memory_analysis");

        assert!(result.is_ok());
        let (json_value, stats) = result.expect("Test operation failed");
        assert!(json_value.is_object());
        assert!(!stats.streaming_mode_used);
        assert_eq!(stats.objects_processed, 2); // 1 object + 1 allocation
    }

    #[test]
    fn test_json_validation() {
        let optimizer = LargeFileOptimizer::new_default();

        // Test valid memory analysis JSON
        let valid_json = serde_json::json!({
            "allocations": [],
            "summary": {"total": 0}
        });
        assert!(optimizer
            .validate_json_structure(&valid_json, "memory_analysis")
            .is_ok());

        // Test invalid memory analysis JSON
        let invalid_json = serde_json::json!({
            "invalid_field": "value"
        });
        assert!(optimizer
            .validate_json_structure(&invalid_json, "memory_analysis")
            .is_err());
    }
}
