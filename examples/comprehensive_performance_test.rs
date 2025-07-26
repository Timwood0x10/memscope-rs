//! 综合性能测试示例
//!
//! 这个示例展示了如何使用性能测试模块进行全面的性能测试和优化分析。
//! 包括使用 complex_lifecycle_showcase.rs 作为基准测试用例。

use memscope_rs::export::performance_testing::{
    PerformanceTestConfig, PerformanceTestSuite, PerformanceBenchmark,
    ConfigurationOptimizer, OptimizationTarget, PerformanceTestResult,
};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 综合性能测试示例");
    println!("==================");
    
    // 测试 1: Complex Lifecycle Showcase 基准测试
    println!("\n1️⃣ Complex Lifecycle Showcase 基准测试:");
    run_complex_lifecycle_benchmark()?;
    
    // 测试 2: 分片大小优化测试
    println!("\n2️⃣ 分片大小优化测试:");
    run_shard_size_optimization_test()?;
    
    // 测试 3: 多线程扩展性测试
    println!("\n3️⃣ 多线程扩展性测试:");
    run_thread_scalability_test()?;
    
    // 测试 4: 内存使用限制测试
    println!("\n4️⃣ 内存使用限制测试:");
    run_memory_limit_test()?;
    
    // 测试 5: 配置优化建议
    println!("\n5️⃣ 配置优化建议:");
    run_configuration_optimization()?;
    
    // 测试 6: 完整性能测试套件
    println!("\n6️⃣ 完整性能测试套件:");
    run_comprehensive_test_suite()?;
    
    println!("\n✅ 综合性能测试完成!");
    
    Ok(())
}

/// 运行 Complex Lifecycle Showcase 基准测试
fn run_complex_lifecycle_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    println!("运行 complex_lifecycle_showcase.rs 基准测试...");
    
    let start_time = Instant::now();
    let benchmark_result = PerformanceBenchmark::run_complex_lifecycle_benchmark()?;
    let total_time = start_time.elapsed();
    
    println!("\n📊 基准测试摘要:");
    println!("  总测试时间: {:?}", total_time);
    
    // 验证性能目标
    let target_improvement = 2.0; // 目标：减少 60-80% 导出时间
    let memory_limit = 64.0; // 64MB 内存限制
    
    let mut goals_met = 0;
    let total_goals = 3;
    
    if benchmark_result.time_improvement_factor >= target_improvement {
        println!("  ✅ 时间优化目标达成 ({:.2}x >= {:.1}x)", 
            benchmark_result.time_improvement_factor, target_improvement);
        goals_met += 1;
    } else {
        println!("  ❌ 时间优化目标未达成 ({:.2}x < {:.1}x)", 
            benchmark_result.time_improvement_factor, target_improvement);
    }
    
    if benchmark_result.fast_export.peak_memory_mb <= memory_limit {
        println!("  ✅ 内存限制目标达成 ({:.2} MB <= {} MB)", 
            benchmark_result.fast_export.peak_memory_mb, memory_limit);
        goals_met += 1;
    } else {
        println!("  ❌ 内存限制目标未达成 ({:.2} MB > {} MB)", 
            benchmark_result.fast_export.peak_memory_mb, memory_limit);
    }
    
    if benchmark_result.traditional_export.success && benchmark_result.fast_export.success {
        println!("  ✅ 导出功能正常工作");
        goals_met += 1;
    } else {
        println!("  ❌ 导出功能存在问题");
    }
    
    println!("  📈 目标达成率: {}/{} ({:.1}%)", 
        goals_met, total_goals, goals_met as f64 / total_goals as f64 * 100.0);
    
    Ok(())
}

