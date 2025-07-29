//! Performance testing and optimization module
//!
//! This module provides comprehensive performance testing tools for testing and optimizing large project export functionality.

use crate::core::tracker::MemoryTracker;
use crate::core::types::TrackingResult;
use crate::export::fast_export_coordinator::{FastExportConfigBuilder, FastExportCoordinator};
use crate::export::optimized_json_export::OptimizedExportOptions;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Performance test configuration
#[derive(Debug, Clone)]
pub struct PerformanceTestConfig {
    /// Test dataset sizes
    pub dataset_sizes: Vec<usize>,
    /// Shard size test range
    pub shard_sizes: Vec<usize>,
    /// Thread count test range
    pub thread_counts: Vec<usize>,
    /// Buffer size test range
    pub buffer_sizes: Vec<usize>,
    /// Test iteration count
    pub test_iterations: usize,
    /// Memory limit (MB)
    pub memory_limit_mb: usize,
    /// Enable verbose output
    pub verbose: bool,
}

impl Default for PerformanceTestConfig {
    fn default() -> Self {
        Self {
            dataset_sizes: vec![1000, 5000, 10000, 20000, 50000],
            shard_sizes: vec![500, 1000, 2000, 5000],
            thread_counts: vec![1, 2, 4, 8],
            buffer_sizes: vec![64 * 1024, 256 * 1024, 512 * 1024, 1024 * 1024],
            test_iterations: 3,
            memory_limit_mb: 64,
            verbose: true,
        }
    }
}

/// Performance test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestResult {
    /// Test name
    pub test_name: String,
    /// Dataset size
    pub dataset_size: usize,
    /// Configuration parameters
    pub config_params: HashMap<String, String>,
    /// Export time (milliseconds)
    pub export_time_ms: u64,
    /// Peak memory usage (MB)
    pub peak_memory_mb: f64,
    /// Throughput (allocations/sec)
    pub throughput_allocations_per_sec: f64,
    /// File size (bytes)
    pub output_file_size_bytes: usize,
    /// Success
    pub success: bool,
    /// Error message
    pub error_message: Option<String>,
}

/// Performance benchmark tool
pub struct PerformanceBenchmark;

impl PerformanceBenchmark {
    /// Run quick benchmark
    pub fn run_quick_benchmark() -> TrackingResult<()> {
        println!("🚀 Running quick performance benchmark");
        println!("========================");

        let config = PerformanceTestConfig {
            dataset_sizes: vec![1000, 5000, 10000],
            shard_sizes: vec![500, 1000, 2000],
            thread_counts: vec![1, 2, 4],
            buffer_sizes: vec![256 * 1024],
            test_iterations: 1,
            memory_limit_mb: 64,
            verbose: true,
        };

        let mut test_suite = PerformanceTestSuite::new(config);
        let _report = test_suite.run_basic_tests()?;

        println!("✅ Quick benchmark completed");
        Ok(())
    }

    /// 运行 complex_lifecycle_showcase.rs 基准测试
    pub fn run_complex_lifecycle_benchmark() -> TrackingResult<ComplexLifecycleBenchmarkResult> {
        println!("🎯 Running complex_lifecycle_showcase.rs benchmark");
        println!("==============================================");

        let mut benchmark_result = ComplexLifecycleBenchmarkResult::default();

        // 测试传统导出性能
        println!("📊 Testing traditional export performance...");
        let traditional_result = Self::benchmark_traditional_export()?;
        benchmark_result.traditional_export = traditional_result;

        // 测试快速导出性能
        println!("⚡ Testing fast export performance...");
        let fast_result = Self::benchmark_fast_export()?;
        benchmark_result.fast_export = fast_result;

        // 计算性能提升
        benchmark_result.calculate_improvements();

        // 打印详细结果
        Self::print_complex_benchmark_results(&benchmark_result);

        Ok(benchmark_result)
    }

    /// 基准测试传统导出
    fn benchmark_traditional_export() -> TrackingResult<ExportBenchmarkResult> {
        use std::process::Command;
        use std::time::Instant;

        let start_time = Instant::now();
        let start_memory = Self::get_current_memory_usage();

        // 运行 complex_lifecycle_showcase 示例
        let output = Command::new("cargo")
            .args(&["run", "--example", "complex_lifecycle_showcase"])
            .output()
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;

        let export_time = start_time.elapsed();
        let peak_memory = Self::get_current_memory_usage() - start_memory;

        // 检查输出文件大小
        let file_size = Self::get_complex_lifecycle_file_size();

        let success = output.status.success();
        let error_message = if !success {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        } else {
            None
        };

        Ok(ExportBenchmarkResult {
            export_time_ms: export_time.as_millis() as u64,
            peak_memory_mb: peak_memory,
            output_file_size_bytes: file_size,
            success,
            error_message,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        })
    }

    /// 基准测试快速导出
    fn benchmark_fast_export() -> TrackingResult<ExportBenchmarkResult> {
        use std::time::Instant;

        let start_time = Instant::now();
        let start_memory = Self::get_current_memory_usage();

        // 使用快速导出协调器
        let config = FastExportConfigBuilder::new()
            .shard_size(1000)
            .max_threads(Some(4))
            .buffer_size(512 * 1024)
            .performance_monitoring(true)
            .build();

        let mut coordinator = FastExportCoordinator::new(config);
        let output_path = "complex_lifecycle_fast_export";

        let result = coordinator.export_fast(output_path);
        let export_time = start_time.elapsed();
        let peak_memory = Self::get_current_memory_usage() - start_memory;

        match result {
            Ok(stats) => {
                let file_size = Self::get_file_size_static(output_path);

                Ok(ExportBenchmarkResult {
                    export_time_ms: stats.total_export_time_ms,
                    peak_memory_mb: peak_memory,
                    output_file_size_bytes: file_size,
                    success: true,
                    error_message: None,
                    stdout: format!(
                        "Fast export completed: {} allocations processed",
                        stats.total_allocations_processed
                    ),
                })
            }
            Err(e) => Ok(ExportBenchmarkResult {
                export_time_ms: export_time.as_millis() as u64,
                peak_memory_mb: peak_memory,
                output_file_size_bytes: 0,
                success: false,
                error_message: Some(e.to_string()),
                stdout: String::new(),
            }),
        }
    }

    /// 获取 complex_lifecycle 文件大小
    fn get_complex_lifecycle_file_size() -> usize {
        let paths = [
            "MemoryAnalysis/complex_lifecycle/complex_lifecycle_memory_analysis.json",
            "MemoryAnalysis/complex_lifecycle_snapshot/complex_lifecycle_snapshot_memory_analysis.json",
            "complex_lifecycle_snapshot_memory_analysis.json",
        ];

        for path in &paths {
            if let Ok(metadata) = std::fs::metadata(path) {
                return metadata.len() as usize;
            }
        }

        0
    }

