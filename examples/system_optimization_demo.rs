//! 系统优化器演示
//!
//! 这个示例展示了如何使用系统优化器进行配置优化、性能诊断和对比分析。

use memscope_rs::export::system_optimizer::{
    SystemOptimizer, ConfigurationValidationResult,
};
use memscope_rs::export::performance_testing::OptimizationTarget;
use memscope_rs::export::performance_comparison::{
    PerformanceComparator, TestConfiguration, EnvironmentInfo,
};
use memscope_rs::export::performance_testing::PerformanceTestResult;
use memscope_rs::export::fast_export_coordinator::FastExportConfigBuilder;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 系统优化器演示");
    println!("================");
    
    // 演示 1: 系统资源检测
    println!("\n1️⃣ 系统资源检测:");
    demo_system_resource_detection()?;
    
    // 演示 2: 配置优化建议
    println!("\n2️⃣ 配置优化建议:");
    demo_configuration_recommendations()?;
    
    // 演示 3: 配置验证
    println!("\n3️⃣ 配置验证:");
    demo_configuration_validation()?;
    
    // 演示 4: 性能诊断
    println!("\n4️⃣ 性能诊断:");
    demo_performance_diagnosis()?;
    
    // 演示 5: 性能对比分析
    println!("\n5️⃣ 性能对比分析:");
    demo_performance_comparison()?;
    
    // 演示 6: 自动化优化流程
    println!("\n6️⃣ 自动化优化流程:");
    demo_automated_optimization()?;
    
    println!("\n✅ 系统优化器演示完成!");
    
    Ok(())
}

/// 演示系统资源检测
fn demo_system_resource_detection() -> Result<(), Box<dyn std::error::Error>> {
    println!("检测系统资源...");
    
    let optimizer = SystemOptimizer::new()?;
    let resources = optimizer.get_system_resources();
    
    println!("系统资源信息:");
    println!("  CPU 核心数: {}", resources.cpu_cores);
    println!("  可用内存: {} MB", resources.available_memory_mb);
    println!("  系统负载: {:.2}", resources.system_load);
    println!("  磁盘空间: {} MB", resources.disk_space_mb);
    println!("  系统类型: {:?}", resources.system_type);
    
    // 分析系统特征
    let system_analysis = analyze_system_characteristics(resources);
    println!("\n系统分析:");
    for analysis in system_analysis {
        println!("  • {}", analysis);
    }
    
    Ok(())
}

/// 分析系统特征
fn analyze_system_characteristics(resources: &memscope_rs::export::system_optimizer::SystemResources) -> Vec<String> {
    let mut analysis = Vec::new();
    
    // CPU 分析
    match resources.cpu_cores {
        cores if cores >= 16 => analysis.push("高性能多核系统，适合大规模并行处理".to_string()),
        cores if cores >= 8 => analysis.push("中高端多核系统，支持良好的并行性能".to_string()),
        cores if cores >= 4 => analysis.push("标准多核系统，适合中等规模并行处理".to_string()),
        cores if cores >= 2 => analysis.push("双核系统，并行处理能力有限".to_string()),
        _ => analysis.push("单核系统，不建议使用并行处理".to_string()),
    }
    
    // 内存分析
    match resources.available_memory_mb {
        mem if mem >= 16384 => analysis.push("大内存系统，可以使用大缓冲区和分片".to_string()),
        mem if mem >= 8192 => analysis.push("充足内存，支持中等规模的缓冲区配置".to_string()),
        mem if mem >= 4096 => analysis.push("标准内存配置，需要平衡内存使用".to_string()),
        mem if mem >= 2048 => analysis.push("内存较少，建议使用小缓冲区和分片".to_string()),
        _ => analysis.push("低内存系统，需要严格控制内存使用".to_string()),
    }
    
    // 负载分析
    match resources.system_load {
        load if load < 0.5 => analysis.push("系统负载很低，可以使用最大性能配置".to_string()),
        load if load < 1.0 => analysis.push("系统负载正常，适合标准性能配置".to_string()),
        load if load < 2.0 => analysis.push("系统负载较高，建议减少并行度".to_string()),
        _ => analysis.push("系统负载很高，建议使用保守配置".to_string()),
    }
    
    analysis
}

