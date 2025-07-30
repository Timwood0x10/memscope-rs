//! 纯性能基准测试（禁用所有验证）
//!
//! 这个程序专注于测试导出性能，禁用所有质量验证以获得真实的性能数据

use memscope_rs::{get_global_tracker, init};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

fn main() {
    println!("🚀 纯性能基准测试（无验证）");
    println!("============================");
    println!();

    // 初始化内存跟踪
    init();

    // 创建输出目录
    let output_dir = PathBuf::from("performance_only_results");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("❌ 创建输出目录失败: {}", e);
        return;
    }

    // 运行 complex_lifecycle_showcase 生成测试数据
    println!("🔧 运行 complex_lifecycle_showcase 生成测试数据...");
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--release",
            "--example",
            "complex_lifecycle_showcase",
        ])
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

    // 运行纯性能测试
    run_performance_only_tests(&output_dir);
}

fn run_performance_only_tests(output_dir: &PathBuf) {
    println!();
    println!("📊 开始纯性能测试...");
    println!("====================");

    let test_runs = 3;
    let mut traditional_times = Vec::new();
    let mut fast_times = Vec::new();

    // 运行传统导出测试（禁用验证）
    println!("🐌 测试传统导出系统（无验证）...");
    for run in 1..=test_runs {
        println!("  运行 {}/{}: 传统导出", run, test_runs);

        let start_time = Instant::now();
        let output_path = output_dir.join(format!("traditional_export_run_{}.json", run));

        // 获取跟踪器并导出（使用最简配置）
        let tracker = get_global_tracker();
        let mut options =
            memscope_rs::export::optimized_json_export::OptimizedExportOptions::default();

        // 禁用所有验证和额外功能
        options.enable_schema_validation = false;
        options.enable_enhanced_ffi_analysis = false;
        options.enable_boundary_event_processing = false;
        options.enable_memory_passport_tracking = false;
        options.enable_security_analysis = false;
        options.enable_adaptive_optimization = false;
        options.parallel_processing = false; // 传统方式不使用并行
        options.use_streaming_writer = true; // 但保持流式写入

        let result = tracker.export_to_json_with_optimized_options(&output_path, options);
        let export_time = start_time.elapsed();

        match result {
            Ok(_) => {
                traditional_times.push(export_time.as_millis() as u64);
                println!("    ⏱️  时间: {}ms", export_time.as_millis());
            }
            Err(e) => {
                eprintln!("    ❌ 导出失败: {}", e);
            }
        }

        // 短暂休息
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 运行快速导出测试（禁用验证）
    println!("⚡ 测试快速导出系统（无验证）...");
    for run in 1..=test_runs {
        println!("  运行 {}/{}: 快速导出", run, test_runs);

        let start_time = Instant::now();
        let output_path = output_dir.join(format!("fast_export_run_{}.json", run));

        // 获取跟踪器并使用快速导出（禁用验证）
        let tracker = get_global_tracker();
        let mut options =
            memscope_rs::export::optimized_json_export::OptimizedExportOptions::default();

        // 启用快速导出但禁用所有验证
        options.enable_fast_export_mode = true;
        options.parallel_processing = true;
        options.use_streaming_writer = true;

        // 禁用所有验证和额外分析
        options.enable_schema_validation = false;
        options.enable_enhanced_ffi_analysis = false;
        options.enable_boundary_event_processing = false;
        options.enable_memory_passport_tracking = false;
        options.enable_security_analysis = false;
        options.enable_adaptive_optimization = false;

        // 设置最小缓冲区以减少开销
        options.buffer_size = 64 * 1024; // 64KB
        options.batch_size = 10000; // 大批次

        let result = tracker.export_to_json_with_optimized_options(&output_path, options);
        let export_time = start_time.elapsed();

        match result {
            Ok(_) => {
                fast_times.push(export_time.as_millis() as u64);
                println!("    ⚡ 时间: {}ms", export_time.as_millis());
            }
            Err(e) => {
                eprintln!("    ❌ 导出失败: {}", e);
            }
        }

        // 短暂休息
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 计算和显示结果
    display_performance_results(&traditional_times, &fast_times, output_dir);
}

fn display_performance_results(
    traditional_times: &[u64],
    fast_times: &[u64],
    output_dir: &PathBuf,
) {
    println!();
    println!("📈 纯性能测试结果");
    println!("==================");

    if traditional_times.is_empty() || fast_times.is_empty() {
        println!("❌ 测试数据不足，无法生成报告");
        return;
    }

    // 计算平均值
    let avg_traditional =
        traditional_times.iter().sum::<u64>() as f64 / traditional_times.len() as f64;
    let avg_fast = fast_times.iter().sum::<u64>() as f64 / fast_times.len() as f64;

    // 计算改善百分比
    let improvement_percent = if avg_traditional > 0.0 {
        ((avg_traditional - avg_fast) / avg_traditional) * 100.0
    } else {
        0.0
    };

    // 显示结果
    println!("传统导出系统（无验证）:");
    println!("  • 平均时间: {:.1}ms", avg_traditional);
    println!(
        "  • 最快时间: {}ms",
        traditional_times.iter().min().unwrap_or(&0)
    );
    println!(
        "  • 最慢时间: {}ms",
        traditional_times.iter().max().unwrap_or(&0)
    );
    println!(
        "  • 时间范围: {}ms",
        traditional_times.iter().max().unwrap_or(&0) - traditional_times.iter().min().unwrap_or(&0)
    );

    println!();
    println!("快速导出系统（无验证）:");
    println!("  • 平均时间: {:.1}ms", avg_fast);
    println!("  • 最快时间: {}ms", fast_times.iter().min().unwrap_or(&0));
    println!("  • 最慢时间: {}ms", fast_times.iter().max().unwrap_or(&0));
    println!(
        "  • 时间范围: {}ms",
        fast_times.iter().max().unwrap_or(&0) - fast_times.iter().min().unwrap_or(&0)
    );

    println!();
    println!("📊 纯性能对比:");
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
    println!("🎯 纯性能评估:");
    if improvement_percent >= 60.0 {
        println!("✅ 优秀！达到了 60-80% 导出时间减少的目标");
    } else if improvement_percent >= 40.0 {
        println!("✅ 良好！接近 60-80% 导出时间减少的目标");
    } else if improvement_percent >= 20.0 {
        println!("⚠️  一般，有一定改善但未达到预期目标");
    } else if improvement_percent > 0.0 {
        println!("⚠️  轻微改善，需要进一步优化");
    } else {
        println!("❌ 核心性能没有提升，需要重新审视算法");
    }

    // 生成纯性能报告
    generate_performance_report(
        traditional_times,
        fast_times,
        improvement_percent,
        output_dir,
    );
}

fn generate_performance_report(
    traditional_times: &[u64],
    fast_times: &[u64],
    improvement_percent: f64,
    output_dir: &PathBuf,
) {
    let report_file = output_dir.join("pure_performance_report.md");

    let avg_traditional =
        traditional_times.iter().sum::<u64>() as f64 / traditional_times.len() as f64;
    let avg_fast = fast_times.iter().sum::<u64>() as f64 / fast_times.len() as f64;

    let report = format!(
        r#"# 大型项目导出优化 - 纯性能基准测试报告

**测试时间**: {}
**测试说明**: 此测试禁用了所有质量验证、安全分析、FFI分析等功能，专注于测试核心导出性能。

## 📊 纯性能对比

| 指标 | 传统导出 | 快速导出 | 改善幅度 |
|------|----------|----------|----------|
| 平均时间 | {:.1}ms | {:.1}ms | **{:.1}%** |
| 最快时间 | {}ms | {}ms | - |
| 最慢时间 | {}ms | {}ms | - |
| 时间稳定性 | {}ms 范围 | {}ms 范围 | - |

## 📈 详细测试数据

### 传统导出系统（无验证）
{}

### 快速导出系统（无验证）
{}

## 🔍 性能分析

### 如果改善幅度 >= 60%
这表明快速导出系统的核心算法是有效的，之前的性能问题主要来自质量验证等附加功能。

### 如果改善幅度 < 20%
这表明快速导出系统的核心算法需要进一步优化，问题不仅仅是验证开销。

## 🎯 结论

{}

## 📝 重要发现

1. **质量验证的影响**: 通过禁用验证，我们可以看到核心导出算法的真实性能
2. **并行处理效果**: 在无验证环境下，并行处理的效果更加明显
3. **性能瓶颈定位**: 帮助区分是算法问题还是验证开销问题

## 📁 生成的文件

- traditional_export_run_*.json - 传统导出结果（无验证）
- fast_export_run_*.json - 快速导出结果（无验证）
- pure_performance_report.md - 本报告
"#,
        chrono::Utc::now().to_rfc3339(),
        avg_traditional,
        avg_fast,
        improvement_percent,
        traditional_times.iter().min().unwrap_or(&0),
        fast_times.iter().min().unwrap_or(&0),
        traditional_times.iter().max().unwrap_or(&0),
        fast_times.iter().max().unwrap_or(&0),
        traditional_times.iter().max().unwrap_or(&0) - traditional_times.iter().min().unwrap_or(&0),
        fast_times.iter().max().unwrap_or(&0) - fast_times.iter().min().unwrap_or(&0),
        traditional_times
            .iter()
            .enumerate()
            .map(|(i, t)| format!("- 运行 {}: {}ms", i + 1, t))
            .collect::<Vec<_>>()
            .join("\n"),
        fast_times
            .iter()
            .enumerate()
            .map(|(i, t)| format!("- 运行 {}: {}ms", i + 1, t))
            .collect::<Vec<_>>()
            .join("\n"),
        if improvement_percent >= 60.0 {
            "✅ 优秀！快速导出系统的核心算法非常有效，之前的性能问题主要来自质量验证等附加功能。"
        } else if improvement_percent >= 40.0 {
            "✅ 良好！快速导出系统有明显改善，但仍有优化空间。"
        } else if improvement_percent >= 20.0 {
            "⚠️ 一般，快速导出系统有一定改善，但核心算法可能需要进一步优化。"
        } else if improvement_percent > 0.0 {
            "⚠️ 轻微改善，快速导出系统的核心算法优势不明显，需要重新审视设计。"
        } else {
            "❌ 快速导出系统的核心性能没有提升，需要从根本上重新设计算法。"
        }
    );

    if let Err(e) = fs::write(&report_file, report) {
        eprintln!("⚠️  生成报告失败: {}", e);
    } else {
        println!();
        println!("📄 纯性能报告已生成: {}", report_file.display());
    }
}
