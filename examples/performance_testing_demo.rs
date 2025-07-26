//! 性能测试演示
//!
//! 这个示例展示了如何使用性能测试模块来测试和优化大型项目导出功能。
//! 包括基准测试、配置优化、内存使用测试等。

use memscope_rs::export::performance_testing::{
    PerformanceTestConfig, PerformanceTestSuite, PerformanceBenchmark,
    ConfigurationOptimizer, OptimizationTarget,
};
use memscope_rs::export::fast_export_coordinator::FastExportConfigBuilder;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 性能测试演示");
    println!("===============");
    
    // 演示 1: 快速基准测试
    println!("\n1️⃣ 快速基准测试:");
    demo_quick_benchmark()?;
    
    // 演示 2: 自定义性能测试
    println!("\n2️⃣ 自定义性能测试:");
    demo_custom_performance_test()?;
    
    // 演示 3: 配置优化建议
    println!("\n3️⃣ 配置优化建议:");
    demo_configuration_optimization()?;
    
    // 演示 4: 内存使用分析
    println!("\n4️⃣ 内存使用分析:");
    demo_memory_analysis()?;
    
    // 演示 5: 性能对比测试
    println!("\n5️⃣ 性能对比测试:");
    demo_performance_comparison()?;
    
    println!("\n✅ 性能测试演示完成!");
    
    Ok(())
}

/// 演示快速基准测试
fn demo_quick_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    println!("运行快速基准测试...");
    
    let start_time = Instant::now();
    PerformanceBenchmark::run_quick_benchmark()?;
    let duration = start_time.elapsed();
    
    println!("快速基准测试完成，耗时: {:?}", duration);
    
    Ok(())
}

/// 演示自定义性能测试
fn demo_custom_performance_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("运行自定义性能测试...");
    
    // 创建自定义测试配置
    let config = PerformanceTestConfig {
        dataset_sizes: vec![2000, 8000, 15000],
        shard_sizes: vec![800, 1200, 1600],
        thread_counts: vec![2, 4, 6],
        buffer_sizes: vec![128 * 1024, 384 * 1024],
        test_iterations: 2,
        memory_limit_mb: 48,
        verbose: true,
    };
    
    let mut test_suite = PerformanceTestSuite::new(config);
    
    // 只运行基准测试和分片优化测试
    println!("  运行基准性能测试...");
    test_suite.run_baseline_performance_tests()?;
    
    println!("  运行分片大小优化测试...");
    test_suite.run_shard_size_optimization_tests()?;
    
    let report = test_suite.generate_performance_report();
    
    println!("自定义测试完成:");
    println!("  总测试数: {}", report.test_summary.total_tests);
    println!("  成功率: {:.1}%", 
        report.test_summary.successful_tests as f64 / report.test_summary.total_tests as f64 * 100.0);
    
    Ok(())
}

/// 演示配置优化建议
fn demo_configuration_optimization() -> Result<(), Box<dyn std::error::Error>> {
    println!("生成配置优化建议...");
    
    let optimizer = ConfigurationOptimizer::new();
    
    // 为不同目标生成配置建议
    let speed_config = optimizer.recommend_optimal_config(OptimizationTarget::Speed);
    let memory_config = optimizer.recommend_optimal_config(OptimizationTarget::Memory);
    let balanced_config = optimizer.recommend_optimal_config(OptimizationTarget::Balanced);
    
    println!("配置建议:");
    println!("  速度优化配置:");
    println!("    - 分片大小: 2000");
    println!("    - 线程数: {} (所有CPU核心)", num_cpus::get());
    println!("    - 缓冲区: 512KB");
    
    println!("  内存优化配置:");
    println!("    - 分片大小: 500");
    println!("    - 线程数: 2");
    println!("    - 缓冲区: 64KB");
    
    println!("  平衡配置:");
    println!("    - 分片大小: 1000");
    println!("    - 线程数: {} (一半CPU核心)", num_cpus::get() / 2);
    println!("    - 缓冲区: 256KB");
    
    Ok(())
}

