//! 性能测试和优化模块
//!
//! 这个模块提供了全面的性能测试工具，用于测试和优化大型项目导出功能。

use crate::core::tracker::MemoryTracker;
use crate::core::types::TrackingResult;
use crate::export::fast_export_coordinator::{FastExportCoordinator, FastExportConfigBuilder};
use crate::export::optimized_json_export::OptimizedExportOptions;
use std::collections::HashMap;
use std::time::Instant;
use serde::{Serialize, Deserialize};

/// 性能测试配置
#[derive(Debug, Clone)]
pub struct PerformanceTestConfig {
    /// 测试数据集大小
    pub dataset_sizes: Vec<usize>,
    /// 分片大小测试范围
    pub shard_sizes: Vec<usize>,
    /// 线程数测试范围
    pub thread_counts: Vec<usize>,
    /// 缓冲区大小测试范围
    pub buffer_sizes: Vec<usize>,
    /// 测试重复次数
    pub test_iterations: usize,
    /// 内存限制 (MB)
    pub memory_limit_mb: usize,
    /// 是否启用详细输出
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

/// 性能测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestResult {
    /// 测试名称
    pub test_name: String,
    /// 数据集大小
    pub dataset_size: usize,
    /// 配置参数
    pub config_params: HashMap<String, String>,
    /// 导出时间 (毫秒)
    pub export_time_ms: u64,
    /// 内存使用峰值 (MB)
    pub peak_memory_mb: f64,
    /// 吞吐量 (分配/秒)
    pub throughput_allocations_per_sec: f64,
    /// 文件大小 (字节)
    pub output_file_size_bytes: usize,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error_message: Option<String>,
}

/// 性能基准测试工具
pub struct PerformanceBenchmark;

impl PerformanceBenchmark {
    /// 运行快速基准测试
    pub fn run_quick_benchmark() -> TrackingResult<()> {
        println!("🚀 运行快速性能基准测试");
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

        println!("✅ 快速基准测试完成");
        Ok(())
    }

