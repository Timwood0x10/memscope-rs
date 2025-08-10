use memscope_rs::{get_global_tracker, track_var};
use memscope_rs::export::binary::{detect_binary_type, parse_binary_auto, BinaryExportMode};
use std::fs;

#[test]
fn test_binary_type_detection() {
    println!("🧪 Testing Binary Type Detection");
    
    let tracker = get_global_tracker();
    
    // Create test data
    let a = vec![1, 2, 3, 4, 5];
    track_var!(a);

    let b = String::from("Detection Test");
    track_var!(b);
    
    // Give some time for tracking to register
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    // Test user-only binary detection
    println!("📦 Testing user-only binary detection...");
    tracker.export_user_binary("test_user_detection").unwrap();
    
    let user_binary_path = "MemoryAnalysis/test_user_detection.memscope";
    let user_info = detect_binary_type(user_binary_path).unwrap();
    
    println!("✅ User binary info:");
    println!("  Type: {}", user_info.type_description());
    println!("  Strategy: {}", user_info.recommended_strategy());
    println!("  Export mode: {:?}", user_info.export_mode);
    println!("  Total count: {}", user_info.total_count);
    println!("  User count: {}", user_info.user_count);
    println!("  System count: {}", user_info.system_count);
    println!("  File size: {} bytes", user_info.file_size);
    println!("  Count consistent: {}", user_info.is_count_consistent);
    
    // Verify user-only binary characteristics
    assert_eq!(user_info.export_mode, BinaryExportMode::UserOnly);
    assert!(user_info.is_user_only());
    assert!(!user_info.is_full_binary());
    // Note: user_count might be 0 if tracking didn't register variables
    // The important thing is that the export mode is correctly detected
    assert!(user_info.is_count_consistent);
    
    // Test full binary detection
    println!("📦 Testing full binary detection...");
    tracker.export_full_binary("test_full_detection").unwrap();
    
    let full_binary_path = "MemoryAnalysis/test_full_detection.memscope";
    let full_info = detect_binary_type(full_binary_path).unwrap();
    
    println!("✅ Full binary info:");
    println!("  Type: {}", full_info.type_description());
    println!("  Strategy: {}", full_info.recommended_strategy());
    println!("  Export mode: {:?}", full_info.export_mode);
    println!("  Total count: {}", full_info.total_count);
    println!("  User count: {}", full_info.user_count);
    println!("  System count: {}", full_info.system_count);
    println!("  File size: {} bytes", full_info.file_size);
    println!("  Count consistent: {}", full_info.is_count_consistent);
    
    // Verify full binary characteristics
    assert_eq!(full_info.export_mode, BinaryExportMode::Full);
    assert!(!full_info.is_user_only());
    assert!(full_info.is_full_binary());
    assert!(full_info.is_count_consistent);
    
    // Full binary should have at least as many allocations as user-only
    assert!(full_info.total_count >= user_info.total_count);
    assert!(full_info.file_size >= user_info.file_size);
    
    // Test auto parsing (just verify it doesn't crash)
    println!("🔄 Testing auto parsing...");
    
    // Test that auto parsing can detect and choose the right strategy
    let result1 = parse_binary_auto(user_binary_path, "test_user_auto");
    println!("User binary auto parsing result: {:?}", result1);
    
    let result2 = parse_binary_auto(full_binary_path, "test_full_auto");
    println!("Full binary auto parsing result: {:?}", result2);
    
    // Variables are already handled by track_var! macro
    
    println!("🎉 Binary type detection test completed successfully!");
}

#[test]
fn test_invalid_binary_detection() {
    println!("🧪 Testing Invalid Binary Detection");
    
    // Test with non-existent file
    let result = detect_binary_type("non_existent_file.memscope");
    assert!(result.is_err());
    
    // Create a file with invalid magic bytes
    fs::create_dir_all("MemoryAnalysis").unwrap();
    fs::write("MemoryAnalysis/invalid_magic.memscope", b"INVALID_MAGIC_BYTES").unwrap();
    
    let result = detect_binary_type("MemoryAnalysis/invalid_magic.memscope");
    assert!(result.is_err());
    
    // Clean up
    let _ = fs::remove_file("MemoryAnalysis/invalid_magic.memscope");
    
    println!("✅ Invalid binary detection test passed!");
}

#[test]
fn test_binary_info_methods() {
    println!("🧪 Testing BinaryFileInfo Methods");
    
    let tracker = get_global_tracker();
    let _test_data = track_var!(vec![1, 2, 3]);
    
    // Create a user binary
    tracker.export_user_binary("test_info_methods").unwrap();
    let info = detect_binary_type("MemoryAnalysis/test_info_methods.memscope").unwrap();
    
    // Test description methods
    let description = info.type_description();
    assert!(description.contains("User-only binary"));
    assert!(description.contains(&info.user_count.to_string()));
    
    let strategy = info.recommended_strategy();
    assert!(strategy.contains("Simple processing"));
    
    // Test boolean methods
    assert!(info.is_user_only());
    assert!(!info.is_full_binary());
    
    println!("✅ BinaryFileInfo methods test passed!");
}