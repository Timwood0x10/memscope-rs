// Enhanced simple showcase with complex user variables
// Focus on trackable types that will create interesting JSON output

use memscope_rs::{get_global_tracker, init, track_var};
use memscope_rs::analysis::unsafe_ffi_tracker::{get_global_unsafe_ffi_tracker, BoundaryEventType};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::ffi::CString;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

fn main() {
    init();
    println!("🚀 Enhanced Simple Showcase - Complex User Variables");
    println!("===================================================");
    println!("Creating many complex user variables to test optimized conversion");
    println!();

    let total_start = Instant::now();

    // Phase 1: Basic complex collections
    println!("📊 Phase 1: Complex Collections");
    create_complex_collections();

    // Phase 2: String operations and FFI
    println!("🔗 Phase 2: String Operations & FFI");
    create_string_and_ffi_operations();

    // Phase 3: Smart pointers
    println!("🧠 Phase 3: Smart Pointers");
    create_smart_pointers();

    // Phase 4: Large data structures
    println!("📈 Phase 4: Large Data Structures");
    create_large_data_structures();

    // Phase 5: Nested structures
    println!("🏗️  Phase 5: Nested Structures");
    create_nested_structures();

    let data_collection_time = total_start.elapsed();

    // Generate binary export and test conversion
    println!("\n📊 Final Analysis & Binary Export");
    println!("=================================");
    let tracker = get_global_tracker();
    let stats = tracker.get_stats().unwrap();
    
    println!("Memory Statistics (collected in {:.2}ms):", data_collection_time.as_millis());
    println!("  • Total allocations: {}", stats.total_allocations);
    println!("  • Active allocations: {}", stats.active_allocations);
    println!("  • Active memory: {} bytes ({:.2} MB)", stats.active_memory, stats.active_memory as f64 / 1024.0 / 1024.0);
    println!("  • Peak memory: {} bytes ({:.2} MB)", stats.peak_memory, stats.peak_memory as f64 / 1024.0 / 1024.0);

    // Export to binary and test conversion performance - comparing user vs full modes
    println!("\n🎨 Binary Export and Conversion Performance Comparison:");
    
    // Create directory first
    std::fs::create_dir_all("MemoryAnalysis/enhanced_simple").unwrap();
    
    // Test user binary export (only user-defined variables)
    let user_binary_start = Instant::now();
    tracker.export_user_binary("enhanced_simple/enhanced_simple_user.memscope").unwrap();
    let user_binary_time = user_binary_start.elapsed();
    
    // Test full binary export (all allocations including system)
    let full_binary_start = Instant::now();
    tracker.export_full_binary("enhanced_simple/enhanced_simple_full.memscope").unwrap();
    let full_binary_time = full_binary_start.elapsed();
    
    // Test JSON conversion from user binary
    let user_json_start = Instant::now();
    memscope_rs::export::binary::BinaryParser::to_standard_json_files(
        "MemoryAnalysis/enhanced_simple/enhanced_simple_user.memscope",
        "enhanced_simple_user"
    ).unwrap();
    let user_json_time = user_json_start.elapsed();
    
    // Test JSON conversion from full binary  
    let full_json_start = Instant::now();
    memscope_rs::export::binary::BinaryParser::to_standard_json_files(
        "MemoryAnalysis/enhanced_simple/enhanced_simple_full.memscope",
        "enhanced_simple_full"
    ).unwrap();
    let full_json_time = full_json_start.elapsed();

    println!("✅ User binary export completed in {:.2}ms", user_binary_time.as_millis());
    println!("✅ Full binary export completed in {:.2}ms", full_binary_time.as_millis());
    println!("✅ User binary -> JSON conversion completed in {:.2}ms", user_json_time.as_millis());
    println!("✅ Full binary -> JSON conversion completed in {:.2}ms", full_json_time.as_millis());

    let total_time = total_start.elapsed();
    println!("\n📊 Performance Summary Comparison:");
    println!("  User binary export: {:.2}ms", user_binary_time.as_millis());
    println!("  Full binary export: {:.2}ms", full_binary_time.as_millis());
    println!("  User binary -> JSON: {:.2}ms", user_json_time.as_millis());
    println!("  Full binary -> JSON: {:.2}ms", full_json_time.as_millis());
    println!("  Total workflow: {:.2}ms", total_time.as_millis());

    println!("\n🎯 Enhanced User Variable Analysis:");
    println!("===================================");
    println!("This test creates many complex user-defined variables including:");
    println!("✓ Large collections (HashMap, Vec, BTreeMap)");
    println!("✓ String operations and C strings");
    println!("✓ Smart pointers (Box, Rc, Arc)");
    println!("✓ Nested data structures");
    println!("✓ Large data sets");
    println!("\nGenerated files in MemoryAnalysis/enhanced_simple/:");
    println!("  User Binary Mode (smaller, faster):");
    println!("    1. enhanced_simple_user.memscope - Binary with user variables only");
    println!("    2. enhanced_simple_user_*.json - 5 JSON analysis files from user data");
    println!("  Full Binary Mode (complete, optimized):");
    println!("    3. enhanced_simple_full.memscope - Binary with all allocations");
    println!("    4. enhanced_simple_full_*.json - 5 JSON analysis files from full data");
}

