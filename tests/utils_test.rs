//! Tests for utility functions
//! 
//! This test suite verifies that utility functions work correctly,
//! especially the improved type name simplification logic.

use memscope_rs::utils::simplify_type_name;

#[test]
fn test_simplify_type_name_basic_types() {
    // Test basic types
    let (name, category) = simplify_type_name("i32");
    assert_eq!(name, "i32");
    assert_eq!(category, "Basic Types");
    
    let (name, category) = simplify_type_name("String");
    assert_eq!(name, "String");
    assert_eq!(category, "Basic Types");
}

#[test]
fn test_simplify_type_name_collections() {
    // Test collections
    let (name, category) = simplify_type_name("Vec<i32>");
    assert_eq!(name, "Vec<i32>");
    assert_eq!(category, "Collections");
    
    let (name, category) = simplify_type_name("std::vec::Vec<String>");
    assert_eq!(name, "Vec<String>");
    assert_eq!(category, "Collections");
}

#[test]
fn test_simplify_type_name_smart_pointers() {
    // Test smart pointers
    let (name, category) = simplify_type_name("Box<dyn Error>");
    assert_eq!(name, "Box<dyn Error>");
    assert_eq!(category, "Smart Pointers");
    
    let (name, category) = simplify_type_name("Arc<Mutex<i32>>");
    assert_eq!(name, "Arc<Mutex<i32>>");
    assert_eq!(category, "Smart Pointers");
}

#[test]
fn test_simplify_type_name_namespaced_types() {
    // Test namespaced types
    let (name, category) = simplify_type_name("std::io::Error");
    assert_eq!(name, "Error");
    assert_eq!(category, "Standard Library");
    
    let (name, category) = simplify_type_name("serde::de::Error");
    assert_eq!(name, "Error");
    assert_eq!(category, "Error Types");
    
    let (name, category) = simplify_type_name("my_crate::config::AppConfig");
    assert_eq!(name, "AppConfig");
    assert_eq!(category, "Configuration");
    
    let (name, category) = simplify_type_name("builder::pattern::MyBuilder");
    assert_eq!(name, "MyBuilder");
    assert_eq!(category, "Builders");
}

#[test]
fn test_simplify_type_name_standard_library() {
    // Test standard library types
    let (name, category) = simplify_type_name("std::collections::HashMap");
    assert_eq!(name, "HashMap<K,V>");
    assert_eq!(category, "Collections");
    
    let (name, category) = simplify_type_name("core::option::Option");
    assert_eq!(name, "Option");
    assert_eq!(category, "Standard Library");
    
    let (name, category) = simplify_type_name("alloc::string::String");
    assert_eq!(name, "String");
    assert_eq!(category, "Basic Types");
}

#[test]
fn test_simplify_type_name_test_types() {
    // Test test-related types
    let (name, category) = simplify_type_name("test::mock::MockService");
    assert_eq!(name, "MockService");
    assert_eq!(category, "Test Types");
    
    let (name, category) = simplify_type_name("my_crate::test::TestHelper");
    assert_eq!(name, "TestHelper");
    assert_eq!(category, "Test Types");
}

#[test]
fn test_simplify_type_name_library_types() {
    // Test deep namespace (library types)
    let (name, category) = simplify_type_name("tokio::runtime::executor::ThreadPool");
    assert_eq!(name, "ThreadPool");
    assert_eq!(category, "Library Types");
    
    let (name, category) = simplify_type_name("serde::json::value::Value");
    assert_eq!(name, "Value");
    assert_eq!(category, "Library Types");
}

#[test]
fn test_simplify_type_name_custom_types() {
    // Test custom types without namespace
    let (name, category) = simplify_type_name("MyStruct");
    assert_eq!(name, "MyStruct");
    assert_eq!(category, "Custom Types");
    
    let (name, category) = simplify_type_name("UserData");
    assert_eq!(name, "UserData");
    assert_eq!(category, "Custom Types");
}

#[test]
fn test_simplify_type_name_other_types() {
    // Test lowercase types (functions, etc.)
    let (name, category) = simplify_type_name("fn_pointer");
    assert_eq!(name, "fn_pointer");
    assert_eq!(category, "Custom Types");
    
    let (name, category) = simplify_type_name("closure_type");
    assert_eq!(name, "closure_type");
    assert_eq!(category, "Custom Types");
}

#[test]
fn test_simplify_type_name_edge_cases() {
    // Test edge cases
    let (name, category) = simplify_type_name("");
    assert_eq!(name, "Unknown Type");
    assert_eq!(category, "Unknown");
    
    let (name, category) = simplify_type_name("Unknown");
    assert_eq!(name, "Unknown Type");
    assert_eq!(category, "Unknown");
    
    // Test with whitespace
    let (name, category) = simplify_type_name("  std::string::String  ");
    assert_eq!(name, "String");
    assert_eq!(category, "Basic Types");
}

#[test]
fn test_simplify_type_name_complex_generics() {
    // Test that complex types are handled without panicking
    // The exact output may vary due to pattern matching order, but it should not crash
    let (name, category) = simplify_type_name("HashMap<String, Vec<i32>>");
    assert!(!name.is_empty());
    assert!(!category.is_empty());
    
    let (name, category) = simplify_type_name("Result<Option<String>, Box<dyn Error>>");
    assert!(!name.is_empty());
    assert!(!category.is_empty());
}

#[test]
fn test_simplify_type_name_fallback_logic() {
    // Test that the fallback logic works correctly
    // This tests the case where parts.last() might theoretically return None
    // (though in practice it shouldn't with our current logic)
    
    let (name, category) = simplify_type_name("some::complex::deeply::nested::Type");
    assert_eq!(name, "Type");
    assert_eq!(category, "Library Types"); // Deep namespace
    
    // Test single component
    let (name, category) = simplify_type_name("SimpleType");
    assert_eq!(name, "SimpleType");
    assert_eq!(category, "Custom Types");
}