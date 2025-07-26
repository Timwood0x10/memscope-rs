//! 进度监控和取消机制演示
//!
//! 这个示例展示了如何使用进度监控功能来跟踪导出进度，
//! 以及如何实现取消机制来中断长时间运行的导出操作。

use memscope_rs::core::tracker::MemoryTracker;
use memscope_rs::export::fast_export_coordinator::{FastExportCoordinator, FastExportConfigBuilder};
use memscope_rs::export::progress_monitor::{
    ProgressMonitor, ExportProgress, ExportStage, ConsoleProgressDisplay, CancellationToken,
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 进度监控和取消机制演示");
    println!("============================");
    
    // 创建内存跟踪器
    let tracker = MemoryTracker::new();
    
    // 模拟一些内存分配
    println!("📊 创建测试数据...");
    create_test_allocations(&tracker, 10000);
    
    // 演示 1: 基本进度监控
    println!("\n1️⃣ 基本进度监控演示:");
    demo_basic_progress_monitoring()?;
    
    // 演示 2: 控制台进度显示
    println!("\n2️⃣ 控制台进度显示演示:");
    demo_console_progress_display()?;
    
    // 演示 3: 取消机制
    println!("\n3️⃣ 取消机制演示:");
    demo_cancellation_mechanism()?;
    
    // 演示 4: 快速导出协调器的进度监控
    println!("\n4️⃣ 快速导出协调器进度监控:");
    demo_fast_export_with_progress()?;
    
    // 演示 5: 自定义进度回调
    println!("\n5️⃣ 自定义进度回调演示:");
    demo_custom_progress_callback()?;
    
    println!("\n✅ 进度监控演示完成!");
    
    Ok(())
}

/// 演示基本进度监控功能
fn demo_basic_progress_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    let mut monitor = ProgressMonitor::new(1000);
    
    // 模拟导出过程的各个阶段
    let stages = [
        (ExportStage::Initializing, "初始化导出环境"),
        (ExportStage::DataLocalization, "本地化数据"),
        (ExportStage::ParallelProcessing, "并行分片处理"),
        (ExportStage::Writing, "高速缓冲写入"),
    ];
    
    for (stage, description) in &stages {
        monitor.set_stage(stage.clone());
        println!("   📍 {}", description);
        
        // 模拟阶段进度
        for i in 0..=10 {
            let progress = i as f64 / 10.0;
            monitor.update_progress(progress, Some(format!("{}进度: {:.0}%", description, progress * 100.0)));
            
            // 模拟处理一些分配
            monitor.add_processed(10);
            
            thread::sleep(Duration::from_millis(50));
        }
    }
    
    monitor.complete();
    let final_progress = monitor.get_progress_snapshot();
    println!("   ✅ 导出完成! 总耗时: {:?}", final_progress.elapsed_time);
    
    Ok(())
}

/// 演示控制台进度显示
fn demo_console_progress_display() -> Result<(), Box<dyn std::error::Error>> {
    let mut monitor = ProgressMonitor::new(500);
    let mut display = ConsoleProgressDisplay::new();
    
    // 设置进度回调
    monitor.set_callback(Box::new(move |progress| {
        // 这里我们不能直接使用 display，因为它被移动了
        // 在实际应用中，你可能需要使用 Arc<Mutex<>> 来共享 display
    }));
    
    // 模拟处理过程
    monitor.set_stage(ExportStage::ParallelProcessing);
    
    for i in 0..=100 {
        let progress = i as f64 / 100.0;
        monitor.update_progress(progress, Some(format!("处理中... {:.0}%", progress * 100.0)));
        monitor.add_processed(5);
        
        // 手动显示进度（在实际应用中这会通过回调自动完成）
        let current_progress = monitor.get_progress_snapshot();
        display.display(&current_progress);
        
        thread::sleep(Duration::from_millis(20));
    }
    
    display.finish();
    monitor.complete();
    
    Ok(())
}

/// 演示取消机制
fn demo_cancellation_mechanism() -> Result<(), Box<dyn std::error::Error>> {
    let mut monitor = ProgressMonitor::new(1000);
    let cancellation_token = monitor.cancellation_token();
    
    // 在另一个线程中模拟长时间运行的任务
    let token_clone = cancellation_token.clone();
    let handle = thread::spawn(move || {
        for i in 0..1000 {
            // 检查是否应该取消
            if token_clone.is_cancelled() {
                println!("   ⚠️ 任务在第 {} 步被取消", i);
                return Err("Task was cancelled");
            }
            
            // 模拟一些工作
            thread::sleep(Duration::from_millis(5));
        }
        Ok("Task completed")
    });
    
    // 等待一段时间后取消任务
    thread::sleep(Duration::from_millis(100));
    println!("   🛑 发送取消信号...");
    cancellation_token.cancel();
    monitor.cancel();
    
    // 等待任务完成
    match handle.join().unwrap() {
        Ok(msg) => println!("   ✅ {}", msg),
        Err(msg) => println!("   ❌ {}", msg),
    }
    
    Ok(())
}

