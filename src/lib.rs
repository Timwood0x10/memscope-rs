//! Memory tracking and visualization tools for Rust applications.
//!
//! This crate provides tools for tracking memory allocations and visualizing
//! memory usage in Rust applications. It includes a custom global allocator
//! that tracks all heap allocations and deallocations, and provides utilities
//! for exporting memory usage data in various formats.

#![warn(missing_docs)]

/// Macro for advanced type Trackable implementations
pub mod advanced_trackable_macro;
/// Advanced type analysis framework
pub mod advanced_types;
/// Advanced memory analysis functionality
pub mod analysis;
/// Command-line interface functionality
pub mod cli;
/// Core memory tracking functionality
pub mod core;
/// Export and visualization functionality
pub mod export;
/// Utility functions
pub mod utils;
/// Variable registry for lightweight HashMap-based variable tracking
pub mod variable_registry;

// Re-export key functions from unified modules
/// Enhanced types for comprehensive memory analysis
pub mod enhanced_types;
pub use advanced_types::*;
pub use analysis::*;
pub use export::*;
// Note: Macros are automatically available when the crate is imported

// Re-export main types for easier use
pub use analysis::enhanced_memory_analysis::EnhancedMemoryAnalyzer;
pub use analysis::unsafe_ffi_tracker::{get_global_unsafe_ffi_tracker, UnsafeFFITracker};
pub use core::allocator::TrackingAllocator;
pub use core::tracker::{get_global_tracker, MemoryTracker};
pub use core::types::{AllocationInfo, TrackingError, TrackingResult};
pub use export::html_export::export_interactive_html;
pub use export::visualization::{export_lifecycle_timeline, export_memory_analysis};
pub use utils::{format_bytes, get_simple_type, simplify_type_name};

// Re-export the derive macro when the derive feature is enabled
#[cfg(feature = "derive")]
pub use memscope_derive::Trackable;

// Set up the global allocator when the tracking-allocator feature is enabled
#[cfg(feature = "tracking-allocator")]
#[global_allocator]
/// Global tracking allocator instance used when the tracking-allocator feature is enabled.
pub static GLOBAL: TrackingAllocator = TrackingAllocator::new();

/// Trait for types that can be tracked by the memory tracker.
pub trait Trackable {
    /// Get the pointer to the heap allocation for this value.
    fn get_heap_ptr(&self) -> Option<usize>;

    /// Get the type name for this value.
    fn get_type_name(&self) -> &'static str;

    /// Get estimated size of the allocation.
    fn get_size_estimate(&self) -> usize;

    /// Get the reference count for smart pointers (default: 1 for non-smart pointers)
    fn get_ref_count(&self) -> usize {
        1
    }

    /// Get the data pointer for grouping related instances (default: same as heap_ptr)
    fn get_data_ptr(&self) -> usize {
        self.get_heap_ptr().unwrap_or(0)
    }

    /// Get all internal heap allocations for composite types (default: empty for simple types)
    fn get_internal_allocations(&self, _var_name: &str) -> Vec<(usize, String)> {
        Vec::new()
    }

    /// Track clone relationship for smart pointers (default: no-op for non-smart pointers)
    fn track_clone_relationship(&self, _clone_ptr: usize, _source_ptr: usize) {
        // Default implementation does nothing
    }

    /// Update reference count tracking for smart pointers (default: no-op for non-smart pointers)
    fn update_ref_count_tracking(&self, _ptr: usize) {
        // Default implementation does nothing
    }

    /// Get advanced type analysis information (default: None for simple types)
    fn get_advanced_type_info(&self) -> Option<crate::advanced_types::AdvancedTypeInfo> {
        // Check if this is an advanced type and analyze it
        let type_name = self.get_type_name();
        if crate::advanced_types::is_advanced_type(type_name) {
            // Create a minimal allocation info for analysis
            let allocation = crate::core::types::AllocationInfo {
                ptr: self.get_heap_ptr().unwrap_or(0),
                size: self.get_size_estimate(),
                var_name: None,
                type_name: Some(type_name.to_string()),
                scope_name: None,
                timestamp_alloc: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
                timestamp_dealloc: None,
                thread_id: std::thread::current().id(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: None,
                smart_pointer_info: None,
                memory_layout: None,
                generic_info: None,
                dynamic_type_info: None,
                runtime_state: None,
                stack_allocation: None,
                temporary_object: None,
                fragmentation_analysis: None,
                generic_instantiation: None,
                type_relationships: None,
                type_usage: None,
                function_call_tracking: None,
                lifecycle_tracking: None,
                access_tracking: None,
            };

            Some(
                crate::advanced_types::GenericAdvancedTypeAnalyzer::analyze_by_type_name(
                    type_name,
                    &allocation,
                ),
            )
        } else {
            None
        }
    }
}

// Implement Trackable for common heap-allocated types
impl<T> Trackable for Vec<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        if self.capacity() > 0 {
            Some(self.as_ptr() as usize)
        } else {
            None
        }
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<Vec<T>>()
    }

    fn get_size_estimate(&self) -> usize {
        self.capacity() * std::mem::size_of::<T>()
    }
}

