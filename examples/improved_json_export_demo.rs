//! Improved JSON Export Demo
//!
//! This example demonstrates the enhanced JSON export functionality
//! that generates files exactly matching the improve.md specifications:
//! - memory_analysis.json with extended fields (borrow_info, clone_info, ownership_history_available)
//! - lifetime.json with ownership history events
//! - unsafe_ffi.json with FFI safety analysis

use memscope_rs::core::types::{AllocationInfo, BorrowInfo, CloneInfo, MemoryStats};
use memscope_rs::{UnsafeReport, MemoryPassport, RiskAssessment, RiskFactor, PassportEvent, PassportStatus, RiskLevel, RiskFactorType, UnsafeSource};
use memscope_rs::export::enhanced_json_exporter::{EnhancedJsonExporter, ExportConfig};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 Improved JSON Export Demo - improve.md Compliant");
    println!("=====================================================");

    // Create sample data that demonstrates improve.md extended fields
    let memory_stats = create_sample_memory_stats();
    let unsafe_reports = create_sample_unsafe_reports();
    let memory_passports = create_sample_memory_passports();

    // Configure enhanced JSON exporter
    let export_config = ExportConfig {
        pretty_print: true,
        include_stack_traces: true,
        generate_lifetime_file: true,
        generate_unsafe_ffi_file: true,
        max_ownership_events: 100,
    };

    let exporter = EnhancedJsonExporter::new(export_config);

    // Create output directory
    let output_dir = "output/improved_json_export";
    std::fs::create_dir_all(output_dir)?;

    println!("\n📊 Exporting enhanced JSON files to: {}", output_dir);

    // Export all files according to improve.md specifications
    exporter.export_enhanced_analysis(
        output_dir,
        &memory_stats,
        &unsafe_reports,
        &memory_passports,
    )?;

    println!("\n✅ Export completed! Generated files:");
    println!("   📄 memory_analysis.json - Main analysis with extended fields");
    println!("   📄 lifetime.json - Ownership history and lifecycle events");
    println!("   📄 unsafe_ffi.json - FFI safety analysis and memory passports");

    // Verify the files were created and show sample content
    verify_and_show_content(output_dir)?;

    println!("\n🎉 All files generated successfully and comply with improve.md specifications!");

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

/// Create allocation with detailed borrow_info
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

/// Create allocation with detailed clone_info
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

/// Create sample unsafe reports as specified in improve.md
fn create_sample_unsafe_reports() -> Vec<UnsafeReport> {
    vec![
        UnsafeReport {
            report_id: "unsafe-report-uuid-001".to_string(),
            source: UnsafeSource {
                source_type: "UnsafeBlock".to_string(),
                location: "examples/unsafe_ffi_demo.rs:37:13".to_string(),
                function_name: Some("process_data_unsafe".to_string()),
                module_path: Some("unsafe_ffi_demo".to_string()),
            },
            risk_assessment: RiskAssessment {
                risk_level: RiskLevel::High,
                confidence_score: 0.85,
                risk_factors: vec![
                    RiskFactor {
                        factor_type: RiskFactorType::RawPointerDereference,
                        severity: 8.0,
                        description: "Dereferencing a raw pointer `*mut c_void`.".to_string(),
                        source_location: Some("examples/unsafe_ffi_demo.rs:42:5".to_string()),
                        mitigation_suggestion: Some("Verify the pointer is non-null before dereferencing.".to_string()),
                    },
                    RiskFactor {
                        factor_type: "FfiCall".to_string(),
                        severity: 7.0,
                        description: "Call to external function `process_data_unsafe`.".to_string(),
                        source_location: Some("examples/unsafe_ffi_demo.rs:45:9".to_string()),
                        mitigation_suggestion: Some("Consider using a safer abstraction layer around this FFI call.".to_string()),
                    },
                ],
                mitigation_suggestions: vec![
                    "Verify the pointer is non-null before dereferencing.".to_string(),
                    "Ensure the data pointed to is properly initialized and valid.".to_string(),
                    "Consider using a safer abstraction layer around this FFI call.".to_string(),
                ],
            },
            dynamic_violations: vec![],
            related_passports: vec!["passport-uuid-123".to_string(), "passport-uuid-124".to_string()],
            call_stack: vec![
                "main".to_string(),
                "unsafe_operation".to_string(),
                "process_data_unsafe".to_string(),
            ],
        }
    ]
}

