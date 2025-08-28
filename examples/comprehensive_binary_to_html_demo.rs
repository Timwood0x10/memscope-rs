//! Comprehensive Binary to HTML Demo
//! 
//! This example demonstrates the full binary to HTML conversion functionality,
//! showcasing all trackable types and improve.md extensions including:
//! - Basic types (primitives, collections)
//! - Smart pointers (Arc, Rc, Box, RefCell)
//! - Complex generic types
//! - Trait objects and dynamic dispatch
//! - Async/await contexts
//! - FFI and unsafe operations
//! - Memory layout analysis
//! - Borrow checker interactions
//! - Clone optimization tracking
//! - Ownership transfer patterns

use memscope_rs::core::types::{AllocationInfo, BorrowInfo, CloneInfo};
use memscope_rs::export::binary;
use std::path::Path;
use tempfile::TempDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Comprehensive Binary to HTML Demo");
    println!("=====================================");
    
    // Create temporary directory for output files
    let temp_dir = TempDir::new()?;
    let binary_path = temp_dir.path().join("comprehensive_demo.memscope");
    let html_path = temp_dir.path().join("comprehensive_report.html");
    
    // Generate comprehensive allocation data
    let allocations = create_comprehensive_allocations();
    
    println!("📊 Generated {} allocations covering all trackable types", allocations.len());
    
    // Export to binary format
    println!("💾 Exporting to binary format...");
    binary::export_to_binary(&allocations, &binary_path)?;
    
    let binary_size = std::fs::metadata(&binary_path)?.len();
    println!("   Binary file size: {} bytes", binary_size);
    
    // Convert binary to HTML using binary_dashboard.html template
    println!("🎨 Converting binary to HTML report...");
    println!("🔄 Calling parse_binary_to_html_direct...");
    binary::parse_binary_to_html_direct(&binary_path, &html_path, "Comprehensive Memory Analysis Demo")?;
    println!("✅ parse_binary_to_html_direct completed");
    
    let html_size = std::fs::metadata(&html_path)?.len();
    println!("   HTML file size: {} bytes", html_size);
    
    // Display results
    println!("\n✅ Conversion completed successfully!");
    println!("📁 Files generated:");
    println!("   Binary: {}", binary_path.display());
    println!("   HTML:   {}", html_path.display());
    
    // Copy files to current directory for easy access
    let current_binary = Path::new("comprehensive_demo.memscope");
    let current_html = Path::new("comprehensive_report.html");
    
    std::fs::copy(&binary_path, current_binary)?;
    std::fs::copy(&html_path, current_html)?;
    
    println!("\n📋 Files copied to current directory:");
    println!("   📄 comprehensive_demo.memscope");
    println!("   🌐 comprehensive_report.html");
    
    // Analyze the generated HTML
    analyze_html_content(&current_html)?;
    
    println!("\n🎯 Demo completed! Open 'comprehensive_report.html' in your browser to see the results.");
    
    Ok(())
}

