//! Advanced Binary Export Showcase
//!
//! This example demonstrates the binary export functionality with complex,
//! real-world scenarios including:
//! - Complex generic types and trait objects
//! - Unsafe code and FFI operations
//! - Async/await patterns
//! - Closures and function pointers
//! - Complex memory layouts
//! - Multi-threaded scenarios
//! - Performance-critical data structures

use memscope_rs::{get_global_tracker, track_var, Trackable};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::f64::consts::PI;
use std::ffi::CString;
use std::future::Future;
use std::hash::RandomState;
use std::os::raw::{c_char, c_int, c_void};
use std::pin::Pin;
use std::ptr;
use std::rc::Rc;
use std::slice;
use std::sync::{Arc, Mutex, RwLock, Weak};

// Complex trait definitions for trait objects
#[allow(dead_code)]
trait DataProcessor: Send + Sync {
    fn process(&self, data: &[u8]) -> Vec<u8>;
    fn get_name(&self) -> &str;
}

trait AsyncProcessor: Send + Sync {
    fn process_async(&self, data: Vec<u8>) -> Pin<Box<dyn Future<Output = Vec<u8>> + Send>>;
}

// Complex generic structures
#[derive(Debug)]
#[allow(dead_code)]
struct GenericCache<K, V, H>
where
    K: Clone + std::hash::Hash + Eq,
    V: Clone,
    H: std::hash::BuildHasher,
{
    data: HashMap<K, V, H>,
    metadata: BTreeMap<String, String>,
    access_log: VecDeque<(K, std::time::Instant)>,
    capacity: usize,
    hit_count: u64,
    miss_count: u64,
}

// Complex nested structure with various smart pointers
#[allow(dead_code)]
struct ComplexDataStructure {
    // Various smart pointer types
    shared_config: Arc<RwLock<HashMap<String, String>>>,
    local_cache: Rc<RefCell<Vec<String>>>,
    weak_reference: Weak<Mutex<i32>>,

    // Complex nested generics
    processing_queue: Arc<Mutex<VecDeque<Box<dyn DataProcessor>>>>,
    async_handlers: Vec<Box<dyn AsyncProcessor>>,

    // Function pointers and closures
    callback: Option<Box<dyn Fn(&str) -> String + Send + Sync>>,
    filter_fn: fn(&str) -> bool,

    // Unsafe data
    raw_buffer: *mut u8,
    buffer_size: usize,

    // FFI data
    c_string: CString,
    external_handle: *mut c_void,
}

// Implementation of trait objects
#[allow(dead_code)]
struct JsonProcessor {
    name: String,
    indent_size: usize,
}

