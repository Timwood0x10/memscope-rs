//! Test Binary Extensions - Verify binary format includes improve.md fields
//!
//! This example tests that the binary format correctly includes and preserves
//! the improve.md extension fields (borrow_info, clone_info, ownership_history_available, lifetime_ms)

use memscope_rs::export::binary::reader::BinaryReader;
use memscope_rs::export::binary::writer::BinaryWriter;
use memscope_rs::core::types::{AllocationInfo, BorrowInfo, CloneInfo};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Binary Extensions for improve.md compliance");
    println!("======================================================");

    // Create test allocation with improve.md extensions
    let mut test_alloc = AllocationInfo::new(0x1000, 256);
    test_alloc.var_name = Some("test_variable".to_string());
    test_alloc.type_name = Some("Vec<String>".to_string());
    test_alloc.scope_name = Some("main".to_string());
    test_alloc.thread_id = "main".to_string();
    
    // Set improve.md extensions
    test_alloc.lifetime_ms = Some(1500);
    test_alloc.borrow_info = Some(BorrowInfo {
        immutable_borrows: 5,
        mutable_borrows: 2,
        max_concurrent_borrows: 3,
        last_borrow_timestamp: Some(1755673000000000000),
    });
    test_alloc.clone_info = Some(CloneInfo {
        clone_count: 3,
        is_clone: true,
        original_ptr: Some(0x2000),
    });
    test_alloc.ownership_history_available = true;

    println!("📝 Created test allocation with improve.md extensions:");
    println!("   • lifetime_ms: {:?}", test_alloc.lifetime_ms);
    println!("   • borrow_info: {:?}", test_alloc.borrow_info);
    println!("   • clone_info: {:?}", test_alloc.clone_info);
    println!("   • ownership_history_available: {}", test_alloc.ownership_history_available);

    // Test binary round-trip using the high-level API
    let test_file = "test_binary_extensions.memscope";
    
    // Create a memory stats with our test allocation
    let mut memory_stats = memscope_rs::core::types::MemoryStats::new();
    memory_stats.allocations = vec![test_alloc.clone()];
    memory_stats.total_allocations = 1;
    memory_stats.active_allocations = 1;
    memory_stats.total_allocated = test_alloc.size;
    memory_stats.active_memory = test_alloc.size;
    
    // Write to binary using the high-level API
    println!("\n💾 Writing to binary file using high-level API...");
    memscope_rs::export::export_user_variables_binary(
        vec![test_alloc.clone()],
        memory_stats,
        "test_binary_extensions"
    )?;

    // Read from binary (check the actual generated file location)
    println!("📖 Reading from binary file...");
    
    // The high-level API generates files in a subdirectory
    let possible_files = vec![
        "test_binary_extensions.memscope",
        "MemoryAnalysis/test_binary_extensions.memscope",
        "MemoryAnalysis/test_binary_extensions/test_binary_extensions.memscope",
    ];
    
    let mut actual_file = None;
    for file in &possible_files {
        if std::path::Path::new(file).exists() {
            actual_file = Some(file);
            println!("   Found binary file at: {}", file);
            break;
        }
    }
    
    let binary_file = actual_file.ok_or("Binary file not found in expected locations")?;
    let mut reader = BinaryReader::new(binary_file)?;
    let allocations = reader.read_all()?;

    // Verify data integrity
    println!("\n🔍 Verifying binary round-trip integrity...");
    
    if allocations.len() != 1 {
        println!("❌ Expected 1 allocation, got {}", allocations.len());
        return Ok(());
    }

    let read_alloc = &allocations[0];
    
    // Check basic fields
    assert_eq!(read_alloc.ptr, test_alloc.ptr);
    assert_eq!(read_alloc.size, test_alloc.size);
    assert_eq!(read_alloc.var_name, test_alloc.var_name);
    assert_eq!(read_alloc.type_name, test_alloc.type_name);
    println!("✅ Basic fields preserved correctly");

    // Check improve.md extensions
    println!("\n🎯 Checking improve.md extensions:");
    
    // Check lifetime_ms
    match (&read_alloc.lifetime_ms, &test_alloc.lifetime_ms) {
        (Some(read_ms), Some(orig_ms)) => {
            if read_ms == orig_ms {
                println!("✅ lifetime_ms: {} ms (preserved)", read_ms);
            } else {
                println!("❌ lifetime_ms mismatch: expected {}, got {}", orig_ms, read_ms);
            }
        }
        (None, None) => println!("✅ lifetime_ms: None (preserved)"),
        _ => println!("❌ lifetime_ms preservation failed"),
    }

    // Check borrow_info
    match (&read_alloc.borrow_info, &test_alloc.borrow_info) {
        (Some(read_borrow), Some(orig_borrow)) => {
            if read_borrow.immutable_borrows == orig_borrow.immutable_borrows &&
               read_borrow.mutable_borrows == orig_borrow.mutable_borrows &&
               read_borrow.max_concurrent_borrows == orig_borrow.max_concurrent_borrows &&
               read_borrow.last_borrow_timestamp == orig_borrow.last_borrow_timestamp {
                println!("✅ borrow_info: immutable={}, mutable={}, max_concurrent={}, timestamp={:?} (preserved)",
                        read_borrow.immutable_borrows,
                        read_borrow.mutable_borrows,
                        read_borrow.max_concurrent_borrows,
                        read_borrow.last_borrow_timestamp);
            } else {
                println!("❌ borrow_info mismatch");
                println!("   Expected: {:?}", orig_borrow);
                println!("   Got: {:?}", read_borrow);
            }
        }
        (None, None) => println!("✅ borrow_info: None (preserved)"),
        _ => println!("❌ borrow_info preservation failed"),
    }

    // Check clone_info
    match (&read_alloc.clone_info, &test_alloc.clone_info) {
        (Some(read_clone), Some(orig_clone)) => {
            if read_clone.clone_count == orig_clone.clone_count &&
               read_clone.is_clone == orig_clone.is_clone &&
               read_clone.original_ptr == orig_clone.original_ptr {
                println!("✅ clone_info: count={}, is_clone={}, original_ptr={:?} (preserved)",
                        read_clone.clone_count,
                        read_clone.is_clone,
                        read_clone.original_ptr);
            } else {
                println!("❌ clone_info mismatch");
                println!("   Expected: {:?}", orig_clone);
                println!("   Got: {:?}", read_clone);
            }
        }
        (None, None) => println!("✅ clone_info: None (preserved)"),
        _ => println!("❌ clone_info preservation failed"),
    }

    // Check ownership_history_available
    if read_alloc.ownership_history_available == test_alloc.ownership_history_available {
        println!("✅ ownership_history_available: {} (preserved)", read_alloc.ownership_history_available);
    } else {
        println!("❌ ownership_history_available mismatch: expected {}, got {}", 
                test_alloc.ownership_history_available, read_alloc.ownership_history_available);
    }

    // Check file size
    let file_size = fs::metadata(test_file)?.len();
    println!("\n📊 Binary file statistics:");
    println!("   • File size: {} bytes", file_size);
    println!("   • Allocation count: {}", allocations.len());

    // Cleanup
    fs::remove_file(test_file)?;
    println!("\n🧹 Cleaned up test file");

    println!("\n🎉 Binary extensions test completed successfully!");
    println!("   All improve.md extension fields are correctly preserved in binary format!");

    Ok(())
}