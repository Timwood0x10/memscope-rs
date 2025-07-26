//! 数据本地化器演示
//!
//! 这个示例展示了如何使用数据本地化器来减少全局状态访问，
//! 从而提高导出性能。

use memscope_rs::export::data_localizer::DataLocalizer;
use memscope_rs::{init, track_var};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化内存跟踪
    init();

    println!("🚀 数据本地化器演示");
    println!("===================");

    // 创建一些测试数据
    create_test_allocations();

    // 演示数据本地化的性能优势
    demonstrate_data_localization()?;

    Ok(())
}

fn create_test_allocations() {
    println!("\n📦 创建测试分配...");

    // 创建各种类型的分配
    let vec1 = vec![1, 2, 3, 4, 5];
    track_var!(vec1);

    let string1 = String::from("Hello, World!");
    track_var!(string1);

    let vec2 = vec![10; 1000];
    track_var!(vec2);

    let string2 = "A".repeat(500);
    track_var!(string2);

    println!("   ✅ 创建了多个测试分配");
}

fn demonstrate_data_localization() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 演示数据本地化性能...");

    let mut localizer = DataLocalizer::new();

    // 第一次获取数据（冷启动）
    println!("\n--- 第一次数据获取（冷启动）---");
    let start_time = Instant::now();
    let (data1, stats1) = localizer.gather_all_export_data()?;
    let first_time = start_time.elapsed();

    println!("数据摘要: {}", data1.get_summary());
    println!("获取统计: {:?}", stats1);
    println!("总耗时: {:?}", first_time);

    // 立即再次获取数据（缓存命中）
    println!("\n--- 第二次数据获取（缓存命中）---");
    let start_time = Instant::now();
    let (data2, stats2) = localizer.gather_all_export_data()?;
    let second_time = start_time.elapsed();

    println!("数据摘要: {}", data2.get_summary());
    println!("获取统计: {:?}", stats2);
    println!("总耗时: {:?}", second_time);

    // 显示缓存统计
    let cache_stats = localizer.get_cache_stats();
    println!("\n📊 缓存统计: {:?}", cache_stats);

    // 计算性能提升
    if stats1.total_time_ms > 0 {
        let speedup = stats1.total_time_ms as f64 / stats2.total_time_ms.max(1) as f64;
        println!("\n🚀 性能提升:");
        println!("   缓存命中加速比: {:.2}x", speedup);
        println!(
            "   避免的全局访问: {} 次",
            estimate_avoided_accesses(&stats1)
        );
    }

    // 演示强制刷新
    println!("\n--- 强制刷新缓存 ---");
    let start_time = Instant::now();
    let (data3, stats3) = localizer.refresh_cache()?;
    let refresh_time = start_time.elapsed();

    println!("刷新后数据摘要: {}", data3.get_summary());
    println!("刷新统计: {:?}", stats3);
    println!("刷新耗时: {:?}", refresh_time);

    Ok(())
}

fn estimate_avoided_accesses(
    stats: &memscope_rs::export::data_localizer::DataGatheringStats,
) -> usize {
    // 估算在传统导出中需要的全局状态访问次数
    let basic_accesses = stats.allocation_count * 2; // 每个分配需要访问 tracker 2 次
    let ffi_accesses = stats.ffi_allocation_count * 3; // FFI 分配需要更多访问
    let scope_accesses = stats.scope_count * 1; // 作用域访问

    basic_accesses + ffi_accesses + scope_accesses
}
