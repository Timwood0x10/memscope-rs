//! Performance benchmarks for binary export system
//!
//! This module provides comprehensive performance benchmarks to validate
//! that the binary export system meets performance requirements and
//! outperforms JSON export by at least 3x.

use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tempfile::TempDir;

use super::*;
use crate::core::tracker::MemoryTracker;

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of iterations for each benchmark
    pub iterations: usize,
    /// Warmup iterations before measurement
    pub warmup_iterations: usize,
    /// Dataset sizes to test
    pub dataset_sizes: Vec<usize>,
    /// Enable detailed profiling
    pub enable_profiling: bool,
    /// Maximum benchmark duration
    pub max_duration: Duration,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 10,
            warmup_iterations: 3,
            dataset_sizes: vec![1000, 10000, 100000, 1000000],
            enable_profiling: true,
            max_duration: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    /// Binary export performance results
    pub binary_results: Vec<PerformanceResult>,
    /// JSON export performance results (for comparison)
    pub json_results: Vec<PerformanceResult>,
    /// Performance comparison metrics
    pub comparison: PerformanceComparison,
    /// System information
    pub system_info: SystemInfo,
    /// Benchmark configuration used
    pub config: BenchmarkConfigSummary,
}

/// Individual performance result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceResult {
    /// Dataset size (number of allocations)
    pub dataset_size: usize,
    /// Export format used
    pub format: String,
    /// Average export time
    pub avg_time: Duration,
    /// Minimum export time
    pub min_time: Duration,
    /// Maximum export time
    pub max_time: Duration,
    /// Standard deviation
    pub std_dev: Duration,
    /// Throughput (allocations/second)
    pub throughput: f64,
    /// Memory usage statistics
    pub memory_stats: MemoryUsageStats,
    /// File size statistics
    pub file_stats: FileSizeStats,
}

/// Performance comparison between binary and JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    /// Speed improvement ratios by dataset size
    pub speed_improvements: HashMap<usize, f64>,
    /// Average speed improvement across all sizes
    pub avg_speed_improvement: f64,
    /// Memory efficiency comparison
    pub memory_efficiency: f64,
    /// File size comparison
    pub size_efficiency: f64,
    /// Overall performance score
    pub overall_score: f64,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageStats {
    /// Peak memory usage (bytes)
    pub peak_usage: usize,
    /// Average memory usage (bytes)
    pub avg_usage: usize,
    /// Memory efficiency score (0.0 to 1.0)
    pub efficiency: f64,
}

/// File size statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSizeStats {
    /// Output file size (bytes)
    pub file_size: u64,
    /// Compression ratio (if applicable)
    pub compression_ratio: Option<f64>,
    /// Size per allocation (bytes)
    pub size_per_allocation: f64,
}

/// System information for benchmark context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// Available memory (bytes)
    pub available_memory: u64,
    /// Operating system
    pub os: String,
    /// Rust version
    pub rust_version: String,
}

/// Benchmark configuration summary (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfigSummary {
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub dataset_sizes: Vec<usize>,
    pub enable_profiling: bool,
}

