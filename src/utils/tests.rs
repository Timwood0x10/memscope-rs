use super::*;
use std::thread;
use std::time::Duration;

#[test]
fn test_current_timestamp_nanos() {
    let timestamp1 = current_timestamp_nanos();
    thread::sleep(Duration::from_millis(1));
    let timestamp2 = current_timestamp_nanos();

    assert!(timestamp2 > timestamp1);
    assert!(timestamp1 > 0);
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(0), "0B");
    assert_eq!(format_bytes(512), "512B");
    assert_eq!(format_bytes(1024), "1.0KB");
    assert_eq!(format_bytes(1536), "1.5KB");
    assert_eq!(format_bytes(1024 * 1024), "1.0MB");
    assert_eq!(format_bytes(1024 * 1024 + 512 * 1024), "1.5MB");
    assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0GB");
    assert_eq!(format_bytes(1024 * 1024 * 1024 * 2), "2.0GB");
    assert_eq!(format_bytes(1024usize.pow(4)), "1.0TB");
    assert_eq!(format_bytes(1024usize.pow(4) * 5), "5.0TB");
    assert_eq!(format_bytes(1024usize.pow(5)), "1.0PB");
    assert_eq!(format_bytes(1024usize.pow(5) * 10), "10.0PB");
}

#[test]
fn test_simplify_type_name_basic_types() {
    let (simplified, category) = simplify_type_name("String");
    assert_eq!(simplified, "String");
    assert_eq!(category, "Basic Types");

    let (simplified, category) = simplify_type_name("&str");
    assert_eq!(simplified, "&str");
    assert_eq!(category, "Basic Types");

    let (simplified, category) = simplify_type_name("i32");
    assert_eq!(simplified, "i32");
    assert_eq!(category, "Basic Types");

    let (simplified, category) = simplify_type_name("bool");
    assert_eq!(simplified, "bool");
    assert_eq!(category, "Basic Types");
}

#[test]
fn test_simplify_type_name_collections() {
    let (simplified, category) = simplify_type_name("Vec<i32>");
    assert_eq!(simplified, "Vec<i32>");
    assert_eq!(category, "Collections");

    let (simplified, category) = simplify_type_name("HashMap<String, i32>");
    assert_eq!(simplified, "HashMap<K,V>");
    assert_eq!(category, "Collections");

    let (simplified, category) = simplify_type_name("BTreeMap<String, i32>");
    assert_eq!(simplified, "BTreeMap<K,V>");
    assert_eq!(category, "Collections");

    let (simplified, category) = simplify_type_name("HashSet<String>");
    assert_eq!(simplified, "HashSet<T>");
    assert_eq!(category, "Collections");
}

#[test]
fn test_simplify_type_name_smart_pointers() {
    let (simplified, category) = simplify_type_name("Box<i32>");
    assert_eq!(simplified, "Box<i32>");
    assert_eq!(category, "Smart Pointers");

    let (simplified, category) = simplify_type_name("Rc<String>");
    assert_eq!(simplified, "String");
    assert_eq!(category, "Basic Types");

    let (simplified, category) = simplify_type_name("Arc<Mutex<i32>>");
    assert_eq!(simplified, "Arc<Mutex<i32>>");
    assert_eq!(category, "Smart Pointers");

    let (simplified, category) = simplify_type_name("Box<HashMap<String, i32>>");
    assert_eq!(simplified, "HashMap<K,V>");
    assert_eq!(category, "Collections");
}

#[test]
fn test_simplify_type_name_special_cases() {
    let (simplified, category) = simplify_type_name("");
    assert_eq!(simplified, "Unknown Type");
    assert_eq!(category, "Unknown");

    let (simplified, category) = simplify_type_name("Unknown");
    assert_eq!(simplified, "Unknown Type");
    assert_eq!(category, "Unknown");

    let (simplified, category) = simplify_type_name("Option<i32>");
    assert_eq!(simplified, "Option<T>");
    assert_eq!(category, "Optionals");

    // String detection takes priority
    let (simplified, category) = simplify_type_name("Result<String, Error>");
    assert_eq!(simplified, "String");
    assert_eq!(category, "Basic Types");
}

#[test]
fn test_simplify_type_name_arrays_tuples() {
    // The function returns the full array notation, but category is Basic Types due to i32 detection
    let (simplified, category) = simplify_type_name("[i32; 10]");
    assert_eq!(simplified, "[i32; 10]");
    assert_eq!(category, "Basic Types");

    // String detection takes priority over tuple
    let (simplified, category) = simplify_type_name("(i32, String)");
    assert_eq!(simplified, "String");
    assert_eq!(category, "Basic Types");
}