/// 演示配置优化建议
fn demo_configuration_recommendations() -> Result<(), Box<dyn std::error::Error>> {
    println!("生成配置优化建议...");
    
    let optimizer = SystemOptimizer::new()?;
    
    // 为不同优化目标生成建议
    let targets = [
        (OptimizationTarget::Speed, "速度优化"),
        (OptimizationTarget::Memory, "内存优化"),
        (OptimizationTarget::Balanced, "平衡优化"),
    ];
    
    for (target, name) in &targets {
        println!("\n{}配置建议:", name);
        
        let recommendation = optimizer.generate_configuration_recommendation(*target, Some(15000));
        
        println!("  分片大小: {}", recommendation.recommended_shard_size);
        println!("  线程数: {}", recommendation.recommended_thread_count);
        println!("  缓冲区大小: {} KB", recommendation.recommended_buffer_size / 1024);
        println!("  预期性能提升: {:.2}x", recommendation.expected_performance_gain);
        println!("  预期内存使用: {:.1} MB", recommendation.expected_memory_usage_mb);
        println!("  配置置信度: {:.1}%", recommendation.confidence * 100.0);
        
        println!("  建议原因:");
        for reason in &recommendation.reasoning {
            println!("    • {}", reason);
        }
    }
    
    Ok(())
}

/// 演示配置验证
fn demo_configuration_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("验证不同配置...");
    
    let optimizer = SystemOptimizer::new()?;
    
    // 测试不同的配置
    let test_configs = [
        ("高性能配置", FastExportConfigBuilder::new()
            .shard_size(3000)
            .max_threads(Some(8))
            .buffer_size(1024 * 1024)),
        ("内存优化配置", FastExportConfigBuilder::new()
            .shard_size(500)
            .max_threads(Some(2))
            .buffer_size(64 * 1024)),
        ("极端配置", FastExportConfigBuilder::new()
            .shard_size(10000)
            .max_threads(Some(32))
            .buffer_size(8 * 1024 * 1024)),
    ];
    
    for (name, config_builder) in &test_configs {
        println!("\n验证{}:", name);
        
        let validation_result = optimizer.validate_configuration(config_builder);
        
        print_validation_result(&validation_result);
    }
    
    Ok(())
}

/// 打印验证结果
fn print_validation_result(result: &ConfigurationValidationResult) {
    if result.is_valid {
        println!("  ✅ 配置有效");
    } else {
        println!("  ❌ 配置无效");
    }
    
    if !result.errors.is_empty() {
        println!("  错误:");
        for error in &result.errors {
            println!("    • {}", error);
        }
    }
    
    if !result.warnings.is_empty() {
        println!("  警告:");
        for warning in &result.warnings {
            println!("    • {}", warning);
        }
    }
    
    if !result.suggestions.is_empty() {
        println!("  建议:");
        for suggestion in &result.suggestions {
            println!("    • {}", suggestion);
        }
    }
    
    let impact = &result.estimated_performance_impact;
    println!("  性能影响评估:");
    println!("    性能评分: {}/10", impact.performance_score);
    println!("    内存效率: {}/10", impact.memory_efficiency);
    println!("    稳定性: {}/10", impact.stability_score);
    println!("    总体评分: {}/10", impact.overall_score);
}

/// 演示性能诊断
fn demo_performance_diagnosis() -> Result<(), Box<dyn std::error::Error>> {
    println!("运行性能诊断...");
    
    let optimizer = SystemOptimizer::new()?;
    let diagnosis = optimizer.diagnose_performance();
    
    println!("诊断结果:");
    println!("  系统健康评分: {}/100", diagnosis.health_score);
    
    println!("  系统资源状态:");
    let status = &diagnosis.system_status;
    println!("    CPU 使用率: {:.1}%", status.cpu_usage_percent);
    println!("    内存使用率: {:.1}%", status.memory_usage_percent);
    println!("    磁盘使用率: {:.1}%", status.disk_usage_percent);
    println!("    负载状态: {:?}", status.load_status);
    
    if !diagnosis.bottlenecks.is_empty() {
        println!("  性能瓶颈:");
        for bottleneck in &diagnosis.bottlenecks {
            println!("    {} (严重程度: {}/10)", bottleneck.description, bottleneck.severity);
            println!("      影响: {}", bottleneck.impact);
            println!("      建议解决方案:");
            for solution in &bottleneck.suggested_solutions {
                println!("        • {}", solution);
            }
        }
    } else {
        println!("  ✅ 未发现明显的性能瓶颈");
    }
    
    if !diagnosis.optimization_suggestions.is_empty() {
        println!("  优化建议:");
        for suggestion in &diagnosis.optimization_suggestions {
            println!("    {} (优先级: {}/10)", suggestion.title, suggestion.priority);
            println!("      描述: {}", suggestion.description);
            println!("      预期效果: {}", suggestion.expected_impact);
            println!("      实施难度: {}/10", suggestion.implementation_difficulty);
        }
    }
    
    Ok(())
}

