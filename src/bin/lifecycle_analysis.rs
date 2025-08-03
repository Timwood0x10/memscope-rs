//! 生命周期分析程序
//!
//! 分析为什么大部分分配在complex_lifecycle_showcase中被释放

use memscope_rs::{get_global_tracker, init, track_var};
use std::collections::HashMap;

fn main() {
    tracing::info!("🔍 生命周期分析程序");
    tracing::info!("==================");
    tracing::info!("");

    init();

    tracing::info!("📊 初始状态:");
    print_stats("初始化后");

    tracing::info!("\n🧪 测试1: 短生命周期分配");
    test_short_lifetime_allocations();
    print_stats("短生命周期测试后");

    tracing::info!("\n🧪 测试2: 长生命周期分配");
    let _long_lived = test_long_lifetime_allocations();
    print_stats("长生命周期测试后");

    tracing::info!("\n🧪 测试3: 混合生命周期模式");
    let _mixed_lived = test_mixed_lifetime_pattern();
    print_stats("混合生命周期测试后");

    tracing::info!("\n📊 最终分析:");
    analyze_lifecycle_patterns();
}

fn test_short_lifetime_allocations() {
    tracing::info!("  创建1000个短生命周期分配...");

    for i in 0..1000 {
        let temp_vec = vec![i; 100];
        let _tracked = track_var!(temp_vec);

        let temp_string = format!("Temporary string {}", i);
        let _tracked_string = track_var!(temp_string);

        let mut temp_map = HashMap::new();
        temp_map.insert(format!("key_{}", i), i);
        let _tracked_map = track_var!(temp_map);
    } // ← 所有变量在这里被释放

    tracing::info!("  ✅ 1000个短生命周期分配完成（已自动释放）");
}

fn test_long_lifetime_allocations() -> Vec<Box<dyn std::any::Any>> {
    tracing::info!("  创建100个长生命周期分配...");
    let mut keep_alive = Vec::new();

    for i in 0..100 {
        let long_vec = vec![i; 100];
        let tracked_vec = track_var!(long_vec);
        keep_alive.push(Box::new(tracked_vec) as Box<dyn std::any::Any>);

        let long_string = format!("Long-lived string {}", i);
        let tracked_string = track_var!(long_string);
        keep_alive.push(Box::new(tracked_string) as Box<dyn std::any::Any>);
    }

    tracing::info!("  ✅ 100个长生命周期分配完成（保持存活）");
    keep_alive
}

fn test_mixed_lifetime_pattern() -> Vec<Box<dyn std::any::Any>> {
    tracing::info!("  创建混合生命周期模式...");
    let mut keep_alive = Vec::new();

    // 创建500个短生命周期 + 50个长生命周期
    for i in 0..500 {
        // 短生命周期（会被释放）
        let temp_data = vec![i; 50];
        let _tracked_temp = track_var!(temp_data);

        // 每10个创建一个长生命周期（会保持存活）
        if i % 10 == 0 {
            let long_data = vec![i; 50];
            let tracked_long = track_var!(long_data);
            keep_alive.push(Box::new(tracked_long) as Box<dyn std::any::Any>);
        }
    }

    tracing::info!("  ✅ 混合模式: 500个短生命周期 + 50个长生命周期");
    keep_alive
}

fn print_stats(phase: &str) {
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        tracing::info!(
            "  📊 {}: 总分配={}, 活跃分配={}, 已释放={}",
            phase,
            stats.total_allocations,
            stats.active_allocations,
            stats.total_allocations - stats.active_allocations
        );
    }
}

fn analyze_lifecycle_patterns() {
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        tracing::info!("🔍 生命周期模式分析:");
        tracing::info!("  • 总分配数: {}", stats.total_allocations);
        tracing::info!("  • 活跃分配数: {}", stats.active_allocations);
        tracing::info!(
            "  • 已释放分配数: {}",
            stats.total_allocations - stats.active_allocations
        );
        tracing::info!(
            "  • 释放率: {:.1}%",
            (stats.total_allocations - stats.active_allocations) as f64
                / stats.total_allocations as f64
                * 100.0
        );

        let lifecycle = &stats.lifecycle_stats;
        tracing::info!("\n📈 生命周期分布:");
        tracing::info!("  • 瞬时分配 (< 1ms): {}", lifecycle.instant_allocations);
        tracing::info!(
            "  • 短期分配 (1-100ms): {}",
            lifecycle.short_term_allocations
        );
        tracing::info!(
            "  • 中期分配 (100ms-1s): {}",
            lifecycle.medium_term_allocations
        );
        tracing::info!("  • 长期分配 (> 1s): {}", lifecycle.long_term_allocations);

        tracing::info!("\n💡 结论:");
        if stats.active_allocations < stats.total_allocations / 2 {
            tracing::info!("  ⚠️  大部分分配已被释放，这解释了为什么导出文件大小不随总分配数增长");
            tracing::info!("  📝 导出系统只导出活跃分配，已释放的分配不会出现在文件中");
        } else {
            tracing::info!("  ✅ 大部分分配仍然活跃，文件大小应该与分配数成正比");
        }
    }
}
