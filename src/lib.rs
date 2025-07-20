//! Memory tracking and visualization tools for Rust applications.
//!
//! This crate provides tools for tracking memory allocations and visualizing
//! memory usage in Rust applications. It includes a custom global allocator
//! that tracks all heap allocations and deallocations, and provides utilities
//! for exporting memory usage data in various formats.

#![warn(missing_docs)]

/// Memory allocation tracking and analysis
pub mod allocator;
/// Advanced memory analysis functionality
pub mod analysis;
// Removed export_enhanced - functionality consolidated into visualization.rs
/// Scope tracking functionality
pub mod scope_tracker;
/// Core memory tracking functionality
pub mod tracker;
/// Type definitions and data structures
pub mod types;
/// Variable registry for lightweight HashMap-based variable tracking
pub mod variable_registry;
/// Unsafe and FFI operation tracking
pub mod unsafe_ffi_tracker;
/// Utility functions
pub mod utils;
/// Visualization and chart generation
pub mod visualization;

// Re-export key functions from unified modules
/// Enhanced export functionality
pub mod export_enhanced;
/// HTML export functionality for interactive visualization
pub mod html_export;
pub use visualization::*;
pub use analysis::*;
// Re-export main types for easier use
pub use allocator::TrackingAllocator;
pub use tracker::{get_global_tracker, MemoryTracker};
pub use types::{AllocationInfo, TrackingError, TrackingResult};
pub use unsafe_ffi_tracker::{get_global_unsafe_ffi_tracker, UnsafeFFITracker};
pub use utils::{format_bytes, get_simple_type, simplify_type_name};
pub use visualization::{export_lifecycle_timeline, export_memory_analysis};
pub use html_export::export_interactive_html;

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
        // For Rc, the allocation tracking is complex because Rc uses a control block
        // We'll track the Rc itself rather than the inner data to avoid pointer issues
        Some(std::rc::Rc::as_ptr(self) as usize)
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::rc::Rc<T>>()
    }
    
    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<T>() + std::mem::size_of::<usize>() * 2 // Data + ref counts
    }
}

impl<T> Trackable for std::sync::Arc<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        // For Arc, the allocation tracking is complex because Arc uses a control block
        // We'll track the Arc itself rather than the inner data to avoid pointer issues
        Some(std::sync::Arc::as_ptr(self) as usize)
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::sync::Arc<T>>()
    }
    
    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<T>() + std::mem::size_of::<std::sync::atomic::AtomicUsize>() * 2 // Data + atomic ref counts
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

/// Macro to track a variable's memory allocation with automatic lifecycle management.
///
/// This enhanced macro creates a tracking wrapper that automatically handles:
/// - Variable creation tracking
/// - Automatic destruction tracking via Drop
/// - Accurate lifetime calculation
/// - Seamless integration with auto-export
///
/// # Example
/// ```rust
/// use memscope_rs::track_var;
///
/// let my_vec = vec![1, 2, 3, 4, 5];
/// let tracked_vec = track_var!(my_vec);
/// // tracked_vec behaves exactly like my_vec but with automatic lifecycle tracking
/// ```
#[macro_export]
macro_rules! track_var {
    ($var:expr) => {{
        let var_name = stringify!($var);
        $crate::TrackedVariable::new($var, var_name.to_string())
    }};
}

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
}

impl<T: Trackable> TrackedVariable<T> {
    /// Create a new tracked variable wrapper.
    pub fn new(value: T, var_name: String) -> Self {
        let creation_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let ptr = value.get_heap_ptr();
        
        // Track creation
        if let Some(ptr_val) = ptr {
            let tracker = get_global_tracker();
            let type_name = value.get_type_name().to_string();

            // Register in variable registry
            let _ = crate::variable_registry::VariableRegistry::register_variable(
                ptr_val,
                var_name.clone(),
                type_name.clone(),
                value.get_size_estimate(),
            );

            // Associate with tracker
            let _ = tracker.associate_var(ptr_val, var_name.clone(), type_name);

            tracing::debug!(
                "🎯 Created tracked variable '{}' at ptr 0x{:x}",
                var_name,
                ptr_val
            );
        }

        Self {
            inner: value,
            var_name,
            ptr,
            creation_time,
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
        
        // Prevent automatic Drop from running
        let inner = unsafe { std::ptr::read(&self.inner) };
        std::mem::forget(self);
        inner
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

        // Track deallocation in memory tracker
        let tracker = get_global_tracker();
        let _ = tracker.track_deallocation(ptr);

        tracing::debug!(
            "💀 Destroyed tracked variable '{}' at ptr 0x{:x}, lifetime: {}ms",
            var_name,
            ptr,
            lifetime_ms
        );
    }
}

impl<T: Trackable> Drop for TrackedVariable<T> {
    fn drop(&mut self) {
        if let Some(ptr_val) = self.ptr {
            Self::track_destruction(&self.var_name, ptr_val, self.creation_time);
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
        // Create a new tracked variable for the clone
        Self::new(self.inner.clone(), format!("{}_clone", self.var_name))
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
    
    println!("📋 Auto-export enabled - JSON will be exported to: {}", 
             export_path.unwrap_or("memscope_final_snapshot.json"));
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
struct ExitGuard;

impl ExitGuard {
    fn new() -> Self {
        Self
    }
}

impl Drop for ExitGuard {
    fn drop(&mut self) {
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
}

/// Export final memory snapshot with complete lifecycle data
fn export_final_snapshot(base_path: &str) -> TrackingResult<()> {
    let tracker = get_global_tracker();
    
    // Force a final garbage collection attempt to capture any remaining deallocations
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let json_path = format!("{}.json", base_path);
    tracker.export_to_json(&json_path)?;
    
    // Also export HTML if requested
    let export_format = std::env::var("MEMSCOPE_EXPORT_FORMAT").unwrap_or_else(|_| "json".to_string());
    if export_format == "html" || export_format == "both" {
        let html_path = format!("{}.html", base_path);
        let _ = tracker.export_interactive_dashboard(&html_path);
    }
    
    Ok(())
}
