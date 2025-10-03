# 单线程模块：零开销内存跟踪

单线程模块是**大多数应用程序的推荐起点**。它通过 `track_var!` 系列宏提供精确的零开销内存跟踪。

## 🎯 适用场景

**✅ 完美适用于：**
- 开发和调试
- 单线程应用程序
- 线程数 < 10 的应用程序
- 需要精确跟踪数据的场景
- 学习和试验 memscope-rs

**❌ 考虑其他模块：**
- 高并发应用程序（20+ 线程）
- 性能关键的生产系统
- 近似数据足够的场景

## 🧩 核心跟踪宏

单线程模块提供三个专门的跟踪宏：

### 1. `track_var!` - **[推荐]**

通过引用进行零成本跟踪。变量保持完全可用。

```rust
use memscope_rs::{init, track_var, get_global_tracker};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    // 创建并跟踪变量
    let my_vec = vec![1, 2, 3, 4, 5];
    track_var!(my_vec);  // 零成本跟踪
    
    let my_string = String::from("你好，memscope！");
    track_var!(my_string);
    
    let my_box = Box::new(42);
    track_var!(my_box);
    
    // 变量正常工作 - 跟踪是透明的
    println!("向量: {:?}", my_vec);
    println!("字符串: {}", my_string);
    println!("Box: {}", *my_box);
    
    // 导出分析结果
    let tracker = get_global_tracker();
    tracker.export_to_json("analysis")?;
    tracker.export_to_html("analysis.html")?;
    
    Ok(())
}
```

**性能：** 真正的零开销 - 无克隆、无包装器、无所有权变更。

### 2. `track_var_smart!` - **[智能]**

根据类型自动选择最佳跟踪策略：

```rust
use memscope_rs::{track_var_smart, init};
use std::rc::Rc;

fn main() {
    init();
    
    // Copy 类型 - 自动复制（便宜）
    let number = 42i32;
    track_var_smart!(number);
    
    // 非 Copy 类型 - 引用跟踪（零成本）
    let text = String::from("你好");
    track_var_smart!(text);
    
    // 智能指针 - 克隆指针（便宜的引用递增）
    let rc_data = Rc::new(vec![1, 2, 3]);
    track_var_smart!(rc_data);
    
    // 所有变量都保持完全可用！
    println!("{}, {}, {:?}", number, text, rc_data);
}
```

**智能性：**
- `Copy` 类型（i32, f64, bool）：创建副本
- 非 `Copy` 类型：引用跟踪
- 智能指针（Rc, Arc）：克隆指针

### 3. `track_var_owned!` - **[高级]**

带所有权转移的完整生命周期管理：

```rust
use memscope_rs::{track_var_owned, init};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    let data = vec![1, 2, 3, 4, 5];
    let tracked = track_var_owned!(data);  // 获取所有权
    
    // 通过包装器方法访问
    println!("长度: {}", tracked.len());
    println!("第一个: {}", tracked[0]);
    
    // 需要时提取原始值
    let original = tracked.into_inner();
    println!("提取的: {:?}", original);
    
    Ok(())
}
```

**特性：**
- 精确的生命周期跟踪
- 自动清理检测
- 防重复的 Drop 保护
- 智能指针检测

## 📊 智能指针支持

所有跟踪宏都对 Rust 的智能指针有特殊处理：

```rust
use memscope_rs::{track_var, init};
use std::rc::Rc;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    // 引用计数指针
    let rc_data = Rc::new(vec![1, 2, 3]);
    track_var!(rc_data);
    
    // 克隆操作被跟踪
    let rc_clone = Rc::clone(&rc_data);
    track_var!(rc_clone);
    
    // 原子引用计数（线程安全）
    let arc_data = Arc::new(String::from("共享数据"));
    track_var!(arc_data);
    
    // 堆分配
    let boxed = Box::new(42);
    track_var!(boxed);
    
    // 导出智能指针分析
    let tracker = get_global_tracker();
    tracker.export_to_json("smart_pointers")?;
    
    Ok(())
}
```

## 🔧 导出和分析

### JSON 导出 - 详细分析

