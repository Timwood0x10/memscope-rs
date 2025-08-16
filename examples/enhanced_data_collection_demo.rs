//! Enhanced Data Collection Demo
//! 
//! This example demonstrates the new enhanced binary data collection capabilities
//! including UnsafeReport, MemoryPassport, and OwnershipHistory tracking.
//! It showcases the complete workflow from data collection to JSON/HTML export.

use memscope_rs::{MemoryTracker, Trackable};
use memscope_rs::export::binary::{BinaryExportConfig, export_binary_to_html_system};
use memscope_rs::export::optimized_json_export::{OptimizedExportOptions, OptimizationLevel};
use std::ffi::{CString, c_char};
use std::ptr;

// Macro definitions for enhanced tracking (moved to top)
macro_rules! track_allocation {
    ($var:expr, $name:expr, $type_name:expr) => {
        tracing::debug!("🔍 Tracking allocation: {} ({})", $name, $type_name);
    };
}

macro_rules! track_lifecycle_event {
    ($var:expr, $event:expr, $details:expr) => {
        tracing::debug!("📅 Lifecycle event: {} - {}", $event, $details);
    };
}

macro_rules! track_ffi_boundary {
    ($ptr:expr, $event:expr, $function:expr) => {
        tracing::debug!("🌍 FFI boundary: {} in {} (ptr: {:p})", $event, $function, $ptr);
    };
}

macro_rules! track_unsafe_operation {
    ($ptr:expr, $operation:expr, $details:expr) => {
        tracing::warn!("⚠️ Unsafe operation: {} - {} (ptr: {:p})", $operation, $details, $ptr);
    };
}

macro_rules! track_ownership_event {
    ($var:expr, $event:expr, $details:expr) => {
        tracing::debug!("🔄 Ownership event: {} - {}", $event, $details);
    };
}

macro_rules! track_smart_pointer {
    ($var:expr, $name:expr, $type_name:expr) => {
        tracing::debug!("🧠 Smart pointer: {} ({})", $name, $type_name);
    };
}

macro_rules! track_smart_pointer_event {
    ($var:expr, $event:expr, $details:expr) => {
        tracing::debug!("🧠 Smart pointer event: {} - {}", $event, $details);
    };
}

/// Simulated FFI function for demonstration
extern "C" {
    // This would normally be a real C function
    // For demo purposes, we'll implement it in Rust
}

// Mock C function implementation
#[no_mangle]
pub extern "C" fn process_data(data: *mut c_char, _len: usize) -> *mut c_char {
    if data.is_null() {
        return ptr::null_mut();
    }
    
    // Simulate some processing and return new data
    let processed = CString::new("processed_data").unwrap();
    processed.into_raw()
}