/// 演示内存使用分析
fn demo_memory_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("运行内存使用分析...");
    
    let config = PerformanceTestConfig {
        dataset_sizes: vec![5000, 15000, 30000],
        shard_sizes: vec![1000],
        thread_counts: vec![4],
        buffer_sizes: vec![256 * 1024],
        test_iterations: 1,
        memory_limit_mb: 64,
        verbose: false,
    };
    
    let mut test_suite = PerformanceTestSuite::new(config);
    test_suite.run_memory_usage_tests()?;
    
    let report = test_suite.generate_performance_report();
    
    println!("内存使用分析结果:");
    println!("  平均内存使用: {:.2} MB", report.performance_analysis.average_memory_usage_mb);
    println!("  内存效率分数: {:.1}%", report.performance_analysis.memory_efficiency_score);
    
    let memory_results: Vec<_> = report.detailed_results.iter()
        .filter(|r| r.test_name == "memory_usage_test")
        .collect();
    
    for result in memory_results {
        let status = if result.success { "✅" } else { "❌" };
        println!("  数据集 {}: {} ({:.2} MB)", 
            result.dataset_size, status, result.peak_memory_mb);
    }
    
    Ok(())
}

/// 演示性能对比测试
fn demo_performance_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("运行性能对比测试...");
    
    let config = PerformanceTestConfig {
        dataset_sizes: vec![10000],
        shard_sizes: vec![1000],
        thread_counts: vec![4],
        buffer_sizes: vec![256 * 1024],
        test_iterations: 1,
        memory_limit_mb: 64,
        verbose: false,
    };
    
    let mut test_suite = PerformanceTestSuite::new(config);
    test_suite.run_before_after_comparison_tests()?;
    
    let report = test_suite.generate_performance_report();
    
    // 找到对比结果
    let traditional_result = report.detailed_results.iter()
        .find(|r| r.test_name == "traditional_export");
    let optimized_result = report.detailed_results.iter()
        .find(|r| r.test_name == "optimized_export");
    
    if let (Some(traditional), Some(optimized)) = (traditional_result, optimized_result) {
        let time_improvement = traditional.export_time_ms as f64 / optimized.export_time_ms as f64;
        let throughput_improvement = optimized.throughput_allocations_per_sec / traditional.throughput_allocations_per_sec;
        
        println!("性能对比结果:");
        println!("  传统导出:");
        println!("    - 时间: {} ms", traditional.export_time_ms);
        println!("    - 吞吐量: {:.0} 分配/秒", traditional.throughput_allocations_per_sec);
        println!("    - 内存: {:.2} MB", traditional.peak_memory_mb);
        
        println!("  优化导出:");
        println!("    - 时间: {} ms", optimized.export_time_ms);
        println!("    - 吞吐量: {:.0} 分配/秒", optimized.throughput_allocations_per_sec);
        println!("    - 内存: {:.2} MB", optimized.peak_memory_mb);
        
        println!("  性能提升:");
        println!("    - 时间提升: {:.2}x", time_improvement);
        println!("    - 吞吐量提升: {:.2}x", throughput_improvement);
        
        if time_improvement >= 2.0 {
            println!("    🎯 达到预期性能提升目标 (>2x)!");
        } else {
            println!("    ⚠️ 未达到预期性能提升目标 (>2x)");
        }
    }
    
    Ok(())
}

/// 创建测试数据的辅助函数
fn create_test_allocations(count: usize) {
    use std::alloc::{alloc, dealloc, Layout};
    
    let mut allocations = Vec::new();
    
    for i in 0..count {
        let size = 64 + (i % 1000);
        let layout = Layout::from_size_align(size, 8).unwrap();
        
        unsafe {
            let ptr = alloc(layout);
            if !ptr.is_null() {
                allocations.push((ptr, layout));
                
                // 模拟一些释放
                if i % 3 == 0 && !allocations.is_empty() {
                    let (old_ptr, old_layout) = allocations.remove(0);
                    dealloc(old_ptr, old_layout);
                }
            }
        }
    }
    
    // 清理剩余分配
    for (ptr, layout) in allocations {
        unsafe {
            dealloc(ptr, layout);
        }
    }
}