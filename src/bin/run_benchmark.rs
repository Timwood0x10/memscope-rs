//! 性能基准测试主程序
//!
//! 这个程序运行 complex_lifecycle_showcase.rs 的性能基准测试，
//! 对比传统导出系统和快速导出系统的性能。

use memscope_rs::export::performance_benchmark::{BenchmarkConfig, PerformanceBenchmark};
use std::path::PathBuf;
use std::process;

fn main() {
    tracing::info!("🚀 大型项目导出优化 - 性能基准测试");
    tracing::info!("=====================================");
    tracing::info!("");

    // 配置基准测试
    let config = BenchmarkConfig {
        test_runs: 5,
        output_dir: PathBuf::from("benchmark_results"),
        verbose: true,
        verify_consistency: true,
        generate_detailed_report: true,
    };

    // 创建基准测试器
    let mut benchmark = match PerformanceBenchmark::new(config) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("❌ 创建基准测试器失败: {}", e);
            process::exit(1);
        }
    };

    // 运行完整的基准测试
    match benchmark.run_full_benchmark() {
        Ok(comparison) => {
            tracing::info!("");
            tracing::info!("🎉 基准测试完成！");
            tracing::info!("==================");

            let perf = &comparison.performance_improvement;
            tracing::info!("📊 性能提升摘要:");
            tracing::info!(
                "  • 平均导出时间改善: {:.1}%",
                perf.avg_time_improvement_percent
            );
            tracing::info!(
                "  • 平均内存使用改善: {:.1}%",
                perf.avg_memory_improvement_percent
            );
            tracing::info!(
                "  • 平均吞吐量提升: +{:.1}%",
                perf.avg_throughput_improvement_percent
            );
            tracing::info!(
                "  • 平均写入速度提升: +{:.1}%",
                perf.avg_write_speed_improvement_percent
            );
            tracing::info!(
                "  • 最佳时间改善: {:.1}%",
                perf.best_time_improvement_percent
            );
            tracing::info!("  • 一致性评分: {:.1}/100", perf.consistency_score);
            tracing::info!("");

            // 评估是否达到目标
            if perf.avg_time_improvement_percent >= 60.0 {
                tracing::info!("✅ 优秀！达到了 60-80% 导出时间减少的目标");
            } else if perf.avg_time_improvement_percent >= 40.0 {
                tracing::info!("✅ 良好！接近 60-80% 导出时间减少的目标");
            } else if perf.avg_time_improvement_percent >= 20.0 {
                tracing::info!("⚠️  一般，有一定改善但未达到预期目标");
            } else {
                tracing::info!("❌ 性能提升不明显，需要进一步优化");
            }

            tracing::info!("");
            tracing::info!("📁 生成的文件:");
            tracing::info!("  • benchmark_results/benchmark_results.json - 详细测试数据");
            tracing::info!("  • benchmark_results/performance_report.md - 性能报告");
            tracing::info!("  • benchmark_results/traditional_export_run_*.json - 传统导出结果");
            tracing::info!("  • benchmark_results/fast_export_run_*.json - 快速导出结果");
        }
        Err(e) => {
            tracing::error!("❌ 基准测试失败: {}", e);
            process::exit(1);
        }
    }
}
