//! Performance benchmark module
//!
//! This module provides comprehensive performance benchmarking functionality
//! to compare export performance before and after optimization,
//! particularly using complex_lifecycle_showcase.rs as the benchmark test case.

use crate::core::tracker::get_global_tracker;
use crate::core::types::{TrackingResult, AllocationInfo};
use crate::export::fast_export_coordinator::{FastExportCoordinator, FastExportConfig};
use crate::export::optimized_json_export::OptimizedExportOptions;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    }    /// 准备测试数据
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

    /// 运行单次快速导出测试
    fn run_single_fast_test(&self, run_number: usize) -> TrackingResult<BenchmarkResult> {
        let start_time = Instant::now();
        let output_path = self.config.output_dir.join(format!("fast_export_run_{}.json", run_number));

        // 获取当前内存跟踪器状态
        let tracker = get_global_tracker();
        let initial_stats = tracker.get_stats()?;

        // 使用快速导出配置
        let fast_config = FastExportConfig::default();

        // 创建快速导出协调器
        let mut coordinator = FastExportCoordinator::new(fast_config);

        // 执行快速导出
        let export_result = coordinator.export_fast(&output_path);
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
            println!("    ⚡ 时间: {}ms, 📊 分配: {}, 📁 大小: {:.2}MB", 
                    result.export_time_ms, 
                    result.allocations_processed,
                    result.output_file_size as f64 / 1024.0 / 1024.0);
        }

        Ok(result)
    }

    /// 计算性能提升统计
    fn calculate_performance_improvement(
        &self,
        traditional_results: &[BenchmarkResult],
        fast_results: &[BenchmarkResult],
    ) -> PerformanceImprovement {
        // 计算平均值
        let avg_traditional_time = traditional_results.iter()
            .map(|r| r.export_time_ms as f64)
            .sum::<f64>() / traditional_results.len() as f64;

        let avg_fast_time = fast_results.iter()
            .map(|r| r.export_time_ms as f64)
            .sum::<f64>() / fast_results.len() as f64;

        let avg_traditional_memory = traditional_results.iter()
            .map(|r| r.peak_memory_bytes as f64)
            .sum::<f64>() / traditional_results.len() as f64;

        let avg_fast_memory = fast_results.iter()
            .map(|r| r.peak_memory_bytes as f64)
            .sum::<f64>() / fast_results.len() as f64;

        let avg_traditional_throughput = traditional_results.iter()
            .map(|r| r.throughput_allocations_per_sec)
            .sum::<f64>() / traditional_results.len() as f64;

        let avg_fast_throughput = fast_results.iter()
            .map(|r| r.throughput_allocations_per_sec)
            .sum::<f64>() / fast_results.len() as f64;

        let avg_traditional_write_speed = traditional_results.iter()
            .map(|r| r.write_speed_mbps)
            .sum::<f64>() / traditional_results.len() as f64;

        let avg_fast_write_speed = fast_results.iter()
            .map(|r| r.write_speed_mbps)
            .sum::<f64>() / fast_results.len() as f64;

        // 计算改善百分比
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
            ((avg_fast_throughput - avg_traditional_throughput) / avg_traditional_throughput) * 100.0
        } else {
            0.0
        };

        let avg_write_speed_improvement_percent = if avg_traditional_write_speed > 0.0 {
            ((avg_fast_write_speed - avg_traditional_write_speed) / avg_traditional_write_speed) * 100.0
        } else {
            0.0
        };

        // 计算最佳和最差改善
        let traditional_times: Vec<f64> = traditional_results.iter().map(|r| r.export_time_ms as f64).collect();
        let fast_times: Vec<f64> = fast_results.iter().map(|r| r.export_time_ms as f64).collect();

        let best_traditional_time = traditional_times.iter().cloned().fold(f64::INFINITY, f64::min);
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

        // 计算一致性评分（基于标准差）
        let traditional_std = self.calculate_std_dev(&traditional_times);
        let fast_std = self.calculate_std_dev(&fast_times);
        let consistency_score = if traditional_std > 0.0 {
            ((traditional_std - fast_std) / traditional_std * 100.0).max(0.0).min(100.0)
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

    /// 计算标准差
    fn calculate_std_dev(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        variance.sqrt()
    }

    /// 保存基准测试结果
    fn save_benchmark_results(&self, comparison: &BenchmarkComparison) -> TrackingResult<()> {
        let results_file = self.config.output_dir.join("benchmark_results.json");
        let json_data = serde_json::to_string_pretty(comparison)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))?;
        
        fs::write(&results_file, json_data)
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;

        if self.config.verbose {
            println!("💾 基准测试结果已保存到: {}", results_file.display());
        }

        Ok(())
    }

    /// 生成详细报告
    fn generate_detailed_report(&self, comparison: &BenchmarkComparison) -> TrackingResult<()> {
        let report_file = self.config.output_dir.join("performance_report.md");
        let mut report = String::new();

        // 报告标题
        report.push_str("# 大型项目导出优化 - 性能基准测试报告\n\n");
        report.push_str(&format!("**测试时间**: {}\n\n", comparison.timestamp));
        report.push_str(&format!("**测试配置**:\n"));
        report.push_str(&format!("- 运行次数: {}\n", comparison.config.test_runs));
        report.push_str(&format!("- 验证一致性: {}\n", comparison.config.verify_consistency));
        report.push_str("\n");

        // 性能提升摘要
        let perf = &comparison.performance_improvement;
        report.push_str("## 📊 性能提升摘要\n\n");
        report.push_str(&format!("| 指标 | 改善幅度 |\n"));
        report.push_str(&format!("|------|----------|\n"));
        report.push_str(&format!("| 平均导出时间 | **{:.1}%** |\n", perf.avg_time_improvement_percent));
        report.push_str(&format!("| 平均内存使用 | **{:.1}%** |\n", perf.avg_memory_improvement_percent));
        report.push_str(&format!("| 平均吞吐量 | **+{:.1}%** |\n", perf.avg_throughput_improvement_percent));
        report.push_str(&format!("| 平均写入速度 | **+{:.1}%** |\n", perf.avg_write_speed_improvement_percent));
        report.push_str(&format!("| 最佳时间改善 | **{:.1}%** |\n", perf.best_time_improvement_percent));
        report.push_str(&format!("| 最差时间改善 | **{:.1}%** |\n", perf.worst_time_improvement_percent));
        report.push_str(&format!("| 一致性评分 | **{:.1}/100** |\n", perf.consistency_score));
        report.push_str("\n");

        // 详细结果对比
        report.push_str("## 📈 详细结果对比\n\n");
        report.push_str("### 传统导出系统\n\n");
        report.push_str("| 运行 | 时间(ms) | 内存(MB) | 文件大小(MB) | 吞吐量(alloc/s) | 写入速度(MB/s) |\n");
        report.push_str("|------|----------|----------|--------------|-----------------|----------------|\n");
        
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

        report.push_str("\n### 快速导出系统\n\n");
        report.push_str("| 运行 | 时间(ms) | 内存(MB) | 文件大小(MB) | 吞吐量(alloc/s) | 写入速度(MB/s) |\n");
        report.push_str("|------|----------|----------|--------------|-----------------|----------------|\n");
        
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

        // 结论和建议
        report.push_str("\n## 🎯 结论和建议\n\n");
        
        if perf.avg_time_improvement_percent > 50.0 {
            report.push_str("✅ **优秀**: 快速导出系统实现了显著的性能提升，超过了50%的时间改善目标。\n\n");
        } else if perf.avg_time_improvement_percent > 30.0 {
            report.push_str("✅ **良好**: 快速导出系统实现了良好的性能提升，达到了30%以上的时间改善。\n\n");
        } else {
            report.push_str("⚠️ **需要改进**: 快速导出系统的性能提升低于预期，建议进一步优化。\n\n");
        }

        if perf.consistency_score > 80.0 {
            report.push_str("✅ **一致性优秀**: 快速导出系统表现稳定，结果一致性高。\n\n");
        } else if perf.consistency_score > 60.0 {
            report.push_str("✅ **一致性良好**: 快速导出系统表现较为稳定。\n\n");
        } else {
            report.push_str("⚠️ **一致性需要改进**: 快速导出系统结果波动较大，建议优化稳定性。\n\n");
        }

        // 保存报告
        fs::write(&report_file, report)
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;

        if self.config.verbose {
            println!("📄 详细报告已生成: {}", report_file.display());
        }

        Ok(())
    }

    /// 验证输出一致性
    fn verify_output_consistency(&self, traditional_path: &Path, fast_path: &Path) -> TrackingResult<bool> {
        if !self.config.verify_consistency {
            return Ok(true);
        }

        // 读取两个文件
        let traditional_content = fs::read_to_string(traditional_path)
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;
        let fast_content = fs::read_to_string(fast_path)
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;

        // 解析 JSON 并比较结构
        let traditional_json: serde_json::Value = serde_json::from_str(&traditional_content)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))?;
        let fast_json: serde_json::Value = serde_json::from_str(&fast_content)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))?;

        // 比较关键字段
        let consistent = self.compare_json_structure(&traditional_json, &fast_json);

        if self.config.verbose {
            if consistent {
                println!("✅ 输出一致性验证通过");
            } else {
                println!("❌ 输出一致性验证失败");
            }
        }

        Ok(consistent)
    }

    /// 比较 JSON 结构
    fn compare_json_structure(&self, traditional: &serde_json::Value, fast: &serde_json::Value) -> bool {
        // 简化的结构比较 - 检查关键字段是否存在
        match (traditional, fast) {
            (serde_json::Value::Object(t_obj), serde_json::Value::Object(f_obj)) => {
                // 检查关键字段
                let key_fields = ["allocations", "stats", "metadata"];
                for field in &key_fields {
                    if t_obj.contains_key(*field) != f_obj.contains_key(*field) {
                        return false;
                    }
                }
                
                // 如果都有 allocations 字段，检查数量
                if let (Some(t_allocs), Some(f_allocs)) = (t_obj.get("allocations"), f_obj.get("allocations")) {
                    if let (serde_json::Value::Array(t_arr), serde_json::Value::Array(f_arr)) = (t_allocs, f_allocs) {
                        if t_arr.len() != f_arr.len() {
                            return false;
                        }
                    }
                }
                
                true
            }
            _ => traditional == fast,
        }
    }

    /// 运行单个基准测试并返回结果
    pub fn run_single_benchmark(&mut self, test_name: &str) -> TrackingResult<BenchmarkComparison> {
        println!("🎯 运行单个基准测试: {}", test_name);
        
        // 准备测试数据
        self.prepare_test_data()?;
        
        // 运行单次测试
        let traditional_result = self.run_single_traditional_test(1)?;
        let fast_result = self.run_single_fast_test(1)?;
        
        let performance_improvement = self.calculate_performance_improvement(
            &[traditional_result.clone()], 
            &[fast_result.clone()]
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