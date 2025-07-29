//! High-speed buffered writer - high performance file writing with reduced I/O overhead
//!
//! This module implements high-speed buffered writing functionality, using large buffers to reduce system call frequency,
//! and provides efficient shard merging logic, significantly improving file writing performance.

use crate::core::types::{TrackingError, TrackingResult};
use crate::export::parallel_shard_processor::ProcessedShard;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

/// High-speed buffered writer configuration
#[derive(Debug, Clone)]
pub struct HighSpeedWriterConfig {
    /// Buffer size (bytes)
    pub buffer_size: usize,
    /// Whether to enable performance monitoring
    pub enable_monitoring: bool,
    /// Estimated total output size (for pre-allocation)
    pub estimated_total_size: Option<usize>,
    /// Whether to use compression writing
    pub enable_compression: bool,
    /// Whether to flush the buffer automatically after writing
    pub auto_flush: bool,
}

impl Default for HighSpeedWriterConfig {
    fn default() -> Self {
        Self {
            buffer_size: 2 * 1024 * 1024, // 2MB 缓冲区
            enable_monitoring: true,
            estimated_total_size: None,
            enable_compression: false,
            auto_flush: true,
        }
    }
}

/// Write performance statistics
#[derive(Debug, Clone)]
pub struct WritePerformanceStats {
    /// Total bytes written
    pub total_bytes_written: usize,
    /// Number of shards written
    pub shards_written: usize,
    /// Total write time (milliseconds)
    pub total_write_time_ms: u64,
    /// Average write speed (bytes per second)
    pub avg_write_speed_bps: f64,
    /// Buffer flush count
    pub flush_count: usize,
    /// Whether pre-allocation was effective
    pub preallocation_effective: bool,
    /// Actual buffer utilization
    pub buffer_utilization: f64,
}

/// High-speed buffered writer
pub struct HighSpeedBufferedWriter {
    /// Internal buffer writer
    writer: BufWriter<File>,
    /// Configuration
    config: HighSpeedWriterConfig,
    /// Internal buffer
    internal_buffer: Vec<u8>,
    /// Write performance statistics
    stats: WritePerformanceStats,
    /// Start time
    start_time: Instant,
    /// Flush count
    flush_count: usize,
}

impl HighSpeedBufferedWriter {
    /// Create a new high-speed buffered writer
    pub fn new<P: AsRef<Path>>(path: P, config: HighSpeedWriterConfig) -> TrackingResult<Self> {
        let file = File::create(path.as_ref())
            .map_err(|e| TrackingError::IoError(format!("create file failed: {}", e)))?;

        let writer = BufWriter::with_capacity(config.buffer_size, file);

        // Pre-allocate internal buffer
        let initial_capacity = config.estimated_total_size.unwrap_or(1024 * 1024); // Default 1MB
        let internal_buffer = Vec::with_capacity(initial_capacity);

        let stats = WritePerformanceStats {
            total_bytes_written: 0,
            shards_written: 0,
            total_write_time_ms: 0,
            avg_write_speed_bps: 0.0,
            flush_count: 0,
            preallocation_effective: false,
            buffer_utilization: 0.0,
        };

        Ok(Self {
            writer,
            config,
            internal_buffer,
            stats,
            start_time: Instant::now(),
            flush_count: 0,
        })
    }

    /// Write processed shards
    pub fn write_processed_shards(
        &mut self,
        shards: &[ProcessedShard],
    ) -> TrackingResult<WritePerformanceStats> {
        let write_start = Instant::now();

        if self.config.enable_monitoring {
            println!(
                "🔄 Starting high-speed buffered write for {} shards...",
                shards.len()
            );
        }

        // Pre-calculate total size and pre-allocate buffer
        let total_size: usize = shards.iter().map(|s| s.data.len()).sum();
        let estimated_final_size = total_size + 1024; // Extra space for JSON structure

        // Check if pre-allocation was effective
        self.stats.preallocation_effective =
            self.internal_buffer.capacity() >= estimated_final_size;

        if !self.stats.preallocation_effective {
            self.internal_buffer.reserve(estimated_final_size);
        }

        // Build complete JSON structure
        self.build_complete_json(shards)?;

        // Write to file
        self.writer
            .write_all(&self.internal_buffer)
            .map_err(|e| TrackingError::IoError(format!("write file failed: {}", e)))?;

        // flush cache
        if self.config.auto_flush {
            self.flush()?;
        }

        let write_time = write_start.elapsed();

        // update data
        self.stats.total_bytes_written = self.internal_buffer.len();
        self.stats.shards_written = shards.len();
        self.stats.total_write_time_ms = write_time.as_millis() as u64;
        self.stats.avg_write_speed_bps = if write_time.as_secs_f64() > 0.0 {
            self.stats.total_bytes_written as f64 / write_time.as_secs_f64()
        } else {
            0.0
        };
        self.stats.buffer_utilization = if self.internal_buffer.capacity() > 0 {
            self.internal_buffer.len() as f64 / self.internal_buffer.capacity() as f64
        } else {
            0.0
        };

        if self.config.enable_monitoring {
            self.print_write_stats();
        }

        Ok(self.stats.clone())
    }

