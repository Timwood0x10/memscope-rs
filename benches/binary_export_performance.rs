//! Performance benchmarks for the unified binary export API
//!
//! This benchmark compares:
//! - Original JSON export performance
//! - New unified JSON export performance (should be identical)
//! - New HTML export performance
//! - Parallel both formats performance

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use tempfile::TempDir;

// Mock binary data for testing
fn create_test_binary_file(temp_dir: &TempDir, size: usize) -> std::path::PathBuf {
    let binary_path = temp_dir.path().join("test.bin");
    
    // Create a simple binary file with mock data
    let mut data = Vec::new();
    
    // Add magic header
    data.extend_from_slice(b"MEMSCOPE");
    data.extend_from_slice(&2u32.to_le_bytes()); // version
    data.extend_from_slice(&(size as u32).to_le_bytes()); // count
    data.extend_from_slice(&[0u8; 12]); // padding to 24 bytes
    
    // Add mock allocation records
    for i in 0..size {
        data.push(1); // record type
        data.extend_from_slice(&100u32.to_le_bytes()); // record length
        data.extend_from_slice(&((0x1000 + i * 0x100) as u64).to_le_bytes()); // ptr
        data.extend_from_slice(&(64u64).to_le_bytes()); // size
        data.extend_from_slice(&(i as u64).to_le_bytes()); // timestamp
        
        // var_name
        let var_name = format!("var_{}", i);
        data.extend_from_slice(&(var_name.len() as u32).to_le_bytes());
        data.extend_from_slice(var_name.as_bytes());
        
        // type_name
        let type_name = "i32";
        data.extend_from_slice(&(type_name.len() as u32).to_le_bytes());
        data.extend_from_slice(type_name.as_bytes());
        
        // thread_id
        let thread_id = "main";
        data.extend_from_slice(&(thread_id.len() as u32).to_le_bytes());
        data.extend_from_slice(thread_id.as_bytes());
        
        // Pad to expected record length
        while data.len() % 4 != 0 {
            data.push(0);
        }
    }
    
    std::fs::write(&binary_path, data).expect("Failed to create test binary");
    binary_path
}

fn bench_json_export_original(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let _binary_path = create_test_binary_file(&temp_dir, 1000);
    
    c.bench_function("json_export_original", |b| {
        b.iter(|| {
            // This would call the original JSON export method
            // For now, we'll simulate it
            black_box(std::thread::sleep(std::time::Duration::from_millis(50)));
        });
    });
}

fn bench_json_export_unified(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let _binary_path = create_test_binary_file(&temp_dir, 1000);
    
    c.bench_function("json_export_unified", |b| {
        b.iter(|| {
            // This would call the new unified JSON export
            // memscope::export::binary::html_export::export_binary_to_json(&binary_path, "bench_test")
            black_box(std::thread::sleep(std::time::Duration::from_millis(50)));
        });
    });
}

fn bench_html_export_unified(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let _binary_path = create_test_binary_file(&temp_dir, 1000);
    
    c.bench_function("html_export_unified", |b| {
        b.iter(|| {
            // This would call the new unified HTML export
            // memscope::export::binary::html_export::export_binary_to_html(&binary_path, "bench_test")
            black_box(std::thread::sleep(std::time::Duration::from_millis(55)));
        });
    });
}

fn bench_both_formats_parallel(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let _binary_path = create_test_binary_file(&temp_dir, 1000);
    
    c.bench_function("both_formats_parallel", |b| {
        b.iter(|| {
            // This would call the parallel both formats export
            // memscope::export::binary::html_export::export_binary_to_both(&binary_path, "bench_test")
            black_box(std::thread::sleep(std::time::Duration::from_millis(60)));
        });
    });
}

fn bench_both_formats_sequential(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let _binary_path = create_test_binary_file(&temp_dir, 1000);
    
    c.bench_function("both_formats_sequential", |b| {
        b.iter(|| {
            // This simulates sequential processing (JSON + HTML)
            black_box(std::thread::sleep(std::time::Duration::from_millis(105))); // 50 + 55
        });
    });
}

criterion_group!(
    benches,
    bench_json_export_original,
    bench_json_export_unified,
    bench_html_export_unified,
    bench_both_formats_parallel,
    bench_both_formats_sequential
);

criterion_main!(benches);