impl DataProcessor for JsonProcessor {
    fn process(&self, data: &[u8]) -> Vec<u8> {
        format!(
            "{{\"processed_by\":\"{}\",\"data_size\":{},\"indent\":{}}}",
            self.name,
            data.len(),
            self.indent_size
        )
        .into_bytes()
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[allow(dead_code)]
struct BinaryProcessor {
    compression_level: u8,
}

impl DataProcessor for BinaryProcessor {
    fn process(&self, data: &[u8]) -> Vec<u8> {
        // Simulate compression
        let mut result = Vec::with_capacity(data.len() / 2);
        result.extend_from_slice(b"COMPRESSED:");
        result.extend_from_slice(&[self.compression_level]);
        result.extend_from_slice(&data[..std::cmp::min(data.len(), 100)]);
        result
    }

    fn get_name(&self) -> &str {
        "BinaryProcessor"
    }
}

// Async processor implementation
struct AsyncDataProcessor {
    delay_ms: u64,
}

impl AsyncProcessor for AsyncDataProcessor {
    fn process_async(&self, data: Vec<u8>) -> Pin<Box<dyn Future<Output = Vec<u8>> + Send>> {
        let delay = self.delay_ms;
        Box::pin(async move {
            // Simulate async processing
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            format!("ASYNC_PROCESSED:{}", data.len()).into_bytes()
        })
    }
}

// External C functions (simulated)
extern "C" {
    // These would normally be from a C library
    // For demo purposes, we'll implement them in Rust
}

// Simulated C functions
#[no_mangle]
pub extern "C" fn external_malloc(size: usize) -> *mut c_void {
    unsafe {
        let layout = std::alloc::Layout::from_size_align_unchecked(size, 8);
        std::alloc::alloc(layout) as *mut c_void
    }
}

#[no_mangle]
pub extern "C" fn external_free(ptr: *mut c_void) {
    unsafe {
        if !ptr.is_null() {
            let layout = std::alloc::Layout::from_size_align_unchecked(1024, 8); // Assume size
            std::alloc::dealloc(ptr as *mut u8, layout);
        }
    }
}

#[no_mangle]
pub extern "C" fn process_data_c(data: *const c_char, len: c_int) -> *mut c_char {
    unsafe {
        if data.is_null() || len <= 0 {
            return ptr::null_mut();
        }

        let input = slice::from_raw_parts(data as *const u8, len as usize);
        let processed = format!("C_PROCESSED:{}", input.len());
        let c_string = CString::new(processed).unwrap();
        c_string.into_raw()
    }
}

#[allow(dead_code)]
impl<K, V, H> GenericCache<K, V, H>
where
    K: Clone + std::hash::Hash + Eq,
    V: Clone,
    H: std::hash::BuildHasher,
{
    fn new(capacity: usize, hasher: H) -> Self {
        Self {
            data: HashMap::with_hasher(hasher),
            metadata: BTreeMap::new(),
            access_log: VecDeque::new(),
            capacity,
            hit_count: 0,
            miss_count: 0,
        }
    }

    fn insert(&mut self, key: K, value: V) {
        if self.data.len() >= self.capacity {
            // Remove oldest entry
            if let Some((old_key, _)) = self.access_log.pop_front() {
                self.data.remove(&old_key);
            }
        }

        self.data.insert(key.clone(), value);
        self.access_log.push_back((key, std::time::Instant::now()));
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(value) = self.data.get(key) {
            self.hit_count += 1;
            Some(value)
        } else {
            self.miss_count += 1;
            None
        }
    }
}

unsafe fn create_unsafe_buffer(size: usize) -> (*mut u8, usize) {
    let layout = std::alloc::Layout::from_size_align_unchecked(size, 8);
    let ptr = std::alloc::alloc(layout);
    if ptr.is_null() {
        panic!("Failed to allocate memory");
    }

    // Initialize with pattern
    for i in 0..size {
        *ptr.add(i) = (i % 256) as u8;
    }

    (ptr, size)
}

unsafe fn cleanup_unsafe_buffer(ptr: *mut u8, size: usize) {
    if !ptr.is_null() {
        let layout = std::alloc::Layout::from_size_align_unchecked(size, 8);
        std::alloc::dealloc(ptr, layout);
    }
}

fn create_complex_closures() -> Vec<Box<dyn Fn(i32) -> i32 + Send + Sync>> {
    let mut closures: Vec<Box<dyn Fn(i32) -> i32 + Send + Sync>> = Vec::new();

    // Closure with captured environment
    let multiplier = 42;
    let multiply_closure: Box<dyn Fn(i32) -> i32 + Send + Sync> =
        Box::new(move |x: i32| x * multiplier);
    closures.push(multiply_closure);

    // Closure with complex logic
    let complex_closure: Box<dyn Fn(i32) -> i32 + Send + Sync> = Box::new(|x: i32| {
        let mut result = x;
        for i in 1..=5 {
            result = result.wrapping_mul(i).wrapping_add(i * i);
        }
        result
    });
    closures.push(complex_closure);

    closures
}

// Implement Trackable for our custom types
impl<K, V, H> Trackable for GenericCache<K, V, H>
where
    K: Clone + std::hash::Hash + Eq,
    V: Clone,
    H: std::hash::BuildHasher,
{
    fn get_heap_ptr(&self) -> Option<usize> {
        // Use the HashMap's heap pointer as our base
        if let Some(map_ptr) = self.data.get_heap_ptr() {
            Some(0xF600_0000 + (map_ptr % 0x0FFF_FFFF))
        } else {
            Some(0xF600_0000 + (self as *const _ as usize % 0x0FFF_FFFF))
        }
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<GenericCache<K, V, H>>()
    }

    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.data.get_size_estimate()
            + self.metadata.len() * (std::mem::size_of::<String>() * 2)
            + self.access_log.len() * std::mem::size_of::<(K, std::time::Instant)>()
    }
}

impl Trackable for ComplexDataStructure {
    fn get_heap_ptr(&self) -> Option<usize> {
        // Use the shared_config Arc as our primary identifier
        if let Some(config_ptr) = self.shared_config.get_heap_ptr() {
            Some(0xF700_0000 + (config_ptr % 0x0FFF_FFFF))
        } else {
            Some(0xF700_0000 + (self as *const _ as usize % 0x0FFF_FFFF))
        }
    }

