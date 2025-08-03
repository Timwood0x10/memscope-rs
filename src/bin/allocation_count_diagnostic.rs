//! allocation count diagnostic program
//!
//! check why the exported file size does not linearly increase with allocation count

use memscope_rs::{get_global_tracker, init};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    tracing::info!("🔍 allocation count diagnostic program");
    tracing::info!("==================");
    tracing::info!("");

    // init memory tracker
    init();

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
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // 诊断分配数量
    diagnose_allocation_count();
}

fn diagnose_allocation_count() {
    tracing::info!("");
    tracing::info!("📊 diagnose allocation count and file size relationship");
    tracing::info!("==============================");

    let tracker = get_global_tracker();
    let stats = tracker.get_stats().unwrap();

    tracing::info!("🔍 global tracker stats:");
    tracing::info!("  • total allocations: {}", stats.total_allocations);
    tracing::info!("  • active allocations: {}", stats.active_allocations);
    tracing::info!(
        "  • peak memory: {:.2} MB",
        stats.peak_memory as f64 / 1024.0 / 1024.0
    );
    tracing::info!(
        "  • current memory: {:.2} MB",
        stats.active_memory as f64 / 1024.0 / 1024.0
    );

    // create output directory
    let output_dir = PathBuf::from("diagnostic_results");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        tracing::error!("❌ create output directory failed: {}", e);
        return;
    }

    // test different export methods
    test_traditional_export(&output_dir, &stats);
    test_fast_export(&output_dir, &stats);
    test_raw_data_access(&stats);
}

fn test_traditional_export(output_dir: &PathBuf, stats: &memscope_rs::core::types::MemoryStats) {
    tracing::info!("");
    tracing::info!("🐌 test traditional export:");

    let output_path = output_dir.join("traditional_diagnostic.json");
    let tracker = get_global_tracker();

    match tracker.export_to_json(&output_path) {
        Ok(_) => {
            if let Ok(metadata) = fs::metadata(&output_path) {
                let file_size = metadata.len();
                tracing::info!(
                    "  • file size: {:.2} MB ({} bytes)",
                    file_size as f64 / 1024.0 / 1024.0,
                    file_size
                );
                tracing::info!(
                    "  • average allocation size: {:.1} bytes",
                    file_size as f64 / stats.total_allocations as f64
                );

                // read file content analysis
                if let Ok(content) = fs::read_to_string(&output_path) {
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(allocations) = json_value.get("allocations") {
                            if let Some(alloc_array) = allocations.as_array() {
                                tracing::info!("  • JSON allocations count: {}", alloc_array.len());
                                tracing::info!("  • tracker reported allocations count: {}", stats.total_allocations);
                                if alloc_array.len() != stats.total_allocations {
                                    tracing::info!("  ⚠️  allocations count mismatch! possible data loss");
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("  ❌ traditional export failed: {}", e);
        }
    }
}

fn test_fast_export(output_dir: &PathBuf, stats: &memscope_rs::core::types::MemoryStats) {
    tracing::info!("");
    tracing::info!("⚡ test fast export:");

    let output_path = output_dir.join("fast_diagnostic");

    // use fast export coordinator
    let config = memscope_rs::export::fast_export_coordinator::FastExportConfig::default();
    let mut coordinator =
        memscope_rs::export::fast_export_coordinator::FastExportCoordinator::new(config);

    match coordinator.export_fast(&output_path) {
        Ok(export_stats) => {
            tracing::info!(
                "  • processed allocations count: {}",
                export_stats.parallel_processing.total_allocations
            );
            tracing::info!(
                "  • written bytes: {:.2} MB ({} bytes)",
                export_stats.write_performance.total_bytes_written as f64 / 1024.0 / 1024.0,
                export_stats.write_performance.total_bytes_written
            );
            tracing::info!(
                "  • shard count: {}",
                export_stats.parallel_processing.shard_count
            );
            tracing::info!(
                "  • average allocation size: {:.1} bytes",
                export_stats.write_performance.total_bytes_written as f64
                    / export_stats.parallel_processing.total_allocations as f64
            );

            if export_stats.parallel_processing.total_allocations != stats.total_allocations {
                tracing::info!(
                    "  ⚠️  processed allocations count mismatch! possible data loss",
                    export_stats.parallel_processing.total_allocations, stats.total_allocations
                );
            }
        }
        Err(e) => {
            tracing::error!("  ❌ fast export failed: {}", e);
        }
    }
}

fn test_raw_data_access(stats: &memscope_rs::core::types::MemoryStats) {
    tracing::info!("");
    tracing::info!("🔍 test raw data access:");

    let _tracker = get_global_tracker();

    // try to access allocation data directly
    tracing::info!("  • try to access allocation data directly...");

    // we can only infer through statistics
    tracing::info!("  • tracker stats:");
    tracing::info!("    - total allocations: {}", stats.total_allocations);
    tracing::info!("    - active allocations: {}", stats.active_allocations);
    tracing::info!(
        "    - released allocations: {}",
        stats.total_allocations - stats.active_allocations
    );

    if stats.active_allocations < stats.total_allocations {
        tracing::info!(
            "  💡Find {} released allocations: this may affect the export data volume",
            stats.total_allocations - stats.active_allocations
        );
    }
}
