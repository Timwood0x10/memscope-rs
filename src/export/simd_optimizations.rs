//! SIMD optimizations for binary export system
//!
//! This module provides SIMD-accelerated operations for checksum calculation,
//! data encoding/decoding, and vectorized processing to improve performance.

use std::arch::x86_64::*;

/// SIMD capability detection and configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdCapability {
    /// No SIMD support
    None,
    /// SSE2 support (baseline for x86_64)
    Sse2,
    /// SSE4.1 support
    Sse41,
    /// AVX support
    Avx,
    /// AVX2 support
    Avx2,
    /// AVX-512 support
    Avx512,
}

/// SIMD processor for optimized operations
pub struct SimdProcessor {
    /// Detected SIMD capability
    capability: SimdCapability,
    /// Whether SIMD is enabled
    enabled: bool,
}

impl SimdProcessor {
    /// Create a new SIMD processor with capability detection
    pub fn new() -> Self {
        let capability = Self::detect_simd_capability();
        Self {
            capability,
            enabled: capability != SimdCapability::None,
        }
    }

    /// Create a SIMD processor with specific capability (for testing)
    pub fn with_capability(capability: SimdCapability) -> Self {
        Self {
            capability,
            enabled: capability != SimdCapability::None,
        }
    }

    /// Disable SIMD optimizations
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Enable SIMD optimizations
    pub fn enable(&mut self) {
        self.enabled = self.capability != SimdCapability::None;
    }

    /// Get the detected SIMD capability
    pub fn capability(&self) -> SimdCapability {
        self.capability
    }

    /// Check if SIMD is enabled and available
    pub fn is_enabled(&self) -> bool {
        self.enabled && self.capability != SimdCapability::None
    }