/// Create comprehensive allocation data covering all trackable types and scenarios
fn create_comprehensive_allocations() -> Vec<AllocationInfo> {
    let mut allocations = Vec::new();
    let mut ptr_counter = 0x1000;
    
    // Helper to get next pointer
    let mut next_ptr = || {
        ptr_counter += 0x100;
        ptr_counter
    };
    
    // 1. Basic Collections
    allocations.push(create_allocation(
        next_ptr(),
        1024,
        "user_data",
        "Vec<User>",
        "main",
        "main",
        false,
        Some(BorrowInfo {
            immutable_borrows: 5,
            mutable_borrows: 2,
            max_concurrent_borrows: 3,
            last_borrow_timestamp: Some(1234567890),
        }),
        Some(CloneInfo {
            clone_count: 2,
            is_clone: false,
            original_ptr: None,
        }),
        true,
        None, // Simplified
        Some(150),
    ));
    
    // 2. Hash Map with Complex Key-Value Types
    allocations.push(create_allocation(
        next_ptr(),
        2048,
        "cache_storage",
        "HashMap<String, Arc<Mutex<CacheEntry>>>",
        "cache_module",
        "worker_1",
        false,
        Some(BorrowInfo {
            immutable_borrows: 15,
            mutable_borrows: 3,
            max_concurrent_borrows: 8,
            last_borrow_timestamp: Some(1234567920),
        }),
        Some(CloneInfo {
            clone_count: 0, // HashMap doesn't clone, it moves
            is_clone: false,
            original_ptr: None,
        }),
        true,
        None, // Simplified
        Some(300),
    ));
    
    // 3. Smart Pointer - Arc with Reference Counting
    allocations.push(create_allocation(
        next_ptr(),
        512,
        "shared_config",
        "Arc<AppConfig>",
        "config_manager",
        "main",
        false,
        Some(BorrowInfo {
            immutable_borrows: 25, // Arc allows many immutable borrows
            mutable_borrows: 0,    // Arc doesn't allow mutable borrows
            max_concurrent_borrows: 25,
            last_borrow_timestamp: Some(1234567950),
        }),
        Some(CloneInfo {
            clone_count: 8, // Arc is frequently cloned
            is_clone: false,
            original_ptr: None,
        }),
        true,
        None, // Simplified
        Some(500),
    ));
    
    // 4. Box with Heap Allocation
    allocations.push(create_allocation(
        next_ptr(),
        256,
        "large_object",
        "Box<LargeStruct>",
        "processing_module",
        "worker_2",
        false,
        Some(BorrowInfo {
            immutable_borrows: 3,
            mutable_borrows: 1,
            max_concurrent_borrows: 2,
            last_borrow_timestamp: Some(1234567980),
        }),
        Some(CloneInfo {
            clone_count: 0, // Box moves, doesn't clone
            is_clone: false,
            original_ptr: None,
        }),
        true,
        None, // Simplified
        Some(75),
    ));
    
    // 5. RefCell with Interior Mutability
    allocations.push(create_allocation(
        next_ptr(),
        128,
        "mutable_state",
        "Rc<RefCell<GameState>>",
        "game_engine",
        "main",
        false,
        Some(BorrowInfo {
            immutable_borrows: 8,
            mutable_borrows: 4,
            max_concurrent_borrows: 6,
            last_borrow_timestamp: Some(1234568000),
        }),
        Some(CloneInfo {
            clone_count: 5, // Rc is cloned
            is_clone: false,
            original_ptr: None,
        }),
        true,
        None, // Simplified
        Some(200),
    ));
    
    // 6. Generic Type with Multiple Parameters
    allocations.push(create_allocation(
        next_ptr(),
        1536,
        "generic_container",
        "BTreeMap<UserId, Vec<Message<JsonValue>>>",
        "messaging_system",
        "async_runtime",
        false,
        Some(BorrowInfo {
            immutable_borrows: 12,
            mutable_borrows: 2,
            max_concurrent_borrows: 7,
            last_borrow_timestamp: Some(1234568030),
        }),
        Some(CloneInfo {
            clone_count: 1,
            is_clone: false,
            original_ptr: None,
        }),
        true,
        None, // Simplified
        Some(450),
    ));
    
    // 7. Trait Object with Dynamic Dispatch
    allocations.push(create_allocation(
        next_ptr(),
        64,
        "event_handler",
        "Box<dyn EventHandler + Send + Sync>",
        "event_system",
        "event_loop",
        false,
        Some(BorrowInfo {
            immutable_borrows: 6,
            mutable_borrows: 1,
            max_concurrent_borrows: 4,
            last_borrow_timestamp: Some(1234568060),
        }),
        Some(CloneInfo {
            clone_count: 0, // Trait objects can't be cloned
            is_clone: false,
            original_ptr: None,
        }),
        true,
        None, // Simplified
        Some(100),
    ));
    
    // 8. Async Context Allocation
    allocations.push(create_allocation(
        next_ptr(),
        896,
        "future_state",
        "Pin<Box<dyn Future<Output = Result<Response, Error>>>>",
        "async_handler",
        "tokio_runtime",
        false,
        Some(BorrowInfo {
            immutable_borrows: 4,
            mutable_borrows: 2,
            max_concurrent_borrows: 3,
            last_borrow_timestamp: Some(1234568090),
        }),
        Some(CloneInfo {
            clone_count: 0, // Futures are moved, not cloned
            is_clone: false,
            original_ptr: None,
        }),
        true,
        None, // Simplified
        Some(1200),
    ));
    
    // 9. FFI and Unsafe Allocation
    allocations.push(create_allocation(
        next_ptr(),
        4096,
        "ffi_buffer",
        "*mut c_void",
        "ffi_module",
        "main",
        false,
        Some(BorrowInfo {
            immutable_borrows: 0, // Raw pointers don't participate in borrow checking
            mutable_borrows: 0,
            max_concurrent_borrows: 0,
            last_borrow_timestamp: None,
        }),
        Some(CloneInfo {
            clone_count: 0, // Raw pointers are copied, not cloned
            is_clone: false,
            original_ptr: None,
        }),
        false, // No ownership history for raw pointers
        None, // Simplified
        Some(800),
    ));
    
    // 10. Memory Leak Example
    allocations.push(create_allocation(
        next_ptr(),
        2048,
        "leaked_data",
        "Vec<String>",
        "buggy_function",
        "worker_3",
        true, // This is leaked
        Some(BorrowInfo {
            immutable_borrows: 2,
            mutable_borrows: 1,
            max_concurrent_borrows: 2,
            last_borrow_timestamp: Some(1234568120),
        }),
        Some(CloneInfo {
            clone_count: 1,
            is_clone: false,
            original_ptr: None,
        }),
        true,
        None, // Simplified
        None, // No lifetime for leaked memory
    ));
    
    // 11. Circular Reference Detection
    allocations.push(create_allocation(
        next_ptr(),
        384,
        "node_a",
        "Rc<RefCell<Node>>",
        "graph_structure",
        "main",
        false,
        Some(BorrowInfo {
            immutable_borrows: 6,
            mutable_borrows: 3,
            max_concurrent_borrows: 5,
            last_borrow_timestamp: Some(1234568150),
        }),
        Some(CloneInfo {
            clone_count: 4, // Rc cloned for circular references
            is_clone: false,
            original_ptr: None,
        }),
        true,
        None, // Simplified
        Some(600),
    ));
    
    // 12. High-Performance Buffer
    allocations.push(create_allocation(
        next_ptr(),
        8192,
        "ring_buffer",
        "Vec<u8>",
        "network_io",
        "io_thread",
        false,
        Some(BorrowInfo {
            immutable_borrows: 20,
            mutable_borrows: 8,
            max_concurrent_borrows: 12,
            last_borrow_timestamp: Some(1234568180),
        }),
        Some(CloneInfo {
            clone_count: 0, // Zero-copy operations
            is_clone: false,
            original_ptr: None,
        }),
        true,
        None, // Simplified
        Some(50), // Very short-lived
    ));
    
    // 13. Clone-Heavy Data Structure
    allocations.push(create_allocation(
        next_ptr(),
        1024,
        "cloned_config",
        "AppConfig",
        "config_distribution",
        "worker_4",
        false,
        Some(BorrowInfo {
            immutable_borrows: 8,
            mutable_borrows: 0,
            max_concurrent_borrows: 8,
            last_borrow_timestamp: Some(1234568210),
        }),
        Some(CloneInfo {
            clone_count: 15, // Heavily cloned
            is_clone: true,
            original_ptr: Some(0x1300), // Reference to original Arc<AppConfig>
        }),
        true,
        None, // Simplified
        Some(25), // Short-lived clone
    ));
    
    // 14. Thread-Local Storage
    allocations.push(create_allocation(
        next_ptr(),
        256,
        "thread_local_cache",
        "ThreadLocal<HashMap<String, CachedValue>>",
        "thread_local_storage",
        "worker_5",
        false,
        Some(BorrowInfo {
            immutable_borrows: 12,
            mutable_borrows: 4,
            max_concurrent_borrows: 8,
            last_borrow_timestamp: Some(1234568240),
        }),
        Some(CloneInfo {
            clone_count: 0, // Thread-local data isn't cloned across threads
            is_clone: false,
            original_ptr: None,
        }),
        true,
        None, // Simplified
        Some(2000), // Long-lived thread-local data
    ));
    
    // 15. Deallocated Memory (for comparison)
    allocations.push(create_deallocated_allocation(
        next_ptr(),
        512,
        "temp_buffer",
        "Vec<u8>",
        "temporary_processing",
        "worker_1",
        1234567890,
        1234567990, // Deallocated after 100ms
        Some(BorrowInfo {
            immutable_borrows: 3,
            mutable_borrows: 2,
            max_concurrent_borrows: 3,
            last_borrow_timestamp: Some(1234567985),
        }),
        Some(CloneInfo {
            clone_count: 1,
            is_clone: false,
            original_ptr: None,
        }),
        100,
    ));
    
    println!("📋 Created allocations covering:");
    println!("   • Basic collections (Vec, HashMap, BTreeMap)");
    println!("   • Smart pointers (Arc, Rc, Box, RefCell)");
    println!("   • Generic types with complex parameters");
    println!("   • Trait objects with dynamic dispatch");
    println!("   • Async/Future contexts");
    println!("   • FFI and unsafe allocations");
    println!("   • Memory leaks and circular references");
    println!("   • High-performance and clone-heavy scenarios");
    println!("   • Thread-local storage");
    println!("   • Deallocated memory examples");
    
    allocations
}

