//! Common utility functions shared across modules

/// Format bytes in a human-readable format
pub fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes}B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Simplify Rust type names for better readability - Enhanced Unknown Type identification
pub fn simplify_type_name(type_name: &str) -> (String, String) {
    // Handle empty or explicitly unknown types first
    if type_name.is_empty() || type_name == "Unknown" {
        return ("Unknown Type".to_string(), "Unknown".to_string());
    }
    
    // Clean up the type name - remove extra whitespace and normalize
    let clean_type = type_name.trim();
    
    // Enhanced pattern matching with more comprehensive coverage
    if clean_type.contains("Vec<") || clean_type.contains("vec::Vec") {
        let inner = extract_generic_type(clean_type, "Vec");
        (format!("Vec<{inner}>"), "Collections".to_string())
    } else if clean_type.contains("String") || clean_type.contains("string::String") {
        ("String".to_string(), "Basic Types".to_string())
    } else if clean_type.contains("Box<") || clean_type.contains("boxed::Box") {
        let inner = extract_generic_type(clean_type, "Box");
        // Check if the inner type is a collection - if so, categorize as collection
        if inner.contains("HashMap") || inner.contains("hash_map") {
            ("HashMap<K,V>".to_string(), "Collections".to_string())
        } else if inner.contains("BTreeMap") || inner.contains("btree_map") {
            ("BTreeMap<K,V>".to_string(), "Collections".to_string())
        } else if inner.contains("BTreeSet") || inner.contains("btree_set") {
            ("BTreeSet<T>".to_string(), "Collections".to_string())
        } else if inner.contains("HashSet") || inner.contains("hash_set") {
            ("HashSet<T>".to_string(), "Collections".to_string())
        } else if inner.contains("VecDeque") || inner.contains("vec_deque") {
            ("VecDeque<T>".to_string(), "Collections".to_string())
        } else if inner.contains("Vec") || inner.contains("vec::Vec") {
            let vec_inner = extract_generic_type(&inner, "Vec");
            (format!("Vec<{vec_inner}>"), "Collections".to_string())
        } else {
            (format!("Box<{inner}>"), "Smart Pointers".to_string())
        }
    } else if clean_type.contains("Rc<") || clean_type.contains("rc::Rc") {
        let inner = extract_generic_type(clean_type, "Rc");
        (format!("Rc<{inner}>"), "Smart Pointers".to_string())
    } else if clean_type.contains("Arc<") || clean_type.contains("sync::Arc") {
        let inner = extract_generic_type(clean_type, "Arc");
        (format!("Arc<{inner}>"), "Smart Pointers".to_string())
    } else if clean_type.contains("HashMap") || clean_type.contains("hash_map") {
        ("HashMap<K,V>".to_string(), "Collections".to_string())
    } else if clean_type.contains("BTreeMap") || clean_type.contains("btree_map") {
        ("BTreeMap<K,V>".to_string(), "Collections".to_string())
    } else if clean_type.contains("BTreeSet") || clean_type.contains("btree_set") {
        ("BTreeSet<T>".to_string(), "Collections".to_string())
    } else if clean_type.contains("HashSet") || clean_type.contains("hash_set") {
        ("HashSet<T>".to_string(), "Collections".to_string())
    } else if clean_type.contains("VecDeque") || clean_type.contains("vec_deque") {
        ("VecDeque<T>".to_string(), "Collections".to_string())
    } else if clean_type.contains("LinkedList") {
        ("LinkedList<T>".to_string(), "Collections".to_string())
    } else if clean_type.contains("&str") || clean_type == "str" {
        ("&str".to_string(), "Basic Types".to_string())
    } else if clean_type.contains("CString") || clean_type.contains("CStr") {
        ("CString".to_string(), "Basic Types".to_string())
    } else if clean_type.contains("OsString") || clean_type.contains("OsStr") {
        ("OsString".to_string(), "Basic Types".to_string())
    } else if clean_type.contains("PathBuf") || clean_type.contains("Path") {
        ("PathBuf".to_string(), "Basic Types".to_string())
    } else if clean_type.matches("i32").count() > 0 || clean_type.matches("u32").count() > 0 || 
              clean_type.matches("i64").count() > 0 || clean_type.matches("u64").count() > 0 || 
              clean_type.matches("f64").count() > 0 || clean_type.matches("f32").count() > 0 ||
              clean_type.matches("i8").count() > 0 || clean_type.matches("u8").count() > 0 ||
              clean_type.matches("i16").count() > 0 || clean_type.matches("u16").count() > 0 ||
              clean_type.matches("isize").count() > 0 || clean_type.matches("usize").count() > 0 ||
              clean_type.matches("bool").count() > 0 || clean_type.matches("char").count() > 0 {
        let primitive = clean_type.split("::").last().unwrap_or(clean_type);
        (primitive.to_string(), "Basic Types".to_string())
    } else if clean_type.contains("[") && clean_type.contains("]") {
        ("Array".to_string(), "Arrays".to_string())
    } else if clean_type.starts_with("(") && clean_type.ends_with(")") {
        ("Tuple".to_string(), "Tuples".to_string())
    } else if clean_type.contains("Option<") {
        ("Option<T>".to_string(), "Optionals".to_string())
    } else if clean_type.contains("Result<") {
        ("Result<T,E>".to_string(), "Results".to_string())
    } else if clean_type.contains("Mutex<") || clean_type.contains("RwLock<") {
        ("Mutex/RwLock".to_string(), "Synchronization".to_string())
    } else if clean_type.contains("Cell<") || clean_type.contains("RefCell<") {
        ("Cell/RefCell".to_string(), "Interior Mutability".to_string())
    } else if clean_type.contains("Weak<") {
        ("Weak<T>".to_string(), "Smart Pointers".to_string())
    } else if clean_type.starts_with("std::") || clean_type.starts_with("alloc::") || clean_type.starts_with("core::") {
        let simplified = clean_type.split("::").last().unwrap_or(clean_type);
        (simplified.to_string(), "Standard Library".to_string())
    } else if clean_type.contains("::") {
        // Handle custom types with namespaces
        let parts: Vec<&str> = clean_type.split("::").collect();
        if parts.len() >= 2 {
            let type_part = parts.last().unwrap();
            // Try to categorize based on common patterns
            if type_part.ends_with("Error") || type_part.contains("Err") {
                (type_part.to_string(), "Error Types".to_string())
            } else if type_part.ends_with("Config") || type_part.ends_with("Settings") {
                (type_part.to_string(), "Configuration".to_string())
            } else if type_part.ends_with("Builder") {
                (type_part.to_string(), "Builders".to_string())
            } else {
                (type_part.to_string(), "Custom Types".to_string())
            }
        } else {
            (clean_type.to_string(), "Custom Types".to_string())
        }
    } else {
        // For simple type names without namespace
        if clean_type.len() > 0 {
            (clean_type.to_string(), "Custom Types".to_string())
        } else {
            ("Unknown Type".to_string(), "Unknown".to_string())
        }
    }
}

