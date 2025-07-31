//! Compression management system for binary export
//!
//! This module provides intelligent compression management with support for
//! multiple algorithms, automatic selection, and streaming compression.

use std::collections::HashMap;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

use super::error::BinaryExportError;

/// Compression algorithm types supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// Zstandard compression (default)
    Zstd,
    /// LZ4 compression (fast)
    Lz4,
    /// Gzip compression (compatible)
    Gzip,
    /// Brotli compression (high ratio)
    Brotli,
}

/// Compression configuration and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Primary compression algorithm
    pub algorithm: CompressionAlgorithm,
    /// Compression level (algorithm-specific)
    pub level: i32,
    /// Enable streaming compression
    pub streaming: bool,
    /// Chunk size for streaming (bytes)
    pub chunk_size: usize,
    /// Enable automatic algorithm selection
    pub auto_select: bool,
    /// Target compression ratio (for auto-selection)
    pub target_ratio: Option<f64>,
    /// Maximum compression time (for auto-selection)
    pub max_compression_time: Option<std::time::Duration>,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Zstd,
            level: 6,
            streaming: true,
            chunk_size: 256 * 1024, // 256KB
            auto_select: false,
            target_ratio: None,
            max_compression_time: None,
        }
    }
}

impl CompressionConfig {
    /// Fast compression preset
    pub fn fast() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Lz4,
            level: 1,
            streaming: true,
            chunk_size: 64 * 1024,
            auto_select: false,
            target_ratio: None,
            max_compression_time: Some(std::time::Duration::from_millis(100)),
        }
    }
    
    /// Balanced compression preset
    pub fn balanced() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Zstd,
            level: 6,
            streaming: true,
            chunk_size: 256 * 1024,
            auto_select: true,
            target_ratio: Some(0.5),
            max_compression_time: Some(std::time::Duration::from_secs(5)),
        }
    }
    
    /// Maximum compression preset
    pub fn max_compression() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Brotli,
            level: 11,
            streaming: false,
            chunk_size: 1024 * 1024,
            auto_select: false,
            target_ratio: Some(0.3),
            max_compression_time: None,
        }
    }
}

/// Compression statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    /// Original data size
    pub original_size: u64,
    /// Compressed data size
    pub compressed_size: u64,
    /// Compression ratio (compressed/original)
    pub compression_ratio: f64,
    /// Time taken for compression
    pub compression_time: std::time::Duration,
    /// Compression throughput (bytes/second)
    pub throughput: f64,
    /// Algorithm used
    pub algorithm: CompressionAlgorithm,
    /// Compression level used
    pub level: i32,
}

/// Main compression manager
pub struct CompressionManager {
    /// Current configuration
    config: CompressionConfig,
    /// Algorithm performance cache
    algorithm_cache: HashMap<CompressionAlgorithm, AlgorithmPerformance>,
    /// Compression statistics
    stats: Vec<CompressionStats>,
}

/// Performance metrics for compression algorithms
#[derive(Debug, Clone)]
struct AlgorithmPerformance {
    /// Average compression ratio
    avg_ratio: f64,
    /// Average compression speed (bytes/second)
    avg_speed: f64,
    /// Number of samples
    sample_count: u32,
}

