//! SIMD demonstration for both x86_64 and ARM64 architectures
//!
//! This example demonstrates the SIMD capabilities of the binary export system
//! and provides benchmarks for different architectures.

use memscope_rs::export::simd_optimizations::{SimdCapability, SimdProcessor};

fn main() {
    println!("=== SIMD Capabilities Demo ===");
    println!("Platform: {}", std::env::consts::ARCH);

    let processor = SimdProcessor::new();
    let capability = processor.capability();

    println!("Detected SIMD capability: {:?}", capability);
    println!("SIMD enabled: {}", processor.is_enabled());

    // Test CRC64 calculation
    test_crc64_functionality(&processor);

    // Test u64 array encoding/decoding
    test_u64_array_operations(&processor);

    // Test memory comparison
    test_memory_comparison(&processor);

    // Run performance benchmarks
    run_performance_benchmarks(&processor);

    println!("\n=== Demo completed successfully! ===");
}

fn test_crc64_functionality(processor: &SimdProcessor) {
    println!("\n--- Testing CRC64 Functionality ---");

    let test_data = b"Hello, SIMD world! This is a test string for CRC64 calculation.";

    let simd_checksum = processor.crc64_checksum(test_data);
    let scalar_checksum = processor.crc64_scalar(test_data);

    println!("Test data: {:?}", std::str::from_utf8(test_data).unwrap());
    println!("SIMD CRC64:   {:016x}", simd_checksum);
    println!("Scalar CRC64: {:016x}", scalar_checksum);

    if simd_checksum == scalar_checksum {
        println!("✅ CRC64 consistency check passed");
    } else {
        println!("❌ CRC64 consistency check failed");
    }
}

fn test_u64_array_operations(processor: &SimdProcessor) {
    println!("\n--- Testing U64 Array Operations ---");

    let test_values = vec![
        0x123456789ABCDEFu64,
        0xFEDCBA9876543210u64,
        0x0000000000000000u64,
        0xFFFFFFFFFFFFFFFFu64,
        0x5555555555555555u64,
        0xAAAAAAAAAAAAAAAAu64,
    ];

    let mut simd_output = vec![0u8; test_values.len() * 8];
    let mut scalar_output = vec![0u8; test_values.len() * 8];

    // Test encoding
    let simd_encode_result = processor.encode_u64_array(&test_values, &mut simd_output);
    let scalar_encode_result = processor.encode_u64_array_scalar(&test_values, &mut scalar_output);

    match (simd_encode_result, scalar_encode_result) {
        (Ok(simd_len), Ok(scalar_len)) => {
            println!(
                "Encoded {} values ({} bytes each)",
                test_values.len(),
                simd_len
            );

            if simd_output == scalar_output && simd_len == scalar_len {
                println!("✅ U64 encoding consistency check passed");
            } else {
                println!("❌ U64 encoding consistency check failed");
                return;
            }

            // Test decoding
            let mut simd_decoded = vec![0u64; test_values.len()];
            let mut scalar_decoded = vec![0u64; test_values.len()];

            let simd_decode_result = processor.decode_u64_array(&simd_output, &mut simd_decoded);
            let scalar_decode_result =
                processor.decode_u64_array_scalar(&scalar_output, &mut scalar_decoded);

            match (simd_decode_result, scalar_decode_result) {
                (Ok(_), Ok(_)) => {
                    if simd_decoded == scalar_decoded && simd_decoded == test_values {
                        println!("✅ U64 decoding and round-trip check passed");
                    } else {
                        println!("❌ U64 decoding or round-trip check failed");
                    }
                }
                _ => println!("❌ U64 decoding failed"),
            }
        }
        _ => println!("❌ U64 encoding failed"),
    }
}