impl Trackable for String {
    fn get_heap_ptr(&self) -> Option<usize> {
        if self.capacity() > 0 {
            Some(self.as_ptr() as usize)
        } else {
            None
        }
    }

    fn get_type_name(&self) -> &'static str {
        "String"
    }

    fn get_size_estimate(&self) -> usize {
        self.capacity()
    }
}

impl<T> Trackable for Box<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        Some(self.as_ref() as *const T as usize)
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<Box<T>>()
    }

    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<T>()
    }
}

impl<T> Trackable for std::rc::Rc<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        // For Rc, we create a truly unique identifier by using the Rc instance address
        // This ensures each TrackedVariable<Rc<T>> gets a completely unique identifier
        let instance_ptr = self as *const _ as usize;

        // Use the instance pointer directly, but ensure it's in a safe range for JSON
        // Add an offset to distinguish from regular heap pointers
        Some(0x5000_0000 + (instance_ptr % 0x0FFF_FFFF))
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::rc::Rc<T>>()
    }

    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<T>() + std::mem::size_of::<usize>() * 2 // Data + ref counts
    }

    /// Get the reference count for this Rc
    fn get_ref_count(&self) -> usize {
        std::rc::Rc::strong_count(self)
    }

    /// Get the data pointer for grouping related Rc instances
    fn get_data_ptr(&self) -> usize {
        std::rc::Rc::as_ptr(self) as usize
    }

    fn track_clone_relationship(&self, clone_ptr: usize, source_ptr: usize) {
        let tracker = crate::core::tracker::get_global_tracker();
        let data_ptr = self.get_data_ptr();
        let strong_count = std::rc::Rc::strong_count(self);
        let weak_count = std::rc::Rc::weak_count(self);

        if let Err(e) = tracker.track_smart_pointer_clone(
            clone_ptr,
            source_ptr,
            data_ptr,
            strong_count,
            weak_count,
        ) {
            tracing::warn!("Failed to track Rc clone relationship: {}", e);
        }
    }

    fn update_ref_count_tracking(&self, ptr: usize) {
        let tracker = crate::core::tracker::get_global_tracker();
        let strong_count = std::rc::Rc::strong_count(self);
        let weak_count = std::rc::Rc::weak_count(self);

        if let Err(e) = tracker.update_smart_pointer_ref_count(ptr, strong_count, weak_count) {
            tracing::warn!("Failed to update Rc ref count: {}", e);
        }
    }
}

impl<T> Trackable for std::sync::Arc<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        // For Arc, we create a truly unique identifier by using the Arc instance address
        // This ensures each TrackedVariable<Arc<T>> gets a completely unique identifier
        let instance_ptr = self as *const _ as usize;

        // Use the instance pointer directly, but ensure it's in a safe range for JSON
        // Add an offset to distinguish from regular heap pointers and Rc
        Some(0x6000_0000 + (instance_ptr % 0x0FFF_FFFF))
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::sync::Arc<T>>()
    }

    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<T>() + std::mem::size_of::<std::sync::atomic::AtomicUsize>() * 2
        // Data + atomic ref counts
    }

    /// Get the reference count for this Arc
    fn get_ref_count(&self) -> usize {
        std::sync::Arc::strong_count(self)
    }

    /// Get the data pointer for grouping related Arc instances
    fn get_data_ptr(&self) -> usize {
        std::sync::Arc::as_ptr(self) as usize
    }

    fn track_clone_relationship(&self, clone_ptr: usize, source_ptr: usize) {
        let tracker = crate::core::tracker::get_global_tracker();
        let data_ptr = self.get_data_ptr();
        let strong_count = std::sync::Arc::strong_count(self);
        let weak_count = std::sync::Arc::weak_count(self);

        if let Err(e) = tracker.track_smart_pointer_clone(
            clone_ptr,
            source_ptr,
            data_ptr,
            strong_count,
            weak_count,
        ) {
            tracing::warn!("Failed to track Arc clone relationship: {}", e);
        }
    }

    fn update_ref_count_tracking(&self, ptr: usize) {
        let tracker = crate::core::tracker::get_global_tracker();
        let strong_count = std::sync::Arc::strong_count(self);
        let weak_count = std::sync::Arc::weak_count(self);

        if let Err(e) = tracker.update_smart_pointer_ref_count(ptr, strong_count, weak_count) {
            tracing::warn!("Failed to update Arc ref count: {}", e);
        }
    }
}

