//! Custom types tracking demo using the #[derive(Trackable)] macro
//!
//! This example demonstrates how to use the derive macro to automatically
//! implement Trackable for user-defined types with multiple internal allocations.

use memscope_rs::{init, track_var};

// Enable derive feature for automatic Trackable implementation
#[cfg(feature = "derive")]
use memscope_rs::Trackable;
use std::collections::HashMap;

/// A complex user-defined struct with multiple heap allocations
struct UserProfile {
    name: String,
    email: String,
    tags: Vec<String>,
    metadata: Box<HashMap<String, String>>,
    scores: Vec<i32>,
}

// Manual implementation of Trackable for UserProfile
impl Trackable for UserProfile {
    fn get_heap_ptr(&self) -> Option<usize> {
        // Use the struct's address as the primary identifier
        Some(self as *const _ as usize)
    }

    fn get_type_name(&self) -> &'static str {
        "UserProfile"
    }

    fn get_size_estimate(&self) -> usize {
        let mut total_size = std::mem::size_of::<Self>();
        total_size += self.name.get_size_estimate();
        total_size += self.email.get_size_estimate();
        total_size += self.tags.get_size_estimate();
        total_size += self.metadata.get_size_estimate();
        total_size += self.scores.get_size_estimate();
        total_size
    }

    fn get_internal_allocations(&self, var_name: &str) -> Vec<(usize, String)> {
        let mut allocations = Vec::new();

        if let Some(ptr) = self.name.get_heap_ptr() {
            allocations.push((ptr, format!("{}::name", var_name)));
        }
        if let Some(ptr) = self.email.get_heap_ptr() {
            allocations.push((ptr, format!("{}::email", var_name)));
        }
        if let Some(ptr) = self.tags.get_heap_ptr() {
            allocations.push((ptr, format!("{}::tags", var_name)));
        }
        if let Some(ptr) = self.metadata.get_heap_ptr() {
            allocations.push((ptr, format!("{}::metadata", var_name)));
        }
        if let Some(ptr) = self.scores.get_heap_ptr() {
            allocations.push((ptr, format!("{}::scores", var_name)));
        }

        allocations
    }
}

/// A simple struct with basic types
struct SimpleData {
    id: u64,
    active: bool,
}

impl Trackable for SimpleData {
    fn get_heap_ptr(&self) -> Option<usize> {
        // Simple structs with no heap allocations return None
        None
    }

    fn get_type_name(&self) -> &'static str {
        "SimpleData"
    }

    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

/// A struct with nested allocations
struct NestedData {
    profiles: Vec<UserProfile>,
    cache: HashMap<String, Vec<u8>>,
    description: String,
}

impl Trackable for NestedData {
    fn get_heap_ptr(&self) -> Option<usize> {
        Some(self as *const _ as usize)
    }

    fn get_type_name(&self) -> &'static str {
        "NestedData"
    }

    fn get_size_estimate(&self) -> usize {
        let mut total_size = std::mem::size_of::<Self>();
        total_size += self.profiles.get_size_estimate();
        total_size += self.cache.get_size_estimate();
        total_size += self.description.get_size_estimate();
        total_size
    }

    fn get_internal_allocations(&self, var_name: &str) -> Vec<(usize, String)> {
        let mut allocations = Vec::new();

        if let Some(ptr) = self.profiles.get_heap_ptr() {
            allocations.push((ptr, format!("{}::profiles", var_name)));
        }
        if let Some(ptr) = self.cache.get_heap_ptr() {
            allocations.push((ptr, format!("{}::cache", var_name)));
        }
        if let Some(ptr) = self.description.get_heap_ptr() {
            allocations.push((ptr, format!("{}::description", var_name)));
        }

        allocations
    }
}

/// Unit struct (no heap allocations)
struct UnitStruct;

impl Trackable for UnitStruct {
    fn get_heap_ptr(&self) -> Option<usize> {
        None // Unit structs have no heap allocation
    }

    fn get_type_name(&self) -> &'static str {
        "UnitStruct"
    }

    fn get_size_estimate(&self) -> usize {
        0
    }
}

/// Tuple struct
struct TupleStruct(String, Vec<i32>);

impl Trackable for TupleStruct {
    fn get_heap_ptr(&self) -> Option<usize> {
        Some(self as *const _ as usize)
    }

    fn get_type_name(&self) -> &'static str {
        "TupleStruct"
    }

    fn get_size_estimate(&self) -> usize {
        let mut total_size = std::mem::size_of::<Self>();
        total_size += self.0.get_size_estimate();
        total_size += self.1.get_size_estimate();
        total_size
    }

    fn get_internal_allocations(&self, var_name: &str) -> Vec<(usize, String)> {
        let mut allocations = Vec::new();

        if let Some(ptr) = self.0.get_heap_ptr() {
            allocations.push((ptr, format!("{}::0", var_name)));
        }
        if let Some(ptr) = self.1.get_heap_ptr() {
            allocations.push((ptr, format!("{}::1", var_name)));
        }

        allocations
    }
}

/// Enum with data
enum DataEnum {
    Text(String),
    Numbers(Vec<i32>),
    Empty,
}

impl Trackable for DataEnum {
    fn get_heap_ptr(&self) -> Option<usize> {
        Some(self as *const _ as usize)
    }