/// Create a standard allocation with all improve.md extensions
fn create_allocation(
    ptr: usize,
    size: usize,
    var_name: &str,
    type_name: &str,
    scope_name: &str,
    thread_id: &str,
    is_leaked: bool,
    borrow_info: Option<BorrowInfo>,
    clone_info: Option<CloneInfo>,
    ownership_history_available: bool,
    _extra_info: Option<()>, // Simplified
    lifetime_ms: Option<u64>,
) -> AllocationInfo {
    AllocationInfo {
        ptr,
        size,
        var_name: Some(var_name.to_string()),
        type_name: Some(type_name.to_string()),
        scope_name: Some(scope_name.to_string()),
        timestamp_alloc: 1234567890,
        timestamp_dealloc: None,
        thread_id: thread_id.to_string(),
        borrow_count: borrow_info.as_ref().map(|b| b.immutable_borrows + b.mutable_borrows).unwrap_or(0),
        stack_trace: Some(vec![
            format!("{}::{}", scope_name, var_name),
            "main::run".to_string(),
            "std::rt::lang_start".to_string(),
        ]),
        is_leaked,
        lifetime_ms,
        borrow_info,
        clone_info,
        ownership_history_available,
        smart_pointer_info: None, // Simplified for demo
        memory_layout: None, // Simplified for demo
        generic_info: None, // Simplified for demo
        dynamic_type_info: None, // Simplified for demo
        runtime_state: None, // Simplified for demo
        stack_allocation: None, // Simplified for demo
        temporary_object: None, // Simplified for demo
        fragmentation_analysis: None, // Simplified for demo
        generic_instantiation: None, // Simplified for demo
        type_relationships: None, // Simplified for demo
        type_usage: None, // Simplified for demo
        function_call_tracking: None, // Simplified for demo
        lifecycle_tracking: None, // Simplified for demo
        access_tracking: None, // Simplified for demo
        drop_chain_analysis: None, // Simplified for demo
    }
}

