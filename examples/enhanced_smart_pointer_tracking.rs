//! Enhanced smart pointer tracking demonstration
//!
//! This example demonstrates the new smart pointer tracking capabilities:
//! - Clone relationship tracking
//! - Reference count history
//! - Data lifetime vs instance lifetime separation
//! - Weak reference integration

use memscope_rs::{init, track_var};
use std::collections::HashMap;
use std::rc::{Rc, Weak as RcWeak};
use std::sync::{Arc, Weak as ArcWeak};

fn main() {
    init();

    println!("🚀 Enhanced Smart Pointer Tracking Demo");
    println!("=======================================");

    // Test Rc clone relationship tracking
    println!("\n📦 Testing Rc<T> clone relationships:");

    // Create original Rc
    let original_data = Rc::new("Shared data".to_string());
    let _tracked_original = track_var!(original_data);
    println!(
        "✅ Created original Rc<String> (ref_count: {})",
        Rc::strong_count(&original_data)
    );

    // Create clones and track relationships
    let clone1 = original_data;
    let _tracked_clone1 = track_var!(clone1);
    println!(
        "✅ Created clone1 (ref_count: {})",
        Rc::strong_count(&original_data)
    );

    let clone2 = original_data;
    let _tracked_clone2 = track_var!(clone2);
    println!(
        "✅ Created clone2 (ref_count: {})",
        Rc::strong_count(&original_data)
    );

    let clone3 = original_data;
    let _tracked_clone3 = track_var!(clone3);
    println!(
        "✅ Created clone3 (ref_count: {})",
        Rc::strong_count(&original_data)
    );

    // Create weak references
    let weak1: RcWeak<String> = Rc::downgrade(&original_data);
    let _tracked_weak1 = track_var!(weak1);
    println!(
        "✅ Created weak1 (weak_count: {})",
        Rc::weak_count(&original_data)
    );

    let weak2: RcWeak<String> = Rc::downgrade(&original_data);
    let _tracked_weak2 = track_var!(weak2);
    println!(
        "✅ Created weak2 (weak_count: {})",
        Rc::weak_count(&original_data)
    );

    // Test Arc clone relationship tracking
    println!("\n🔄 Testing Arc<T> clone relationships:");

    let shared_vec = Arc::new(vec![1, 2, 3, 4, 5]);
    let _tracked_arc_original = track_var!(shared_vec);
    println!(
        "✅ Created original Arc<Vec<i32>> (ref_count: {})",
        Arc::strong_count(&shared_vec)
    );

    let arc_clone1 = shared_vec;
    let _tracked_arc_clone1 = track_var!(arc_clone1);
    println!(
        "✅ Created Arc clone1 (ref_count: {})",
        Arc::strong_count(&shared_vec)
    );

    let arc_clone2 = shared_vec;
    let _tracked_arc_clone2 = track_var!(arc_clone2);
    println!(
        "✅ Created Arc clone2 (ref_count: {})",
        Arc::strong_count(&shared_vec)
    );

    let arc_weak: ArcWeak<Vec<i32>> = Arc::downgrade(&shared_vec);
    let _tracked_arc_weak = track_var!(arc_weak);
    println!(
        "✅ Created Arc weak reference (weak_count: {})",
        Arc::weak_count(&shared_vec)
    );

    // Test complex nested structures with smart pointers
    println!("\n🏗️ Testing complex nested smart pointer structures:");

    let complex_data = Rc::new(HashMap::<String, Arc<Vec<String>>>::new());
    let _tracked_complex = track_var!(complex_data);

    // Add some nested data
    {
        let vec_data1 = Arc::new(vec!["item1".to_string(), "item2".to_string()]);
        let vec_data2 = Arc::new(vec!["item3".to_string(), "item4".to_string()]);

        let _tracked_vec1 = track_var!(vec_data1);
        let _tracked_vec2 = track_var!(vec_data2);

        // Create clones of the Arc data
        let vec_clone1 = vec_data1;
        let vec_clone2 = vec_data2;
        let _tracked_vec_clone1 = track_var!(vec_clone1);
        let _tracked_vec_clone2 = track_var!(vec_clone2);

        println!("✅ Created nested Arc<Vec<String>> structures");
        println!("   vec_data1 ref_count: {}", Arc::strong_count(&vec_data1));
        println!("   vec_data2 ref_count: {}", Arc::strong_count(&vec_data2));
    }

    // Test reference count changes over time
    println!("\n📊 Testing reference count changes:");

    let shared_string = Rc::new("Reference counting test".to_string());
    let _tracked_shared = track_var!(shared_string);
    println!("Initial ref_count: {}", Rc::strong_count(&shared_string));

    {
        let temp_clone1 = shared_string;
        let _tracked_temp1 = track_var!(temp_clone1);
        println!("After temp_clone1: {}", Rc::strong_count(&shared_string));

        {
            let temp_clone2 = shared_string;
            let _tracked_temp2 = track_var!(temp_clone2);
            println!("After temp_clone2: {}", Rc::strong_count(&shared_string));

            {
                let temp_clone3 = shared_string;
                let _tracked_temp3 = track_var!(temp_clone3);
                println!("After temp_clone3: {}", Rc::strong_count(&shared_string));
            }
            println!(
                "After temp_clone3 dropped: {}",
                Rc::strong_count(&shared_string)
            );
        }
        println!(
            "After temp_clone2 dropped: {}",
            Rc::strong_count(&shared_string)
        );
    }
    println!(
        "After temp_clone1 dropped: {}",
        Rc::strong_count(&shared_string)
    );

    // Test data lifetime vs instance lifetime
    println!("\n🔍 Testing data lifetime vs instance lifetime:");

    let data_owner = Rc::new(42i32);
    let _tracked_owner = track_var!(data_owner);
    let data_ptr = Rc::as_ptr(&data_owner) as usize;
    println!("Created data owner, data_ptr: 0x{:x}", data_ptr);

    let instance1 = data_owner;
    let _tracked_instance1 = track_var!(instance1);
    println!(
        "Created instance1, same data_ptr: 0x{:x}",
        Rc::as_ptr(&instance1) as usize
    );

    let instance2 = data_owner;
    let _tracked_instance2 = track_var!(instance2);
    println!(
        "Created instance2, same data_ptr: 0x{:x}",
        Rc::as_ptr(&instance2) as usize
    );

    // Drop the original owner
    drop(_tracked_owner);
    println!(
        "Dropped original owner, ref_count: {}",
        Rc::strong_count(&instance1)
    );

    // Test weak reference upgrade/downgrade cycles
    println!("\n🔄 Testing weak reference upgrade/downgrade:");

    let strong_ref = Rc::new("Weak reference test".to_string());
    let _tracked_strong = track_var!(strong_ref);

    let weak_ref = Rc::downgrade(&strong_ref);
    let _tracked_weak = track_var!(weak_ref);
    println!(
        "Created weak reference, can upgrade: {}",
        weak_ref.upgrade().is_some()
    );

    // Try multiple upgrades
    for i in 0..3 {
        if let Some(upgraded) = weak_ref.upgrade() {
            let _tracked_upgraded = track_var!(upgraded);
            println!(
                "Upgrade {} successful, ref_count: {}",
                i + 1,
                Rc::strong_count(&strong_ref)
            );
        }
    }

    println!("\n🎯 Enhanced tracking features demonstrated:");
    println!("   ✅ Clone relationship tracking for Rc/Arc");
    println!("   ✅ Reference count history recording");
    println!("   ✅ Data pointer grouping for related instances");
    println!("   ✅ Weak reference integration");
    println!("   ✅ Instance lifetime vs data lifetime separation");
    println!("   ✅ Complex nested smart pointer structures");

    // Export analysis with enhanced smart pointer data
    use memscope_rs::get_global_tracker;
    let tracker = get_global_tracker();
    if let Err(e) = tracker.export_to_json("enhanced_smart_pointer_analysis.json") {
        println!("❌ Failed to export analysis: {}", e);
    } else {
        println!("📄 Enhanced analysis exported to enhanced_smart_pointer_analysis.json");
        println!("   This file now contains:");
        println!("   - Clone relationship data");
        println!("   - Reference count history");
        println!("   - Smart pointer metadata");
        println!("   - Data lifetime information");
    }
}
