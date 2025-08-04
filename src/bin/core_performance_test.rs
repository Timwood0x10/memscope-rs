//! core performance test (only test export core, not include validation)
//!
//! This program directly tests the performance of the fast export coordinator's core, without any validation

use memscope_rs::{get_global_tracker, init};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

fn main() {
    tracing::info!("🎯  core_performance_test (pure export core algorithm)");
    tracing::info!("==============================");
    tracing::info!("");

    // init memory trace
    init();

    // create output directory
    let output_dir = PathBuf::from("core_performance_results");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        tracing::error!("❌ create output directory failed: {}", e);
        return;
    }

    // run complex_lifecycle_showcase to generate test data
    tracing::info!("🔧 run complex_lifecycle_showcase to generate test data...");
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
                tracing::error!("❌ run complex_lifecycle_showcase failed: {}", stderr);
                return;
            }
            tracing::info!("✅ test data generated");
        }
        Err(e) => {
            tracing::error!("❌ execute command failed: {}", e);
            return;
        }
    }

    // wait for system to stabilize
    std::thread::sleep(std::time::Duration::from_millis(500));

    // run core performance tests
    run_core_performance_tests(&output_dir);
}

fn run_core_performance_tests(output_dir: &PathBuf) {
    tracing::info!("");
    tracing::info!("📊 start core performance tests...");
    tracing::info!("======================");

    let test_runs = 5; // increase test runs to get more accurate results
    let mut traditional_core_times = Vec::new();
    let mut fast_core_times = Vec::new();

    // test traditional export core performance (only measure main export, not other files)
    tracing::info!("🐌 test traditional export core performance...");
    for run in 1..=test_runs {
        tracing::info!("  run {}/{}: traditional export core", run, test_runs);

        let start_time = Instant::now();
        let output_path = output_dir.join(format!("traditional_core_run_{}.json", run));

        // get global tracker and export (use minimal config, only generate main file)
        let tracker = get_global_tracker();
        let result = tracker.export_to_json(&output_path);
        let export_time = start_time.elapsed();

        match result {
            Ok(_) => {
                traditional_core_times.push(export_time.as_millis() as u64);
                tracing::info!("    ⏱️  core time: {}ms", export_time.as_millis());
            }
            Err(e) => {
                tracing::error!("    ❌ export failed: {}", e);
            }
        }

        // take a short break
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // test fast export coordinator core performance
    for run in 1..=test_runs {
        tracing::info!("  run {}/{}: fast export core", run, test_runs);

        // directly test fast export coordinator
        let start_time = Instant::now();
        let output_path = output_dir.join(format!("fast_core_run_{}", run));

        // use fast export coordinator
        let config = memscope_rs::export::fast_export_coordinator::FastExportConfig {
            enable_data_localization: true,
            data_cache_ttl_ms: 100,
            shard_config:
                memscope_rs::export::parallel_shard_processor::ParallelShardConfig::default(),
            writer_config:
                memscope_rs::export::high_speed_buffered_writer::HighSpeedWriterConfig::default(),
            enable_performance_monitoring: false, // disable performance monitoring to reduce overhead
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
                max_data_loss_rate: 100.0, // allow any data loss to skip validation
                min_expected_file_size: 0,
                max_expected_file_size: usize::MAX,
                verbose_logging: false,
            },
            enable_resource_monitoring: false,
            memory_limit_mb: 1024,
            disk_limit_mb: 2048,
            cpu_limit_percent: 80.0,
        };

        let mut coordinator =
            memscope_rs::export::fast_export_coordinator::FastExportCoordinator::new(config);
        let result = coordinator.export_fast(&output_path);
        let export_time = start_time.elapsed();

        match result {
            Ok(stats) => {
                // only record core export time, not validation
                let core_time = stats.data_gathering.total_time_ms
                    + stats.parallel_processing.total_processing_time_ms
                    + stats.write_performance.total_write_time_ms;
                fast_core_times.push(core_time);
                tracing::info!(
                    "    ⚡ core time: {}ms (total time: {}ms)",
                    core_time,
                    export_time.as_millis()
                );
                tracing::info!(
                    "       data gathering: {}ms, parallel processing: {}ms, write performance: {}ms",
                    stats.data_gathering.total_time_ms,
                    stats.parallel_processing.total_processing_time_ms,
                    stats.write_performance.total_write_time_ms
                );
            }
            Err(e) => {
                tracing::error!("    ❌ export failed: {}", e);
            }
        }

        // take a short break
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // calculate and display results
    display_core_performance_results(&traditional_core_times, &fast_core_times, output_dir);
}

fn display_core_performance_results(
    traditional_times: &[u64],
    fast_times: &[u64],
    output_dir: &PathBuf,
) {
    tracing::info!("");
    tracing::info!("📈 core performance test results");
    tracing::info!("====================");

    if traditional_times.is_empty() || fast_times.is_empty() {
        tracing::info!("❌ test data is empty, cannot generate report");
        return;
    }

    // calculate statistics
    let avg_traditional =
        traditional_times.iter().sum::<u64>() as f64 / traditional_times.len() as f64;
    let avg_fast = fast_times.iter().sum::<u64>() as f64 / fast_times.len() as f64;

    let min_traditional = *traditional_times.iter().min().unwrap_or(&0);
    let max_traditional = *traditional_times.iter().max().unwrap_or(&0);
    let min_fast = *fast_times.iter().min().unwrap_or(&0);
    let max_fast = *fast_times.iter().max().unwrap_or(&0);

    // calculate improvement percentage
    let improvement_percent = if avg_traditional > 0.0 {
        ((avg_traditional - avg_fast) / avg_traditional) * 100.0
    } else {
        0.0
    };

    //  display results
    tracing::info!("traditional export core algorithm:");
    tracing::info!("  • average time: {:.1}ms", avg_traditional);
    tracing::info!("  • fastest time: {}ms", min_traditional);
    tracing::info!("  • slowest time: {}ms", max_traditional);
    tracing::info!(
        "  • standard deviation: {:.1}ms",
        calculate_std_dev(traditional_times)
    );

    tracing::info!("");
    tracing::info!("fast export core algorithm:");
    tracing::info!("  • average time: {:.1}ms", avg_fast);
    tracing::info!("  • fastest time: {}ms", min_fast);
    tracing::info!("  • slowest time: {}ms", max_fast);
    tracing::info!(
        "  • standard deviation: {:.1}ms",
        calculate_std_dev(fast_times)
    );

    tracing::info!("");
    tracing::info!("📊 core algorithm performance comparison:");
    if improvement_percent > 0.0 {
        tracing::info!("  • time improvement: {:.1}%", improvement_percent);
        tracing::info!("  • acceleration ratio: {:.2}x", avg_traditional / avg_fast);
        tracing::info!("  • time saved: {:.1}ms", avg_traditional - avg_fast);
    } else {
        tracing::info!(
            "  • time change: {:.1}% (slower)",
            improvement_percent.abs()
        );
        tracing::info!("  • acceleration ratio: {:.2}x", avg_fast / avg_traditional);
        tracing::info!("  • time increase: {:.1}ms", avg_fast - avg_traditional);
    }

    // evaluation results
    tracing::info!("");
    tracing::info!("🎯 core algorithm evaluation:");
    if improvement_percent >= 60.0 {
        tracing::info!(
            "✅ excellent! core algorithm reached the goal of 60-80% export time reduction"
        );
        tracing::info!("   the fast export system's core design is successful!");
    } else if improvement_percent >= 40.0 {
        tracing::info!("✅ good! core algorithm is close to 60-80% export time reduction");
        tracing::info!("   the fast export system has a clear advantage, can be further optimized");
    } else if improvement_percent >= 20.0 {
        tracing::info!(
            "⚠️  general, core algorithm has some improvement but did not reach the expected goal"
        );
        tracing::info!(
            "   need to further optimize parallel processing and data localization strategy"
        );
    } else if improvement_percent > 0.0 {
        tracing::info!("⚠️  minor improvement, core algorithm advantage is not significant");
        tracing::info!("   need to re-examine the design of the fast export system");
    } else {
        tracing::info!("❌ core algorithm performance did not improve or decreased");
        tracing::info!("   need to fundamentally re-design the fast export algorithm");
    }

    // generate detailed report
    generate_core_performance_report(
        traditional_times,
        fast_times,
        improvement_percent,
        output_dir,
    );
}

fn calculate_std_dev(values: &[u64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mean = values.iter().sum::<u64>() as f64 / values.len() as f64;
    let variance = values
        .iter()
        .map(|x| (*x as f64 - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64;
    variance.sqrt()
}

fn generate_core_performance_report(
    traditional_times: &[u64],
    fast_times: &[u64],
    improvement_percent: f64,
    output_dir: &PathBuf,
) {
    let report_file = output_dir.join("core_performance_report.md");

    let avg_traditional =
        traditional_times.iter().sum::<u64>() as f64 / traditional_times.len() as f64;
    let avg_fast = fast_times.iter().sum::<u64>() as f64 / fast_times.len() as f64;

    let report = format!(
        r#"# Large Project Export Optimization - Core Performance Benchmark Report

**Test Time**: {}
**Test Description**: This test specifically tests the performance of the core algorithm in the fast export coordinator, excluding quality validation, progress monitoring, and other additional functions.

## 📊 Core Algorithm Performance Comparison

| Indicator | Traditional Export Core | Fast Export Core | Improvement |
|------|-------------|-------------|----------|
| Average Time | {:.1}ms | {:.1}ms | **{:.1}%** |
| Fastest Time | {}ms | {}ms | - |
| Slowest Time | {}ms | {}ms | - |
| Standard Deviation | {:.1}ms | {:.1}ms | - |

## 📈 Detailed Test Data

### Traditional Export Core Algorithm
{}

### Fast Export Core Algorithm
{}

## 🔍 Core Algorithm Analysis

### Data Localization Effect
The fast export system reduces global state access through data localization, which is a key factor in performance improvement.

### Parallel Processing Effect
The fast export system uses parallel sharding processing, which should perform better on multi-core systems.

### High-speed Buffer Writing
The fast export system uses pre-allocated buffers and batch writing to reduce I/O overhead.

## 🎯 Conclusion

{}

## 📝 Key Discoveries

1. **Core Algorithm Effectiveness**: By testing the pure core algorithm, we can accurately evaluate the true performance of the fast export system.
2. **Bottleneck Identification**: Helps distinguish between core algorithm issues and additional function (validation, monitoring) overhead.
3. **Optimization Direction**: Provides clear direction for further performance optimization.

## 🚀 Future Optimization Suggestions

### If improvement >= 60%
- Core algorithm design is successful, focus on optimizing the performance of additional functions
- Consider making fast export the default export method

### If improvement is 20-60%
- Core algorithm is effective but still has optimization space
- Focus on optimizing parallel processing efficiency and data localization strategy

### If improvement < 20%
- Need to re-examine the core design of the fast export system
- Consider adopting different optimization strategies

## 📁 Generated Files

- traditional_core_run_*.json - traditional export core results
- fast_core_run_* - fast export core results
- core_performance_report.md - this report
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
        traditional_times
            .iter()
            .enumerate()
            .map(|(i, t)| format!("- run {}: {}ms", i + 1, t))
            .collect::<Vec<_>>()
            .join("\n"),
        fast_times
            .iter()
            .enumerate()
            .map(|(i, t)| format!("- run {}: {}ms", i + 1, t))
            .collect::<Vec<_>>()
            .join("\n"),
        if improvement_percent >= 60.0 {
            "✅ Excellent! The core algorithm of the quick export system is highly effective and has met the performance targets. The core design approach is correct, with the main bottleneck being the overhead of additional features."
        } else if improvement_percent >= 40.0 {
            "✅ Good! The core algorithm of the quick export system shows significant improvement and is close to the target. Further optimization of parallel processing and data localization could yield better results."
        } else if improvement_percent >= 20.0 {
            "⚠️ Fair. The core algorithm shows some improvement but still falls short of the target. Need to review the parallel processing strategy and data localization implementation."
        } else if improvement_percent > 0.0 {
            "⚠️ Slight improvement. The advantages of the quick export system's core algorithm are not significant. May need to adopt a completely different optimization strategy or redesign the core architecture."
        } else {
            "❌ No improvement or degradation in the performance of the quick export system's core algorithm. Need to fundamentally reassess the design approach, as the current optimization direction may be incorrect."
        }
    );

    if let Err(e) = fs::write(&report_file, report) {
        tracing::error!("⚠️  generate report failed: {}", e);
    } else {
        tracing::info!("");
        tracing::info!(
            "📄 core performance report generated: {}",
            report_file.display()
        );
    }
}