/// Extract generic type parameter for display
pub fn extract_generic_type(type_name: &str, container: &str) -> String {
    if let Some(start) = type_name.find(&format!("{container}<")) {
        let start = start + container.len() + 1;
        if let Some(end) = type_name[start..].rfind('>') {
            let inner = &type_name[start..start + end];
            // Simplify the inner type too
            return inner.split("::").last().unwrap_or(inner).to_string();
        }
    }
    "?".to_string()
}

/// Get a simplified type name for display
pub fn get_simple_type(type_name: &str) -> String {
    if type_name.contains("String") {
        "String".to_string()
    } else if type_name.contains("Vec") {
        "Vec".to_string()
    } else if type_name.contains("Box") {
        "Box".to_string()
    } else if type_name.contains("Rc") {
        "Rc".to_string()
    } else if type_name.contains("Arc") {
        "Arc".to_string()
    } else if type_name.contains("HashMap") {
        "HashMap".to_string()
    } else {
        type_name
            .split("::")
            .last()
            .unwrap_or("Unknown")
            .to_string()
    }
}

/// Get color for category - Enhanced with new categories
pub fn get_category_color(category: &str) -> String {
    match category {
        "Collections" => "#3498db".to_string(),        // Blue
        "Basic Types" => "#27ae60".to_string(),        // Green for Basic Types
        "Strings" => "#27ae60".to_string(),            // Green (legacy support)
        "Text" => "#27ae60".to_string(),               // Green (legacy support)
        "Smart Pointers" => "#e74c3c".to_string(),     // Red
        "Reference Counted" => "#f39c12".to_string(),  // Orange
        "Thread-Safe Shared" => "#9b59b6".to_string(), // Purple
        "Primitives" => "#1abc9c".to_string(),         // Teal
        "Arrays" => "#34495e".to_string(),             // Dark Gray
        "Tuples" => "#16a085".to_string(),             // Dark Teal
        "Optionals" => "#8e44ad".to_string(),          // Dark Purple
        "Results" => "#d35400".to_string(),            // Dark Orange
        "Standard Library" => "#2980b9".to_string(),   // Dark Blue
        "Custom Types" => "#c0392b".to_string(),       // Dark Red
        "Synchronization" => "#e67e22".to_string(),    // Orange
        "Interior Mutability" => "#95a5a6".to_string(), // Light Gray
        "Error Types" => "#e74c3c".to_string(),        // Red
        "Configuration" => "#3498db".to_string(),      // Blue
        "Builders" => "#9b59b6".to_string(),           // Purple
        "Runtime/System Allocation" => "#bdc3c7".to_string(), // Light Gray for system allocations
        "Unknown" => "#bdc3c7".to_string(),            // Light Gray (legacy support)
        _ => "#7f8c8d".to_string(),                    // Medium Gray for other unknowns
    }
}

