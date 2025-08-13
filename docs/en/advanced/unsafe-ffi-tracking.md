# Unsafe and FFI Tracking

Track and analyze memory usage in unsafe code blocks and FFI calls in Rust programs.

## 🎯 Objectives

- Track unsafe code blocks
- Monitor FFI calls
- Detect memory safety issues
- Analyze C/C++ interoperability

## ⚠️ Unsafe Code Tracking

### Basic Unsafe Tracking

```rust
use memscope_rs::{init, track_var, track_unsafe_operation};

fn main() {
    init();
    
    // Track unsafe operations
    unsafe {
        track_unsafe_operation("raw_pointer_deref", "main.rs:10");
        
        let ptr = Box::into_raw(Box::new(42));
        let value = *ptr;
        track_var!(value);
        
        // Remember to free memory
        let _ = Box::from_raw(ptr);
    }
}
```

### Unsafe Operation Classification

```rust
use memscope_rs::UnsafeOperationType;

fn track_various_unsafe_operations() {
    // 1. Raw pointer dereference
    unsafe {
        let ptr = 0x1000 as *const i32;
        track_unsafe_operation(
            UnsafeOperationType::RawPointerDereference,
            "Dereferencing raw pointer",
            Some("main.rs:25".to_string())
        );
        // let value = *ptr; // Dangerous operation
    }
    
    // 2. Mutable static variable access
    static mut COUNTER: i32 = 0;
    unsafe {
        track_unsafe_operation(
            UnsafeOperationType::StaticMutAccess,
            "Accessing mutable static",
            Some("main.rs:35".to_string())
        );
        COUNTER += 1;
    }
    
    // 3. Union field access
    union MyUnion {
        i: i32,
        f: f32,
    }
    
    let u = MyUnion { i: 42 };
    unsafe {
        track_unsafe_operation(
            UnsafeOperationType::UnionFieldAccess,
            "Accessing union field",
            Some("main.rs:50".to_string())
        );
        let value = u.f;
        track_var!(value);
    }
}
```

## 🔗 FFI Call Tracking

### C Library Integration

```rust
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int, c_void};

// External C function declarations
extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn strlen(s: *const c_char) -> usize;
}

fn track_c_memory_operations() {
    memscope_rs::init();
    
    unsafe {
        // Track C memory allocation
        track_unsafe_operation(
            UnsafeOperationType::FFICall,
            "malloc call",
            Some("ffi.rs:15".to_string())
        );
        
        let ptr = malloc(1024);
        if !ptr.is_null() {
            // Track allocated memory
            let allocated_memory = std::slice::from_raw_parts_mut(
                ptr as *mut u8, 1024
            );
            track_var!(allocated_memory);
            
            // Use memory...
            
            // Free memory
            track_unsafe_operation(
                UnsafeOperationType::FFICall,
                "free call",
                Some("ffi.rs:30".to_string())
            );
            free(ptr);
        }
    }
}
```

### Complex FFI Scenarios

```rust
// Simulate complex C struct
#[repr(C)]
struct CStruct {
    data: *mut u8,
    size: usize,
    capacity: usize,
}

extern "C" {
    fn create_c_struct(size: usize) -> *mut CStruct;
    fn destroy_c_struct(s: *mut CStruct);
    fn c_struct_add_data(s: *mut CStruct, data: *const u8, len: usize) -> c_int;
}

fn complex_ffi_tracking() {
    memscope_rs::init();
    
    unsafe {
        // Create C struct
        track_unsafe_operation(
            UnsafeOperationType::FFICall,
            "create_c_struct",
            Some("complex_ffi.rs:10".to_string())
        );
        
        let c_struct = create_c_struct(1024);
        if !c_struct.is_null() {
            track_var!(c_struct);
            
            // Add data to C struct
            let rust_data = b"Hello from Rust!";
            track_unsafe_operation(
                UnsafeOperationType::FFICall,
                "c_struct_add_data",
                Some("complex_ffi.rs:20".to_string())
            );
            
            let result = c_struct_add_data(
                c_struct,
                rust_data.as_ptr(),
                rust_data.len()
            );
            
            if result == 0 {
                println!("Data added successfully");
            }
            
            // Cleanup
            track_unsafe_operation(
                UnsafeOperationType::FFICall,
                "destroy_c_struct",
                Some("complex_ffi.rs:35".to_string())
            );
            destroy_c_struct(c_struct);
        }
    }
}
```

