use memscope_rs::core::types::AllocationInfo;
use memscope_rs::export::binary::{BinaryWriter, BinaryReader};
use tempfile::NamedTempFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Simple binary test...");
    
    // Create minimal test allocation
    let test_alloc = AllocationInfo {
        ptr: 0x1000,
        size: 1024,
        var_name: None,
        type_name: None,
        scope_name: None,
        timestamp_alloc: 1234567890,
        timestamp_dealloc: None,
        thread_id: "main".to_string(),
        borrow_count: 0,
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
    
    let temp_file = NamedTempFile::new()?;
    let file_path = temp_file.path();
    
    // Write
    {
        let mut writer = BinaryWriter::new(file_path)?;
        writer.write_header(1)?;
        writer.write_allocation(&test_alloc)?;
        writer.finish()?;
    }
    
    // Read
    let mut reader = BinaryReader::new(file_path)?;
    let _header = reader.read_header()?;
    let read_alloc = reader.read_allocation()?;
    
    println!("Original lifetime_ms: {:?}", test_alloc.lifetime_ms);
    println!("Read lifetime_ms: {:?}", read_alloc.lifetime_ms);
    
    assert_eq!(test_alloc.lifetime_ms, read_alloc.lifetime_ms);
    println!("✅ Test passed!");
    
    Ok(())
}