/// Create a deallocated allocation for comparison
fn create_deallocated_allocation(
    ptr: usize,
    size: usize,
    var_name: &str,
    type_name: &str,
    scope_name: &str,
    thread_id: &str,
    timestamp_alloc: u64,
    timestamp_dealloc: u64,
    borrow_info: Option<BorrowInfo>,
    clone_info: Option<CloneInfo>,
    lifetime_ms: u64,
) -> AllocationInfo {
    let mut allocation = create_allocation(
        ptr, size, var_name, type_name, scope_name, thread_id,
        false, borrow_info, clone_info, true, None, Some(lifetime_ms)
    );
    allocation.timestamp_alloc = timestamp_alloc;
    allocation.timestamp_dealloc = Some(timestamp_dealloc);
    allocation
}

/// Extract type parameters from generic type names
fn _extract_type_parameters(type_name: &str) -> Vec<String> {
    if let Some(start) = type_name.find('<') {
        if let Some(end) = type_name.rfind('>') {
            let params = &type_name[start + 1..end];
            return params.split(',').map(|s| s.trim().to_string()).collect();
        }
    }
    vec![]
}

/// Extract parent types from type names
fn _extract_parent_types(type_name: &str) -> Vec<String> {
    if type_name.contains("Arc") { vec!["Arc".to_string()] }
    else if type_name.contains("Rc") { vec!["Rc".to_string()] }
    else if type_name.contains("Box") { vec!["Box".to_string()] }
    else if type_name.contains("Vec") { vec!["Vec".to_string()] }
    else if type_name.contains("HashMap") { vec!["HashMap".to_string()] }
    else { vec![] }
}

/// Extract associated types from type names
fn _extract_associated_types(type_name: &str) -> Vec<String> {
    let mut types = vec![];
    if type_name.contains("Iterator") { types.push("Item".to_string()); }
    if type_name.contains("Future") { types.push("Output".to_string()); }
    if type_name.contains("Result") { types.push("Error".to_string()); }
    types
}

/// Analyze the generated HTML content
fn analyze_html_content(html_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(html_path)?;
    
    println!("\n📊 HTML Analysis:");
    println!("   File size: {} bytes", content.len());
    
    // Count different sections
    let allocation_count = content.matches("ptr").count();
    let borrow_info_count = content.matches("borrow_info").count();
    let clone_info_count = content.matches("clone_info").count();
    let smart_pointer_count = content.matches("smart_pointer").count();
    
    println!("   Allocations displayed: {}", allocation_count);
    println!("   Borrow info entries: {}", borrow_info_count);
    println!("   Clone info entries: {}", clone_info_count);
    println!("   Smart pointer entries: {}", smart_pointer_count);
    
    // Check for key features
    let features = [
        ("Modern CSS Grid", content.contains("stats-grid")),
        ("Interactive Table", content.contains("allocations-table")),
        ("JavaScript Data", content.contains("const allocations")),
        ("Memory Formatting", content.contains("formatSize")),
        ("Project Branding", content.contains("Comprehensive Memory Analysis Demo")),
        ("Responsive Design", content.contains("viewport")),
        ("Clean Template", content.contains("Memory Analysis Report")),
    ];
    
    println!("   Features detected:");
    for (feature, present) in features {
        println!("     {} {}", if present { "✅" } else { "❌" }, feature);
    }
    
    Ok(())
}