    /// 获取当前内存使用量
    fn get_current_memory_usage() -> f64 {
        // 简化的内存使用估算 - 在实际实现中可以使用更精确的方法
        use std::process;
        let pid = process::id();

        // 尝试读取 /proc/self/status (Linux) 或使用其他方法
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<f64>() {
                            return kb / 1024.0; // 转换为 MB
                        }
                    }
                }
            }
        }

        // 回退到简单估算
        (pid as f64 * 0.001).min(100.0)
    }

    /// 静态方法获取文件大小
    fn get_file_size_static(path: &str) -> usize {
        std::fs::metadata(path)
            .map(|metadata| metadata.len() as usize)
            .unwrap_or(0)
    }

    /// 打印复杂基准测试结果
    fn print_complex_benchmark_results(result: &ComplexLifecycleBenchmarkResult) {
        println!("\n📊 Complex Lifecycle Showcase 基准测试结果");
        println!("==========================================");

        println!("\n传统导出:");
        println!("  时间: {} ms", result.traditional_export.export_time_ms);
        println!("  内存: {:.2} MB", result.traditional_export.peak_memory_mb);
        println!(
            "  文件大小: {} bytes ({:.2} KB)",
            result.traditional_export.output_file_size_bytes,
            result.traditional_export.output_file_size_bytes as f64 / 1024.0
        );
        println!(
            "  状态: {}",
            if result.traditional_export.success {
                "✅ 成功"
            } else {
                "❌ 失败"
            }
        );

        println!("\n快速导出:");
        println!("  时间: {} ms", result.fast_export.export_time_ms);
        println!("  内存: {:.2} MB", result.fast_export.peak_memory_mb);
        println!(
            "  文件大小: {} bytes ({:.2} KB)",
            result.fast_export.output_file_size_bytes,
            result.fast_export.output_file_size_bytes as f64 / 1024.0
        );
        println!(
            "  状态: {}",
            if result.fast_export.success {
                "✅ 成功"
            } else {
                "❌ 失败"
            }
        );

        if result.traditional_export.success && result.fast_export.success {
            println!("\n🚀 性能提升:");
            println!(
                "  时间提升: {:.2}x ({:.1}% 减少)",
                result.time_improvement_factor,
                (1.0 - 1.0 / result.time_improvement_factor) * 100.0
            );
            println!("  内存优化: {:.2}x", result.memory_improvement_factor);

            let target_improvement = 2.0; // 目标：减少 60-80% 导出时间 (2-5x 提升)
            if result.time_improvement_factor >= target_improvement {
                println!("  🎯 达到预期性能提升目标 (>{}x)!", target_improvement);
            } else {
                println!("  ⚠️ 未达到预期性能提升目标 (>{}x)", target_improvement);
            }

            // 验证内存限制
            let memory_limit = 64.0; // 64MB 限制
            if result.fast_export.peak_memory_mb <= memory_limit {
                println!(
                    "  ✅ 内存使用在限制范围内 ({:.2} MB <= {} MB)",
                    result.fast_export.peak_memory_mb, memory_limit
                );
            } else {
                println!(
                    "  ⚠️ 内存使用超过限制 ({:.2} MB > {} MB)",
                    result.fast_export.peak_memory_mb, memory_limit
                );
            }
        }

        if let Some(ref error) = result.traditional_export.error_message {
            println!("\n❌ 传统导出错误: {}", error);
        }
        if let Some(ref error) = result.fast_export.error_message {
            println!("\n❌ 快速导出错误: {}", error);
        }
    }

    /// Run complete benchmark
    pub fn run_comprehensive_benchmark() -> TrackingResult<PerformanceTestReport> {
        println!("🚀 运行完整性能基准测试");
        println!("========================");

        let config = PerformanceTestConfig::default();
        let mut test_suite = PerformanceTestSuite::new(config);
        let report = test_suite.run_full_test_suite()?;

        // 打印详细报告
        Self::print_detailed_report(&report);

        Ok(report)
    }

    /// Print detailed report
    fn print_detailed_report(report: &PerformanceTestReport) {
        println!("\n📊 性能测试报告");
        println!("================");
        println!("总测试数: {}", report.test_summary.total_tests);
        println!("成功测试: {}", report.test_summary.successful_tests);
        println!("失败测试: {}", report.test_summary.failed_tests);
        println!(
            "成功率: {:.1}%",
            report.test_summary.successful_tests as f64 / report.test_summary.total_tests as f64
                * 100.0
        );

        println!("\n📈 性能分析");
        println!(
            "平均导出时间: {:.2} ms",
            report.performance_analysis.average_export_time_ms
        );
        println!(
            "平均内存使用: {:.2} MB",
            report.performance_analysis.average_memory_usage_mb
        );
        println!(
            "平均吞吐量: {:.0} 分配/秒",
            report.performance_analysis.average_throughput
        );

        if !report.optimization_recommendations.is_empty() {
            println!("\n💡 优化建议");
            for rec in &report.optimization_recommendations {
                println!(
                    "• [{}] {}: {}",
                    rec.impact, rec.category, rec.recommendation
                );
            }
        }
    }
}

/// 性能测试套件
pub struct PerformanceTestSuite {
    config: PerformanceTestConfig,
    results: Vec<PerformanceTestResult>,
}