#[test]
fn test_simplify_type_name_synchronization() {
    // Mutex detection takes priority over i32
    let (simplified, category) = simplify_type_name("Mutex<i32>");
    assert_eq!(simplified, "Mutex<i32>");
    assert_eq!(category, "Basic Types");

    // String detection takes priority
    let (simplified, category) = simplify_type_name("RwLock<String>");
    assert_eq!(simplified, "String");
    assert_eq!(category, "Basic Types");

    let (simplified, category) = simplify_type_name("Cell<i32>");
    assert_eq!(simplified, "Cell<i32>");
    assert_eq!(category, "Basic Types");
}

#[test]
fn test_simplify_type_name_namespaced() {
    // HashMap detection works for namespaced types
    let (simplified, category) = simplify_type_name("std::collections::HashMap");
    assert_eq!(simplified, "HashMap<K,V>");
    assert_eq!(category, "Collections");

    let (simplified, category) = simplify_type_name("my_crate::MyStruct");
    assert_eq!(simplified, "MyStruct");
    assert_eq!(category, "Custom Types");

    let (simplified, category) = simplify_type_name("my_crate::error::MyError");
    assert_eq!(simplified, "MyError");
    assert_eq!(category, "Error Types");

    let (simplified, category) = simplify_type_name("my_crate::config::AppConfig");
    assert_eq!(simplified, "AppConfig");
    assert_eq!(category, "Configuration");
}

#[test]
fn test_extract_generic_type() {
    assert_eq!(extract_generic_type("Vec<i32>", "Vec"), "i32");
    assert_eq!(extract_generic_type("Box<String>", "Box"), "String");
    assert_eq!(
        extract_generic_type("HashMap<String, i32>", "HashMap"),
        "String, i32"
    );
    assert_eq!(extract_generic_type("Vec", "Vec"), "?");
    assert_eq!(
        extract_generic_type("std::vec::Vec<std::string::String>", "Vec"),
        "String"
    );
}

#[test]
fn test_get_simple_type() {
    assert_eq!(get_simple_type("std::string::String"), "String");
    assert_eq!(get_simple_type("std::vec::Vec<i32>"), "Vec");
    // String detection takes priority over Box
    assert_eq!(get_simple_type("std::boxed::Box<String>"), "String");
    assert_eq!(get_simple_type("std::rc::Rc<i32>"), "Rc");
    assert_eq!(get_simple_type("std::sync::Arc<String>"), "String");
    assert_eq!(get_simple_type("std::collections::HashMap<K,V>"), "HashMap");
    assert_eq!(get_simple_type("my_crate::MyType"), "MyType");
    assert_eq!(get_simple_type("UnknownType"), "UnknownType");
}

#[test]
fn test_get_category_color() {
    assert_eq!(get_category_color("Collections"), "#3498db");
    assert_eq!(get_category_color("Basic Types"), "#27ae60");
    assert_eq!(get_category_color("Smart Pointers"), "#e74c3c");
    assert_eq!(get_category_color("Synchronization"), "#e67e22");
    assert_eq!(get_category_color("Unknown"), "#bdc3c7");
    assert_eq!(get_category_color("NonExistentCategory"), "#7f8c8d");
}

#[test]
fn test_get_type_gradient_colors() {
    let (start, end) = get_type_gradient_colors("String");
    assert_eq!(start, "#00BCD4");
    assert_eq!(end, "#00ACC1");

    let (start, end) = get_type_gradient_colors("Vec");
    assert_eq!(start, "#2196F3");
    assert_eq!(end, "#1976D2");

    let (start, end) = get_type_gradient_colors("UnknownType");
    assert_eq!(start, "#607D8B");
    assert_eq!(end, "#455A64");
}

#[test]
fn test_get_type_color() {
    assert_eq!(get_type_color("String"), "#2ecc71");
    assert_eq!(get_type_color("Vec"), "#3498db");
    assert_eq!(get_type_color("Box"), "#e74c3c");
    assert_eq!(get_type_color("HashMap"), "#f39c12");
    assert_eq!(get_type_color("Rc"), "#9b59b6");
    assert_eq!(get_type_color("Arc"), "#1abc9c");
    assert_eq!(get_type_color("UnknownType"), "#95a5a6");
}

