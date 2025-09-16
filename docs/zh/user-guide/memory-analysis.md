# 内存分析功能

memscope-rs 提供强大的内存分析功能，帮你深入理解程序的内存使用模式、识别潜在问题并优化性能。

## 🎯 分析功能概览

### 核心分析能力

| 分析类型 | 功能描述 | 适用场景 | 输出格式 |
|---------|---------|----------|----------|
| **基础统计** | 内存使用量、分配次数 | 日常监控 | 实时数据 |
| **生命周期分析** | 对象创建到销毁的完整轨迹 | 内存泄漏检测 | 时间线图 |
| **类型分析** | 不同数据类型的内存占用 | 结构优化 | 饼图/柱状图 |
| **引用分析** | 智能指针引用计数变化 | 循环引用检测 | 关系图 |
| **热点分析** | 高频分配的代码位置 | 性能优化 | 热力图 |

## 📊 基础统计分析

### 实时内存统计

```rust
use memscope_rs::{get_global_tracker, track_var, init};

fn basic_statistics_demo() {
    init();
    
    // 创建一些测试数据
    let data1 = vec![1; 1000];
    track_var!(data1);
    
    let data2 = String::from("Hello, Analysis!");
    track_var!(data2);
    
    let data3 = Box::new(vec![0u8; 2048]);
    track_var!(data3);
    
    // 获取详细统计信息
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("📈 内存统计报告:");
        println!("  活跃分配数量: {}", stats.active_allocations);
        println!("  活跃内存总量: {} bytes ({:.2} KB)", 
                stats.active_memory, 
                stats.active_memory as f64 / 1024.0);
        println!("  历史分配总数: {}", stats.total_allocations);
        println!("  历史释放总数: {}", stats.total_deallocations);
        println!("  内存使用峰值: {} bytes ({:.2} KB)", 
                stats.peak_memory,
                stats.peak_memory as f64 / 1024.0);
        println!("  平均分配大小: {:.2} bytes", 
                stats.active_memory as f64 / stats.active_allocations as f64);
    }
}
```

### 内存效率分析

```rust
use memscope_rs::{get_global_tracker, track_var, init};

fn memory_efficiency_analysis() {
    init();
    let tracker = get_global_tracker();
    
    // 记录基线
    let baseline = tracker.get_stats().unwrap();
    
    // 模拟不同的内存使用模式
    
    // 模式 1: 大块连续分配
    println!("🔍 分析模式 1: 大块连续分配");
    {
        let large_blocks: Vec<Vec<u8>> = (0..10)
            .map(|i| {
                let block = vec![i as u8; 10240]; // 10KB 每块
                track_var!(block);
                block
            })
            .collect();
        
        let after_large = tracker.get_stats().unwrap();
        println!("  大块分配效率: {:.2}%", 
                calculate_efficiency(&baseline, &after_large));
    }
    
    // 模式 2: 小块频繁分配
    println!("🔍 分析模式 2: 小块频繁分配");
    {
        let small_blocks: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                let block = vec![i as u8; 100]; // 100B 每块
                track_var!(block);
                block
            })
            .collect();
        
        let after_small = tracker.get_stats().unwrap();
        println!("  小块分配效率: {:.2}%", 
                calculate_efficiency(&baseline, &after_small));
    }
}

fn calculate_efficiency(baseline: &memscope_rs::MemoryStats, current: &memscope_rs::MemoryStats) -> f64 {
    let allocated_memory = current.active_memory - baseline.active_memory;
    let allocation_count = current.active_allocations - baseline.active_allocations;
    
    if allocation_count == 0 {
        return 100.0;
    }
    
    let average_size = allocated_memory as f64 / allocation_count as f64;
    let efficiency = (average_size / 1024.0).min(1.0) * 100.0; // 假设 1KB 为理想大小
    efficiency
}
```

## 🔄 生命周期分析

### 对象生命周期跟踪