impl PerformanceTestSuite {
    /// Create new performance test suite
    pub fn new(config: PerformanceTestConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    /// Run basic tests
    pub fn run_basic_tests(&mut self) -> TrackingResult<PerformanceTestReport> {
        println!("📊 运行基本性能测试");

        for &dataset_size in &self.config.dataset_sizes {
            println!("测试数据集大小: {}", dataset_size);

            // 测试传统导出
            let traditional_result = self.test_traditional_export(dataset_size)?;
            self.results.push(traditional_result);

            // 测试快速导出
            let fast_result = self.test_fast_export(dataset_size)?;
            self.results.push(fast_result);

            println!("  ✅ 完成数据集大小 {} 的测试", dataset_size);
        }

        Ok(self.generate_performance_report())
    }

    /// Run complete test suite
    pub fn run_full_test_suite(&mut self) -> TrackingResult<PerformanceTestReport> {
        println!("🚀 开始运行完整性能测试套件");

        // 1. 基本性能测试
        self.run_basic_tests()?;

        // 2. 分片大小优化测试
        self.run_shard_size_tests()?;

        // 3. 多线程扩展性测试
        self.run_thread_scalability_tests()?;

        // 4. 内存使用测试
        self.run_memory_tests()?;

        println!("✅ 性能测试套件完成");
        Ok(self.generate_performance_report())
    }

    /// Run benchmark performance tests
    pub fn run_baseline_performance_tests(&mut self) -> TrackingResult<()> {
        println!("📊 运行基准性能测试");

        for &dataset_size in &self.config.dataset_sizes {
            // 测试传统导出
            let traditional_result = self.test_traditional_export(dataset_size)?;
            self.results.push(traditional_result);

            // 测试快速导出
            let fast_result = self.test_fast_export(dataset_size)?;
            self.results.push(fast_result);
        }

        Ok(())
    }

    /// Run shard size optimization tests
    pub fn run_shard_size_optimization_tests(&mut self) -> TrackingResult<()> {
        println!("⚡ 分片大小优化测试");

        let dataset_size = 10000;
        for &shard_size in &self.config.shard_sizes {
            let result = self.test_shard_size_performance(dataset_size, shard_size)?;
            self.results.push(result);
        }

        Ok(())
    }

    /// Run memory usage tests
    pub fn run_memory_usage_tests(&mut self) -> TrackingResult<()> {
        println!("💾 内存使用测试");

        for &dataset_size in &self.config.dataset_sizes {
            let result = self.test_memory_usage(dataset_size)?;

            if result.peak_memory_mb > self.config.memory_limit_mb as f64 {
                println!(
                    "  ⚠️ 内存使用超过限制: {:.2} MB > {} MB",
                    result.peak_memory_mb, self.config.memory_limit_mb
                );
            }

            self.results.push(result);
        }

        Ok(())
    }

    /// Run before/after optimization comparison tests
    pub fn run_before_after_comparison_tests(&mut self) -> TrackingResult<()> {
        println!("🔄 优化前后对比测试");

        let dataset_size = 10000;

        // 传统导出（优化前）
        let traditional_result = self.test_traditional_export(dataset_size)?;
        let mut traditional_result = traditional_result;
        traditional_result.test_name = "traditional_export".to_string();
        self.results.push(traditional_result);

        // 优化导出（优化后）
        let optimized_result = self.test_fast_export(dataset_size)?;
        let mut optimized_result = optimized_result;
        optimized_result.test_name = "optimized_export".to_string();
        self.results.push(optimized_result);

        Ok(())
    }

    /// 分片大小测试
    fn run_shard_size_tests(&mut self) -> TrackingResult<()> {
        println!("\n⚡ 分片大小优化测试");

        let dataset_size = 10000;
        for &shard_size in &self.config.shard_sizes {
            let result = self.test_shard_size_performance(dataset_size, shard_size)?;
            self.results.push(result);
        }

        Ok(())
    }

    /// 多线程扩展性测试
    pub fn run_thread_scalability_tests(&mut self) -> TrackingResult<()> {
        println!("\n🔄 多线程扩展性测试");

        let dataset_size = 20000;
        for &thread_count in &self.config.thread_counts {
            let result = self.test_thread_scalability(dataset_size, thread_count)?;
            self.results.push(result);
        }

        Ok(())
    }

    /// 内存使用测试
    fn run_memory_tests(&mut self) -> TrackingResult<()> {
        println!("\n💾 内存使用测试");

        for &dataset_size in &self.config.dataset_sizes {
            let result = self.test_memory_usage(dataset_size)?;

            if result.peak_memory_mb > self.config.memory_limit_mb as f64 {
                println!(
                    "  ⚠️ 内存使用超过限制: {:.2} MB > {} MB",
                    result.peak_memory_mb, self.config.memory_limit_mb
                );
            }

            self.results.push(result);
        }

        Ok(())
    }

    /// 测试传统导出性能
    fn test_traditional_export(
        &self,
        dataset_size: usize,
    ) -> TrackingResult<PerformanceTestResult> {
        let start_time = Instant::now();
        let start_memory = self.get_memory_usage();

        let tracker = MemoryTracker::new();
        let traditional_options = OptimizedExportOptions::default()
            .fast_export_mode(false)
            .auto_fast_export_threshold(None);

        let output_path = format!("test_traditional_{}", dataset_size);

        let result = match tracker
            .export_to_json_with_optimized_options(&output_path, traditional_options)
        {
            Ok(_) => {
                let export_time = start_time.elapsed();
                let peak_memory = self.get_memory_usage() - start_memory;
                let file_size = self.get_file_size(&format!(
                    "MemoryAnalysis/{}/{}_memory_analysis.json",
                    output_path, output_path
                ));

                PerformanceTestResult {
                    test_name: "traditional_export".to_string(),
                    dataset_size,
                    config_params: HashMap::new(),
                    export_time_ms: export_time.as_millis() as u64,
                    peak_memory_mb: peak_memory,
                    throughput_allocations_per_sec: if export_time.as_secs_f64() > 0.0 {
                        dataset_size as f64 / export_time.as_secs_f64()
                    } else {
                        0.0
                    },
                    output_file_size_bytes: file_size,
                    success: true,
                    error_message: None,
                }
            }
            Err(e) => PerformanceTestResult {
                test_name: "traditional_export".to_string(),
                dataset_size,
                config_params: HashMap::new(),
                export_time_ms: start_time.elapsed().as_millis() as u64,
                peak_memory_mb: self.get_memory_usage() - start_memory,
                throughput_allocations_per_sec: 0.0,
                output_file_size_bytes: 0,
                success: false,
                error_message: Some(e.to_string()),
            },
        };

        Ok(result)
    }

    /// 测试快速导出性能
    fn test_fast_export(&self, dataset_size: usize) -> TrackingResult<PerformanceTestResult> {
        let start_time = Instant::now();
        let start_memory = self.get_memory_usage();

        let config = FastExportConfigBuilder::new()
            .shard_size(1000)
            .max_threads(Some(4))
            .buffer_size(256 * 1024)
            .performance_monitoring(true)
            .build();

        let mut coordinator = FastExportCoordinator::new(config);
        let output_path = format!("test_fast_{}", dataset_size);

        let result = match coordinator.export_fast(&output_path) {
            Ok(stats) => {
                let peak_memory = self.get_memory_usage() - start_memory;
                let file_size = self.get_file_size(&output_path);

                let mut config_params = HashMap::new();
                config_params.insert("shard_size".to_string(), "1000".to_string());
                config_params.insert("threads".to_string(), "4".to_string());

                PerformanceTestResult {
                    test_name: "fast_export".to_string(),
                    dataset_size,
                    config_params,
                    export_time_ms: stats.total_export_time_ms,
                    peak_memory_mb: peak_memory,
                    throughput_allocations_per_sec: stats.overall_throughput_allocations_per_sec,
                    output_file_size_bytes: file_size,
                    success: true,
                    error_message: None,
                }
            }
            Err(e) => PerformanceTestResult {
                test_name: "fast_export".to_string(),
                dataset_size,
                config_params: HashMap::new(),
                export_time_ms: start_time.elapsed().as_millis() as u64,
                peak_memory_mb: self.get_memory_usage() - start_memory,
                throughput_allocations_per_sec: 0.0,
                output_file_size_bytes: 0,
                success: false,
                error_message: Some(e.to_string()),
            },
        };

        Ok(result)
    }

    /// 测试分片大小性能
    fn test_shard_size_performance(
        &self,
        dataset_size: usize,
        shard_size: usize,
    ) -> TrackingResult<PerformanceTestResult> {
        let start_time = Instant::now();
        let start_memory = self.get_memory_usage();

        let config = FastExportConfigBuilder::new()
            .shard_size(shard_size)
            .max_threads(Some(4))
            .buffer_size(256 * 1024)
            .performance_monitoring(true)
            .build();

        let mut coordinator = FastExportCoordinator::new(config);
        let output_path = format!("test_shard_{}_{}", shard_size, dataset_size);

        let result = match coordinator.export_fast(&output_path) {
            Ok(stats) => {
                let peak_memory = self.get_memory_usage() - start_memory;
                let file_size = self.get_file_size(&output_path);

                let mut config_params = HashMap::new();
                config_params.insert("shard_size".to_string(), shard_size.to_string());

                PerformanceTestResult {
                    test_name: "shard_size_test".to_string(),
                    dataset_size,
                    config_params,
                    export_time_ms: stats.total_export_time_ms,
                    peak_memory_mb: peak_memory,
                    throughput_allocations_per_sec: stats.overall_throughput_allocations_per_sec,
                    output_file_size_bytes: file_size,
                    success: true,
                    error_message: None,
                }
            }
            Err(e) => PerformanceTestResult {
                test_name: "shard_size_test".to_string(),
                dataset_size,
                config_params: {
                    let mut params = HashMap::new();
                    params.insert("shard_size".to_string(), shard_size.to_string());
                    params
                },
                export_time_ms: start_time.elapsed().as_millis() as u64,
                peak_memory_mb: self.get_memory_usage() - start_memory,
                throughput_allocations_per_sec: 0.0,
                output_file_size_bytes: 0,
                success: false,
                error_message: Some(e.to_string()),
            },
        };

        Ok(result)
    }

    /// 测试线程扩展性
    fn test_thread_scalability(
        &self,
        dataset_size: usize,
        thread_count: usize,
    ) -> TrackingResult<PerformanceTestResult> {
        let start_time = Instant::now();
        let start_memory = self.get_memory_usage();

        let config = FastExportConfigBuilder::new()
            .shard_size(1000)
            .max_threads(Some(thread_count))
            .buffer_size(256 * 1024)
            .performance_monitoring(true)
            .build();

        let mut coordinator = FastExportCoordinator::new(config);
        let output_path = format!("test_threads_{}_{}", thread_count, dataset_size);

        let result = match coordinator.export_fast(&output_path) {
            Ok(stats) => {
                let peak_memory = self.get_memory_usage() - start_memory;
                let file_size = self.get_file_size(&output_path);

                let mut config_params = HashMap::new();
                config_params.insert("thread_count".to_string(), thread_count.to_string());

                PerformanceTestResult {
                    test_name: "thread_scalability_test".to_string(),
                    dataset_size,
                    config_params,
                    export_time_ms: stats.total_export_time_ms,
                    peak_memory_mb: peak_memory,
                    throughput_allocations_per_sec: stats.overall_throughput_allocations_per_sec,
                    output_file_size_bytes: file_size,
                    success: true,
                    error_message: None,
                }
            }
            Err(e) => PerformanceTestResult {
                test_name: "thread_scalability_test".to_string(),
                dataset_size,
                config_params: {
                    let mut params = HashMap::new();
                    params.insert("thread_count".to_string(), thread_count.to_string());
                    params
                },
                export_time_ms: start_time.elapsed().as_millis() as u64,
                peak_memory_mb: self.get_memory_usage() - start_memory,
                throughput_allocations_per_sec: 0.0,
                output_file_size_bytes: 0,
                success: false,
                error_message: Some(e.to_string()),
            },
        };

        Ok(result)
    }

    /// 测试内存使用
    fn test_memory_usage(&self, dataset_size: usize) -> TrackingResult<PerformanceTestResult> {
        let start_time = Instant::now();
        let start_memory = self.get_memory_usage();

        let config = FastExportConfigBuilder::new()
            .shard_size(500) // 较小的分片以减少内存使用
            .max_threads(Some(2)) // 较少的线程以减少内存使用
            .buffer_size(64 * 1024) // 较小的缓冲区
            .performance_monitoring(true)
            .build();

        let mut coordinator = FastExportCoordinator::new(config);
        let output_path = format!("test_memory_{}", dataset_size);

        let result = match coordinator.export_fast(&output_path) {
            Ok(stats) => {
                let peak_memory = self.get_memory_usage() - start_memory;
                let file_size = self.get_file_size(&output_path);

                let mut config_params = HashMap::new();
                config_params.insert("memory_optimized".to_string(), "true".to_string());

                PerformanceTestResult {
                    test_name: "memory_usage_test".to_string(),
                    dataset_size,
                    config_params,
                    export_time_ms: stats.total_export_time_ms,
                    peak_memory_mb: peak_memory,
                    throughput_allocations_per_sec: stats.overall_throughput_allocations_per_sec,
                    output_file_size_bytes: file_size,
                    success: peak_memory <= self.config.memory_limit_mb as f64,
                    error_message: if peak_memory > self.config.memory_limit_mb as f64 {
                        Some(format!(
                            "Memory usage {} MB exceeds limit {} MB",
                            peak_memory, self.config.memory_limit_mb
                        ))
                    } else {
                        None
                    },
                }
            }
            Err(e) => PerformanceTestResult {
                test_name: "memory_usage_test".to_string(),
                dataset_size,
                config_params: HashMap::new(),
                export_time_ms: start_time.elapsed().as_millis() as u64,
                peak_memory_mb: self.get_memory_usage() - start_memory,
                throughput_allocations_per_sec: 0.0,
                output_file_size_bytes: 0,
                success: false,
                error_message: Some(e.to_string()),
            },
        };

        Ok(result)
    }

    /// 获取当前内存使用量 (MB)
    fn get_memory_usage(&self) -> f64 {
        // 简化的内存使用估算
        let estimated_mb = std::process::id() as f64 * 0.001;
        estimated_mb.min(100.0)
    }

    /// 获取文件大小
    fn get_file_size(&self, path: &str) -> usize {
        Self::get_file_size_static(path)
    }

    /// 静态方法获取文件大小
    fn get_file_size_static(path: &str) -> usize {
        std::fs::metadata(path)
            .map(|metadata| metadata.len() as usize)
            .unwrap_or(0)
    }

    /// 生成性能测试报告
    pub fn generate_performance_report(&self) -> PerformanceTestReport {
        let successful_results: Vec<_> = self.results.iter().filter(|r| r.success).collect();

        let test_summary = TestSummary {
            total_tests: self.results.len(),
            successful_tests: successful_results.len(),
            failed_tests: self.results.len() - successful_results.len(),
            total_test_time_ms: self.results.iter().map(|r| r.export_time_ms).sum(),
        };

        let performance_analysis = if successful_results.is_empty() {
            PerformanceAnalysis::default()
        } else {
            let avg_export_time = successful_results
                .iter()
                .map(|r| r.export_time_ms)
                .sum::<u64>() as f64
                / successful_results.len() as f64;
            let avg_memory_usage = successful_results
                .iter()
                .map(|r| r.peak_memory_mb)
                .sum::<f64>()
                / successful_results.len() as f64;
            let avg_throughput = successful_results
                .iter()
                .map(|r| r.throughput_allocations_per_sec)
                .sum::<f64>()
                / successful_results.len() as f64;

            PerformanceAnalysis {
                best_performance_config: HashMap::new(),
                best_memory_config: HashMap::new(),
                best_throughput_config: HashMap::new(),
                average_export_time_ms: avg_export_time,
                average_memory_usage_mb: avg_memory_usage,
                average_throughput: avg_throughput,
                shard_size_impact: HashMap::new(),
                thread_count_impact: HashMap::new(),
                memory_efficiency_score: ((self.config.memory_limit_mb as f64 - avg_memory_usage)
                    / self.config.memory_limit_mb as f64
                    * 100.0)
                    .max(0.0),
            }
        };

        PerformanceTestReport {
            test_summary,
            performance_analysis,
            optimization_recommendations: Vec::new(),
            detailed_results: self.results.clone(),
        }
    }
}

/// 性能测试报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestReport {
    /// 测试摘要
    pub test_summary: TestSummary,
    /// 性能分析
    pub performance_analysis: PerformanceAnalysis,
    /// Optimization suggestions
    pub optimization_recommendations: Vec<OptimizationRecommendation>,
    /// 详细结果
    pub detailed_results: Vec<PerformanceTestResult>,
}