/// 演示快速导出协调器的进度监控
fn demo_fast_export_with_progress() -> Result<(), Box<dyn std::error::Error>> {
    let config = FastExportConfigBuilder::new()
        .progress_monitoring(true)
        .verbose_logging(false)
        .build();
    
    let mut coordinator = FastExportCoordinator::new(config);
    
    // 创建进度回调
    let progress_callback = Box::new(|progress: ExportProgress| {
        println!("   📊 {} - {:.1}% ({}/{})", 
            progress.current_stage.description(),
            progress.overall_progress * 100.0,
            progress.processed_allocations,
            progress.total_allocations
        );
    });
    
    // 执行带进度监控的快速导出
    match coordinator.export_fast_with_progress("demo_progress_export", Some(progress_callback)) {
        Ok(stats) => {
            println!("   ✅ 导出完成:");
            println!("      总耗时: {}ms", stats.total_export_time_ms);
            println!("      处理分配: {}", stats.total_allocations_processed);
            println!("      性能提升: {:.2}x", stats.performance_improvement_factor);
        }
        Err(e) => {
            println!("   ❌ 导出失败: {}", e);
        }
    }
    
    Ok(())
}

/// 演示自定义进度回调
fn demo_custom_progress_callback() -> Result<(), Box<dyn std::error::Error>> {
    let mut monitor = ProgressMonitor::new(200);
    
    // 创建共享的进度状态
    let progress_state = Arc::new(Mutex::new(Vec::new()));
    let progress_state_clone = progress_state.clone();
    
    // 设置自定义进度回调
    monitor.set_callback(Box::new(move |progress| {
        let mut state = progress_state_clone.lock().unwrap();
        state.push((
            progress.current_stage.clone(),
            progress.overall_progress,
            progress.processing_speed,
            progress.estimated_remaining,
        ));
        
        // 只显示关键进度点
        if progress.overall_progress == 0.0 || 
           progress.overall_progress >= 1.0 || 
           (progress.overall_progress * 100.0) as u32 % 25 == 0 {
            println!("   🎯 阶段: {} | 进度: {:.1}% | 速度: {:.0} 分配/秒 | 剩余: {:?}",
                progress.current_stage.description(),
                progress.overall_progress * 100.0,
                progress.processing_speed,
                progress.estimated_remaining.map(|d| format!("{:.1}s", d.as_secs_f64())).unwrap_or("未知".to_string())
            );
        }
    }));
    
    // 模拟处理过程
    let stages = [
        ExportStage::Initializing,
        ExportStage::DataLocalization,
        ExportStage::ParallelProcessing,
        ExportStage::Writing,
    ];
    
    for stage in &stages {
        monitor.set_stage(stage.clone());
        
        for i in 0..=20 {
            let progress = i as f64 / 20.0;
            monitor.update_progress(progress, None);
            monitor.add_processed(10);
            thread::sleep(Duration::from_millis(25));
        }
    }
    
    monitor.complete();
    
    // 显示收集的进度统计
    let final_state = progress_state.lock().unwrap();
    println!("   📈 收集了 {} 个进度更新", final_state.len());
    
    Ok(())
}

/// 创建测试分配
fn create_test_allocations(_tracker: &MemoryTracker, count: usize) {
    use std::alloc::{alloc, dealloc, Layout};
    
    let mut allocations = Vec::new();
    
    for i in 0..count {
        let size = 64 + (i % 1000);
        let layout = Layout::from_size_align(size, 8).unwrap();
        
        unsafe {
            let ptr = alloc(layout);
            if !ptr.is_null() {
                allocations.push((ptr, layout));
                
                // 模拟一些释放
                if i % 4 == 0 && !allocations.is_empty() {
                    let (old_ptr, old_layout) = allocations.remove(0);
                    dealloc(old_ptr, old_layout);
                }
            }
        }
    }
    
    // 清理剩余分配
    for (ptr, layout) in allocations {
        unsafe {
            dealloc(ptr, layout);
        }
    }
    
    println!("   创建了 {} 个测试分配", count);
}