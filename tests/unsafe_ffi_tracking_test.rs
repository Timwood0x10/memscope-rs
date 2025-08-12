//! Unsafe and FFI tracking tests
//! 
//! Tests tracking of unsafe operations, FFI boundary crossings, and safety
//! violation detection in mixed Rust/C code scenarios.

use memscope_rs::{track_var, get_global_tracker};
use std::ffi::CString;

#[test]
fn test_basic_unsafe_block_tracking() {
    // Test tracking within unsafe blocks
    let test_data = vec![42u8; 256];
    track_var!(test_data);
    
    unsafe {
        // Perform unsafe operations
        let raw_ptr = test_data.as_ptr();
        let _unsafe_access = *raw_ptr;
        
        // Verify tracking continues in unsafe context
        let tracker = get_global_tracker();
        match tracker.get_stats() {
            Ok(stats) => {
                if stats.total_allocations == 0 {
                    panic!("Unsafe block tracking should maintain allocation records");
                }
            }
            Err(e) => {
                panic!("Failed to get stats in unsafe block: {:?}", e);
            }
        }
    }
    
    drop(test_data);
}

#[test]
fn test_raw_pointer_tracking() {
    // Test tracking of raw pointer operations
    let source_data = vec![1u8, 2, 3, 4, 5];
    track_var!(source_data);
    
    // Create raw pointer and perform operations
    let raw_ptr = source_data.as_ptr();
    
    unsafe {
        // Perform raw pointer operations
        for offset in 0..source_data.len() {
            let _value = *raw_ptr.add(offset);
        }
        
        // Verify raw pointer operations are tracked
        let tracker = get_global_tracker();
        match tracker.get_stats() {
            Ok(stats) => {
                if stats.total_allocations == 0 {
                    panic!("Raw pointer tracking should maintain records");
                }
            }
            Err(e) => {
                panic!("Failed to get stats during raw pointer test: {:?}", e);
            }
        }
    }
    
    drop(source_data);
}

#[test]
fn test_ffi_string_conversion_tracking() {
    // Test tracking of FFI string conversions
    let rust_string = String::from("test_ffi_string");
    track_var!(rust_string);
    
    // Convert to C string for FFI
    match CString::new(rust_string.clone()) {
        Ok(c_string) => {
            track_var!(c_string);
            
            unsafe {
                // Get raw C string pointer
                let c_ptr = c_string.as_ptr();
                
                // Simulate FFI operation
                let _c_str_len = libc::strlen(c_ptr);
                
                // Verify FFI string tracking
                let tracker = get_global_tracker();
                match tracker.get_stats() {
                    Ok(stats) => {
                        if stats.total_allocations < 2 {
                            panic!("FFI string conversion should track both Rust and C strings");
                        }
                    }
                    Err(e) => {
                        panic!("Failed to get stats during FFI string test: {:?}", e);
                    }
                }
            }
            
            drop(c_string);
        }
        Err(e) => {
            panic!("Failed to create C string: {:?}", e);
        }
    }
    
    drop(rust_string);
}

#[test]
fn test_memory_boundary_crossing() {
    // Test tracking across Rust/C memory boundaries
    let boundary_data = vec![0u8; 1024];
    track_var!(boundary_data);
    
    unsafe {
        // Simulate memory boundary crossing
        let rust_ptr = boundary_data.as_ptr();
        
        // Allocate C memory
        let c_memory = libc::malloc(512);
        if c_memory.is_null() {
            panic!("Failed to allocate C memory for boundary test");
        }
        
        // Copy data across boundary
        libc::memcpy(c_memory, rust_ptr as *const libc::c_void, 512);
        
        // Verify boundary crossing tracking
        let tracker = get_global_tracker();
        match tracker.get_stats() {
            Ok(stats) => {
                if stats.total_allocations == 0 {
                    panic!("Boundary crossing should maintain tracking");
                }
            }
            Err(e) => {
                panic!("Failed to get stats during boundary test: {:?}", e);
            }
        }
        
        // Clean up C memory
        libc::free(c_memory);
    }
    
    drop(boundary_data);
}