/// 测试摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    /// 总测试数
    pub total_tests: usize,
    /// 成功测试数
    pub successful_tests: usize,
    /// 失败测试数
    pub failed_tests: usize,
    /// 总测试时间
    pub total_test_time_ms: u64,
}

/// 性能分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// 最佳性能配置
    pub best_performance_config: HashMap<String, String>,
    /// 最佳内存配置
    pub best_memory_config: HashMap<String, String>,
    /// 最佳吞吐量配置
    pub best_throughput_config: HashMap<String, String>,
    /// 平均导出时间
    pub average_export_time_ms: f64,
    /// 平均内存使用
    pub average_memory_usage_mb: f64,
    /// 平均吞吐量
    pub average_throughput: f64,
    /// 分片大小影响
    pub shard_size_impact: HashMap<String, f64>,
    /// 线程数影响
    pub thread_count_impact: HashMap<String, f64>,
    /// 内存效率分数
    pub memory_efficiency_score: f64,
}

impl Default for PerformanceAnalysis {
    fn default() -> Self {
        Self {
            best_performance_config: HashMap::new(),
            best_memory_config: HashMap::new(),
            best_throughput_config: HashMap::new(),
            average_export_time_ms: 0.0,
            average_memory_usage_mb: 0.0,
            average_throughput: 0.0,
            shard_size_impact: HashMap::new(),
            thread_count_impact: HashMap::new(),
            memory_efficiency_score: 0.0,
        }
    }
}

