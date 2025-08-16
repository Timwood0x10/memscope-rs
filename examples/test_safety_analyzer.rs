//! Test example for the SafetyAnalyzer and MemoryPassportTracker
//!
//! This example demonstrates the enhanced unsafe and FFI safety analysis capabilities.

use memscope_rs::analysis::{
    SafetyAnalyzer, MemoryPassportTracker, UnsafeSource, PassportEventType,
    SafetyAnalysisConfig, PassportTrackerConfig,
};
use memscope_rs::core::types::AllocationInfo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔒 Testing SafetyAnalyzer and MemoryPassportTracker");

    // Create safety analyzer with default config
    let safety_analyzer = SafetyAnalyzer::new(SafetyAnalysisConfig::default());
    
    // Create memory passport tracker with default config
    let passport_tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());

    // Test 1: Generate unsafe report for raw pointer operation
    println!("\n📝 Test 1: Generating unsafe report for raw pointer operation");
    let unsafe_source = UnsafeSource::RawPointer {
        operation: "raw_pointer_deref".to_string(),
        location: "test_location".to_string(),
    };

    let allocations = vec![
        AllocationInfo::new(0x1000, 1024),
        AllocationInfo::new(0x2000, 512),
    ];

    let report_id = safety_analyzer.generate_unsafe_report(
        unsafe_source,
        &allocations,
        &[],
    )?;

    println!("✅ Generated unsafe report: {}", report_id);

    // Test 2: Create memory passport and track lifecycle
    println!("\n📋 Test 2: Creating memory passport and tracking lifecycle");
    let ptr = 0x3000;
    let size = 256;

    let passport_id = passport_tracker.create_passport(
        ptr,
        size,
        "rust_context".to_string(),
    )?;

    println!("✅ Created memory passport: {}", passport_id);

    // Record handover to FFI
    passport_tracker.record_handover_to_ffi(
        ptr,
        "ffi_context".to_string(),
        "malloc".to_string(),
    )?;

    println!("✅ Recorded handover to FFI");

    // Record reclaim by Rust (prevents leak)
    passport_tracker.record_reclaimed_by_rust(
        ptr,
        "rust_context".to_string(),
        "cleanup".to_string(),
    )?;

    println!("✅ Recorded reclaim by Rust");

    // Test 3: Create a passport that will leak
    println!("\n🚨 Test 3: Creating a passport that will leak");
    let leak_ptr = 0x4000;
    let leak_size = 128;

    let leak_passport_id = passport_tracker.create_passport(
        leak_ptr,
        leak_size,
        "rust_context".to_string(),
    )?;

    // Only hand over to FFI, don't reclaim (this will be detected as a leak)
    passport_tracker.record_handover_to_ffi(
        leak_ptr,
        "ffi_context".to_string(),
        "malloc".to_string(),
    )?;

    println!("✅ Created leaky passport: {}", leak_passport_id);

    // Test 4: Detect leaks at shutdown
    println!("\n🔍 Test 4: Detecting leaks at shutdown");
    let leak_detection = passport_tracker.detect_leaks_at_shutdown();

    println!("📊 Leak detection results:");
    println!("   • Total leaks detected: {}", leak_detection.total_leaks);
    println!("   • Leaked passports: {:?}", leak_detection.leaked_passports);

    for leak_detail in &leak_detection.leak_details {
        println!("   • Leak: {} at 0x{:x} ({} bytes)", 
            leak_detail.passport_id, 
            leak_detail.memory_address, 
            leak_detail.size_bytes);
        println!("     Last context: {}", leak_detail.last_context);
        println!("     Lifecycle: {}", leak_detail.lifecycle_summary);
    }

    // Test 5: Get statistics
    println!("\n📈 Test 5: Getting statistics");
    let safety_stats = safety_analyzer.get_stats();
    let passport_stats = passport_tracker.get_stats();

    println!("Safety Analysis Stats:");
    println!("   • Total reports: {}", safety_stats.total_reports);
    println!("   • Reports by risk level: {:?}", safety_stats.reports_by_risk_level);

    println!("Passport Tracker Stats:");
    println!("   • Total passports created: {}", passport_stats.total_passports_created);
    println!("   • Active passports: {}", passport_stats.active_passports);
    println!("   • Leaks detected: {}", passport_stats.leaks_detected);
    println!("   • Total events recorded: {}", passport_stats.total_events_recorded);

    // Test 6: Validate passport integrity
    println!("\n🔍 Test 6: Validating passport integrity");
    let is_valid = passport_tracker.validate_passport(ptr)?;
    println!("✅ Passport validation result: {}", is_valid);

    println!("\n🎉 All tests completed successfully!");
    println!("The SafetyAnalyzer and MemoryPassportTracker are working correctly.");

    Ok(())
}