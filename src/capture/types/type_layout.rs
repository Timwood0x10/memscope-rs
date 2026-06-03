//! Type layout snapshot for tracking allocation metadata.
//!
//! Provides `TypeLayoutSnapshot` and related types for recording
//! type-level layout information (size, alignment, representation,
//! container utilization) alongside allocation events.

use serde::{Deserialize, Serialize};
use std::any::type_name;

/// Representation hint for a Rust type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ReprHint {
    /// Default Rust layout (unspecified field order)
    Rust,
    /// C-compatible layout
    C,
    /// #[repr(transparent)]
    Transparent,
    /// #[repr(packed)]
    Packed,
    /// #[repr(align(N))]
    Align(u64),
    /// Representation unknown
    #[default]
    Unknown,
}

/// High-level layout category for a type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum LayoutKind {
    /// Primitive type (i32, f64, bool, etc.)
    Primitive,
    /// Struct type
    Struct,
    /// Enum type
    Enum,
    /// Union type
    Union,
    /// Tuple type
    Tuple,
    /// Fixed-size array [T; N]
    Array,
    /// Dynamically-sized slice [T]
    Slice,
    /// String slice str
    Str,
    /// Trait object dyn Trait
    TraitObject,
    /// Raw or safe pointer
    Pointer,
    /// Smart pointer (Box, Rc, Arc, etc.)
    SmartPointer,
    /// Container (Vec, HashMap, String, etc.)
    Container,
    /// Zero-sized type
    Zst,
    /// Layout kind unknown
    #[default]
    Unknown,
}

/// Pointer width category (thin vs fat pointers).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum PointerWidth {
    /// Thin pointer (single usize)
    Thin,
    /// Fat slice pointer (ptr + length)
    FatSlice,
    /// Fat trait-object pointer (ptr + vtable)
    FatTraitObject,
    /// Function pointer
    Function,
    /// Pointer width unknown
    #[default]
    Unknown,
}

/// A snapshot of type-level layout metadata for an allocation.
///
/// Captures everything from simple `size_of::<T>()` / `align_of::<T>()`
/// up to container utilization, smart-pointer counts, and repr hints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeLayoutSnapshot {
    /// Fully-qualified type name
    pub type_name: String,
    /// `size_of::<T>()` in bytes
    pub size_of_t: usize,
    /// `align_of::<T>()` in bytes
    pub align_of_t: usize,
    /// Whether the type is `Sized`
    pub is_sized: bool,
    /// Representation hint
    #[serde(default)]
    pub repr_hint: ReprHint,
    /// High-level layout category
    #[serde(default)]
    pub layout_kind: LayoutKind,
    /// Pointer width category (if this is a pointer type)
    #[serde(default)]
    pub pointer_width: PointerWidth,
    /// Logical size of the value (may differ from heap allocation size)
    pub logical_size_bytes: usize,
    /// Actual heap allocation size
    pub allocated_size_bytes: usize,
    /// Used bytes (for containers: length * element_size)
    pub used_size_bytes: usize,
    /// Reserved bytes (for containers: capacity * element_size)
    pub reserved_size_bytes: usize,
    /// Container element size if applicable
    #[serde(default)]
    pub element_size: Option<usize>,
    /// Container element alignment if applicable
    #[serde(default)]
    pub element_align: Option<usize>,
    /// Container length (Vec/String len)
    #[serde(default)]
    pub container_len: Option<usize>,
    /// Container capacity (Vec/String cap)
    #[serde(default)]
    pub container_capacity: Option<usize>,
    /// Smart-pointer strong count (Rc/Arc)
    #[serde(default)]
    pub strong_count: Option<usize>,
    /// Smart-pointer weak count (Rc/Arc)
    #[serde(default)]
    pub weak_count: Option<usize>,
    /// Smart-pointer pointee type name if known
    #[serde(default)]
    pub pointee_type: Option<String>,
    /// Allocation generation id this snapshot is linked to
    #[serde(default)]
    pub generation_id: usize,
}