```rust
use memscope_rs::{track_var_owned, get_global_tracker, init};
use std::rc::Rc;

fn lifecycle_analysis_demo() {
    init();
    println!("🔄 对象生命周期分析");
    
    // 阶段 1: 创建阶段
    println!("  📦 阶段 1: 对象创建");
    let creation_time = std::time::Instant::now();
    
    let long_lived_data = track_var_owned!(vec![1; 5000]);
    let short_lived_data = track_var_owned!(String::from("临时数据"));
    
    println!("    创建耗时: {:?}", creation_time.elapsed());
    
    // 阶段 2: 使用阶段
    println!("  🔧 阶段 2: 对象使用");
    let usage_time = std::time::Instant::now();
    
    // 模拟数据使用
    let _sum: i32 = long_lived_data.iter().sum();
    let _length = short_lived_data.len();
    
    println!("    使用耗时: {:?}", usage_time.elapsed());
    
    // 阶段 3: 部分清理
    println!("  🧹 阶段 3: 部分清理");
    drop(short_lived_data); // 显式释放短期数据
    
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("    清理后活跃内存: {} bytes", stats.active_memory);
    }
    
    // 阶段 4: 完全清理
    println!("  🗑️ 阶段 4: 完全清理");
    drop(long_lived_data);
    
    if let Ok(stats) = tracker.get_stats() {
        println!("    最终活跃内存: {} bytes", stats.active_memory);
    }
}
```

### 智能指针生命周期

```rust
use memscope_rs::{track_var, get_global_tracker, init};
use std::rc::Rc;
use std::sync::Arc;

fn smart_pointer_lifecycle() {
    init();
    println!("🔗 智能指针生命周期分析");
    
    // Rc 引用计数分析
    println!("  📊 Rc 引用计数分析:");
    {
        let original = Rc::new(vec![1, 2, 3, 4, 5]);
        track_var!(original);
        println!("    初始引用计数: {}", Rc::strong_count(&original));
        
        let clone1 = Rc::clone(&original);
        track_var!(clone1);
        println!("    第一次克隆后: {}", Rc::strong_count(&original));
        
        let clone2 = Rc::clone(&original);
        track_var!(clone2);
        println!("    第二次克隆后: {}", Rc::strong_count(&original));
        
        // 分析内存使用
        let tracker = get_global_tracker();
        if let Ok(stats) = tracker.get_stats() {
            println!("    当前活跃分配: {}", stats.active_allocations);
        }
        
        drop(clone1);
        println!("    释放 clone1 后: {}", Rc::strong_count(&original));
        
        drop(clone2);
        println!("    释放 clone2 后: {}", Rc::strong_count(&original));
    }
    
    // Arc 线程安全分析
    println!("  🧵 Arc 线程安全分析:");
    {
        let shared_data = Arc::new(vec![1; 1000]);
        track_var!(shared_data);
        
        let handles: Vec<_> = (0..3)
            .map(|i| {
                let data_clone = Arc::clone(&shared_data);
                track_var!(data_clone);
                
                std::thread::spawn(move || {
                    println!("    线程 {} 访问数据长度: {}", i, data_clone.len());
                    std::thread::sleep(std::time::Duration::from_millis(100));
                })
            })
            .collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        println!("    最终引用计数: {}", Arc::strong_count(&shared_data));
    }
}
```

## 📈 类型分析

### 数据类型内存占用分析

