//! Comprehensive Rust Types Demo with JSON Export and Binary Comparison
//! 
//! This example demonstrates:
//! 1. Basic and advanced Rust types
//! 2. Unsafe/FFI operations  
//! 3. Export user-binary format (only user-defined variables)
//! 4. Generate direct JSON and binary-parsed JSON for comparison
//! 5. Compare differences between direct JSON and binary-parsed JSON

use memscope_rs::{track_var_owned, get_global_tracker, get_global_unsafe_ffi_tracker, Trackable, BinaryExportMode};
use memscope_rs::export::binary::{export_binary_to_html_system, BinaryParser};
use std::collections::{HashMap, BTreeMap, HashSet, VecDeque};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::ffi::CString;
use std::fs;
use chrono::Utc;

// Custom struct for testing
#[derive(Clone)]
struct UserData {
    id: u64,
    name: String,
    scores: Vec<i32>,
    metadata: HashMap<String, String>,
}

impl Trackable for UserData {
    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<Self>() + 
        self.name.capacity() + 
        self.scores.capacity() * 4 + 
        self.metadata.capacity() * 64
    }
    
    fn get_heap_ptr(&self) -> Option<usize> {
        Some(self as *const _ as usize)
    }
    
    fn get_type_name(&self) -> &'static str {
        "UserData"
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 Comprehensive Rust Types Demo with JSON Export");
    println!("=================================================");

    let tracker = get_global_tracker();
    let ffi_tracker = get_global_unsafe_ffi_tracker();

    // ===== Create Various Rust Types =====
    println!("\n📊 Creating Various Rust Types...");
    
    // String
    let owned_string = String::from("Hello, Rust Memory Tracking!");
    let tracked_string = track_var_owned!(owned_string);
    println!("✅ String: {}", tracked_string);

    // Vector
    let vector = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let tracked_vector = track_var_owned!(vector);
    println!("✅ Vec<i32>: {} elements", tracked_vector.len());

    // HashMap
    let mut hash_map = HashMap::new();
    hash_map.insert("rust".to_string(), 2015);
    hash_map.insert("python".to_string(), 1991);
    hash_map.insert("javascript".to_string(), 1995);
    let tracked_hashmap = track_var_owned!(hash_map);
    println!("✅ HashMap: {} entries", tracked_hashmap.len());

    // BTreeMap
    let mut btree_map = BTreeMap::new();
    btree_map.insert("alpha".to_string(), 1);
    btree_map.insert("beta".to_string(), 2);
    btree_map.insert("gamma".to_string(), 3);
    let tracked_btreemap = track_var_owned!(btree_map);
    println!("✅ BTreeMap: {} entries", tracked_btreemap.len());

    // HashSet
    let mut hash_set = HashSet::new();
    hash_set.insert("red".to_string());
    hash_set.insert("green".to_string());
    hash_set.insert("blue".to_string());
    let tracked_hashset = track_var_owned!(hash_set);
    println!("✅ HashSet: {} items", tracked_hashset.len());

    // VecDeque
    let mut vec_deque = VecDeque::new();
    vec_deque.push_back("first".to_string());
    vec_deque.push_back("second".to_string());
    vec_deque.push_front("zeroth".to_string());
    let tracked_vecdeque = track_var_owned!(vec_deque);
    println!("✅ VecDeque: {} items", tracked_vecdeque.len());

    // Smart Pointers
    let rc_data = Rc::new("Shared data via Rc".to_string());
    let tracked_rc = track_var_owned!(rc_data);
    println!("✅ Rc<String>: ref count {}", tracked_rc.get_ref_count());

    let rc_clone = Rc::clone(&tracked_rc);
    let tracked_rc_clone = track_var_owned!(rc_clone);
    println!("✅ Rc clone: ref count {}", Rc::strong_count(&tracked_rc_clone));

    let arc_data = Arc::new(vec!["thread", "safe", "data"]);
    let tracked_arc = track_var_owned!(arc_data);
    println!("✅ Arc<Vec<&str>>: ref count {}", tracked_arc.get_ref_count());

    let refcell_data = RefCell::new(vec![10, 20, 30]);
    let tracked_refcell = track_var_owned!(refcell_data);
    {
        let mut borrowed = tracked_refcell.borrow_mut();
        borrowed.push(40);
        borrowed.push(50);
    }
    println!("✅ RefCell<Vec<i32>>: {} elements", tracked_refcell.borrow().len());

    let mutex_data = Mutex::new(HashMap::<String, i32>::new());
    let tracked_mutex = track_var_owned!(mutex_data);
    {
        let mut map = tracked_mutex.lock().unwrap();
        map.insert("key1".to_string(), 100);
        map.insert("key2".to_string(), 200);
    }
    println!("✅ Mutex<HashMap>: {} entries", tracked_mutex.lock().unwrap().len());

    // Custom Types
    let mut user_data_list = Vec::new();
    for i in 0..3 {
        let mut metadata = HashMap::new();
        metadata.insert("created_at".to_string(), format!("2024-01-{:02}", i + 1));
        metadata.insert("status".to_string(), if i % 2 == 0 { "active" } else { "inactive" }.to_string());
        
        let user_data = UserData {
            id: i as u64,
            name: format!("User_{}", i),
            scores: vec![i * 10, i * 20, i * 30],
            metadata,
        };
        
        let tracked_user = track_var_owned!(user_data);
        user_data_list.push(tracked_user.into_inner());
    }
    println!("✅ Created {} UserData objects", user_data_list.len());

    // ===== Unsafe/FFI Operations =====
    println!("\n⚠️ Unsafe/FFI Operations...");
    
    for i in 0..3 {
        let data = format!("FFI operation {}", i);
        let c_str = CString::new(data)?;
        let tracked_ffi = track_var_owned!(c_str);
        
        unsafe {
            let ptr = tracked_ffi.as_ptr();
            ffi_tracker.track_ffi_allocation(
                ptr as usize,
                tracked_ffi.as_bytes().len(),
                "test_lib".to_string(),
                format!("operation_{}", i),
            )?;
            
            ffi_tracker.track_unsafe_allocation(
                ptr as usize,
                tracked_ffi.as_bytes().len(),
                format!("Unsafe allocation {}", i),
            )?;
        }
    }
    println!("✅ FFI operations: 3 operations tracked");

    // ===== Export Analysis =====
    println!("\n📊 Export and Analysis...");

    // Get statistics
    let stats = tracker.get_stats()?;
    println!("📈 Total allocations: {}", stats.total_allocations);
    println!("📈 Active memory: {} bytes", stats.active_memory);
    println!("📈 Peak memory: {} bytes", stats.peak_memory);

    let all_allocations = tracker.get_active_allocations()?;
    let user_allocations: Vec<_> = all_allocations
        .iter()
        .filter(|alloc| alloc.var_name.is_some())
        .collect();
    
    println!("📊 Total allocations: {}", all_allocations.len());
    println!("📊 User allocations: {}", user_allocations.len());

    // ===== STEP 1: Export User Binary (only user-defined variables) =====
    println!("\n💾 STEP 1: Export User Binary (user-defined variables only)...");
    let binary_path = "comprehensive_user_binary.memscope";
    tracker.export_to_binary_with_mode(binary_path, BinaryExportMode::UserOnly)?;
    
    let actual_binary_path = if std::path::Path::new("MemoryAnalysis/comprehensive_user_binary.memscope").exists() {
        "MemoryAnalysis/comprehensive_user_binary.memscope"
    } else {
        binary_path
    };
    
    let binary_size = std::fs::metadata(actual_binary_path)?.len();
    println!("✅ User binary exported: {} ({} bytes)", actual_binary_path, binary_size);

    // ===== STEP 2: Export Direct JSON =====
    println!("\n📋 STEP 2: Export Direct JSON...");
    tracker.export_to_json("comprehensive_direct_json")?;
    
    // Create directory structure for direct JSON
    let direct_json_dir = "MemoryAnalysis/direct_json_analysis";
    std::fs::create_dir_all(direct_json_dir)?;
    
    // Check if JSON files were created and move them to organized directory
    let default_json_dir = "MemoryAnalysis/comprehensive_direct_json_analysis";
    if std::path::Path::new(default_json_dir).exists() {
        println!("✅ Direct JSON files created, organizing...");
        for entry in std::fs::read_dir(default_json_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let source = entry.path();
            let dest = std::path::Path::new(direct_json_dir).join(&file_name);
            std::fs::copy(&source, &dest)?;
            let size = entry.metadata()?.len();
            println!("  📄 {}: {} bytes", file_name.to_string_lossy(), size);
        }
    }

    // ===== STEP 3: Parse Binary to JSON =====
    println!("\n📋 STEP 3: Parse Binary to JSON...");
    BinaryParser::parse_user_binary_to_json(actual_binary_path, "comprehensive_binary_parsed")?;
    
    // Create directory structure for binary-parsed JSON
    let binary_json_dir = "MemoryAnalysis/binary_parsed_json_analysis";
    std::fs::create_dir_all(binary_json_dir)?;
    
    // Check if binary-parsed JSON files were created and move them
    let default_binary_json_dir = "MemoryAnalysis/comprehensive_binary_parsed_analysis";
    if std::path::Path::new(default_binary_json_dir).exists() {
        println!("✅ Binary-parsed JSON files created, organizing...");
        for entry in std::fs::read_dir(default_binary_json_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let source = entry.path();
            let dest = std::path::Path::new(binary_json_dir).join(&file_name);
            std::fs::copy(&source, &dest)?;
            let size = entry.metadata()?.len();
            println!("  📄 {}: {} bytes", file_name.to_string_lossy(), size);
        }
    }

    // ===== STEP 4: Compare JSON Files =====
    println!("\n🔍 STEP 4: Compare Direct JSON vs Binary-Parsed JSON...");
    
    // Function to compare two JSON files
    fn compare_json_files(direct_path: &str, binary_path: &str, file_type: &str) -> Result<(), Box<dyn std::error::Error>> {
        if std::path::Path::new(direct_path).exists() && std::path::Path::new(binary_path).exists() {
            let direct_content = std::fs::read_to_string(direct_path)?;
            let binary_content = std::fs::read_to_string(binary_path)?;
            
            let direct_size = direct_content.len();
            let binary_size = binary_content.len();
            
            println!("📊 {} Analysis:", file_type);
            println!("   Direct JSON: {} bytes", direct_size);
            println!("   Binary-parsed JSON: {} bytes", binary_size);
            
            if direct_content == binary_content {
                println!("   ✅ Content IDENTICAL");
            } else {
                println!("   ⚠️  Content DIFFERENT");
                
                // Try to parse as JSON and compare structure
                if let (Ok(direct_json), Ok(binary_json)) = (
                    serde_json::from_str::<serde_json::Value>(&direct_content),
                    serde_json::from_str::<serde_json::Value>(&binary_content)
                ) {
                    if direct_json == binary_json {
                        println!("   ✅ JSON structure IDENTICAL (formatting differences only)");
                    } else {
                        println!("   ❌ JSON structure DIFFERENT");
                        
                        // Save comparison report
                        let comparison_report = format!(
                            "=== {} COMPARISON REPORT ===\n\
                            Direct JSON size: {} bytes\n\
                            Binary-parsed JSON size: {} bytes\n\
                            Content identical: {}\n\
                            Structure identical: {}\n\n\
                            Direct JSON keys: {:?}\n\
                            Binary JSON keys: {:?}\n",
                            file_type,
                            direct_size,
                            binary_size,
                            direct_content == binary_content,
                            direct_json == binary_json,
                            if direct_json.is_object() { 
                                Some(direct_json.as_object().unwrap().keys().collect::<Vec<_>>()) 
                            } else { None },
                            if binary_json.is_object() { 
                                Some(binary_json.as_object().unwrap().keys().collect::<Vec<_>>()) 
                            } else { None }
                        );
                        
                        let report_path = format!("MemoryAnalysis/{}_comparison_report.txt", file_type.to_lowercase().replace(" ", "_"));
                        std::fs::write(&report_path, comparison_report)?;
                        println!("   📄 Detailed comparison saved: {}", report_path);
                    }
                }
            }
            println!();
        } else {
            println!("⚠️  {} files not found for comparison", file_type);
        }
        Ok(())
    }
    
    // Compare common JSON files
    compare_json_files(
        "MemoryAnalysis/direct_json_analysis/memory_analysis.json",
        "MemoryAnalysis/binary_parsed_json_analysis/memory_analysis.json",
        "Memory Analysis"
    )?;
    
    compare_json_files(
        "MemoryAnalysis/direct_json_analysis/lifecycle_analysis.json",
        "MemoryAnalysis/binary_parsed_json_analysis/lifecycle_analysis.json",
        "Lifecycle Analysis"
    )?;
    
    compare_json_files(
        "MemoryAnalysis/direct_json_analysis/performance_analysis.json",
        "MemoryAnalysis/binary_parsed_json_analysis/performance_analysis.json",
        "Performance Analysis"
    )?;

    // ===== STEP 5: Generate Overall Comparison Report =====
    println!("\n📋 STEP 5: Generate Overall Comparison Report...");
    
    let mut overall_report = String::new();
    overall_report.push_str("=== COMPREHENSIVE JSON COMPARISON REPORT ===\n\n");
    overall_report.push_str(&format!("Generated: {}\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    overall_report.push_str(&format!("User Binary: {} ({} bytes)\n\n", actual_binary_path, binary_size));
    
    overall_report.push_str("TRACKED DATA SUMMARY:\n");
    overall_report.push_str(&format!("• Total allocations: {}\n", all_allocations.len()));
    overall_report.push_str(&format!("• User allocations: {}\n", user_allocations.len()));
    overall_report.push_str(&format!("• Active memory: {} bytes\n", stats.active_memory));
    overall_report.push_str(&format!("• Peak memory: {} bytes\n\n", stats.peak_memory));
    
    overall_report.push_str("RUST TYPES TRACKED:\n");
    overall_report.push_str("• Basic types: String, Vec, HashMap, BTreeMap, HashSet, VecDeque\n");
    overall_report.push_str("• Smart pointers: Rc, Arc, RefCell, Mutex\n");
    overall_report.push_str("• Custom types: UserData with complex fields\n");
    overall_report.push_str("• Unsafe/FFI: 3 FFI operations tracked\n\n");
    
    overall_report.push_str("EXPORT COMPARISON:\n");
    overall_report.push_str("• User-binary format: Only user-defined variables stored\n");
    overall_report.push_str("• Direct JSON: Generated directly from memory tracker\n");
    overall_report.push_str("• Binary-parsed JSON: Generated by parsing user-binary file\n\n");
    
    overall_report.push_str("DIRECTORY STRUCTURE:\n");
    overall_report.push_str("• MemoryAnalysis/direct_json_analysis/ - Direct JSON exports\n");
    overall_report.push_str("• MemoryAnalysis/binary_parsed_json_analysis/ - Binary-parsed JSON exports\n");
    overall_report.push_str("• MemoryAnalysis/*_comparison_report.txt - Individual comparison reports\n\n");
    
    // Check for any additional comparison files
    if let Ok(entries) = std::fs::read_dir("MemoryAnalysis") {
        let mut comparison_files = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                if file_name.to_string_lossy().contains("comparison_report") {
                    comparison_files.push(file_name.to_string_lossy().to_string());
                }
            }
        }
        if !comparison_files.is_empty() {
            overall_report.push_str("COMPARISON REPORTS GENERATED:\n");
            for file in comparison_files {
                overall_report.push_str(&format!("• {}\n", file));
            }
            overall_report.push_str("\n");
        }
    }
    
    overall_report.push_str("KEY FINDINGS:\n");
    overall_report.push_str("• User-binary format successfully stores only user-defined variable memory activities\n");
    overall_report.push_str("• Binary parsing reconstructs JSON data from binary format\n");
    overall_report.push_str("• Comparison reveals any differences between direct and binary-parsed JSON\n");
    overall_report.push_str("• This analysis helps validate binary format integrity and completeness\n");
    
    let report_path = "MemoryAnalysis/comprehensive_comparison_report.txt";
    std::fs::write(report_path, &overall_report)?;
    println!("✅ Overall comparison report saved: {}", report_path);

    // ===== STEP 6: Summary =====
    println!("\n🎯 Summary");
    println!("==========");
    println!("✅ Rust types: String, Vec, HashMap, BTreeMap, HashSet, VecDeque");
    println!("✅ Smart pointers: Rc, Arc, RefCell, Mutex");
    println!("✅ Custom types: UserData with complex fields");
    println!("✅ Unsafe/FFI: 3 FFI operations tracked");
    println!("✅ User-binary export: {} bytes (user-defined variables only)", binary_size);
    println!("✅ Direct JSON: Generated from memory tracker");
    println!("✅ Binary-parsed JSON: Generated from user-binary file");
    println!("✅ Comparison analysis: Completed with detailed reports");
    
    println!("\n📁 Generated Files and Directories:");
    println!("   • User Binary: {}", actual_binary_path);
    println!("   • Direct JSON: MemoryAnalysis/direct_json_analysis/");
    println!("   • Binary-parsed JSON: MemoryAnalysis/binary_parsed_json_analysis/");
    println!("   • Comparison Reports: MemoryAnalysis/*_comparison_report.txt");
    println!("   • Overall Report: {}", report_path);

    println!("\n🎉 Comprehensive JSON vs Binary Comparison Analysis COMPLETED!");
    println!("📊 Check the MemoryAnalysis directory for detailed comparison results.");

    Ok(())
}