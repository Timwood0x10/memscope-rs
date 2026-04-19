# 快速开始

## 安装

将 memscope-rs 添加到你的 `Cargo.toml`：

```toml
[dependencies]
memscope-rs = "0.2"
```

## 基础使用

### 1. 初始化跟踪器

```rust
use memscope_rs::{global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    // 初始化全局跟踪器
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // 你的代码...

    Ok(())
}
```

### 2. 跟踪变量

```rust
use memscope_rs::track;

fn main() -> memscope_rs::MemScopeResult<()> {
    let tracker = global_tracker()?;

    // 跟踪分配
    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);

    let string_data = String::from("Hello, world!");
    track!(tracker, string_data);

    Ok(())
}
```

### 3. 分析内存使用

```rust
use memscope_rs::analyzer;

fn main() -> memscope_rs::MemScopeResult<()> {
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3];
    track!(tracker, data);

    // 分析内存使用
    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    println!("总分配数: {}", report.stats.allocation_count);
    println!("总字节数: {}", report.stats.total_bytes);
    println!("峰值内存: {} bytes", report.stats.peak_bytes);

    Ok(())
}
```

### 4. 导出报告

```rust
fn main() -> memscope_rs::MemScopeResult<()> {
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3];
    track!(tracker, data);

    // 导出 JSON 报告
    tracker.export_all_json("output")?;

    // 导出 HTML Dashboard
    tracker.render_unified_dashboard("output/dashboard.html")?;

    Ok(())
}
```

## 智能指针追踪

### 自动检测

```rust
use std::rc::Rc;
use std::sync::Arc;

fn main() -> memscope_rs::MemScopeResult<()> {
    let tracker = global_tracker()?;

    // 自动检测智能指针类型
    let rc_data = Rc::new(vec![1, 2, 3]);
    track!(tracker, rc_data);

    let arc_data = Arc::new(vec![4, 5, 6]);
    track!(tracker, arc_data);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    println!("智能指针数量: {}", report.circular_references.total_smart_pointers);
    println!("循环引用数量: {}", report.circular_references.count);

    Ok(())
}
```

### 循环引用检测

```rust
use std::cell::RefCell;
use std::rc::Rc;

struct Node {
    data: i32,
    next: Option<Rc<RefCell<Node>>>,
}

fn main() -> memscope_rs::MemScopeResult<()> {
    let tracker = global_tracker()?;

    // 创建循环引用
    let node1 = Rc::new(RefCell::new(Node { data: 1, next: None }));
    let node2 = Rc::new(RefCell::new(Node { data: 2, next: None }));

    node1.borrow_mut().next = Some(node2.clone());
    node2.borrow_mut().next = Some(node1.clone());

    track!(tracker, node1);
    track!(tracker, node2);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    if report.circular_references.has_cycles {
        println!("检测到循环引用!");
        println!("泄漏内存: {} bytes", report.circular_references.total_leaked_memory);
    }

    Ok(())
}
```

## 生命周期分析

### 自动追踪

```rust
fn main() -> memscope_rs::MemScopeResult<()> {
    let tracker = global_tracker()?;

    {
        let temporary = vec![1, 2, 3];
        track!(tracker, temporary);
        // temporary 在此处离开作用域，生命周期自动记录
    }

    let long_lived = vec![1; 1000];
    track!(tracker, long_lived);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    // 查看生命周期
    for alloc in &report.allocations {
        println!("{}: lifetime_ms = {}", alloc.type_name, alloc.lifetime_ms);
    }

    Ok(())
}
```

## 性能优化

### 采样模式

```rust
use memscope_rs::{tracker, tracker::SamplingConfig};

fn main() -> memscope_rs::MemScopeResult<()> {
    // 使用高性能采样模式（1% 采样率）
    let tracker = tracker!()
        .with_sampling(SamplingConfig::high_performance());

    for i in 0..10000 {
        let data = vec![i; 100];
        track!(tracker, data);
    }

    Ok(())
}
```

## 完整示例

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};
use std::rc::Rc;
use std::sync::Arc;

fn main() -> memscope_rs::MemScopeResult<()> {
    // 1. 初始化
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // 2. 跟踪各种类型
    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);

    let string_data = String::from("Hello");
    track!(tracker, string_data);

    let rc_data = Rc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, rc_data);

    let arc_data = Arc::new(vec![4.0, 5.0, 6.0]);
    track!(tracker, arc_data);

    // 3. 分析
    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    println!("=== 内存分析报告 ===");
    println!("总分配数: {}", report.stats.allocation_count);
    println!("总字节数: {}", report.stats.total_bytes);
    println!("峰值内存: {} bytes", report.stats.peak_bytes);
    println!("线程数: {}", report.stats.thread_count);
    println!("\n=== 智能指针 ===");
    println!("智能指针数量: {}", report.circular_references.total_smart_pointers);
    println!("循环引用数量: {}", report.circular_references.count);

    // 4. 导出
    tracker.export_all_json("output")?;
    tracker.render_unified_dashboard("output/dashboard.html")?;

    println!("\n报告已导出到 output/ 目录");

    Ok(())
}
```

## 下一步

- 查看 [编译期语义增强](./compile-time-enhancement.md) 了解详细功能
- 查看 [智能指针追踪](./smart-pointer-tracking.md) 了解循环引用检测
- 查看 [API 文档](./api.md) 了解完整的 API 参考
