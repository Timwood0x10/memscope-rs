//! Type Inference Demo
//!
//! Demonstrates the unsafe type inference engine capabilities.

use memscope_rs::analysis::unsafe_inference::UnsafeInferenceEngine;

fn main() {
    println!("=== MemScope Type Inference Engine Demo ===\n");

    demo_vec_detection();
    demo_string_detection();
    demo_cstring_detection();
    demo_pointer_detection();
    demo_buffer_detection();
    demo_cstruct_detection();
    demo_stack_trace_boost();
    demo_lifetime_analysis();
    demo_real_world_examples();

    println!("\n=== Demo Complete ===");
}

fn demo_vec_detection() {
    println!("--- Vec Detection ---");

    // Simulate Vec<i32> memory layout: (ptr, len, cap)
    let ptr: usize = 0x10000;
    let len: usize = 100;
    let cap: usize = 128; // Power of two, larger than len

    let mut memory = vec![0u8; 24];
    memory[..8].copy_from_slice(&ptr.to_le_bytes());
    memory[8..16].copy_from_slice(&len.to_le_bytes());
    memory[16..24].copy_from_slice(&cap.to_le_bytes());

    let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);
    println!("  Vec layout (ptr=0x{:x}, len={}, cap={})", ptr, len, cap);
    println!(
        "  Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );
    println!("  Method: {:?}\n", guess.method);
}