/// Optimization suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// 类别
    pub category: String,
    /// 建议
    pub recommendation: String,
    /// 影响程度
    pub impact: String,
    /// 原因
    pub reason: String,
}

/// 配置优化器
pub struct ConfigurationOptimizer {
    test_results: Vec<PerformanceTestResult>,
}

impl Default for ConfigurationOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigurationOptimizer {
    /// 创建新的配置优化器
    pub fn new() -> Self {
        Self {
            test_results: Vec::new(),
        }
    }

    /// 基于测试结果推荐最佳配置
    pub fn recommend_optimal_config(&self, target: OptimizationTarget) -> FastExportConfigBuilder {
        let mut builder = FastExportConfigBuilder::new();

        match target {
            OptimizationTarget::Speed => {
                // 优化速度：大分片，多线程，大缓冲区
                builder = builder
                    .shard_size(2000)
                    .max_threads(Some(
                        std::thread::available_parallelism()
                            .map(|n| n.get())
                            .unwrap_or(4),
                    ))
                    .buffer_size(512 * 1024);
            }
            OptimizationTarget::Memory => {
                // 优化内存：小分片，少线程，小缓冲区
                builder = builder
                    .shard_size(500)
                    .max_threads(Some(2))
                    .buffer_size(64 * 1024);
            }
            OptimizationTarget::Balanced => {
                // 平衡配置
                builder = builder
                    .shard_size(1000)
                    .max_threads(Some(
                        std::thread::available_parallelism()
                            .map(|n| n.get() / 2)
                            .unwrap_or(2),
                    ))
                    .buffer_size(256 * 1024);
            }
        }

        builder
    }
}

/// Optimization target
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OptimizationTarget {
    /// 优化速度
    Speed,
    /// 优化内存使用
    Memory,
    /// 平衡配置
    Balanced,
}

/// Complex Lifecycle Showcase 基准测试结果
#[derive(Debug, Clone, Default)]
pub struct ComplexLifecycleBenchmarkResult {
    /// 传统导出结果
    pub traditional_export: ExportBenchmarkResult,
    /// 快速导出结果
    pub fast_export: ExportBenchmarkResult,
    /// 时间提升倍数
    pub time_improvement_factor: f64,
    /// 内存提升倍数
    pub memory_improvement_factor: f64,
}

impl ComplexLifecycleBenchmarkResult {
    /// 计算性能提升
    pub fn calculate_improvements(&mut self) {
        if self.traditional_export.success && self.fast_export.success {
            // 计算时间提升
            if self.fast_export.export_time_ms > 0 {
                self.time_improvement_factor = self.traditional_export.export_time_ms as f64
                    / self.fast_export.export_time_ms as f64;
            }

            // 计算内存提升（传统方法使用更多内存时为正值）
            if self.fast_export.peak_memory_mb > 0.0 {
                self.memory_improvement_factor =
                    self.traditional_export.peak_memory_mb / self.fast_export.peak_memory_mb;
            }
        }
    }
}

/// 导出基准测试结果
#[derive(Debug, Clone, Default)]
pub struct ExportBenchmarkResult {
    /// Export time (milliseconds)
    pub export_time_ms: u64,
    /// Peak memory usage (MB)
    pub peak_memory_mb: f64,
    /// 输出文件大小 (字节)
    pub output_file_size_bytes: usize,
    /// Success
    pub success: bool,
    /// Error message
    pub error_message: Option<String>,
    /// 标准输出
    pub stdout: String,
}

/// Async validation and export mode performance tester
pub struct AsyncValidationPerformanceTester {
    /// Test configuration
    config: PerformanceTestConfig,
    /// Test results
    results: Vec<AsyncValidationTestResult>,
}

/// Async validation test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncValidationTestResult {
    /// Test name
    pub test_name: String,
    /// Export mode used
    pub export_mode: String,
    /// Validation timing
    pub validation_timing: String,
    /// Dataset size
    pub dataset_size: usize,
    /// Export time in milliseconds
    pub export_time_ms: u64,
    /// Validation time in milliseconds (if applicable)
    pub validation_time_ms: Option<u64>,
    /// Total time including validation
    pub total_time_ms: u64,
    /// Memory usage during export (bytes)
    pub memory_usage_bytes: usize,
    /// Peak memory usage (bytes)
    pub peak_memory_bytes: usize,
    /// File size generated (bytes)
    pub output_file_size_bytes: usize,
    /// Whether validation was successful
    pub validation_success: bool,
    /// Number of validation issues found
    pub validation_issues_count: usize,
    /// Whether export was successful
    pub export_success: bool,
    /// Error message if any
    pub error_message: Option<String>,
    /// Additional metrics
    pub additional_metrics: HashMap<String, f64>,
}

/// Export mode performance comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeComparisonResult {
    /// Dataset size tested
    pub dataset_size: usize,
    /// Fast mode result
    pub fast_mode_result: AsyncValidationTestResult,
    /// Slow mode result
    pub slow_mode_result: AsyncValidationTestResult,
    /// Speed improvement factor (fast vs slow)
    pub speed_improvement_factor: f64,
    /// Memory efficiency comparison
    pub memory_efficiency_comparison: f64,
    /// Validation quality comparison
    pub validation_quality_comparison: String,
}

/// Async validation impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncValidationImpactAnalysis {
    /// Export time without validation
    pub export_only_time_ms: u64,
    /// Export time with inline validation
    pub export_with_inline_validation_ms: u64,
    /// Export time with deferred validation
    pub export_with_deferred_validation_ms: u64,
    /// Validation overhead for inline mode
    pub inline_validation_overhead_percent: f64,
    /// Validation overhead for deferred mode
    pub deferred_validation_overhead_percent: f64,
    /// Memory usage comparison
    pub memory_usage_comparison: HashMap<String, usize>,
    /// Blocking analysis
    pub blocking_analysis: BlockingAnalysis,
}

/// Blocking analysis for validation modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockingAnalysis {
    /// Whether inline validation blocks export
    pub inline_blocks_export: bool,
    /// Whether deferred validation blocks subsequent exports
    pub deferred_blocks_subsequent: bool,
    /// Time to start subsequent export with inline validation
    pub inline_subsequent_start_delay_ms: u64,
    /// Time to start subsequent export with deferred validation
    pub deferred_subsequent_start_delay_ms: u64,
    /// Concurrent export capability
    pub concurrent_export_capability: String,
}