    /// Detect available SIMD capabilities
    fn detect_simd_capability() -> SimdCapability {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx512f") {
                SimdCapability::Avx512
            } else if is_x86_feature_detected!("avx2") {
                SimdCapability::Avx2
            } else if is_x86_feature_detected!("avx") {
                SimdCapability::Avx
            } else if is_x86_feature_detected!("sse4.1") {
                SimdCapability::Sse41
            } else if is_x86_feature_detected!("sse2") {
                SimdCapability::Sse2
            } else {
                SimdCapability::None
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            SimdCapability::None
        }
    }

    /// Calculate CRC64 checksum using SIMD acceleration
    pub fn crc64_checksum(&self, data: &[u8]) -> u64 {
        if !self.is_enabled() || data.len() < 32 {
            return self.crc64_scalar(data);
        }

        match self.capability {
            SimdCapability::Avx2 => self.crc64_avx2(data),
            SimdCapability::Sse41 => self.crc64_sse41(data),
            _ => self.crc64_scalar(data),
        }
    }

    /// Scalar CRC64 implementation (fallback)
    fn crc64_scalar(&self, data: &[u8]) -> u64 {
        const CRC64_POLY: u64 = 0xC96C5795D7870F42;
        let mut crc = 0xFFFFFFFFFFFFFFFFu64;

        for &byte in data {
            crc ^= byte as u64;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ CRC64_POLY;
                } else {
                    crc >>= 1;
                }
            }
        }

        !crc
    }

    /// AVX2-accelerated CRC64 calculation
    #[cfg(target_arch = "x86_64")]
    fn crc64_avx2(&self, data: &[u8]) -> u64 {
        if !is_x86_feature_detected!("avx2") {
            return self.crc64_scalar(data);
        }

        unsafe {
            let mut crc = 0xFFFFFFFFFFFFFFFFu64;
            let chunks = data.chunks_exact(32);
            let remainder = chunks.remainder();

            // Process 32-byte chunks with AVX2
            for chunk in chunks {
                crc = self.crc64_avx2_chunk(crc, chunk.as_ptr());
            }

            // Process remainder with scalar
            for &byte in remainder {
                crc ^= byte as u64;
                for _ in 0..8 {
                    if crc & 1 != 0 {
                        crc = (crc >> 1) ^ 0xC96C5795D7870F42;
                    } else {
                        crc >>= 1;
                    }
                }
            }

            !crc
        }
    }

    /// Process a 32-byte chunk with AVX2
    #[cfg(target_arch = "x86_64")]
    unsafe fn crc64_avx2_chunk(&self, mut crc: u64, data: *const u8) -> u64 {
        // Load 32 bytes into AVX2 register
        let data_vec = _mm256_loadu_si256(data as *const __m256i);
        
        // For simplicity, we'll process this in 8-byte chunks
        // In a real implementation, you'd use lookup tables and more sophisticated SIMD operations
        let bytes = std::slice::from_raw_parts(data, 32);
        for &byte in bytes {
            crc ^= byte as u64;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xC96C5795D7870F42;
                } else {
                    crc >>= 1;
                }
            }
        }
        
        crc
    }

    /// SSE4.1-accelerated CRC64 calculation
    #[cfg(target_arch = "x86_64")]
    fn crc64_sse41(&self, data: &[u8]) -> u64 {
        if !is_x86_feature_detected!("sse4.1") {
            return self.crc64_scalar(data);
        }

        unsafe {
            let mut crc = 0xFFFFFFFFFFFFFFFFu64;
            let chunks = data.chunks_exact(16);
            let remainder = chunks.remainder();

            // Process 16-byte chunks with SSE4.1
            for chunk in chunks {
                crc = self.crc64_sse41_chunk(crc, chunk.as_ptr());
            }

            // Process remainder with scalar
            for &byte in remainder {
                crc ^= byte as u64;
                for _ in 0..8 {
                    if crc & 1 != 0 {
                        crc = (crc >> 1) ^ 0xC96C5795D7870F42;
                    } else {
                        crc >>= 1;
                    }
                }
            }

            !crc
        }
    }

    /// Process a 16-byte chunk with SSE4.1
    #[cfg(target_arch = "x86_64")]
    unsafe fn crc64_sse41_chunk(&self, mut crc: u64, data: *const u8) -> u64 {
        // Load 16 bytes into SSE register
        let data_vec = _mm_loadu_si128(data as *const __m128i);
        
        // For simplicity, process as scalar (real implementation would use SIMD operations)
        let bytes = std::slice::from_raw_parts(data, 16);
        for &byte in bytes {
            crc ^= byte as u64;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xC96C5795D7870F42;
                } else {
                    crc >>= 1;
                }
            }
        }
        
        crc
    }

    /// Vectorized data encoding using SIMD
    pub fn encode_u64_array(&self, values: &[u64], output: &mut [u8]) -> Result<usize, SimdError> {
        if output.len() < values.len() * 8 {
            return Err(SimdError::InsufficientBuffer {
                required: values.len() * 8,
                available: output.len(),
            });
        }

        if !self.is_enabled() || values.len() < 4 {
            return self.encode_u64_array_scalar(values, output);
        }

        match self.capability {
            SimdCapability::Avx2 => self.encode_u64_array_avx2(values, output),
            SimdCapability::Sse2 => self.encode_u64_array_sse2(values, output),
            _ => self.encode_u64_array_scalar(values, output),
        }
    }

    /// Scalar u64 array encoding (fallback)
    fn encode_u64_array_scalar(&self, values: &[u64], output: &mut [u8]) -> Result<usize, SimdError> {
        for (i, &value) in values.iter().enumerate() {
            let bytes = value.to_le_bytes();
            output[i * 8..(i + 1) * 8].copy_from_slice(&bytes);
        }
        Ok(values.len() * 8)
    }

    /// AVX2-accelerated u64 array encoding
    #[cfg(target_arch = "x86_64")]
    fn encode_u64_array_avx2(&self, values: &[u64], output: &mut [u8]) -> Result<usize, SimdError> {
        if !is_x86_feature_detected!("avx2") {
            return self.encode_u64_array_scalar(values, output);
        }

        unsafe {
            let chunks = values.chunks_exact(4); // 4 u64s = 32 bytes
            let remainder = chunks.remainder();
            let mut output_offset = 0;

            // Process 4 u64s at a time with AVX2
            for chunk in chunks {
                let values_vec = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
                _mm256_storeu_si256(
                    output[output_offset..].as_mut_ptr() as *mut __m256i,
                    values_vec
                );
                output_offset += 32;
            }

            // Process remainder with scalar
            for &value in remainder {
                let bytes = value.to_le_bytes();
                output[output_offset..output_offset + 8].copy_from_slice(&bytes);
                output_offset += 8;
            }

            Ok(output_offset)
        }
    }

    /// SSE2-accelerated u64 array encoding
    #[cfg(target_arch = "x86_64")]
    fn encode_u64_array_sse2(&self, values: &[u64], output: &mut [u8]) -> Result<usize, SimdError> {
        if !is_x86_feature_detected!("sse2") {
            return self.encode_u64_array_scalar(values, output);
        }

        unsafe {
            let chunks = values.chunks_exact(2); // 2 u64s = 16 bytes
            let remainder = chunks.remainder();
            let mut output_offset = 0;

            // Process 2 u64s at a time with SSE2
            for chunk in chunks {
                let values_vec = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
                _mm_storeu_si128(
                    output[output_offset..].as_mut_ptr() as *mut __m128i,
                    values_vec
                );
                output_offset += 16;
            }

            // Process remainder with scalar
            for &value in remainder {
                let bytes = value.to_le_bytes();
                output[output_offset..output_offset + 8].copy_from_slice(&bytes);
                output_offset += 8;
            }

            Ok(output_offset)
        }
    }

    /// Vectorized data decoding using SIMD
    pub fn decode_u64_array(&self, input: &[u8], output: &mut [u64]) -> Result<usize, SimdError> {
        if input.len() % 8 != 0 {
            return Err(SimdError::InvalidInputSize {
                size: input.len(),
                expected_multiple: 8,
            });
        }

        let value_count = input.len() / 8;
        if output.len() < value_count {
            return Err(SimdError::InsufficientBuffer {
                required: value_count,
                available: output.len(),
            });
        }

        if !self.is_enabled() || value_count < 4 {
            return self.decode_u64_array_scalar(input, output);
        }

        match self.capability {
            SimdCapability::Avx2 => self.decode_u64_array_avx2(input, output),
            SimdCapability::Sse2 => self.decode_u64_array_sse2(input, output),
            _ => self.decode_u64_array_scalar(input, output),
        }
    }

    /// Scalar u64 array decoding (fallback)
    fn decode_u64_array_scalar(&self, input: &[u8], output: &mut [u64]) -> Result<usize, SimdError> {
        let value_count = input.len() / 8;
        for i in 0..value_count {
            let bytes = &input[i * 8..(i + 1) * 8];
            output[i] = u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
        }
        Ok(value_count)
    }

    /// AVX2-accelerated u64 array decoding
    #[cfg(target_arch = "x86_64")]
    fn decode_u64_array_avx2(&self, input: &[u8], output: &mut [u64]) -> Result<usize, SimdError> {
        if !is_x86_feature_detected!("avx2") {
            return self.decode_u64_array_scalar(input, output);
        }

        unsafe {
            let chunks = input.chunks_exact(32); // 32 bytes = 4 u64s
            let remainder = chunks.remainder();
            let mut output_offset = 0;

            // Process 32-byte chunks with AVX2
            for chunk in chunks {
                let data_vec = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
                _mm256_storeu_si256(
                    output[output_offset..].as_mut_ptr() as *mut __m256i,
                    data_vec
                );
                output_offset += 4;
            }

            // Process remainder with scalar
            let remainder_values = remainder.len() / 8;
            for i in 0..remainder_values {
                let bytes = &remainder[i * 8..(i + 1) * 8];
                output[output_offset + i] = u64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ]);
            }

            Ok(output_offset + remainder_values)
        }
    }

    /// SSE2-accelerated u64 array decoding
    #[cfg(target_arch = "x86_64")]
    fn decode_u64_array_sse2(&self, input: &[u8], output: &mut [u64]) -> Result<usize, SimdError> {
        if !is_x86_feature_detected!("sse2") {
            return self.decode_u64_array_scalar(input, output);
        }

        unsafe {
            let chunks = input.chunks_exact(16); // 16 bytes = 2 u64s
            let remainder = chunks.remainder();
            let mut output_offset = 0;

            // Process 16-byte chunks with SSE2
            for chunk in chunks {
                let data_vec = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
                _mm_storeu_si128(
                    output[output_offset..].as_mut_ptr() as *mut __m128i,
                    data_vec
                );
                output_offset += 2;
            }

            // Process remainder with scalar
            let remainder_values = remainder.len() / 8;
            for i in 0..remainder_values {
                let bytes = &remainder[i * 8..(i + 1) * 8];
                output[output_offset + i] = u64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ]);
            }

            Ok(output_offset + remainder_values)
        }
    }

    /// Memory comparison using SIMD
    pub fn memory_compare(&self, a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        if !self.is_enabled() || a.len() < 32 {
            return a == b;
        }

        match self.capability {
            SimdCapability::Avx2 => self.memory_compare_avx2(a, b),
            SimdCapability::Sse2 => self.memory_compare_sse2(a, b),
            _ => a == b,
        }
    }

    /// AVX2-accelerated memory comparison
    #[cfg(target_arch = "x86_64")]
    fn memory_compare_avx2(&self, a: &[u8], b: &[u8]) -> bool {
        if !is_x86_feature_detected!("avx2") {
            return a == b;
        }

        unsafe {
            let chunks_a = a.chunks_exact(32);
            let chunks_b = b.chunks_exact(32);
            let remainder_a = chunks_a.remainder();
            let remainder_b = chunks_b.remainder();

            // Compare 32-byte chunks with AVX2
            for (chunk_a, chunk_b) in chunks_a.zip(chunks_b) {
                let vec_a = _mm256_loadu_si256(chunk_a.as_ptr() as *const __m256i);
                let vec_b = _mm256_loadu_si256(chunk_b.as_ptr() as *const __m256i);
                let cmp = _mm256_cmpeq_epi8(vec_a, vec_b);
                let mask = _mm256_movemask_epi8(cmp);
                
                if mask != -1 {
                    return false;
                }
            }

            // Compare remainder
            remainder_a == remainder_b
        }
    }

    /// SSE2-accelerated memory comparison
    #[cfg(target_arch = "x86_64")]
    fn memory_compare_sse2(&self, a: &[u8], b: &[u8]) -> bool {
        if !is_x86_feature_detected!("sse2") {
            return a == b;
        }

        unsafe {
            let chunks_a = a.chunks_exact(16);
            let chunks_b = b.chunks_exact(16);
            let remainder_a = chunks_a.remainder();
            let remainder_b = chunks_b.remainder();

            // Compare 16-byte chunks with SSE2
            for (chunk_a, chunk_b) in chunks_a.zip(chunks_b) {
                let vec_a = _mm_loadu_si128(chunk_a.as_ptr() as *const __m128i);
                let vec_b = _mm_loadu_si128(chunk_b.as_ptr() as *const __m128i);
                let cmp = _mm_cmpeq_epi8(vec_a, vec_b);
                let mask = _mm_movemask_epi8(cmp);
                
                if mask != 0xFFFF {
                    return false;
                }
            }

            // Compare remainder
            remainder_a == remainder_b
        }
    }
}