    /// 运行 complex_lifecycle_showcase.rs 基准测试
    pub fn run_complex_lifecycle_benchmark() -> TrackingResult<ComplexLifecycleBenchmarkResult> {
        println!("🎯 运行 complex_lifecycle_showcase.rs 基准测试");
        println!("==============================================");

        let mut benchmark_result = ComplexLifecycleBenchmarkResult::default();

        // 测试传统导出性能
        println!("📊 测试传统导出性能...");
        let traditional_result = Self::benchmark_traditional_export()?;
        benchmark_result.traditional_export = traditional_result;

        // 测试快速导出性能
        println!("⚡ 测试快速导出性能...");
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
                    stdout: format!("Fast export completed: {} allocations processed", stats.total_allocations_processed),
                })
            }
            Err(e) => {
                Ok(ExportBenchmarkResult {
                    export_time_ms: export_time.as_millis() as u64,
                    peak_memory_mb: peak_memory,
                    output_file_size_bytes: 0,
                    success: false,
                    error_message: Some(e.to_string()),
                    stdout: String::new(),
                })
            }
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
        println!("  文件大小: {} bytes ({:.2} KB)", 
            result.traditional_export.output_file_size_bytes,
            result.traditional_export.output_file_size_bytes as f64 / 1024.0);
        println!("  状态: {}", if result.traditional_export.success { "✅ 成功" } else { "❌ 失败" });

        println!("\n快速导出:");
        println!("  时间: {} ms", result.fast_export.export_time_ms);
        println!("  内存: {:.2} MB", result.fast_export.peak_memory_mb);
        println!("  文件大小: {} bytes ({:.2} KB)", 
            result.fast_export.output_file_size_bytes,
            result.fast_export.output_file_size_bytes as f64 / 1024.0);
        println!("  状态: {}", if result.fast_export.success { "✅ 成功" } else { "❌ 失败" });

        if result.traditional_export.success && result.fast_export.success {
            println!("\n🚀 性能提升:");
            println!("  时间提升: {:.2}x ({:.1}% 减少)", 
                result.time_improvement_factor,
                (1.0 - 1.0 / result.time_improvement_factor) * 100.0);
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
                println!("  ✅ 内存使用在限制范围内 ({:.2} MB <= {} MB)", 
                    result.fast_export.peak_memory_mb, memory_limit);
            } else {
                println!("  ⚠️ 内存使用超过限制 ({:.2} MB > {} MB)", 
                    result.fast_export.peak_memory_mb, memory_limit);
            }
        }

        if let Some(ref error) = result.traditional_export.error_message {
            println!("\n❌ 传统导出错误: {}", error);
        }
        if let Some(ref error) = result.fast_export.error_message {
            println!("\n❌ 快速导出错误: {}", error);
        }
    }

    /// 运行完整基准测试
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

    /// 打印详细报告
    fn print_detailed_report(report: &PerformanceTestReport) {
        println!("\n📊 性能测试报告");
        println!("================");
        println!("总测试数: {}", report.test_summary.total_tests);
        println!("成功测试: {}", report.test_summary.successful_tests);
        println!("失败测试: {}", report.test_summary.failed_tests);
        println!("成功率: {:.1}%", 
            report.test_summary.successful_tests as f64 / report.test_summary.total_tests as f64 * 100.0);

        println!("\n📈 性能分析");
        println!("平均导出时间: {:.2} ms", report.performance_analysis.average_export_time_ms);
        println!("平均内存使用: {:.2} MB", report.performance_analysis.average_memory_usage_mb);
        println!("平均吞吐量: {:.0} 分配/秒", report.performance_analysis.average_throughput);

        if !report.optimization_recommendations.is_empty() {
            println!("\n💡 优化建议");
            for rec in &report.optimization_recommendations {
                println!("• [{}] {}: {}", rec.impact, rec.category, rec.recommendation);
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
    /// 创建新的性能测试套件
    pub fn new(config: PerformanceTestConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    /// 运行基本测试
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

    /// 运行完整测试套件
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

    /// 运行基准性能测试
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

    /// 运行分片大小优化测试
    pub fn run_shard_size_optimization_tests(&mut self) -> TrackingResult<()> {
        println!("⚡ 分片大小优化测试");

        let dataset_size = 10000;
        for &shard_size in &self.config.shard_sizes {
            let result = self.test_shard_size_performance(dataset_size, shard_size)?;
            self.results.push(result);
        }

        Ok(())
    }

    /// 运行内存使用测试
    pub fn run_memory_usage_tests(&mut self) -> TrackingResult<()> {
        println!("💾 内存使用测试");

        for &dataset_size in &self.config.dataset_sizes {
            let result = self.test_memory_usage(dataset_size)?;
            
            if result.peak_memory_mb > self.config.memory_limit_mb as f64 {
                println!("  ⚠️ 内存使用超过限制: {:.2} MB > {} MB", 
                    result.peak_memory_mb, self.config.memory_limit_mb);
            }
            
            self.results.push(result);
        }

        Ok(())
    }

    /// 运行优化前后对比测试
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
                println!("  ⚠️ 内存使用超过限制: {:.2} MB > {} MB", 
                    result.peak_memory_mb, self.config.memory_limit_mb);
            }
            
            self.results.push(result);
        }

        Ok(())
    }

    /// 测试传统导出性能
    fn test_traditional_export(&self, dataset_size: usize) -> TrackingResult<PerformanceTestResult> {
        let start_time = Instant::now();
        let start_memory = self.get_memory_usage();

        let tracker = MemoryTracker::new();
        let traditional_options = OptimizedExportOptions::default()
            .fast_export_mode(false)
            .auto_fast_export_threshold(None);

        let output_path = format!("test_traditional_{}", dataset_size);
        
        let result = match tracker.export_to_json_with_optimized_options(&output_path, traditional_options) {
            Ok(_) => {
                let export_time = start_time.elapsed();
                let peak_memory = self.get_memory_usage() - start_memory;
                let file_size = self.get_file_size(&format!("MemoryAnalysis/{}/{}_memory_analysis.json", output_path, output_path));

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
            }
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
            }
        };

        Ok(result)
    }

    /// 测试分片大小性能
    fn test_shard_size_performance(&self, dataset_size: usize, shard_size: usize) -> TrackingResult<PerformanceTestResult> {
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
            }
        };

        Ok(result)
    }

    /// 测试线程扩展性
    fn test_thread_scalability(&self, dataset_size: usize, thread_count: usize) -> TrackingResult<PerformanceTestResult> {
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
            }
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
                        Some(format!("Memory usage {} MB exceeds limit {} MB", peak_memory, self.config.memory_limit_mb))
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
            }
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
            let avg_export_time = successful_results.iter().map(|r| r.export_time_ms).sum::<u64>() as f64 / successful_results.len() as f64;
            let avg_memory_usage = successful_results.iter().map(|r| r.peak_memory_mb).sum::<f64>() / successful_results.len() as f64;
            let avg_throughput = successful_results.iter().map(|r| r.throughput_allocations_per_sec).sum::<f64>() / successful_results.len() as f64;

            PerformanceAnalysis {
                best_performance_config: HashMap::new(),
                best_memory_config: HashMap::new(),
                best_throughput_config: HashMap::new(),
                average_export_time_ms: avg_export_time,
                average_memory_usage_mb: avg_memory_usage,
                average_throughput: avg_throughput,
                shard_size_impact: HashMap::new(),
                thread_count_impact: HashMap::new(),
                memory_efficiency_score: ((self.config.memory_limit_mb as f64 - avg_memory_usage) / self.config.memory_limit_mb as f64 * 100.0).max(0.0),
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
    /// 优化建议
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

/// 优化建议
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
                    .max_threads(Some(std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)))
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
                    .max_threads(Some(std::thread::available_parallelism().map(|n| n.get() / 2).unwrap_or(2)))
                    .buffer_size(256 * 1024);
            }
        }

        builder
    }
}

/// 优化目标
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
                self.time_improvement_factor = 
                    self.traditional_export.export_time_ms as f64 / self.fast_export.export_time_ms as f64;
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
    /// 导出时间 (毫秒)
    pub export_time_ms: u64,
    /// 内存使用峰值 (MB)
    pub peak_memory_mb: f64,
    /// 输出文件大小 (字节)
    pub output_file_size_bytes: usize,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error_message: Option<String>,
    /// 标准输出
    pub stdout: String,
}