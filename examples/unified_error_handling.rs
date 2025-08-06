//! Example demonstrating the unified error handling system
//! 
//! This example shows how the new MemScopeError system works while maintaining
//! backward compatibility with existing TrackingError code.

use memscope_rs::core::{
    MemScopeError, MemoryOperation, SystemErrorType, ErrorRecovery, DefaultErrorRecovery,
    from_tracking_error, to_tracking_error, adapt_result,
    TrackingError, TrackingResult
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Unified Error Handling System Demo ===\n");
    
    // Demonstrate new error creation
    demonstrate_new_error_creation();
    
    // Demonstrate error recovery
    demonstrate_error_recovery();
    
    // Demonstrate backward compatibility
    demonstrate_backward_compatibility();
    
    // Demonstrate error conversion
    demonstrate_error_conversion();
    
    Ok(())
}

fn demonstrate_new_error_creation() {
    println!("1. Creating new MemScopeError instances:");
    
    // Memory operation errors
    let alloc_error = MemScopeError::memory(MemoryOperation::Allocation, "Failed to allocate 1024 bytes");
    println!("   Memory error: {}", alloc_error);
    println!("   Category: {}, Severity: {:?}, Recoverable: {}", 
        alloc_error.category(), alloc_error.severity(), alloc_error.is_recoverable());
    
    // Analysis errors
    let analysis_error = MemScopeError::analysis("borrow_checker", "Mutable borrow conflict detected");
    println!("   Analysis error: {}", analysis_error);
    
    // Export errors with partial success
    let export_error = MemScopeError::export_partial("json", "Some data could not be serialized");
    println!("   Export error: {}", export_error);
    
    // System errors
    let system_error = MemScopeError::system(SystemErrorType::Io, "File not found");
    println!("   System error: {}", system_error);
    
    println!();
}

fn demonstrate_error_recovery() {
    println!("2. Error recovery mechanisms:");
    
    let recovery = DefaultErrorRecovery::new();
    
    // Test recoverable error
    let recoverable_error = MemScopeError::memory(MemoryOperation::Allocation, "Temporary allocation failure");
    println!("   Error: {}", recoverable_error);
    println!("   Can recover: {}", recovery.can_recover(&recoverable_error));
    
    if let Some(action) = recovery.get_recovery_action(&recoverable_error) {
        println!("   Recovery action: {:?}", action);
        if recovery.execute_recovery(&action).is_ok() {
            println!("   Recovery executed successfully");
        }
    }
    
    // Test non-recoverable error
    let critical_error = MemScopeError::internal("Critical system failure");
    println!("   Critical error: {}", critical_error);
    println!("   Can recover: {}", recovery.can_recover(&critical_error));
    
    println!();
}

fn demonstrate_backward_compatibility() {
    println!("3. Backward compatibility with TrackingError:");
    
    // Simulate old code that returns TrackingError
    let old_result = simulate_old_function();
    println!("   Old function result: {:?}", old_result);
    
    // Convert to new error system
    let adapted_result = adapt_result(old_result);
    match adapted_result {
        Ok(value) => println!("   Adapted result: Success with value {}", value),
        Err(error) => {
            println!("   Adapted error: {}", error);
            println!("   New error category: {}", error.category());
        }
    }
    
    println!();
}

fn demonstrate_error_conversion() {
    println!("4. Error type conversion:");
    
    // Convert from old to new
    let old_error = TrackingError::BorrowCheckError("Invalid mutable borrow".to_string());
    println!("   Old error: {}", old_error);
    
    let new_error = from_tracking_error(old_error);
    println!("   Converted to new: {}", new_error);
    println!("   Category: {}, Recoverable: {}", new_error.category(), new_error.is_recoverable());
    
    // Convert back to old
    let converted_back = to_tracking_error(&new_error);
    println!("   Converted back to old: {}", converted_back);
    
    // Test different error types
    test_error_type_conversions();
    
    println!();
}

fn test_error_type_conversions() {
    println!("   Testing various error type conversions:");
    
    let test_cases = vec![
        TrackingError::AllocationFailed("Memory exhausted".to_string()),
        TrackingError::SerializationError("JSON parse error".to_string()),
        TrackingError::ThreadSafetyError("Data race detected".to_string()),
        TrackingError::AnalysisError("Type inference failed".to_string()),
        TrackingError::ExportError("File write failed".to_string()),
    ];
    
    for old_error in test_cases {
        let new_error = from_tracking_error(old_error.clone());
        let converted_back = to_tracking_error(&new_error);
        
        println!("     {} -> {} -> {}", 
            error_type_name(&old_error),
            new_error.category(),
            error_type_name(&converted_back)
        );
    }
}

fn error_type_name(error: &TrackingError) -> &'static str {
    match error {
        TrackingError::AllocationFailed(_) => "AllocationFailed",
        TrackingError::DeallocationFailed(_) => "DeallocationFailed",
        TrackingError::SerializationError(_) => "SerializationError",
        TrackingError::ThreadSafetyError(_) => "ThreadSafetyError",
        TrackingError::AnalysisError(_) => "AnalysisError",
        TrackingError::ExportError(_) => "ExportError",
        _ => "Other",
    }
}

// Simulate an old function that returns TrackingResult
fn simulate_old_function() -> TrackingResult<i32> {
    // Simulate some operation that might fail
    if std::env::var("SIMULATE_SUCCESS").is_ok() {
        Ok(42)
    } else {
        Err(TrackingError::AllocationFailed("Simulated allocation failure".to_string()))
    }
}