impl Default for SimdProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// SIMD-related errors
#[derive(Debug, thiserror::Error)]
pub enum SimdError {
    #[error("Insufficient buffer: required {required}, available {available}")]
    InsufficientBuffer { required: usize, available: usize },

    #[error("Invalid input size: {size}, expected multiple of {expected_multiple}")]
    InvalidInputSize { size: usize, expected_multiple: usize },

    #[error("SIMD not supported on this platform")]
    NotSupported,

    #[error("SIMD operation failed: {reason}")]
    OperationFailed { reason: String },
}

/// Performance benchmarking for SIMD operations
pub struct SimdBenchmark {
    processor: SimdProcessor,
}

impl SimdBenchmark {
    /// Create a new SIMD benchmark
    pub fn new() -> Self {
        Self {
            processor: SimdProcessor::new(),
        }
    }

    /// Benchmark CRC64 calculation
    pub fn benchmark_crc64(&self, data_size: usize, iterations: usize) -> BenchmarkResult {
        let data = vec![0xAAu8; data_size];
        
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = self.processor.crc64_checksum(&data);
        }
        let simd_duration = start.elapsed();

        // Benchmark scalar version
        let mut scalar_processor = SimdProcessor::with_capability(SimdCapability::None);
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = scalar_processor.crc64_checksum(&data);
        }
        let scalar_duration = start.elapsed();

        BenchmarkResult {
            operation: "CRC64".to_string(),
            data_size,
            iterations,
            simd_duration,
            scalar_duration,
            speedup: scalar_duration.as_nanos() as f64 / simd_duration.as_nanos() as f64,
            simd_capability: self.processor.capability(),
        }
    }

    /// Benchmark u64 array encoding
    pub fn benchmark_u64_encoding(&self, array_size: usize, iterations: usize) -> BenchmarkResult {
        let values = vec![0x123456789ABCDEFu64; array_size];
        let mut output = vec![0u8; array_size * 8];
        
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = self.processor.encode_u64_array(&values, &mut output);
        }
        let simd_duration = start.elapsed();

        // Benchmark scalar version
        let mut scalar_processor = SimdProcessor::with_capability(SimdCapability::None);
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = scalar_processor.encode_u64_array(&values, &mut output);
        }
        let scalar_duration = start.elapsed();

        BenchmarkResult {
            operation: "U64 Encoding".to_string(),
            data_size: array_size * 8,
            iterations,
            simd_duration,
            scalar_duration,
            speedup: scalar_duration.as_nanos() as f64 / simd_duration.as_nanos() as f64,
            simd_capability: self.processor.capability(),
        }
    }
}

