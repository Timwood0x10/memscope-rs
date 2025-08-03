//! 分配数量诊断程序
//!
//! 检查为什么导出的文件大小不随分配数量线性增长

use memscope_rs::{get_global_tracker, init};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    tracing::info!("🔍 分配数量诊断程序");
    tracing::info!("==================");
    tracing::info!("");

    // 初始化内存跟踪
    init();

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
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // 诊断分配数量
    diagnose_allocation_count();
}

fn diagnose_allocation_count() {
    tracing::info!("");
    tracing::info!("📊 诊断分配数量和文件大小关系");
    tracing::info!("==============================");

    let tracker = get_global_tracker();
    let stats = tracker.get_stats().unwrap();

    tracing::info!("🔍 全局跟踪器统计:");
    tracing::info!("  • 总分配数: {}", stats.total_allocations);
    tracing::info!("  • 活跃分配数: {}", stats.active_allocations);
    tracing::info!(
        "  • 峰值内存: {:.2} MB",
        stats.peak_memory as f64 / 1024.0 / 1024.0
    );
    tracing::info!(
        "  • 当前内存: {:.2} MB",
        stats.active_memory as f64 / 1024.0 / 1024.0
    );

    // 创建输出目录
    let output_dir = PathBuf::from("diagnostic_results");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        tracing::error!("❌ 创建输出目录失败: {}", e);
        return;
    }

    // 测试不同的导出方式
    test_traditional_export(&output_dir, &stats);
    test_fast_export(&output_dir, &stats);
    test_raw_data_access(&stats);
}

fn test_traditional_export(output_dir: &PathBuf, stats: &memscope_rs::core::types::MemoryStats) {
    tracing::info!("");
    tracing::info!("🐌 测试传统导出:");

    let output_path = output_dir.join("traditional_diagnostic.json");
    let tracker = get_global_tracker();

    match tracker.export_to_json(&output_path) {
        Ok(_) => {
            if let Ok(metadata) = fs::metadata(&output_path) {
                let file_size = metadata.len();
                tracing::info!(
                    "  • 文件大小: {:.2} MB ({} bytes)",
                    file_size as f64 / 1024.0 / 1024.0,
                    file_size
                );
                tracing::info!(
                    "  • 每个分配平均大小: {:.1} bytes",
                    file_size as f64 / stats.total_allocations as f64
                );

                // 读取文件内容分析
                if let Ok(content) = fs::read_to_string(&output_path) {
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(allocations) = json_value.get("allocations") {
                            if let Some(alloc_array) = allocations.as_array() {
                                tracing::info!("  • JSON中的分配数量: {}", alloc_array.len());
                                tracing::info!("  • 跟踪器报告的分配数量: {}", stats.total_allocations);
                                if alloc_array.len() != stats.total_allocations {
                                    tracing::info!("  ⚠️  数量不匹配！可能存在数据丢失");
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("  ❌ 传统导出失败: {}", e);
        }
    }
}

fn test_fast_export(output_dir: &PathBuf, stats: &memscope_rs::core::types::MemoryStats) {
    tracing::info!("");
    tracing::info!("⚡ 测试快速导出:");

    let output_path = output_dir.join("fast_diagnostic");

    // 使用快速导出协调器
    let config = memscope_rs::export::fast_export_coordinator::FastExportConfig::default();
    let mut coordinator =
        memscope_rs::export::fast_export_coordinator::FastExportCoordinator::new(config);

    match coordinator.export_fast(&output_path) {
        Ok(export_stats) => {
            tracing::info!(
                "  • 处理的分配数量: {}",
                export_stats.parallel_processing.total_allocations
            );
            tracing::info!(
                "  • 写入的字节数: {:.2} MB ({} bytes)",
                export_stats.write_performance.total_bytes_written as f64 / 1024.0 / 1024.0,
                export_stats.write_performance.total_bytes_written
            );
            tracing::info!(
                "  • 分片数量: {}",
                export_stats.parallel_processing.shard_count
            );
            tracing::info!(
                "  • 每个分配平均大小: {:.1} bytes",
                export_stats.write_performance.total_bytes_written as f64
                    / export_stats.parallel_processing.total_allocations as f64
            );

            if export_stats.parallel_processing.total_allocations != stats.total_allocations {
                tracing::info!(
                    "  ⚠️  快速导出处理的分配数量 ({}) 与跟踪器报告的数量 ({}) 不匹配！",
                    export_stats.parallel_processing.total_allocations, stats.total_allocations
                );
            }
        }
        Err(e) => {
            tracing::error!("  ❌ 快速导出失败: {}", e);
        }
    }
}

fn test_raw_data_access(stats: &memscope_rs::core::types::MemoryStats) {
    tracing::info!("");
    tracing::info!("🔍 测试原始数据访问:");

    let _tracker = get_global_tracker();

    // 尝试获取所有分配信息
    tracing::info!("  • 尝试直接访问分配数据...");

    // 这里我们需要检查跟踪器是否有获取所有分配的方法
    // 由于API限制，我们只能通过统计信息来推断
    tracing::info!("  • 跟踪器统计显示:");
    tracing::info!("    - 总分配数: {}", stats.total_allocations);
    tracing::info!("    - 活跃分配数: {}", stats.active_allocations);
    tracing::info!(
        "    - 已释放分配数: {}",
        stats.total_allocations - stats.active_allocations
    );

    if stats.active_allocations < stats.total_allocations {
        tracing::info!(
            "  💡 发现: 有 {} 个分配已被释放，这可能影响导出的数据量",
            stats.total_allocations - stats.active_allocations
        );
    }
}