impl<K, V> Trackable for std::collections::HashMap<K, V> {
    fn get_heap_ptr(&self) -> Option<usize> {
        // HashMap has internal heap allocations for buckets
        // We'll use the HashMap's address as a proxy
        Some(self as *const _ as usize)
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::collections::HashMap<K, V>>()
    }

    fn get_size_estimate(&self) -> usize {
        // Rough estimate: capacity * (key_size + value_size + overhead)
        self.capacity() * (std::mem::size_of::<K>() + std::mem::size_of::<V>() + 16)
    }
}

impl<K, V> Trackable for std::collections::BTreeMap<K, V> {
    fn get_heap_ptr(&self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            Some(self as *const _ as usize)
        }
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::collections::BTreeMap<K, V>>()
    }

    fn get_size_estimate(&self) -> usize {
        // BTreeMap nodes: rough estimate
        self.len() * (std::mem::size_of::<K>() + std::mem::size_of::<V>() + 32)
    }
}

impl<T> Trackable for std::collections::HashSet<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            Some(self as *const _ as usize)
        }
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::collections::HashSet<T>>()
    }

    fn get_size_estimate(&self) -> usize {
        self.capacity() * (std::mem::size_of::<T>() + 8) // T + hash overhead
    }
}

impl<T> Trackable for std::collections::BTreeSet<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            Some(self as *const _ as usize)
        }
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::collections::BTreeSet<T>>()
    }

    fn get_size_estimate(&self) -> usize {
        self.len() * (std::mem::size_of::<T>() + 24) // T + tree node overhead
    }
}

impl<T> Trackable for std::collections::VecDeque<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        if self.capacity() > 0 {
            Some(self.as_slices().0.as_ptr() as usize)
        } else {
            None
        }
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::collections::VecDeque<T>>()
    }

    fn get_size_estimate(&self) -> usize {
        self.capacity() * std::mem::size_of::<T>()
    }
}

impl<T> Trackable for std::collections::LinkedList<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            Some(self as *const _ as usize)
        }
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::collections::LinkedList<T>>()
    }

    fn get_size_estimate(&self) -> usize {
        self.len() * (std::mem::size_of::<T>() + std::mem::size_of::<usize>() * 2)
        // T + prev/next pointers
    }
}

impl<T> Trackable for std::collections::BinaryHeap<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        if self.capacity() > 0 {
            Some(self as *const _ as usize)
        } else {
            None
        }
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::collections::BinaryHeap<T>>()
    }

    fn get_size_estimate(&self) -> usize {
        self.capacity() * std::mem::size_of::<T>()
    }
}

impl<T> Trackable for std::rc::Weak<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        // Weak pointers don't own the data, but we can track the weak reference itself
        let instance_ptr = self as *const _ as usize;
        Some(0x7000_0000 + (instance_ptr % 0x0FFF_FFFF))
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::rc::Weak<T>>()
    }

    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<std::rc::Weak<T>>()
    }

    fn get_ref_count(&self) -> usize {
        self.weak_count()
    }

    fn get_data_ptr(&self) -> usize {
        // Try to upgrade and get data pointer, return 0 if data is gone
        if let Some(upgraded) = self.upgrade() {
            std::rc::Rc::as_ptr(&upgraded) as usize
        } else {
            0 // Data has been deallocated
        }
    }
}

impl<T> Trackable for std::sync::Weak<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        // Weak pointers don't own the data, but we can track the weak reference itself
        let instance_ptr = self as *const _ as usize;
        Some(0x8000_0000 + (instance_ptr % 0x0FFF_FFFF))
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::sync::Weak<T>>()
    }

    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<std::sync::Weak<T>>()
    }

    fn get_ref_count(&self) -> usize {
        self.weak_count()
    }

    fn get_data_ptr(&self) -> usize {
        // Try to upgrade and get data pointer, return 0 if data is gone
        if let Some(upgraded) = self.upgrade() {
            std::sync::Arc::as_ptr(&upgraded) as usize
        } else {
            0 // Data has been deallocated
        }
    }
}