fn create_complex_collections() {
    // Large HashMap with complex keys and values
    let user_database: HashMap<String, String> = (0..1000)
        .map(|i| (format!("user_id_{}", i), format!("user_data_complex_value_{}", i)))
        .collect();
    track_var!(user_database);

    // BTreeMap for ordered data
    let ordered_config: BTreeMap<String, String> = (0..500)
        .map(|i| (format!("config_key_{:04}", i), format!("config_value_{}", i)))
        .collect();
    track_var!(ordered_config);

    // HashSet with complex data
    let unique_identifiers: HashSet<String> = (0..300)
        .map(|i| format!("unique_id_{}_{}", i, i * 2))
        .collect();
    track_var!(unique_identifiers);

    // VecDeque for queue operations
    let message_queue: VecDeque<String> = (0..200)
        .map(|i| format!("message_{}_priority_{}", i, i % 10))
        .collect();
    track_var!(message_queue);

    // Nested HashMap
    let nested_cache: HashMap<String, HashMap<String, String>> = (0..50)
        .map(|i| {
            let inner: HashMap<String, String> = (0..20)
                .map(|j| (format!("inner_key_{}_{}", i, j), format!("inner_value_{}_{}", i, j)))
                .collect();
            (format!("outer_key_{}", i), inner)
        })
        .collect();
    track_var!(nested_cache);

    println!("  ✅ Created complex collections with thousands of entries");
}

