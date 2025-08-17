//! simple performance benchmark
//!
//! this program runs a simple performance benchmark, comparing the performance of traditional export and fast export

use memscope_rs::{get_global_tracker, init};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

fn main() {
    tracing::info!("🚀 large project export optimization - simple performance benchmark");
    tracing::info!("=========================================");
    tracing::info!("");

    // init memory tracker
    init();

    // create output directory
    let output_dir = PathBuf::from("benchmark_results");
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
    std::thread::sleep(std::time::Duration::from_millis(50)); // Reduced from 500ms to 50ms

    // run benchmark tests
    run_benchmark_tests(&output_dir);
}

fn run_benchmark_tests(output_dir: &PathBuf) {
    tracing::info!("");
    tracing::info!("📊 start benchmark tests...");
    tracing::info!("==================");

    let test_runs = 3;
    let mut traditional_times = Vec::new();
    let mut fast_times = Vec::new();

    // run traditional export test
    tracing::info!("🐌 test traditional export system...");
    for run in 1..=test_runs {
        tracing::info!("  run {}/{}: traditional export", run, test_runs);

        let start_time = Instant::now();
        let output_path = output_dir.join(format!("traditional_export_run_{}.json", run));

        // get tracker and export
        let tracker = get_global_tracker();
        let result = tracker.export_to_json(&output_path);
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

        // short rest
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // run fast export test
    tracing::info!("⚡ test fast export system...");
    for run in 1..=test_runs {
        tracing::info!("  run {}/{}: fast export", run, test_runs);

        let start_time = Instant::now();
        let output_path = output_dir.join(format!("fast_export_run_{}.json", run));

        // get tracker and use optimized export
        let tracker = get_global_tracker();
        let mut options =
            memscope_rs::export::optimized_json_export::OptimizedExportOptions::default();
        options.parallel_processing = true; // enable parallel processing
        options.enable_fast_export_mode = true; // enable fast export mode
        options.enable_schema_validation = false; // disable schema validation to improve performance

        let result = tracker.export_json_with_options(&output_path, options);
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
    display_results(&traditional_times, &fast_times, output_dir);
}

fn display_results(traditional_times: &[u64], fast_times: &[u64], output_dir: &PathBuf) {
    tracing::info!("");
    tracing::info!("📈 benchmark results");
    tracing::info!("================");

    if traditional_times.is_empty() || fast_times.is_empty() {
        tracing::info!("❌ test data insufficient, cannot generate report");
        return;
    }

    // calculate average values
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
    tracing::info!("traditional export system:");
    tracing::info!("  • average time: {:.1}ms", avg_traditional);
    tracing::info!(
        "  • fastest time: {}ms",
        traditional_times.iter().min().unwrap_or(&0)
    );
    tracing::info!(
        "  • slowest time: {}ms",
        traditional_times.iter().max().unwrap_or(&0)
    );

    tracing::info!("");
    tracing::info!("fast export system:");
    tracing::info!("  • average time: {:.1}ms", avg_fast);
    tracing::info!(
        "  • fastest time: {}ms",
        fast_times.iter().min().unwrap_or(&0)
    );
    tracing::info!(
        "  • slowest time: {}ms",
        fast_times.iter().max().unwrap_or(&0)
    );

    tracing::info!("");
    tracing::info!("📊 performance improvement:");
    if improvement_percent > 0.0 {
        tracing::info!("  • time improvement: {:.1}%", improvement_percent);
        tracing::info!("  • acceleration ratio: {:.2}x", avg_traditional / avg_fast);
    } else {
        tracing::info!(
            "  • time change: {:.1}% (slower)",
            improvement_percent.abs()
        );
    }

    // evaluation result
    tracing::info!("");
    tracing::info!("🎯 evaluation result:");
    if improvement_percent >= 60.0 {
        tracing::info!("✅ excellent! reached 60-80% export time reduction target");
    } else if improvement_percent >= 40.0 {
        tracing::info!("✅ good! close to 60-80% export time reduction target");
    } else if improvement_percent >= 20.0 {
        tracing::info!("⚠️ general, some improvement but not met the target");
    } else if improvement_percent > 0.0 {
        tracing::info!("⚠️ minor improvement, need further optimization");
    } else {
        tracing::info!("❌ performance not improved, need to re-examine the algorithm");
    }

    // generate simple report
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
        r#"# large project export optimization - simple benchmark report

**test time**: {}

## 📊 performance improvement summary

| metric | traditional export | fast export | improvement |
|------|----------|----------|----------|
| average time | {:.1}ms | {:.1}ms | **{:.1}%** |
| fastest time | {}ms | {}ms | - |
| slowest time | {}ms | {}ms | - |

## 📈 detailed results

### traditional export system
{}

### fast export system
{}

## 🎯 evaluation result

{}

## 📁 generated files

- traditional_export_run_*.json - traditional export results
- fast_export_run_*.json - fast export results
- simple_benchmark_report.md - this report
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
            "✅ excellent! reached 60-80% export time reduction target"
        } else if improvement_percent >= 40.0 {
            "✅ good! close to 60-80% export time reduction target"
        } else if improvement_percent >= 20.0 {
            "⚠️ general, some improvement but not met the target"
        } else if improvement_percent > 0.0 {
            "⚠️ minor improvement, need further optimization"
        } else {
            "❌ performance not improved, need to re-examine the algorithm"
        }
    );

    if let Err(e) = fs::write(&report_file, report) {
        tracing::error!("⚠️  genarate report failed: {}", e);
    } else {
        tracing::info!("");
        tracing::info!("📄 report generated: {}", report_file.display());
    }
}