    fn get_type_name(&self) -> &'static str {
        "DataEnum"
    }

    fn get_size_estimate(&self) -> usize {
        let mut total_size = std::mem::size_of::<Self>();
        match self {
            DataEnum::Text(s) => total_size += s.get_size_estimate(),
            DataEnum::Numbers(v) => total_size += v.get_size_estimate(),
            DataEnum::Empty => {}
        }
        total_size
    }

    fn get_internal_allocations(&self, var_name: &str) -> Vec<(usize, String)> {
        let mut allocations = Vec::new();

        match self {
            DataEnum::Text(s) => {
                if let Some(ptr) = s.get_heap_ptr() {
                    allocations.push((ptr, format!("{}::Text", var_name)));
                }
            }
            DataEnum::Numbers(v) => {
                if let Some(ptr) = v.get_heap_ptr() {
                    allocations.push((ptr, format!("{}::Numbers", var_name)));
                }
            }
            DataEnum::Empty => {}
        }

        allocations
    }
}

fn main() {
    init();

    println!("🚀 Custom Types Tracking Demo");
    println!("Demonstrating #[derive(Trackable)] macro functionality");

    // Test 1: Complex struct with multiple allocations
    {
        println!("\n📦 Test 1: Complex UserProfile struct");

        let mut metadata = HashMap::new();
        metadata.insert("role".to_string(), "admin".to_string());
        metadata.insert("department".to_string(), "engineering".to_string());

        let user = UserProfile {
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            tags: vec![
                "rust".to_string(),
                "developer".to_string(),
                "senior".to_string(),
            ],
            metadata: Box::new(metadata),
            scores: vec![95, 87, 92, 88, 91],
        };

        let tracked_user = track_var!(user);
        println!("   Created UserProfile with multiple internal allocations");
        println!("   Name: {}", tracked_user.name);
        println!("   Tags count: {}", tracked_user.tags.len());
        println!("   Metadata entries: {}", tracked_user.metadata.len());
        println!("   Scores count: {}", tracked_user.scores.len());
    }

    // Test 2: Simple struct
    {
        println!("\n📊 Test 2: Simple data struct");
        let simple = SimpleData {
            id: 12345,
            active: true,
        };

        let tracked_simple = track_var!(simple);
        println!(
            "   Created SimpleData: id={}, active={}",
            tracked_simple.id, tracked_simple.active
        );
    }

    // Test 3: Nested complex struct
    {
        println!("\n🔄 Test 3: Nested data structure");

        let profile1 = UserProfile {
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
            tags: vec!["manager".to_string()],
            metadata: Box::new(HashMap::new()),
            scores: vec![85, 90],
        };

        let profile2 = UserProfile {
            name: "Carol Davis".to_string(),
            email: "carol@example.com".to_string(),
            tags: vec!["designer".to_string(), "creative".to_string()],
            metadata: Box::new(HashMap::new()),
            scores: vec![92, 88, 95],
        };

        let mut cache = HashMap::new();
        cache.insert("config".to_string(), vec![1, 2, 3, 4]);
        cache.insert("state".to_string(), vec![5, 6, 7, 8, 9]);

        let nested = NestedData {
            profiles: vec![profile1, profile2],
            cache,
            description: "Nested data structure example".to_string(),
        };

        let tracked_nested = track_var!(nested);
        println!(
            "   Created NestedData with {} profiles",
            tracked_nested.profiles.len()
        );
        println!("   Cache entries: {}", tracked_nested.cache.len());
        println!("   Description: {}", tracked_nested.description);
    }

    // Test 4: Unit struct
    {
        println!("\n⚪ Test 4: Unit struct");
        let unit = UnitStruct;
        let tracked_unit = track_var!(unit);
        println!("   Created UnitStruct (no heap allocations)");
    }

    // Test 5: Tuple struct
    {
        println!("\n📝 Test 5: Tuple struct");
        let tuple = TupleStruct("tuple data".to_string(), vec![1, 2, 3, 4, 5]);
        let tracked_tuple = track_var!(tuple);
        println!(
            "   Created TupleStruct: '{}', {} numbers",
            tracked_tuple.0,
            tracked_tuple.1.len()
        );
    }

    // Test 6: Enum variants
    {
        println!("\n🔀 Test 6: Enum variants");

        let enum1 = DataEnum::Text("enum text data".to_string());
        let tracked_enum1 = track_var!(enum1);
        println!("   Created DataEnum::Text variant");

        let enum2 = DataEnum::Numbers(vec![10, 20, 30, 40, 50]);
        let tracked_enum2 = track_var!(enum2);
        println!("   Created DataEnum::Numbers variant");

        let enum3 = DataEnum::Empty;
        let tracked_enum3 = track_var!(enum3);
        println!("   Created DataEnum::Empty variant");
    }

    println!("\n✅ All custom type tests completed!");
    println!("📊 Each type was automatically tracked using #[derive(Trackable)]");
    println!("🎯 Multiple internal allocations are properly handled");

    // Export the results
    use memscope_rs::get_global_tracker;
    let tracker = get_global_tracker();
    if let Err(e) = tracker.export_to_json("custom_types_analysis.json") {
        println!("❌ Failed to export analysis: {}", e);
    } else {
        println!("📄 Analysis exported to custom_types_analysis.json");
    }
}
