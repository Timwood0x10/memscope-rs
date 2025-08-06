//! Demonstration of safe unwrap utilities
//! 
//! This example shows how to use the UnwrapSafe trait and related utilities
//! to safely handle unwrap operations while maintaining compatibility with
//! existing code that expects panic behavior.

use core::f64;

use memscope_rs::core::{UnwrapSafe, get_unwrap_stats, update_unwrap_stats};
use memscope_rs::{unwrap_safe, unwrap_or_default_safe, try_unwrap_safe};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Safe Unwrap Demonstration ===\n");
    
    // Initialize tracing to see the safe unwrap messages
    tracing_subscriber::fmt::init();
    
    demonstrate_option_unwrap();
    demonstrate_result_unwrap();
    demonstrate_macros();
    demonstrate_statistics();
    
    Ok(())
}

fn demonstrate_option_unwrap() {
    println!("1. Option unwrap demonstrations:");
    
    // Successful unwrap
    let some_value = Some(42);
    let result = some_value.unwrap_safe("successful option unwrap");
    println!("   Successful unwrap: {}", result);
    
    // Unwrap with default value instead of panic
    let none_value: Option<i32> = None;
    let result = none_value.unwrap_or_default_safe(99, "using default value");
    println!("   Default value used: {}", result);
    
    // Try unwrap (returns Result instead of panicking)
    let some_value = Some("hello");
    match some_value.try_unwrap_safe("trying to unwrap string") {
        Ok(value) => println!("   Try unwrap succeeded: {}", value),
        Err(error) => println!("   Try unwrap failed: {}", error),
    }
    
    let none_value: Option<&str> = None;
    match none_value.try_unwrap_safe("trying to unwrap None") {
        Ok(value) => println!("   Try unwrap succeeded: {}", value),
        Err(error) => println!("   Try unwrap failed: {}", error),
    }
    
    println!();
}

fn demonstrate_result_unwrap() {
    println!("2. Result unwrap demonstrations:");
    
    // Successful unwrap
    let ok_result: Result<i32, &str> = Ok(123);
    let result = ok_result.unwrap_safe("successful result unwrap");
    println!("   Successful unwrap: {}", result);
    
    // Unwrap with default value instead of panic
    let err_result: Result<i32, &str> = Err("something went wrong");
    let result = err_result.unwrap_or_default_safe(456, "using default for error");
    println!("   Default value used: {}", result);
    
    // Try unwrap (returns Result instead of panicking)
    let ok_result: Result<&str, &str> = Ok("success");
    match ok_result.try_unwrap_safe("trying to unwrap ok result") {
        Ok(value) => println!("   Try unwrap succeeded: {}", value),
        Err(error) => println!("   Try unwrap failed: {}", error),
    }
    
    let err_result: Result<&str, &str> = Err("failure");
    match err_result.try_unwrap_safe("trying to unwrap error result") {
        Ok(value) => println!("   Try unwrap succeeded: {}", value),
        Err(error) => println!("   Try unwrap failed: {}", error),
    }
    
    println!();
}

fn demonstrate_macros() {
    println!("3. Macro demonstrations:");
    
    // Using the unwrap_safe! macro
    let some_value = Some("macro test");
    let result = unwrap_safe!(some_value, "using unwrap_safe macro");
    println!("   Macro unwrap: {}", result);
    
    // Using the unwrap_or_default_safe! macro
    let none_value: Option<i32> = None;
    let result = unwrap_or_default_safe!(none_value, 777, "using default macro");
    println!("   Macro default: {}", result);
    
    // Using the try_unwrap_safe! macro
    let some_value = Some(f64::consts::PI);
    match try_unwrap_safe!(some_value, "trying with macro") {
        Ok(value) => println!("   Macro try unwrap: {}", value),
        Err(error) => println!("   Macro try unwrap failed: {}", error),
    }
    
    println!();
}

fn demonstrate_statistics() {
    println!("4. Unwrap statistics:");
    
    // Record some operations
    update_unwrap_stats(|stats| {
        stats.record_success();
        stats.record_success();
        stats.record_default_use();
        stats.record_failure();
    });
    
    let stats = get_unwrap_stats();
    println!("   Total operations: {}", stats.total_operations());
    println!("   Successful unwraps: {}", stats.successful_unwraps);
    println!("   Failed unwraps: {}", stats.failed_unwraps);
    println!("   Default value uses: {}", stats.default_value_uses);
    println!("   Success rate: {:.2}%", stats.success_rate() * 100.0);
    
    println!();
}

// Example of how to migrate existing unwrap code
fn example_migration() {
    println!("5. Migration example:");
    
    // Old code (would panic on None):
    // let value = some_option.unwrap();
    
    // New code options:
    
    // Option 1: Maintain panic behavior but with logging
    let some_option = Some("test");
    let _value = some_option.unwrap_safe("migrated unwrap with logging");
    
    // Option 2: Use default value instead of panicking
    let none_option: Option<&str> = None;
    let _value = none_option.unwrap_or_default_safe("default", "safe migration");
    
    // Option 3: Handle error gracefully
    let none_option: Option<&str> = None;
    match none_option.try_unwrap_safe("graceful handling") {
        Ok(value) => println!("   Got value: {}", value),
        Err(_) => println!("   Handled error gracefully"),
    }
    
    println!("   Migration examples completed");
}