/// 运行分片大小优化测试
fn run_shard_size_optimization_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试不同分片大小对性能的影响...");
    
    let config = PerformanceTestConfig {
        dataset_sizes: vec![10000],
        shard_sizes: vec![250, 500, 1000, 2000, 4000],
        thread_counts: vec![4],
        buffer_sizes: vec![256 * 1024],
        test_iterations: 2,
        memory_limit_mb: 64,
        verbose: false,
    };
    
    let mut test_suite = PerformanceTestSuite::new(config);
    test_suite.run_shard_size_optimization_tests()?;
    
    let report = test_suite.generate_performance_report();
    
    // 分析分片大小影响
    let shard_results: Vec<_> = report.detailed_results.iter()
        .filter(|r| r.test_name == "shard_size_test")
        .collect();
    
    if !shard_results.is_empty() {
        println!("分片大小性能分析:");
        
        let mut best_performance: Option<&PerformanceTestResult> = None;
        let mut best_memory: Option<&PerformanceTestResult> = None;
        
        for result in &shard_results {
            if let Some(shard_size) = result.config_params.get("shard_size") {
                println!("  分片大小 {}: {} ms, {:.2} MB, {:.0} 分配/秒", 
                    shard_size, 
                    result.export_time_ms,
                    result.peak_memory_mb,
                    result.throughput_allocations_per_sec);
                
                // 找到最佳性能配置
                if best_performance.is_none() || result.export_time_ms < best_performance.unwrap().export_time_ms {
                    best_performance = Some(result);
                }
                
                // 找到最佳内存配置
                if best_memory.is_none() || result.peak_memory_mb < best_memory.unwrap().peak_memory_mb {
                    best_memory = Some(result);
                }
            }
        }
        
        if let Some(best_perf) = best_performance {
            if let Some(shard_size) = best_perf.config_params.get("shard_size") {
                println!("  🏆 最佳性能分片大小: {} ({}ms)", shard_size, best_perf.export_time_ms);
            }
        }
        
        if let Some(best_mem) = best_memory {
            if let Some(shard_size) = best_mem.config_params.get("shard_size") {
                println!("  💾 最佳内存分片大小: {} ({:.2}MB)", shard_size, best_mem.peak_memory_mb);
            }
        }
    }
    
    Ok(())
}

