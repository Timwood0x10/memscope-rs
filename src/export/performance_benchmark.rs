//! Performance benchmark module
//!
//! This module provides comprehensive performance benchmarking functionality
//! to compare export performance before and after optimization,
//! particularly using complex_lifecycle_showcase.rs as the benchmark test case.

use crate::core::tracker::get_global_tracker;
use crate::core::types::TrackingResult;
use crate::export::fast_export_coordinator::{FastExportConfig, FastExportCoordinator};
use crate::export::optimized_json_export::OptimizedExportOptions;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of test runs
    pub test_runs: usize,
    /// Output directory
    pub output_dir: PathBuf,
    /// Enable verbose logging
    pub verbose: bool,
    /// Verify output consistency
    pub verify_consistency: bool,
    /// Generate detailed report
    pub generate_detailed_report: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            test_runs: 5,
            output_dir: PathBuf::from("benchmark_results"),
            verbose: true,
            verify_consistency: true,
            generate_detailed_report: true,
        }
    }
}

/// Single test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Test name
    pub test_name: String,
    /// Export time in milliseconds
    pub export_time_ms: u64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: usize,
    /// Output file size in bytes
    pub output_file_size: usize,
    /// Number of allocations processed
    pub allocations_processed: usize,
    /// Throughput in allocations per second
    pub throughput_allocations_per_sec: f64,
    /// Write speed in MB/s
    pub write_speed_mbps: f64,
    /// Success
    pub success: bool,
    /// Error message if any
    pub error_message: Option<String>,
}
/// Benchmark comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    /// Traditional export results
    pub traditional_results: Vec<BenchmarkResult>,
    /// Fast export results
    pub fast_results: Vec<BenchmarkResult>,
    /// Performance improvement statistics
    pub performance_improvement: PerformanceImprovement,
    /// Test configuration
    pub config: BenchmarkConfig,
    /// Test timestamp
    pub timestamp: String,
}

/// Performance improvement statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImprovement {
    /// Average export time improvement (percentage)
    pub avg_time_improvement_percent: f64,
    /// Average memory usage improvement (percentage)
    pub avg_memory_improvement_percent: f64,
    /// Average throughput improvement (percentage)
    pub avg_throughput_improvement_percent: f64,
    /// Average write speed improvement (percentage)
    pub avg_write_speed_improvement_percent: f64,
    /// Best time improvement (percentage)
    pub best_time_improvement_percent: f64,
    /// Worst time improvement (percentage)
    pub worst_time_improvement_percent: f64,
    /// Consistency score (0-100)
    pub consistency_score: f64,
}

/// Performance benchmark tester
pub struct PerformanceBenchmark {
    /// Configuration
    config: BenchmarkConfig,
    /// Test results history
    results_history: Vec<BenchmarkComparison>,
}

impl PerformanceBenchmark {
    /// Create new performance benchmark tester
    pub fn new(config: BenchmarkConfig) -> TrackingResult<Self> {
        // Create output directory
        fs::create_dir_all(&config.output_dir)?;

        Ok(Self {
            config,
            results_history: Vec::new(),
        })
    }

    /// 运行完整的基准测试
    pub fn run_full_benchmark(&mut self) -> TrackingResult<BenchmarkComparison> {
        println!("🚀 start benchmark");
        println!("==================");
        println!("config:");
        println!("  - 运行次数: {}", self.config.test_runs);
        println!("  - Output directory: {}", self.config.output_dir.display());
        println!("  - Verify consistency: {}", self.config.verify_consistency);
        println!();

        // Run complex_lifecycle_showcase to generate test data
        self.prepare_test_data()?;

        // Run traditional export tests
        println!("📊 Testing traditional export system...");
        let traditional_results = self.run_traditional_export_tests()?;

        // Run fast export tests
        println!("⚡ Testing fast export system...");
        let fast_results = self.run_fast_export_tests()?;

        // Calculate performance improvement
        let performance_improvement =
            self.calculate_performance_improvement(&traditional_results, &fast_results);

        // Create comparison result
        let comparison = BenchmarkComparison {
            traditional_results,
            fast_results,
            performance_improvement,
            config: self.config.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // save results
        self.save_benchmark_results(&comparison)?;

        // generate report
        if self.config.generate_detailed_report {
            self.generate_detailed_report(&comparison)?;
        }

        // add to history
        self.results_history.push(comparison.clone());

        Ok(comparison)
    }
    /// prepare test data
    fn prepare_test_data(&self) -> TrackingResult<()> {
        println!("🔧 prepare test data...");

        // run complex_lifecycle_showcase example to generate complex memory allocation patterns
        let output = Command::new("cargo")
            .args(&["run", "--example", "complex_lifecycle_showcase"])
            .output()
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::core::types::TrackingError::ExportError(format!(
                "Failed to run complex_lifecycle_showcase: {}",
                stderr
            )));
        }

