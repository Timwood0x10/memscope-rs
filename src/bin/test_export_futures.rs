//! 测试导出 Future 系统
//! 
//! 验证 Fast Future 和 Normal Future 的功能

use memscope_rs::{init, track_var};
use memscope_rs::export::export_modes::{
    ExportCoordinator, ExportMode, ExportOutcome,
    export_fast, export_with_validation
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 测试导出 Future 系统");
    println!("=======================");
    
    // 初始化内存跟踪
    init();
    
    // 创建一些测试数据
    let mut keep_alive = Vec::new();
    
    println!("📦 创建测试数据...");
    for i in 0..1000 {
        let test_vec = vec![i; 10];
        let tracked_vec = track_var!(test_vec);
        keep_alive.push(Box::new(tracked_vec) as Box<dyn std::any::Any>);
        
        let mut test_map = HashMap::new();
        test_map.insert(format!("key_{}", i), i * 2);
        let tracked_map = track_var!(test_map);
        keep_alive.push(Box::new(tracked_map) as Box<dyn std::any::Any>);
    }
    
    println!("✅ 测试数据创建完成，保持 {} 个变量存活", keep_alive.len());
    
    // 测试 Fast Future
    println!("\n🏃 测试 Fast Future (纯导出，无验证)");
    println!("=====================================");
    
    let fast_result = export_fast("test_fast_export.json").await;
    match fast_result {
        Ok(stats) => {
            println!("✅ Fast Future 成功:");
            println!("   处理分配: {}", stats.parallel_processing.total_allocations);
            println!("   文件大小: {:.2} MB", stats.write_performance.total_bytes_written as f64 / 1024.0 / 1024.0);
            println!("   总耗时: {} ms", stats.total_export_time_ms);
        }
        Err(e) => {
            println!("❌ Fast Future 失败: {}", e);
        }
    }
    
    // 测试 Normal Future
    println!("\n🚶 测试 Normal Future (先导出后验证)");
    println!("====================================");
    
    let normal_result = export_with_validation("test_normal_export.json").await;
    match normal_result {
        Ok((stats, validation)) => {
            println!("✅ Normal Future 成功:");
            println!("   处理分配: {}", stats.parallel_processing.total_allocations);
            println!("   文件大小: {:.2} MB", stats.write_performance.total_bytes_written as f64 / 1024.0 / 1024.0);
            println!("   导出耗时: {} ms", stats.total_export_time_ms);
            println!("   验证结果: {}", if validation.is_valid { "通过" } else { "失败" });
            println!("   验证耗时: {} ms", validation.validation_time_ms);
        }
        Err(e) => {
            println!("❌ Normal Future 失败: {}", e);
        }
    }
    
    // 测试 ExportCoordinator
    println!("\n🎯 测试 ExportCoordinator 统一接口");
    println!("==================================");
    
    // 快速模式
    let coordinator = ExportCoordinator::new_fast();
    let outcome = coordinator.export("test_coordinator_fast.json").await;
    match outcome {
        Ok(ExportOutcome::Fast(stats)) => {
            println!("✅ Coordinator Fast 模式成功:");
            println!("   处理分配: {}", stats.parallel_processing.total_allocations);
            println!("   总耗时: {} ms", stats.total_export_time_ms);
        }
        Ok(ExportOutcome::WithValidation(_, _)) => {
            println!("⚠️ 意外的验证结果");
        }
        Err(e) => {
            println!("❌ Coordinator Fast 模式失败: {}", e);
        }
    }
    
    // 正常模式
    let coordinator = ExportCoordinator::new_normal();
    let outcome = coordinator.export("test_coordinator_normal.json").await;
    match outcome {
        Ok(ExportOutcome::WithValidation(stats, validation)) => {
            println!("✅ Coordinator Normal 模式成功:");
            println!("   处理分配: {}", stats.parallel_processing.total_allocations);
            println!("   导出耗时: {} ms", stats.total_export_time_ms);
            println!("   验证结果: {}", if validation.is_valid { "通过" } else { "失败" });
        }
        Ok(ExportOutcome::Fast(_)) => {
            println!("⚠️ 意外的快速结果");
        }
        Err(e) => {
            println!("❌ Coordinator Normal 模式失败: {}", e);
        }
    }
    
    println!("\n🎉 测试完成！");
    println!("保持 {} 个变量存活直到程序结束", keep_alive.len());
    
    Ok(())
}