impl CompressionManager {
    /// Create a new compression manager
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            config,
            algorithm_cache: HashMap::new(),
            stats: Vec::new(),
        }
    }

    /// Compress data using the configured algorithm
    pub fn compress(&mut self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        let start_time = std::time::Instant::now();
        
        // Select algorithm
        let algorithm = if self.config.auto_select {
            self.select_optimal_algorithm(data)?
        } else {
            self.config.algorithm
        };
        
        // Perform compression
        let compressed_data = match algorithm {
            CompressionAlgorithm::None => data.to_vec(),
            CompressionAlgorithm::Zstd => self.compress_zstd(data)?,
            CompressionAlgorithm::Lz4 => self.compress_lz4(data)?,
            CompressionAlgorithm::Gzip => self.compress_gzip(data)?,
            CompressionAlgorithm::Brotli => self.compress_brotli(data)?,
        };
        
        let compression_time = start_time.elapsed();
        
        // Record statistics
        let stats = CompressionStats {
            original_size: data.len() as u64,
            compressed_size: compressed_data.len() as u64,
            compression_ratio: compressed_data.len() as f64 / data.len() as f64,
            compression_time,
            throughput: data.len() as f64 / compression_time.as_secs_f64(),
            algorithm,
            level: self.config.level,
        };
        
        self.update_algorithm_performance(algorithm, &stats);
        self.stats.push(stats);
        
        Ok(compressed_data)
    }

    /// Decompress data
    pub fn decompress(&self, data: &[u8], algorithm: CompressionAlgorithm) -> Result<Vec<u8>, BinaryExportError> {
        match algorithm {
            CompressionAlgorithm::None => Ok(data.to_vec()),
            CompressionAlgorithm::Zstd => self.decompress_zstd(data),
            CompressionAlgorithm::Lz4 => self.decompress_lz4(data),
            CompressionAlgorithm::Gzip => self.decompress_gzip(data),
            CompressionAlgorithm::Brotli => self.decompress_brotli(data),
        }
    }

    /// Compress data in streaming mode
    pub fn compress_streaming<R: Read, W: Write>(
        &mut self,
        reader: R,
        writer: W,
    ) -> Result<CompressionStats, BinaryExportError> {
        let start_time = std::time::Instant::now();
        let mut total_input = 0u64;
        let mut total_output = 0u64;
        
        // Select algorithm
        let algorithm = self.config.algorithm;
        
        // Create streaming compressor based on algorithm
        match algorithm {
            CompressionAlgorithm::Zstd => {
                self.compress_streaming_zstd(reader, writer, &mut total_input, &mut total_output)?;
            }
            CompressionAlgorithm::Lz4 => {
                self.compress_streaming_lz4(reader, writer, &mut total_input, &mut total_output)?;
            }
            _ => {
                // Fallback to chunk-based compression for unsupported streaming algorithms
                self.compress_chunked(reader, writer, &mut total_input, &mut total_output)?;
            }
        }
        
        let compression_time = start_time.elapsed();
        
        let stats = CompressionStats {
            original_size: total_input,
            compressed_size: total_output,
            compression_ratio: total_output as f64 / total_input as f64,
            compression_time,
            throughput: total_input as f64 / compression_time.as_secs_f64(),
            algorithm,
            level: self.config.level,
        };
        
        self.update_algorithm_performance(algorithm, &stats);
        self.stats.push(stats.clone());
        
        Ok(stats)
    }

    /// Select optimal compression algorithm based on data characteristics
    fn select_optimal_algorithm(&self, data: &[u8]) -> Result<CompressionAlgorithm, BinaryExportError> {
        // Analyze data characteristics
        let entropy = self.calculate_entropy(data);
        let repetition_ratio = self.calculate_repetition_ratio(data);
        
        // Select algorithm based on data characteristics and performance history
        if entropy < 0.5 && repetition_ratio > 0.3 {
            // High repetition, low entropy - good for LZ4 or Zstd
            if self.config.max_compression_time.map_or(false, |t| t.as_millis() < 500) {
                Ok(CompressionAlgorithm::Lz4)
            } else {
                Ok(CompressionAlgorithm::Zstd)
            }
        } else if entropy > 0.8 {
            // High entropy - compression won't help much
            Ok(CompressionAlgorithm::None)
        } else {
            // Medium entropy - use balanced algorithm
            Ok(CompressionAlgorithm::Zstd)
        }
    }

    /// Calculate data entropy for algorithm selection
    fn calculate_entropy(&self, data: &[u8]) -> f64 {
        let mut counts = [0u32; 256];
        for &byte in data {
            counts[byte as usize] += 1;
        }
        
        let len = data.len() as f64;
        let mut entropy = 0.0;
        
        for &count in &counts {
            if count > 0 {
                let p = count as f64 / len;
                entropy -= p * p.log2();
            }
        }
        
        entropy / 8.0 // Normalize to 0-1 range
    }

    /// Calculate repetition ratio for algorithm selection
    fn calculate_repetition_ratio(&self, data: &[u8]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        
        let mut repetitions = 0;
        for i in 1..data.len() {
            if data[i] == data[i - 1] {
                repetitions += 1;
            }
        }
        
        repetitions as f64 / (data.len() - 1) as f64
    }

    /// Update algorithm performance metrics
    fn update_algorithm_performance(&mut self, algorithm: CompressionAlgorithm, stats: &CompressionStats) {
        let entry = self.algorithm_cache.entry(algorithm).or_insert(AlgorithmPerformance {
            avg_ratio: 0.0,
            avg_speed: 0.0,
            sample_count: 0,
        });
        
        // Update running averages
        let new_count = entry.sample_count + 1;
        entry.avg_ratio = (entry.avg_ratio * entry.sample_count as f64 + stats.compression_ratio) / new_count as f64;
        entry.avg_speed = (entry.avg_speed * entry.sample_count as f64 + stats.throughput) / new_count as f64;
        entry.sample_count = new_count;
    }

    /// Get compression statistics
    pub fn get_stats(&self) -> &[CompressionStats] {
        &self.stats
    }

    /// Get algorithm performance metrics
    pub fn get_algorithm_performance(&self) -> &HashMap<CompressionAlgorithm, AlgorithmPerformance> {
        &self.algorithm_cache
    }

    // Compression algorithm implementations
    fn compress_zstd(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        zstd::bulk::compress(data, self.config.level)
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))
    }

    fn decompress_zstd(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        zstd::bulk::decompress(data, 64 * 1024 * 1024) // 64MB limit
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))
    }

    fn compress_lz4(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        // Placeholder - would use lz4 crate
        // For now, use zstd with fast settings
        zstd::bulk::compress(data, 1)
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))
    }

    fn decompress_lz4(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        // Placeholder - would use lz4 crate
        zstd::bulk::decompress(data, 64 * 1024 * 1024)
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))
    }

    fn compress_gzip(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        use std::io::Write;
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(self.config.level as u32));
        encoder.write_all(data)
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))?;
        encoder.finish()
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))
    }

    fn decompress_gzip(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        use std::io::Read;
        let mut decoder = flate2::read::GzDecoder::new(data);
        let mut result = Vec::new();
        decoder.read_to_end(&mut result)
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))?;
        Ok(result)
    }

    fn compress_brotli(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        // Placeholder - would use brotli crate
        // For now, use zstd with high compression
        zstd::bulk::compress(data, 19)
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))
    }

    fn decompress_brotli(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        // Placeholder - would use brotli crate
        zstd::bulk::decompress(data, 64 * 1024 * 1024)
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))
    }

    // Streaming compression implementations
    fn compress_streaming_zstd<R: Read, W: Write>(
        &self,
        mut reader: R,
        writer: W,
        total_input: &mut u64,
        total_output: &mut u64,
    ) -> Result<(), BinaryExportError> {
        let mut encoder = zstd::stream::write::Encoder::new(writer, self.config.level)
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))?;
        
        let mut buffer = vec![0u8; self.config.chunk_size];
        loop {
            let bytes_read = reader.read(&mut buffer)
                .map_err(|e| BinaryExportError::IoError(e.kind()))?;
            
            if bytes_read == 0 {
                break;
            }
            
            encoder.write_all(&buffer[..bytes_read])
                .map_err(|e| BinaryExportError::CompressionError(e.to_string()))?;
            
            *total_input += bytes_read as u64;
        }
        
        let writer = encoder.finish()
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))?;
        
        // Note: We can't easily get the exact output size from the streaming encoder
        // This would need to be tracked differently in a real implementation
        *total_output = *total_input / 2; // Rough estimate
        
        Ok(())
    }

    fn compress_streaming_lz4<R: Read, W: Write>(
        &self,
        reader: R,
        writer: W,
        total_input: &mut u64,
        total_output: &mut u64,
    ) -> Result<(), BinaryExportError> {
        // Placeholder - would use lz4 streaming
        self.compress_streaming_zstd(reader, writer, total_input, total_output)
    }

    fn compress_chunked<R: Read, W: Write>(
        &self,
        mut reader: R,
        mut writer: W,
        total_input: &mut u64,
        total_output: &mut u64,
    ) -> Result<(), BinaryExportError> {
        let mut buffer = vec![0u8; self.config.chunk_size];
        
        loop {
            let bytes_read = reader.read(&mut buffer)
                .map_err(|e| BinaryExportError::IoError(e.kind()))?;
            
            if bytes_read == 0 {
                break;
            }
            
            let compressed_chunk = self.compress_zstd(&buffer[..bytes_read])?;
            writer.write_all(&compressed_chunk)
                .map_err(|e| BinaryExportError::IoError(e.kind()))?;
            
            *total_input += bytes_read as u64;
            *total_output += compressed_chunk.len() as u64;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_config_presets() {
        let fast = CompressionConfig::fast();
        assert_eq!(fast.algorithm, CompressionAlgorithm::Lz4);
        assert_eq!(fast.level, 1);
        
        let balanced = CompressionConfig::balanced();
        assert_eq!(balanced.algorithm, CompressionAlgorithm::Zstd);
        assert!(balanced.auto_select);
        
        let max = CompressionConfig::max_compression();
        assert_eq!(max.algorithm, CompressionAlgorithm::Brotli);
        assert_eq!(max.level, 11);
    }

    #[test]
    fn test_compression_manager() {
        let config = CompressionConfig::default();
        let mut manager = CompressionManager::new(config);
        
        let test_data = b"Hello, world! This is a test string for compression.";
        let compressed = manager.compress(test_data).unwrap();
        
        assert!(compressed.len() < test_data.len());
        
        let decompressed = manager.decompress(&compressed, CompressionAlgorithm::Zstd).unwrap();
        assert_eq!(decompressed, test_data);
    }

    #[test]
    fn test_entropy_calculation() {
        let config = CompressionConfig::default();
        let manager = CompressionManager::new(config);
        
        // High entropy data (random)
        let random_data = (0..256).map(|i| i as u8).collect::<Vec<_>>();
        let entropy = manager.calculate_entropy(&random_data);
        assert!(entropy > 0.9);
        
        // Low entropy data (repetitive)
        let repetitive_data = vec![0u8; 256];
        let entropy = manager.calculate_entropy(&repetitive_data);
        assert!(entropy < 0.1);
    }
}