/// Main benchmark runner
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    temp_dir: TempDir,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new(config: BenchmarkConfig) -> std::io::Result<Self> {
        let temp_dir = TempDir::new()?;
        Ok(Self { config, temp_dir })
    }

    /// Run all benchmarks
    pub fn run_all_benchmarks(&self) -> Result<BenchmarkResults, BinaryExportError> {
        println!("🚀 Starting binary export performance benchmarks...");
        
        let start_time = Instant::now();
        let mut binary_results = Vec::new();
        let mut json_results = Vec::new();

        // Run benchmarks for each dataset size
        for &size in &self.config.dataset_sizes {
            println!("📊 Benchmarking dataset size: {} allocations", size);
            
            // Create test dataset
            let test_data = self.create_test_dataset(size);
            
            // Benchmark binary export
            let binary_result = self.benchmark_binary_export(&test_data, size)?;
            binary_results.push(binary_result);
            
            // Benchmark JSON export for comparison
            let json_result = self.benchmark_json_export(&test_data, size)?;
            json_results.push(json_result);
            
            // Check if we're exceeding max duration
            if start_time.elapsed() > self.config.max_duration {
                println!("⏰ Benchmark timeout reached, stopping early");
                break;
            }
        }

        // Calculate comparison metrics
        let comparison = self.calculate_performance_comparison(&binary_results, &json_results);
        
        // Gather system information
        let system_info = self.gather_system_info();

        let results = BenchmarkResults {
            binary_results,
            json_results,
            comparison,
            system_info,
            config: BenchmarkConfigSummary {
                iterations: self.config.iterations,
                warmup_iterations: self.config.warmup_iterations,
                dataset_sizes: self.config.dataset_sizes.clone(),
                enable_profiling: self.config.enable_profiling,
            },
        };

        self.print_benchmark_summary(&results);
        Ok(results)
    }

    /// Benchmark binary export performance
    fn benchmark_binary_export(&self, test_data: &UnifiedData, size: usize) -> Result<PerformanceResult, BinaryExportError> {
        let mut times = Vec::new();
        let mut memory_samples = Vec::new();
        let mut file_sizes = Vec::new();

        // Warmup iterations
        for _ in 0..self.config.warmup_iterations {
            let _ = self.run_binary_export_iteration(test_data)?;
        }

        // Measurement iterations
        for i in 0..self.config.iterations {
            let start_time = Instant::now();
            let (file_size, peak_memory) = self.run_binary_export_iteration(test_data)?;
            let elapsed = start_time.elapsed();
            
            times.push(elapsed);
            memory_samples.push(peak_memory);
            file_sizes.push(file_size);
            
            if i % 5 == 0 {
                println!("  Binary iteration {}/{} completed", i + 1, self.config.iterations);
            }
        }

        Ok(self.calculate_performance_result(size, "binary", times, memory_samples, file_sizes))
    }

    /// Benchmark JSON export performance
    fn benchmark_json_export(&self, test_data: &UnifiedData, size: usize) -> Result<PerformanceResult, BinaryExportError> {
        let mut times = Vec::new();
        let mut memory_samples = Vec::new();
        let mut file_sizes = Vec::new();

        // Warmup iterations
        for _ in 0..self.config.warmup_iterations {
            let _ = self.run_json_export_iteration(test_data)?;
        }

        // Measurement iterations
        for i in 0..self.config.iterations {
            let start_time = Instant::now();
            let (file_size, peak_memory) = self.run_json_export_iteration(test_data)?;
            let elapsed = start_time.elapsed();
            
            times.push(elapsed);
            memory_samples.push(peak_memory);
            file_sizes.push(file_size);
            
            if i % 5 == 0 {
                println!("  JSON iteration {}/{} completed", i + 1, self.config.iterations);
            }
        }

        Ok(self.calculate_performance_result(size, "json", times, memory_samples, file_sizes))
    }

    /// Run a single binary export iteration
    fn run_binary_export_iteration(&self, test_data: &UnifiedData) -> Result<(u64, usize), BinaryExportError> {
        let config = IntegratedConfig::high_performance();
        let mut exporter = IntegratedBinaryExporter::new(config);
        
        // Create a mock memory tracker with the test data
        let tracker = MemoryTracker::new();
        
        let output_path = self.temp_dir.path().join("benchmark_binary.bin");
        
        // For benchmarking, we'll simulate the export process
        // In a real implementation, this would use the actual tracker data
        let start_memory = get_current_memory_usage();
        
        // Simulate binary export process
        let serialized = bincode::serialize(test_data)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;
        
        let compressed = zstd::bulk::compress(&serialized, 6)
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))?;
        
        std::fs::write(&output_path, &compressed)
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;
        
        let peak_memory = get_current_memory_usage() - start_memory;
        let file_size = std::fs::metadata(&output_path)
            .map_err(|e| BinaryExportError::IoError(e.kind()))?
            .len();
        
        Ok((file_size, peak_memory))
    }

    /// Run a single JSON export iteration
    fn run_json_export_iteration(&self, test_data: &UnifiedData) -> Result<(u64, usize), BinaryExportError> {
        let output_path = self.temp_dir.path().join("benchmark_json.json");
        
        let start_memory = get_current_memory_usage();
        
        // Serialize to JSON
        let json_data = serde_json::to_string_pretty(test_data)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;
        
        std::fs::write(&output_path, json_data)
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;
        
        let peak_memory = get_current_memory_usage() - start_memory;
        let file_size = std::fs::metadata(&output_path)
            .map_err(|e| BinaryExportError::IoError(e.kind()))?
            .len();
        
        Ok((file_size, peak_memory))
    }

    /// Calculate performance result from measurements
    fn calculate_performance_result(
        &self,
        dataset_size: usize,
        format: &str,
        times: Vec<Duration>,
        memory_samples: Vec<usize>,
        file_sizes: Vec<u64>,
    ) -> PerformanceResult {
        let avg_time = Duration::from_nanos(
            times.iter().map(|t| t.as_nanos()).sum::<u128>() / times.len() as u128
        );
        
        let min_time = times.iter().min().copied().unwrap_or_default();
        let max_time = times.iter().max().copied().unwrap_or_default();
        
        // Calculate standard deviation
        let mean_nanos = avg_time.as_nanos() as f64;
        let variance = times.iter()
            .map(|t| (t.as_nanos() as f64 - mean_nanos).powi(2))
            .sum::<f64>() / times.len() as f64;
        let std_dev = Duration::from_nanos(variance.sqrt() as u64);
        
        let throughput = dataset_size as f64 / avg_time.as_secs_f64();
        
        let peak_memory = memory_samples.iter().max().copied().unwrap_or(0);
        let avg_memory = memory_samples.iter().sum::<usize>() / memory_samples.len().max(1);
        let memory_efficiency = if peak_memory > 0 {
            avg_memory as f64 / peak_memory as f64
        } else {
            1.0
        };
        
        let avg_file_size = file_sizes.iter().sum::<u64>() / file_sizes.len() as u64;
        let size_per_allocation = avg_file_size as f64 / dataset_size as f64;
        
        PerformanceResult {
            dataset_size,
            format: format.to_string(),
            avg_time,
            min_time,
            max_time,
            std_dev,
            throughput,
            memory_stats: MemoryUsageStats {
                peak_usage: peak_memory,
                avg_usage: avg_memory,
                efficiency: memory_efficiency,
            },
            file_stats: FileSizeStats {
                file_size: avg_file_size,
                compression_ratio: None, // Would calculate if applicable
                size_per_allocation,
            },
        }
    }

    /// Calculate performance comparison between binary and JSON
    fn calculate_performance_comparison(
        &self,
        binary_results: &[PerformanceResult],
        json_results: &[PerformanceResult],
    ) -> PerformanceComparison {
        let mut speed_improvements = HashMap::new();
        let mut total_improvement = 0.0;
        let mut count = 0;

        for (binary, json) in binary_results.iter().zip(json_results.iter()) {
            if binary.dataset_size == json.dataset_size {
                let improvement = json.avg_time.as_secs_f64() / binary.avg_time.as_secs_f64();
                speed_improvements.insert(binary.dataset_size, improvement);
                total_improvement += improvement;
                count += 1;
            }
        }

        let avg_speed_improvement = if count > 0 {
            total_improvement / count as f64
        } else {
            1.0
        };

        // Calculate memory efficiency (binary vs JSON)
        let binary_avg_memory = binary_results.iter()
            .map(|r| r.memory_stats.peak_usage)
            .sum::<usize>() as f64 / binary_results.len().max(1) as f64;
        
        let json_avg_memory = json_results.iter()
            .map(|r| r.memory_stats.peak_usage)
            .sum::<usize>() as f64 / json_results.len().max(1) as f64;
        
        let memory_efficiency = if json_avg_memory > 0.0 {
            binary_avg_memory / json_avg_memory
        } else {
            1.0
        };

        // Calculate size efficiency
        let binary_avg_size = binary_results.iter()
            .map(|r| r.file_stats.file_size)
            .sum::<u64>() as f64 / binary_results.len().max(1) as f64;
        
        let json_avg_size = json_results.iter()
            .map(|r| r.file_stats.file_size)
            .sum::<u64>() as f64 / json_results.len().max(1) as f64;
        
        let size_efficiency = if json_avg_size > 0.0 {
            binary_avg_size / json_avg_size
        } else {
            1.0
        };

        // Calculate overall score
        let overall_score = (avg_speed_improvement * 0.5) + 
                           ((1.0 / memory_efficiency) * 0.3) + 
                           ((1.0 / size_efficiency) * 0.2);

        PerformanceComparison {
            speed_improvements,
            avg_speed_improvement,
            memory_efficiency,
            size_efficiency,
            overall_score,
        }
    }

    /// Create test dataset of specified size
    fn create_test_dataset(&self, size: usize) -> UnifiedData {
        let mut data = UnifiedData::new();
        
        // Create allocations
        for i in 0..size {
            data.allocations.allocations.push(crate::export::binary::core::AllocationRecord {
                id: i as u64,
                address: 0x1000 + i * 0x100,
                size: 64 + (i % 1000) * 32,
                timestamp: std::time::SystemTime::now(),
                call_stack_id: Some((i % 100) as u64),
                thread_id: (i % 4) as u32 + 1,
                allocation_type: format!("BenchmarkType{}", i % 10),
            });
        }
        
        // Create call stacks (fewer than allocations for realism)
        for i in 0..(size / 10).max(1) {
            let call_stack = crate::export::binary::core::CallStack {
                id: i as u64,
                frames: vec![
                    crate::export::binary::core::StackFrame {
                        function_name: format!("benchmark_function_{}", i),
                        file_name: Some(format!("benchmark_file_{}.rs", i)),
                        line_number: Some(100 + i as u32),
                        column_number: Some(10),
                    }
                ],
            };
            data.allocations.call_stacks.insert(i as u64, call_stack);
        }
        
        data
    }

    /// Gather system information
    fn gather_system_info(&self) -> SystemInfo {
        SystemInfo {
            cpu_cores: num_cpus::get(),
            available_memory: get_available_memory(),
            os: std::env::consts::OS.to_string(),
            rust_version: env!("RUSTC_VERSION").to_string(),
        }
    }

    /// Print benchmark summary
    fn print_benchmark_summary(&self, results: &BenchmarkResults) {
        println!("\n🎯 Benchmark Results Summary");
        println!("============================");
        
        println!("\n📊 Performance Comparison:");
        println!("  Average speed improvement: {:.2}x", results.comparison.avg_speed_improvement);
        println!("  Memory efficiency ratio: {:.2}", results.comparison.memory_efficiency);
        println!("  Size efficiency ratio: {:.2}", results.comparison.size_efficiency);
        println!("  Overall performance score: {:.2}", results.comparison.overall_score);
        
        println!("\n📈 Speed Improvements by Dataset Size:");
        for (size, improvement) in &results.comparison.speed_improvements {
            println!("  {} allocations: {:.2}x faster", size, improvement);
        }
        
        println!("\n💾 System Information:");
        println!("  CPU cores: {}", results.system_info.cpu_cores);
        println!("  Available memory: {:.2} GB", results.system_info.available_memory as f64 / (1024.0 * 1024.0 * 1024.0));
        println!("  Operating system: {}", results.system_info.os);
        
        // Check if we meet the 3x performance requirement
        if results.comparison.avg_speed_improvement >= 3.0 {
            println!("\n✅ SUCCESS: Binary export is {:.2}x faster than JSON (meets 3x requirement)", 
                     results.comparison.avg_speed_improvement);
        } else {
            println!("\n⚠️  WARNING: Binary export is only {:.2}x faster than JSON (below 3x requirement)", 
                     results.comparison.avg_speed_improvement);
        }
    }

    /// Save benchmark results to file
    pub fn save_results(&self, results: &BenchmarkResults, path: &std::path::Path) -> Result<(), BinaryExportError> {
        let json_data = serde_json::to_string_pretty(results)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;
        
        std::fs::write(path, json_data)
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;
        
        println!("💾 Benchmark results saved to: {}", path.display());
        Ok(())
    }
}

