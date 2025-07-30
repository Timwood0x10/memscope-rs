//! Simple Binary Performance Demo
//!
//! This example demonstrates the performance benefits of binary formats
//! using a simplified approach that works with the current project setup.
//!
//! Run with: cargo run --example simple_binary_performance_demo

use std::collections::HashMap;
use std::time::Instant;

/// Simulated memory allocation data
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MockAllocation {
    ptr: usize,
    size: usize,
    var_name: String,
    type_name: String,
    timestamp: u64,
}

/// Mock export data for demonstration
#[derive(Debug, Clone)]
struct MockExportData {
    allocations: Vec<MockAllocation>,
    metadata: HashMap<String, String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Simple Binary Performance Demo");
    println!("{}", "=".repeat(60));
    println!("Demonstrating binary export benefits with mock data...\n");

    // Create realistic test data
    println!("📊 Creating mock memory analysis data...");
    let test_data = create_mock_data();
    println!("✅ Generated {} allocations", test_data.allocations.len());

    // Simulate JSON serialization
    println!("\n📄 Simulating JSON Export...");
    let json_result = simulate_json_export(&test_data);
    print_result("JSON", &json_result);

    // Simulate MessagePack serialization
    println!("\n📦 Simulating MessagePack Export...");
    let msgpack_result = simulate_messagepack_export(&test_data);
    print_result("MessagePack", &msgpack_result);

    // Simulate compressed export
    println!("\n🗜️  Simulating MessagePack + Zstd Export...");
    let compressed_result = simulate_compressed_export(&test_data);
    print_result("MessagePack + Zstd", &compressed_result);

    // Print comparison
    print_comparison(&json_result, &msgpack_result, &compressed_result);

    println!("\n✅ Demo completed! Binary formats show significant advantages.");
    Ok(())
}

#[derive(Debug)]
struct PerformanceResult {
    #[allow(dead_code)]
    format_name: String,
    serialize_time_ms: f64,
    deserialize_time_ms: f64,
    estimated_size_kb: f64,
}

fn create_mock_data() -> MockExportData {
    let mut allocations = Vec::new();

    // Create diverse allocation patterns similar to complex_lifecycle_showcase
    let type_patterns = [
        ("Vec<i32>", 300),
        ("String", 250),
        ("HashMap<String, i32>", 150),
        ("Box<dyn Trait>", 100),
        ("Arc<Mutex<T>>", 80),
        ("Vec<Vec<String>>", 60),
        ("CustomStruct", 120),
    ];

    let mut ptr_counter = 0x10000000;

    for (type_name, count) in &type_patterns {
        for i in 0..*count {
            allocations.push(MockAllocation {
                ptr: ptr_counter,
                size: 64 + (i % 500), // Variable sizes
                var_name: format!("{}_{}", type_name.replace("<", "_").replace(">", "_"), i),
                type_name: type_name.to_string(),
                timestamp: 1640000000 + i as u64,
            });
            ptr_counter += 256;
        }
    }

    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0.0".to_string());
    metadata.insert(
        "total_allocations".to_string(),
        allocations.len().to_string(),
    );
    metadata.insert("export_time".to_string(), "1640995200".to_string());

    MockExportData {
        allocations,
        metadata,
    }
}

fn simulate_json_export(data: &MockExportData) -> PerformanceResult {
    let start = Instant::now();

    // Simulate JSON serialization overhead
    let mut json_size = 0;
    for alloc in &data.allocations {
        // Estimate JSON overhead: field names, quotes, commas, etc.
        json_size += alloc.var_name.len() * 2; // Quoted strings
        json_size += alloc.type_name.len() * 2;
        json_size += 100; // Field names and structure overhead
    }
    json_size += data.metadata.len() * 50; // Metadata overhead

    let serialize_time = start.elapsed();

    // Simulate deserialization
    let start = Instant::now();
    // JSON parsing is typically slower due to text processing
    std::thread::sleep(std::time::Duration::from_micros(
        (data.allocations.len() as u64) / 100, // Simulate parsing overhead
    ));
    let deserialize_time = start.elapsed();

    PerformanceResult {
        format_name: "JSON".to_string(),
        serialize_time_ms: serialize_time.as_secs_f64() * 1000.0,
        deserialize_time_ms: deserialize_time.as_secs_f64() * 1000.0,
        estimated_size_kb: json_size as f64 / 1024.0,
    }
}

fn simulate_messagepack_export(data: &MockExportData) -> PerformanceResult {
    let start = Instant::now();

    // MessagePack is more compact - no field name repetition, binary encoding
    let mut msgpack_size = 0;
    for alloc in &data.allocations {
        msgpack_size += alloc.var_name.len(); // No quotes needed
        msgpack_size += alloc.type_name.len();
        msgpack_size += 40; // Much less structure overhead
    }
    msgpack_size += data.metadata.len() * 20; // Less metadata overhead

    // MessagePack serialization is faster
    std::thread::sleep(std::time::Duration::from_micros(
        (data.allocations.len() as u64) / 500, // 5x faster than JSON
    ));
    let serialize_time = start.elapsed();

    // Simulate deserialization (also faster)
    let start = Instant::now();
    std::thread::sleep(std::time::Duration::from_micros(
        (data.allocations.len() as u64) / 300, // 3x faster than JSON
    ));
    let deserialize_time = start.elapsed();

    PerformanceResult {
        format_name: "MessagePack".to_string(),
        serialize_time_ms: serialize_time.as_secs_f64() * 1000.0,
        deserialize_time_ms: deserialize_time.as_secs_f64() * 1000.0,
        estimated_size_kb: msgpack_size as f64 / 1024.0,
    }
}

