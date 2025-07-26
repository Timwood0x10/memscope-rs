//! Performance benchmark module
//!
//! This module provides comprehensive performance benchmarking functionality
//! to compare export performance before and after optimization,
//! particularly using complex_lifecycle_showcase.rs as the benchmark test case.

use crate::core::tracker::get_global_tracker;
use crate::core::types::{TrackingResult, AllocationInfo};
use crate::export::fast_export_coordinator::{FastExportCoordinator, FastExportConfig, CompleteExportStats};
use crate::export::optimized_json_export::OptimizedExportOptions;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of test runs
    pub test_runs: usize,
    /// Output directory
    pub output_dir: PathBuf,
    /// 是否启用详细日志
    pub verbose: bool,
    /// 是否验证输出一致性
    pub verify_consistency: bool,
    /// 是否生成详细报告
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

/// 单次测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// 测试名称
    pub test_name: String,
    /// 导出时间（毫秒）
    pub export_time_ms: u64,
    /// 内存使用峰值（字节）
    pub peak_memory_bytes: usize,
    /// 输出文件大小（字节）
    pub output_file_size: usize,
    /// 处理的分配数量
    pub allocations_processed: usize,
    /// 吞吐量（分配/秒）
    pub throughput_allocations_per_sec: f64,
    /// 写入速度（MB/s）
    pub write_speed_mbps: f64,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果有）
    pub error_message: Option<String>,
}/// 基准测试比
较结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    /// 传统导出结果
    pub traditional_results: Vec<BenchmarkResult>,
    /// 快速导出结果
    pub fast_results: Vec<BenchmarkResult>,
    /// 性能提升统计
    pub performance_improvement: PerformanceImprovement,
    /// 测试配置
    pub config: BenchmarkConfig,
    /// 测试时间戳
    pub timestamp: String,
}

/// 性能提升统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImprovement {
    /// 平均导出时间改善（百分比）
    pub avg_time_improvement_percent: f64,
    /// 平均内存使用改善（百分比）
    pub avg_memory_improvement_percent: f64,
    /// 平均吞吐量提升（百分比）
    pub avg_throughput_improvement_percent: f64,
    /// 平均写入速度提升（百分比）
    pub avg_write_speed_improvement_percent: f64,
    /// 最佳时间改善（百分比）
    pub best_time_improvement_percent: f64,
    /// 最差时间改善（百分比）
    pub worst_time_improvement_percent: f64,
    /// 一致性评分（0-100）
    pub consistency_score: f64,
}

/// 性能基准测试器
pub struct PerformanceBenchmark {
    /// 配置
    config: BenchmarkConfig,
    /// 测试结果历史
    results_history: Vec<BenchmarkComparison>,
}

impl PerformanceBenchmark {
    /// 创建新的性能基准测试器
    pub fn new(config: BenchmarkConfig) -> TrackingResult<Self> {
        // 创建输出目录
        fs::create_dir_all(&config.output_dir)?;
        
        Ok(Self {
            config,
            results_history: Vec::new(),
        })
    }

    /// 运行完整的基准测试
    pub fn run_full_benchmark(&mut self) -> TrackingResult<BenchmarkComparison> {
        println!("🚀 开始性能基准测试");
        println!("==================");
        println!("测试配置:");
        println!("  - 运行次数: {}", self.config.test_runs);
        println!("  - 输出目录: {}", self.config.output_dir.display());
        println!("  - 验证一致性: {}", self.config.verify_consistency);
        println!();

        // 运行 complex_lifecycle_showcase 生成测试数据
        self.prepare_test_data()?;

        // 运行传统导出测试
        println!("📊 测试传统导出系统...");
        let traditional_results = self.run_traditional_export_tests()?;

        // 运行快速导出测试
        println!("⚡ 测试快速导出系统...");
        let fast_results = self.run_fast_export_tests()?;

        // 计算性能提升
        let performance_improvement = self.calculate_performance_improvement(&traditional_results, &fast_results);

        // 创建比较结果
        let comparison = BenchmarkComparison {
            traditional_results,
            fast_results,
            performance_improvement,
            config: self.config.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // 保存结果
        self.save_benchmark_results(&comparison)?;

        // 生成报告
        if self.config.generate_detailed_report {
            self.generate_detailed_report(&comparison)?;
        }

        // 添加到历史记录
        self.results_history.push(comparison.clone());

        Ok(comparison)
    }    /// 准备
测试数据
    fn prepare_test_data(&self) -> TrackingResult<()> {
        println!("🔧 准备测试数据...");
        
        // 运行 complex_lifecycle_showcase 示例来生成复杂的内存分配模式
        let output = Command::new("cargo")
            .args(&["run", "--example", "complex_lifecycle_showcase"])
            .output()
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::core::types::TrackingError::ExportError(
                format!("Failed to run complex_lifecycle_showcase: {}", stderr)
            ));
        }

        if self.config.verbose {
            println!("✅ 测试数据准备完成");
        }

        Ok(())
    }

    /// 运行传统导出测试
    fn run_traditional_export_tests(&self) -> TrackingResult<Vec<BenchmarkResult>> {
        let mut results = Vec::new();

        for run in 1..=self.config.test_runs {
            if self.config.verbose {
                println!("  运行 {}/{}: 传统导出", run, self.config.test_runs);
            }

            let result = self.run_single_traditional_test(run)?;
            results.push(result);

            // 短暂休息以避免系统负载影响
            std::thread::sleep(Duration::from_millis(100));
        }

        Ok(results)
    }

    /// 运行快速导出测试
    fn run_fast_export_tests(&self) -> TrackingResult<Vec<BenchmarkResult>> {
        let mut results = Vec::new();

        for run in 1..=self.config.test_runs {
            if self.config.verbose {
                println!("  运行 {}/{}: 快速导出", run, self.config.test_runs);
            }

            let result = self.run_single_fast_test(run)?;
            results.push(result);

            // 短暂休息以避免系统负载影响
            std::thread::sleep(Duration::from_millis(100));
        }

        Ok(results)
    }

    /// 运行单次传统导出测试
    fn run_single_traditional_test(&self, run_number: usize) -> TrackingResult<BenchmarkResult> {
        let start_time = Instant::now();
        let output_path = self.config.output_dir.join(format!("traditional_export_run_{}.json", run_number));

        // 获取当前内存跟踪器状态
        let tracker = get_global_tracker();
        let initial_stats = tracker.get_stats()?;

        // 使用传统的优化导出选项
        let options = OptimizedExportOptions {
            enable_streaming: true,
            enable_compression: false,
            batch_size: 1000,
            enable_parallel_processing: false, // 传统方式不使用并行
            max_file_size_mb: 100,
            output_format: crate::export::optimized_json_export::OutputFormat::Json,
        };

        // 执行传统导出
        let export_result = tracker.export_to_json_with_optimized_options(&output_path, options);
        let export_time = start_time.elapsed();

        // 获取最终统计
        let final_stats = tracker.get_stats()?;

        // 检查文件大小
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
            println!("    ⏱️  时间: {}ms, 📊 分配: {}, 📁 大小: {:.2}MB", 
                    result.export_time_ms, 
                    result.allocations_processed,
                    result.output_file_size as f64 / 1024.0 / 1024.0);
        }

        Ok(result)
    }