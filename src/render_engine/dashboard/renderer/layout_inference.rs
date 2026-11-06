//! Layout inference used by the dashboard report builder.
//!
//! The inference is name/size based and therefore heuristic. It is kept in its
//! own module so the report builder stays within the project line budget and so
//! the heuristics can be replaced by real type observations (plan §6.3) without
//! touching the DTO mapping code.

/// Infer a `TypeLayoutSnapshot` from allocation attributes at report-build time.
///
/// Every field produced here is an inference from the type name and requested
/// size, so callers must not present it as an observed layout.
pub(super) fn infer_layout_snapshot(
    type_name: &str,
    size: usize,
    generation_id: usize,
) -> Option<crate::capture::types::TypeLayoutSnapshot> {
    let gen = generation_id;

    // Try Vec<T> — pattern like "Vec<i32>" or "std::vec::Vec<u8>"
    if let Some(inner) = type_name
        .rsplit("Vec<")
        .nth(1)
        .and_then(|_| type_name.split("Vec<").nth(1))
        .and_then(|s| s.strip_suffix('>'))
        .map(|s| s.trim())
    {
        let elem_size = estimate_type_size(inner);
        let cap = size.checked_div(elem_size).unwrap_or(0);
        return Some(crate::capture::types::TypeLayoutSnapshot {
            type_name: format!("Vec<{}>", inner),
            size_of_t: std::mem::size_of::<*const usize>() * 3,
            align_of_t: std::mem::align_of::<usize>(),
            is_sized: true,
            repr_hint: crate::capture::types::ReprHint::Rust,
            layout_kind: crate::capture::types::LayoutKind::Container,
            pointer_width: crate::capture::types::PointerWidth::Unknown,
            logical_size_bytes: std::mem::size_of::<*const usize>() * 3,
            allocated_size_bytes: size,
            used_size_bytes: 0,
            reserved_size_bytes: size,
            element_size: Some(elem_size),
            element_align: Some(elem_size),
            container_len: Some(0),
            container_capacity: Some(cap),
            strong_count: None,
            weak_count: None,
            pointee_type: Some(inner.to_string()),
            generation_id: gen,
        });
    }

    // String / std::string::String
    if type_name == "String" || type_name.contains("std::string::String") {
        return Some(crate::capture::types::TypeLayoutSnapshot {
            type_name: "String".to_string(),
            size_of_t: std::mem::size_of::<*const usize>() * 3,
            align_of_t: std::mem::align_of::<usize>(),
            is_sized: true,
            repr_hint: crate::capture::types::ReprHint::Rust,
            layout_kind: crate::capture::types::LayoutKind::Container,
            pointer_width: crate::capture::types::PointerWidth::Unknown,
            logical_size_bytes: std::mem::size_of::<*const usize>() * 3,
            allocated_size_bytes: size,
            used_size_bytes: 0,
            reserved_size_bytes: size,
            element_size: Some(1),
            element_align: Some(1),
            container_len: Some(0),
            container_capacity: Some(size),
            strong_count: None,
            weak_count: None,
            pointee_type: Some("u8".to_string()),
            generation_id: gen,
        });
    }

    // Box<T>
    if let Some(inner) = type_name
        .rsplit("Box<")
        .nth(1)
        .and_then(|_| type_name.split("Box<").nth(1))
        .and_then(|s| s.strip_suffix('>'))
        .map(|s| s.trim())
    {
        let pointee_size = estimate_type_size(inner);
        return Some(crate::capture::types::TypeLayoutSnapshot {
            type_name: format!("Box<{}>", inner),
            size_of_t: std::mem::size_of::<usize>(),
            align_of_t: std::mem::align_of::<usize>(),
            is_sized: true,
            repr_hint: crate::capture::types::ReprHint::Rust,
            layout_kind: crate::capture::types::LayoutKind::SmartPointer,
            pointer_width: crate::capture::types::PointerWidth::Thin,
            logical_size_bytes: std::mem::size_of::<usize>(),
            allocated_size_bytes: pointee_size,
            used_size_bytes: pointee_size,
            reserved_size_bytes: pointee_size,
            element_size: Some(pointee_size),
            element_align: Some(pointee_size),
            container_len: None,
            container_capacity: None,
            strong_count: None,
            weak_count: None,
            pointee_type: Some(inner.to_string()),
            generation_id: gen,
        });
    }

    // Arc<T>
    if let Some(inner) = type_name
        .rsplit("Arc<")
        .nth(1)
        .and_then(|_| type_name.split("Arc<").nth(1))
        .and_then(|s| s.strip_suffix('>'))
        .map(|s| s.trim())
    {
        let pointee_size = estimate_type_size(inner);
        return Some(crate::capture::types::TypeLayoutSnapshot {
            type_name: format!("Arc<{}>", inner),
            size_of_t: std::mem::size_of::<usize>() * 2,
            align_of_t: std::mem::align_of::<usize>(),
            is_sized: true,
            repr_hint: crate::capture::types::ReprHint::Rust,
            layout_kind: crate::capture::types::LayoutKind::SmartPointer,
            pointer_width: crate::capture::types::PointerWidth::Thin,
            logical_size_bytes: std::mem::size_of::<usize>() * 2,
            allocated_size_bytes: pointee_size + std::mem::size_of::<usize>() * 2,
            used_size_bytes: pointee_size,
            reserved_size_bytes: pointee_size + std::mem::size_of::<usize>() * 2,
            element_size: Some(pointee_size),
            element_align: Some(pointee_size),
            container_len: None,
            container_capacity: None,
            strong_count: Some(1),
            weak_count: Some(0),
            pointee_type: Some(inner.to_string()),
            generation_id: gen,
        });
    }

    // Rc<T>
    if let Some(inner) = type_name
        .rsplit("Rc<")
        .nth(1)
        .and_then(|_| type_name.split("Rc<").nth(1))
        .and_then(|s| s.strip_suffix('>'))
        .map(|s| s.trim())
    {
        let pointee_size = estimate_type_size(inner);
        return Some(crate::capture::types::TypeLayoutSnapshot {
            type_name: format!("Rc<{}>", inner),
            size_of_t: std::mem::size_of::<usize>() * 2,
            align_of_t: std::mem::align_of::<usize>(),
            is_sized: true,
            repr_hint: crate::capture::types::ReprHint::Rust,
            layout_kind: crate::capture::types::LayoutKind::SmartPointer,
            pointer_width: crate::capture::types::PointerWidth::Thin,
            logical_size_bytes: std::mem::size_of::<usize>() * 2,
            allocated_size_bytes: pointee_size + std::mem::size_of::<usize>() * 2,
            used_size_bytes: pointee_size,
            reserved_size_bytes: pointee_size + std::mem::size_of::<usize>() * 2,
            element_size: Some(pointee_size),
            element_align: Some(pointee_size),
            container_len: None,
            container_capacity: None,
            strong_count: Some(1),
            weak_count: Some(0),
            pointee_type: Some(inner.to_string()),
            generation_id: gen,
        });
    }

    // Fallback: generic snapshot from type name and size
    let layout_kind = crate::capture::types::infer_layout_kind_from_name(type_name);
    Some(crate::capture::types::TypeLayoutSnapshot {
        type_name: type_name.to_string(),
        size_of_t: size,
        align_of_t: 0,
        is_sized: true,
        repr_hint: crate::capture::types::ReprHint::Unknown,
        layout_kind,
        pointer_width: crate::capture::types::PointerWidth::Unknown,
        logical_size_bytes: size,
        allocated_size_bytes: size,
        used_size_bytes: size,
        reserved_size_bytes: size,
        element_size: None,
        element_align: None,
        container_len: None,
        container_capacity: None,
        strong_count: None,
        weak_count: None,
        pointee_type: None,
        generation_id: gen,
    })
}

/// Estimate the size of a Rust type from its name string.
fn estimate_type_size(type_name: &str) -> usize {
    match type_name.trim() {
        "u8" | "i8" | "bool" => 1,
        "u16" | "i16" | "char" => 2,
        "u32" | "i32" | "f32" => 4,
        "u64" | "i64" | "f64" | "usize" | "isize" => 8,
        "u128" | "i128" => 16,
        "()" => 0,
        "*mut u8" | "*const u8" | "*mut c_void" | "*const c_void" | "String" => 8,
        s if s.contains("String") => 24, // String struct on stack
        s if s.starts_with('&') => 16,   // fat reference (ptr + len)
        s if s.contains('*') => 8,       // pointer
        _ => 8,                          // default fallback
    }
}
