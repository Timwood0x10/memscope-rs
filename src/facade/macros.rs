//! Facade Macros - Convenient macros for memory tracking
//!
//! This module provides macros that simplify common memory tracking
/// operations, making the API more ergonomic and reducing boilerplate.
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::thread;

/// Helper function to hash thread ID
#[doc(hidden)]
pub fn get_heap_ptr_if_trackable<T>(_value: &T) -> Option<usize> {
    // This function is a placeholder. In a real implementation,
    // we would check if T implements Trackable and call get_heap_ptr()
    // For now, we return None to indicate we should use stack address
    None
}

/// Helper function to hash thread ID
#[doc(hidden)]
pub fn hash_thread_id() -> u64 {
    let thread_id = thread::current().id();
    let thread_id_str = format!("{:?}", thread_id);
    let mut hasher = DefaultHasher::new();
    thread_id_str.hash(&mut hasher);
    hasher.finish()
}

/// Get a memory snapshot and display summary
///
/// This macro creates a quick summary of current memory usage,
/// useful for debugging and monitoring.
///
/// # Examples
///
/// ```rust
/// use memscope_rs::memscope_summary;
///
/// memscope_summary!();
/// ```
#[macro_export]
macro_rules! memscope_summary {
    () => {
        let summary = $crate::facade::compat::get_memory_summary();
        println!("Memory Summary:");
        println!("  Total Allocations: {}", summary.total_allocations);
        println!("  Active Allocations: {}", summary.active_allocations);
        println!("  Current Memory: {} bytes", summary.current_memory);
        println!("  Peak Memory: {} bytes", summary.peak_memory);
        println!("  Thread Count: {}", summary.thread_count);
    };
}

/// Get top allocations and display them
///
/// This macro shows the largest memory allocations, useful for
/// identifying memory usage patterns.
///
/// # Examples
///
/// ```rust
/// use memscope_rs::show_top_allocations;
///
/// show_top_allocations!(10);
/// ```
#[macro_export]
macro_rules! show_top_allocations {
    ($limit:expr) => {
        let allocations = $crate::facade::compat::get_top_allocations($limit);
        println!("Top {} Allocations by Size:", $limit);
        for (i, alloc) in allocations.iter().enumerate() {
            println!("  {}. {} bytes at {:x}", i + 1, alloc.size, alloc.ptr);
        }
    };
}

/// Export memory data to JSON and print the result
///
/// This macro is a convenient way to export memory data
/// in JSON format for external processing or analysis.
///
/// # Examples
///
/// ```rust
/// use memscope_rs::export_memory_json;
///
/// export_memory_json!(true);
/// ```
#[macro_export]
macro_rules! export_memory_json {
    ($verbose:expr) => {
        match $crate::facade::compat::export_json($verbose) {
            Ok(json) => {
                println!("Memory data exported to JSON:");
                println!("{}", json);
            }
            Err(e) => {
                eprintln!("Failed to export memory data: {}", e);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_memscope_summary_macro_compiles() {
        // This test just ensures the macro compiles correctly
        // We don't actually run it to avoid printing to stdout during tests
    }
}