// Use the macro to implement Trackable for advanced types
impl_advanced_trackable!(std::cell::RefCell<T>, 0xA000_0000);
impl_advanced_trackable!(std::sync::Mutex<T>, 0xB000_0000);
impl_advanced_trackable!(std::sync::RwLock<T>, 0xC000_0000);

// Additional advanced types with the macro
impl_advanced_trackable!(std::cell::Cell<T>, 0xA100_0000);
impl_advanced_trackable!(std::sync::mpsc::Sender<T>, 0xD000_0000);
impl_advanced_trackable!(std::sync::mpsc::Receiver<T>, 0xD100_0000);
impl_advanced_trackable!(std::sync::atomic::AtomicBool, 0xE000_0000, no_generics);
impl_advanced_trackable!(std::sync::atomic::AtomicUsize, 0xE100_0000, no_generics);
impl_advanced_trackable!(std::sync::atomic::AtomicIsize, 0xE200_0000, no_generics);
impl_advanced_trackable!(std::sync::atomic::AtomicU8, 0xE300_0000, no_generics);
impl_advanced_trackable!(std::sync::atomic::AtomicU16, 0xE400_0000, no_generics);
impl_advanced_trackable!(std::sync::atomic::AtomicU32, 0xE500_0000, no_generics);
impl_advanced_trackable!(std::sync::atomic::AtomicU64, 0xE600_0000, no_generics);
impl_advanced_trackable!(std::sync::atomic::AtomicI8, 0xE700_0000, no_generics);
impl_advanced_trackable!(std::sync::atomic::AtomicI16, 0xE800_0000, no_generics);
impl_advanced_trackable!(std::sync::atomic::AtomicI32, 0xE900_0000, no_generics);
impl_advanced_trackable!(std::sync::atomic::AtomicI64, 0xEA00_0000, no_generics);
impl_advanced_trackable!(std::mem::ManuallyDrop<T>, 0xF000_0000);
impl_advanced_trackable!(std::mem::MaybeUninit<T>, 0xF100_0000);
impl_advanced_trackable!(std::pin::Pin<T>, 0xF200_0000);

// Implement for Option<T> where T: Trackable
impl<T: Trackable> Trackable for Option<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        match self {
            Some(value) => value.get_heap_ptr(),
            None => None,
        }
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<Option<T>>()
    }

    fn get_size_estimate(&self) -> usize {
        match self {
            Some(value) => std::mem::size_of::<Option<T>>() + value.get_size_estimate(),
            None => std::mem::size_of::<Option<T>>(),
        }
    }

    fn get_internal_allocations(&self, var_name: &str) -> Vec<(usize, String)> {
        match self {
            Some(value) => value.get_internal_allocations(&format!("{}::Some", var_name)),
            None => Vec::new(),
        }
    }
}

// Implement for Result<T, E> where T: Trackable, E: Trackable
impl<T: Trackable, E: Trackable> Trackable for Result<T, E> {
    fn get_heap_ptr(&self) -> Option<usize> {
        match self {
            Ok(value) => value.get_heap_ptr(),
            Err(error) => error.get_heap_ptr(),
        }
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<Result<T, E>>()
    }

    fn get_size_estimate(&self) -> usize {
        match self {
            Ok(value) => std::mem::size_of::<Result<T, E>>() + value.get_size_estimate(),
            Err(error) => std::mem::size_of::<Result<T, E>>() + error.get_size_estimate(),
        }
    }

    fn get_internal_allocations(&self, var_name: &str) -> Vec<(usize, String)> {
        match self {
            Ok(value) => value.get_internal_allocations(&format!("{}::Ok", var_name)),
            Err(error) => error.get_internal_allocations(&format!("{}::Err", var_name)),
        }
    }
}