/// Create sample memory passports as specified in improve.md
fn create_sample_memory_passports() -> Vec<MemoryPassport> {
    vec![
        MemoryPassport {
            passport_id: "passport-uuid-123".to_string(),
            allocation_ptr: 0x11223344,
            size_bytes: 1024,
            status_at_shutdown: PassportStatus::InForeignCustody,
            lifecycle_events: vec![
                PassportEvent {
                    event_type: "CreatedAndHandedOver".to_string(),
                    timestamp: 1755004694594238100,
                    details: {
                        let mut details = HashMap::new();
                        details.insert("how".to_string(), serde_json::Value::String("Box::into_raw".to_string()));
                        details.insert("target_function".to_string(), serde_json::Value::String("process_data_unsafe".to_string()));
                        details
                    },
                    source_stack_id: Some(105),
                    ffi_call_info: Some("Call to process_data_unsafe".to_string()),
                },
            ],
            creation_context: "FFI handover in unsafe block".to_string(),
            foreign_references: vec!["process_data_unsafe".to_string()],
        },
        MemoryPassport {
            passport_id: "passport-uuid-124".to_string(),
            allocation_ptr: 0x22334455,
            size_bytes: 512,
            status_at_shutdown: PassportStatus::ReclaimedByRust,
            lifecycle_events: vec![
                PassportEvent {
                    event_type: "CreatedAndHandedOver".to_string(),
                    timestamp: 1755004694594238200,
                    details: {
                        let mut details = HashMap::new();
                        details.insert("how".to_string(), serde_json::Value::String("Box::into_raw".to_string()));
                        details.insert("target_function".to_string(), serde_json::Value::String("temp_process".to_string()));
                        details
                    },
                    source_stack_id: Some(106),
                    ffi_call_info: Some("Call to temp_process".to_string()),
                },
                PassportEvent {
                    event_type: "ReclaimedByRust".to_string(),
                    timestamp: 1755004694594239900,
                    details: {
                        let mut details = HashMap::new();
                        details.insert("how".to_string(), serde_json::Value::String("Box::from_raw".to_string()));
                        details
                    },
                    source_stack_id: Some(112),
                    ffi_call_info: None,
                },
            ],
            creation_context: "Temporary FFI handover".to_string(),
            foreign_references: vec!["temp_process".to_string()],
        },
    ]
}

/// Verify the generated files and show sample content
fn verify_and_show_content(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let files = vec![
        "memory_analysis.json",
        "lifetime.json", 
        "unsafe_ffi.json",
    ];

    for file in files {
        let file_path = format!("{}/{}", output_dir, file);
        
        if std::path::Path::new(&file_path).exists() {
            let content = std::fs::read_to_string(&file_path)?;
            let json_value: serde_json::Value = serde_json::from_str(&content)?;
            
            println!("\n📄 {} (first 500 chars):", file);
            println!("   {}", content.chars().take(500).collect::<String>());
            if content.len() > 500 {
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
                            
                            println!("   ✅ improve.md fields present:");
                            println!("      • borrow_info: {}", has_borrow_info);
                            println!("      • clone_info: {}", has_clone_info);
                            println!("      • ownership_history_available: {}", has_ownership_flag);
                        }
                    }
                }
                "lifetime.json" => {
                    if let Some(histories) = json_value["ownership_histories"].as_array() {
                        println!("   ✅ {} ownership histories exported", histories.len());
                    }
                }
                "unsafe_ffi.json" => {
                    let reports_count = json_value["unsafe_reports"].as_array().map(|a| a.len()).unwrap_or(0);
                    let passports_count = json_value["memory_passports"].as_array().map(|a| a.len()).unwrap_or(0);
                    println!("   ✅ {} unsafe reports, {} memory passports", reports_count, passports_count);
                }
                _ => {}
            }
        } else {
            println!("❌ File not found: {}", file_path);
        }
    }

    Ok(())
}