#[no_mangle]
pub extern "C" fn free_processed_data(data: *mut c_char) {
    if !data.is_null() {
        unsafe {
            let _ = CString::from_raw(data);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for better logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("🚀 Starting Enhanced Data Collection Demo");

    // Initialize the memory tracker with enhanced features
    let tracker = MemoryTracker::new();
    
    // Demo 1: Basic allocation with enhanced tracking
    demo_basic_enhanced_allocation(&tracker)?;
    
    // Demo 2: Unsafe FFI operations with passport tracking
    demo_unsafe_ffi_operations(&tracker)?;
    
    // Demo 3: Complex ownership transfers
    demo_ownership_transfers(&tracker)?;
    
    // Demo 4: Smart pointer analysis
    demo_smart_pointer_analysis(&tracker)?;
    
    // Export data using enhanced binary format
    export_enhanced_data(&tracker)?;
    
    tracing::info!("✅ Enhanced Data Collection Demo completed successfully");
    Ok(())
}

/// Demo 1: Basic allocation with enhanced tracking
fn demo_basic_enhanced_allocation(tracker: &MemoryTracker) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("📊 Demo 1: Basic Enhanced Allocation");
    
    // Create a vector with enhanced tracking
    let mut data = Vec::<u64>::with_capacity(1000);
    track_allocation!(data, "demo_vector", "Vec<u64>");
    
    // Fill with data to trigger reallocations
    for i in 0..1500 {
        data.push(i);
        
        // Track significant growth points
        if data.len() % 500 == 0 {
            track_lifecycle_event!(data, "growth_milestone", format!("Size: {}", data.len()));
        }
    }
    
    // Simulate some processing
    let sum: u64 = data.iter().sum();
    tracing::info!("📈 Processed {} elements, sum: {}", data.len(), sum);
    
    // Track final state before drop
    track_lifecycle_event!(data, "final_processing", "Ready for cleanup");
    
    Ok(())
}

/// Demo 2: Unsafe FFI operations with passport tracking
fn demo_unsafe_ffi_operations(tracker: &MemoryTracker) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🌍 Demo 2: Unsafe FFI Operations");
    
    // Create data for FFI
    let input_data = CString::new("Hello, FFI World!")?;
    let input_ptr = input_data.as_ptr() as *mut c_char;
    
    // Track the handover to FFI
    track_ffi_boundary!(input_ptr, "handover_to_c", "process_data");
    
    unsafe {
        // Call the FFI function
        let result_ptr = process_data(input_ptr, input_data.as_bytes().len());
        
        if !result_ptr.is_null() {
            // Track the return from FFI
            track_ffi_boundary!(result_ptr, "return_from_c", "process_data");
            
            // Convert back to Rust string
            let result_cstring = CString::from_raw(result_ptr);
            let result_str = result_cstring.to_str()?;
            tracing::info!("🔄 FFI Result: {}", result_str);
            
            // Note: result_cstring will be dropped here, freeing the memory
            track_ffi_boundary!(result_ptr, "reclaimed_by_rust", "automatic_drop");
        }
    }
    
    // Track potential unsafe operations
    unsafe {
        let raw_ptr = input_ptr as *const u8;
        let _byte_value = *raw_ptr; // Potentially unsafe dereference
        track_unsafe_operation!(raw_ptr, "raw_pointer_dereference", "Reading first byte");
    }
    
    Ok(())
}

/// Demo 3: Complex ownership transfers
fn demo_ownership_transfers(tracker: &MemoryTracker) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🔄 Demo 3: Ownership Transfers");
    
    // Create original data
    let original_data = vec![1, 2, 3, 4, 5];
    track_allocation!(original_data, "original_data", "Vec<i32>");
    
    // Clone the data (ownership duplication)
    let cloned_data = original_data.clone();
    track_ownership_event!(cloned_data, "cloned", "Data cloned for parallel processing");
    
    // Move data to a function (ownership transfer)
    let processed_data = process_data_ownership(original_data);
    track_ownership_event!(processed_data, "ownership_transferred", "Moved to processing function");
    
    // Create a reference (borrowing)
    let data_ref = &processed_data;
    track_ownership_event!(data_ref, "borrowed", "Immutable borrow for reading");
    
    tracing::info!("📋 Final data: {:?}", data_ref);
    
    // Both cloned_data and processed_data will be dropped here
    track_ownership_event!(cloned_data, "dropped", "End of scope - clone");
    track_ownership_event!(processed_data, "dropped", "End of scope - processed");
    
    Ok(())
}

fn process_data_ownership(mut data: Vec<i32>) -> Vec<i32> {
    // Simulate processing
    data.iter_mut().for_each(|x| *x *= 2);
    data
}