/// **[RECOMMENDED]** Track a variable's memory allocation without taking ownership.
///
/// This is the **default and recommended** tracking macro for most use cases.
/// It performs zero-cost tracking by reference, allowing continued use of the original variable.
///
/// ## ✅ Use this when:
/// - You want to track memory usage without changing your code
/// - Performance is critical (zero overhead)
/// - You need to continue using the variable after tracking
/// - You're tracking many variables and don't want clone overhead
/// - You're doing basic memory profiling and analysis
///
/// ## ❌ Don't use this when:
/// - You need precise lifecycle tracking with automatic cleanup
/// - You're tracking temporary variables that will be moved/consumed immediately
///
/// # Example
/// ```rust
/// use memscope_rs::track_var;
///
/// let my_vec = vec![1, 2, 3, 4, 5];
/// track_var!(my_vec); // Zero-cost tracking
/// // my_vec can still be used normally - no ownership changes!
/// println!("Vector: {:?}", my_vec);
/// my_vec.push(6); // Still fully usable
/// ```
#[macro_export]
macro_rules! track_var {
    ($var:expr) => {{
        let var_name = stringify!($var);
        let _ = $crate::_track_var_impl(&$var, var_name);
        // Pure tracking - no return value to avoid any ownership implications
    }};
}

/// **[ADVANCED]** Track a variable with full lifecycle management and ownership transfer.
///
/// This macro creates a tracking wrapper that takes ownership of the variable
/// and provides automatic lifecycle tracking with precise timing measurements.
///
/// ## ✅ Use this when:
/// - You need precise lifecycle tracking with automatic cleanup detection
/// - You want to measure exact variable lifetimes
/// - You're doing advanced memory analysis or debugging
/// - You're tracking variables that will be consumed/moved anyway
/// - You need the wrapper's additional methods (get(), get_mut(), into_inner())
///
/// ## ❌ Don't use this when:
/// - You need to continue using the original variable (use `track_var!` instead)
/// - Performance is critical and you don't need lifecycle timing
/// - You're tracking many variables (clone overhead)
/// - You're doing basic memory profiling
///
/// ## ⚠️ Performance Note:
/// This macro takes ownership of the variable. If you need the original variable
/// afterwards, you'll need to clone it first, which has performance implications.
///
/// # Example
/// ```rust
/// use memscope_rs::track_var_owned;
///
/// let my_vec = vec![1, 2, 3, 4, 5];
/// let tracked_vec = track_var_owned!(my_vec); // Takes ownership
/// // tracked_vec behaves like my_vec but with automatic lifecycle tracking
/// println!("Length: {}", tracked_vec.len()); // Transparent access via Deref
/// let original = tracked_vec.into_inner(); // Get original back if needed
/// ```
#[macro_export]
macro_rules! track_var_owned {
    ($var:expr) => {{
        let var_name = stringify!($var);
        $crate::TrackedVariable::new($var, var_name.to_string())
    }};
}

/// **[SMART]** Intelligent tracking that automatically chooses the best strategy.
///
/// This macro automatically detects the variable type and chooses the optimal tracking approach:
/// - For `Copy` types (i32, f64, bool, etc.): Creates a copy for tracking (zero overhead)
/// - For non-`Copy` types: Uses reference-based tracking like `track_var!`
/// - For smart pointers (Rc, Arc): Clones the pointer (cheap reference increment)
///
/// ## ✅ Use this when:
/// - You want the best of both worlds without thinking about it
/// - You're tracking mixed types (some Copy, some not)
/// - You want automatic optimization based on type characteristics
/// - You're prototyping and want convenience
///
/// ## ❌ Don't use this when:
/// - You need explicit control over tracking behavior
/// - You're in performance-critical code and want predictable behavior
/// - You need precise lifecycle tracking (use `track_var_owned!` instead)
///
/// # Example
/// ```rust
/// use memscope_rs::track_var_smart;
///
/// let number = 42i32;           // Copy type - will be copied
/// let my_vec = vec![1, 2, 3];   // Non-Copy - will be tracked by reference
/// let rc_data = Rc::new(vec![]); // Smart pointer - will clone the Rc
///
/// track_var_smart!(number);   // Copies the i32 (cheap)
/// track_var_smart!(my_vec);    // Tracks by reference (zero cost)
/// track_var_smart!(rc_data);   // Clones Rc (cheap reference increment)
///
/// // All variables remain fully usable!
/// println!("{}, {:?}, {:?}", number, my_vec, rc_data);
/// ```
#[macro_export]
macro_rules! track_var_smart {
    ($var:expr) => {{
        let var_name = stringify!($var);
        $crate::_smart_track_var_impl($var, var_name)
    }};
}

// Global counter for generating unique identifiers for TrackedVariable instances
static TRACKED_VARIABLE_COUNTER: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(1);