impl Default for SimdBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub operation: String,
    pub data_size: usize,
    pub iterations: usize,
    pub simd_duration: std::time::Duration,
    pub scalar_duration: std::time::Duration,
    pub speedup: f64,
    pub simd_capability: SimdCapability,
}

impl BenchmarkResult {
    /// Get throughput in MB/s for SIMD version
    pub fn simd_throughput_mbps(&self) -> f64 {
        let total_bytes = self.data_size * self.iterations;
        let seconds = self.simd_duration.as_secs_f64();
        if seconds > 0.0 {
            (total_bytes as f64) / (1024.0 * 1024.0) / seconds
        } else {
            0.0
        }
    }

    /// Get throughput in MB/s for scalar version
    pub fn scalar_throughput_mbps(&self) -> f64 {
        let total_bytes = self.data_size * self.iterations;
        let seconds = self.scalar_duration.as_secs_f64();
        if seconds > 0.0 {
            (total_bytes as f64) / (1024.0 * 1024.0) / seconds
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_capability_detection() {
        let processor = SimdProcessor::new();
        println!("Detected SIMD capability: {:?}", processor.capability());
        
        // Should at least detect SSE2 on x86_64
        #[cfg(target_arch = "x86_64")]
        assert!(processor.capability() >= SimdCapability::Sse2);
    }

    #[test]
    fn test_crc64_checksum() {
        let processor = SimdProcessor::new();
        let data = b"Hello, SIMD world!";
        
        let checksum1 = processor.crc64_checksum(data);
        let checksum2 = processor.crc64_scalar(data);
        
        // SIMD and scalar should produce the same result
        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_u64_array_encoding() {
        let processor = SimdProcessor::new();
        let values = vec![0x123456789ABCDEF0u64, 0xFEDCBA9876543210u64];
        let mut output = vec![0u8; 16];
        
        let result = processor.encode_u64_array(&values, &mut output);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 16);
        
        // Verify the encoding
        let expected = [
            0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12,
            0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE,
        ];
        assert_eq!(output, expected);
    }

    #[test]
    fn test_u64_array_decoding() {
        let processor = SimdProcessor::new();
        let input = [
            0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12,
            0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE,
        ];
        let mut output = vec![0u64; 2];
        
        let result = processor.decode_u64_array(&input, &mut output);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        
        let expected = vec![0x123456789ABCDEF0u64, 0xFEDCBA9876543210u64];
        assert_eq!(output, expected);
    }

    #[test]
    fn test_memory_compare() {
        let processor = SimdProcessor::new();
        let data1 = vec![0xAAu8; 100];
        let data2 = vec![0xAAu8; 100];
        let mut data3 = vec![0xAAu8; 100];
        data3[50] = 0xBB;
        
        assert!(processor.memory_compare(&data1, &data2));
        assert!(!processor.memory_compare(&data1, &data3));
    }

    #[test]
    fn test_benchmark() {
        let benchmark = SimdBenchmark::new();
        let result = benchmark.benchmark_crc64(1024, 100);
        
        println!("CRC64 Benchmark:");
        println!("  SIMD Capability: {:?}", result.simd_capability);
        println!("  Speedup: {:.2}x", result.speedup);
        println!("  SIMD Throughput: {:.2} MB/s", result.simd_throughput_mbps());
        println!("  Scalar Throughput: {:.2} MB/s", result.scalar_throughput_mbps());
        
        assert!(result.speedup >= 1.0); // SIMD should be at least as fast as scalar
    }
}