/// 运行多线程扩展性测试
fn run_thread_scalability_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试多线程处理的扩展性...");
    
    let config = PerformanceTestConfig {
        dataset_sizes: vec![20000],
        shard_sizes: vec![1000],
        thread_counts: vec![1, 2, 4, 8],
        buffer_sizes: vec![256 * 1024],
        test_iterations: 1,
        memory_limit_mb: 64,
        verbose: false,
    };
    
    let mut test_suite = PerformanceTestSuite::new(config);
    test_suite.run_thread_scalability_tests()?;
    
    let report = test_suite.generate_performance_report();
    
    // 分析线程扩展性
    let thread_results: Vec<_> = report.detailed_results.iter()
        .filter(|r| r.test_name == "thread_scalability_test")
        .collect();
    
    if !thread_results.is_empty() {
        println!("多线程扩展性分析:");
        
        let single_thread_time = thread_results.iter()
            .find(|r| r.config_params.get("thread_count") == Some(&"1".to_string()))
            .map(|r| r.export_time_ms);
        
        for result in &thread_results {
            if let Some(thread_count) = result.config_params.get("thread_count") {
                let speedup = if let Some(single_time) = single_thread_time {
                    if result.export_time_ms > 0 {
                        single_time as f64 / result.export_time_ms as f64
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                
                println!("  {} 线程: {} ms, {:.2} MB, 加速比: {:.2}x", 
                    thread_count, 
                    result.export_time_ms,
                    result.peak_memory_mb,
                    speedup);
            }
        }
        
        // 计算并行效率
        let max_threads = thread_results.iter()
            .filter_map(|r| r.config_params.get("thread_count")?.parse::<usize>().ok())
            .max()
            .unwrap_or(1);
        
        if let Some(max_thread_result) = thread_results.iter()
            .find(|r| r.config_params.get("thread_count") == Some(&max_threads.to_string())) {
            
            if let Some(single_time) = single_thread_time {
                let actual_speedup = single_time as f64 / max_thread_result.export_time_ms as f64;
                let efficiency = actual_speedup / max_threads as f64 * 100.0;
                
                println!("  📊 并行效率 ({} 线程): {:.1}%", max_threads, efficiency);
                
                if efficiency >= 70.0 {
                    println!("  ✅ 并行效率良好 (>= 70%)");
                } else if efficiency >= 50.0 {
                    println!("  ⚠️ 并行效率中等 (50-70%)");
                } else {
                    println!("  ❌ 并行效率较低 (< 50%)");
                }
            }
        }
    }
    
    Ok(())
}

/// 运行内存使用限制测试
fn run_memory_limit_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试内存使用是否在限制范围内...");
    
    let config = PerformanceTestConfig {
        dataset_sizes: vec![5000, 15000, 30000, 50000],
        shard_sizes: vec![1000],
        thread_counts: vec![4],
        buffer_sizes: vec![256 * 1024],
        test_iterations: 1,
        memory_limit_mb: 64,
        verbose: false,
    };
    
    let memory_limit_mb = config.memory_limit_mb;
    let mut test_suite = PerformanceTestSuite::new(config);
    test_suite.run_memory_usage_tests()?;
    
    let report = test_suite.generate_performance_report();
    
    // 分析内存使用
    let memory_results: Vec<_> = report.detailed_results.iter()
        .filter(|r| r.test_name == "memory_usage_test")
        .collect();
    
    if !memory_results.is_empty() {
        println!("内存使用分析:");
        
        let mut within_limit = 0;
        let total_tests = memory_results.len();
        
        for result in &memory_results {
            let status = if result.peak_memory_mb <= memory_limit_mb as f64 {
                within_limit += 1;
                "✅"
            } else {
                "❌"
            };
            
            println!("  数据集 {}: {} {:.2} MB", 
                result.dataset_size, status, result.peak_memory_mb);
        }
        
        let compliance_rate = within_limit as f64 / total_tests as f64 * 100.0;
        println!("  📊 内存限制合规率: {}/{} ({:.1}%)", 
            within_limit, total_tests, compliance_rate);
        println!("  📏 内存限制: {} MB", memory_limit_mb);
        
        if compliance_rate >= 90.0 {
            println!("  ✅ 内存使用控制良好");
        } else if compliance_rate >= 70.0 {
            println!("  ⚠️ 内存使用控制中等");
        } else {
            println!("  ❌ 内存使用控制需要改进");
        }
        
        // 分析内存增长趋势
        let mut sorted_results = memory_results.clone();
        sorted_results.sort_by_key(|r| r.dataset_size);
        
        if sorted_results.len() >= 2 {
            let first = &sorted_results[0];
            let last = &sorted_results[sorted_results.len() - 1];
            
            let data_growth = last.dataset_size as f64 / first.dataset_size as f64;
            let memory_growth = last.peak_memory_mb / first.peak_memory_mb;
            
            println!("  📈 内存增长分析:");
            println!("    数据量增长: {:.1}x", data_growth);
            println!("    内存增长: {:.1}x", memory_growth);
            
            let memory_efficiency = data_growth / memory_growth;
            if memory_efficiency >= 0.8 {
                println!("    ✅ 内存效率良好 (近线性增长)");
            } else if memory_efficiency >= 0.5 {
                println!("    ⚠️ 内存效率中等");
            } else {
                println!("    ❌ 内存效率较低 (超线性增长)");
            }
        }
    }
    
    Ok(())
}