/// A wrapper that provides automatic lifecycle tracking for variables.
///
/// This struct wraps any `Trackable` type and automatically handles:
/// - Creation tracking when constructed
/// - Destruction tracking when dropped
/// - Transparent access to the wrapped value
pub struct TrackedVariable<T: Trackable> {
    inner: T,
    var_name: String,
    ptr: Option<usize>,
    creation_time: u64,
    unique_id: usize, // Unique identifier for this TrackedVariable instance
}

impl<T: Trackable> TrackedVariable<T> {
    /// Create a new tracked variable wrapper.
    pub fn new(value: T, var_name: String) -> Self {
        let creation_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let unique_id = TRACKED_VARIABLE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let type_name = value.get_type_name().to_string();
        let is_smart_pointer = type_name.contains("::Rc<") || type_name.contains("::Arc<");

        // For smart pointers, use a unique synthetic pointer based on the TrackedVariable instance
        let ptr = if is_smart_pointer {
            // Generate a unique pointer for this TrackedVariable instance
            if type_name.contains("::Rc<") {
                Some(0x5000_0000 + unique_id)
            } else {
                Some(0x6000_0000 + unique_id)
            }
        } else {
            value.get_heap_ptr()
        };

        // Track creation
        if let Some(ptr_val) = ptr {
            let tracker = get_global_tracker();

            // Register in variable registry
            let _ = crate::variable_registry::VariableRegistry::register_variable(
                ptr_val,
                var_name.clone(),
                type_name.clone(),
                value.get_size_estimate(),
            );

            if is_smart_pointer {
                // For Rc/Arc, always create a specialized smart pointer allocation
                let ref_count = value.get_ref_count();
                let data_ptr = value.get_data_ptr();

                let _ = tracker.create_smart_pointer_allocation(
                    ptr_val,
                    value.get_size_estimate(),
                    var_name.clone(),
                    type_name.clone(),
                    creation_time,
                    ref_count,
                    data_ptr,
                );

                tracing::debug!(
                    "🎯 Created smart pointer allocation for '{}' at unique ptr 0x{:x} (id={}), ref_count={}, data_ptr=0x{:x}",
                    var_name,
                    ptr_val,
                    unique_id,
                    ref_count,
                    data_ptr
                );
            } else {
                // For regular types, check if already tracked to prevent duplicates
                if let Ok(active_allocations) = tracker.get_active_allocations() {
                    let already_tracked =
                        active_allocations.iter().any(|alloc| alloc.ptr == ptr_val);
                    if !already_tracked {
                        let _ = tracker.associate_var(ptr_val, var_name.clone(), type_name.clone());
                    } else {
                        // Just update the existing allocation with variable info
                        let _ = tracker.update_allocation_info(
                            ptr_val,
                            var_name.clone(),
                            type_name.clone(),
                        );
                    }
                } else {
                    // Fallback: try to associate anyway
                    let _ = tracker.associate_var(ptr_val, var_name.clone(), type_name.clone());
                }

                tracing::debug!(
                    "🎯 Created tracked variable '{}' at ptr 0x{:x}",
                    var_name,
                    ptr_val
                );
            }
        }

        Self {
            inner: value,
            var_name,
            ptr,
            creation_time,
            unique_id,
        }
    }

    /// Get a reference to the inner value.
    pub fn get(&self) -> &T {
        &self.inner
    }

    /// Get a mutable reference to the inner value.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Consume the wrapper and return the inner value.
    pub fn into_inner(self) -> T {
        // Manually trigger drop logic before consuming
        if let Some(ptr_val) = self.ptr {
            Self::track_destruction(&self.var_name, ptr_val, self.creation_time);
        }

        // Safe ownership transfer using ManuallyDrop
        let manual_drop_self = std::mem::ManuallyDrop::new(self);
        // SAFETY: We're taking ownership of the inner value and preventing Drop from running
        unsafe { std::ptr::read(&manual_drop_self.inner) }
    }

    /// Internal method to track variable destruction.
    fn track_destruction(var_name: &str, ptr: usize, creation_time: u64) {
        let destruction_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let lifetime_ms = (destruction_time.saturating_sub(creation_time)) / 1_000_000;

        // Update variable registry with destruction info
        let _ = crate::variable_registry::VariableRegistry::mark_variable_destroyed(
            ptr,
            destruction_time,
        );

        // Track deallocation with precise lifetime in memory tracker
        let tracker = get_global_tracker();
        let _ = tracker.track_deallocation_with_lifetime(ptr, lifetime_ms);

        tracing::debug!(
            "💀 Destroyed tracked variable '{}' at ptr 0x{:x}, lifetime: {}ms",
            var_name,
            ptr,
            lifetime_ms
        );
    }