```rust
use memscope_rs::{track_var, get_global_tracker, init};
use std::collections::{HashMap, BTreeMap, HashSet};

fn type_analysis_demo() {
    init();
    println!("📊 数据类型内存分析");
    
    let tracker = get_global_tracker();
    let baseline = tracker.get_stats().unwrap();
    
    // 字符串类型分析
    println!("  📝 字符串类型:");
    let string_data = String::from("这是一个测试字符串，用于分析内存占用");
    track_var!(string_data);
    let after_string = tracker.get_stats().unwrap();
    println!("    String 内存增长: {} bytes", 
            after_string.active_memory - baseline.active_memory);
    
    // 向量类型分析
    println!("  📋 向量类型:");
    let vec_data = vec![1i32; 1000];
    track_var!(vec_data);
    let after_vec = tracker.get_stats().unwrap();
    println!("    Vec<i32> 内存增长: {} bytes", 
            after_vec.active_memory - after_string.active_memory);
    
    // HashMap 分析
    println!("  🗂️ HashMap 类型:");
    let mut map_data = HashMap::new();
    for i in 0..100 {
        map_data.insert(format!("key_{}", i), i);
    }
    track_var!(map_data);
    let after_map = tracker.get_stats().unwrap();
    println!("    HashMap 内存增长: {} bytes", 
            after_map.active_memory - after_vec.active_memory);
    
    // BTreeMap 对比分析
    println!("  🌳 BTreeMap 类型:");
    let mut btree_data = BTreeMap::new();
    for i in 0..100 {
        btree_data.insert(format!("key_{}", i), i);
    }
    track_var!(btree_data);
    let after_btree = tracker.get_stats().unwrap();
    println!("    BTreeMap 内存增长: {} bytes", 
            after_btree.active_memory - after_map.active_memory);
    
    // 生成类型分析报告
    generate_type_analysis_report();
}

fn generate_type_analysis_report() {
    let tracker = get_global_tracker();
    
    // 导出详细的类型分析
    if let Err(e) = tracker.export_to_json("type_analysis") {
        eprintln!("类型分析导出失败: {}", e);
    } else {
        println!("  ✅ 类型分析报告已生成: MemoryAnalysis/type_analysis/");
    }
}
```

## 🔍 热点分析

### 内存分配热点识别

```rust
use memscope_rs::{track_var, get_global_tracker, init};

fn hotspot_analysis_demo() {
    init();
    println!("🔥 内存分配热点分析");
    
    // 模拟不同的分配模式
    
    // 热点 1: 循环中的频繁分配
    println!("  🔄 热点 1: 循环分配");
    for i in 0..100 {
        let data = vec![i; 50];
        track_var!(data);
        
        // 模拟一些处理时间
        if i % 10 == 0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    
    // 热点 2: 递归函数中的分配
    println!("  🌀 热点 2: 递归分配");
    recursive_allocation(5, 100);
    
    // 热点 3: 条件分配
    println!("  🎯 热点 3: 条件分配");
    for i in 0..50 {
        if i % 3 == 0 {
            let large_data = vec![0u8; 1024];
            track_var!(large_data);
        } else {
            let small_data = vec![i as u8; 10];
            track_var!(small_data);
        }
    }
    
    // 生成热点分析报告
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("  📊 热点分析结果:");
        println!("    总分配次数: {}", stats.total_allocations);
        println!("    当前活跃分配: {}", stats.active_allocations);
        println!("    分配效率: {:.2}%", 
                (stats.active_allocations as f64 / stats.total_allocations as f64) * 100.0);
    }
    
    // 导出热点分析
    if let Err(e) = tracker.export_to_html("hotspot_analysis.html") {
        eprintln!("热点分析导出失败: {}", e);
    } else {
        println!("  ✅ 热点分析报告: MemoryAnalysis/hotspot_analysis/");
    }
}

fn recursive_allocation(depth: usize, size: usize) {
    if depth == 0 {
        return;
    }
    
    let data = vec![depth; size];
    track_var!(data);
    
    recursive_allocation(depth - 1, size / 2);
}
```

## 🔗 引用关系分析

### 循环引用检测

