//! Demonstration of the #[derive(Trackable)] macro and extended built-in type support
//!
//! This example shows how to use the derive macro to automatically implement
//! Trackable for custom types, and demonstrates tracking of various built-in types.

use memscope_rs::{init, track_var};
use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet, VecDeque, LinkedList, BinaryHeap};
use std::sync::{Arc, Mutex, RwLock};
use std::rc::{Rc, Weak};
use std::cell::RefCell;

// Enable the derive feature for this example
#[cfg(feature = "derive")]
use memscope_rs::Trackable;

// Example custom types using the derive macro
#[cfg(feature = "derive")]
#[derive(Trackable)]
struct UserProfile {
    name: String,
    email: String,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
    scores: Vec<i32>,
}

#[cfg(feature = "derive")]
#[derive(Trackable)]
struct NestedData {
    profiles: Vec<UserProfile>,
    cache: BTreeMap<String, Vec<u8>>,
    description: String,
}

#[cfg(feature = "derive")]
#[derive(Trackable)]
struct TupleStruct(String, Vec<i32>, HashMap<u32, String>);

#[cfg(feature = "derive")]
#[derive(Trackable)]
enum DataEnum {
    Text(String),
    Numbers(Vec<i32>),
    Map(HashMap<String, i32>),
    Empty,
}

#[cfg(feature = "derive")]
#[derive(Trackable)]
struct UnitStruct;