/// Demo 4: Smart pointer analysis
fn demo_smart_pointer_analysis(tracker: &MemoryTracker) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🧠 Demo 4: Smart Pointer Analysis");
    
    use std::rc::Rc;
    use std::sync::Arc;
    use std::cell::RefCell;
    
    // Rc (Reference Counted) pointer
    let rc_data = Rc::new(vec![10, 20, 30]);
    track_smart_pointer!(rc_data, "rc_pointer", "Rc<Vec<i32>>");
    
    let rc_clone1 = Rc::clone(&rc_data);
    let rc_clone2 = Rc::clone(&rc_data);
    track_smart_pointer_event!(rc_clone1, "rc_cloned", format!("Ref count: {}", Rc::strong_count(&rc_data)));
    
    // Arc (Atomic Reference Counted) for thread safety
    let arc_data = Arc::new(vec![100, 200, 300]);
    track_smart_pointer!(arc_data, "arc_pointer", "Arc<Vec<i32>>");
    
    // RefCell for interior mutability
    let refcell_data = RefCell::new(vec![1000, 2000]);
    track_smart_pointer!(refcell_data, "refcell_pointer", "RefCell<Vec<i32>>");
    
    // Demonstrate borrowing from RefCell
    {
        let borrowed = refcell_data.borrow();
        track_smart_pointer_event!(borrowed, "refcell_borrowed", "Immutable borrow");
        tracing::info!("📖 RefCell data: {:?}", *borrowed);
    }
    
    {
        let mut borrowed_mut = refcell_data.borrow_mut();
        borrowed_mut.push(3000);
        track_smart_pointer_event!(borrowed_mut, "refcell_borrowed_mut", "Mutable borrow");
        tracing::info!("✏️ Modified RefCell data: {:?}", *borrowed_mut);
    }
    
    tracing::info!("🔢 Final Rc strong count: {}", Rc::strong_count(&rc_data));
    tracing::info!("🔢 Final Arc strong count: {}", Arc::strong_count(&arc_data));
    
    Ok(())
}

/// Export enhanced data using new binary format and JSON/HTML generation
fn export_enhanced_data(tracker: &MemoryTracker) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("📤 Exporting Enhanced Data");
    
    // Create enhanced export configuration
    let config = BinaryExportConfig::debug_comprehensive();
    
    // Export to binary format first
    let binary_path = "enhanced_demo_data.bin";
    tracker.export_to_binary_with_mode(binary_path, memscope_rs::BinaryExportMode::UserOnly)?;
    tracing::info!("💾 Binary data exported to: {}", binary_path);
    
    // Generate enhanced JSON files
    let json_options = OptimizedExportOptions::with_optimization_level(OptimizationLevel::High);
    
    // Export JSON using the tracker's method
    tracker.export_to_json("enhanced_demo_memory_analysis.json")?;
    tracing::info!("📋 Enhanced JSON files generated");
    
    // Generate enhanced HTML dashboard
    export_binary_to_html_system(binary_path, "enhanced_demo")?;
    tracing::info!("🌐 Enhanced HTML dashboard generated");
    
    // Print summary
    print_export_summary(tracker)?;
    
    Ok(())
}

fn print_export_summary(tracker: &MemoryTracker) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("📊 Export Summary:");
    tracing::info!("  📁 Files generated:");
    tracing::info!("    • enhanced_demo_data.bin (Binary format)");
    tracing::info!("    • enhanced_demo_memory_analysis.json");
    tracing::info!("    • enhanced_demo_lifetime.json");
    tracing::info!("    • enhanced_demo_performance.json");
    tracing::info!("    • enhanced_demo_unsafe_ffi.json");
    tracing::info!("    • enhanced_demo_complex_types.json");
    tracing::info!("    • enhanced_demo.html (Full dashboard)");
    tracing::info!("    • enhanced_demo_light.html (Lightweight)");
    tracing::info!("    • enhanced_demo_progressive.html (Progressive)");
    
    let allocations = tracker.get_active_allocations()?;
    tracing::info!("  📈 Statistics:");
    tracing::info!("    • Total allocations tracked: {}", allocations.len());
    tracing::info!("    • Enhanced features enabled: ✅");
    tracing::info!("    • Memory passport tracking: ✅");
    tracing::info!("    • Ownership history: ✅");
    tracing::info!("    • Unsafe operation analysis: ✅");
    tracing::info!("    • Smart pointer analysis: ✅");
    
    Ok(())
}