/// 运行配置优化建议
fn run_configuration_optimization() -> Result<(), Box<dyn std::error::Error>> {
    println!("生成配置优化建议...");
    
    let optimizer = ConfigurationOptimizer::new();
    
    // 为不同目标生成配置建议
    let _speed_config = optimizer.recommend_optimal_config(OptimizationTarget::Speed);
    let _memory_config = optimizer.recommend_optimal_config(OptimizationTarget::Memory);
    let _balanced_config = optimizer.recommend_optimal_config(OptimizationTarget::Balanced);
    
    println!("配置优化建议:");
    
    println!("  🚀 速度优化配置:");
    println!("    - 分片大小: 2000 (大分片减少开销)");
    println!("    - 线程数: {} (充分利用CPU)", num_cpus::get());
    println!("    - 缓冲区: 512KB (大缓冲区减少I/O)");
    println!("    - 适用场景: 高性能服务器，CPU资源充足");
    
    println!("  💾 内存优化配置:");
    println!("    - 分片大小: 500 (小分片减少内存占用)");
    println!("    - 线程数: 2 (减少并发内存使用)");
    println!("    - 缓冲区: 64KB (小缓冲区节省内存)");
    println!("    - 适用场景: 内存受限环境，嵌入式系统");
    
    println!("  ⚖️ 平衡配置:");
    println!("    - 分片大小: 1000 (平衡性能和内存)");
    println!("    - 线程数: {} (适中的并行度)", num_cpus::get() / 2);
    println!("    - 缓冲区: 256KB (平衡I/O和内存)");
    println!("    - 适用场景: 一般应用，默认推荐");
    
    // 系统资源检测和建议
    let cpu_count = num_cpus::get();
    println!("\n🖥️ 系统资源分析:");
    println!("  CPU 核心数: {}", cpu_count);
    
    if cpu_count >= 8 {
        println!("  💡 建议: 使用速度优化配置，充分利用多核优势");
    } else if cpu_count >= 4 {
        println!("  💡 建议: 使用平衡配置，适合大多数场景");
    } else {
        println!("  💡 建议: 使用内存优化配置，减少线程竞争");
    }
    
    // 估算内存使用
    println!("\n📊 配置影响预估:");
    println!("  速度优化配置预估内存: ~80-120 MB");
    println!("  平衡配置预估内存: ~40-80 MB");
    println!("  内存优化配置预估内存: ~20-40 MB");
    
    Ok(())
}

/// 运行完整性能测试套件
fn run_comprehensive_test_suite() -> Result<(), Box<dyn std::error::Error>> {
    println!("运行完整性能测试套件...");
    
    let config = PerformanceTestConfig {
        dataset_sizes: vec![2000, 8000, 20000],
        shard_sizes: vec![500, 1000, 2000],
        thread_counts: vec![1, 2, 4],
        buffer_sizes: vec![128 * 1024, 256 * 1024, 512 * 1024],
        test_iterations: 1,
        memory_limit_mb: 64,
        verbose: false,
    };
    
    let mut test_suite = PerformanceTestSuite::new(config);
    let start_time = Instant::now();
    let report = test_suite.run_full_test_suite()?;
    let total_time = start_time.elapsed();
    
    println!("\n📊 完整测试套件结果:");
    println!("  总测试时间: {:?}", total_time);
    println!("  总测试数: {}", report.test_summary.total_tests);
    println!("  成功测试: {}", report.test_summary.successful_tests);
    println!("  失败测试: {}", report.test_summary.failed_tests);
    println!("  成功率: {:.1}%", 
        report.test_summary.successful_tests as f64 / report.test_summary.total_tests as f64 * 100.0);
    
    println!("\n📈 性能统计:");
    println!("  平均导出时间: {:.2} ms", report.performance_analysis.average_export_time_ms);
    println!("  平均内存使用: {:.2} MB", report.performance_analysis.average_memory_usage_mb);
    println!("  平均吞吐量: {:.0} 分配/秒", report.performance_analysis.average_throughput);
    println!("  内存效率分数: {:.1}%", report.performance_analysis.memory_efficiency_score);
    
    // 生成性能报告文件
    if let Ok(json_report) = serde_json::to_string_pretty(&report) {
        if let Err(e) = std::fs::write("performance_test_report.json", json_report) {
            println!("⚠️ 无法保存性能报告: {}", e);
        } else {
            println!("  📄 性能报告已保存到: performance_test_report.json");
        }
    }
    
    // 总结和建议
    println!("\n💡 测试总结:");
    if report.test_summary.successful_tests as f64 / report.test_summary.total_tests as f64 >= 0.9 {
        println!("  ✅ 系统稳定性良好");
    } else {
        println!("  ⚠️ 系统稳定性需要改进");
    }
    
    if report.performance_analysis.average_export_time_ms <= 5000.0 {
        println!("  ✅ 导出性能达到目标 (<5秒)");
    } else {
        println!("  ❌ 导出性能需要优化 (>5秒)");
    }
    
    if report.performance_analysis.memory_efficiency_score >= 70.0 {
        println!("  ✅ 内存效率良好");
    } else {
        println!("  ⚠️ 内存效率需要改进");
    }
    
    Ok(())
}