fn main() {
    init();
    
    println!("🚀 Derive Macro and Extended Built-in Types Demo");
    println!("================================================");
    
    // Test derive macro functionality
    #[cfg(feature = "derive")]
    {
        println!("\n📦 Testing #[derive(Trackable)] macro:");
        
        // Create a complex user profile
        let user = UserProfile {
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            tags: vec!["developer".to_string(), "rust".to_string(), "memory".to_string()],
            metadata: {
                let mut map = HashMap::new();
                map.insert("department".to_string(), "Engineering".to_string());
                map.insert("level".to_string(), "Senior".to_string());
                map
            },
            scores: vec![95, 87, 92, 88, 94],
        };
        let _tracked_user = track_var!(user);
        println!("✅ Created UserProfile with derive macro");
        
        // Create nested data structure
        let nested = NestedData {
            profiles: vec![],
            cache: {
                let mut btree = BTreeMap::new();
                btree.insert("key1".to_string(), vec![1, 2, 3, 4, 5]);
                btree.insert("key2".to_string(), vec![6, 7, 8, 9, 10]);
                btree
            },
            description: "Complex nested structure".to_string(),
        };
        let _tracked_nested = track_var!(nested);
        println!("✅ Created NestedData with derive macro");
        
        // Test tuple struct
        let tuple_data = TupleStruct(
            "tuple data".to_string(),
            vec![1, 2, 3, 4, 5],
            {
                let mut map = HashMap::new();
                map.insert(1, "one".to_string());
                map.insert(2, "two".to_string());
                map
            }
        );
        let _tracked_tuple = track_var!(tuple_data);
        println!("✅ Created TupleStruct with derive macro");
        
        // Test enum variants
        let enum_text = DataEnum::Text("Hello, World!".to_string());
        let _tracked_enum1 = track_var!(enum_text);
        
        let enum_numbers = DataEnum::Numbers(vec![10, 20, 30, 40, 50]);
        let _tracked_enum2 = track_var!(enum_numbers);
        
        let enum_map = DataEnum::Map({
            let mut map = HashMap::new();
            map.insert("count".to_string(), 42);
            map.insert("total".to_string(), 100);
            map
        });
        let _tracked_enum3 = track_var!(enum_map);
        
        let enum_empty = DataEnum::Empty;
        let _tracked_enum4 = track_var!(enum_empty);
        println!("✅ Created DataEnum variants with derive macro");
        
        // Test unit struct
        let unit = UnitStruct;
        let _tracked_unit = track_var!(unit);
        println!("✅ Created UnitStruct with derive macro");
    }
    
    #[cfg(not(feature = "derive"))]
    {
        println!("⚠️  Derive feature not enabled. Enable with --features derive");
    }
    
    println!("\n🔧 Testing extended built-in type support:");
    
    // Test various collection types
    let mut hash_map: HashMap<String, i32> = HashMap::new();
    hash_map.insert("apple".to_string(), 5);
    hash_map.insert("banana".to_string(), 3);
    hash_map.insert("cherry".to_string(), 8);
    let _tracked_hashmap = track_var!(hash_map);
    println!("✅ HashMap<String, i32>");
    
    let mut btree_map: BTreeMap<i32, String> = BTreeMap::new();
    btree_map.insert(1, "first".to_string());
    btree_map.insert(2, "second".to_string());
    btree_map.insert(3, "third".to_string());
    let _tracked_btreemap = track_var!(btree_map);
    println!("✅ BTreeMap<i32, String>");
    
    let mut hash_set: HashSet<String> = HashSet::new();
    hash_set.insert("rust".to_string());
    hash_set.insert("memory".to_string());
    hash_set.insert("tracking".to_string());
    let _tracked_hashset = track_var!(hash_set);
    println!("✅ HashSet<String>");
    
    let mut btree_set: BTreeSet<i32> = BTreeSet::new();
    btree_set.insert(10);
    btree_set.insert(20);
    btree_set.insert(30);
    let _tracked_btreeset = track_var!(btree_set);
    println!("✅ BTreeSet<i32>");
    
    let mut vec_deque: VecDeque<String> = VecDeque::new();
    vec_deque.push_back("first".to_string());
    vec_deque.push_back("second".to_string());
    vec_deque.push_front("zero".to_string());
    let _tracked_vecdeque = track_var!(vec_deque);
    println!("✅ VecDeque<String>");
    
    let mut linked_list: LinkedList<i32> = LinkedList::new();
    linked_list.push_back(1);
    linked_list.push_back(2);
    linked_list.push_back(3);
    let _tracked_linkedlist = track_var!(linked_list);
    println!("✅ LinkedList<i32>");
    
    let mut binary_heap: BinaryHeap<i32> = BinaryHeap::new();
    binary_heap.push(10);
    binary_heap.push(5);
    binary_heap.push(15);
    binary_heap.push(3);
    let _tracked_binaryheap = track_var!(binary_heap);
    println!("✅ BinaryHeap<i32>");
    
    // Test smart pointers and synchronization primitives
    let rc_data = Rc::new("Shared data".to_string());
    let _tracked_rc = track_var!(rc_data.clone());
    let weak_ref: Weak<String> = Rc::downgrade(&rc_data);
    let _tracked_weak = track_var!(weak_ref);
    println!("✅ Rc<String> and Weak<String>");
    
    let arc_data = Arc::new(vec![1, 2, 3, 4, 5]);
    let _tracked_arc = track_var!(arc_data.clone());
    let weak_arc = Arc::downgrade(&arc_data);
    let _tracked_weak_arc = track_var!(weak_arc);
    println!("✅ Arc<Vec<i32>> and Weak<Vec<i32>>");
    
    let ref_cell = RefCell::new("Mutable data".to_string());
    let _tracked_refcell = track_var!(ref_cell);
    println!("✅ RefCell<String>");
    
    let mutex_data = Mutex::new(HashMap::<String, i32>::new());
    let _tracked_mutex = track_var!(mutex_data);
    println!("✅ Mutex<HashMap<String, i32>>");
    
    let rwlock_data = RwLock::new(vec!["read", "write", "lock"]);
    let _tracked_rwlock = track_var!(rwlock_data);
    println!("✅ RwLock<Vec<&str>>");
    
    // Test Option and Result types
    let some_data: Option<Vec<String>> = Some(vec!["option".to_string(), "data".to_string()]);
    let _tracked_option = track_var!(some_data);
    println!("✅ Option<Vec<String>>");
    
    let none_data: Option<String> = None;
    let _tracked_none = track_var!(none_data);
    println!("✅ Option<String> (None)");
    
    let ok_result: Result<String, String> = Ok("Success!".to_string());
    let _tracked_ok = track_var!(ok_result);
    println!("✅ Result<String, String> (Ok)");
    
    let err_result: Result<Vec<i32>, String> = Err("Error occurred".to_string());
    let _tracked_err = track_var!(err_result);
    println!("✅ Result<Vec<i32>, String> (Err)");
    
    println!("\n🎯 All types successfully tracked!");
    println!("📊 Memory analysis will be exported automatically on program exit");
    
    // Export the results
    use memscope_rs::get_global_tracker;
    let tracker = get_global_tracker();
    if let Err(e) = tracker.export_to_json("derive_demo_analysis.json") {
        println!("❌ Failed to export analysis: {}", e);
    } else {
        println!("📄 Analysis exported to derive_demo_analysis.json");
    }
}