/// 演示性能对比分析
fn demo_performance_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("运行性能对比分析...");
    
    // 创建模拟的测试结果
    let baseline_results = create_mock_baseline_results();
    let optimized_results = create_mock_optimized_results();
    
    let mut comparator = PerformanceComparator::new();
    
    // 添加测试结果
    for result in baseline_results {
        comparator.add_baseline_result(result);
    }
    
    for result in optimized_results {
        comparator.add_optimized_result(result);
    }
    
    // 设置测试配置
    let test_config = TestConfiguration {
        dataset_sizes: vec![1000, 5000, 10000, 20000],
        iterations: 3,
        environment_info: EnvironmentInfo::default(),
    };
    comparator.set_test_configuration(test_config);
    
    // 生成对比报告
    let report = comparator.generate_comparison_report()?;
    
    // 打印报告
    comparator.print_comparison_report(&report);
    
    Ok(())
}

/// 创建模拟的基准测试结果
fn create_mock_baseline_results() -> Vec<PerformanceTestResult> {
    vec![
        PerformanceTestResult {
            test_name: "baseline_1000".to_string(),
            dataset_size: 1000,
            config_params: std::collections::HashMap::new(),
            export_time_ms: 150,
            peak_memory_mb: 25.0,
            throughput_allocations_per_sec: 6666.0,
            output_file_size_bytes: 50000,
            success: true,
            error_message: None,
        },
        PerformanceTestResult {
            test_name: "baseline_5000".to_string(),
            dataset_size: 5000,
            config_params: std::collections::HashMap::new(),
            export_time_ms: 750,
            peak_memory_mb: 35.0,
            throughput_allocations_per_sec: 6666.0,
            output_file_size_bytes: 250000,
            success: true,
            error_message: None,
        },
        PerformanceTestResult {
            test_name: "baseline_10000".to_string(),
            dataset_size: 10000,
            config_params: std::collections::HashMap::new(),
            export_time_ms: 1500,
            peak_memory_mb: 45.0,
            throughput_allocations_per_sec: 6666.0,
            output_file_size_bytes: 500000,
            success: true,
            error_message: None,
        },
        PerformanceTestResult {
            test_name: "baseline_20000".to_string(),
            dataset_size: 20000,
            config_params: std::collections::HashMap::new(),
            export_time_ms: 3000,
            peak_memory_mb: 55.0,
            throughput_allocations_per_sec: 6666.0,
            output_file_size_bytes: 1000000,
            success: true,
            error_message: None,
        },
    ]
}

/// 创建模拟的优化后测试结果
fn create_mock_optimized_results() -> Vec<PerformanceTestResult> {
    vec![
        PerformanceTestResult {
            test_name: "optimized_1000".to_string(),
            dataset_size: 1000,
            config_params: std::collections::HashMap::new(),
            export_time_ms: 45,
            peak_memory_mb: 20.0,
            throughput_allocations_per_sec: 22222.0,
            output_file_size_bytes: 50000,
            success: true,
            error_message: None,
        },
        PerformanceTestResult {
            test_name: "optimized_5000".to_string(),
            dataset_size: 5000,
            config_params: std::collections::HashMap::new(),
            export_time_ms: 180,
            peak_memory_mb: 25.0,
            throughput_allocations_per_sec: 27777.0,
            output_file_size_bytes: 250000,
            success: true,
            error_message: None,
        },
        PerformanceTestResult {
            test_name: "optimized_10000".to_string(),
            dataset_size: 10000,
            config_params: std::collections::HashMap::new(),
            export_time_ms: 300,
            peak_memory_mb: 30.0,
            throughput_allocations_per_sec: 33333.0,
            output_file_size_bytes: 500000,
            success: true,
            error_message: None,
        },
        PerformanceTestResult {
            test_name: "optimized_20000".to_string(),
            dataset_size: 20000,
            config_params: std::collections::HashMap::new(),
            export_time_ms: 500,
            peak_memory_mb: 35.0,
            throughput_allocations_per_sec: 40000.0,
            output_file_size_bytes: 1000000,
            success: true,
            error_message: None,
        },
    ]
}