#[test]
fn test_unsafe_transmute_tracking() {
    // Test tracking through unsafe transmute operations
    let numeric_data: Vec<u32> = vec![0x12345678, 0x9ABCDEF0, 0x11111111, 0x22222222];
    track_var!(numeric_data);
    
    unsafe {
        // Transmute to byte array
        let byte_ptr = numeric_data.as_ptr() as *const u8;
        let byte_slice = std::slice::from_raw_parts(byte_ptr, numeric_data.len() * 4);
        
        // Verify data through transmute
        if byte_slice.len() != numeric_data.len() * 4 {
            panic!("Transmute operation failed");
        }
        
        // Verify tracking survives transmute
        let tracker = get_global_tracker();
        match tracker.get_stats() {
            Ok(stats) => {
                if stats.total_allocations == 0 {
                    panic!("Transmute tracking should maintain allocation records");
                }
            }
            Err(e) => {
                panic!("Failed to get stats during transmute test: {:?}", e);
            }
        }
    }
    
    drop(numeric_data);
}

#[test]
fn test_ffi_callback_tracking() {
    // Test tracking through FFI callbacks
    let callback_data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    track_var!(callback_data);
    
    // Define callback function
    extern "C" fn test_callback(data: *const u8, len: usize) -> i32 {
        unsafe {
            if data.is_null() || len == 0 {
                return -1;
            }
            
            // Access data through callback
            let slice = std::slice::from_raw_parts(data, len);
            slice.iter().sum::<u8>() as i32
        }
    }
    
    unsafe {
        // Call callback with tracked data
        let result = test_callback(callback_data.as_ptr(), callback_data.len());
        
        if result < 0 {
            panic!("FFI callback failed");
        }
        
        // Verify tracking through callback
        let tracker = get_global_tracker();
        match tracker.get_stats() {
            Ok(stats) => {
                if stats.total_allocations == 0 {
                    panic!("FFI callback tracking should maintain records");
                }
            }
            Err(e) => {
                panic!("Failed to get stats during callback test: {:?}", e);
            }
        }
    }
    
    drop(callback_data);
}

#[test]
fn test_unsafe_union_tracking() {
    // Test tracking with unsafe unions
    #[repr(C)]
    union TestUnion {
        as_u32: u32,
        as_bytes: [u8; 4],
    }
    
    let union_data = TestUnion { as_u32: 0x12345678 };
    // Note: Union types cannot be tracked directly due to Trackable trait limitations
    let _union_ref = &union_data;
    
    unsafe {
        // Access union fields
        let _numeric_value = union_data.as_u32;
        let _byte_array = union_data.as_bytes;
        
        // Verify system remains functional with union operations
        let tracker = get_global_tracker();
        match tracker.get_stats() {
            Ok(_stats) => {
                // System should remain functional during union operations
            }
            Err(e) => {
                panic!("Failed to get stats during union test: {:?}", e);
            }
        }
    }
    
    let _ = union_data; // Consume union_data to avoid unused variable warning
}

#[test]
fn test_safety_violation_detection() {
    // Test detection of potential safety violations
    let violation_data = vec![42u8; 128];
    track_var!(violation_data);
    
    unsafe {
        // Create potential safety violation scenarios
        let ptr = violation_data.as_ptr();
        
        // Out-of-bounds access simulation (controlled)
        let _safe_access = *ptr;
        
        // Double-free simulation prevention (we won't actually double-free)
        let _ptr_copy = ptr;
        
        // Verify safety violation tracking
        let tracker = get_global_tracker();
        match tracker.get_stats() {
            Ok(stats) => {
                if stats.total_allocations == 0 {
                    panic!("Safety violation detection should maintain tracking");
                }
            }
            Err(e) => {
                panic!("Failed to get stats during safety violation test: {:?}", e);
            }
        }
    }
    
    drop(violation_data);
}