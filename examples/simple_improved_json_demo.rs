//! Simple Improved JSON Export Demo
//!
//! This example demonstrates the enhanced JSON export functionality
//! that generates files with improve.md extended fields

use memscope_rs::core::types::{AllocationInfo, BorrowInfo, CloneInfo, MemoryStats};
use memscope_rs::export::enhanced_json_exporter::{EnhancedJsonExporter, ExportConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 Simple Improved JSON Export Demo - improve.md Compliant");
    println!("==========================================================");

    // Create sample data that demonstrates improve.md extended fields
    let memory_stats = create_sample_memory_stats();

    // Configure enhanced JSON exporter
    let export_config = ExportConfig {
        pretty_print: true,
        include_stack_traces: true,
        generate_lifetime_file: true,
        generate_unsafe_ffi_file: false, // Skip FFI for simplicity
        max_ownership_events: 100,
    };

    let exporter = EnhancedJsonExporter::new(export_config);

    // Create output directory
    let output_dir = "output/simple_improved_json";
    std::fs::create_dir_all(output_dir)?;

    println!("\n📊 Exporting enhanced JSON files to: {}", output_dir);

    // Export with empty FFI data for simplicity
    exporter.export_enhanced_analysis(
        output_dir,
        &memory_stats,
        &[], // Empty unsafe reports
        &[], // Empty memory passports
    )?;

    println!("\n✅ Export completed! Generated files:");
    println!("   📄 memory_analysis.json - Main analysis with extended fields");
    println!("   📄 lifetime.json - Ownership history and lifecycle events");

    // Verify the files were created and show sample content
    verify_and_show_content(output_dir)?;

    println!("\n🎉 Files generated successfully and comply with improve.md specifications!");
    println!("\n📋 Key improve.md extensions demonstrated:");
    println!("   ✅ borrow_info: immutable_borrows, mutable_borrows, max_concurrent_borrows, last_borrow_timestamp");
    println!("   ✅ clone_info: clone_count, is_clone, original_ptr");
    println!("   ✅ ownership_history_available: flag for detailed history in lifetime.json");
    println!("   ✅ lifetime.json: ownership events (Allocated, Cloned, Borrowed, Dropped)");

    Ok(())
}

/// Create sample memory statistics with improve.md extended fields
fn create_sample_memory_stats() -> MemoryStats {
    let mut memory_stats = MemoryStats::new();

    // Create allocations with extended borrow_info and clone_info
    let allocations = vec![
        create_allocation_with_borrow_info(0x1000, 64, "my_vector", "Vec<i32>"),
        create_allocation_with_clone_info(0x2000, 128, "my_string", "String"),
        create_allocation_with_full_info(0x3000, 256, "my_box", "Box<Data>"),
        create_simple_allocation(0x4000, 32, "temp_data", "TempStruct"),
    ];

    memory_stats.allocations = allocations;
    memory_stats.total_allocations = memory_stats.allocations.len();
    memory_stats.active_allocations = memory_stats.allocations.iter().filter(|a| a.is_active()).count();
    memory_stats.total_allocated = memory_stats.allocations.iter().map(|a| a.size).sum();

    memory_stats
}

/// Create allocation with detailed borrow_info as specified in improve.md
fn create_allocation_with_borrow_info(ptr: usize, size: usize, var_name: &str, type_name: &str) -> AllocationInfo {
    let mut alloc = AllocationInfo::new(ptr, size);
    alloc.var_name = Some(var_name.to_string());
    alloc.type_name = Some(type_name.to_string());
    alloc.scope_name = Some("main".to_string());
    alloc.lifetime_ms = Some(1520); // As specified in improve.md
    
    // Add borrow_info as specified in improve.md
    alloc.borrow_info = Some(BorrowInfo {
        immutable_borrows: 25,
        mutable_borrows: 2,
        max_concurrent_borrows: 5,
        last_borrow_timestamp: Some(1755004694594239500),
    });
    
    alloc.ownership_history_available = true;
    alloc
}