impl TypeLayoutSnapshot {
    /// Create a new layout snapshot for a sized type `T`.
    ///
    /// Records `size_of::<T>()`, `align_of::<T>()`, `type_name::<T>()`,
    /// and sets `logical_size_bytes` and `allocated_size_bytes` to `size_of::<T>()`.
    pub fn of<T: 'static>() -> Self {
        let s = std::mem::size_of::<T>();
        Self {
            type_name: type_name::<T>().to_string(),
            size_of_t: s,
            align_of_t: std::mem::align_of::<T>(),
            is_sized: true,
            repr_hint: ReprHint::Unknown,
            layout_kind: infer_layout_kind::<T>(),
            pointer_width: PointerWidth::Unknown,
            logical_size_bytes: s,
            allocated_size_bytes: s,
            used_size_bytes: s,
            reserved_size_bytes: s,
            element_size: None,
            element_align: None,
            container_len: None,
            container_capacity: None,
            strong_count: None,
            weak_count: None,
            pointee_type: None,
            generation_id: 0,
        }
    }

    /// Create a snapshot for a `Vec<T>` allocation.
    pub fn vec<T: 'static>(vec: &Vec<T>) -> Self {
        let elem_size = std::mem::size_of::<T>();
        let len = vec.len();
        let cap = vec.capacity();
        let used = len * elem_size;
        let reserved = cap * elem_size;
        Self {
            type_name: format!("Vec<{}>", type_name::<T>()),
            size_of_t: std::mem::size_of::<Vec<T>>(),
            align_of_t: std::mem::align_of::<Vec<T>>(),
            is_sized: true,
            repr_hint: ReprHint::Rust,
            layout_kind: LayoutKind::Container,
            pointer_width: PointerWidth::Unknown,
            logical_size_bytes: std::mem::size_of::<Vec<T>>(),
            allocated_size_bytes: reserved,
            used_size_bytes: used,
            reserved_size_bytes: reserved,
            element_size: Some(elem_size),
            element_align: Some(std::mem::align_of::<T>()),
            container_len: Some(len),
            container_capacity: Some(cap),
            strong_count: None,
            weak_count: None,
            pointee_type: Some(type_name::<T>().to_string()),
            generation_id: 0,
        }
    }

    /// Create a snapshot for a `String` allocation.
    pub fn string(s: &String) -> Self {
        let cap = s.capacity();
        let len = s.len();
        Self {
            type_name: "String".to_string(),
            size_of_t: std::mem::size_of::<String>(),
            align_of_t: std::mem::align_of::<String>(),
            is_sized: true,
            repr_hint: ReprHint::Rust,
            layout_kind: LayoutKind::Container,
            pointer_width: PointerWidth::Unknown,
            logical_size_bytes: std::mem::size_of::<String>(),
            allocated_size_bytes: cap,
            used_size_bytes: len,
            reserved_size_bytes: cap,
            element_size: Some(1),
            element_align: Some(1),
            container_len: Some(len),
            container_capacity: Some(cap),
            strong_count: None,
            weak_count: None,
            pointee_type: Some("u8".to_string()),
            generation_id: 0,
        }
    }

    /// Create a snapshot for a `Box<T>` allocation.
    pub fn boxed<T: 'static>(_inner: &T) -> Self {
        let pointee_size = std::mem::size_of::<T>();
        Self {
            type_name: format!("Box<{}>", type_name::<T>()),
            size_of_t: std::mem::size_of::<Box<T>>(),
            align_of_t: std::mem::align_of::<Box<T>>(),
            is_sized: true,
            repr_hint: ReprHint::Rust,
            layout_kind: LayoutKind::SmartPointer,
            pointer_width: PointerWidth::Thin,
            logical_size_bytes: std::mem::size_of::<Box<T>>(),
            allocated_size_bytes: pointee_size,
            used_size_bytes: pointee_size,
            reserved_size_bytes: pointee_size,
            element_size: Some(pointee_size),
            element_align: Some(std::mem::align_of::<T>()),
            container_len: None,
            container_capacity: None,
            strong_count: None,
            weak_count: None,
            pointee_type: Some(type_name::<T>().to_string()),
            generation_id: 0,
        }
    }

    /// Create a snapshot for an `Rc<T>` allocation (strong/weak counts provided).
    pub fn rc<T: 'static>(strong: usize, weak: usize) -> Self {
        let pointee_size = std::mem::size_of::<T>();
        Self {
            type_name: format!("Rc<{}>", type_name::<T>()),
            size_of_t: std::mem::size_of::<usize>() * 2,
            align_of_t: std::mem::align_of::<usize>(),
            is_sized: true,
            repr_hint: ReprHint::Rust,
            layout_kind: LayoutKind::SmartPointer,
            pointer_width: PointerWidth::Thin,
            logical_size_bytes: std::mem::size_of::<usize>() * 2,
            allocated_size_bytes: pointee_size + std::mem::size_of::<usize>() * 2,
            used_size_bytes: pointee_size,
            reserved_size_bytes: pointee_size + std::mem::size_of::<usize>() * 2,
            element_size: Some(pointee_size),
            element_align: Some(std::mem::align_of::<T>()),
            container_len: None,
            container_capacity: None,
            strong_count: Some(strong),
            weak_count: Some(weak),
            pointee_type: Some(type_name::<T>().to_string()),
            generation_id: 0,
        }
    }

    /// Create a snapshot for an `Arc<T>` allocation (strong/weak counts provided).
    pub fn arc<T: 'static>(strong: usize, weak: usize) -> Self {
        let mut snap = Self::rc::<T>(strong, weak);
        snap.type_name = format!("Arc<{}>", type_name::<T>());
        snap
    }

    /// Create a snapshot for a slice `&[T]`.
    pub fn slice<T: 'static>(slice: &[T]) -> Self {
        let elem_size = std::mem::size_of::<T>();
        let len = slice.len();
        Self {
            type_name: format!("[{}]", type_name::<T>()),
            size_of_t: 0,
            align_of_t: std::mem::align_of::<T>(),
            is_sized: false,
            repr_hint: ReprHint::Rust,
            layout_kind: LayoutKind::Slice,
            pointer_width: PointerWidth::FatSlice,
            logical_size_bytes: std::mem::size_of_val(slice),
            allocated_size_bytes: std::mem::size_of_val(slice),
            used_size_bytes: std::mem::size_of_val(slice),
            reserved_size_bytes: std::mem::size_of_val(slice),
            element_size: Some(elem_size),
            element_align: Some(std::mem::align_of::<T>()),
            container_len: Some(len),
            container_capacity: None,
            strong_count: None,
            weak_count: None,
            pointee_type: Some(type_name::<T>().to_string()),
            generation_id: 0,
        }
    }

    /// Set the generation id for this snapshot.
    pub fn with_generation(mut self, gen: usize) -> Self {
        self.generation_id = gen;
        self
    }

    /// Set the repr hint for this snapshot.
    pub fn with_repr(mut self, hint: ReprHint) -> Self {
        self.repr_hint = hint;
        self
    }
}

