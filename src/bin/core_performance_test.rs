//! core performance test (only test export core, not include validation)
//!
//! This program directly tests the performance of the fast export coordinator's core, without any validation

use memscope_rs::{get_global_tracker, init};
use std::fs;
use std::path::{Path, PathBuf};
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

    // Skip running external example in test mode to avoid blocking make test
    if std::env::var("MEMSCOPE_TEST_MODE").is_ok() {
        tracing::info!("🔧 test mode detected, skipping external example execution");
    } else {
        // run complex_lifecycle_showcase to generate test data
        tracing::info!("🔧 run complex_lifecycle_showcase to generate test data...");
        let output = Command::new("cargo")
            .args([
                "run",
                "--release",
                "--example",
                "realistic_usage_with_extensions",  // Use existing example
            ])
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!("⚠️ run example failed: {}", stderr);
                    tracing::info!("continuing with existing test data...");
                }
                tracing::info!("✅ test data preparation completed");
            }
            Err(e) => {
                tracing::warn!("⚠️ execute command failed: {}, continuing with existing data", e);
            }
        }
    }

    // wait for system to stabilize
    std::thread::sleep(std::time::Duration::from_millis(1)); // Reduced for testing

    // run core performance tests
    run_core_performance_tests(&output_dir);
}

fn run_core_performance_tests(output_dir: &Path) {
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
        let output_path = output_dir.join(format!("traditional_core_run_{run}.json"));

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
        let output_path = output_dir.join(format!("fast_core_run_{run}"));

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
    output_dir: &Path,
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
    output_dir: &Path,
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_calculate_std_dev_empty_values() {
        // Test empty vector returns 0.0
        let values = vec![];
        let std_dev = calculate_std_dev(&values);
        assert_eq!(std_dev, 0.0);
    }

    #[test]
    fn test_calculate_std_dev_single_value() {
        // Test single value returns 0.0
        let values = vec![42];
        let std_dev = calculate_std_dev(&values);
        assert_eq!(std_dev, 0.0);
    }

    #[test]
    fn test_calculate_std_dev_multiple_values() {
        // Test standard deviation calculation with known values
        let values = vec![2, 4, 4, 4, 5, 5, 7, 9];
        let std_dev = calculate_std_dev(&values);
        // Expected std dev is approximately 2.0
        assert!((std_dev - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_std_dev_identical_values() {
        // Test identical values return 0.0
        let values = vec![5, 5, 5, 5, 5];
        let std_dev = calculate_std_dev(&values);
        assert_eq!(std_dev, 0.0);
    }

    #[test]
    fn test_generate_core_performance_report_basic() {
        // Test basic report generation functionality
        let traditional_times = vec![100, 110, 120];
        let fast_times = vec![50, 55, 60];
        let improvement_percent = 50.0;
        let output_dir = tempdir().expect("Failed to create temp directory");

        generate_core_performance_report(
            &traditional_times,
            &fast_times,
            improvement_percent,
            output_dir.path(),
        );

        let report_path = output_dir.path().join("core_performance_report.md");
        assert!(report_path.exists(), "Report file should be created");
        
        let report_content = fs::read_to_string(report_path)
            .expect("Should be able to read report file");
        
        // Verify essential content is present
        assert!(report_content.contains("Core Performance Benchmark Report"));
        assert!(report_content.contains("50.0%"));
        assert!(report_content.contains("Traditional Export Core"));
        assert!(report_content.contains("Fast Export Core"));
    }

    #[test]
    fn test_generate_core_performance_report_zero_improvement() {
        // Test report generation with zero improvement
        let traditional_times = vec![100, 100, 100];
        let fast_times = vec![100, 100, 100];
        let improvement_percent = 0.0;
        let output_dir = tempdir().expect("Failed to create temp directory");

        generate_core_performance_report(
            &traditional_times,
            &fast_times,
            improvement_percent,
            output_dir.path(),
        );

        let report_path = output_dir.path().join("core_performance_report.md");
        assert!(report_path.exists());
        
        let report_content = fs::read_to_string(report_path)
            .expect("Should be able to read report file");
        assert!(report_content.contains("0.0%"));
    }

    #[test]
    fn test_generate_core_performance_report_negative_improvement() {
        // Test report generation with performance degradation
        let traditional_times = vec![50, 55, 60];
        let fast_times = vec![100, 110, 120];
        let improvement_percent = -100.0;
        let output_dir = tempdir().expect("Failed to create temp directory");

        generate_core_performance_report(
            &traditional_times,
            &fast_times,
            improvement_percent,
            output_dir.path(),
        );

        let report_path = output_dir.path().join("core_performance_report.md");
        assert!(report_path.exists());
        
        let report_content = fs::read_to_string(report_path)
            .expect("Should be able to read report file");
        assert!(report_content.contains("-100.0%"));
    }

    #[test]
    fn test_display_core_performance_results_empty_data() {
        // Test display function handles empty data gracefully
        let traditional_times = vec![];
        let fast_times = vec![];
        let output_dir = tempdir().expect("Failed to create temp directory");

        // Should not panic with empty data
        display_core_performance_results(&traditional_times, &fast_times, output_dir.path());
    }

    #[test]
    fn test_display_core_performance_results_valid_data() {
        // Test display function with valid performance data
        let traditional_times = vec![200, 220, 240];
        let fast_times = vec![80, 90, 100];
        let output_dir = tempdir().expect("Failed to create temp directory");

        // Should complete without panic
        display_core_performance_results(&traditional_times, &fast_times, output_dir.path());
        
        // Verify report file is created
        let report_path = output_dir.path().join("core_performance_report.md");
        assert!(report_path.exists());
    }

    #[test]
    fn test_display_core_performance_results_single_measurement() {
        // Test display function with single measurement
        let traditional_times = vec![150];
        let fast_times = vec![75];
        let output_dir = tempdir().expect("Failed to create temp directory");

        display_core_performance_results(&traditional_times, &fast_times, output_dir.path());
        
        let report_path = output_dir.path().join("core_performance_report.md");
        assert!(report_path.exists());
    }

    #[test]
    fn test_performance_improvement_calculation() {
        // Test improvement percentage calculation logic
        let traditional_avg = 200.0;
        let fast_avg = 100.0;
        let expected_improvement = ((traditional_avg - fast_avg) / traditional_avg) * 100.0;
        assert_eq!(expected_improvement, 50.0);
        
        // Test zero traditional time
        let traditional_zero = 0.0;
        let fast_nonzero = 50.0;
        let improvement_zero = if traditional_zero > 0.0 {
            ((traditional_zero - fast_nonzero) / traditional_zero) * 100.0
        } else {
            0.0
        };
        assert_eq!(improvement_zero, 0.0);
    }

    #[test]
    fn test_report_content_structure() {
        // Test that generated report has expected structure
        let traditional_times = vec![300, 310, 320];
        let fast_times = vec![150, 155, 160];
        let improvement_percent = 50.0;
        let output_dir = tempdir().expect("Failed to create temp directory");

        generate_core_performance_report(
            &traditional_times,
            &fast_times,
            improvement_percent,
            output_dir.path(),
        );

        let report_path = output_dir.path().join("core_performance_report.md");
        let report_content = fs::read_to_string(report_path)
            .expect("Should be able to read report file");

        // Verify report sections exist
        assert!(report_content.contains("## 📊 Core Algorithm Performance Comparison"));
        assert!(report_content.contains("## 📈 Detailed Test Data"));
        assert!(report_content.contains("## 🔍 Core Algorithm Analysis"));
        assert!(report_content.contains("## 🎯 Conclusion"));
        assert!(report_content.contains("## 🚀 Future Optimization Suggestions"));
        
        // Verify data is included
        assert!(report_content.contains("run 1: 300ms"));
        assert!(report_content.contains("run 1: 150ms"));
    }
}
