use memscope_rs::{get_global_tracker, track_var};
use memscope_rs::export::binary::BinaryParser;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 JSON格式对比分析");
    println!("对比user-binary和full-binary生成的JSON文件差异");
    
    let tracker = get_global_tracker();
    
    // 创建一些测试数据
    println!("\n📦 创建测试数据...");
    let user_vector = track_var!(vec![1, 2, 3, 4, 5]);
    let user_string = track_var!(String::from("测试字符串"));
    
    // 等待一段时间让系统分配发生
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // 导出两种binary
    println!("📤 导出user-only binary...");
    tracker.export_user_binary("json_comparison_user")?;
    
    println!("📤 导出full binary...");
    tracker.export_full_binary("json_comparison_full")?;
    
    // 解析为JSON
    println!("🔄 解析user binary为JSON...");
    BinaryParser::parse_user_binary_to_json(
        "MemoryAnalysis/json_comparison_user.memscope", 
        "json_comparison_user"
    )?;
    
    println!("🔄 解析full binary为JSON...");
    BinaryParser::parse_full_binary_to_json(
        "MemoryAnalysis/json_comparison_full.memscope", 
        "json_comparison_full"
    )?;
    
    println!("\n📊 详细对比分析:");
    println!("{}", "=".repeat(80));
    
    // 分析memory_analysis.json
    analyze_memory_analysis_json()?;
    
    // 分析null字段情况
    analyze_null_fields()?;
    
    // 分析数据量差异
    analyze_data_volume()?;
    
    // 保持变量存活
    drop(user_vector);
    drop(user_string);
    
    println!("\n🎉 JSON对比分析完成!");
    
    Ok(())
}

fn analyze_memory_analysis_json() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n1️⃣ memory_analysis.json 对比:");
    println!("{}", "-".repeat(50));
    
    let user_file = "MemoryAnalysis/json_comparison_user/json_comparison_user_memory_analysis.json";
    let full_file = "MemoryAnalysis/json_comparison_full/json_comparison_full_memory_analysis.json";
    
    let user_content = fs::read_to_string(user_file)?;
    let full_content = fs::read_to_string(full_file)?;
    
    let user_json: serde_json::Value = serde_json::from_str(&user_content)?;
    let full_json: serde_json::Value = serde_json::from_str(&full_content)?;
    
    // 提取allocations数组
    let empty_vec = vec![];
    let user_allocs = user_json["allocations"].as_array().unwrap_or(&empty_vec);
    let full_allocs = full_json["allocations"].as_array().unwrap_or(&empty_vec);
    
    println!("📈 分配数量对比:");
    println!("  User binary: {} 个分配", user_allocs.len());
    println!("  Full binary: {} 个分配", full_allocs.len());
    
    println!("\n📄 文件大小对比:");
    println!("  User JSON: {} 字节", user_content.len());
    println!("  Full JSON: {} 字节", full_content.len());
    println!("  大小比例: {:.1}倍", full_content.len() as f64 / user_content.len() as f64);
    
    // 显示字段结构
    println!("\n🏗️ JSON结构对比:");
    if let (Some(user_obj), Some(full_obj)) = (user_json.as_object(), full_json.as_object()) {
        println!("  User binary字段: {:?}", user_obj.keys().collect::<Vec<_>>());
        println!("  Full binary字段: {:?}", full_obj.keys().collect::<Vec<_>>());
        println!("  结构是否一致: {}", user_obj.keys().collect::<std::collections::BTreeSet<_>>() == full_obj.keys().collect::<std::collections::BTreeSet<_>>());
    }
    
    // 显示样本数据
    if !full_allocs.is_empty() {
        println!("\n📋 Full binary样本分配数据:");
        let sample = &full_allocs[0];
        println!("  {}", serde_json::to_string_pretty(sample)?);
    }
    
    if !user_allocs.is_empty() {
        println!("\n📋 User binary样本分配数据:");
        let sample = &user_allocs[0];
        println!("  {}", serde_json::to_string_pretty(sample)?);
    } else {
        println!("\n📋 User binary: 无用户分配数据 (只包含系统分配)");
    }
    
    Ok(())
}