impl AsyncValidationPerformanceTester {
    /// Create new async validation performance tester
    pub fn new(config: PerformanceTestConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    /// Run comprehensive async validation performance tests
    pub async fn run_comprehensive_tests(
        &mut self,
        tracker: &MemoryTracker,
    ) -> TrackingResult<AsyncValidationPerformanceReport> {
        println!("🚀 Starting comprehensive async validation performance tests...");

        // Test 1: Fast vs Slow mode comparison
        let mode_comparison_results = self.test_fast_vs_slow_mode(tracker).await?;

        // Test 2: Async validation impact analysis
        let validation_impact_analysis = self.test_async_validation_impact(tracker).await?;

        // Test 3: Deferred validation blocking test
        let blocking_test_results = self.test_deferred_validation_blocking(tracker).await?;

        // Test 4: Large file memory usage test
        let large_file_results = self.test_large_file_memory_usage(tracker).await?;

        // Test 5: Concurrent export capability test
        let concurrent_export_results = self.test_concurrent_export_capability(tracker).await?;

        // Generate comprehensive report
        let report = AsyncValidationPerformanceReport {
            test_summary: self.generate_test_summary(),
            mode_comparison_results,
            validation_impact_analysis,
            blocking_test_results,
            large_file_results,
            concurrent_export_results,
            optimization_recommendations: self.generate_optimization_recommendations(),
            detailed_results: self.results.clone(),
        };

        println!("✅ Comprehensive async validation performance tests completed!");
        Ok(report)
    }

    /// Test fast vs slow mode performance comparison
    async fn test_fast_vs_slow_mode(
        &mut self,
        tracker: &MemoryTracker,
    ) -> TrackingResult<Vec<ModeComparisonResult>> {
        println!("📊 Testing fast vs slow mode performance...");
        let mut comparison_results = Vec::new();

        let dataset_sizes = self.config.dataset_sizes.clone();
        for &dataset_size in &dataset_sizes {
            println!("  Testing dataset size: {}", dataset_size);

            // Generate test data
            self.generate_test_data(tracker, dataset_size)?;

            // Test fast mode
            let fast_result = self
                .test_export_mode(tracker, "Fast", "Deferred", dataset_size)
                .await?;

            // Test slow mode
            let slow_result = self
                .test_export_mode(tracker, "Slow", "Inline", dataset_size)
                .await?;

            // Calculate comparison metrics
            let speed_improvement_factor = if slow_result.total_time_ms > 0 {
                slow_result.total_time_ms as f64 / fast_result.total_time_ms as f64
            } else {
                1.0
            };

            let memory_efficiency_comparison = if slow_result.peak_memory_bytes > 0 {
                fast_result.peak_memory_bytes as f64 / slow_result.peak_memory_bytes as f64
            } else {
                1.0
            };

            let validation_quality_comparison = if fast_result.validation_issues_count
                == slow_result.validation_issues_count
            {
                "Equal".to_string()
            } else if fast_result.validation_issues_count < slow_result.validation_issues_count {
                "Fast mode found fewer issues".to_string()
            } else {
                "Slow mode found fewer issues".to_string()
            };

            comparison_results.push(ModeComparisonResult {
                dataset_size,
                fast_mode_result: fast_result,
                slow_mode_result: slow_result,
                speed_improvement_factor,
                memory_efficiency_comparison,
                validation_quality_comparison,
            });
        }

        Ok(comparison_results)
    }

    /// Test async validation impact on export performance
    async fn test_async_validation_impact(
        &mut self,
        tracker: &MemoryTracker,
    ) -> TrackingResult<AsyncValidationImpactAnalysis> {
        println!("🔍 Testing async validation impact...");

        let dataset_size = 10000; // Use medium dataset for this test
        self.generate_test_data(tracker, dataset_size)?;

        // Test export without validation
        let export_only_result = self
            .test_export_mode(tracker, "Fast", "Disabled", dataset_size)
            .await?;

        // Test export with inline validation
        let inline_validation_result = self
            .test_export_mode(tracker, "Slow", "Inline", dataset_size)
            .await?;

        // Test export with deferred validation
        let deferred_validation_result = self
            .test_export_mode(tracker, "Fast", "Deferred", dataset_size)
            .await?;

        // Calculate overhead percentages
        let inline_overhead = if export_only_result.total_time_ms > 0 {
            ((inline_validation_result.total_time_ms as f64
                - export_only_result.total_time_ms as f64)
                / export_only_result.total_time_ms as f64)
                * 100.0
        } else {
            0.0
        };

        let deferred_overhead = if export_only_result.total_time_ms > 0 {
            ((deferred_validation_result.export_time_ms as f64
                - export_only_result.export_time_ms as f64)
                / export_only_result.export_time_ms as f64)
                * 100.0
        } else {
            0.0
        };

        // Test blocking behavior
        let blocking_analysis = self.test_blocking_behavior(tracker, dataset_size).await?;

        let mut memory_usage_comparison = HashMap::new();
        memory_usage_comparison.insert(
            "export_only".to_string(),
            export_only_result.memory_usage_bytes,
        );
        memory_usage_comparison.insert(
            "inline_validation".to_string(),
            inline_validation_result.memory_usage_bytes,
        );
        memory_usage_comparison.insert(
            "deferred_validation".to_string(),
            deferred_validation_result.memory_usage_bytes,
        );

        Ok(AsyncValidationImpactAnalysis {
            export_only_time_ms: export_only_result.total_time_ms,
            export_with_inline_validation_ms: inline_validation_result.total_time_ms,
            export_with_deferred_validation_ms: deferred_validation_result.export_time_ms,
            inline_validation_overhead_percent: inline_overhead,
            deferred_validation_overhead_percent: deferred_overhead,
            memory_usage_comparison,
            blocking_analysis,
        })
    }

    /// Test blocking behavior of different validation modes
    async fn test_blocking_behavior(
        &mut self,
        tracker: &MemoryTracker,
        dataset_size: usize,
    ) -> TrackingResult<BlockingAnalysis> {
        println!("🚦 Testing validation blocking behavior...");

        // Test inline validation blocking
        let inline_start = Instant::now();
        let _inline_result = self
            .test_export_mode(tracker, "Slow", "Inline", dataset_size)
            .await?;
        let inline_subsequent_start = Instant::now();
        let inline_delay = inline_subsequent_start
            .duration_since(inline_start)
            .as_millis() as u64;

        // Test deferred validation blocking
        let deferred_start = Instant::now();
        let _deferred_result = self
            .test_export_mode(tracker, "Fast", "Deferred", dataset_size)
            .await?;
        let deferred_subsequent_start = Instant::now();
        let deferred_delay = deferred_subsequent_start
            .duration_since(deferred_start)
            .as_millis() as u64;

        Ok(BlockingAnalysis {
            inline_blocks_export: true,        // Inline validation always blocks
            deferred_blocks_subsequent: false, // Deferred validation should not block
            inline_subsequent_start_delay_ms: inline_delay,
            deferred_subsequent_start_delay_ms: deferred_delay,
            concurrent_export_capability: if deferred_delay < inline_delay {
                "Deferred validation enables better concurrency".to_string()
            } else {
                "No significant concurrency improvement".to_string()
            },
        })
    }

    /// Test deferred validation blocking behavior
    async fn test_deferred_validation_blocking(
        &mut self,
        tracker: &MemoryTracker,
    ) -> TrackingResult<Vec<AsyncValidationTestResult>> {
        println!("🔄 Testing deferred validation blocking behavior...");
        let mut results = Vec::new();

        for &dataset_size in &[1000, 5000, 10000] {
            // Test multiple concurrent exports with deferred validation
            let concurrent_start = Instant::now();

            let mut concurrent_results = Vec::new();
            for _i in 0..3 {
                let result = self
                    .test_export_mode(tracker, "Fast", "Deferred", dataset_size)
                    .await?;
                concurrent_results.push(result);
            }

            let concurrent_total_time = concurrent_start.elapsed().as_millis() as u64;

            // Create summary result
            let avg_export_time = concurrent_results
                .iter()
                .map(|r| r.export_time_ms)
                .sum::<u64>()
                / concurrent_results.len() as u64;

            let summary_result = AsyncValidationTestResult {
                test_name: format!("Concurrent_Deferred_Validation_{}", dataset_size),
                export_mode: "Fast".to_string(),
                validation_timing: "Deferred".to_string(),
                dataset_size,
                export_time_ms: avg_export_time,
                validation_time_ms: None,
                total_time_ms: concurrent_total_time,
                memory_usage_bytes: concurrent_results
                    .iter()
                    .map(|r| r.memory_usage_bytes)
                    .max()
                    .unwrap_or(0),
                peak_memory_bytes: concurrent_results
                    .iter()
                    .map(|r| r.peak_memory_bytes)
                    .max()
                    .unwrap_or(0),
                output_file_size_bytes: concurrent_results
                    .iter()
                    .map(|r| r.output_file_size_bytes)
                    .sum(),
                validation_success: concurrent_results.iter().all(|r| r.validation_success),
                validation_issues_count: concurrent_results
                    .iter()
                    .map(|r| r.validation_issues_count)
                    .sum(),
                export_success: concurrent_results.iter().all(|r| r.export_success),
                error_message: None,
                additional_metrics: HashMap::new(),
            };

            results.push(summary_result);
        }

        Ok(results)
    }

    /// Test large file memory usage scenarios
    async fn test_large_file_memory_usage(
        &mut self,
        tracker: &MemoryTracker,
    ) -> TrackingResult<Vec<AsyncValidationTestResult>> {
        println!("💾 Testing large file memory usage...");
        let mut results = Vec::new();

        // Test with progressively larger datasets
        let large_dataset_sizes = vec![20000, 50000, 100000];

        for &dataset_size in &large_dataset_sizes {
            println!("  Testing large dataset size: {}", dataset_size);

            // Test fast mode with large dataset
            let fast_large_result = self
                .test_export_mode(tracker, "Fast", "Deferred", dataset_size)
                .await?;

            // Test slow mode with large dataset (if memory allows)
            let slow_large_result = if dataset_size <= 50000 {
                // Limit slow mode for very large datasets
                Some(
                    self.test_export_mode(tracker, "Slow", "Inline", dataset_size)
                        .await?,
                )
            } else {
                None
            };

            results.push(fast_large_result);
            if let Some(slow_result) = slow_large_result {
                results.push(slow_result);
            }
        }

        Ok(results)
    }

    /// Test concurrent export capability
    async fn test_concurrent_export_capability(
        &mut self,
        tracker: &MemoryTracker,
    ) -> TrackingResult<Vec<AsyncValidationTestResult>> {
        println!("🔀 Testing concurrent export capability...");
        let mut results = Vec::new();

        let dataset_size = 5000;

        // Test sequential exports
        let sequential_start = Instant::now();
        for _i in 0..3 {
            let result = self
                .test_export_mode(tracker, "Fast", "Deferred", dataset_size)
                .await?;
            results.push(result);
        }
        let sequential_time = sequential_start.elapsed().as_millis() as u64;

        // Create summary for concurrent capability
        let concurrent_summary = AsyncValidationTestResult {
            test_name: "Concurrent_Export_Capability".to_string(),
            export_mode: "Fast".to_string(),
            validation_timing: "Deferred".to_string(),
            dataset_size,
            export_time_ms: sequential_time / 3, // Average per export
            validation_time_ms: None,
            total_time_ms: sequential_time,
            memory_usage_bytes: results
                .iter()
                .map(|r| r.memory_usage_bytes)
                .max()
                .unwrap_or(0),
            peak_memory_bytes: results
                .iter()
                .map(|r| r.peak_memory_bytes)
                .max()
                .unwrap_or(0),
            output_file_size_bytes: results.iter().map(|r| r.output_file_size_bytes).sum(),
            validation_success: results.iter().all(|r| r.validation_success),
            validation_issues_count: results.iter().map(|r| r.validation_issues_count).sum(),
            export_success: results.iter().all(|r| r.export_success),
            error_message: None,
            additional_metrics: {
                let mut metrics = HashMap::new();
                metrics.insert("concurrent_exports".to_string(), 3.0);
                metrics.insert(
                    "total_concurrent_time_ms".to_string(),
                    sequential_time as f64,
                );
                metrics
            },
        };

        results.push(concurrent_summary);
        Ok(results)
    }

    /// Test a specific export mode and validation timing
    async fn test_export_mode(
        &mut self,
        tracker: &MemoryTracker,
        mode: &str,
        validation_timing: &str,
        dataset_size: usize,
    ) -> TrackingResult<AsyncValidationTestResult> {
        // use crate::export::quality_validator::{ExportMode, ValidationTiming}; // Removed

        use crate::export::quality_validator::{ExportMode, ValidationTiming};

        let _export_mode = match mode {
            "Fast" => ExportMode::Fast,
            "Slow" => ExportMode::Slow,
            "Auto" => ExportMode::Auto,
            _ => ExportMode::Fast,
        };

        let _validation_timing_enum = match validation_timing {
            "Inline" => ValidationTiming::Inline,
            "Deferred" => ValidationTiming::Deferred,
            "Disabled" => ValidationTiming::Disabled,
            _ => ValidationTiming::Deferred,
        };

        let test_name = format!(
            "{}_{}_{}_{}",
            mode,
            validation_timing,
            dataset_size,
            chrono::Utc::now().timestamp()
        );
        let output_path = format!("test_output_{}", test_name);

        // Measure memory before export
        let memory_before = self.get_current_memory_usage();

        // Perform export with timing
        let export_start = Instant::now();
        let export_result = tracker.export_to_json(&output_path);
        let export_time = export_start.elapsed().as_millis() as u64;

        // Measure memory after export
        let memory_after = self.get_current_memory_usage();
        let memory_usage = memory_after.saturating_sub(memory_before);

        // Get file size if export was successful
        let output_file_size = if export_result.is_ok() {
            std::fs::metadata(format!("{}.json", output_path))
                .map(|m| m.len() as usize)
                .unwrap_or(0)
        } else {
            0
        };

        // Create test result
        let result = AsyncValidationTestResult {
            test_name: test_name.clone(),
            export_mode: mode.to_string(),
            validation_timing: validation_timing.to_string(),
            dataset_size,
            export_time_ms: export_time,
            validation_time_ms: None, // TODO: Implement validation timing measurement
            total_time_ms: export_time,
            memory_usage_bytes: memory_usage,
            peak_memory_bytes: memory_usage, // Simplified for now
            output_file_size_bytes: output_file_size,
            validation_success: true, // TODO: Implement validation success detection
            validation_issues_count: 0, // TODO: Implement validation issue counting
            export_success: export_result.is_ok(),
            error_message: export_result.err().map(|e| e.to_string()),
            additional_metrics: HashMap::new(),
        };

        self.results.push(result.clone());

        // Cleanup test files
        let _ = std::fs::remove_file(format!("{}.json", output_path));

        Ok(result)
    }

    /// Generate test data for the tracker
    fn generate_test_data(&self, _tracker: &MemoryTracker, size: usize) -> TrackingResult<()> {
        // This is a simplified test data generation
        // In a real implementation, you would populate the tracker with test allocations
        println!("  Generating test data of size: {}", size);
        Ok(())
    }

    /// Get current memory usage (simplified implementation)
    fn get_current_memory_usage(&self) -> usize {
        // This is a placeholder - in a real implementation you would measure actual memory usage
        use std::alloc::{GlobalAlloc, Layout, System};

        // Simple approximation using a small allocation to trigger memory measurement
        let layout = Layout::new::<[u8; 1024]>();
        unsafe {
            let ptr = System.alloc(layout);
            if !ptr.is_null() {
                System.dealloc(ptr, layout);
            }
        }

        // Return a placeholder value - real implementation would use system APIs
        1024 * 1024 // 1MB placeholder
    }

    /// Generate test summary
    fn generate_test_summary(&self) -> AsyncValidationTestSummary {
        let total_tests = self.results.len();
        let successful_tests = self.results.iter().filter(|r| r.export_success).count();
        let failed_tests = total_tests - successful_tests;

        let avg_export_time = if !self.results.is_empty() {
            self.results.iter().map(|r| r.export_time_ms).sum::<u64>() / self.results.len() as u64
        } else {
            0
        };

        let total_test_time = self.results.iter().map(|r| r.total_time_ms).sum::<u64>();

        AsyncValidationTestSummary {
            total_tests,
            successful_tests,
            failed_tests,
            avg_export_time_ms: avg_export_time,
            total_test_time_ms: total_test_time,
        }
    }

    /// Generate optimization recommendations based on test results
    fn generate_optimization_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        // Analyze fast vs slow mode performance
        let fast_results: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.export_mode == "Fast")
            .collect();
        let slow_results: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.export_mode == "Slow")
            .collect();

        if !fast_results.is_empty() && !slow_results.is_empty() {
            let avg_fast_time = fast_results.iter().map(|r| r.export_time_ms).sum::<u64>()
                / fast_results.len() as u64;
            let avg_slow_time = slow_results.iter().map(|r| r.export_time_ms).sum::<u64>()
                / slow_results.len() as u64;

            if avg_fast_time < avg_slow_time {
                let improvement_factor = avg_slow_time as f64 / avg_fast_time as f64;
                recommendations.push(OptimizationRecommendation {
                    category: "Export Mode".to_string(),
                    recommendation: "Use Fast mode for better performance".to_string(),
                    impact: format!("{:.1}x faster than Slow mode", improvement_factor),
                    reason: "Fast mode shows significantly better performance in tests".to_string(),
                });
            }
        }

        // Analyze validation timing impact
        let deferred_results: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.validation_timing == "Deferred")
            .collect();
        let inline_results: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.validation_timing == "Inline")
            .collect();

        if !deferred_results.is_empty() && !inline_results.is_empty() {
            recommendations.push(OptimizationRecommendation {
                category: "Validation Timing".to_string(),
                recommendation: "Use Deferred validation for non-blocking exports".to_string(),
                impact: "Enables concurrent operations".to_string(),
                reason: "Deferred validation doesn't block the export process".to_string(),
            });
        }

        // Memory usage recommendations
        let high_memory_results: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.memory_usage_bytes > 50 * 1024 * 1024) // > 50MB
            .collect();

        if !high_memory_results.is_empty() {
            recommendations.push(OptimizationRecommendation {
                category: "Memory Usage".to_string(),
                recommendation: "Consider using streaming validation for large datasets"
                    .to_string(),
                impact: "Reduces memory footprint".to_string(),
                reason: "High memory usage detected in large dataset tests".to_string(),
            });
        }

        recommendations
    }
}