    /// Internal method to track smart pointer destruction with enhanced metadata.
    fn track_smart_pointer_destruction(
        var_name: &str,
        ptr: usize,
        creation_time: u64,
        final_ref_count: usize,
    ) {
        let destruction_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let lifetime_ms = (destruction_time.saturating_sub(creation_time)) / 1_000_000;

        // Update variable registry with destruction info
        let _ = crate::variable_registry::VariableRegistry::mark_variable_destroyed(
            ptr,
            destruction_time,
        );

        // Track smart pointer deallocation with enhanced metadata
        let tracker = get_global_tracker();
        let _ = tracker.track_smart_pointer_deallocation(ptr, lifetime_ms, final_ref_count);

        tracing::debug!(
            "💀 Destroyed smart pointer '{}' at ptr 0x{:x}, lifetime: {}ms, final_ref_count: {}",
            var_name,
            ptr,
            lifetime_ms,
            final_ref_count
        );
    }
}

impl<T: Trackable> Drop for TrackedVariable<T> {
    fn drop(&mut self) {
        if let Some(ptr_val) = self.ptr {
            let type_name = self.inner.get_type_name();
            let is_smart_pointer = type_name.contains("::Rc<") || type_name.contains("::Arc<");

            if is_smart_pointer {
                // For smart pointers, get the final reference count before destruction
                let final_ref_count = self.inner.get_ref_count();
                Self::track_smart_pointer_destruction(
                    &self.var_name,
                    ptr_val,
                    self.creation_time,
                    final_ref_count,
                );
            } else {
                // For regular types, use standard destruction tracking
                Self::track_destruction(&self.var_name, ptr_val, self.creation_time);
            }
        }
    }
}

// Implement Deref and DerefMut for transparent access
impl<T: Trackable> std::ops::Deref for TrackedVariable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Trackable> std::ops::DerefMut for TrackedVariable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// Implement common traits to make TrackedVariable behave like the inner type
impl<T: Trackable + std::fmt::Debug> std::fmt::Debug for TrackedVariable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TrackedVariable({:?})", self.inner)
    }
}

impl<T: Trackable + std::fmt::Display> std::fmt::Display for TrackedVariable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<T: Trackable + Clone> Clone for TrackedVariable<T> {
    fn clone(&self) -> Self {
        // Create a new tracked variable for the clone with a unique name
        let clone_name = format!("{}_clone_{}", self.var_name, self.unique_id);
        Self::new(self.inner.clone(), clone_name)
    }
}

/// Internal implementation function for the track_var! macro.
/// This function should not be called directly.
///
/// Enhanced with log-based variable name persistence for lifecycle-independent tracking.
#[doc(hidden)]
pub fn _track_var_impl<T: Trackable>(var: &T, var_name: &str) -> TrackingResult<()> {
    if let Some(ptr) = var.get_heap_ptr() {
        let tracker = get_global_tracker();
        let type_name = var.get_type_name().to_string();

        // 1. Register variable in HashMap registry (lightweight and fast)
        let _ = crate::variable_registry::VariableRegistry::register_variable(
            ptr,
            var_name.to_string(),
            type_name.clone(),
            var.get_size_estimate(),
        );

        // 2. Original tracking logic remains unchanged
        tracing::debug!(
            "Tracking variable '{}' of type '{}' at ptr 0x{:x}",
            var_name,
            type_name,
            ptr
        );

        tracker.associate_var(ptr, var_name.to_string(), type_name)
    } else {
        // Variable doesn't have a heap allocation (e.g., empty Vec)
        tracing::debug!("Variable '{}' has no heap allocation to track", var_name);
        Ok(())
    }
}

