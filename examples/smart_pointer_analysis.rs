//! Smart pointer analysis demonstration
//!
//! This example demonstrates the current smart pointer tracking capabilities
//! and showcases areas for future improvement based on rc_arc_improvements.md

use memscope_rs::{init, track_var};
use std::rc::{Rc, Weak as RcWeak};
use std::sync::{Arc, Weak as ArcWeak};
use std::collections::HashMap;

fn main() {
    init();
    
    println!("🧠 Smart Pointer Analysis Demo");
    println!("==============================");
    
    // Test Rc and Weak references
    println!("\n📦 Testing Rc<T> and Weak<T> relationships:");
    
    // Create original Rc
    let original_data = Rc::new("Shared data".to_string());
    let _tracked_original = track_var!(original_data.clone());
    println!("✅ Created original Rc<String>");
    
    // Create clones
    let clone1 = original_data.clone();
    let _tracked_clone1 = track_var!(clone1);
    println!("✅ Created clone1 (ref_count should be 3)");
    
    let clone2 = original_data.clone();
    let _tracked_clone2 = track_var!(clone2);
    println!("✅ Created clone2 (ref_count should be 4)");
    
    // Create weak references
    let weak1: RcWeak<String> = Rc::downgrade(&original_data);
    let _tracked_weak1 = track_var!(weak1);
    println!("✅ Created weak1 from original");
    
    let weak2: RcWeak<String> = Rc::downgrade(&original_data);
    let _tracked_weak2 = track_var!(weak2);
    println!("✅ Created weak2 from original");
    
    // Test Arc and Weak references
    println!("\n🔄 Testing Arc<T> and Weak<T> relationships:");
    
    let shared_vec = Arc::new(vec![1, 2, 3, 4, 5]);
    let _tracked_arc_original = track_var!(shared_vec.clone());
    println!("✅ Created original Arc<Vec<i32>>");
    
    let arc_clone1 = shared_vec.clone();
    let _tracked_arc_clone1 = track_var!(arc_clone1);
    println!("✅ Created Arc clone1");
    
    let arc_weak: ArcWeak<Vec<i32>> = Arc::downgrade(&shared_vec);
    let _tracked_arc_weak = track_var!(arc_weak);
    println!("✅ Created Arc weak reference");
    
    // Test complex nested structures
    println!("\n🏗️ Testing complex nested smart pointers:");
    
    let complex_data = Rc::new(HashMap::<String, Arc<Vec<String>>>::new());
    let _tracked_complex = track_var!(complex_data.clone());
    
    // Add some data to the HashMap
    {
        let mut map = HashMap::new();
        let vec_data = Arc::new(vec!["item1".to_string(), "item2".to_string()]);
        map.insert("key1".to_string(), vec_data.clone());
        
        let _tracked_vec_data = track_var!(vec_data);
        let _tracked_map = track_var!(map);
    }
    
    println!("✅ Created complex nested structure");
    
    // Test data pointer relationships
    println!("\n🔍 Analyzing data pointer relationships:");
    
    // Create multiple Rc instances pointing to the same data
    let shared_string = "This is shared data".to_string();
    let rc1 = Rc::new(shared_string);
    let rc2 = rc1.clone();
    let rc3 = rc1.clone();
    
    let _tracked_rc1 = track_var!(rc1);
    let _tracked_rc2 = track_var!(rc2);
    let _tracked_rc3 = track_var!(rc3);
    
    println!("✅ Created 3 Rc instances sharing the same data");
    println!("   Note: All should have the same data_ptr but different heap_ptr");
    
    // Test weak reference upgrade/downgrade
    println!("\n🔄 Testing weak reference operations:");
    
    let strong_ref = Rc::new(42i32);
    let weak_ref = Rc::downgrade(&strong_ref);
    
    let _tracked_strong = track_var!(strong_ref);
    let _tracked_weak_before = track_var!(weak_ref.clone());
    
    // Try to upgrade
    if let Some(upgraded) = weak_ref.upgrade() {
        let _tracked_upgraded = track_var!(upgraded);
        println!("✅ Successfully upgraded weak reference");
    }
    
    // Drop the strong reference
    drop(_tracked_strong);
    
    // Try to upgrade again (should fail)
    let weak_ref_clone = weak_ref.clone();
    let _tracked_weak_after = track_var!(weak_ref_clone);
    if weak_ref.upgrade().is_none() {
        println!("✅ Weak reference upgrade failed after strong ref dropped");
    }
    
    println!("\n📊 Analysis complete!");
    println!("💡 Current tracking captures:");
    println!("   - Individual Rc/Arc instance lifecycles");
    println!("   - Reference counts at creation time");
    println!("   - Data pointers for grouping related instances");
    println!("   - Weak reference tracking with upgrade capability");
    
    println!("\n🚀 Future improvements (from rc_arc_improvements.md):");
    println!("   - Clone relationship tracking");
    println!("   - Data lifetime vs instance lifetime separation");
    println!("   - Reference count change history");
    println!("   - Circular reference detection");
    println!("   - Enhanced visualization of smart pointer relationships");
    
    // Export analysis
    use memscope_rs::get_global_tracker;
    let tracker = get_global_tracker();
    if let Err(e) = tracker.export_to_json("smart_pointer_analysis.json") {
        println!("❌ Failed to export analysis: {}", e);
    } else {
        println!("📄 Analysis exported to smart_pointer_analysis.json");
    }
}