fn create_string_and_ffi_operations() {
    // Large strings
    let large_text_buffer = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(1000);
    track_var!(large_text_buffer);

    let json_like_string = serde_json::json!({
        "users": (0..100).map(|i| serde_json::json!({
            "id": i,
            "name": format!("User {}", i),
            "email": format!("user{}@example.com", i),
            "active": i % 2 == 0
        })).collect::<Vec<_>>(),
        "metadata": {
            "total": 100,
            "created_at": "2024-01-01T00:00:00Z",
            "version": "1.0"
        }
    }).to_string();
    track_var!(json_like_string);

    // C strings for FFI simulation
    let c_string_1 = CString::new("Hello from C world!").unwrap();
    track_var!(c_string_1);

    let c_string_2 = CString::new("Complex C string with special chars: àáâãäåæçèéêë").unwrap();
    track_var!(c_string_2);

    let c_string_3 = CString::new("Long C string: ".to_string() + &"x".repeat(5000)).unwrap();
    track_var!(c_string_3);

    // String collections
    let string_collection: Vec<String> = (0..500)
        .map(|i| format!("string_item_{}_with_data_{}", i, "x".repeat(i % 100)))
        .collect();
    track_var!(string_collection);

    // Add real unsafe/FFI operations to generate cross-boundary events
    let unsafe_ffi_tracker = get_global_unsafe_ffi_tracker();
    
    unsafe {
        use std::alloc::{alloc, dealloc, Layout};
        
        // Unsafe Rust allocation
        let layout = Layout::new::<[u8; 1024]>();
        let ptr = alloc(layout);
        if !ptr.is_null() {
            // Initialize some data
            std::ptr::write_bytes(ptr, 0x42, 1024);
            
            // First track the unsafe allocation
            let _ = unsafe_ffi_tracker.track_unsafe_allocation(
                ptr as usize,
                1024,
                "examples/enhanced_simple_showcase.rs:185:13".to_string(),
            );
            
            // Then record boundary event for unsafe allocation
            let _ = unsafe_ffi_tracker.record_boundary_event(
                ptr as usize,
                BoundaryEventType::RustToFfi,
                "unsafe_rust_block".to_string(),
                "potential_ffi_target".to_string(),
            );
            
            // Simulate cross-boundary event by using libc
            extern "C" {
                fn malloc(size: usize) -> *mut std::ffi::c_void;
                fn free(ptr: *mut std::ffi::c_void);
            }
            
            // FFI allocation
            let ffi_ptr = malloc(512);
            if !ffi_ptr.is_null() {
                // Write some data through FFI
                std::ptr::write_bytes(ffi_ptr as *mut u8, 0x55, 512);
                
                // Track the FFI allocation
                let _ = unsafe_ffi_tracker.track_ffi_allocation(
                    ffi_ptr as usize,
                    512,
                    "libc".to_string(),
                    "malloc".to_string(),
                );
                
                // Record boundary event for FFI allocation
                let _ = unsafe_ffi_tracker.record_boundary_event(
                    ffi_ptr as usize,
                    BoundaryEventType::FfiToRust,
                    "libc".to_string(),
                    "rust_main".to_string(),
                );
                
                free(ffi_ptr);
            }
            
            dealloc(ptr, layout);
        }
    }

    // Debug: Check if UnsafeFFITracker has any data
    let debug_tracker = get_global_unsafe_ffi_tracker();
    if let Ok(enhanced_allocations) = debug_tracker.get_enhanced_allocations() {
        println!("  📊 UnsafeFFITracker has {} enhanced allocations", enhanced_allocations.len());
        for (i, alloc) in enhanced_allocations.iter().enumerate() {
            println!("    {}: ptr=0x{:x}, events={}, source={:?}", 
                i, alloc.base.ptr, alloc.cross_boundary_events.len(), alloc.source);
        }
    } else {
        println!("  ❌ Failed to get enhanced allocations from UnsafeFFITracker");
    }

    println!("  ✅ Created complex strings and real FFI operations with cross-boundary events");
}

fn create_smart_pointers() {
    // Boxed large data
    let boxed_large_vec = Box::new((0..10000u64).collect::<Vec<_>>());
    track_var!(boxed_large_vec);

    let boxed_string_map = Box::new(
        (0..200)
            .map(|i| (format!("boxed_key_{}", i), format!("boxed_value_{}", i)))
            .collect::<HashMap<String, String>>()
    );
    track_var!(boxed_string_map);

    // Rc for shared ownership
    let shared_data = Rc::new((0..1000).map(|i| format!("shared_item_{}", i)).collect::<Vec<_>>());
    let shared_clone_1 = Rc::clone(&shared_data);
    let shared_clone_2 = Rc::clone(&shared_data);
    track_var!(shared_data);
    track_var!(shared_clone_1);
    track_var!(shared_clone_2);

    // Arc for thread-safe sharing
    let thread_safe_data = Arc::new(
        (0..500)
            .map(|i| (i, format!("thread_safe_value_{}", i)))
            .collect::<HashMap<usize, String>>()
    );
    let thread_safe_clone_1 = Arc::clone(&thread_safe_data);
    let thread_safe_clone_2 = Arc::clone(&thread_safe_data);
    track_var!(thread_safe_data);
    track_var!(thread_safe_clone_1);
    track_var!(thread_safe_clone_2);

    println!("  ✅ Created smart pointers with shared ownership");
}