```rust
use memscope_rs::{track_var, get_global_tracker, init};
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Node {
    value: i32,
    children: RefCell<Vec<Rc<Node>>>,
    parent: RefCell<Weak<Node>>,
}

impl Node {
    fn new(value: i32) -> Rc<Self> {
        Rc::new(Node {
            value,
            children: RefCell::new(Vec::new()),
            parent: RefCell::new(Weak::new()),
        })
    }
    
    fn add_child(&self, child: Rc<Node>) {
        child.parent.borrow_mut().clone_from(&Rc::downgrade(&Rc::new(self.clone())));
        self.children.borrow_mut().push(child);
    }
}

fn reference_analysis_demo() {
    init();
    println!("🔗 引用关系分析");
    
    let tracker = get_global_tracker();
    let baseline = tracker.get_stats().unwrap();
    
    // 创建节点树结构
    println!("  🌳 创建节点树:");
    let root = Node::new(0);
    track_var!(root);
    
    let child1 = Node::new(1);
    track_var!(child1);
    
    let child2 = Node::new(2);
    track_var!(child2);
    
    let grandchild = Node::new(3);
    track_var!(grandchild);
    
    // 建立父子关系
    root.add_child(child1.clone());
    root.add_child(child2.clone());
    child1.add_child(grandchild.clone());
    
    let after_tree = tracker.get_stats().unwrap();
    println!("    树结构内存使用: {} bytes", 
            after_tree.active_memory - baseline.active_memory);
    
    // 分析引用计数
    println!("  📊 引用计数分析:");
    println!("    root 引用计数: {}", Rc::strong_count(&root));
    println!("    child1 引用计数: {}", Rc::strong_count(&child1));
    println!("    child2 引用计数: {}", Rc::strong_count(&child2));
    println!("    grandchild 引用计数: {}", Rc::strong_count(&grandchild));
    
    // 检测潜在的循环引用
    detect_potential_cycles(&root);
    
    // 清理分析
    drop(grandchild);
    drop(child2);
    drop(child1);
    drop(root);
    
    let after_cleanup = tracker.get_stats().unwrap();
    println!("  🧹 清理后内存: {} bytes", 
            after_cleanup.active_memory - baseline.active_memory);
}

fn detect_potential_cycles(node: &Rc<Node>) {
    println!("  🔍 循环引用检测:");
    
    // 简单的循环引用检测逻辑
    let strong_refs = Rc::strong_count(node);
    let weak_refs = Rc::weak_count(node);
    
    println!("    节点 {} - 强引用: {}, 弱引用: {}", 
            node.value, strong_refs, weak_refs);
    
    if strong_refs > 2 {
        println!("    ⚠️ 警告: 节点 {} 可能存在循环引用", node.value);
    }
    
    // 递归检查子节点
    for child in node.children.borrow().iter() {
        detect_potential_cycles(child);
    }
}
```

## 📊 高级分析技巧

### 内存使用模式识别

```rust
use memscope_rs::{get_global_tracker, track_var, init};
use std::time::{Duration, Instant};

fn pattern_analysis_demo() {
    init();
    println!("🎯 内存使用模式识别");
    
    let tracker = get_global_tracker();
    
    // 模式 1: 稳定增长模式
    println!("  📈 模式 1: 稳定增长");
    analyze_steady_growth_pattern();
    
    // 模式 2: 波动模式
    println!("  🌊 模式 2: 波动使用");
    analyze_fluctuation_pattern();
    
    // 模式 3: 峰值模式
    println!("  ⛰️ 模式 3: 峰值使用");
    analyze_peak_pattern();
    
    // 生成模式分析报告
    if let Err(e) = tracker.export_to_html("pattern_analysis.html") {
        eprintln!("模式分析导出失败: {}", e);
    } else {
        println!("  ✅ 模式分析报告: MemoryAnalysis/pattern_analysis/");
    }
}

fn analyze_steady_growth_pattern() {
    let mut data_store = Vec::new();
    
    for i in 0..20 {
        let data = vec![i; 100 * (i + 1)]; // 逐渐增大
        track_var!(data);
        data_store.push(data);
        
        std::thread::sleep(Duration::from_millis(50));
    }
    
    println!("    稳定增长模式完成");
}

fn analyze_fluctuation_pattern() {
    for i in 0..30 {
        let size = if i % 2 == 0 { 1000 } else { 100 };
        let data = vec![i; size];
        track_var!(data);
        
        std::thread::sleep(Duration::from_millis(30));
    }
    
    println!("    波动模式完成");
}

fn analyze_peak_pattern() {
    // 正常使用
    for i in 0..10 {
        let data = vec![i; 100];
        track_var!(data);
    }
    
    // 突然的大分配（峰值）
    let peak_data = vec![0u8; 50000];
    track_var!(peak_data);
    
    // 回到正常使用
    for i in 10..20 {
        let data = vec![i; 100];
        track_var!(data);
    }
    
    println!("    峰值模式完成");
}
```

