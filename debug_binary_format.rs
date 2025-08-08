use memscope_rs::export::binary::{BinaryWriter, BinaryReader};
use memscope_rs::core::types::AllocationInfo;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

fn create_test_allocation() -> AllocationInfo {
    AllocationInfo {
        ptr: 0x1000,
        size: 1024,
        var_name: Some("test_var".to_string()),
        type_name: Some("i32".to_string()),
        scope_name: None,
        timestamp_alloc: 1234567890,
        timestamp_dealloc: None,
        thread_id: "main".to_string(),
        borrow_count: 0,
        stack_trace: None,
        is_leaked: false,
        lifetime_ms: None,
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
        drop_chain_analysis: None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_file = "debug_test.memscope";
    let test_allocations = vec![create_test_allocation()];

    // Write test data
    {
        let mut writer = BinaryWriter::new(test_file)?;
        writer.write_header(test_allocations.len() as u32)?;
        for alloc in &test_allocations {
            writer.write_allocation(alloc)?;
        }
        writer.finish()?;
    }

    // Read and debug the file content
    let mut file = File::open(test_file)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    println!("File size: {} bytes", buffer.len());
    println!("First 64 bytes:");
    for (i, chunk) in buffer.chunks(16).take(4).enumerate() {
        print!("{:04x}: ", i * 16);
        for byte in chunk {
            print!("{:02x} ", byte);
        }
        println!();
    }
    
    // Try to read with BinaryReader
    println!("\nTrying to read with BinaryReader:");
    let mut reader = BinaryReader::new(test_file)?;
    match reader.read_all() {
        Ok(allocations) => {
            println!("Successfully read {} allocations", allocations.len());
        }
        Err(e) => {
            println!("Error reading: {:?}", e);
        }
    }

    std::fs::remove_file(test_file)?;
    Ok(())
}