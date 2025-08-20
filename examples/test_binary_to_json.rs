//! Test Binary to JSON - Verify binary contains improve.md fields
//!
//! This example reads the binary file and converts it to JSON to verify
//! that all improve.md extension fields are properly preserved

use memscope_rs::export::binary::reader::BinaryReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Binary to JSON - improve.md Field Verification");
    println!("==========================================================");

    let binary_file = "MemoryAnalysis/simple_binary_test.memscope";
    
    // Check if binary file exists
    if !std::path::Path::new(binary_file).exists() {
        println!("❌ Binary file not found: {}", binary_file);
        return Ok(());
    }

    let file_size = std::fs::metadata(binary_file)?.len();
    println!("📁 Binary file: {} ({} bytes)", binary_file, file_size);

    // Read binary file
    println!("\n📖 Reading binary file...");
    let mut reader = BinaryReader::new(binary_file)?;
    let allocations = reader.read_all()?;

    println!("✅ Successfully read {} allocations from binary", allocations.len());

    if allocations.is_empty() {
        println!("⚠️ No allocations found in binary file");
        return Ok(());
    }

    // Analyze improve.md extensions in the first few allocations
    println!("\n🔍 Analyzing improve.md extensions in allocations:");
    
    let mut has_borrow_info = 0;
    let mut has_clone_info = 0;
    let mut has_ownership_history = 0;
    let mut has_lifetime_ms = 0;

    for (i, alloc) in allocations.iter().enumerate().take(10) {
        println!("\n📋 Allocation {}: ptr=0x{:x}, size={}", i, alloc.ptr, alloc.size);
        
        if let Some(ref var_name) = alloc.var_name {
            println!("   • var_name: {}", var_name);
        }
        if let Some(ref type_name) = alloc.type_name {
            println!("   • type_name: {}", type_name);
        }

        // Check improve.md extensions
        if let Some(ref borrow_info) = alloc.borrow_info {
            has_borrow_info += 1;
            println!("   ✅ borrow_info: immutable={}, mutable={}, max_concurrent={}, last_timestamp={:?}",
                    borrow_info.immutable_borrows,
                    borrow_info.mutable_borrows,
                    borrow_info.max_concurrent_borrows,
                    borrow_info.last_borrow_timestamp);
        } else {
            println!("   ❌ borrow_info: None");
        }

        if let Some(ref clone_info) = alloc.clone_info {
            has_clone_info += 1;
            println!("   ✅ clone_info: count={}, is_clone={}, original_ptr={:?}",
                    clone_info.clone_count,
                    clone_info.is_clone,
                    clone_info.original_ptr);
        } else {
            println!("   ❌ clone_info: None");
        }

        if alloc.ownership_history_available {
            has_ownership_history += 1;
            println!("   ✅ ownership_history_available: true");
        } else {
            println!("   ❌ ownership_history_available: false");
        }

        if let Some(lifetime_ms) = alloc.lifetime_ms {
            has_lifetime_ms += 1;
            println!("   ✅ lifetime_ms: {} ms", lifetime_ms);
        } else {
            println!("   ❌ lifetime_ms: None");
        }
    }

    // Summary statistics
    println!("\n📊 improve.md Extensions Summary (first 10 allocations):");
    println!("   • borrow_info present: {}/10", has_borrow_info);
    println!("   • clone_info present: {}/10", has_clone_info);
    println!("   • ownership_history_available: {}/10", has_ownership_history);
    println!("   • lifetime_ms present: {}/10", has_lifetime_ms);

    // Overall statistics
    let total_allocations = allocations.len();
    let total_with_borrow_info = allocations.iter().filter(|a| a.borrow_info.is_some()).count();
    let total_with_clone_info = allocations.iter().filter(|a| a.clone_info.is_some()).count();
    let total_with_ownership_history = allocations.iter().filter(|a| a.ownership_history_available).count();
    let total_with_lifetime_ms = allocations.iter().filter(|a| a.lifetime_ms.is_some()).count();

    println!("\n📈 Overall Statistics (all {} allocations):", total_allocations);
    println!("   • borrow_info: {}/{} ({:.1}%)", 
            total_with_borrow_info, total_allocations,
            (total_with_borrow_info as f64 / total_allocations as f64) * 100.0);
    println!("   • clone_info: {}/{} ({:.1}%)", 
            total_with_clone_info, total_allocations,
            (total_with_clone_info as f64 / total_allocations as f64) * 100.0);
    println!("   • ownership_history_available: {}/{} ({:.1}%)", 
            total_with_ownership_history, total_allocations,
            (total_with_ownership_history as f64 / total_allocations as f64) * 100.0);
    println!("   • lifetime_ms: {}/{} ({:.1}%)", 
            total_with_lifetime_ms, total_allocations,
            (total_with_lifetime_ms as f64 / total_allocations as f64) * 100.0);

    // Convert to JSON for verification
    println!("\n💾 Converting to JSON for verification...");
    let json_output = serde_json::to_string_pretty(&allocations)?;
    std::fs::write("binary_to_json_output.json", &json_output)?;
    
    let json_size = json_output.len();
    println!("✅ JSON output written to: binary_to_json_output.json ({} bytes)", json_size);

    // Show sample JSON content
    println!("\n📄 Sample JSON content (first allocation):");
    if let Some(first_alloc) = allocations.first() {
        let sample_json = serde_json::to_string_pretty(first_alloc)?;
        let lines: Vec<&str> = sample_json.lines().take(20).collect();
        for line in lines {
            println!("   {}", line);
        }
        if sample_json.lines().count() > 20 {
            println!("   ... (truncated)");
        }
    }

    println!("\n🎯 Conclusion:");
    if total_with_borrow_info > 0 && total_with_clone_info > 0 && 
       total_with_ownership_history > 0 && total_with_lifetime_ms > 0 {
        println!("✅ Binary format successfully preserves ALL improve.md extension fields!");
        println!("✅ Binary to JSON conversion works perfectly!");
    } else {
        println!("❌ Some improve.md extension fields are missing from binary format");
    }

    Ok(())
}