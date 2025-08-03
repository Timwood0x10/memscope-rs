//! 简化的性能基准测试
//!
//! 这个程序运行简化的性能基准测试，对比传统导出和快速导出的性能

use memscope_rs::{get_global_tracker, init};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

fn main() {
    tracing::info!("🚀 大型项目导出优化 - 简化性能基准测试");
    tracing::info!("=========================================");
    tracing::info!("");

    // 初始化内存跟踪
    init();

    // 创建输出目录
    let output_dir = PathBuf::from("benchmark_results");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        tracing::error!("❌ 创建输出目录失败: {}", e);
        return;
    }

    // 运行 complex_lifecycle_showcase 生成测试数据
    tracing::info!("🔧 运行 complex_lifecycle_showcase 生成测试数据...");
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
                tracing::error!("❌ 运行 complex_lifecycle_showcase 失败: {}", stderr);
                return;
            }
            tracing::info!("✅ 测试数据生成完成");
        }
        Err(e) => {
            tracing::error!("❌ 执行命令失败: {}", e);
            return;
        }
    }

    // 等待系统稳定
    std::thread::sleep(std::time::Duration::from_millis(500));

    // 运行基准测试
    run_benchmark_tests(&output_dir);
}

fn run_benchmark_tests(output_dir: &PathBuf) {
    tracing::info!("");
    tracing::info!("📊 开始基准测试...");
    tracing::info!("==================");

    let test_runs = 3;
    let mut traditional_times = Vec::new();
    let mut fast_times = Vec::new();

    // 运行传统导出测试
    tracing::info!("🐌 测试传统导出系统...");
    for run in 1..=test_runs {
        tracing::info!("  运行 {}/{}: 传统导出", run, test_runs);

        let start_time = Instant::now();
        let output_path = output_dir.join(format!("traditional_export_run_{}.json", run));

        // 获取跟踪器并导出
        let tracker = get_global_tracker();
        let result = tracker.export_to_json(&output_path);
        let export_time = start_time.elapsed();

        match result {
            Ok(_) => {
                traditional_times.push(export_time.as_millis() as u64);
                tracing::info!("    ⏱️  时间: {}ms", export_time.as_millis());
            }
            Err(e) => {
                tracing::error!("    ❌ 导出失败: {}", e);
            }
        }

        // 短暂休息
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 运行快速导出测试（使用优化选项）
    tracing::info!("⚡ 测试快速导出系统...");
    for run in 1..=test_runs {
        tracing::info!("  运行 {}/{}: 快速导出", run, test_runs);

        let start_time = Instant::now();
        let output_path = output_dir.join(format!("fast_export_run_{}.json", run));

        // 获取跟踪器并使用优化导出
        let tracker = get_global_tracker();
        let mut options =
            memscope_rs::export::optimized_json_export::OptimizedExportOptions::default();
        options.parallel_processing = true; // 启用并行处理
        options.enable_fast_export_mode = true; // 启用快速导出模式
        options.enable_schema_validation = false; // 禁用模式验证以提高性能

        let result = tracker.export_to_json_with_optimized_options(&output_path, options);
        let export_time = start_time.elapsed();

        match result {
            Ok(_) => {
                fast_times.push(export_time.as_millis() as u64);
                tracing::info!("    ⚡ 时间: {}ms", export_time.as_millis());
            }
            Err(e) => {
                tracing::error!("    ❌ 导出失败: {}", e);
            }
        }

        // 短暂休息
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 计算和显示结果
    display_results(&traditional_times, &fast_times, output_dir);
}

fn display_results(traditional_times: &[u64], fast_times: &[u64], output_dir: &PathBuf) {
    tracing::info!("");
    tracing::info!("📈 基准测试结果");
    tracing::info!("================");

    if traditional_times.is_empty() || fast_times.is_empty() {
        tracing::info!("❌ 测试数据不足，无法生成报告");
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
    tracing::info!("传统导出系统:");
    tracing::info!("  • 平均时间: {:.1}ms", avg_traditional);
    tracing::info!(
        "  • 最快时间: {}ms",
        traditional_times.iter().min().unwrap_or(&0)
    );
    tracing::info!(
        "  • 最慢时间: {}ms",
        traditional_times.iter().max().unwrap_or(&0)
    );

    tracing::info!("");
    tracing::info!("快速导出系统:");
    tracing::info!("  • 平均时间: {:.1}ms", avg_fast);
    tracing::info!("  • 最快时间: {}ms", fast_times.iter().min().unwrap_or(&0));
    tracing::info!("  • 最慢时间: {}ms", fast_times.iter().max().unwrap_or(&0));

    tracing::info!("");
    tracing::info!("📊 性能提升:");
    if improvement_percent > 0.0 {
        tracing::info!("  • 时间改善: {:.1}%", improvement_percent);
        tracing::info!("  • 加速比: {:.2}x", avg_traditional / avg_fast);
    } else {
        tracing::info!("  • 时间变化: {:.1}% (变慢)", improvement_percent.abs());
    }

    // 评估结果
    tracing::info!("");
    tracing::info!("🎯 评估结果:");
    if improvement_percent >= 60.0 {
        tracing::info!("✅ 优秀！达到了 60-80% 导出时间减少的目标");
    } else if improvement_percent >= 40.0 {
        tracing::info!("✅ 良好！接近 60-80% 导出时间减少的目标");
    } else if improvement_percent >= 20.0 {
        tracing::info!("⚠️  一般，有一定改善但未达到预期目标");
    } else if improvement_percent > 0.0 {
        tracing::info!("⚠️  轻微改善，需要进一步优化");
    } else {
        tracing::info!("❌ 性能没有提升或有所下降，需要检查实现");
    }

    // 生成简单报告
    generate_simple_report(
        traditional_times,
        fast_times,
        improvement_percent,
        output_dir,
    );
}

fn generate_simple_report(
    traditional_times: &[u64],
    fast_times: &[u64],
    improvement_percent: f64,
    output_dir: &PathBuf,
) {
    let report_file = output_dir.join("simple_benchmark_report.md");

    let avg_traditional =
        traditional_times.iter().sum::<u64>() as f64 / traditional_times.len() as f64;
    let avg_fast = fast_times.iter().sum::<u64>() as f64 / fast_times.len() as f64;

    let report = format!(
        r#"# 大型项目导出优化 - 简化基准测试报告

**测试时间**: {}

## 📊 性能提升摘要

| 指标 | 传统导出 | 快速导出 | 改善幅度 |
|------|----------|----------|----------|
| 平均时间 | {:.1}ms | {:.1}ms | **{:.1}%** |
| 最快时间 | {}ms | {}ms | - |
| 最慢时间 | {}ms | {}ms | - |

## 📈 详细结果

### 传统导出系统
{}

### 快速导出系统
{}

## 🎯 结论

{}

## 📁 生成的文件

- traditional_export_run_*.json - 传统导出结果
- fast_export_run_*.json - 快速导出结果
- simple_benchmark_report.md - 本报告
"#,
        chrono::Utc::now().to_rfc3339(),
        avg_traditional,
        avg_fast,
        improvement_percent,
        traditional_times.iter().min().unwrap_or(&0),
        fast_times.iter().min().unwrap_or(&0),
        traditional_times.iter().max().unwrap_or(&0),
        fast_times.iter().max().unwrap_or(&0),
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
            "✅ 优秀！达到了 60-80% 导出时间减少的目标"
        } else if improvement_percent >= 40.0 {
            "✅ 良好！接近 60-80% 导出时间减少的目标"
        } else if improvement_percent >= 20.0 {
            "⚠️ 一般，有一定改善但未达到预期目标"
        } else if improvement_percent > 0.0 {
            "⚠️ 轻微改善，需要进一步优化"
        } else {
            "❌ 性能没有提升或有所下降，需要检查实现"
        }
    );

    if let Err(e) = fs::write(&report_file, report) {
        tracing::error!("⚠️  生成报告失败: {}", e);
    } else {
        tracing::info!("");
        tracing::info!("📄 详细报告已生成: {}", report_file.display());
    }
}
