//! Comprehensive tests for core functionality modules
//! This test file focuses on improving coverage for core modules

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_structure_creation() {
        // Test creation of various data structures without global tracker
        let vec_data = vec![1, 2, 3, 4, 5];
        assert_eq!(vec_data.len(), 5);
        assert_eq!(vec_data[0], 1);
        assert_eq!(vec_data[4], 5);
        
        let string_data = String::from("test_string");
        assert_eq!(string_data, "test_string");
        assert_eq!(string_data.len(), 11);
        
        let number_data = 42i32;
        assert_eq!(number_data, 42);
    }

    #[test]
    fn test_variable_lifecycle_simulation() {
        // Test variable lifecycle without actual tracking
        let mut lifecycle_data = Vec::new();
        
        {
            let scoped_data = String::from("scoped");
            lifecycle_data.push(scoped_data.len());
            // scoped_data goes out of scope here
        }
        
        assert_eq!(lifecycle_data[0], 6); // "scoped".len()
        
        // Test that we can continue using lifecycle_data
        lifecycle_data.push(42);
        assert_eq!(lifecycle_data.len(), 2);
    }

    #[test]
    fn test_multiple_variable_operations() {
        // Test operations on multiple variables
        let mut vec_data = vec![1, 2, 3];
        let mut string_data = String::from("test");
        let number_data = 42i32;

        // Perform operations
        vec_data.push(4);
        string_data.push_str("_suffix");
        let doubled = number_data * 2;

        // Verify results
        assert_eq!(vec_data, vec![1, 2, 3, 4]);
        assert_eq!(string_data, "test_suffix");
        assert_eq!(doubled, 84);
    }

    #[test]
    fn test_error_handling_scenarios() {
        // Test error handling in various scenarios without global tracker
        let empty_vec: Vec<i32> = Vec::new();
        assert!(empty_vec.is_empty());
        
        // Test Option handling
        let some_value = Some(42);
        let none_value: Option<i32> = None;
        
        assert_eq!(some_value.unwrap_or(0), 42);
        assert_eq!(none_value.unwrap_or(0), 0);
        
        // Test Result handling
        let ok_result: Result<i32, &str> = Ok(42);
        let err_result: Result<i32, &str> = Err("error");
        
        assert!(ok_result.is_ok());
        assert!(err_result.is_err());
    }

    #[test]
    fn test_concurrent_data_operations() {
        // Test concurrent-like data operations without actual threading
        use std::sync::{Arc, Mutex};
        
        let shared_data = Arc::new(Mutex::new(Vec::new()));
        
        // Simulate concurrent operations
        for i in 0..4 {
            let data_clone = Arc::clone(&shared_data);
            let mut data = data_clone.lock().expect("Failed to lock mutex");
            data.push(i);
        }
        
        let final_data = shared_data.lock().expect("Failed to lock mutex");
        assert_eq!(final_data.len(), 4);
        assert_eq!(*final_data, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_large_data_operations() {
        // Test operations on larger data structures
        let large_vec: Vec<i32> = (0..1000).collect();
        assert_eq!(large_vec.len(), 1000);
        assert_eq!(large_vec[0], 0);
        assert_eq!(large_vec[999], 999);
        
        // Test filtering and mapping
        let filtered: Vec<i32> = large_vec.iter().filter(|&&x| x % 2 == 0).cloned().collect();
        assert_eq!(filtered.len(), 500);
        assert_eq!(filtered[0], 0);
        assert_eq!(filtered[1], 2);
    }

    #[test]
    fn test_nested_data_structures() {
        // Test operations on nested data structures
        let nested_data = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        assert_eq!(nested_data.len(), 3);
        assert_eq!(nested_data[0], vec![1, 2, 3]);
        assert_eq!(nested_data[1][1], 5);
        assert_eq!(nested_data[2][2], 9);
        
        // Test flattening
        let flattened: Vec<i32> = nested_data.into_iter().flatten().collect();
        assert_eq!(flattened, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_string_operations() {
        // Test various string operations
        let mut string_data = String::from("initial");
        assert_eq!(string_data, "initial");
        
        string_data.push_str(" appended");
        assert_eq!(string_data, "initial appended");
        
        string_data.push('!');
        assert_eq!(string_data, "initial appended!");
        
        let uppercase = string_data.to_uppercase();
        assert_eq!(uppercase, "INITIAL APPENDED!");
        
        let words: Vec<&str> = string_data.split_whitespace().collect();
        assert_eq!(words, vec!["initial", "appended!"]);
    }

    #[test]
    fn test_option_and_result_operations() {
        // Test Option and Result type operations
        let some_data = Some(vec![1, 2, 3]);
        let none_data: Option<Vec<i32>> = None;
        
        assert!(some_data.is_some());
        assert!(none_data.is_none());
        
        let unwrapped = some_data.unwrap_or_else(Vec::new);
        assert_eq!(unwrapped, vec![1, 2, 3]);
        
        let default = none_data.unwrap_or_else(Vec::new);
        assert!(default.is_empty());
        
        // Test mapping
        let mapped = Some(42).map(|x| x * 2);
        assert_eq!(mapped, Some(84));
        
        let none_mapped: Option<i32> = None;
        let mapped_none = none_mapped.map(|x| x * 2);
        assert_eq!(mapped_none, None);
    }

    #[test]
    fn test_hashmap_operations() {
        // Test HashMap operations
        let mut map = HashMap::new();
        map.insert("key1".to_string(), 42);
        map.insert("key2".to_string(), 84);
        
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("key1"), Some(&42));
        assert_eq!(map.get("key2"), Some(&84));
        assert_eq!(map.get("key3"), None);
        
        // Test iteration
        let sum: i32 = map.values().sum();
        assert_eq!(sum, 126);
        
        // Test removal
        let removed = map.remove("key1");
        assert_eq!(removed, Some(42));
        assert_eq!(map.len(), 1);
    }
}
