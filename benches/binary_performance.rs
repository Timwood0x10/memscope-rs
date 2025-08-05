//! Performance benchmarks for binary export functionality

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use memscope_rs::core::types::AllocationInfo;
use memscope_rs::export::binary;
use std::time::Instant;
use tempfile::TempDir;

/// Create test allocation data with specified count
fn create_test_allocations(count: usize) -> Vec<AllocationInfo> {
    (0..count)
        .map(|i| AllocationInfo {
            ptr: 0x1000 + (i * 0x100),
            size: 1024 + (i % 512),
            var_name: Some(format!("test_var_{}", i)),
            type_name: Some(format!("TestType{}", i % 10)),
            scope_name: None,
            timestamp_alloc: 1234567890 + i as u64,
            timestamp_dealloc: None,
            thread_id: format!("thread_{}", i % 4),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        })
        .collect()
}

/// Benchmark binary export performance
fn bench_binary_export(c: &mut Criterion) {
    let mut group = c.benchmark_group("binary_export");

    for size in [100, 1000, 5000, 10000].iter() {
        let allocations = create_test_allocations(*size);

        group.bench_with_input(BenchmarkId::new("export", size), size, |b, _| {
            b.iter(|| {
                let temp_dir = TempDir::new().unwrap();
                let binary_path = temp_dir.path().join("benchmark.memscope");

                let result =
                    binary::export_to_binary(black_box(&allocations), black_box(&binary_path));

                assert!(result.is_ok());
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark binary read performance
fn bench_binary_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("binary_read");

    for size in [100, 1000, 5000, 10000].iter() {
        let allocations = create_test_allocations(*size);
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("benchmark.memscope");

        // Pre-create the binary file
        binary::export_to_binary(&allocations, &binary_path).unwrap();

        group.bench_with_input(BenchmarkId::new("read", size), size, |b, _| {
            b.iter(|| {
                let mut reader = binary::BinaryReader::new(black_box(&binary_path)).unwrap();
                let result = reader.read_all();

                assert!(result.is_ok());
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark JSON conversion performance
fn bench_json_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_conversion");

    for size in [100, 1000, 5000, 10000].iter() {
        let allocations = create_test_allocations(*size);
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("benchmark.memscope");
        let json_path = temp_dir.path().join("benchmark.json");

        // Pre-create the binary file
        binary::export_to_binary(&allocations, &binary_path).unwrap();

        group.bench_with_input(BenchmarkId::new("binary_to_json", size), size, |b, _| {
            b.iter(|| {
                let result =
                    binary::parse_binary_to_json(black_box(&binary_path), black_box(&json_path));

                assert!(result.is_ok());
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Compare binary vs JSON export performance
fn bench_format_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_comparison");

    let allocations = create_test_allocations(1000);

    group.bench_function("binary_export_1k", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let binary_path = temp_dir.path().join("test.memscope");

            let result = binary::export_to_binary(black_box(&allocations), black_box(&binary_path));

            assert!(result.is_ok());
            black_box(result)
        });
    });

    group.bench_function("json_export_1k", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let json_path = temp_dir.path().join("test.json");

            let start = Instant::now();
            let json_data = serde_json::to_string_pretty(black_box(&allocations)).unwrap();
            std::fs::write(black_box(&json_path), json_data).unwrap();
            let duration = start.elapsed();

            black_box(duration)
        });
    });

    group.finish();
}

/// Benchmark file size comparison
fn bench_file_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_sizes");

    for size in [100, 1000, 5000].iter() {
        let allocations = create_test_allocations(*size);

        group.bench_with_input(BenchmarkId::new("size_comparison", size), size, |b, _| {
            b.iter(|| {
                let temp_dir = TempDir::new().unwrap();
                let binary_path = temp_dir.path().join("test.memscope");
                let json_path = temp_dir.path().join("test.json");

                // Export to binary
                binary::export_to_binary(&allocations, &binary_path).unwrap();
                let binary_size = std::fs::metadata(&binary_path).unwrap().len();

                // Export to JSON
                let json_data = serde_json::to_string_pretty(&allocations).unwrap();
                std::fs::write(&json_path, json_data).unwrap();
                let json_size = std::fs::metadata(&json_path).unwrap().len();

                let compression_ratio = (json_size as f64 - binary_size as f64) / json_size as f64;

                black_box((binary_size, json_size, compression_ratio))
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_binary_export,
    bench_binary_read,
    bench_json_conversion,
    bench_format_comparison,
    bench_file_sizes
);

criterion_main!(benches);