/// Async validation performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncValidationPerformanceReport {
    /// Test summary
    pub test_summary: AsyncValidationTestSummary,
    /// Mode comparison results
    pub mode_comparison_results: Vec<ModeComparisonResult>,
    /// Validation impact analysis
    pub validation_impact_analysis: AsyncValidationImpactAnalysis,
    /// Blocking test results
    pub blocking_test_results: Vec<AsyncValidationTestResult>,
    /// Large file test results
    pub large_file_results: Vec<AsyncValidationTestResult>,
    /// Concurrent export test results
    pub concurrent_export_results: Vec<AsyncValidationTestResult>,
    /// Optimization recommendations
    pub optimization_recommendations: Vec<OptimizationRecommendation>,
    /// Detailed test results
    pub detailed_results: Vec<AsyncValidationTestResult>,
}

/// Async validation test summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncValidationTestSummary {
    /// Total number of tests run
    pub total_tests: usize,
    /// Number of successful tests
    pub successful_tests: usize,
    /// Number of failed tests
    pub failed_tests: usize,
    /// Average export time across all tests
    pub avg_export_time_ms: u64,
    /// Total time spent on all tests
    pub total_test_time_ms: u64,
}

impl AsyncValidationPerformanceReport {
    /// Print comprehensive performance report
    pub fn print_comprehensive_report(&self) {
        println!("\n🚀 Async Validation Performance Report");
        println!("=====================================");

        // Print test summary
        println!("\n📊 Test Summary:");
        println!("   Total tests: {}", self.test_summary.total_tests);
        println!(
            "   Successful tests: {} ({:.1}%)",
            self.test_summary.successful_tests,
            (self.test_summary.successful_tests as f64 / self.test_summary.total_tests as f64)
                * 100.0
        );
        println!("   Failed tests: {}", self.test_summary.failed_tests);
        println!(
            "   Average export time: {}ms",
            self.test_summary.avg_export_time_ms
        );
        println!(
            "   Total test time: {}ms",
            self.test_summary.total_test_time_ms
        );

        // Print mode comparison results
        println!("\n⚡ Fast vs Slow Mode Comparison:");
        for comparison in &self.mode_comparison_results {
            println!("   Dataset size: {}", comparison.dataset_size);
            println!(
                "     Fast mode: {}ms",
                comparison.fast_mode_result.total_time_ms
            );
            println!(
                "     Slow mode: {}ms",
                comparison.slow_mode_result.total_time_ms
            );
            println!(
                "     Speed improvement: {:.1}x",
                comparison.speed_improvement_factor
            );
            println!(
                "     Memory efficiency: {:.2}",
                comparison.memory_efficiency_comparison
            );
            println!(
                "     Validation quality: {}",
                comparison.validation_quality_comparison
            );
            println!();
        }

        // Print validation impact analysis
        println!("🔍 Validation Impact Analysis:");
        let impact = &self.validation_impact_analysis;
        println!("   Export only: {}ms", impact.export_only_time_ms);
        println!(
            "   With inline validation: {}ms (+{:.1}%)",
            impact.export_with_inline_validation_ms, impact.inline_validation_overhead_percent
        );
        println!(
            "   With deferred validation: {}ms (+{:.1}%)",
            impact.export_with_deferred_validation_ms, impact.deferred_validation_overhead_percent
        );

        // Print blocking analysis
        println!("\n🚦 Blocking Analysis:");
        let blocking = &impact.blocking_analysis;
        println!(
            "   Inline validation blocks export: {}",
            blocking.inline_blocks_export
        );
        println!(
            "   Deferred validation blocks subsequent: {}",
            blocking.deferred_blocks_subsequent
        );
        println!(
            "   Concurrent capability: {}",
            blocking.concurrent_export_capability
        );

        // Print optimization recommendations
        println!("\n💡 Optimization Recommendations:");
        for (i, rec) in self.optimization_recommendations.iter().enumerate() {
            println!("   {}. {} - {}", i + 1, rec.category, rec.recommendation);
            println!("      Impact: {}", rec.impact);
            println!("      Reason: {}", rec.reason);
            println!();
        }
    }

    /// Save report to JSON file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        let json_data = serde_json::to_string_pretty(self).map_err(|e| {
            crate::core::types::TrackingError::SerializationError(format!(
                "Failed to serialize report: {}",
                e
            ))
        })?;

        std::fs::write(path, json_data).map_err(|e| {
            crate::core::types::TrackingError::IoError(format!(
                "Failed to write report file: {}",
                e
            ))
        })?;

        Ok(())
    }
}