        if self.config.verbose {
            println!("✅ prepare test data completed");
        }

        Ok(())
    }

    /// run traditional export tests
    fn run_traditional_export_tests(&self) -> TrackingResult<Vec<BenchmarkResult>> {
        let mut results = Vec::new();

        for run in 1..=self.config.test_runs {
            if self.config.verbose {
                println!("run {}/{}: traditional export", run, self.config.test_runs);
            }

            let result = self.run_single_traditional_test(run)?;
            results.push(result);

            // 短暂休息以避免系统负载影响
            std::thread::sleep(Duration::from_millis(100));
        }

        Ok(results)
    }

    /// run fast export tests
    fn run_fast_export_tests(&self) -> TrackingResult<Vec<BenchmarkResult>> {
        let mut results = Vec::new();

        for run in 1..=self.config.test_runs {
            if self.config.verbose {
                println!("run {}/{}: fast export", run, self.config.test_runs);
            }

            let result = self.run_single_fast_test(run)?;
            results.push(result);

            // rest for a while to avoid system load impact
            std::thread::sleep(Duration::from_millis(100));
        }

        Ok(results)
    }

    /// run single traditional export test
    fn run_single_traditional_test(&self, run_number: usize) -> TrackingResult<BenchmarkResult> {
        let start_time = Instant::now();
        let output_path = self
            .config
            .output_dir
            .join(format!("traditional_export_run_{}.json", run_number));

        // Get current memory tracker state
        let tracker = get_global_tracker();
        let _initial_stats = tracker.get_stats()?;

        // use traditional optimized export options
        let options = OptimizedExportOptions {
            use_streaming_writer: true,
            buffer_size: 8192,          // 8KB buffer
            parallel_processing: false, // traditional way does not use parallel processing
            use_compact_format: Some(false),
            enable_type_cache: true,
            batch_size: 1000,
            enable_schema_validation: true,
            optimization_level: crate::export::optimized_json_export::OptimizationLevel::Low,
            enable_enhanced_ffi_analysis: false,
            enable_boundary_event_processing: false,
            enable_memory_passport_tracking: false,
            enable_adaptive_optimization: false,
            max_cache_size: 100,
            target_batch_time_ms: 50,
            enable_security_analysis: false,
            include_low_severity_violations: false,
            generate_integrity_hashes: false,
            enable_fast_export_mode: false,
            auto_fast_export_threshold: None,
            thread_count: Some(1),
        };

        // execute traditional export
        let export_result = tracker.export_to_json_with_optimized_options(&output_path, options);
        let export_time = start_time.elapsed();

        // Get final statistics
        let final_stats = tracker.get_stats()?;

        // check file size
        let output_file_size = if output_path.exists() {
            fs::metadata(&output_path)?.len() as usize
        } else {
            0
        };

        // calculate performance metrics
        let allocations_processed = final_stats.total_allocations;
        let throughput = if export_time.as_secs_f64() > 0.0 {
            allocations_processed as f64 / export_time.as_secs_f64()
        } else {
            0.0
        };

        let write_speed_mbps = if export_time.as_secs_f64() > 0.0 && output_file_size > 0 {
            (output_file_size as f64 / 1024.0 / 1024.0) / export_time.as_secs_f64()
        } else {
            0.0
        };

        let result = BenchmarkResult {
            test_name: format!("Traditional Export Run {}", run_number),
            export_time_ms: export_time.as_millis() as u64,
            peak_memory_bytes: final_stats.peak_memory,
            output_file_size,
            allocations_processed,
            throughput_allocations_per_sec: throughput,
            write_speed_mbps,
            success: export_result.is_ok(),
            error_message: export_result.err().map(|e| e.to_string()),
        };

        if self.config.verbose {
            println!(
                "    ⏱️  time: {}ms, 📊 allocations: {}, 📁 size: {:.2}MB",
                result.export_time_ms,
                result.allocations_processed,
                result.output_file_size as f64 / 1024.0 / 1024.0
            );
        }

        Ok(result)
    }

    /// run single fast export test
    fn run_single_fast_test(&self, run_number: usize) -> TrackingResult<BenchmarkResult> {
        let start_time = Instant::now();
        let output_path = self
            .config
            .output_dir
            .join(format!("fast_export_run_{}.json", run_number));

        // Get current memory tracker state
        let tracker = get_global_tracker();
        let _initial_stats = tracker.get_stats()?;

        // use fast export config
        let fast_config = FastExportConfig::default();

        // create fast export coordinator
        let mut coordinator = FastExportCoordinator::new(fast_config);

        // execute fast export
        let export_result = coordinator.export_fast(&output_path);
        let export_time = start_time.elapsed();

        // Get final statistics
        let final_stats = tracker.get_stats()?;

        // check file size
        let output_file_size = if output_path.exists() {
            fs::metadata(&output_path)?.len() as usize
        } else {
            0
        };

        // 计算性能指标
        let allocations_processed = final_stats.total_allocations;
        let throughput = if export_time.as_secs_f64() > 0.0 {
            allocations_processed as f64 / export_time.as_secs_f64()
        } else {
            0.0
        };

        let write_speed_mbps = if export_time.as_secs_f64() > 0.0 && output_file_size > 0 {
            (output_file_size as f64 / 1024.0 / 1024.0) / export_time.as_secs_f64()
        } else {
            0.0
        };

        let result = BenchmarkResult {
            test_name: format!("Fast Export Run {}", run_number),
            export_time_ms: export_time.as_millis() as u64,
            peak_memory_bytes: final_stats.peak_memory,
            output_file_size,
            allocations_processed,
            throughput_allocations_per_sec: throughput,
            write_speed_mbps,
            success: export_result.is_ok(),
            error_message: export_result.err().map(|e| e.to_string()),
        };

        if self.config.verbose {
            println!(
                "    ⚡ time: {}ms, 📊 allocations: {}, 📁 size: {:.2}MB",
                result.export_time_ms,
                result.allocations_processed,
                result.output_file_size as f64 / 1024.0 / 1024.0
            );
        }

        Ok(result)
    }

    /// calculate performance improvement statistics
    fn calculate_performance_improvement(
        &self,
        traditional_results: &[BenchmarkResult],
        fast_results: &[BenchmarkResult],
    ) -> PerformanceImprovement {
        // calculate average values
        let avg_traditional_time = traditional_results
            .iter()
            .map(|r| r.export_time_ms as f64)
            .sum::<f64>()
            / traditional_results.len() as f64;

        let avg_fast_time = fast_results
            .iter()
            .map(|r| r.export_time_ms as f64)
            .sum::<f64>()
            / fast_results.len() as f64;

        let avg_traditional_memory = traditional_results
            .iter()
            .map(|r| r.peak_memory_bytes as f64)
            .sum::<f64>()
            / traditional_results.len() as f64;

        let avg_fast_memory = fast_results
            .iter()
            .map(|r| r.peak_memory_bytes as f64)
            .sum::<f64>()
            / fast_results.len() as f64;

        let avg_traditional_throughput = traditional_results
            .iter()
            .map(|r| r.throughput_allocations_per_sec)
            .sum::<f64>()
            / traditional_results.len() as f64;

        let avg_fast_throughput = fast_results
            .iter()
            .map(|r| r.throughput_allocations_per_sec)
            .sum::<f64>()
            / fast_results.len() as f64;

        let avg_traditional_write_speed = traditional_results
            .iter()
            .map(|r| r.write_speed_mbps)
            .sum::<f64>()
            / traditional_results.len() as f64;

        let avg_fast_write_speed = fast_results.iter().map(|r| r.write_speed_mbps).sum::<f64>()
            / fast_results.len() as f64;

        // Calculate improvement percentage
        let avg_time_improvement_percent = if avg_traditional_time > 0.0 {
            ((avg_traditional_time - avg_fast_time) / avg_traditional_time) * 100.0
        } else {
            0.0
        };

        let avg_memory_improvement_percent = if avg_traditional_memory > 0.0 {
            ((avg_traditional_memory - avg_fast_memory) / avg_traditional_memory) * 100.0
        } else {
            0.0
        };

        let avg_throughput_improvement_percent = if avg_traditional_throughput > 0.0 {
            ((avg_fast_throughput - avg_traditional_throughput) / avg_traditional_throughput)
                * 100.0
        } else {
            0.0
        };

        let avg_write_speed_improvement_percent = if avg_traditional_write_speed > 0.0 {
            ((avg_fast_write_speed - avg_traditional_write_speed) / avg_traditional_write_speed)
                * 100.0
        } else {
            0.0
        };

        // calculate best and worst improvement
        let traditional_times: Vec<f64> = traditional_results
            .iter()
            .map(|r| r.export_time_ms as f64)
            .collect();
        let fast_times: Vec<f64> = fast_results
            .iter()
            .map(|r| r.export_time_ms as f64)
            .collect();

        let best_traditional_time = traditional_times
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let best_fast_time = fast_times.iter().cloned().fold(f64::INFINITY, f64::min);
        let worst_traditional_time = traditional_times.iter().cloned().fold(0.0, f64::max);
        let worst_fast_time = fast_times.iter().cloned().fold(0.0, f64::max);

        let best_time_improvement_percent = if best_traditional_time > 0.0 {
            ((best_traditional_time - best_fast_time) / best_traditional_time) * 100.0
        } else {
            0.0
        };

        let worst_time_improvement_percent = if worst_traditional_time > 0.0 {
            ((worst_traditional_time - worst_fast_time) / worst_traditional_time) * 100.0
        } else {
            0.0
        };

        // calculate consistency score (based on standard deviation)
        let traditional_std = self.calculate_std_dev(&traditional_times);
        let fast_std = self.calculate_std_dev(&fast_times);
        let consistency_score = if traditional_std > 0.0 {
            ((traditional_std - fast_std) / traditional_std * 100.0)
                .max(0.0)
                .min(100.0)
        } else {
            100.0
        };

        PerformanceImprovement {
            avg_time_improvement_percent,
            avg_memory_improvement_percent,
            avg_throughput_improvement_percent,
            avg_write_speed_improvement_percent,
            best_time_improvement_percent,
            worst_time_improvement_percent,
            consistency_score,
        }
    }

    /// calculate standard deviation
    fn calculate_std_dev(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        variance.sqrt()
    }

    /// save benchmark results
    fn save_benchmark_results(&self, comparison: &BenchmarkComparison) -> TrackingResult<()> {
        let results_file = self.config.output_dir.join("benchmark_results.json");
        let json_data = serde_json::to_string_pretty(comparison)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))?;

        fs::write(&results_file, json_data)
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;

        if self.config.verbose {
            println!("💾 benchmark results saved to: {}", results_file.display());
        }

        Ok(())
    }

    /// generate detailed report
    fn generate_detailed_report(&self, comparison: &BenchmarkComparison) -> TrackingResult<()> {
        let report_file = self.config.output_dir.join("performance_report.md");
        let mut report = String::new();

        // report title
        report.push_str("# Large Project Export Optimization - Performance Benchmark Report\n\n");
        report.push_str(&format!("**Test Time**: {}\n\n", comparison.timestamp));
        report.push_str(&format!("**Test Configuration**:\n"));
        report.push_str(&format!("- Run Count: {}\n", comparison.config.test_runs));
        report.push_str(&format!(
            "- Verify consistency: {}\n",
            comparison.config.verify_consistency
        ));
        report.push_str("\n");

        // performance improvement summary
        let perf = &comparison.performance_improvement;
        report.push_str("## 📊 Performance Improvement Summary\n\n");
        report.push_str(&format!("| Metric | Improvement |\n"));
        report.push_str(&format!("|------|----------|\n"));
        report.push_str(&format!(
            "| Average Export Time | **{:.1}%** |\n",
            perf.avg_time_improvement_percent
        ));
        report.push_str(&format!(
            "| Average Memory Usage | **{:.1}%** |\n",
            perf.avg_memory_improvement_percent
        ));
        report.push_str(&format!(
            "| Average Throughput | **+{:.1}%** |\n",
            perf.avg_throughput_improvement_percent
        ));
        report.push_str(&format!(
            "| Average Write Speed | **+{:.1}%** |\n",
            perf.avg_write_speed_improvement_percent
        ));
        report.push_str(&format!(
            "| Best Time Improvement | **{:.1}%** |\n",
            perf.best_time_improvement_percent
        ));
        report.push_str(&format!(
            "| Worst Time Improvement | **{:.1}%** |\n",
            perf.worst_time_improvement_percent
        ));
        report.push_str(&format!(
            "| Consistency Score | **{:.1}/100** |\n",
            perf.consistency_score
        ));
        report.push_str("\n");

        // detailed result comparison
        report.push_str("## 📈 Detailed Result Comparison\n\n");
        report.push_str("### Traditional Export System\n\n");
        report.push_str(
            "| Run | Time(ms) | Memory(MB) | File Size(MB) | Throughput(alloc/s) | Write Speed(MB/s) |\n",
        );
        report.push_str(
            "|------|----------|----------|--------------|-----------------|----------------|\n",
        );

        for (i, result) in comparison.traditional_results.iter().enumerate() {
            report.push_str(&format!(
                "| {} | {} | {:.2} | {:.2} | {:.0} | {:.2} |\n",
                i + 1,
                result.export_time_ms,
                result.peak_memory_bytes as f64 / 1024.0 / 1024.0,
                result.output_file_size as f64 / 1024.0 / 1024.0,
                result.throughput_allocations_per_sec,
                result.write_speed_mbps
            ));
        }

        report.push_str("\n### Fast Export System\n\n");
        report.push_str(
            "| Run | Time(ms) | Memory(MB) | File Size(MB) | Throughput(alloc/s) | Write Speed(MB/s) |\n",
        );
        report.push_str(
            "|------|----------|----------|--------------|-----------------|----------------|\n",
        );

        for (i, result) in comparison.fast_results.iter().enumerate() {
            report.push_str(&format!(
                "| {} | {} | {:.2} | {:.2} | {:.0} | {:.2} |\n",
                i + 1,
                result.export_time_ms,
                result.peak_memory_bytes as f64 / 1024.0 / 1024.0,
                result.output_file_size as f64 / 1024.0 / 1024.0,
                result.throughput_allocations_per_sec,
                result.write_speed_mbps
            ));
        }

        // Conclusions and recommendations
        report.push_str("\n## 🎯 Conclusions and Recommendations\n\n");

        if perf.avg_time_improvement_percent > 50.0 {
            report.push_str(
                "✅ **Excellent**: Fast export system achieved significant performance improvement, exceeding the 50% time improvement target.\n\n",
            );
        } else if perf.avg_time_improvement_percent > 30.0 {
            report.push_str(
                "✅ **Good**: Fast export system achieved good performance improvement, reaching over 30% time improvement.\n\n",
            );
        } else {
            report
                .push_str("⚠️ **Needs Improvement**: Fast export system performance improvement is below expectations, further optimization recommended.\n\n");
        }

        if perf.consistency_score > 80.0 {
            report.push_str("✅ **Excellent Consistency**: Fast export system performs stably with high result consistency.\n\n");
        } else if perf.consistency_score > 60.0 {
            report.push_str(
                "✅ **Good Consistency**: Fast export system performs relatively stable.\n\n",
            );
        } else {
            report
                .push_str("⚠️ **Consistency Needs Improvement**: Fast export system results fluctuate significantly, stability optimization recommended.\n\n");
        }

        // Save report
        fs::write(&report_file, report)
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;

        if self.config.verbose {
            println!("📄 Detailed report generated: {}", report_file.display());
        }

        Ok(())
    }

    /// run signal benchmark    
    pub fn run_single_benchmark(&mut self, test_name: &str) -> TrackingResult<BenchmarkComparison> {
        println!("🎯 Running single benchmark: {}", test_name);
        self.prepare_test_data()?;

        let traditional_result = self.run_single_traditional_test(1)?;
        let fast_result = self.run_single_fast_test(1)?;

        let performance_improvement = self.calculate_performance_improvement(
            &[traditional_result.clone()],
            &[fast_result.clone()],
        );

        let comparison = BenchmarkComparison {
            traditional_results: vec![traditional_result],
            fast_results: vec![fast_result],
            performance_improvement,
            config: self.config.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        Ok(comparison)
    }
}