#[test]
fn test_get_type_category_hierarchy_collections() {
    let hierarchy = get_type_category_hierarchy("HashMap<String, i32>");
    assert_eq!(hierarchy.major_category, "Collections");
    assert_eq!(hierarchy.sub_category, "Maps");
    assert_eq!(hierarchy.specific_type, "HashMap");
    assert_eq!(hierarchy.full_type, "HashMap<String, i32>");

    let hierarchy = get_type_category_hierarchy("Vec<u8>");
    assert_eq!(hierarchy.major_category, "Collections");
    assert_eq!(hierarchy.sub_category, "Sequences");
    assert_eq!(hierarchy.specific_type, "Vec");
    assert_eq!(hierarchy.full_type, "Vec<u8>");

    let hierarchy = get_type_category_hierarchy("HashSet<String>");
    assert_eq!(hierarchy.major_category, "Collections");
    assert_eq!(hierarchy.sub_category, "Sets");
    assert_eq!(hierarchy.specific_type, "HashSet");
    assert_eq!(hierarchy.full_type, "HashSet<String>");
}

#[test]
fn test_get_type_category_hierarchy_strings() {
    let hierarchy = get_type_category_hierarchy("String");
    assert_eq!(hierarchy.major_category, "Strings");
    assert_eq!(hierarchy.sub_category, "Owned");
    assert_eq!(hierarchy.specific_type, "String");
    assert_eq!(hierarchy.full_type, "String");

    let hierarchy = get_type_category_hierarchy("&str");
    assert_eq!(hierarchy.major_category, "Strings");
    assert_eq!(hierarchy.sub_category, "Borrowed");
    assert_eq!(hierarchy.specific_type, "&str");
    assert_eq!(hierarchy.full_type, "&str");
}

#[test]
fn test_get_type_category_hierarchy_smart_pointers() {
    let hierarchy = get_type_category_hierarchy("Box<i32>");
    assert_eq!(hierarchy.major_category, "Smart Pointers");
    assert_eq!(hierarchy.sub_category, "Owned");
    assert_eq!(hierarchy.specific_type, "Box");
    assert_eq!(hierarchy.full_type, "Box<i32>");

    let hierarchy = get_type_category_hierarchy("Rc<String>");
    assert_eq!(hierarchy.major_category, "Smart Pointers");
    assert_eq!(hierarchy.sub_category, "Reference Counted");
    assert_eq!(hierarchy.specific_type, "Rc");
    assert_eq!(hierarchy.full_type, "Rc<String>");

    let hierarchy = get_type_category_hierarchy("Arc<Mutex<i32>>");
    assert_eq!(hierarchy.major_category, "Smart Pointers");
    assert_eq!(hierarchy.sub_category, "Thread-Safe Shared");
    assert_eq!(hierarchy.specific_type, "Arc");
    assert_eq!(hierarchy.full_type, "Arc<Mutex<i32>>");
}

#[test]
fn test_get_type_category_hierarchy_primitives() {
    let hierarchy = get_type_category_hierarchy("i32");
    assert_eq!(hierarchy.major_category, "Primitives");
    assert_eq!(hierarchy.sub_category, "Integers");
    assert_eq!(hierarchy.specific_type, "i32");
    assert_eq!(hierarchy.full_type, "i32");

    let hierarchy = get_type_category_hierarchy("f64");
    assert_eq!(hierarchy.major_category, "Primitives");
    assert_eq!(hierarchy.sub_category, "Floats");
    assert_eq!(hierarchy.specific_type, "f64");
    assert_eq!(hierarchy.full_type, "f64");

    let hierarchy = get_type_category_hierarchy("bool");
    assert_eq!(hierarchy.major_category, "Primitives");
    assert_eq!(hierarchy.sub_category, "Boolean");
    assert_eq!(hierarchy.specific_type, "bool");
    assert_eq!(hierarchy.full_type, "bool");
}

#[test]
fn test_get_type_category_hierarchy_unknown() {
    let hierarchy = get_type_category_hierarchy("");
    assert_eq!(hierarchy.major_category, "Unknown");
    assert_eq!(hierarchy.sub_category, "Unidentified");
    assert_eq!(hierarchy.specific_type, "Unknown Type");

    let hierarchy = get_type_category_hierarchy("Unknown");
    assert_eq!(hierarchy.major_category, "Unknown");
    assert_eq!(hierarchy.sub_category, "Unidentified");
    assert_eq!(hierarchy.specific_type, "Unknown Type");
}

#[test]
fn test_extract_generic_params() {
    assert_eq!(
        extract_generic_params("HashMap<String, i32>", "HashMap"),
        "String, i32"
    );
    assert_eq!(extract_generic_params("Vec<u8>", "Vec"), "u8");
    assert_eq!(extract_generic_params("Box<String>", "Box"), "String");
    assert_eq!(extract_generic_params("Vec", "Vec"), "");
    assert_eq!(
        extract_generic_params("std::vec::Vec<std::string::String>", "Vec"),
        "String"
    );
}