/// Internal implementation for smart tracking that chooses optimal strategy.
/// This function should not be called directly.
#[doc(hidden)]
pub fn _smart_track_var_impl<T: Trackable + 'static>(var: T, var_name: &str) -> TrackingResult<T> {
    use std::any::TypeId;

    let type_id = TypeId::of::<T>();
    let type_name = std::any::type_name::<T>();

    // Check if it's a Copy type by attempting to get TypeId of common Copy types
    let is_copy_type = type_id == TypeId::of::<i8>()
        || type_id == TypeId::of::<i16>()
        || type_id == TypeId::of::<i32>()
        || type_id == TypeId::of::<i64>()
        || type_id == TypeId::of::<i128>()
        || type_id == TypeId::of::<isize>()
        || type_id == TypeId::of::<u8>()
        || type_id == TypeId::of::<u16>()
        || type_id == TypeId::of::<u32>()
        || type_id == TypeId::of::<u64>()
        || type_id == TypeId::of::<u128>()
        || type_id == TypeId::of::<usize>()
        || type_id == TypeId::of::<f32>()
        || type_id == TypeId::of::<f64>()
        || type_id == TypeId::of::<bool>()
        || type_id == TypeId::of::<char>();

    let is_smart_pointer = type_name.contains("::Rc<")
        || type_name.contains("::Arc<")
        || type_name.contains("::Weak<");

    if is_copy_type {
        // For Copy types, we can safely track by reference and return the value
        let _ = _track_var_impl(&var, var_name);
        tracing::debug!(
            "🧠 Smart tracking: Copy type '{}' tracked by reference",
            var_name
        );
        Ok(var)
    } else if is_smart_pointer {
        // For smart pointers, track by reference and return the value
        let _ = _track_var_impl(&var, var_name);
        tracing::debug!(
            "🧠 Smart tracking: Smart pointer '{}' tracked by reference",
            var_name
        );
        Ok(var)
    } else {
        // For other types, track by reference and return the value
        let _ = _track_var_impl(&var, var_name);
        tracing::debug!(
            "🧠 Smart tracking: Non-Copy type '{}' tracked by reference",
            var_name
        );
        Ok(var)
    }
}

/// Initialize the memory tracking system.
///
/// This function sets up the tracing subscriber and prepares the global tracker.
/// Call this early in your application, typically in main().
///
/// # Example
/// ```rust
/// memscope_rs::init();
/// // Your application code here
/// ```
pub fn init() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "memscope_rs=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("memscope-rs initialized");
}

/// Enable automatic JSON export when program ends
/// Call this at the beginning of your program to enable auto-export
pub fn enable_auto_export(export_path: Option<&str>) {
    std::env::set_var("MEMSCOPE_AUTO_EXPORT", "1");
    if let Some(path) = export_path {
        std::env::set_var("MEMSCOPE_EXPORT_PATH", path);
    }

    // Install exit hook for automatic export
    install_exit_hook();

    println!(
        "📋 Auto-export enabled - JSON will be exported to: {}",
        export_path.unwrap_or("memscope_final_snapshot.json")
    );
}

/// Install program exit hook for automatic data export
fn install_exit_hook() {
    use std::sync::Once;
    static HOOK_INSTALLED: Once = Once::new();

    HOOK_INSTALLED.call_once(|| {
        // Install panic hook
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            eprintln!("🚨 Program panicked, attempting to export memory data...");
            let _ = export_final_snapshot("memscope_panic_snapshot");
            original_hook(panic_info);
        }));

        // Use libc atexit for reliable program exit handling
        extern "C" fn exit_handler() {
            if std::env::var("MEMSCOPE_AUTO_EXPORT").is_ok() {
                println!("🔄 Program ending, exporting final memory snapshot...");
                let export_path = std::env::var("MEMSCOPE_EXPORT_PATH")
                    .unwrap_or_else(|_| "memscope_final_snapshot".to_string());

                if let Err(e) = export_final_snapshot(&export_path) {
                    eprintln!("❌ Failed to export final snapshot: {}", e);
                } else {
                    println!("✅ Final memory snapshot exported successfully");
                }
            }
        }

        unsafe {
            libc::atexit(exit_handler);
        }

        tracing::debug!("📌 Exit hooks installed for automatic memory export");
    });
}

/// Guard struct that exports data when dropped (on program exit)
// Removed unused ExitGuard struct

// Removed unused ExitGuard implementation

// Removed unused ExitGuard Drop implementation

/// Export final memory snapshot with complete lifecycle data
fn export_final_snapshot(base_path: &str) -> TrackingResult<()> {
    let tracker = get_global_tracker();

    // Force a final garbage collection attempt to capture any remaining deallocations
    std::thread::sleep(std::time::Duration::from_millis(10));

    let json_path = format!("{}.json", base_path);
    tracker.export_to_json(&json_path)?;

    // Also export HTML if requested
    let export_format =
        std::env::var("MEMSCOPE_EXPORT_FORMAT").unwrap_or_else(|_| "json".to_string());
    if export_format == "html" || export_format == "both" {
        let html_path = format!("{}.html", base_path);
        let _ = tracker.export_interactive_dashboard(&html_path);
    }

    Ok(())
}