## 🚀 分析结果应用

### 基于分析结果的优化建议

```rust
use memscope_rs::{get_global_tracker, MemoryStats};

fn generate_optimization_suggestions(stats: &MemoryStats) {
    println!("🎯 优化建议:");
    
    // 内存使用效率分析
    let efficiency = (stats.active_memory as f64 / stats.peak_memory as f64) * 100.0;
    
    if efficiency < 70.0 {
        println!("  ⚠️ 内存使用效率较低 ({:.1}%)", efficiency);
        println!("     建议: 考虑使用对象池或减少临时分配");
    }
    
    // 分配频率分析
    let avg_allocation_size = stats.active_memory as f64 / stats.active_allocations as f64;
    
    if avg_allocation_size < 100.0 {
        println!("  ⚠️ 平均分配大小较小 ({:.1} bytes)", avg_allocation_size);
        println!("     建议: 考虑批量分配或使用更大的缓冲区");
    }
    
    // 内存泄漏风险评估
    let deallocation_ratio = stats.total_deallocations as f64 / stats.total_allocations as f64;
    
    if deallocation_ratio < 0.8 {
        println!("  🚨 潜在内存泄漏风险 (释放率: {:.1}%)", deallocation_ratio * 100.0);
        println!("     建议: 检查长期存活的对象和循环引用");
    }
    
    // 性能优化建议
    if stats.total_allocations > 10000 {
        println!("  🏃 高频分配检测到 ({} 次)", stats.total_allocations);
        println!("     建议: 考虑预分配或使用栈分配");
    }
}

fn comprehensive_analysis_example() {
    init();
    let tracker = get_global_tracker();
    
    // 运行一些内存操作
    simulate_application_workload();
    
    // 获取最终统计
    if let Ok(stats) = tracker.get_stats() {
        println!("\n📊 综合分析报告:");
        println!("  活跃分配: {}", stats.active_allocations);
        println!("  活跃内存: {} KB", stats.active_memory / 1024);
        println!("  峰值内存: {} KB", stats.peak_memory / 1024);
        println!("  总分配次数: {}", stats.total_allocations);
        
        // 生成优化建议
        generate_optimization_suggestions(&stats);
    }
    
    // 导出完整分析报告
    let _ = tracker.export_to_html("comprehensive_analysis.html");
    println!("\n✅ 完整分析报告已生成!");
}

fn simulate_application_workload() {
    // 模拟真实应用的内存使用模式
    for _ in 0..100 {
        let data = vec![0u8; 1024];
        track_var!(data);
    }
}
```

## 🚀 下一步

现在你已经掌握了 memscope-rs 的高级分析功能，可以继续学习：

- **[导出格式说明](export-formats.md)** - 选择最适合的报告格式
- **[CLI 工具](cli-tools.md)** - 使用命令行进行批量分析
- **[性能优化指南](../advanced/performance-optimization.md)** - 系统性的优化方法

## 💡 关键要点

- **多维度分析** - 结合统计、生命周期、类型等多个角度
- **模式识别** - 识别常见的内存使用模式和问题
- **真实数据** - 在程序运行时持续跟踪内存状态
- **可视化报告** - 使用图表和仪表板直观展示分析结果
- **优化指导** - 基于分析结果进行专门的优化

掌握这些分析技巧，让你的 Rust 程序内存使用更加高效！ 🎯