fn create_large_data_structures() {
    // Very large vector
    let huge_vector: Vec<u64> = (0..50000).collect();
    track_var!(huge_vector);

    // Large byte buffer
    let byte_buffer: Vec<u8> = (0..100000).map(|i| (i % 256) as u8).collect();
    track_var!(byte_buffer);

    // Large string vector
    let string_database: Vec<String> = (0..5000)
        .map(|i| format!("database_record_{}_with_content_{}", i, "data".repeat(i % 50)))
        .collect();
    track_var!(string_database);

    // Large HashMap
    let massive_lookup: HashMap<u64, String> = (0..10000)
        .map(|i| (i as u64, format!("lookup_value_for_key_{}_content_{}", i, "x".repeat(i % 20))))
        .collect();
    track_var!(massive_lookup);

    // Complex tuple vector
    let tuple_data: Vec<(String, u64, Vec<u8>)> = (0..2000)
        .map(|i| (
            format!("tuple_string_{}", i),
            i as u64,
            (0..(i % 100)).map(|j| (j % 256) as u8).collect()
        ))
        .collect();
    track_var!(tuple_data);

    println!("  ✅ Created large data structures with hundreds of thousands of elements");
}

fn create_nested_structures() {
    // Deeply nested HashMap
    let deep_nested: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>> = (0..20)
        .map(|i| {
            let level2: HashMap<String, HashMap<String, Vec<String>>> = (0..10)
                .map(|j| {
                    let level3: HashMap<String, Vec<String>> = (0..5)
                        .map(|k| {
                            let level4: Vec<String> = (0..10)
                                .map(|l| format!("deep_value_{}_{}_{}_{}",i, j, k, l))
                                .collect();
                            (format!("level3_key_{}_{}", j, k), level4)
                        })
                        .collect();
                    (format!("level2_key_{}", j), level3)
                })
                .collect();
            (format!("level1_key_{}", i), level2)
        })
        .collect();
    track_var!(deep_nested);

    // Mixed type collections
    let mixed_data: Vec<HashMap<String, Vec<(String, u64)>>> = (0..100)
        .map(|i| {
            let inner_map: HashMap<String, Vec<(String, u64)>> = (0..10)
                .map(|j| {
                    let tuple_vec: Vec<(String, u64)> = (0..5)
                        .map(|k| (format!("tuple_str_{}_{}", j, k), (j * 10 + k) as u64))
                        .collect();
                    (format!("map_key_{}_{}", i, j), tuple_vec)
                })
                .collect();
            inner_map
        })
        .collect();
    track_var!(mixed_data);

    // Configuration-like nested structure
    let app_config: HashMap<String, HashMap<String, Vec<String>>> = [
        ("database", vec![
            ("hosts", vec!["db1.example.com", "db2.example.com", "db3.example.com"]),
            ("ports", vec!["5432", "5433", "5434"]),
            ("credentials", vec!["user1", "user2", "user3"]),
        ]),
        ("cache", vec![
            ("redis_hosts", vec!["redis1.example.com", "redis2.example.com"]),
            ("memcached_hosts", vec!["mc1.example.com", "mc2.example.com"]),
            ("ttl_settings", vec!["3600", "7200", "1800"]),
        ]),
        ("logging", vec![
            ("levels", vec!["DEBUG", "INFO", "WARN", "ERROR"]),
            ("outputs", vec!["console", "file", "syslog"]),
            ("formats", vec!["json", "text", "structured"]),
        ]),
    ].into_iter()
        .map(|(section, configs)| {
            let section_map: HashMap<String, Vec<String>> = configs.into_iter()
                .map(|(key, values)| (key.to_string(), values.into_iter().map(|s| s.to_string()).collect()))
                .collect();
            (section.to_string(), section_map)
        })
        .collect();
    track_var!(app_config);

    println!("  ✅ Created deeply nested structures with complex relationships");
}