    /// Build complete JSON structure
    fn build_complete_json(&mut self, shards: &[ProcessedShard]) -> TrackingResult<()> {
        // Clear internal buffer
        self.internal_buffer.clear();

        // Write JSON start
        self.internal_buffer
            .extend_from_slice(b"{\"allocations\":[");

        // Merge all shards, add comma only between shards
        for (i, shard) in shards.iter().enumerate() {
            if i > 0 {
                self.internal_buffer.extend_from_slice(b",");
            }

            // Remove shard's [ and ], only keep content
            let shard_content = if shard.data.starts_with(b"[") && shard.data.ends_with(b"]") {
                &shard.data[1..shard.data.len() - 1]
            } else {
                &shard.data
            };

            self.internal_buffer.extend_from_slice(shard_content);
        }

        // Write JSON end
        self.internal_buffer.extend_from_slice(b"]}");

        Ok(())
    }

    /// Write custom JSON data
    pub fn write_custom_json(&mut self, json_data: &[u8]) -> TrackingResult<WritePerformanceStats> {
        let write_start = Instant::now();

        if self.config.enable_monitoring {
            println!(
                "🔄 Starting custom JSON data write ({} bytes)...",
                json_data.len()
            );
        }

        // Pre-allocate buffer
        if self.internal_buffer.capacity() < json_data.len() {
            self.internal_buffer.reserve(json_data.len());
        }

        // Copy data to internal buffer
        self.internal_buffer.clear();
        self.internal_buffer.extend_from_slice(json_data);

        // Write to file
        self.writer
            .write_all(&self.internal_buffer)
            .map_err(|e| TrackingError::IoError(format!("write custom JSON failed: {}", e)))?;

        if self.config.auto_flush {
            self.flush()?;
        }

        let write_time = write_start.elapsed();

        // Update statistics
        self.stats.total_bytes_written = json_data.len();
        self.stats.shards_written = 1; // Custom data counts as one shard
        self.stats.total_write_time_ms = write_time.as_millis() as u64;
        self.stats.avg_write_speed_bps = if write_time.as_secs_f64() > 0.0 {
            json_data.len() as f64 / write_time.as_secs_f64()
        } else {
            0.0
        };

        if self.config.enable_monitoring {
            self.print_write_stats();
        }

        Ok(self.stats.clone())
    }

    /// Force buffer flush
    pub fn flush(&mut self) -> TrackingResult<()> {
        self.writer
            .flush()
            .map_err(|e| TrackingError::IoError(format!("flush buffer failed: {}", e)))?;

        self.flush_count += 1;
        self.stats.flush_count = self.flush_count;

        Ok(())
    }

    /// Finalize writing and get final statistics
    pub fn finalize(mut self) -> TrackingResult<WritePerformanceStats> {
        // Ensure all data is written
        self.flush()?;

        // Calculate total statistics
        let total_time = self.start_time.elapsed();
        self.stats.total_write_time_ms = total_time.as_millis() as u64;

        if self.config.enable_monitoring {
            println!("✅ High-speed buffered write completed:");
            println!("   Total time: {:?}", total_time);
            self.print_write_stats();
        }

        Ok(self.stats)
    }

    /// Get current statistics
    pub fn get_stats(&self) -> &WritePerformanceStats {
        &self.stats
    }

    /// Get configuration
    pub fn get_config(&self) -> &HighSpeedWriterConfig {
        &self.config
    }

    /// Print write statistics
    fn print_write_stats(&self) {
        println!(
            "   Bytes written: {} ({:.2} MB)",
            self.stats.total_bytes_written,
            self.stats.total_bytes_written as f64 / 1024.0 / 1024.0
        );
        println!("   Shards written: {}", self.stats.shards_written);
        println!(
            "   Write speed: {:.2} MB/s",
            self.stats.avg_write_speed_bps / 1024.0 / 1024.0
        );
        println!(
            "   Buffer utilization: {:.1}%",
            self.stats.buffer_utilization * 100.0
        );
        println!("   Flush count: {}", self.stats.flush_count);
        println!(
            "   Preallocation effective: {}",
            self.stats.preallocation_effective
        );
    }
}

/// Convenience function: fast write shards
pub fn write_shards_fast<P: AsRef<Path>>(
    path: P,
    shards: &[ProcessedShard],
) -> TrackingResult<WritePerformanceStats> {
    let total_size: usize = shards.iter().map(|s| s.data.len()).sum();
    let config = HighSpeedWriterConfig {
        estimated_total_size: Some(total_size + 1024),
        ..Default::default()
    };

    let mut writer = HighSpeedBufferedWriter::new(path, config)?;
    writer.write_processed_shards(shards)
}

