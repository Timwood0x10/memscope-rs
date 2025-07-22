//! Fixed lifecycle showcase demonstrating the new track_var! ownership patterns
//!
//! This example shows how to use the improved track_var! macro to avoid
//! ownership conflicts while still getting comprehensive memory tracking.

use memscope_rs::{init, track_var, track_var_owned};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    // Initialize memory tracking
    init();

    println!("🎯 Fixed Lifecycle Showcase - No Ownership Conflicts!\n");

    // ========================================
    // 1. Basic collections with non-invasive tracking
    // ========================================
    println!("1️⃣ Basic Collections:");

    let mut small_vec = Vec::with_capacity(5);
    for i in 0..5 {
        small_vec.push(i);
    }
    track_var!(small_vec); // Non-invasive tracking
    println!("✅ Small Vec: {} items", small_vec.len()); // Works!

    let mut large_vec = Vec::with_capacity(1000);
    for i in 0..100 {
        large_vec.push(format!("Item {}", i));
    }
    track_var!(large_vec); // Non-invasive tracking
    println!("✅ Large Vec: {} items", large_vec.len()); // Works!

    let mut growing_string = String::new();
    for i in 0..10 {
        growing_string.push_str(&format!("Part {} ", i));
    }
    track_var!(growing_string); // Non-invasive tracking
    println!("✅ Growing String: {} bytes", growing_string.len()); // Works!

    let static_string = String::from("Static content that doesn't grow");
    track_var!(static_string); // Non-invasive tracking
    println!("✅ Static String: {} bytes", static_string.len()); // Works!

    // ========================================
    // 2. Boxed collections
    // ========================================
    println!("\n2️⃣ Boxed Collections:");

    let mut hash_map = HashMap::new();
    hash_map.insert("key1".to_string(), 100);
    hash_map.insert("key2".to_string(), 200);
    let boxed_hash_map = Box::new(hash_map);
    track_var!(boxed_hash_map); // Non-invasive tracking
    println!("✅ Box<HashMap>: {} entries", boxed_hash_map.len()); // Works!

    let mut hash_set = HashSet::new();
    hash_set.insert("item1".to_string());
    hash_set.insert("item2".to_string());
    let boxed_hash_set = Box::new(hash_set);
    track_var!(boxed_hash_set); // Non-invasive tracking
    println!("✅ Box<HashSet>: {} unique items", boxed_hash_set.len()); // Works!

    let mut btree_map = BTreeMap::new();
    btree_map.insert(1, "first".to_string());
    btree_map.insert(2, "second".to_string());
    let boxed_btree_map = Box::new(btree_map);
    track_var!(boxed_btree_map); // Non-invasive tracking
    println!("✅ Box<BTreeMap>: {} sorted entries", boxed_btree_map.len()); // Works!

    // ========================================
    // 3. Smart pointers
    // ========================================
    println!("\n3️⃣ Smart Pointers:");

    let shared_data = Rc::new(vec![1, 2, 3, 4, 5]);
    track_var!(shared_data); // Non-invasive tracking
    let shared_clone1 = Rc::clone(&shared_data); // Works!
    let shared_clone2 = Rc::clone(&shared_data); // Works!
    track_var!(shared_clone1);
    track_var!(shared_clone2);

    println!("✅ Shared data: {} items", shared_data.len());
    println!("✅ Reference count: {}", Rc::strong_count(&shared_data));

    let thread_safe_data = Arc::new(String::from("Thread-safe shared string"));
    track_var!(thread_safe_data); // Non-invasive tracking
    let arc_clone = Arc::clone(&thread_safe_data); // Works!
    track_var!(arc_clone);

    println!("✅ Thread-safe data: {} bytes", thread_safe_data.len());
    println!(
        "✅ Arc reference count: {}",
        Arc::strong_count(&thread_safe_data)
    );

    let mutable_data = Rc::new(RefCell::new(vec![10, 20, 30]));
    track_var!(mutable_data); // Non-invasive tracking
    {
        let mut borrowed = mutable_data.borrow_mut(); // Works!
        borrowed.push(40);
    }
    println!("✅ Mutable data: {} items", mutable_data.borrow().len());

    // ========================================
    // 4. Demonstrating different tracking modes
    // ========================================
    println!("\n4️⃣ Different Tracking Modes:");

    // Mode 1: Non-invasive (default)
    let data1 = vec![1, 2, 3];
    track_var!(data1);
    println!("✅ Non-invasive: {} items", data1.len());

    // Mode 2: Move semantics (when you want a wrapper)
    let data2 = vec![4, 5, 6];
    let tracked_data2 = track_var_owned!(data2);
    println!("✅ Move semantics: {} items", tracked_data2.len());

    // Mode 3: Clone semantics (when you want both)
    let data3 = vec![7, 8, 9];
    let tracked_data3 = track_var_owned!(data3.clone());
    println!(
        "✅ Clone semantics - Original: {}, Clone: {}",
        data3.len(),
        tracked_data3.len()
    );

    // ========================================
    // 5. Complex scenarios that now work
    // ========================================
    println!("\n5️⃣ Complex Scenarios:");

    let mut keep_alive: Vec<Box<dyn std::any::Any>> = Vec::new();

    let boxed_large_data = Box::new(vec![0u8; 1024]);
    track_var!(boxed_large_data); // Non-invasive tracking

    let boxed_string = Box::new(String::from("Boxed string data"));
    track_var!(boxed_string); // Non-invasive tracking

    // Now we can use them after tracking!
    keep_alive.push(Box::new(boxed_large_data.len()) as Box<dyn std::any::Any>);
    keep_alive.push(Box::new(boxed_string.len()) as Box<dyn std::any::Any>);

    println!("✅ Keep alive vector: {} items", keep_alive.len());

    println!("\n🎉 All tracking completed without ownership conflicts!");
    println!("📊 Check the generated reports for detailed memory analysis.");
}