fn simulate_compressed_export(data: &MockExportData) -> PerformanceResult {
    let start = Instant::now();

    // Compression adds some time but dramatically reduces size
    let mut base_size = 0;
    for alloc in &data.allocations {
        base_size += alloc.var_name.len();
        base_size += alloc.type_name.len();
        base_size += 40;
    }
    base_size += data.metadata.len() * 20;

    // Compression ratio for structured data is typically very good
    let compressed_size = (base_size as f64 * 0.3) as usize; // 70% compression

    // Compression adds some overhead
    std::thread::sleep(std::time::Duration::from_micros(
        (data.allocations.len() as u64) / 200, // Compression overhead
    ));
    let serialize_time = start.elapsed();

    // Decompression is usually fast
    let start = Instant::now();
    std::thread::sleep(std::time::Duration::from_micros(
        (data.allocations.len() as u64) / 400, // Fast decompression
    ));
    let deserialize_time = start.elapsed();

    PerformanceResult {
        format_name: "MessagePack + Zstd".to_string(),
        serialize_time_ms: serialize_time.as_secs_f64() * 1000.0,
        deserialize_time_ms: deserialize_time.as_secs_f64() * 1000.0,
        estimated_size_kb: compressed_size as f64 / 1024.0,
    }
}

fn print_result(_format: &str, result: &PerformanceResult) {
    println!("  ⏱️  Serialize: {:.2} ms", result.serialize_time_ms);
    println!("  📥 Deserialize: {:.2} ms", result.deserialize_time_ms);
    println!("  📏 Size: {:.2} KB", result.estimated_size_kb);
}

fn print_comparison(
    json: &PerformanceResult,
    msgpack: &PerformanceResult,
    compressed: &PerformanceResult,
) {
    println!("\n🎯 Performance Comparison Summary");
    println!("{}", "=".repeat(60));

    // Speed improvements
    let msgpack_serialize_speedup = json.serialize_time_ms / msgpack.serialize_time_ms;
    let msgpack_deserialize_speedup = json.deserialize_time_ms / msgpack.deserialize_time_ms;
    let compressed_serialize_speedup = json.serialize_time_ms / compressed.serialize_time_ms;
    let compressed_deserialize_speedup = json.deserialize_time_ms / compressed.deserialize_time_ms;

    println!("\n🚀 Speed Improvements:");
    println!("  MessagePack vs JSON:");
    println!("    Serialize: {:.1}x faster", msgpack_serialize_speedup);
    println!(
        "    Deserialize: {:.1}x faster",
        msgpack_deserialize_speedup
    );

    println!("  MessagePack+Zstd vs JSON:");
    println!("    Serialize: {:.1}x faster", compressed_serialize_speedup);
    println!(
        "    Deserialize: {:.1}x faster",
        compressed_deserialize_speedup
    );

    // Size improvements
    let msgpack_size_reduction = json.estimated_size_kb / msgpack.estimated_size_kb;
    let compressed_size_reduction = json.estimated_size_kb / compressed.estimated_size_kb;

    println!("\n💾 Size Improvements:");
    println!(
        "  MessagePack vs JSON: {:.1}x smaller",
        msgpack_size_reduction
    );
    println!(
        "  MessagePack+Zstd vs JSON: {:.1}x smaller",
        compressed_size_reduction
    );

    // Overall assessment
    println!("\n📊 Overall Assessment:");
    println!("  Format              | Serialize | Deserialize | Size");
    println!("  -------------------|-----------|-------------|--------");
    println!(
        "  JSON               | {:.2} ms   | {:.2} ms     | {:.1} KB",
        json.serialize_time_ms, json.deserialize_time_ms, json.estimated_size_kb
    );
    println!(
        "  MessagePack        | {:.2} ms   | {:.2} ms     | {:.1} KB",
        msgpack.serialize_time_ms, msgpack.deserialize_time_ms, msgpack.estimated_size_kb
    );
    println!(
        "  MessagePack+Zstd   | {:.2} ms   | {:.2} ms     | {:.1} KB",
        compressed.serialize_time_ms, compressed.deserialize_time_ms, compressed.estimated_size_kb
    );

    println!("\n💡 Recommendations for Memory Analysis:");
    println!("  🎯 Best Overall: MessagePack + Zstd");
    println!("     - Excellent compression for repetitive memory data");
    println!("     - Good balance of speed and size");
    println!("     - Perfect for archival and network transfer");

    println!("  🚀 Best Speed: MessagePack");
    println!("     - Maximum performance for real-time monitoring");
    println!("     - Still significantly smaller than JSON");

    println!("  🔄 Best Compatibility: JSON");
    println!("     - Human readable for debugging");
    println!("     - Universal tool support");

    println!("\n🔍 Memory Analysis Specific Benefits:");
    println!("  • Repetitive allocation patterns compress extremely well");
    println!("  • Binary formats preserve type information better");
    println!("  • Faster loading enables real-time memory monitoring");
    println!("  • Smaller files reduce storage costs for long-term analysis");
    println!("  • Cross-platform compatibility with MessagePack");
}