```rust
use memscope_rs::{get_global_tracker, ExportOptions};

fn export_detailed_analysis() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = get_global_tracker();
    
    // 基本导出
    tracker.export_to_json("basic_analysis")?;
    
    // 配置导出
    let options = ExportOptions::new()
        .include_system_allocations(false)  // 跳过系统分配
        .verbose_logging(true)              // 详细日志
        .buffer_size(128 * 1024);           // 128KB 缓冲区
    
    tracker.export_to_json_with_options("detailed_analysis", options)?;
    
    // 优化导出（最佳性能）
    let result = tracker.export_to_json_optimized("optimized_analysis")?;
    println!("导出在 {:.2}ms 内完成", result.export_stats.export_time_ms);
    
    Ok(())
}
```

### HTML 仪表板 - 交互式可视化

```rust
use memscope_rs::get_global_tracker;

fn generate_html_dashboard() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = get_global_tracker();
    
    // 生成交互式 HTML 仪表板
    tracker.export_to_html("memory_dashboard.html")?;
    
    println!("📊 交互式仪表板已生成: memory_dashboard.html");
    println!("   - 内存时间线图表");
    println!("   - 变量生命周期分析");
    println!("   - 智能指针引用跟踪");
    println!("   - 内存泄漏检测");
    
    Ok(())
}
```

## ⚡ 性能特征

### 跟踪开销

| 宏 | 开销 | 使用场景 |
|-------|----------|----------|
| `track_var!` | **零** | 生产环境推荐 |
| `track_var_smart!` | **最小** | 混合类型 |
| `track_var_owned!` | **包装器** | 精确分析 |

### 导出性能（真实数据）

基于实际测试跟踪 1000+ 变量：

| 格式 | 导出时间 | 文件大小 | 特性 |
|--------|-------------|-----------|----------|
| **JSON** | 1.3s | 1.2MB | 详细分析，可读 |
| **HTML** | 800ms | 2.1MB | 交互式仪表板 |
| **二进制** | 211ms | 480KB | 高性能 |

## 🛡️ 安全特性

### 自动类型检测

```rust
use memscope_rs::track_var;

fn test_type_detection() {
    // 基本类型
    let number = 42i32;
    track_var!(number);  // 生成合成指针
    
    // 堆分配类型
    let vector = vec![1, 2, 3];
    track_var!(vector);  // 使用真实堆指针
    
    // 智能指针
    let rc = Rc::new(vector);
    track_var!(rc);      // 智能指针跟踪
}
```

### 错误处理

```rust
use memscope_rs::{get_global_tracker, TrackingResult};

fn robust_tracking() -> TrackingResult<()> {
    let tracker = get_global_tracker();
    
    // 测试快速模式
    tracker.enable_fast_mode();
    
    let data = vec![1, 2, 3];
    track_var!(data);
    
    // 带错误处理的导出
    match tracker.export_to_json("analysis") {
        Ok(_) => println!("✅ 导出成功"),
        Err(e) => eprintln!("❌ 导出失败: {}", e),
    }
    
    Ok(())
}
```

## 🎮 完整示例

```rust
use memscope_rs::{init, track_var, track_var_smart, track_var_owned, get_global_tracker};
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化跟踪
    init();
    
    // 不同的跟踪策略
    let basic_data = vec![1, 2, 3, 4, 5];
    track_var!(basic_data);  // 零成本引用跟踪
    
    let smart_data = String::from("你好，世界！");
    track_var_smart!(smart_data);  // 智能跟踪
    
    let owned_data = vec![10, 20, 30];
    let tracked = track_var_owned!(owned_data);  // 完整生命周期
    
    // 智能指针跟踪
    let rc_data = Rc::new(vec![100, 200, 300]);
    track_var!(rc_data);
    let rc_clone = Rc::clone(&rc_data);
    track_var!(rc_clone);
    
    // 正常使用所有变量
    println!("基本: {:?}", basic_data);
    println!("智能: {}", smart_data);
    println!("跟踪: {:?}", *tracked);
    println!("RC 计数: {}", Rc::strong_count(&rc_data));
    
    // 导出综合分析
    let tracker = get_global_tracker();
    tracker.export_to_json("comprehensive_analysis")?;
    tracker.export_to_html("dashboard.html")?;
    
    println!("🎯 分析完成！");
    println!("📁 JSON: comprehensive_analysis.json");
    println!("📊 仪表板: dashboard.html");
    
    Ok(())
}
```

## 🔗 下一步

- **[多线程模块](multithread.md)** - 高并发跟踪
- **[异步模块](async.md)** - 任务中心分析
- **[API 参考](api-reference/tracking-api.md)** - 完整 API 文档
- **[示例](examples/basic-usage.md)** - 更多详细示例