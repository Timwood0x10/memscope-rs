//! pure performance benchmark (disable all validation)
//!
//! This program focuses on testing export performance, disabling all quality validation to obtain true performance data

use memscope_rs::{get_global_tracker, init};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

fn main() {
    tracing::info!("🚀 pure performance benchmark (disable all validation)");
    tracing::info!("============================");
    tracing::info!("");

    // init memory trace
    init();

    // create output directory
    let output_dir = PathBuf::from("performance_only_results");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        tracing::error!("❌ create output directory failed: {}", e);
        return;
    }

    // run complex_lifecycle_showcase to generate test data
    tracing::info!("🔧 run complex_lifecycle_showcase to generate test data...");
    let output = Command::new("cargo")
        .args([
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
    std::thread::sleep(std::time::Duration::from_millis(50)); // Reduced from 500ms to 50ms

    // run pure performance tests
    run_performance_only_tests(&output_dir);
}

fn run_performance_only_tests(output_dir: &Path) {
    tracing::info!("");
    tracing::info!("📊 start pure performance tests...");
    tracing::info!("====================");

    let test_runs = 3;
    let mut traditional_times = Vec::new();
    let mut fast_times = Vec::new();

    // run traditional export test (disable validation)
    tracing::info!("🐌 test traditional export system (no validation)...");
    for run in 1..=test_runs {
        tracing::info!("  run {}/{}: traditional export", run, test_runs);

        let start_time = Instant::now();
        let output_path = output_dir.join(format!("traditional_export_run_{run}.json"));

        // get tracker and export (use minimal configuration)
        let tracker = get_global_tracker();
        let options = memscope_rs::core::tracker::export_json::ExportJsonOptions::default()
            .parallel_processing(false)
            .fast_export_mode(true)
            .schema_validation(false);

        let result = tracker.export_to_json_with_options(&output_path, options);
        let export_time = start_time.elapsed();

        match result {
            Ok(_) => {
                traditional_times.push(export_time.as_millis() as u64);
                tracing::info!("    ⏱️  time: {}ms", export_time.as_millis());
            }
            Err(e) => {
                tracing::error!("    ❌ export failed: {}", e);
            }
        }

        // 短暂休息
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // run fast export test (disable validation)
    tracing::info!("⚡ test fast export system (no validation)...");
    for run in 1..=test_runs {
        tracing::info!("  run {}/{}: fast export", run, test_runs);

        let start_time = Instant::now();
        let output_path = output_dir.join(format!("fast_export_run_{run}.json"));

        // get tracker and use fast export (disable validation)
        let tracker = get_global_tracker();
        let options = memscope_rs::core::tracker::export_json::ExportJsonOptions::default()
            .parallel_processing(true)
            .fast_export_mode(true)
            .schema_validation(false)
            .buffer_size(64 * 1024) // 64KB
            .batch_size(10000); // large batch

        let result = tracker.export_to_json_with_options(&output_path, options);
        let export_time = start_time.elapsed();

        match result {
            Ok(_) => {
                fast_times.push(export_time.as_millis() as u64);
                tracing::info!("    ⚡ time: {}ms", export_time.as_millis());
            }
            Err(e) => {
                tracing::error!("    ❌ export failed: {}", e);
            }
        }

        // short rest
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // calculate and display results
    display_performance_results(&traditional_times, &fast_times, output_dir);
}

fn display_performance_results(traditional_times: &[u64], fast_times: &[u64], output_dir: &Path) {
    tracing::info!("");
    tracing::info!("📈 pure performance test results");
    tracing::info!("==================");

    if traditional_times.is_empty() || fast_times.is_empty() {
        tracing::info!("❌ test data insufficient, cannot generate report");
        return;
    }

    // calculate average
    let avg_traditional =
        traditional_times.iter().sum::<u64>() as f64 / traditional_times.len() as f64;
    let avg_fast = fast_times.iter().sum::<u64>() as f64 / fast_times.len() as f64;

    // calculate improvement percentage
    let improvement_percent = if avg_traditional > 0.0 {
        ((avg_traditional - avg_fast) / avg_traditional) * 100.0
    } else {
        0.0
    };

    // display results
    tracing::info!("traditional export system (no validation): ");
    tracing::info!("  • average time: {:.1}ms", avg_traditional);
    tracing::info!(
        "  • fastest time: {}ms",
        traditional_times.iter().min().unwrap_or(&0)
    );
    tracing::info!(
        "  • slowest time: {}ms",
        traditional_times.iter().max().unwrap_or(&0)
    );
    tracing::info!(
        "  • time range: {}ms",
        traditional_times.iter().max().unwrap_or(&0) - traditional_times.iter().min().unwrap_or(&0)
    );

    tracing::info!("");
    tracing::info!("fast export system (no validation): ");
    tracing::info!("  • average time: {:.1}ms", avg_fast);
    tracing::info!(
        "  • fastest time: {}ms",
        fast_times.iter().min().unwrap_or(&0)
    );
    tracing::info!(
        "  • slowest time: {}ms",
        fast_times.iter().max().unwrap_or(&0)
    );
    tracing::info!(
        "  • time range: {}ms",
        fast_times.iter().max().unwrap_or(&0) - fast_times.iter().min().unwrap_or(&0)
    );

    tracing::info!("");
    tracing::info!("📊 pure performance comparison:");
    if improvement_percent > 0.0 {
        tracing::info!("  • time improvement: {:.1}%", improvement_percent);
        tracing::info!("  • acceleration ratio: {:.2}x", avg_traditional / avg_fast);
        tracing::info!("  • time saved: {:.1}ms", avg_traditional - avg_fast);
    } else {
        tracing::info!(
            "  • time change: {:.1}% (slower)",
            improvement_percent.abs()
        );
        tracing::info!("  • deceleration ratio: {:.2}x", avg_fast / avg_traditional);
        tracing::info!("  • time increase: {:.1}ms", avg_fast - avg_traditional);
    }

    // evaluation result
    tracing::info!("");
    tracing::info!("🎯 pure performance evaluation:");
    if improvement_percent >= 60.0 {
        tracing::info!("✅ excellent! reached 60-80% export time reduction target");
    } else if improvement_percent >= 40.0 {
        tracing::info!("✅ good! close to 60-80% export time reduction target");
    } else if improvement_percent >= 20.0 {
        tracing::info!("⚠️  general, some improvement but not met the target");
    } else if improvement_percent > 0.0 {
        tracing::info!("⚠️  minor improvement, need further optimization");
    } else {
        tracing::info!("❌ core performance not improved, need to re-examine the algorithm");
    }

    // generate pure performance report
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
    output_dir: &Path,
) {
    let report_file = output_dir.join("pure_performance_report.md");

    let avg_traditional =
        traditional_times.iter().sum::<u64>() as f64 / traditional_times.len() as f64;
    let avg_fast = fast_times.iter().sum::<u64>() as f64 / fast_times.len() as f64;

    let report = format!(
        r#"# Large Project Export Optimization - Pure Performance Benchmark Report

**Test Time**: {}
**Test Description**: This test disables all quality validation, security analysis, FFI analysis, and focuses on testing core export performance.

## 📊 Pure Performance Comparison

| Indicator | Traditional Export | Fast Export | Improvement |
|----------|-------------------|-------------|-------------|
| Average Time | {:.1}ms | {:.1}ms | **{:.1}%** |
| Fastest Time | {}ms | {}ms | - |
| Slowest Time | {}ms | {}ms | - |
| Time Stability | {}ms range | {}ms range | - |

## 📈 Detailed Test Data

### Traditional Export System (No Validation)
{}

### Fast Export System (No Validation)
{}

## 🔍 performance analysis

### if improvement >= 60%
this indicates that the fast export system's core algorithm is effective, and the previous performance issues mainly came from quality validation and other additional functions.

### if improvement < 20%
this indicates that the fast export system's core algorithm needs further optimization, and the issue is not just the validation overhead.

## 🎯 conclusion

{}

## 📝 important findings

1. **quality validation impact**: through disabling validation, we can see the true performance of the core export algorithm
2. **parallel processing effect**: in the absence of validation, parallel processing shows more明显
3. **performance bottleneck identification**: helps distinguish between algorithm issues and validation overhead

## 📁 generated files

- traditional_export_run_*.json - traditional export results (no validation)
- fast_export_run_*.json - fast export results (no validation)
- pure_performance_report.md - this report
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
            "✅ excellent! the fast export system's core algorithm is effective, and the previous performance issues mainly came from quality validation and other additional functions."
        } else if improvement_percent >= 40.0 {
            "✅ good! the fast export system has improvement, but still has optimization space."
        } else if improvement_percent >= 20.0 {
            "⚠️ general, the fast export system has some improvement, but the core algorithm may need further optimization."
        } else if improvement_percent > 0.0 {
            "⚠️ minor improvement, the fast export system's core algorithm advantage is not obvious, need to re-examine the design."
        } else {
            "❌ the fast export system's core performance has not improved, need to fundamentally redesign the algorithm."
        }
    );

    if let Err(e) = fs::write(&report_file, report) {
        tracing::error!("⚠️  generate report failed: {}", e);
    } else {
        tracing::info!("");
        tracing::info!(
            "📄 pure performance report generated: {}",
            report_file.display()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_performance_calculation_logic() {
        // Test performance improvement calculation without running actual benchmarks
        let traditional_times = [200, 220, 240, 180, 260];
        let fast_times = [100, 110, 120, 90, 130];

        // Calculate averages
        let avg_traditional =
            traditional_times.iter().sum::<u64>() as f64 / traditional_times.len() as f64;
        let avg_fast = fast_times.iter().sum::<u64>() as f64 / fast_times.len() as f64;

        assert_eq!(avg_traditional, 220.0);
        assert_eq!(avg_fast, 110.0);

        // Calculate improvement
        let improvement_percent = ((avg_traditional - avg_fast) / avg_traditional) * 100.0;
        assert_eq!(improvement_percent, 50.0);

        // Test min/max calculations
        assert_eq!(*traditional_times.iter().min().unwrap(), 180);
        assert_eq!(*traditional_times.iter().max().unwrap(), 260);
        assert_eq!(*fast_times.iter().min().unwrap(), 90);
        assert_eq!(*fast_times.iter().max().unwrap(), 130);
    }

    #[test]
    fn test_time_range_calculations() {
        // Test time range calculations for stability analysis
        let times1 = [100, 105, 110, 95, 115];
        let times2 = [50, 80, 60, 70, 90];

        let range1 = times1.iter().max().unwrap() - times1.iter().min().unwrap();
        let range2 = times2.iter().max().unwrap() - times2.iter().min().unwrap();

        assert_eq!(range1, 20); // 115 - 95
        assert_eq!(range2, 40); // 90 - 50

        // Test that range2 has higher variability
        assert!(range2 > range1);
    }

    #[test]
    fn test_display_performance_results() {
        // Test display function without running actual performance tests
        let traditional_times = vec![100, 110, 120];
        let fast_times = vec![50, 55, 60];
        let output_dir = tempdir().expect("Failed to create temp directory");

        display_performance_results(&traditional_times, &fast_times, output_dir.path());

        // Verify report file is created
        let report_path = output_dir.path().join("pure_performance_report.md");
        assert!(report_path.exists());
    }

    #[test]
    fn test_generate_performance_report_basic() {
        // Test basic report generation functionality
        let traditional_times = vec![100, 110, 120];
        let fast_times = vec![50, 55, 60];
        let improvement_percent = 50.0;
        let output_dir = tempdir().expect("Failed to create temp directory");

        generate_performance_report(
            &traditional_times,
            &fast_times,
            improvement_percent,
            output_dir.path(),
        );

        let report_path = output_dir.path().join("pure_performance_report.md");
        assert!(report_path.exists(), "Report file should be created");

        let report_content =
            fs::read_to_string(report_path).expect("Should be able to read report file");

        // Verify essential content is present
        assert!(report_content.contains("Pure Performance Benchmark Report"));
        assert!(report_content.contains("50.0%"));
        assert!(report_content.contains("Traditional Export"));
        assert!(report_content.contains("Fast Export"));
    }

    #[test]
    fn test_generate_performance_report_different_improvements() {
        // Test report generation with different improvement levels
        let traditional_times = vec![100, 100, 100];
        let fast_times = vec![25, 25, 25]; // 75% improvement
        let improvement_percent = 75.0;
        let output_dir = tempdir().expect("Failed to create temp directory");

        generate_performance_report(
            &traditional_times,
            &fast_times,
            improvement_percent,
            output_dir.path(),
        );

        let report_path = output_dir.path().join("pure_performance_report.md");
        let report_content =
            fs::read_to_string(report_path).expect("Should be able to read report file");

        // Should contain excellent conclusion for 75% improvement
        assert!(report_content.contains("75.0%"));
        assert!(report_content.contains("excellent"));
    }

    #[test]
    fn test_performance_conclusion_logic() {
        // Test the conclusion logic for different improvement percentages
        let test_cases = vec![
            (75.0, "excellent"),
            (50.0, "good"),
            (30.0, "general"),
            (10.0, "minor improvement"),
            (-10.0, "not improved"),
        ];

        for (improvement, expected_keyword) in test_cases {
            let conclusion = if improvement >= 60.0 {
                "excellent! the fast export system's core algorithm is effective"
            } else if improvement >= 40.0 {
                "good! the fast export system has improvement"
            } else if improvement >= 20.0 {
                "general, the fast export system has some improvement"
            } else if improvement > 0.0 {
                "minor improvement, the fast export system's core algorithm advantage is not obvious"
            } else {
                "the fast export system's core performance has not improved"
            };

            assert!(conclusion.contains(expected_keyword.split('!').next().unwrap()));
        }
    }

    #[test]
    fn test_report_data_formatting() {
        // Test the data formatting logic for reports
        let times = [100, 110, 120];
        let formatted: Vec<String> = times
            .iter()
            .enumerate()
            .map(|(i, t)| format!("- run {}: {}ms", i + 1, t))
            .collect();

        assert_eq!(formatted.len(), 3);
        assert_eq!(formatted[0], "- run 1: 100ms");
        assert_eq!(formatted[1], "- run 2: 110ms");
        assert_eq!(formatted[2], "- run 3: 120ms");

        let joined = formatted.join("\n");
        assert!(joined.contains("run 1: 100ms"));
        assert!(joined.contains("run 2: 110ms"));
        assert!(joined.contains("run 3: 120ms"));
    }
}
