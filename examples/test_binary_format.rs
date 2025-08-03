use memscope_rs::core::types::AllocationInfo;
use memscope_rs::export::binary::{BinaryReader, BinaryWriter};
use std::fs;
use tempfile::NamedTempFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing binary format compatibility...");

    // Create test allocation
    let test_alloc = AllocationInfo {
        ptr: 0x1000,
        size: 1024,
        var_name: Some("test_var".to_string()),
        type_name: Some("Vec<u8>".to_string()),
        scope_name: Some("main".to_string()),
        timestamp_alloc: 1234567890,
        timestamp_dealloc: Some(1234567900),
        thread_id: "main".to_string(),
        borrow_count: 2,
        stack_trace: None,
        is_leaked: false,
        lifetime_ms: Some(100),
        smart_pointer_info: None,
        memory_layout: None,
        generic_info: None,
        dynamic_type_info: None,
        runtime_state: None,
        stack_allocation: None,
        temporary_object: None,
        fragmentation_analysis: None,
        generic_instantiation: None,
        type_relationships: None,
        type_usage: None,
        function_call_tracking: None,
        lifecycle_tracking: None,
        access_tracking: None,
    };

    // Create temporary file
    let temp_file = NamedTempFile::new()?;
    let file_path = temp_file.path();

    // Also create a persistent file for debugging
    let debug_path = std::path::Path::new("/tmp/test_binary_format.bin");

    // Write data
    println!("Writing test allocation...");
    {
        let mut writer = BinaryWriter::new(file_path)?;
        writer.write_header(1)?;
        writer.write_allocation(&test_alloc)?;
        writer.finish()?;
    }

    // Also write to debug file
    {
        let mut writer = BinaryWriter::new(debug_path)?;
        writer.write_header(1)?;
        writer.write_allocation(&test_alloc)?;
        writer.finish()?;
    }

    // Check file size
    let file_size = fs::metadata(file_path)?.len();
    println!("File size: {} bytes", file_size);

    // Read data back
    println!("Reading test allocation...");
    let mut reader = BinaryReader::new(file_path)?;
    let header = reader.read_header()?;
    println!(
        "Header: magic={:?}, version={}, count={}",
        std::str::from_utf8(&header.magic).unwrap_or("invalid"),
        header.version,
        header.count
    );

    let read_alloc = reader.read_allocation()?;

    // Compare data
    println!(
        "Original: ptr=0x{:x}, size={}, var_name={:?}",
        test_alloc.ptr, test_alloc.size, test_alloc.var_name
    );
    println!(
        "Read:     ptr=0x{:x}, size={}, var_name={:?}",
        read_alloc.ptr, read_alloc.size, read_alloc.var_name
    );

    // Verify key fields
    println!("Comparing fields...");
    assert_eq!(test_alloc.ptr, read_alloc.ptr);
    println!("✓ ptr matches");
    assert_eq!(test_alloc.size, read_alloc.size);
    println!("✓ size matches");
    assert_eq!(test_alloc.var_name, read_alloc.var_name);
    println!("✓ var_name matches");
    assert_eq!(test_alloc.type_name, read_alloc.type_name);
    println!("✓ type_name matches");
    assert_eq!(test_alloc.scope_name, read_alloc.scope_name);
    println!("✓ scope_name matches");
    assert_eq!(test_alloc.timestamp_alloc, read_alloc.timestamp_alloc);
    println!("✓ timestamp_alloc matches");
    assert_eq!(test_alloc.timestamp_dealloc, read_alloc.timestamp_dealloc);
    println!("✓ timestamp_dealloc matches");
    assert_eq!(test_alloc.borrow_count, read_alloc.borrow_count);
    println!("✓ borrow_count matches");
    assert_eq!(test_alloc.is_leaked, read_alloc.is_leaked);
    println!("✓ is_leaked matches");

    println!("Original lifetime_ms: {:?}", test_alloc.lifetime_ms);
    println!("Read lifetime_ms: {:?}", read_alloc.lifetime_ms);
    assert_eq!(test_alloc.lifetime_ms, read_alloc.lifetime_ms);

    println!("✅ All fields match!");
    println!("Binary format test passed!");

    Ok(())
}