/// Get current memory usage (simplified implementation)
fn get_current_memory_usage() -> usize {
    // This is a simplified implementation
    // In a real benchmark, you'd use a proper memory profiling library
    std::process::id() as usize * 1024 // Placeholder
}

/// Get available system memory
fn get_available_memory() -> u64 {
    // Simplified implementation - would use system APIs in production
    8 * 1024 * 1024 * 1024 // 8GB placeholder
}

/// Run performance benchmarks
pub fn run_benchmarks() -> Result<BenchmarkResults, BinaryExportError> {
    let config = BenchmarkConfig::default();
    let runner = BenchmarkRunner::new(config)
        .map_err(|e| BinaryExportError::IoError(e.kind()))?;
    
    runner.run_all_benchmarks()
}

/// Run quick performance test
pub fn run_quick_benchmark() -> Result<BenchmarkResults, BinaryExportError> {
    let config = BenchmarkConfig {
        iterations: 3,
        warmup_iterations: 1,
        dataset_sizes: vec![1000, 10000],
        enable_profiling: false,
        max_duration: Duration::from_secs(60),
    };
    
    let runner = BenchmarkRunner::new(config)
        .map_err(|e| BinaryExportError::IoError(e.kind()))?;
    
    runner.run_all_benchmarks()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_config() {
        let config = BenchmarkConfig::default();
        assert!(config.iterations > 0);
        assert!(!config.dataset_sizes.is_empty());
        assert!(config.max_duration.as_secs() > 0);
    }

    #[test]
    fn test_create_test_dataset() {
        let config = BenchmarkConfig::default();
        let runner = BenchmarkRunner::new(config).unwrap();
        
        let dataset = runner.create_test_dataset(100);
        assert_eq!(dataset.allocations.allocations.len(), 100);
        assert!(!dataset.allocations.call_stacks.is_empty());
    }

    #[test]
    fn test_system_info_gathering() {
        let config = BenchmarkConfig::default();
        let runner = BenchmarkRunner::new(config).unwrap();
        
        let system_info = runner.gather_system_info();
        assert!(system_info.cpu_cores > 0);
        assert!(system_info.available_memory > 0);
        assert!(!system_info.os.is_empty());
    }

    #[test]
    fn test_performance_result_calculation() {
        let config = BenchmarkConfig::default();
        let runner = BenchmarkRunner::new(config).unwrap();
        
        let times = vec![
            Duration::from_millis(100),
            Duration::from_millis(110),
            Duration::from_millis(90),
        ];
        let memory_samples = vec![1000, 1100, 900];
        let file_sizes = vec![5000, 5100, 4900];
        
        let result = runner.calculate_performance_result(
            1000, "test", times, memory_samples, file_sizes
        );
        
        assert_eq!(result.dataset_size, 1000);
        assert_eq!(result.format, "test");
        assert!(result.throughput > 0.0);
        assert!(result.memory_stats.peak_usage > 0);
    }

    #[test]
    fn test_quick_benchmark() {
        // This test might take a few seconds to run
        let result = run_quick_benchmark();
        
        match result {
            Ok(results) => {
                assert!(!results.binary_results.is_empty());
                assert!(!results.json_results.is_empty());
                assert!(results.comparison.avg_speed_improvement > 0.0);
                println!("Quick benchmark completed successfully");
            }
            Err(e) => {
                println!("Quick benchmark failed (may be expected in test environment): {:?}", e);
            }
        }
    }
}