fn demo_string_detection() {
    println!("--- String Detection ---");

    // Simulate String memory layout: (ptr, len, cap) with small spare
    let ptr: usize = 0x20000;
    let len: usize = 13;
    let cap: usize = 15; // Small spare capacity

    let mut memory = vec![0u8; 24];
    memory[..8].copy_from_slice(&ptr.to_le_bytes());
    memory[8..16].copy_from_slice(&len.to_le_bytes());
    memory[16..24].copy_from_slice(&cap.to_le_bytes());

    let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);
    println!(
        "  String layout (ptr=0x{:x}, len={}, cap={})",
        ptr, len, cap
    );
    println!(
        "  Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );
    println!("  Method: {:?}\n", guess.method);

    // UTF-8 content detection
    let text = b"Hello, World! This is a test string.";
    let guess = UnsafeInferenceEngine::infer_from_bytes(text, text.len());
    println!("  UTF-8 content: \"{}\"", String::from_utf8_lossy(text));
    println!(
        "  Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );
    println!("  Method: {:?}\n", guess.method);
}

fn demo_cstring_detection() {
    println!("--- CString Detection ---");

    // CString: null-terminated string
    let cstr = b"Hello, C World!\0";
    let guess = UnsafeInferenceEngine::infer_from_bytes(cstr, cstr.len());
    println!(
        "  CString content: \"{}\"",
        String::from_utf8_lossy(&cstr[..cstr.len() - 1])
    );
    println!(
        "  Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );
    println!("  Method: {:?}\n", guess.method);
}

fn demo_pointer_detection() {
    println!("--- Pointer Detection ---");

    // 8-byte pointer
    let ptr: usize = 0x7fff_1234_5678;
    let memory = ptr.to_le_bytes().to_vec();
    let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 8);
    println!("  Pointer value: 0x{:x}", ptr);
    println!(
        "  Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );
    println!("  Method: {:?}\n", guess.method);

    // Fat pointer (16 bytes)
    let metadata: usize = 100;
    let mut fat_ptr_memory = vec![0u8; 16];
    fat_ptr_memory[..8].copy_from_slice(&ptr.to_le_bytes());
    fat_ptr_memory[8..16].copy_from_slice(&metadata.to_le_bytes());
    let guess = UnsafeInferenceEngine::infer_from_bytes(&fat_ptr_memory, 16);
    println!("  Fat pointer (ptr + metadata)");
    println!(
        "  Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );
    println!();
}

fn demo_buffer_detection() {
    println!("--- Buffer Detection ---");

    // High entropy binary data
    let binary: Vec<u8> = (0..=255).collect();
    let guess = UnsafeInferenceEngine::infer_from_bytes(&binary, binary.len());
    println!("  Binary data (0-255, {} bytes)", binary.len());
    println!(
        "  Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );
    println!();

    // Zero-filled buffer
    let zeros = vec![0u8; 1024];
    let guess = UnsafeInferenceEngine::infer_from_bytes(&zeros, 1024);
    println!("  Zero-filled buffer (1024 bytes)");
    println!(
        "  Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );
    println!();
}

fn demo_cstruct_detection() {
    println!("--- CStruct Detection ---");

    // Struct with multiple pointers
    let ptr1: usize = 0x10000;
    let ptr2: usize = 0x20000;
    let value: u64 = 42;

    let mut memory = vec![0u8; 24];
    memory[..8].copy_from_slice(&ptr1.to_le_bytes());
    memory[8..16].copy_from_slice(&ptr2.to_le_bytes());
    memory[16..24].copy_from_slice(&value.to_le_bytes());

    let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);
    println!(
        "  CStruct (ptr1=0x{:x}, ptr2=0x{:x}, value={})",
        ptr1, ptr2, value
    );
    println!(
        "  Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );
    println!();
}

fn demo_stack_trace_boost() {
    println!("--- Stack Trace Boost ---");

    // Vec layout without stack trace
    let ptr: usize = 0x10000;
    let len: usize = 50;
    let cap: usize = 64;

    let mut memory = vec![0u8; 24];
    memory[..8].copy_from_slice(&ptr.to_le_bytes());
    memory[8..16].copy_from_slice(&len.to_le_bytes());
    memory[16..24].copy_from_slice(&cap.to_le_bytes());

    let guess_no_stack = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);
    println!("  Without stack trace:");
    println!(
        "    Result: {} (confidence: {}%)",
        guess_no_stack.kind, guess_no_stack.confidence
    );

    // With stack trace
    let stack = vec![
        "alloc::vec::Vec::push".to_string(),
        "my_app::process_data".to_string(),
    ];
    let guess_with_stack =
        UnsafeInferenceEngine::infer_with_context(&memory, 24, Some(&stack), None, None);
    println!("  With stack trace (Vec::push):");
    println!(
        "    Result: {} (confidence: {}%)",
        guess_with_stack.kind, guess_with_stack.confidence
    );
    println!();
}

fn demo_lifetime_analysis() {
    println!("--- Lifetime Analysis ---");

    let memory = b"test".to_vec();

    // Transient allocation (< 1ms)
    let alloc_time = Some(1_000_000);
    let dealloc_time = Some(1_000_500); // 500ns
    let guess =
        UnsafeInferenceEngine::infer_with_context(&memory, 4, None, alloc_time, dealloc_time);
    println!("  Transient allocation (500ns):");
    println!(
        "    Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );

    // Long-lived allocation (> 10s)
    let alloc_time = Some(1_000_000);
    let dealloc_time = Some(15_000_000_000); // 15s
    let guess =
        UnsafeInferenceEngine::infer_with_context(&memory, 4, None, alloc_time, dealloc_time);
    println!("  Long-lived allocation (15s):");
    println!(
        "    Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );
    println!();
}

fn demo_real_world_examples() {
    println!("--- Real World Examples ---");

    // Real Vec<i32>
    let v = vec![1i32, 2, 3, 4, 5];
    let ptr = v.as_ptr() as usize;
    let len = v.len();
    let cap = v.capacity();
    let mut memory = vec![0u8; 24];
    memory[..8].copy_from_slice(&ptr.to_le_bytes());
    memory[8..16].copy_from_slice(&len.to_le_bytes());
    memory[16..24].copy_from_slice(&cap.to_le_bytes());
    let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);
    println!("  Real Vec<i32> (len={}, cap={})", len, cap);
    println!(
        "    Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );

    // Real String
    let s = String::from("Hello, MemScope!");
    let ptr = s.as_ptr() as usize;
    let len = s.len();
    let cap = s.capacity();
    let mut memory = vec![0u8; 24];
    memory[..8].copy_from_slice(&ptr.to_le_bytes());
    memory[8..16].copy_from_slice(&len.to_le_bytes());
    memory[16..24].copy_from_slice(&cap.to_le_bytes());
    let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 24);
    println!("  Real String \"{}\" (len={}, cap={})", s, len, cap);
    println!(
        "    Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );

    // Real CString
    let cstr = std::ffi::CString::new("Hello, FFI!").unwrap();
    let bytes = cstr.as_bytes_with_nul();
    let guess = UnsafeInferenceEngine::infer_from_bytes(bytes, bytes.len());
    println!("  Real CString ({} bytes)", bytes.len());
    println!(
        "    Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );

    // Real Box
    let b = Box::new(42i32);
    let ptr = &*b as *const i32 as usize;
    let memory = ptr.to_le_bytes().to_vec();
    let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, 8);
    println!("  Real Box<i32> (ptr=0x{:x})", ptr);
    println!(
        "    Result: {} (confidence: {}%)",
        guess.kind, guess.confidence
    );
}