    fn get_type_name(&self) -> &'static str {
        "ComplexDataStructure"
    }

    fn get_size_estimate(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.shared_config.get_size_estimate()
            + self.local_cache.get_size_estimate()
            + self.processing_queue.get_size_estimate()
            + self.async_handlers.len() * std::mem::size_of::<Box<dyn AsyncProcessor>>()
            + self.buffer_size
            + self.c_string.as_bytes().len()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced Binary Export Showcase");
    println!("===================================");
    println!("Demonstrating binary export with:");
    println!("• Complex generic types and trait objects");
    println!("• Unsafe code and FFI operations");
    println!("• Async/await patterns");
    println!("• Closures and function pointers");
    println!("• Multi-threaded scenarios");
    println!("• Performance-critical data structures");
    println!();

    let tracker = get_global_tracker();

    // Create output directory
    let output_dir = std::path::Path::new("./MemoryAnalysis/advanced_binary_showcase");
    std::fs::create_dir_all(output_dir)?;
    println!("📁 Created output directory: {}", output_dir.display());

    println!("\n🔧 Creating complex data structures...");

    // 1. Complex Generic Cache
    let mut string_cache = GenericCache::new(100, RandomState::new());
    for i in 0..50 {
        string_cache.insert(
            format!("key_{}", i),
            format!("value_data_{}_with_long_content", i),
        );
    }
    let _tracked_string_cache = track_var!(string_cache);

    // 2. Multi-level nested smart pointers
    let shared_config = Arc::new(RwLock::new({
        let mut config = HashMap::new();
        config.insert(
            "database_url".to_string(),
            "postgresql://localhost:5432/mydb".to_string(),
        );
        config.insert(
            "redis_url".to_string(),
            "redis://localhost:6379".to_string(),
        );
        config.insert("api_key".to_string(), "sk-1234567890abcdef".to_string());
        config
    }));
    let _tracked_shared_config = track_var!(shared_config);

    // 3. Complex trait object collections
    let mut processors: Vec<Box<dyn DataProcessor>> = Vec::new();
    processors.push(Box::new(JsonProcessor {
        name: "PrimaryJsonProcessor".to_string(),
        indent_size: 4,
    }));
    processors.push(Box::new(JsonProcessor {
        name: "CompactJsonProcessor".to_string(),
        indent_size: 0,
    }));
    processors.push(Box::new(BinaryProcessor {
        compression_level: 9,
    }));
    processors.push(Box::new(BinaryProcessor {
        compression_level: 1,
    }));
    let _tracked_processors = track_var!(processors);

    // 4. Async processors
    let async_processors: Vec<Box<dyn AsyncProcessor>> = vec![
        Box::new(AsyncDataProcessor { delay_ms: 10 }),
        Box::new(AsyncDataProcessor { delay_ms: 50 }),
        Box::new(AsyncDataProcessor { delay_ms: 100 }),
    ];
    let _tracked_async_processors = track_var!(async_processors);

    // 5. Complex nested structure with various pointer types
    let strong_ref = Arc::new(Mutex::new(42i32));
    let weak_ref = Arc::downgrade(&strong_ref);

    let local_cache = Rc::new(RefCell::new(vec![
        "cached_item_1".to_string(),
        "cached_item_2_with_longer_content".to_string(),
        "cached_item_3_with_even_more_detailed_information".to_string(),
    ]));

    // 6. Unsafe memory operations
    let (raw_buffer, buffer_size) = unsafe { create_unsafe_buffer(8192) };

    // 7. FFI operations
    let c_string = CString::new("Hello from Rust to C interface!")?;
    let external_handle = external_malloc(1024);

    // Process some data through C
    let test_data = CString::new("Test data for C processing")?;
    let processed_c_data = process_data_c(test_data.as_ptr(), test_data.as_bytes().len() as c_int);

    let complex_structure = ComplexDataStructure {
        shared_config: Arc::new(RwLock::new({
            let mut map = HashMap::new();
            map.insert("thread_pool_size".to_string(), "16".to_string());
            map.insert("max_connections".to_string(), "1000".to_string());
            map
        })),
        local_cache: local_cache.clone(),
        weak_reference: weak_ref,
        processing_queue: Arc::new(Mutex::new(VecDeque::new())),
        async_handlers: async_processors,
        callback: Some(Box::new(|input: &str| {
            format!("CALLBACK_PROCESSED: {}", input.to_uppercase())
        })),
        filter_fn: |s: &str| s.len() > 5,
        raw_buffer,
        buffer_size,
        c_string,
        external_handle,
    };
    let _tracked_complex_structure = track_var!(complex_structure);

    // 8. Complex closures with captured environments
    let closures = create_complex_closures();
    let _tracked_closures = track_var!(closures);

    // 9. Multi-threaded data structures
    let thread_safe_counter = Arc::new(Mutex::new(0u64));
    let mut handles = Vec::new();

    for i in 0..4 {
        let counter = Arc::clone(&thread_safe_counter);
        let handle = std::thread::spawn(move || {
            for j in 0..1000 {
                let mut num = counter.lock().unwrap();
                *num += (i * 1000 + j) as u64;
            }
        });
        handles.push(handle);
    }

    // Wait for threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    let _tracked_thread_safe_counter = track_var!(thread_safe_counter);

    // 10. Performance-critical data structures
    let mut performance_map: HashMap<String, Vec<f64>> = HashMap::with_capacity(1000);
    for i in 0..500 {
        let key = format!("metric_{}_{}", i / 100, i % 100);
        let values: Vec<f64> = (0..100).map(|j| (i as f64 * PI + j as f64).sin()).collect();
        performance_map.insert(key, values);
    }
    let _tracked_performance_map = track_var!(performance_map);

    // 11. Complex async operations
    println!("\n⚡ Running async operations...");
    let async_results = tokio::join!(
        async {
            let data = vec![1u8, 2, 3, 4, 5];
            let processor = AsyncDataProcessor { delay_ms: 20 };
            processor.process_async(data).await
        },
        async {
            let data = vec![10u8, 20, 30, 40, 50];
            let processor = AsyncDataProcessor { delay_ms: 30 };
            processor.process_async(data).await
        },
        async {
            tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
            vec![99u8, 98, 97]
        }
    );
    // Track individual results
    let _tracked_result1 = track_var!(async_results.0);
    let _tracked_result2 = track_var!(async_results.1);
    let _tracked_result3 = track_var!(async_results.2);

    println!("✅ Created complex data structures with:");
    println!("  • Generic cache with {} entries", 50);
    println!("  • {} trait object processors", 4);
    println!("  • {} async processors", 3);
    println!("  • Multi-threaded counter operations");
    println!("  • {} performance metrics", 500);
    println!("  • Unsafe buffer of {} bytes", buffer_size);
    println!("  • FFI operations with C interface");

    // Add some deallocations to show complete lifecycle
    std::thread::sleep(std::time::Duration::from_millis(50));
    let _ = _tracked_string_cache;
    let _ = _tracked_performance_map;

    println!("✅ Simulated some deallocations for lifecycle analysis");

    // Export to binary format
    println!("\n💾 Exporting to binary format...");
    let start_time = std::time::Instant::now();
    tracker.export_to_binary("advanced_binary_showcase")?;
    let binary_export_time = start_time.elapsed();

    // Find the created binary file
    let binary_file = find_binary_file("MemoryAnalysis")?;
    let binary_size = std::fs::metadata(&binary_file)?.len();

    println!("✅ Binary export completed in {:?}", binary_export_time);
    println!(
        "📁 Binary file: {} ({} bytes)",
        binary_file.display(),
        binary_size
    );

    // Convert binary to standard JSON files
    println!("\n🔄 Converting binary to standard JSON files...");
    let start_time = std::time::Instant::now();
    memscope_rs::core::tracker::MemoryTracker::parse_binary_to_standard_json(
        &binary_file,
        "advanced_binary_showcase",
    )?;
    let json_conversion_time = start_time.elapsed();

    // Convert binary to HTML report
    println!("\n🌐 Converting binary to HTML report...");
    let html_file = output_dir.join("advanced_binary_showcase.html");
    let start_time = std::time::Instant::now();
    memscope_rs::core::tracker::MemoryTracker::parse_binary_to_html(&binary_file, &html_file)?;
    let html_conversion_time = start_time.elapsed();

    let _html_size = std::fs::metadata(&html_file)?.len();

    // Performance comparison with direct JSON export
    println!("\n📊 Performance Analysis:");
    println!("========================");

    let start_time = std::time::Instant::now();
    tracker.export_to_json("advanced_binary_direct")?;
    let json_direct_time = start_time.elapsed();

    // Calculate file sizes
    let json_files = [
        "advanced_binary_showcase_memory_analysis.json",
        "advanced_binary_showcase_lifetime.json",
        "advanced_binary_showcase_performance.json",
        "advanced_binary_showcase_unsafe_ffi.json",
        "advanced_binary_showcase_complex_types.json",
    ];

    let mut total_json_size = 0;
    println!("✅ Generated JSON files:");
    for json_file_name in &json_files {
        let json_file_path = output_dir.join(json_file_name);
        if json_file_path.exists() {
            let size = std::fs::metadata(&json_file_path)?.len();
            total_json_size += size;
            println!("  • {} ({} bytes)", json_file_name, size);
        }
    }

    // Calculate performance metrics
    let size_reduction =
        ((total_json_size as f64 - binary_size as f64) / total_json_size as f64) * 100.0;
    let speed_improvement =
        json_direct_time.as_nanos() as f64 / binary_export_time.as_nanos() as f64;

    println!("\nAdvanced Binary Export Performance:");
    println!("  📊 Binary export time:     {:?}", binary_export_time);
    println!("  📊 Standard JSON time:     {:?}", json_direct_time);
    println!(
        "  🚀 Speed improvement:      {:.2}x faster",
        speed_improvement
    );
    println!("  📁 Binary file size:       {} bytes", binary_size);
    println!(
        "  📁 JSON files size:        {} bytes (5 files)",
        total_json_size
    );
    println!("  💾 Size reduction:         {:.1}%", size_reduction);

    println!("\nConversion Performance:");
    println!("  🔄 Binary → JSON files:    {:?}", json_conversion_time);
    println!("  🌐 Binary → HTML:          {:?}", html_conversion_time);

    println!("\n🎉 Advanced showcase completed successfully!");
    println!("📁 All files generated in: {}", output_dir.display());
    println!("\n💡 Key Features Demonstrated:");
    println!("  ✅ Complex generic types with multiple type parameters");
    println!("  ✅ Trait objects with dynamic dispatch");
    println!("  ✅ Smart pointers (Arc, Rc, Weak, Box)");
    println!("  ✅ Unsafe memory operations and raw pointers");
    println!("  ✅ FFI operations with C interface");
    println!("  ✅ Async/await patterns and futures");
    println!("  ✅ Closures with captured environments");
    println!("  ✅ Multi-threaded data structures");
    println!("  ✅ Performance-critical collections");
    println!("  ✅ Complex memory layouts and lifetimes");

    // Cleanup unsafe resources
    unsafe {
        cleanup_unsafe_buffer(raw_buffer, buffer_size);
        if !external_handle.is_null() {
            external_free(external_handle);
        }
        if !processed_c_data.is_null() {
            let _ = CString::from_raw(processed_c_data);
        }
    }

    println!("\n🧹 Cleaned up unsafe resources");

    Ok(())
}

/// Find the binary file in the MemoryAnalysis directory
fn find_binary_file(base_dir: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let memory_analysis_dir = std::path::Path::new(base_dir);

    if !memory_analysis_dir.exists() {
        return Err("MemoryAnalysis directory not found".into());
    }

    // Look for .memscope files
    for entry in std::fs::read_dir(memory_analysis_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            for sub_entry in std::fs::read_dir(entry.path())? {
                let sub_entry = sub_entry?;
                if sub_entry.path().extension() == Some(std::ffi::OsStr::new("memscope")) {
                    return Ok(sub_entry.path());
                }
            }
        }
    }

    Err("No .memscope file found".into())
}
