//! 🚀 性能压力测试 - 轻量版本
//! 
//! 测试我们的超高性能优化方案在不同数据规模下的表现

use memscope_rs::export::binary::BinaryParser;
use memscope_rs::{track_var, get_global_tracker};
use std::time::Instant;

/// 测试配置
struct TestConfig {
    name: &'static str,
    allocation_count: usize,
    target_time_ms: u128,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 性能压力测试开始！");
    println!("{}", "=".repeat(50));

    // 渐进式测试不同规模
    let test_configs = vec![
        TestConfig {
            name: "小规模测试",
            allocation_count: 1_000,      // 1千
            target_time_ms: 50,
        },
        TestConfig {
            name: "中等规模测试", 
            allocation_count: 10_000,     // 1万
            target_time_ms: 100,
        },
        TestConfig {
            name: "大规模测试",
            allocation_count: 50_000,     // 5万
            target_time_ms: 300,
        },
        TestConfig {
            name: "超大规模测试",
            allocation_count: 100_000,    // 10万
            target_time_ms: 500,
        },
    ];

    for config in test_configs {
        println!("\n🔥 开始 {} ({} 个分配)", config.name, config.allocation_count);
        
        match run_performance_test(&config) {
            Ok(parse_time_ms) => {
                println!("✅ {} 完成！", config.name);
                println!("解析时间: {}ms", parse_time_ms);
                
                if parse_time_ms <= config.target_time_ms {
                    println!("🎉 性能目标达成: {}ms <= {}ms", parse_time_ms, config.target_time_ms);
                } else {
                    println!("⚠️  性能目标未达成: {}ms > {}ms", parse_time_ms, config.target_time_ms);
                }
                
                // 计算吞吐量
                let throughput = config.allocation_count as f64 / (parse_time_ms as f64 / 1000.0);
                println!("处理吞吐量: {:.0} 分配/秒", throughput);
            }
            Err(e) => {
                println!("❌ {} 失败: {}", config.name, e);
            }
        }
        
        println!("{}", "-".repeat(50));
    }

    println!("\n🏁 性能压力测试完成！");
    Ok(())
}

/// 运行单个性能测试
fn run_performance_test(config: &TestConfig) -> Result<u128, Box<dyn std::error::Error>> {
    // 第1步: 创建测试数据
    println!("📊 创建 {} 个分配记录...", config.allocation_count);
    let creation_start = Instant::now();
    
    create_test_data(config.allocation_count)?;
    
    let creation_time = creation_start.elapsed();
    println!("数据创建完成: {}ms", creation_time.as_millis());

    // 第2步: 导出到二进制
    println!("💾 导出到二进制文件...");
    let export_start = Instant::now();
    
    let tracker = get_global_tracker();
    let binary_file = format!("MemoryAnalysis/perf_test_{}.memscope", config.allocation_count);
    tracker.export_to_binary(&binary_file)?;
    
    let export_time = export_start.elapsed();
    let file_size = std::fs::metadata(&binary_file)?.len();
    println!("二进制导出完成: {}ms, 文件大小: {:.2}KB", 
        export_time.as_millis(), file_size as f64 / 1024.0);

    // 第3步: 使用超高性能方法解析
    println!("🚀 超高性能解析到JSON...");
    let parse_start = Instant::now();
    
    let output_name = format!("perf_test_{}", config.allocation_count);
    BinaryParser::parse_user_binary_to_json(
        &binary_file,
        &output_name
    )?;
    
    let parse_time = parse_start.elapsed();
    let parse_time_ms = parse_time.as_millis();

    // 第4步: 计算JSON文件大小
    let json_size = calculate_json_size(&output_name)?;
    println!("JSON文件总大小: {:.2}KB", json_size as f64 / 1024.0);

    // 清理测试文件
    cleanup_test_files(&binary_file, &output_name)?;

    Ok(parse_time_ms)
}

/// 创建测试数据
fn create_test_data(count: usize) -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..count {
        // 创建不同大小的测试数据
        let size = match i % 4 {
            0 => 64,        // 小对象
            1 => 256,       // 中等对象  
            2 => 1024,      // 大对象
            3 => 4096,      // 超大对象
            _ => 64,
        };
        
        let data = vec![i as u8; size];
        track_var!(data);
        drop(data);
    }
    
    Ok(())
}

/// 计算JSON文件总大小
fn calculate_json_size(output_name: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let json_files = [
        format!("MemoryAnalysis/{}/{}_memory_analysis.json", output_name, output_name),
        format!("MemoryAnalysis/{}/{}_lifetime.json", output_name, output_name),
        format!("MemoryAnalysis/{}/{}_performance.json", output_name, output_name),
        format!("MemoryAnalysis/{}/{}_unsafe_ffi.json", output_name, output_name),
        format!("MemoryAnalysis/{}/{}_complex_types.json", output_name, output_name),
    ];

    let mut total_size = 0u64;
    for file_path in &json_files {
        if let Ok(metadata) = std::fs::metadata(file_path) {
            total_size += metadata.len();
        }
    }

    Ok(total_size)
}

/// 清理测试文件
fn cleanup_test_files(binary_file: &str, output_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 删除二进制文件
    if std::path::Path::new(binary_file).exists() {
        std::fs::remove_file(binary_file)?;
    }

    // 删除JSON输出目录
    let output_dir = format!("MemoryAnalysis/{}", output_name);
    if std::path::Path::new(&output_dir).exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    Ok(())
}