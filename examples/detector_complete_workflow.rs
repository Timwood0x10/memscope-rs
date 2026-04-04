//! 完整的工作流示例：数据收集 -> 分析 -> 导出 HTML
//!
//! 这个示例演示了如何使用 memscope-rs 的新架构进行完整的内存分析工作流：
//! 1. 初始化 MemScope
//! 2. 模拟数据收集（内存分配和释放）
//! 3. 运行各种检测器进行分析
//! 4. 导出结果为 HTML 仪表盘
//! 5. 导出结果为 JSON 文件

use memscope_rs::facade::MemScope;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 memscope-rs 完整工作流示例\n");
    println!("====================================\n");

    // ===== 步骤 1: 初始化 MemScope =====
    println!("步骤 1: 初始化 MemScope...");
    let memscope = Arc::new(MemScope::new());
    println!("✅ MemScope 初始化完成\n");

    // ===== 步骤 2: 模拟数据收集 =====
    println!("步骤 2: 模拟数据收集（内存分配和释放）...");
    simulate_memory_allocations(&memscope);
    println!("✅ 数据收集完成，共 {} 个事件\n", memscope.event_count());

    // 显示内存摘要
    println!("当前内存摘要：");
    match memscope.summary() {
        memscope_rs::query::QueryResult::Summary(summary) => {
            println!("  - 活跃分配: {}", summary.active_allocations);
            println!("  - 总分配: {}", summary.total_allocations);
            println!("  - 总释放: {}", summary.total_deallocations);
            println!("  - 当前内存: {} KB", summary.current_memory / 1024);
            println!("  - 峰值内存: {} KB\n", summary.peak_memory / 1024);
        }
        _ => println!("无法获取内存摘要\n"),
    }

    // ===== 步骤 3: 运行检测器 =====
    println!("步骤 3: 运行各种检测器进行分析...\n");

    // 3.1 泄漏检测器
    println!("3.1 运行泄漏检测器...");
    let leak_result = memscope.run_leak_detector();
    print_detection_result(&leak_result);

    // 3.2 Use-After-Free 检测器
    println!("\n3.2 运行 Use-After-Free 检测器...");
    let uaf_result = memscope.run_uaf_detector();
    print_detection_result(&uaf_result);

    // 3.3 溢出检测器
    println!("\n3.3 运行溢出检测器...");
    let overflow_result = memscope.run_overflow_detector();
    print_detection_result(&overflow_result);

    // 3.4 安全检测器
    println!("\n3.4 运行安全检测器...");
    let safety_result = memscope.run_safety_detector();
    print_detection_result(&safety_result);

    // 3.5 生命周期检测器
    println!("\n3.5 运行生命周期检测器...");
    let lifecycle_result = memscope.run_lifecycle_detector();
    print_detection_result(&lifecycle_result);

    // ===== 步骤 4: 导出结果 =====
    println!("\n====================================");
    println!("步骤 4: 导出分析结果...\n");

    // 4.1 导出 HTML 仪表盘
    println!("4.1 导出 HTML 仪表盘...");
    match memscope.export_html("./output") {
        Ok(_) => println!("✅ HTML 仪表盘已导出到 ./output/ 目录"),
        Err(e) => println!("❌ 导出 HTML 失败: {}", e),
    }

    // 4.2 导出 JSON 文件
    println!("\n4.2 导出 JSON 文件...");
    match memscope.export_json("./output") {
        Ok(_) => println!("✅ JSON 文件已导出到 ./output/ 目录"),
        Err(e) => println!("❌ 导出 JSON 失败: {}", e),
    }

    // ===== 完成 =====
    println!("\n====================================");
    println!("🎉 工作流完成！");
    println!("\n你可以打开 ./output/ 目录查看生成的文件：");
    println!("  - binary_dashboard.html: 交互式 HTML 仪表盘");
    println!("  - 8 个 JSON 文件: 详细的内存分析数据");
    println!("\n提示: 在浏览器中打开 binary_dashboard.html 查看可视化结果！");

    Ok(())
}

/// 模拟内存分配和释放
fn simulate_memory_allocations(memscope: &std::sync::Arc<MemScope>) {
    // 模拟线程 1 的内存分配
    let memscope_clone = memscope.clone();
    thread::spawn(move || {
        // 分配一些内存
        for i in 0..10 {
            let ptr = 0x1000 + i * 0x100;
            let size = 1024 + i * 256;
            memscope_clone.capture.capture_alloc(ptr, size, 1);
            thread::sleep(Duration::from_millis(10));
        }

        // 释放部分内存
        for i in 0..5 {
            let ptr = 0x1000 + i * 0x100;
            let size = 1024 + i * 256;
            memscope_clone.capture.capture_dealloc(ptr, size, 1);
        }
    });

    // 模拟线程 2 的内存分配
    let memscope_clone = memscope.clone();
    thread::spawn(move || {
        // 分配一些内存
        for i in 0..8 {
            let ptr = 0x2000 + i * 0x200;
            let size = 2048 + i * 512;
            memscope_clone.capture.capture_alloc(ptr, size, 2);
            thread::sleep(Duration::from_millis(15));
        }

        // 释放部分内存
        for i in 0..3 {
            let ptr = 0x2000 + i * 0x200;
            let size = 2048 + i * 512;
            memscope_clone.capture.capture_dealloc(ptr, size, 2);
        }
    });

    // 主线程也分配一些内存
    for i in 0..6 {
        let ptr = 0x3000 + i * 0x150;
        let size = 512 + i * 128;
        memscope.capture.capture_alloc(ptr, size, 0);
        thread::sleep(Duration::from_millis(5));
    }

    // 等待其他线程完成
    thread::sleep(Duration::from_millis(500));
}

/// 打印检测结果
fn print_detection_result(result: &memscope_rs::analysis::detectors::DetectionResult) {
    println!("  检测器: {}", result.detector_name);
    println!("  检测时间: {} ms", result.detection_time_ms);
    println!("  总分配数: {}", result.statistics.total_allocations);
    println!("  发现问题数: {}", result.issues.len());

    if !result.issues.is_empty() {
        println!("  问题详情:");
        for (i, issue) in result.issues.iter().take(5).enumerate() {
            println!("    {}. [{}] {}", i + 1, issue.severity, issue.description);
        }
        if result.issues.len() > 5 {
            println!("    ... 还有 {} 个问题", result.issues.len() - 5);
        }
    }

    if result.issues.is_empty() {
        println!("  ✅ 未发现问题");
    } else {
        println!("  ⚠️  发现 {} 个问题", result.issues.len());
    }
}