## 🛡️ Safety Analysis

### Memory Safety Checks

```rust
use memscope_rs::analysis::UnsafeFFITracker;

fn analyze_memory_safety() {
    let mut tracker = UnsafeFFITracker::new();
    
    // Simulate some unsafe operations
    unsafe {
        let ptr = Box::into_raw(Box::new(42));
        
        // Record operation
        tracker.track_unsafe_operation(
            UnsafeOperationType::RawPointerDereference,
            "main.rs:100",
            Some("Potential use-after-free".to_string())
        ).unwrap();
        
        // Check safety violations
        let violations = tracker.get_safety_violations();
        for violation in violations {
            println!("Safety violation: {:?}", violation);
        }
        
        // Cleanup
        let _ = Box::from_raw(ptr);
    }
}
```

### FFI Boundary Analysis

```rust
fn analyze_ffi_boundaries() {
    // Analyze data passing between Rust and C
    let rust_string = String::from("Hello, FFI!");
    let c_string = CString::new(rust_string.clone()).unwrap();
    
    track_var!(rust_string);
    track_var!(c_string);
    
    unsafe {
        // Pass to C function
        let c_ptr = c_string.as_ptr();
        track_unsafe_operation(
            UnsafeOperationType::FFICall,
            "strlen",
            Some("Passing Rust string to C".to_string())
        );
        
        let len = strlen(c_ptr);
        println!("C function returned length: {}", len);
    }
}
```

## 📊 Analysis Reports

### Generate Safety Reports

```rust
fn generate_safety_report() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = memscope_rs::get_global_tracker();
    
    // Export report with unsafe/FFI information
    tracker.export_to_json("unsafe_ffi_analysis")?;
    
    // Generate specialized safety report
    let unsafe_tracker = UnsafeFFITracker::new();
    let safety_report = unsafe_tracker.generate_safety_report();
    
    println!("Safety Report:");
    println!("  Unsafe operations: {}", safety_report.unsafe_operations_count);
    println!("  FFI calls: {}", safety_report.ffi_calls_count);
    println!("  Safety violations: {}", safety_report.safety_violations_count);
    
    Ok(())
}
```

## 🔧 Best Practices

### 1. Minimize Unsafe Usage

```rust
// ❌ Avoid unnecessary unsafe
unsafe fn unnecessary_unsafe(data: &[i32]) -> i32 {
    data[0] // This doesn't need unsafe
}

// ✅ Use unsafe only when necessary
fn safe_access(data: &[i32]) -> Option<i32> {
    data.get(0).copied()
}
```

### 2. Encapsulate Unsafe Operations

```rust
pub struct SafeWrapper {
    inner: *mut CStruct,
}

impl SafeWrapper {
    pub fn new(size: usize) -> Option<Self> {
        unsafe {
            let ptr = create_c_struct(size);
            if ptr.is_null() {
                None
            } else {
                Some(Self { inner: ptr })
            }
        }
    }
    
    pub fn add_data(&mut self, data: &[u8]) -> Result<(), &'static str> {
        unsafe {
            let result = c_struct_add_data(
                self.inner,
                data.as_ptr(),
                data.len()
            );
            if result == 0 {
                Ok(())
            } else {
                Err("Failed to add data")
            }
        }
    }
}

impl Drop for SafeWrapper {
    fn drop(&mut self) {
        unsafe {
            destroy_c_struct(self.inner);
        }
    }
}
```

## 🎉 Summary

Unsafe and FFI tracking helps you:
- Identify potential safety issues
- Monitor cross-language boundaries
- Analyze memory safety violations
- Optimize FFI performance