//! Test async validation framework
//!
//! This binary tests the newly implemented async validation framework

use memscope_rs::export::quality_validator::{AsyncValidator, ValidationConfig, DeferredValidation};
use std::path::Path;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Async Validation Framework");
    
    // Test 1: AsyncValidator with default config
    println!("\n1. Testing AsyncValidator with default config");
    let mut validator = AsyncValidator::new(ValidationConfig::default());
    
    // Create a temporary test file
    let temp_file = std::env::temp_dir().join("test_async_validation.json");
    std::fs::write(&temp_file, r#"{"test": "data", "allocations": []}"#)?;
    
    let result = validator.validate_file_async(&temp_file).await?;
    println!("   ✅ Validation result: {}", if result.is_valid { "PASSED" } else { "FAILED" });
    println!("   📊 Validation time: {}ms", result.validation_time_ms);
    println!("   📝 Message: {}", result.message);
    
    // Test 2: Fast mode configuration
    println!("\n2. Testing fast mode configuration");
    let fast_config = ValidationConfig::for_fast_mode();
    let mut fast_validator = AsyncValidator::new(fast_config);
    
    let fast_result = fast_validator.validate_file_async(&temp_file).await?;
    println!("   ✅ Fast validation result: {}", if fast_result.is_valid { "PASSED" } else { "FAILED" });
    println!("   📊 Fast validation time: {}ms", fast_result.validation_time_ms);
    
    // Test 3: Slow mode configuration
    println!("\n3. Testing slow mode configuration");
    let slow_config = ValidationConfig::for_slow_mode();
    let mut slow_validator = AsyncValidator::new(slow_config);
    
    let slow_result = slow_validator.validate_file_async(&temp_file).await?;
    println!("   ✅ Slow validation result: {}", if slow_result.is_valid { "PASSED" } else { "FAILED" });
    println!("   📊 Slow validation time: {}ms", slow_result.validation_time_ms);
    
    // Test 4: DeferredValidation wrapper
    println!("\n4. Testing DeferredValidation wrapper");
    let deferred = DeferredValidation::new(&temp_file, 0, ValidationConfig::default());
    println!("   📁 File path: {}", deferred.get_file_path());
    
    let deferred_result = deferred.await_result().await?;
    println!("   ✅ Deferred validation result: {}", if deferred_result.is_valid { "PASSED" } else { "FAILED" });
    
    // Test 5: Stream validation with larger file
    println!("\n5. Testing stream validation with larger file");
    let large_content = format!(r#"{{"allocations": [{}]}}"#, 
        (0..1000).map(|i| format!(r#"{{"ptr": {}, "size": 64}}"#, 0x1000 + i))
                 .collect::<Vec<_>>()
                 .join(","));
    
    let large_file = std::env::temp_dir().join("test_large_validation.json");
    std::fs::write(&large_file, large_content)?;
    
    let mut stream_validator = AsyncValidator::new(ValidationConfig::for_slow_mode());
    let stream_result = stream_validator.validate_file_async(&large_file).await?;
    println!("   ✅ Stream validation result: {}", if stream_result.is_valid { "PASSED" } else { "FAILED" });
    println!("   📊 Stream validation time: {}ms", stream_result.validation_time_ms);
    println!("   📏 File size validated: {} bytes", stream_result.data_size);
    
    // Cleanup
    let _ = std::fs::remove_file(&temp_file);
    let _ = std::fs::remove_file(&large_file);
    
    println!("\n🎉 All async validation tests completed successfully!");
    
    Ok(())
}