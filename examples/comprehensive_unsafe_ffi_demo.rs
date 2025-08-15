//! Comprehensive Unsafe/FFI Demo
//! 
//! This example demonstrates a complex program with unsafe operations and FFI calls,
//! then exports the memory analysis to both binary and JSON+HTML formats.
//! 
//! Flow: Program → Binary → JSON + HTML
//! Uses user-binary format and project APIs only.

use memscope_rs::{init, track_var, get_global_tracker};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_void, c_int};
use std::ptr;
use std::slice;
use std::collections::HashMap;

// External C library simulation (normally would be linked)
extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn strlen(s: *const c_char) -> usize;
}

// Custom allocator for demonstration
struct UnsafeAllocator {
    allocated_blocks: Vec<*mut u8>,
}

impl UnsafeAllocator {
    fn new() -> Self {
        Self {
            allocated_blocks: Vec::new(),
        }
    }

    unsafe fn allocate(&mut self, size: usize) -> *mut u8 {
        let ptr = malloc(size) as *mut u8;
        if !ptr.is_null() {
            self.allocated_blocks.push(ptr);
        }
        ptr
    }

    unsafe fn deallocate(&mut self, ptr: *mut u8) {
        if let Some(pos) = self.allocated_blocks.iter().position(|&p| p == ptr) {
            self.allocated_blocks.remove(pos);
            free(ptr as *mut c_void);
        }
    }
}

// Complex data structure with unsafe operations
struct UnsafeDataProcessor {
    raw_buffer: *mut u8,
    buffer_size: usize,
    processed_data: Vec<u32>,
    ffi_strings: Vec<CString>,
    allocator: UnsafeAllocator,
}

impl UnsafeDataProcessor {
    fn new() -> Self {
        Self {
            raw_buffer: ptr::null_mut(),
            buffer_size: 0,
            processed_data: Vec::new(),
            ffi_strings: Vec::new(),
            allocator: UnsafeAllocator::new(),
        }
    }

    // Unsafe memory operations
    unsafe fn allocate_raw_buffer(&mut self, size: usize) -> Result<(), &'static str> {
        if !self.raw_buffer.is_null() {
            self.deallocate_raw_buffer();
        }

        self.raw_buffer = self.allocator.allocate(size);
        if self.raw_buffer.is_null() {
            return Err("Failed to allocate memory");
        }

        self.buffer_size = size;
        
        // Initialize buffer with pattern
        for i in 0..size {
            *self.raw_buffer.add(i) = (i % 256) as u8;
        }

        Ok(())
    }

    unsafe fn deallocate_raw_buffer(&mut self) {
        if !self.raw_buffer.is_null() {
            self.allocator.deallocate(self.raw_buffer);
            self.raw_buffer = ptr::null_mut();
            self.buffer_size = 0;
        }
    }

    // FFI operations with C strings
    fn process_c_strings(&mut self, strings: &[&str]) -> Result<Vec<usize>, &'static str> {
        let mut lengths = Vec::new();

        for &s in strings {
            let c_string = CString::new(s).map_err(|_| "Invalid string")?;
            
            unsafe {
                // Use FFI to get string length
                let c_ptr = c_string.as_ptr();
                let length = strlen(c_ptr);
                lengths.push(length);
            }

            self.ffi_strings.push(c_string);
        }

        Ok(lengths)
    }

    // Complex data processing with unsafe pointer arithmetic
    unsafe fn process_raw_data(&mut self) -> Result<(), &'static str> {
        if self.raw_buffer.is_null() || self.buffer_size == 0 {
            return Err("No buffer allocated");
        }

        // Create slice from raw pointer
        let data_slice = slice::from_raw_parts(self.raw_buffer, self.buffer_size);
        
        // Process data in chunks
        for chunk in data_slice.chunks(4) {
            let mut value: u32 = 0;
            for (i, &byte) in chunk.iter().enumerate() {
                value |= (byte as u32) << (i * 8);
            }
            self.processed_data.push(value);
        }

        Ok(())
    }

    // Simulate complex memory patterns (removed since we moved this logic to main)
    fn _create_complex_allocations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // This method is no longer used but kept for reference
        Ok(())
    }
}

impl Drop for UnsafeDataProcessor {
    fn drop(&mut self) {
        unsafe {
            self.deallocate_raw_buffer();
            
            // Clean up remaining allocations
            for ptr in &self.allocator.allocated_blocks {
                free(*ptr as *mut c_void);
            }
        }
    }
}

// Complex generic structure for type analysis
struct GenericContainer<T, U> 
where 
    T: Clone + Send,
    U: std::fmt::Debug,
{
    primary_data: Vec<T>,
    secondary_data: HashMap<String, U>,
    metadata: Box<dyn std::any::Any + Send>,
}

