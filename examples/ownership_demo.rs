//! Demonstration of different tracking modes and ownership patterns
//!
//! This example shows how to use the improved track_var! macro with different
//! ownership semantics to avoid the borrow checker issues.

use memscope_rs::{init, track_var, track_var_owned};

fn main() {
    // Initialize memory tracking
    init();

    println!("🎯 Demonstrating improved track_var! ownership patterns\n");

    // ========================================
    // 1. Non-invasive tracking (recommended)
    // ========================================
    println!("1️⃣ Non-invasive tracking (default):");
    let my_vec = vec![1, 2, 3, 4, 5];

    // Track without taking ownership
    track_var!(my_vec);

    // Original variable remains fully usable!
    println!("✅ Vec length: {}", my_vec.len());
    println!("✅ Vec contents: {:?}", my_vec);
    println!("✅ Can still use my_vec normally!\n");

    // ========================================
    // 2. Ownership-based tracking
    // ========================================
    println!("2️⃣ Ownership-based tracking:");
    let another_vec = vec![10, 20, 30];

    // Explicitly move ownership to get a wrapper
    let tracked_vec = track_var_owned!(another_vec);

    // Use through the wrapper (Deref makes it transparent)
    println!("✅ Tracked vec length: {}", tracked_vec.len());
    println!("✅ Tracked vec contents: {:?}", *tracked_vec);

    // Can get the original value back if needed
    let original_back = tracked_vec.into_inner();
    println!("✅ Got original back: {:?}\n", original_back);

    // ========================================
    // 3. Clone-based tracking
    // ========================================
    println!("3️⃣ Clone-based tracking:");
    let shared_data = vec!["hello", "world"];

    // Track a clone, original remains untouched
    let tracked_clone = track_var_owned!(shared_data.clone());

    // Both original and tracked clone are usable
    println!("✅ Original: {:?}", shared_data);
    println!("✅ Tracked clone: {:?}", *tracked_clone);
    println!("✅ Both are independent!\n");

    // ========================================
    // 4. Smart pointer patterns
    // ========================================
    println!("4️⃣ Smart pointer tracking:");
    use std::rc::Rc;

    let rc_data = Rc::new(vec![1, 2, 3]);

    // Non-invasive tracking of Rc
    track_var!(rc_data);

    // Clone the Rc (cheap reference count increment)
    let rc_clone = Rc::clone(&rc_data);
    track_var!(rc_clone);

    println!("✅ Original Rc: {:?}", rc_data);
    println!("✅ Cloned Rc: {:?}", rc_clone);
    println!("✅ Reference count: {}", Rc::strong_count(&rc_data));

    // ========================================
    // 5. Complex ownership scenarios
    // ========================================
    println!("\n5️⃣ Complex ownership scenarios:");

    let mut complex_data = vec![
        String::from("first"),
        String::from("second"),
        String::from("third"),
    ];

    // Track the vector
    track_var!(complex_data);

    // Can still modify it
    complex_data.push(String::from("fourth"));
    println!("✅ Modified after tracking: {:?}", complex_data);

    // Track individual elements if needed
    for (i, item) in complex_data.iter().enumerate() {
        // This would work for heap-allocated strings
        track_var!(*item);
        println!("  📍 Tracked item {}: {}", i, item);
    }

    println!("\n🎉 All tracking patterns work without ownership conflicts!");
}