fn test_memory_comparison(processor: &SimdProcessor) {
    println!("\n--- Testing Memory Comparison ---");

    let data1 = vec![0xAAu8; 256];
    let data2 = vec![0xAAu8; 256];
    let mut data3 = vec![0xAAu8; 256];
    data3[128] = 0xBB; // Make one byte different

    // Test identical data
    let simd_result1 = processor.memory_compare(&data1, &data2);
    let scalar_result1 = data1 == data2;

    // Test different data
    let simd_result2 = processor.memory_compare(&data1, &data3);
    let scalar_result2 = data1 == data3;

    println!(
        "Identical data - SIMD: {}, Scalar: {}",
        simd_result1, scalar_result1
    );
    println!(
        "Different data - SIMD: {}, Scalar: {}",
        simd_result2, scalar_result2
    );

    if simd_result1 == scalar_result1 && simd_result2 == scalar_result2 {
        println!("✅ Memory comparison consistency check passed");
    } else {
        println!("❌ Memory comparison consistency check failed");
    }
}

fn run_performance_benchmarks(processor: &SimdProcessor) {
    println!("\n--- Performance Benchmarks ---");

    let data_sizes = vec![1024, 4096, 16384];
    let iterations = 1000;

    for size in data_sizes {
        println!(
            "\nBenchmarking with {} bytes, {} iterations:",
            size, iterations
        );

        // CRC64 benchmark
        let data = vec![0xAAu8; size];

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = processor.crc64_checksum(&data);
        }
        let simd_time = start.elapsed();

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = processor.crc64_scalar(&data);
        }
        let scalar_time = start.elapsed();

        let speedup = scalar_time.as_nanos() as f64 / simd_time.as_nanos() as f64;
        let simd_throughput =
            (size * iterations) as f64 / (1024.0 * 1024.0) / simd_time.as_secs_f64();
        let scalar_throughput =
            (size * iterations) as f64 / (1024.0 * 1024.0) / scalar_time.as_secs_f64();

        println!("  CRC64:");
        println!(
            "    SIMD:   {:.2} MB/s ({:.2} ms)",
            simd_throughput,
            simd_time.as_millis()
        );
        println!(
            "    Scalar: {:.2} MB/s ({:.2} ms)",
            scalar_throughput,
            scalar_time.as_millis()
        );
        println!("    Speedup: {:.2}x", speedup);

        // U64 encoding benchmark
        let values: Vec<u64> = (0..size / 8).map(|i| i as u64).collect();
        let mut output = vec![0u8; values.len() * 8];

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = processor.encode_u64_array(&values, &mut output);
        }
        let simd_encode_time = start.elapsed();

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = processor.encode_u64_array_scalar(&values, &mut output);
        }
        let scalar_encode_time = start.elapsed();

        let encode_speedup =
            scalar_encode_time.as_nanos() as f64 / simd_encode_time.as_nanos() as f64;
        let simd_encode_throughput = (values.len() * 8 * iterations) as f64
            / (1024.0 * 1024.0)
            / simd_encode_time.as_secs_f64();
        let scalar_encode_throughput = (values.len() * 8 * iterations) as f64
            / (1024.0 * 1024.0)
            / scalar_encode_time.as_secs_f64();

        println!("  U64 Encoding:");
        println!(
            "    SIMD:   {:.2} MB/s ({:.2} ms)",
            simd_encode_throughput,
            simd_encode_time.as_millis()
        );
        println!(
            "    Scalar: {:.2} MB/s ({:.2} ms)",
            scalar_encode_throughput,
            scalar_encode_time.as_millis()
        );
        println!("    Speedup: {:.2}x", encode_speedup);
    }
}

#[cfg(target_arch = "x86_64")]
fn print_x86_64_features() {
    println!("\n--- x86_64 SIMD Features ---");
    println!("SSE2:    {}", is_x86_feature_detected!("sse2"));
    println!("SSE4.1:  {}", is_x86_feature_detected!("sse4.1"));
    println!("AVX:     {}", is_x86_feature_detected!("avx"));
    println!("AVX2:    {}", is_x86_feature_detected!("avx2"));
    println!("AVX512F: {}", is_x86_feature_detected!("avx512f"));
}

#[cfg(target_arch = "aarch64")]
fn print_aarch64_features() {
    println!("\n--- ARM64 SIMD Features ---");
    println!("NEON: {}", std::arch::is_aarch64_feature_detected!("neon"));
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
fn print_unsupported_arch() {
    println!("\n--- Unsupported Architecture ---");
    println!("SIMD optimizations are not available for this architecture");
}