/// 演示自动化优化流程
fn demo_automated_optimization() -> Result<(), Box<dyn std::error::Error>> {
    println!("运行自动化优化流程...");
    
    let mut optimizer = SystemOptimizer::new()?;
    
    // 步骤 1: 系统诊断
    println!("\n步骤 1: 系统诊断");
    let diagnosis = optimizer.diagnose_performance();
    println!("  系统健康评分: {}/100", diagnosis.health_score);
    
    // 步骤 2: 确定优化目标
    println!("\n步骤 2: 确定优化目标");
    let optimization_target = determine_optimization_target(&diagnosis);
    println!("  选择的优化目标: {:?}", optimization_target);
    
    // 步骤 3: 生成配置建议
    println!("\n步骤 3: 生成配置建议");
    let recommendation = optimizer.generate_configuration_recommendation(optimization_target, Some(15000));
    println!("  推荐分片大小: {}", recommendation.recommended_shard_size);
    println!("  推荐线程数: {}", recommendation.recommended_thread_count);
    println!("  推荐缓冲区大小: {} KB", recommendation.recommended_buffer_size / 1024);
    println!("  配置置信度: {:.1}%", recommendation.confidence * 100.0);
    
    // 步骤 4: 配置验证
    println!("\n步骤 4: 配置验证");
    let config = FastExportConfigBuilder::new()
        .shard_size(recommendation.recommended_shard_size)
        .max_threads(Some(recommendation.recommended_thread_count))
        .buffer_size(recommendation.recommended_buffer_size);
    
    let validation_result = optimizer.validate_configuration(&config);
    if validation_result.is_valid {
        println!("  ✅ 配置验证通过");
    } else {
        println!("  ❌ 配置验证失败，需要调整");
        for error in &validation_result.errors {
            println!("    • {}", error);
        }
    }
    
    // 步骤 5: 性能预测
    println!("\n步骤 5: 性能预测");
    println!("  预期性能提升: {:.2}x", recommendation.expected_performance_gain);
    println!("  预期内存使用: {:.1} MB", recommendation.expected_memory_usage_mb);
    
    // 步骤 6: 生成最终配置
    println!("\n步骤 6: 生成最终配置");
    let final_config = config.build();
    println!("  ✅ 自动化优化配置生成完成");
    
    // 步骤 7: 配置应用建议
    println!("\n步骤 7: 配置应用建议");
    print_configuration_usage_example(&recommendation);
    
    Ok(())
}

/// 确定优化目标
fn determine_optimization_target(diagnosis: &memscope_rs::export::system_optimizer::PerformanceDiagnosis) -> OptimizationTarget {
    // 基于系统诊断结果确定最佳优化目标
    if diagnosis.health_score >= 80 {
        // 系统健康，优先考虑速度
        OptimizationTarget::Speed
    } else if diagnosis.system_status.memory_usage_percent > 80.0 {
        // 内存使用率高，优先考虑内存优化
        OptimizationTarget::Memory
    } else {
        // 其他情况使用平衡配置
        OptimizationTarget::Balanced
    }
}

/// 打印配置使用示例
fn print_configuration_usage_example(recommendation: &memscope_rs::export::system_optimizer::ConfigurationRecommendation) {
    println!("配置使用示例:");
    println!("```rust");
    println!("use memscope_rs::export::fast_export_coordinator::{{FastExportCoordinator, FastExportConfigBuilder}};");
    println!();
    println!("let config = FastExportConfigBuilder::new()");
    println!("    .shard_size({})", recommendation.recommended_shard_size);
    println!("    .max_threads(Some({}))", recommendation.recommended_thread_count);
    println!("    .buffer_size({})", recommendation.recommended_buffer_size);
    println!("    .performance_monitoring(true)");
    println!("    .build();");
    println!();
    println!("let mut coordinator = FastExportCoordinator::new(config);");
    println!("let result = coordinator.export_fast(\"output_path\")?;");
    println!("```");
    
    println!("\n预期效果:");
    println!("  • 性能提升: {:.2}x", recommendation.expected_performance_gain);
    println!("  • 内存使用: {:.1} MB", recommendation.expected_memory_usage_mb);
    println!("  • 配置可靠性: {:.1}%", recommendation.confidence * 100.0);
}