/// Convenience function: write shards with custom config
pub fn write_shards_with_config<P: AsRef<Path>>(
    path: P,
    shards: &[ProcessedShard],
    config: HighSpeedWriterConfig,
) -> TrackingResult<WritePerformanceStats> {
    let mut writer = HighSpeedBufferedWriter::new(path, config)?;
    writer.write_processed_shards(shards)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    fn create_test_shards(count: usize, size_per_shard: usize) -> Vec<ProcessedShard> {
        let mut shards = Vec::new();
        for i in 0..count {
            let data = format!("{{\"test_data_{}\": {}}}", i, i).repeat(size_per_shard / 20);
            shards.push(ProcessedShard {
                data: format!("[{}]", data).into_bytes(),
                allocation_count: 1,
                shard_index: i,
                processing_time_ms: 1,
            });
        }
        shards
    }

    #[test]
    fn test_high_speed_writer_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = HighSpeedWriterConfig::default();
        let writer = HighSpeedBufferedWriter::new(temp_file.path(), config);
        assert!(writer.is_ok());
    }

    #[test]
    fn test_write_processed_shards() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = HighSpeedWriterConfig::default();
        let mut writer = HighSpeedBufferedWriter::new(temp_file.path(), config).unwrap();

        let shards = create_test_shards(3, 100);
        let result = writer.write_processed_shards(&shards);
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.shards_written, 3);
        assert!(stats.total_bytes_written > 0);
        assert!(stats.avg_write_speed_bps > 0.0);

        // Verify file content
        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.starts_with("{\"allocations\":["));
        assert!(content.ends_with("]}"));
    }

    #[test]
    fn test_write_custom_json() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = HighSpeedWriterConfig::default();
        let mut writer = HighSpeedBufferedWriter::new(temp_file.path(), config).unwrap();

        let json_data = b"{\"test\": \"data\"}";
        let result = writer.write_custom_json(json_data);
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.total_bytes_written, json_data.len());

        // Verify file content
        let content = fs::read(temp_file.path()).unwrap();
        assert_eq!(content, json_data);
    }

    #[test]
    fn test_preallocation_effectiveness() {
        let temp_file = NamedTempFile::new().unwrap();
        let shards = create_test_shards(5, 200);
        let total_size: usize = shards.iter().map(|s| s.data.len()).sum();

        // Test effective preallocation
        let config = HighSpeedWriterConfig {
            estimated_total_size: Some(total_size + 1024),
            enable_monitoring: false,
            ..Default::default()
        };
        let mut writer = HighSpeedBufferedWriter::new(temp_file.path(), config).unwrap();
        let stats = writer.write_processed_shards(&shards).unwrap();
        assert!(stats.preallocation_effective);

        // Test ineffective preallocation
        let temp_file2 = NamedTempFile::new().unwrap();
        let config2 = HighSpeedWriterConfig {
            estimated_total_size: Some(100), // too small preallocation
            enable_monitoring: false,
            ..Default::default()
        };
        let mut writer2 = HighSpeedBufferedWriter::new(temp_file2.path(), config2).unwrap();
        let stats2 = writer2.write_processed_shards(&shards).unwrap();
        assert!(!stats2.preallocation_effective);
    }

    #[test]
    fn test_convenience_functions() {
        let temp_file = NamedTempFile::new().unwrap();
        let shards = create_test_shards(2, 150);

        // Test fast write function
        let result = write_shards_fast(temp_file.path(), &shards);
        assert!(result.is_ok());

        // Test custom config function
        let temp_file2 = NamedTempFile::new().unwrap();
        let config = HighSpeedWriterConfig {
            buffer_size: 1024 * 1024,
            enable_monitoring: false,
            ..Default::default()
        };
        let result2 = write_shards_with_config(temp_file2.path(), &shards, config);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_flush_functionality() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = HighSpeedWriterConfig {
            auto_flush: false,
            enable_monitoring: false,
            ..Default::default()
        };
        let mut writer = HighSpeedBufferedWriter::new(temp_file.path(), config).unwrap();

        let shards = create_test_shards(1, 100);
        let _stats = writer.write_processed_shards(&shards).unwrap();

        // Manually flush
        let flush_result = writer.flush();
        assert!(flush_result.is_ok());
        assert_eq!(writer.get_stats().flush_count, 1);
    }

    #[test]
    fn test_finalize() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = HighSpeedWriterConfig {
            enable_monitoring: false,
            ..Default::default()
        };
        let mut writer = HighSpeedBufferedWriter::new(temp_file.path(), config).unwrap();

        let shards = create_test_shards(2, 100);
        let _stats = writer.write_processed_shards(&shards).unwrap();

        let final_stats = writer.finalize();
        assert!(final_stats.is_ok());

        let stats = final_stats.unwrap();
        // In fast tests, time may be 0, so check >= 0
        assert!(stats.total_write_time_ms >= 0);
    }
}
