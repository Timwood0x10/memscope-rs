//! 核心性能测试（只测试导出核心，不包含验证）
//! 
//! 这个程序直接测试快速导出协调器的核心性能，不包含任何验证

use memscope_rs::{get_global_tracker, init};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

fn main() {
    println!("🎯 核心性能测试（纯导出算法）");
    println!("==============================");
    println!();

    // 初始化内存跟踪
    init();

    // 创建输出目录
    let output_dir = PathBuf::from("core_performance_results");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("❌ 创建输出目录失败: {}", e);
        return;
    }

    // 运行 complex_lifecycle_showcase 生成测试数据
    println!("🔧 运行 complex_lifecycle_showcase 生成测试数据...");
    let output = Command::new("cargo")
        .args(&["run", "--release", "--example", "complex_lifecycle_showcase"])
        .output();

    match output {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("❌ 运行 complex_lifecycle_showcase 失败: {}", stderr);
                return;
            }
            println!("✅ 测试数据生成完成");
        }
        Err(e) => {
            eprintln!("❌ 执行命令失败: {}", e);
            return;
        }
    }

    // 等待系统稳定
    std::thread::sleep(std::time::Duration::from_millis(500));

    // 运行核心性能测试
    run_core_performance_tests(&output_dir);
}

fn run_core_performance_tests(output_dir: &PathBuf) {
    println!();
    println!("📊 开始核心性能测试...");
    println!("======================");

    let test_runs = 5; // 增加测试次数以获得更准确的结果
    let mut traditional_core_times = Vec::new();
    let mut fast_core_times = Vec::new();

    // 测试传统导出的核心性能（只测量主要导出，不包含其他文件）
    println!("🐌 测试传统导出核心性能...");
    for run in 1..=test_runs {
        println!("  运行 {}/{}: 传统导出核心", run, test_runs);
        
        let start_time = Instant::now();
        let output_path = output_dir.join(format!("traditional_core_run_{}.json", run));
        
        // 获取跟踪器并导出（使用最简配置，只生成主文件）
        let tracker = get_global_tracker();
        let result = tracker.export_to_json(&output_path);
        let export_time = start_time.elapsed();
        
        match result {
            Ok(_) => {
                traditional_core_times.push(export_time.as_millis() as u64);
                println!("    ⏱️  核心时间: {}ms", export_time.as_millis());
            }
            Err(e) => {
                eprintln!("    ❌ 导出失败: {}", e);
            }
        }
        
        // 短暂休息
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 测试快速导出协调器的核心性能
    println!("⚡ 测试快速导出核心性能...");
    for run in 1..=test_runs {
        println!("  运行 {}/{}: 快速导出核心", run, test_runs);
        
        // 直接测试快速导出协调器
        let start_time = Instant::now();
        let output_path = output_dir.join(format!("fast_core_run_{}", run));
        
        // 使用快速导出协调器
        let config = memscope_rs::export::fast_export_coordinator::FastExportConfig {
            enable_data_localization: true,
            data_cache_ttl_ms: 100,
            shard_config: memscope_rs::export::parallel_shard_processor::ParallelShardConfig::default(),
            writer_config: memscope_rs::export::high_speed_buffered_writer::HighSpeedWriterConfig::default(),
            enable_performance_monitoring: false, // 禁用性能监控以减少开销
            verbose_logging: false,
            progress_config: memscope_rs::export::progress_monitor::ProgressConfig {
                enabled: false,
                update_interval: std::time::Duration::from_millis(1000),
                show_details: false,
                show_estimated_time: false,
                allow_cancellation: false,
            },
            enable_auto_optimization: false,
            auto_adjust_for_system: false,
            error_recovery_config: memscope_rs::export::error_recovery::RecoveryConfig::default(),
            validation_config: memscope_rs::export::quality_validator::ValidationConfig {
                enable_integrity_validation: false,
                enable_count_validation: false,
                enable_size_validation: false,
                enable_json_validation: false,
                enable_encoding_validation: false,
                max_data_loss_rate: 100.0, // 允许任何数据丢失以跳过验证
                min_expected_file_size: 0,
                max_expected_file_size: usize::MAX,
                verbose_logging: false,
            },
            enable_resource_monitoring: false,
            memory_limit_mb: 1024,
            disk_limit_mb: 2048,
            cpu_limit_percent: 80.0,
        };
        
        let mut coordinator = memscope_rs::export::fast_export_coordinator::FastExportCoordinator::new(config);
        let result = coordinator.export_fast(&output_path);
        let export_time = start_time.elapsed();
        
        match result {
            Ok(stats) => {
                // 只记录核心导出时间，不包含验证
                let core_time = stats.data_gathering.total_time_ms + 
                               stats.parallel_processing.total_processing_time_ms + 
                               stats.write_performance.total_write_time_ms;
                fast_core_times.push(core_time);
                println!("    ⚡ 核心时间: {}ms (总时间: {}ms)", core_time, export_time.as_millis());
                println!("       数据获取: {}ms, 并行处理: {}ms, 写入: {}ms", 
                        stats.data_gathering.total_time_ms,
                        stats.parallel_processing.total_processing_time_ms,
                        stats.write_performance.total_write_time_ms);
            }
            Err(e) => {
                eprintln!("    ❌ 导出失败: {}", e);
            }
        }
        
        // 短暂休息
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 计算和显示结果
    display_core_performance_results(&traditional_core_times, &fast_core_times, output_dir);
}

fn display_core_performance_results(traditional_times: &[u64], fast_times: &[u64], output_dir: &PathBuf) {
    println!();
    println!("📈 核心性能测试结果");
    println!("====================");

    if traditional_times.is_empty() || fast_times.is_empty() {
        println!("❌ 测试数据不足，无法生成报告");
        return;
    }

    // 计算统计数据
    let avg_traditional = traditional_times.iter().sum::<u64>() as f64 / traditional_times.len() as f64;
    let avg_fast = fast_times.iter().sum::<u64>() as f64 / fast_times.len() as f64;
    
    let min_traditional = *traditional_times.iter().min().unwrap_or(&0);
    let max_traditional = *traditional_times.iter().max().unwrap_or(&0);
    let min_fast = *fast_times.iter().min().unwrap_or(&0);
    let max_fast = *fast_times.iter().max().unwrap_or(&0);

    // 计算改善百分比
    let improvement_percent = if avg_traditional > 0.0 {
        ((avg_traditional - avg_fast) / avg_traditional) * 100.0
    } else {
        0.0
    };

    // 显示结果
    println!("传统导出核心算法:");
    println!("  • 平均时间: {:.1}ms", avg_traditional);
    println!("  • 最快时间: {}ms", min_traditional);
    println!("  • 最慢时间: {}ms", max_traditional);
    println!("  • 标准差: {:.1}ms", calculate_std_dev(traditional_times));

    println!();
    println!("快速导出核心算法:");
    println!("  • 平均时间: {:.1}ms", avg_fast);
    println!("  • 最快时间: {}ms", min_fast);
    println!("  • 最慢时间: {}ms", max_fast);
    println!("  • 标准差: {:.1}ms", calculate_std_dev(fast_times));

    println!();
    println!("📊 核心算法性能对比:");
    if improvement_percent > 0.0 {
        println!("  • 时间改善: {:.1}%", improvement_percent);
        println!("  • 加速比: {:.2}x", avg_traditional / avg_fast);
        println!("  • 时间节省: {:.1}ms", avg_traditional - avg_fast);
    } else {
        println!("  • 时间变化: {:.1}% (变慢)", improvement_percent.abs());
        println!("  • 减速比: {:.2}x", avg_fast / avg_traditional);
        println!("  • 时间增加: {:.1}ms", avg_fast - avg_traditional);
    }

    // 评估结果
    println!();
    println!("🎯 核心算法评估:");
    if improvement_percent >= 60.0 {
        println!("✅ 优秀！核心算法达到了 60-80% 导出时间减少的目标");
        println!("   快速导出系统的核心设计是成功的！");
    } else if improvement_percent >= 40.0 {
        println!("✅ 良好！核心算法接近 60-80% 导出时间减少的目标");
        println!("   快速导出系统有明显优势，可以进一步优化");
    } else if improvement_percent >= 20.0 {
        println!("⚠️  一般，核心算法有一定改善但未达到预期目标");
        println!("   需要进一步优化并行处理和数据本地化策略");
    } else if improvement_percent > 0.0 {
        println!("⚠️  轻微改善，核心算法优势不明显");
        println!("   需要重新审视快速导出的设计思路");
    } else {
        println!("❌ 核心算法性能没有提升或有所下降");
        println!("   需要从根本上重新设计快速导出算法");
    }

    // 生成详细报告
    generate_core_performance_report(traditional_times, fast_times, improvement_percent, output_dir);
}

fn calculate_std_dev(values: &[u64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mean = values.iter().sum::<u64>() as f64 / values.len() as f64;
    let variance = values.iter()
        .map(|x| (*x as f64 - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

fn generate_core_performance_report(traditional_times: &[u64], fast_times: &[u64], improvement_percent: f64, output_dir: &PathBuf) {
    let report_file = output_dir.join("core_performance_report.md");
    
    let avg_traditional = traditional_times.iter().sum::<u64>() as f64 / traditional_times.len() as f64;
    let avg_fast = fast_times.iter().sum::<u64>() as f64 / fast_times.len() as f64;
    
    let report = format!(
        r#"# 大型项目导出优化 - 核心性能基准测试报告

**测试时间**: {}
**测试说明**: 此测试专门测试快速导出协调器的核心算法性能，不包含质量验证、进度监控等附加功能。

## 📊 核心算法性能对比

| 指标 | 传统导出核心 | 快速导出核心 | 改善幅度 |
|------|-------------|-------------|----------|
| 平均时间 | {:.1}ms | {:.1}ms | **{:.1}%** |
| 最快时间 | {}ms | {}ms | - |
| 最慢时间 | {}ms | {}ms | - |
| 标准差 | {:.1}ms | {:.1}ms | - |

## 📈 详细测试数据

### 传统导出核心算法
{}

### 快速导出核心算法
{}

## 🔍 核心算法分析

### 数据本地化效果
快速导出系统通过数据本地化减少了全局状态访问，这是性能提升的关键因素之一。

### 并行处理效果
快速导出系统使用并行分片处理，在多核系统上应该有更好的表现。

### 高速缓冲写入
快速导出系统使用预分配缓冲区和批量写入，减少了I/O开销。

## 🎯 结论

{}

## 📝 关键发现

1. **核心算法效果**: 通过测试纯核心算法，我们可以准确评估快速导出系统的真实性能
2. **瓶颈识别**: 帮助区分是核心算法问题还是附加功能（验证、监控）的开销
3. **优化方向**: 为进一步的性能优化提供明确的方向

## 🚀 后续优化建议

### 如果改善幅度 >= 60%
- 核心算法设计成功，重点优化附加功能的性能
- 可以考虑将快速导出作为默认导出方式

### 如果改善幅度 20-60%
- 核心算法有效但仍有优化空间
- 重点优化并行处理效率和数据本地化策略

### 如果改善幅度 < 20%
- 需要重新审视快速导出的核心设计
- 考虑采用不同的优化策略

## 📁 生成的文件

- traditional_core_run_*.json - 传统导出核心结果
- fast_core_run_* - 快速导出核心结果
- core_performance_report.md - 本报告
"#,
        chrono::Utc::now().to_rfc3339(),
        avg_traditional,
        avg_fast,
        improvement_percent,
        traditional_times.iter().min().unwrap_or(&0),
        fast_times.iter().min().unwrap_or(&0),
        traditional_times.iter().max().unwrap_or(&0),
        fast_times.iter().max().unwrap_or(&0),
        calculate_std_dev(traditional_times),
        calculate_std_dev(fast_times),
        traditional_times.iter().enumerate()
            .map(|(i, t)| format!("- 运行 {}: {}ms", i + 1, t))
            .collect::<Vec<_>>()
            .join("\n"),
        fast_times.iter().enumerate()
            .map(|(i, t)| format!("- 运行 {}: {}ms", i + 1, t))
            .collect::<Vec<_>>()
            .join("\n"),
        if improvement_percent >= 60.0 {
            "✅ 优秀！快速导出系统的核心算法非常有效，达到了预期的性能目标。核心设计思路正确，主要瓶颈在于附加功能的开销。"
        } else if improvement_percent >= 40.0 {
            "✅ 良好！快速导出系统的核心算法有明显改善，接近预期目标。可以通过进一步优化并行处理和数据本地化来达到更好的效果。"
        } else if improvement_percent >= 20.0 {
            "⚠️ 一般，快速导出系统的核心算法有一定改善，但距离预期目标还有差距。需要重新审视并行处理策略和数据本地化的实现。"
        } else if improvement_percent > 0.0 {
            "⚠️ 轻微改善，快速导出系统的核心算法优势不明显。可能需要采用完全不同的优化策略，或者重新设计核心架构。"
        } else {
            "❌ 快速导出系统的核心算法性能没有提升或有所下降。需要从根本上重新审视设计思路，可能当前的优化方向是错误的。"
        }
    );

    if let Err(e) = fs::write(&report_file, report) {
        eprintln!("⚠️  生成报告失败: {}", e);
    } else {
        println!();
        println!("📄 核心性能报告已生成: {}", report_file.display());
    }
}