/// Get type-specific gradient colors for enhanced visualization
pub fn get_type_gradient_colors(type_name: &str) -> (&'static str, &'static str) {
    match type_name {
        "String" => ("#00BCD4", "#00ACC1"),  // Teal gradient
        "Vec" => ("#2196F3", "#1976D2"),     // Blue gradient
        "Box" => ("#F44336", "#D32F2F"),     // Red gradient
        "HashMap" => ("#4CAF50", "#388E3C"), // Green gradient
        "Rc" => ("#FF9800", "#F57C00"),      // Orange gradient
        "Arc" => ("#9C27B0", "#7B1FA2"),     // Purple gradient
        _ => ("#607D8B", "#455A64"),         // Blue-gray gradient for custom types
    }
}

/// Get color based on type for consistent visualization
pub fn get_type_color(type_name: &str) -> &'static str {
    match type_name {
        "String" => "#2ecc71",
        "Vec" => "#3498db",
        "Box" => "#e74c3c",
        "HashMap" => "#f39c12",
        "Rc" => "#9b59b6",
        "Arc" => "#1abc9c",
        _ => "#95a5a6",
    }
}

/// Enhanced type hierarchy classification for treemap visualization
#[derive(Debug, Clone)]
pub struct TypeHierarchy {
    pub major_category: String,    // Major category: Collections, Strings, Smart Pointers, etc.
    pub sub_category: String,      // Sub category: Maps, Sequences, Owned, Shared, etc.
    pub specific_type: String,     // Specific type: HashMap, Vec, Box, etc.
    pub full_type: String,         // Full type name: HashMap<String, i32>
}

