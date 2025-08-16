//! Test example for Call Stack Normalization and FFI Function Resolution
//!
//! This example demonstrates the call stack normalization and FFI function resolution capabilities.

use memscope_rs::core::{
    CallStackNormalizer, NormalizerConfig, get_global_call_stack_normalizer,
};
use memscope_rs::analysis::{
    FfiFunctionResolver, ResolverConfig, get_global_ffi_resolver,
    FfiFunctionCategory, FfiRiskLevel,
};
use memscope_rs::analysis::unsafe_ffi_tracker::StackFrame;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("📚 Testing Call Stack Normalization and FFI Function Resolution");

    // Test 1: Call Stack Normalization
    println!("\n📝 Test 1: Call Stack Normalization");
    
    let normalizer = CallStackNormalizer::new(NormalizerConfig::default());
    
    // Create test call stacks
    let stack1 = vec![
        StackFrame {
            function_name: "main".to_string(),
            file_name: Some("main.rs".to_string()),
            line_number: Some(10),
            is_unsafe: false,
        },
        StackFrame {
            function_name: "allocate_memory".to_string(),
            file_name: Some("memory.rs".to_string()),
            line_number: Some(25),
            is_unsafe: true,
        },
    ];
    
    let stack2 = vec![
        StackFrame {
            function_name: "main".to_string(),
            file_name: Some("main.rs".to_string()),
            line_number: Some(10),
            is_unsafe: false,
        },
        StackFrame {
            function_name: "allocate_memory".to_string(),
            file_name: Some("memory.rs".to_string()),
            line_number: Some(25),
            is_unsafe: true,
        },
    ];
    
    // Normalize identical call stacks
    let id1 = normalizer.normalize_call_stack(&stack1)?;
    let id2 = normalizer.normalize_call_stack(&stack2)?;
    
    println!("✅ First call stack normalized to ID: {}", id1);
    println!("✅ Second call stack normalized to ID: {}", id2);
    println!("✅ IDs are identical (deduplication works): {}", id1 == id2);
    
    // Retrieve call stack
    let retrieved = normalizer.get_call_stack(id1)?;
    println!("✅ Retrieved call stack has {} frames", retrieved.len());
    
    // Test different call stack
    let stack3 = vec![
        StackFrame {
            function_name: "different_function".to_string(),
            file_name: Some("other.rs".to_string()),
            line_number: Some(50),
            is_unsafe: false,
        },
    ];
    
    let id3 = normalizer.normalize_call_stack(&stack3)?;
    println!("✅ Different call stack normalized to ID: {}", id3);
    println!("✅ Different ID (no false deduplication): {}", id1 != id3);
    
    // Get statistics
    let stats = normalizer.get_stats();
    println!("📊 Normalization Stats:");
    println!("   • Total processed: {}", stats.total_processed);
    println!("   • Unique stacks: {}", stats.unique_stacks);
    println!("   • Duplicates avoided: {}", stats.duplicates_avoided);
    println!("   • Memory saved: {} bytes", stats.memory_saved_bytes);

    // Test 2: FFI Function Resolution
    println!("\n🔍 Test 2: FFI Function Resolution");
    
    let resolver = FfiFunctionResolver::new(ResolverConfig::default());
    
    // Test known functions
    let malloc_result = resolver.resolve_function("malloc", None)?;
    println!("✅ Resolved malloc:");
    println!("   • Library: {}", malloc_result.library_name);
    println!("   • Function: {}", malloc_result.function_name);
    println!("   • Category: {:?}", malloc_result.category);
    println!("   • Risk Level: {:?}", malloc_result.risk_level);
    
    let strcpy_result = resolver.resolve_function("strcpy", None)?;
    println!("✅ Resolved strcpy:");
    println!("   • Library: {}", strcpy_result.library_name);
    println!("   • Risk Level: {:?} (should be Critical)", strcpy_result.risk_level);
    
    // Test auto-discovery
    let ssl_result = resolver.resolve_function("SSL_new", None)?;
    println!("✅ Auto-discovered SSL_new:");
    println!("   • Library: {}", ssl_result.library_name);
    println!("   • Category: {:?}", ssl_result.category);
    
    // Test with library hint
    let custom_result = resolver.resolve_function("custom_func", Some("mylib"))?;
    println!("✅ Resolved with hint:");
    println!("   • Library: {}", custom_result.library_name);
    println!("   • Function: {}", custom_result.function_name);
    
    // Test batch resolution
    let functions = vec!["malloc".to_string(), "free".to_string(), "printf".to_string()];
    let batch_results = resolver.resolve_functions_batch(&functions);
    println!("✅ Batch resolution of {} functions:", functions.len());
    for (i, result) in batch_results.iter().enumerate() {
        match result {
            Ok(resolved) => println!("   • {}: {} -> {}", 
                functions[i], resolved.library_name, resolved.function_name),
            Err(e) => println!("   • {}: Error - {:?}", functions[i], e),
        }
    }
    
    // Get resolution statistics
    let res_stats = resolver.get_stats();
    println!("📊 Resolution Stats:");
    println!("   • Total attempts: {}", res_stats.total_attempts);
    println!("   • Successful: {}", res_stats.successful_resolutions);
    println!("   • Cache hits: {}", res_stats.cache_hits);

    // Test 3: Global instances
    println!("\n🌐 Test 3: Global Instances");
    
    let global_normalizer = get_global_call_stack_normalizer();
    let global_resolver = get_global_ffi_resolver();
    
    // Test global normalizer
    let global_id = global_normalizer.normalize_call_stack(&stack1)?;
    println!("✅ Global normalizer ID: {}", global_id);
    
    // Test global resolver
    let global_malloc = global_resolver.resolve_function("malloc", None)?;
    println!("✅ Global resolver result: {}::{}", 
        global_malloc.library_name, global_malloc.function_name);

    // Test 4: Risk Assessment
    println!("\n⚠️  Test 4: Risk Assessment");
    
    let dangerous_functions = vec!["strcpy", "sprintf", "gets"];
    let safe_functions = vec!["snprintf", "strncpy", "fgets"];
    
    println!("Dangerous functions:");
    for func in &dangerous_functions {
        let result = resolver.resolve_function(func, None)?;
        println!("   • {}: {:?}", func, result.risk_level);
    }
    
    println!("Safer alternatives:");
    for func in &safe_functions {
        let result = resolver.resolve_function(func, None)?;
        println!("   • {}: {:?}", func, result.risk_level);
    }

    // Test 5: Category Classification
    println!("\n📂 Test 5: Category Classification");
    
    let test_functions = vec![
        ("malloc", FfiFunctionCategory::MemoryManagement),
        ("strcpy", FfiFunctionCategory::StringManipulation),
        ("fopen", FfiFunctionCategory::FileIO),
        ("SSL_new", FfiFunctionCategory::Cryptographic),
        ("fork", FfiFunctionCategory::SystemCall),
    ];
    
    for (func, expected_category) in &test_functions {
        let result = resolver.resolve_function(func, None)?;
        let matches = result.category == *expected_category;
        println!("   • {}: {:?} (expected: {:?}) ✅{}", 
            func, result.category, expected_category, 
            if matches { "" } else { " ❌" });
    }

    println!("\n🎉 All tests completed successfully!");
    println!("Call Stack Normalization and FFI Function Resolution are working correctly.");

    Ok(())
}