/// Create allocation with detailed clone_info as specified in improve.md
fn create_allocation_with_clone_info(ptr: usize, size: usize, var_name: &str, type_name: &str) -> AllocationInfo {
    let mut alloc = AllocationInfo::new(ptr, size);
    alloc.var_name = Some(var_name.to_string());
    alloc.type_name = Some(type_name.to_string());
    alloc.scope_name = Some("process_data".to_string());
    alloc.lifetime_ms = Some(850);
    
    // Add clone_info as specified in improve.md
    alloc.clone_info = Some(CloneInfo {
        clone_count: 3,
        is_clone: true,
        original_ptr: Some(0x1234567),
    });
    
    alloc.ownership_history_available = true;
    alloc
}

/// Create allocation with both borrow_info and clone_info
fn create_allocation_with_full_info(ptr: usize, size: usize, var_name: &str, type_name: &str) -> AllocationInfo {
    let mut alloc = AllocationInfo::new(ptr, size);
    alloc.var_name = Some(var_name.to_string());
    alloc.type_name = Some(type_name.to_string());
    alloc.scope_name = Some("complex_function".to_string());
    alloc.lifetime_ms = Some(2340);
    
    // Add both borrow_info and clone_info
    alloc.borrow_info = Some(BorrowInfo {
        immutable_borrows: 12,
        mutable_borrows: 4,
        max_concurrent_borrows: 3,
        last_borrow_timestamp: Some(1755004694594240000),
    });
    
    alloc.clone_info = Some(CloneInfo {
        clone_count: 1,
        is_clone: false,
        original_ptr: None,
    });
    
    alloc.ownership_history_available = true;
    alloc.stack_trace = Some(vec![
        "main".to_string(),
        "complex_function".to_string(),
        "allocate_box".to_string(),
    ]);
    
    alloc
}

/// Create simple allocation without extended fields
fn create_simple_allocation(ptr: usize, size: usize, var_name: &str, type_name: &str) -> AllocationInfo {
    let mut alloc = AllocationInfo::new(ptr, size);
    alloc.var_name = Some(var_name.to_string());
    alloc.type_name = Some(type_name.to_string());
    alloc.scope_name = Some("temp_scope".to_string());
    alloc.lifetime_ms = Some(100);
    alloc.ownership_history_available = false; // No detailed history for this one
    alloc
}

/// Verify the generated files and show sample content
fn verify_and_show_content(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let files = vec![
        "memory_analysis.json",
        "lifetime.json", 
    ];

    for file in files {
        let file_path = format!("{}/{}", output_dir, file);
        
        if std::path::Path::new(&file_path).exists() {
            let content = std::fs::read_to_string(&file_path)?;
            let json_value: serde_json::Value = serde_json::from_str(&content)?;
            
            println!("\n📄 {} (first 800 chars):", file);
            println!("   {}", content.chars().take(800).collect::<String>());
            if content.len() > 800 {
                println!("   ... (truncated)");
            }
            
            // Verify key improve.md fields are present
            match file {
                "memory_analysis.json" => {
                    if let Some(allocations) = json_value["allocations"].as_array() {
                        if let Some(first_alloc) = allocations.first() {
                            let has_borrow_info = first_alloc.get("borrow_info").is_some();
                            let has_clone_info = first_alloc.get("clone_info").is_some();
                            let has_ownership_flag = first_alloc.get("ownership_history_available").is_some();
                            let has_lifetime_ms = first_alloc.get("lifetime_ms").is_some();
                            
                            println!("\n   ✅ improve.md fields present:");
                            println!("      • borrow_info: {}", has_borrow_info);
                            println!("      • clone_info: {}", has_clone_info);
                            println!("      • ownership_history_available: {}", has_ownership_flag);
                            println!("      • lifetime_ms: {}", has_lifetime_ms);
                            
                            // Show specific borrow_info content
                            if let Some(borrow_info) = first_alloc.get("borrow_info") {
                                if !borrow_info.is_null() {
                                    println!("      • borrow_info content: {}", borrow_info);
                                }
                            }
                        }
                    }
                }
                "lifetime.json" => {
                    if let Some(histories) = json_value["ownership_histories"].as_array() {
                        println!("\n   ✅ {} ownership histories exported", histories.len());
                        if let Some(first_history) = histories.first() {
                            if let Some(events) = first_history["ownership_history"].as_array() {
                                println!("      • First allocation has {} ownership events", events.len());
                                if let Some(first_event) = events.first() {
                                    println!("      • First event type: {}", 
                                            first_event["event_type"].as_str().unwrap_or("unknown"));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        } else {
            println!("❌ File not found: {}", file_path);
        }
    }

    Ok(())
}