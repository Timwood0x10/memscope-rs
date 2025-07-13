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
    if type_name.starts_with("alloc::vec::Vec<") || type_name.starts_with("std::vec::Vec<") {
        let inner = extract_generic_type(type_name, "Vec");
        (format!("Vec<{inner}>"), "Collections".to_string())
    } else if type_name.starts_with("alloc::string::String") || type_name == "String" {
        ("String".to_string(), "Text".to_string())
    } else if type_name.starts_with("alloc::boxed::Box<")
        || type_name.starts_with("std::boxed::Box<")
    {
        let inner = extract_generic_type(type_name, "Box");
        (format!("Box<{inner}>"), "Smart Pointers".to_string())
    } else if type_name.starts_with("alloc::rc::Rc<") || type_name.starts_with("std::rc::Rc<") {
        let inner = extract_generic_type(type_name, "Rc");
        (format!("Rc<{inner}>"), "Reference Counted".to_string())
    } else if type_name.starts_with("alloc::sync::Arc<") || type_name.starts_with("std::sync::Arc<")
    {
        let inner = extract_generic_type(type_name, "Arc");
        (format!("Arc<{inner}>"), "Thread-Safe Shared".to_string())
    } else if type_name.contains("HashMap") {
        ("HashMap".to_string(), "Collections".to_string())
    } else if type_name.contains("BTreeMap") {
        ("BTreeMap".to_string(), "Collections".to_string())
    } else if type_name.contains("VecDeque") {
        ("VecDeque".to_string(), "Collections".to_string())
    } else if type_name.is_empty() || type_name == "Unknown" {
        ("Unknown Type".to_string(), "Unknown".to_string())
    } else if type_name.contains("i32") || type_name.contains("u32") || type_name.contains("i64") || type_name.contains("u64") || type_name.contains("f64") || type_name.contains("f32") {
        (type_name.split("::").last().unwrap_or(type_name).to_string(), "Primitives".to_string())
    } else if type_name.contains("[") && type_name.contains("]") {
        ("Array".to_string(), "Arrays".to_string())
    } else if type_name.starts_with("(") && type_name.ends_with(")") {
        ("Tuple".to_string(), "Tuples".to_string())
    } else if type_name.contains("Option<") {
        ("Option".to_string(), "Optionals".to_string())
    } else if type_name.contains("Result<") {
        ("Result".to_string(), "Results".to_string())
    } else if type_name.starts_with("std::") || type_name.starts_with("alloc::") {
        let simplified = type_name.split("::").last().unwrap_or(type_name);
        (format!("std::{}", simplified), "Standard Library".to_string())
    } else if type_name.contains("::") {
        let parts: Vec<&str> = type_name.split("::").collect();
        if parts.len() >= 2 {
            let module = parts[parts.len() - 2];
            let type_part = parts[parts.len() - 1];
            (format!("{}::{}", module, type_part), "Custom Types".to_string())
        } else {
            (parts.last().map_or(type_name, |v| v).to_string(), "Custom Types".to_string())
        }
    } else {
        // For other types, try to extract the last component
        let simplified = type_name
            .split("::")
            .last()
            .unwrap_or(type_name)
            .to_string();
        (simplified, "Custom Types".to_string())
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
        "Text" => "#2ecc71".to_string(),               // Green  
        "Smart Pointers" => "#e74c3c".to_string(),     // Red
        "Reference Counted" => "#f39c12".to_string(),  // Orange
        "Thread-Safe Shared" => "#9b59b6".to_string(), // Purple
        "Primitives" => "#1abc9c".to_string(),         // Teal
        "Arrays" => "#34495e".to_string(),             // Dark Gray
        "Tuples" => "#16a085".to_string(),             // Dark Teal
        "Optionals" => "#27ae60".to_string(),          // Dark Green
        "Results" => "#8e44ad".to_string(),            // Dark Purple
        "Standard Library" => "#2980b9".to_string(),   // Dark Blue
        "Custom Types" => "#d35400".to_string(),       // Dark Orange
        "Unknown" => "#95a5a6".to_string(),            // Gray
        _ => "#7f8c8d".to_string(),                    // Darker Gray
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
