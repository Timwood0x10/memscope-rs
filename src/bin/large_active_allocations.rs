//! 大量活跃分配测试程序
//!
//! 创建大量活跃分配来测试真正的大文件导出性能

use memscope_rs::{get_global_tracker, init, track_var};
use std::collections::HashMap;

fn main() {
    println!("🚀 大量活跃分配测试程序");
    println!("======================");
    println!();

    init();

    // 保持所有分配存活的容器
    let mut keep_alive: Vec<Box<dyn std::any::Any>> = Vec::new();

    println!("📦 创建10,000个活跃分配...");

    for i in 0..10000 {
        // 创建不同类型的分配
        let large_vec = vec![i; 100];
        let tracked_vec = track_var!(large_vec);
        keep_alive.push(Box::new(tracked_vec) as Box<dyn std::any::Any>);

        let large_string = format!("Large string with data {}", i);
        let tracked_string = track_var!(large_string);
        keep_alive.push(Box::new(tracked_string) as Box<dyn std::any::Any>);

        let mut map = HashMap::new();
        map.insert(format!("key_{}", i), i);
        let tracked_map = track_var!(map);
        keep_alive.push(Box::new(tracked_map) as Box<dyn std::any::Any>);

        if i % 1000 == 0 {
            println!("  ✅ 已创建 {} 个分配组", i);
        }
    }

    println!("\n📊 最终统计:");
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("  • 总分配数: {}", stats.total_allocations);
        println!("  • 活跃分配数: {}", stats.active_allocations);
        println!(
            "  • 已释放分配数: {}",
            stats.total_allocations - stats.active_allocations
        );
        println!(
            "  • 活跃率: {:.1}%",
            stats.active_allocations as f64 / stats.total_allocations as f64 * 100.0
        );
        println!(
            "  • 活跃内存: {:.2} MB",
            stats.active_memory as f64 / 1024.0 / 1024.0
        );
    }

    println!("\n🎯 现在可以测试大文件导出性能了！");
    println!("建议使用快速导出来处理这么多活跃分配。");

    // 保持所有分配存活直到程序结束
    println!("📌 保持 {} 个变量存活", keep_alive.len());
}