impl<T, U> GenericContainer<T, U> 
where 
    T: Clone + Send + 'static,
    U: std::fmt::Debug + 'static,
{
    fn new() -> Self {
        Self {
            primary_data: Vec::new(),
            secondary_data: std::collections::HashMap::new(),
            metadata: Box::new(String::from("metadata")),
        }
    }

    fn add_data(&mut self, primary: T, key: String, secondary: U) {
        self.primary_data.push(primary);
        self.secondary_data.insert(key, secondary);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Comprehensive Unsafe/FFI Demo");
    println!("================================");
    
    // Initialize memory tracking
    init();
    
    {
        // Create scope for tracked allocations
        println!("📝 Step 1: Creating complex allocations...");
        
        // Large allocation for demonstration
        let large_buffer: Vec<u64> = vec![0xDEADBEEF; 10000];
        track_var!(large_buffer);

        // Multiple small allocations with different types
        for i in 0..50 {
            let small_vec: Vec<u32> = vec![i; i as usize + 10];
            track_var!(small_vec);
        }

        // Smart pointer allocations
        let boxed_data: Box<[u8; 1024]> = Box::new([0xFF; 1024]);
        track_var!(boxed_data);

        let rc_data = std::rc::Rc::new(vec![1, 2, 3, 4, 5]);
        track_var!(rc_data);

        let arc_data = std::sync::Arc::new(HashMap::<String, i32>::new());
        track_var!(arc_data);

        println!("🔧 Step 2: Performing unsafe operations...");
        let mut processor = UnsafeDataProcessor::new();
        
        unsafe {
            // Allocate raw buffer
            processor.allocate_raw_buffer(8192)?;
            
            // Process raw data
            processor.process_raw_data()?;
        }

        println!("🌐 Step 3: FFI operations...");
        let test_strings = vec![
            "Hello, FFI World!",
            "Unsafe operations demo",
            "Memory tracking in action",
            "Complex data processing",
            "Rust to C interop example"
        ];
        
        let lengths = processor.process_c_strings(&test_strings)?;
        println!("   Processed {} strings with lengths: {:?}", lengths.len(), lengths);

        println!("🧩 Step 4: Complex type allocations...");
        
        // Create various complex types that can be tracked
        let string_map: HashMap<String, String> = (0..20)
            .map(|i| (format!("key_{}", i), format!("value_{}", i)))
            .collect();
        track_var!(string_map);
        
        let nested_vecs: Vec<Vec<f64>> = (0..15)
            .map(|i| vec![i as f64 * 3.14; i + 1])
            .collect();
        track_var!(nested_vecs);
        
        // More complex allocations
        let byte_arrays: Vec<Box<[u8]>> = (0..10)
            .map(|i| vec![i as u8; 100 * (i + 1)].into_boxed_slice())
            .collect();
        track_var!(byte_arrays);

        println!("⚡ Step 5: Concurrent operations...");
        let handles: Vec<_> = (0..5).map(|i| {
            std::thread::spawn(move || {
                let thread_data: Vec<usize> = (0..1000).map(|x| x * i).collect();
                // Note: track_var! doesn't work across thread boundaries in this simple example
                // In a real application, you'd use thread-safe tracking
                
                // Simulate some work
                std::thread::sleep(std::time::Duration::from_millis(10));
                
                thread_data.len()
            })
        }).collect();

        for handle in handles {
            let result = handle.join().unwrap();
            println!("   Thread completed with {} items", result);
        }

        println!("📊 Step 6: Memory analysis summary...");
        let tracker = get_global_tracker();
        if let Ok(stats) = tracker.get_stats() {
            println!("   Total allocations: {}", stats.total_allocations);
            println!("   Active memory: {} bytes", stats.active_memory);
            println!("   Peak memory: {} bytes", stats.peak_memory);
        }

    } // End of tracking scope

    println!("\n💾 Step 7: Exporting to binary format...");
    let binary_file = "MemoryAnalysis/comprehensive_demo.memscope";
    
    // Export to user-binary format
    let tracker = get_global_tracker();
    tracker.export_user_binary(binary_file)?;
    println!("   ✅ Exported to: {}", binary_file);

    println!("\n📄 Step 8: Converting binary to JSON...");
    let json_output_dir = "MemoryAnalysis/comprehensive_demo";
    
    // Convert binary to JSON
    memscope_rs::export::binary::export_binary_to_json(binary_file, "comprehensive_demo")?;
    println!("   ✅ JSON files created in: {}", json_output_dir);

    println!("\n🎨 Step 9: Generating HTML dashboard...");
    
    // Convert binary to HTML
    memscope_rs::export::binary::export_binary_to_html(binary_file, "comprehensive_demo")?;
    let html_file = format!("{}/comprehensive_demo_user_dashboard.html", json_output_dir);
    println!("   ✅ HTML dashboard created: {}", html_file);

    println!("\n🔍 Step 10: Verification...");
    
    // Verify files exist
    let binary_path = std::path::Path::new(binary_file);
    let html_path = std::path::Path::new(&html_file);
    
    if binary_path.exists() {
        let binary_size = std::fs::metadata(binary_path)?.len();
        println!("   ✅ Binary file: {} ({} bytes)", binary_file, binary_size);
    }
    
    if html_path.exists() {
        let html_size = std::fs::metadata(html_path)?.len();
        println!("   ✅ HTML file: {} ({} bytes)", html_file, html_size);
    }

    // List generated JSON files
    if let Ok(entries) = std::fs::read_dir(json_output_dir) {
        println!("   📋 Generated JSON files:");
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json") {
                    let size = std::fs::metadata(&path)?.len();
                    println!("      • {} ({} bytes)", path.file_name().unwrap().to_string_lossy(), size);
                }
            }
        }
    }

    println!("\n🎉 Demo completed successfully!");
    println!("📖 Open the HTML file to view the comprehensive analysis:");
    println!("   file://{}/{}", std::env::current_dir()?.display(), html_file);
    
    println!("\n📊 Expected analysis features:");
    println!("   • Unsafe/FFI operations detection");
    println!("   • Complex type analysis (generics, smart pointers)");
    println!("   • Memory timeline visualization");
    println!("   • Variable lifecycle tracking");
    println!("   • Thread-safe allocation analysis");
    println!("   • Security violation assessment");

    Ok(())
}