fn analyze_null_fields() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n2️⃣ Null字段分析:");
    println!("{}", "-".repeat(50));
    
    let json_files = [
        ("memory_analysis.json", "内存分析"),
        ("lifetime.json", "生命周期分析"),
        ("performance.json", "性能分析"),
        ("unsafe_ffi.json", "不安全FFI分析"),
        ("complex_types.json", "复杂类型分析"),
    ];
    
    for (file_suffix, description) in &json_files {
        let user_file = format!("MemoryAnalysis/json_comparison_user/json_comparison_user_{}", file_suffix);
        let full_file = format!("MemoryAnalysis/json_comparison_full/json_comparison_full_{}", file_suffix);
        
        if let (Ok(user_content), Ok(full_content)) = (
            fs::read_to_string(&user_file),
            fs::read_to_string(&full_file)
        ) {
            let user_json: serde_json::Value = serde_json::from_str(&user_content)?;
            let full_json: serde_json::Value = serde_json::from_str(&full_content)?;
            
            let user_nulls = count_null_values(&user_json);
            let full_nulls = count_null_values(&full_json);
            
            println!("📄 {} ({}):", description, file_suffix);
            println!("  User binary null字段: {}", user_nulls);
            println!("  Full binary null字段: {}", full_nulls);
            
            if full_nulls == 0 {
                println!("  ✅ Full binary成功消除所有null字段!");
            } else {
                println!("  ❌ Full binary仍有{}个null字段", full_nulls);
            }
        }
    }
    
    Ok(())
}

fn analyze_data_volume() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n3️⃣ 数据量差异分析:");
    println!("{}", "-".repeat(50));
    
    // 检查binary文件大小
    let user_binary_size = fs::metadata("MemoryAnalysis/json_comparison_user.memscope")?.len();
    let full_binary_size = fs::metadata("MemoryAnalysis/json_comparison_full.memscope")?.len();
    
    println!("💾 Binary文件大小:");
    println!("  User binary: {} 字节", user_binary_size);
    println!("  Full binary: {} 字节", full_binary_size);
    println!("  大小比例: {:.1}倍", full_binary_size as f64 / user_binary_size as f64);
    
    // 统计所有JSON文件大小
    let json_files = [
        "memory_analysis.json",
        "lifetime.json", 
        "performance.json",
        "unsafe_ffi.json",
        "complex_types.json",
    ];
    
    let mut user_total_size = 0;
    let mut full_total_size = 0;
    
    println!("\n📊 各JSON文件大小对比:");
    for file_suffix in &json_files {
        let user_file = format!("MemoryAnalysis/json_comparison_user/json_comparison_user_{}", file_suffix);
        let full_file = format!("MemoryAnalysis/json_comparison_full/json_comparison_full_{}", file_suffix);
        
        if let (Ok(user_meta), Ok(full_meta)) = (
            fs::metadata(&user_file),
            fs::metadata(&full_file)
        ) {
            let user_size = user_meta.len();
            let full_size = full_meta.len();
            
            user_total_size += user_size;
            full_total_size += full_size;
            
            println!("  {}: {} bytes (user) vs {} bytes (full) - {:.1}倍", 
                file_suffix, user_size, full_size, 
                if user_size > 0 { full_size as f64 / user_size as f64 } else { 0.0 });
        }
    }
    
    println!("\n📈 总计:");
    println!("  User JSON总大小: {} 字节", user_total_size);
    println!("  Full JSON总大小: {} 字节", full_total_size);
    println!("  JSON大小比例: {:.1}倍", 
        if user_total_size > 0 { full_total_size as f64 / user_total_size as f64 } else { 0.0 });
    
    Ok(())
}

/// 递归计算JSON中的null值数量
fn count_null_values(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Null => 1,
        serde_json::Value::Array(arr) => {
            arr.iter().map(count_null_values).sum()
        },
        serde_json::Value::Object(obj) => {
            obj.values().map(count_null_values).sum()
        },
        _ => 0,
    }
}