#[test]
fn test_find_matching_bracket() {
    assert_eq!(find_matching_bracket("Vec<i32>", 3), Some(7));
    assert_eq!(
        find_matching_bracket("HashMap<String, Vec<i32>>", 7),
        Some(24)
    );
    assert_eq!(find_matching_bracket("Vec<i32", 3), None);
    assert_eq!(find_matching_bracket("Vec", 3), None);
}

#[test]
fn test_is_primitive_type() {
    assert!(is_primitive_type("i32"));
    assert!(is_primitive_type("u64"));
    assert!(is_primitive_type("f32"));
    assert!(is_primitive_type("bool"));
    assert!(is_primitive_type("char"));
    assert!(is_primitive_type("isize"));
    assert!(is_primitive_type("usize"));

    assert!(!is_primitive_type("String"));
    assert!(!is_primitive_type("Vec<i32>"));
    assert!(!is_primitive_type("CustomType"));

    // Test with namespace
    assert!(is_primitive_type("std::primitive::i32"));
    assert!(!is_primitive_type("std::string::String"));
}

#[test]
fn test_extract_array_info() {
    assert_eq!(extract_array_info("[i32; 10]"), "[i32; 10]");
    assert_eq!(extract_array_info("[u8; 256]"), "[u8; 256]");
    assert_eq!(extract_array_info("Vec<i32>"), "Array");
    assert_eq!(extract_array_info("no_brackets"), "Array");
}

#[test]
fn test_extract_std_module() {
    assert_eq!(
        extract_std_module("std::collections::HashMap"),
        "Collections"
    );
    assert_eq!(extract_std_module("std::sync::Mutex"), "Synchronization");
    assert_eq!(extract_std_module("std::thread::JoinHandle"), "Threading");
    assert_eq!(extract_std_module("std::fs::File"), "File System");
    assert_eq!(extract_std_module("std::net::TcpStream"), "Networking");
    assert_eq!(extract_std_module("std::io::BufReader"), "Input/Output");
    assert_eq!(extract_std_module("std::unknown::Type"), "Other");
    assert_eq!(extract_std_module("CustomType"), "Other");
}

#[test]
fn test_join_handle_ext_immediate_completion() {
    let handle = thread::spawn(|| 42);

    // Give the thread time to complete
    thread::sleep(Duration::from_millis(10));

    let result = handle.join_timeout(Duration::from_millis(100));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_join_handle_ext_timeout() {
    let handle = thread::spawn(|| {
        thread::sleep(Duration::from_millis(200));
        42
    });

    let result = handle.join_timeout(Duration::from_millis(50));
    assert!(result.is_err());
}

#[test]
fn test_join_handle_ext_within_timeout() {
    let handle = thread::spawn(|| {
        thread::sleep(Duration::from_millis(50));
        42
    });

    let result = handle.join_timeout(Duration::from_millis(200));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_type_hierarchy_custom_types() {
    let hierarchy = get_type_category_hierarchy("MyCustomType");
    assert_eq!(hierarchy.major_category, "Custom Types");
    assert_eq!(hierarchy.sub_category, "User Defined");
    assert_eq!(hierarchy.specific_type, "MyCustomType");
    assert_eq!(hierarchy.full_type, "MyCustomType");
}

#[test]
fn test_simplify_type_name_edge_cases() {
    // Test whitespace handling
    let (simplified, category) = simplify_type_name("  String  ");
    assert_eq!(simplified, "String");
    assert_eq!(category, "Basic Types");

    // Test complex nested types - correctly handles nested brackets
    let (simplified, category) = simplify_type_name("Box<Vec<HashMap<String, i32>>>");
    assert_eq!(simplified, "Vec<HashMap<String, i32>>");
    assert_eq!(category, "Collections");

    // Test weak pointers
    let (simplified, category) = simplify_type_name("Weak<String>");
    assert_eq!(simplified, "String");
    assert_eq!(category, "Basic Types");
}

#[test]
fn test_thread_id_to_u64() {
    let id1 = thread::current().id();
    let id2 = thread::current().id();
    assert_eq!(thread_id_to_u64(id1), thread_id_to_u64(id2));

    let handle = thread::spawn(|| thread_id_to_u64(thread::current().id()));
    let other_id = handle.join().unwrap();
    assert_ne!(thread_id_to_u64(thread::current().id()), other_id);
}

#[test]
fn test_current_thread_id_u64() {
    let id1 = current_thread_id_u64();
    let id2 = current_thread_id_u64();
    assert_eq!(id1, id2);
}