/// Infer a `LayoutKind` from a type `T` based on its `type_name`.
pub fn infer_layout_kind<T: 'static>() -> LayoutKind {
    let name = type_name::<T>().to_lowercase();
    let kind = infer_layout_kind_from_name(&name);
    if kind == LayoutKind::Struct && std::mem::size_of::<T>() == 0 {
        return LayoutKind::Zst;
    }
    kind
}

/// Infer a `LayoutKind` from a type name string (no generic parameter needed).
pub fn infer_layout_kind_from_name(name: &str) -> LayoutKind {
    let name = name.to_lowercase();
    if name == "bool"
        || name == "char"
        || name.starts_with('i')
        || name.starts_with('u')
        || name.starts_with('f')
        || name == "usize"
        || name == "isize"
    {
        return LayoutKind::Primitive;
    }
    if name.contains("string") {
        return LayoutKind::Container;
    }
    if name.contains("vec<") {
        return LayoutKind::Container;
    }
    if name.contains("hashmap") || name.contains("hashset") {
        return LayoutKind::Container;
    }
    if name.contains("btreemap") || name.contains("btreeset") {
        return LayoutKind::Container;
    }
    if name.contains("linkedlist") || name.contains("vecdeque") {
        return LayoutKind::Container;
    }
    if name.starts_with('&') || name.contains('*') {
        return LayoutKind::Pointer;
    }
    if name.contains("box<") || name.contains("rc<") || name.contains("arc<") {
        return LayoutKind::SmartPointer;
    }
    if name.contains("mutex") || name.contains("rwlock") || name.contains("refcell") {
        return LayoutKind::SmartPointer;
    }
    if name.contains("fn(") || name.contains("fn ") || name.contains("closure") {
        return LayoutKind::Pointer;
    }
    if name.starts_with('[') && name.contains(";") {
        return LayoutKind::Array;
    }
    if name.starts_with('[') {
        return LayoutKind::Slice;
    }
    if name.starts_with('(') {
        return LayoutKind::Tuple;
    }
    if name.contains("union") {
        return LayoutKind::Union;
    }
    if name.contains("enum") {
        return LayoutKind::Enum;
    }
    LayoutKind::Struct
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_snapshot() {
        let snap = TypeLayoutSnapshot::of::<i32>();
        assert_eq!(snap.size_of_t, 4);
        assert_eq!(snap.align_of_t, 4);
        assert!(snap.type_name.contains("i32"));
        assert_eq!(snap.layout_kind, LayoutKind::Primitive);
    }

    #[test]
    fn test_string_snapshot() {
        let s = String::from("hello");
        let snap = TypeLayoutSnapshot::string(&s);
        assert_eq!(snap.container_len, Some(5));
        assert!(snap.container_capacity.unwrap() >= 5);
        assert_eq!(snap.layout_kind, LayoutKind::Container);
    }

    #[test]
    fn test_vec_snapshot() {
        let v: Vec<u64> = vec![1, 2, 3];
        let snap = TypeLayoutSnapshot::vec(&v);
        assert_eq!(snap.container_len, Some(3));
        assert_eq!(snap.element_size, Some(8));
        assert_eq!(snap.used_size_bytes, 24);
        assert_eq!(snap.layout_kind, LayoutKind::Container);
    }

    #[test]
    fn test_box_snapshot() {
        let b = Box::new(42u64);
        let snap = TypeLayoutSnapshot::boxed(&b);
        assert_eq!(snap.allocated_size_bytes, 8);
        assert_eq!(snap.layout_kind, LayoutKind::SmartPointer);
        assert!(snap.pointee_type.unwrap().contains("u64"));
    }

    #[test]
    fn test_rc_snapshot() {
        let snap = TypeLayoutSnapshot::rc::<String>(3, 1);
        assert!(snap.type_name.contains("Rc<"));
        assert_eq!(snap.strong_count, Some(3));
        assert_eq!(snap.weak_count, Some(1));
    }

    #[test]
    fn test_arc_snapshot() {
        let snap = TypeLayoutSnapshot::arc::<String>(5, 2);
        assert!(snap.type_name.contains("Arc<"));
        assert_eq!(snap.strong_count, Some(5));
        assert_eq!(snap.weak_count, Some(2));
    }

    #[test]
    fn test_slice_snapshot() {
        let data: &[u8] = &[1, 2, 3, 4, 5];
        let snap = TypeLayoutSnapshot::slice(data);
        assert_eq!(snap.container_len, Some(5));
        assert_eq!(snap.pointer_width, PointerWidth::FatSlice);
    }

    #[test]
    fn test_generation_chain() {
        let snap = TypeLayoutSnapshot::of::<i32>().with_generation(3);
        assert_eq!(snap.generation_id, 3);
    }

    #[test]
    fn test_repr_chain() {
        let snap = TypeLayoutSnapshot::of::<i32>().with_repr(ReprHint::C);
        assert_eq!(snap.repr_hint, ReprHint::C);
    }

    #[test]
    fn test_infer_layout_kind_primitive() {
        assert_eq!(infer_layout_kind::<i32>(), LayoutKind::Primitive);
        assert_eq!(infer_layout_kind::<f64>(), LayoutKind::Primitive);
        assert_eq!(infer_layout_kind::<bool>(), LayoutKind::Primitive);
    }

    #[test]
    fn test_zero_sized_type() {
        struct Zst;
        let snap = TypeLayoutSnapshot::of::<Zst>();
        assert_eq!(snap.size_of_t, 0);
        assert_eq!(snap.layout_kind, LayoutKind::Zst);
    }
}