/// Get comprehensive type hierarchy for treemap visualization
pub fn get_type_category_hierarchy(type_name: &str) -> TypeHierarchy {
    // Handle empty or unknown types first
    if type_name.is_empty() || type_name == "Unknown" {
        return TypeHierarchy {
            major_category: "Unknown".to_string(),
            sub_category: "Unidentified".to_string(),
            specific_type: "Unknown Type".to_string(),
            full_type: type_name.to_string(),
        };
    }

    // Collections
    if type_name.contains("HashMap") || type_name.contains("hash::map") {
        let inner = extract_generic_params(type_name, "HashMap");
        TypeHierarchy {
            major_category: "Collections".to_string(),
            sub_category: "Maps".to_string(),
            specific_type: "HashMap".to_string(),
            full_type: if inner.is_empty() { "HashMap".to_string() } else { format!("HashMap<{}>", inner) },
        }
    } else if type_name.contains("BTreeMap") || type_name.contains("btree::map") {
        let inner = extract_generic_params(type_name, "BTreeMap");
        TypeHierarchy {
            major_category: "Collections".to_string(),
            sub_category: "Maps".to_string(),
            specific_type: "BTreeMap".to_string(),
            full_type: if inner.is_empty() { "BTreeMap".to_string() } else { format!("BTreeMap<{}>", inner) },
        }
    } else if type_name.contains("HashSet") || type_name.contains("hash::set") {
        let inner = extract_generic_params(type_name, "HashSet");
        TypeHierarchy {
            major_category: "Collections".to_string(),
            sub_category: "Sets".to_string(),
            specific_type: "HashSet".to_string(),
            full_type: if inner.is_empty() { "HashSet".to_string() } else { format!("HashSet<{}>", inner) },
        }
    } else if type_name.contains("Vec") && !type_name.contains("VecDeque") {
        let inner = extract_generic_params(type_name, "Vec");
        TypeHierarchy {
            major_category: "Collections".to_string(),
            sub_category: "Sequences".to_string(),
            specific_type: "Vec".to_string(),
            full_type: if inner.is_empty() { "Vec".to_string() } else { format!("Vec<{}>", inner) },
        }
    } else if type_name.contains("VecDeque") {
        let inner = extract_generic_params(type_name, "VecDeque");
        TypeHierarchy {
            major_category: "Collections".to_string(),
            sub_category: "Sequences".to_string(),
            specific_type: "VecDeque".to_string(),
            full_type: if inner.is_empty() { "VecDeque".to_string() } else { format!("VecDeque<{}>", inner) },
        }
    }
    // Strings
    else if type_name.contains("String") && !type_name.contains("<") {
        TypeHierarchy {
            major_category: "Strings".to_string(),
            sub_category: "Owned".to_string(),
            specific_type: "String".to_string(),
            full_type: "String".to_string(),
        }
    } else if type_name.contains("&str") || (type_name.contains("str") && type_name.contains("&")) {
        TypeHierarchy {
            major_category: "Strings".to_string(),
            sub_category: "Borrowed".to_string(),
            specific_type: "&str".to_string(),
            full_type: "&str".to_string(),
        }
    }
    // Smart Pointers
    else if type_name.contains("Box<") {
        let inner = extract_generic_params(type_name, "Box");
        TypeHierarchy {
            major_category: "Smart Pointers".to_string(),
            sub_category: "Owned".to_string(),
            specific_type: "Box".to_string(),
            full_type: if inner.is_empty() { "Box".to_string() } else { format!("Box<{}>", inner) },
        }
    } else if type_name.contains("Rc<") {
        let inner = extract_generic_params(type_name, "Rc");
        TypeHierarchy {
            major_category: "Smart Pointers".to_string(),
            sub_category: "Reference Counted".to_string(),
            specific_type: "Rc".to_string(),
            full_type: if inner.is_empty() { "Rc".to_string() } else { format!("Rc<{}>", inner) },
        }
    } else if type_name.contains("Arc<") {
        let inner = extract_generic_params(type_name, "Arc");
        TypeHierarchy {
            major_category: "Smart Pointers".to_string(),
            sub_category: "Thread-Safe Shared".to_string(),
            specific_type: "Arc".to_string(),
            full_type: if inner.is_empty() { "Arc".to_string() } else { format!("Arc<{}>", inner) },
        }
    }
    // Primitives
    else if is_primitive_type(type_name) {
        let clean_type = type_name.split("::").last().unwrap_or(type_name);
        let sub_cat = if clean_type.contains("i") || clean_type.contains("u") {
            "Integers"
        } else if clean_type.contains("f") {
            "Floats"
        } else if clean_type == "bool" {
            "Boolean"
        } else {
            "Other"
        };
        TypeHierarchy {
            major_category: "Primitives".to_string(),
            sub_category: sub_cat.to_string(),
            specific_type: clean_type.to_string(),
            full_type: clean_type.to_string(),
        }
    }
    // Fallback
    else {
        let simplified = type_name.split("::").last().unwrap_or(type_name);
        TypeHierarchy {
            major_category: "Custom Types".to_string(),
            sub_category: "User Defined".to_string(),
            specific_type: simplified.to_string(),
            full_type: simplified.to_string(),
        }
    }
}

/// Extract generic parameters from type names (enhanced version)
pub fn extract_generic_params(type_name: &str, container: &str) -> String {
    if let Some(start) = type_name.find(&format!("{}<", container)) {
        let start = start + container.len() + 1;
        if let Some(end) = find_matching_bracket(type_name, start - 1) {
            let inner = &type_name[start..end];
            // Simplify the inner type
            return inner.split("::").last().unwrap_or(inner).to_string();
        }
    }
    String::new()
}

/// Find matching closing bracket for generic types
fn find_matching_bracket(s: &str, start: usize) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    if start >= chars.len() || chars[start] != '<' {
        return None;
    }
    
    let mut depth = 1;
    for i in (start + 1)..chars.len() {
        match chars[i] {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Check if a type is a primitive type
pub fn is_primitive_type(type_name: &str) -> bool {
    let clean_type = type_name.split("::").last().unwrap_or(type_name);
    matches!(clean_type, 
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
        "f32" | "f64" | "bool" | "char"
    )
}

/// Extract array information for display
pub fn extract_array_info(type_name: &str) -> String {
    if let Some(start) = type_name.find('[') {
        if let Some(end) = type_name.find(']') {
            return type_name[start..=end].to_string();
        }
    }
    "Array".to_string()
}

/// Extract standard library module name
pub fn extract_std_module(type_name: &str) -> String {
    let parts: Vec<&str> = type_name.split("::").collect();
    if parts.len() >= 2 {
        match parts[1] {
            "collections" => "Collections".to_string(),
            "sync" => "Synchronization".to_string(),
            "thread" => "Threading".to_string(),
            "fs" => "File System".to_string(),
            "net" => "Networking".to_string(),
            "io" => "Input/Output".to_string(),
            _ => "Other".to_string(),
        }
    } else {
        "Other".to_string()
    }
}
