# 基础使用示例

本文档基于 `examples/basic_usage.rs` 提供详细的使用说明和最佳实践。

## 🎯 完整示例解析

### 基础设置
```rust
use memscope_rs::{get_global_tracker, init, track_var};
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    // 1. 初始化内存跟踪系统
    init();
    println!("memscope-rs initialized. Tracking memory allocations...");
```

**关键点**:
- `init()` 必须在任何跟踪操作之前调用
- 只需要调用一次，通常在 `main()` 函数开始处
- 初始化后全局分配器开始工作

### 基础类型跟踪
```rust
    // 2. 分配和跟踪简单类型
    println!("\nAllocating and tracking variables...");

    let numbers_vec = vec![1, 2, 3, 4, 5];
    track_var!(numbers_vec);
    println!("Tracked 'numbers_vec'");

    let text_string = String::from("Hello, Trace Tools!");
    track_var!(text_string);
    println!("Tracked 'text_string'");

    let boxed_value = Box::new(100i32);
    track_var!(boxed_value);
    println!("Tracked 'boxed_value'");
```

**解释**:
- `Vec<T>` - 动态数组，在堆上分配数据
- `String` - 动态字符串，内容存储在堆上
- `Box<T>` - 智能指针，将数据分配到堆上

**内存布局**:
```
Stack (栈)          Heap (堆)
┌─────────────┐    ┌─────────────────┐
│ numbers_vec │───▶│ [1, 2, 3, 4, 5] │
├─────────────┤    ├─────────────────┤
│ text_string │───▶│ "Hello, Trace..." │
├─────────────┤    ├─────────────────┤
│ boxed_value │───▶│      100        │
└─────────────┘    └─────────────────┘
```

### 智能指针跟踪
```rust
    // 3. 跟踪引用计数类型
    let rc_data = Rc::new(vec![10, 20, 30]);
    track_var!(rc_data);
    println!("Tracked 'rc_data'");

    let arc_data = Arc::new(String::from("Shared data"));
    track_var!(arc_data);
    println!("Tracked 'arc_data'");

    // 克隆 Rc 显示共享所有权
    let rc_data_clone = Rc::clone(&rc_data);
    track_var!(rc_data_clone);
    println!("Tracked 'rc_data_clone' (shares allocation with 'rc_data')");
```

**智能指针特性**:
- `Rc<T>` - 单线程引用计数智能指针
- `Arc<T>` - 多线程安全的引用计数智能指针
- 克隆只增加引用计数，不复制数据

**引用计数跟踪**:
```
初始状态: rc_data (引用计数: 1)
克隆后:   rc_data (引用计数: 2) ←─┐
         rc_data_clone ─────────┘
         (共享同一块堆内存)
```

### 变量正常使用
```rust
    // 4. 执行一些操作（变量保持完全可用）
    let sum_of_vec = numbers_vec.iter().sum::<i32>();
    println!("\nSum of 'numbers_vec': {sum_of_vec}");
    println!("Length of 'text_string': {}", text_string.len());
    println!("Value in 'boxed_value': {}", *boxed_value);
    println!("First element of 'rc_data': {}", rc_data[0]);
    println!("Content of 'arc_data': {}", *arc_data);
```

**重要特性**:
- 跟踪后变量完全正常使用
- 零性能开销
- 无所有权变化

### 获取内存统计
```rust
    // 5. 获取内存统计信息
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("\nMemory Statistics:");
        println!("  Active allocations: {}", stats.active_allocations);
        println!("  Active memory: {} bytes", stats.active_memory);
        println!("  Total allocations: {}", stats.total_allocations);
        println!("  Peak memory: {} bytes", stats.peak_memory);
    }
```

**统计信息解释**:
- `active_allocations` - 当前活跃的分配数量
- `active_memory` - 当前使用的内存总量
- `total_allocations` - 程序运行期间的总分配次数
- `peak_memory` - 内存使用的峰值

### 导出分析结果
```rust
    // 6. 导出内存快照到 JSON
    println!("\nExporting memory snapshot to MemoryAnalysis/basic_usage/...");
    if let Err(e) = tracker.export_to_json("basic_usage_snapshot") {
        eprintln!("Failed to export JSON: {e}");
    } else {
        println!("Successfully exported JSON to MemoryAnalysis/basic_usage/");
    }

    // 7. 导出内存使用可视化到 SVG
    println!("\nExporting memory usage visualization to MemoryAnalysis/basic_usage/...");
    if let Err(e) = tracker.export_memory_analysis("basic_usage_graph.svg") {
        eprintln!("Failed to export SVG: {e}");
    } else {
        println!("Successfully exported SVG to MemoryAnalysis/basic_usage/");
    }
```

## 🔍 运行结果分析

### 控制台输出示例
```
memscope-rs initialized. Tracking memory allocations...

Allocating and tracking variables...
Tracked 'numbers_vec'
Tracked 'text_string'
Tracked 'boxed_value'
Tracked 'boxed_value2'
Tracked 'rc_data'
Tracked 'arc_data'
Tracked 'rc_data_clone' (shares allocation with 'rc_data')

Sum of 'numbers_vec': 15
Length of 'text_string': 19
Value in 'boxed_value': 100
Value in 'boxed_value2': 200
First element of 'rc_data': 10
Content of 'arc_data': Shared data

Memory Statistics:
  Active allocations: 7
  Active memory: 234 bytes
  Total allocations: 7
  Peak memory: 234 bytes

Exporting memory snapshot to MemoryAnalysis/basic_usage/...
Successfully exported JSON to MemoryAnalysis/basic_usage/

Exporting memory usage visualization to MemoryAnalysis/basic_usage/...
Successfully exported SVG to MemoryAnalysis/basic_usage/

Example finished. Check 'basic_usage_snapshot.json' and 'basic_usage_graph.svg'.
The SVG shows memory usage by type and individual allocations.
```

### 生成的文件
```
MemoryAnalysis/basic_usage/
├── basic_usage_snapshot_memory_analysis.json  # 基础内存分析
├── basic_usage_snapshot_lifetime.json         # 生命周期数据
├── basic_usage_snapshot_performance.json      # 性能数据
├── basic_usage_snapshot_unsafe_ffi.json       # Unsafe/FFI数据
├── basic_usage_snapshot_complex_types.json    # 复杂类型分析
└── basic_usage_graph.svg                      # 可视化图表
```

### 使用make命令生成HTML报告
```bash
# 运行示例
cargo run --example basic_usage

# 生成HTML报告
make html DIR=MemoryAnalysis/basic_usage BASE=basic_usage_snapshot

# 打开报告
open memory_report.html
```

## 📊 内存分析详解

### JSON 数据结构
生成的 JSON 文件包含：

```json
{
  "metadata": {
    "export_timestamp": 1691234567890,
    "total_allocations": 5,
    "active_allocations": 5
  },
  "allocations": [
    {
      "ptr": 140712345678912,
      "size": 40,
      "var_name": "numbers_vec",
      "type_name": "Vec<i32>",
      "timestamp_alloc": 1691234567123,
      "is_leaked": false
    },
    {
      "ptr": 140712345678952,
      "size": 19,
      "var_name": "text_string", 
      "type_name": "String",
      "timestamp_alloc": 1691234567124,
      "is_leaked": false
    }
    // ... 更多分配信息
  ]
}
```

### SVG 可视化
生成的 SVG 图表显示：
- 内存使用按类型分布
- 分配时间线
- 内存大小对比

## 🚀 扩展示例

### 添加更多跟踪
```rust
use memscope_rs::{track_var, init, get_global_tracker};
use std::collections::{HashMap, VecDeque};

fn extended_example() {
    init();
    
    // 集合类型
    let mut map = HashMap::new();
    map.insert("key1", "value1");
    map.insert("key2", "value2");
    track_var!(map);
    
    let mut deque = VecDeque::new();
    deque.push_back(1);
    deque.push_back(2);
    track_var!(deque);
    
    // 嵌套结构
    let nested = vec![vec![1, 2], vec![3, 4, 5]];
    track_var!(nested);
    
    // 大型分配
    let large_buffer = vec![0u8; 1024 * 1024]; // 1MB
    track_var!(large_buffer);
    
    // 导出详细分析
    let tracker = get_global_tracker();
    tracker.export_to_html("extended_analysis.html").unwrap();
}
```

### 函数级别跟踪
```rust
fn process_data(input: Vec<i32>) -> Vec<i32> {
    track_var!(input);
    
    let mut result = Vec::with_capacity(input.len());
    track_var!(result);
    
    for item in input {
        result.push(item * 2);
    }
    
    result
}

fn main() {
    init();
    
    let data = vec![1, 2, 3, 4, 5];
    let processed = process_data(data);
    track_var!(processed);
    
    let tracker = get_global_tracker();
    tracker.export_to_json("function_level_tracking").unwrap();
}
```

### 生命周期分析
```rust
fn lifecycle_example() {
    init();
    
    {
        let short_lived = vec![1, 2, 3];
        track_var!(short_lived);
        // short_lived 在这里被销毁
    }
    
    let long_lived = vec![4, 5, 6];
    track_var!(long_lived);
    
    // 导出时可以看到不同的生命周期模式
    let tracker = get_global_tracker();
    tracker.export_to_html("lifecycle_analysis.html").unwrap();
}
```

## 💡 最佳实践

### 1. 初始化时机
```rust
// ✅ 好的做法
fn main() {
    memscope_rs::init(); // 在程序开始时初始化
    
    // 你的程序逻辑...
}

// ❌ 避免的做法
fn some_function() {
    memscope_rs::init(); // 不要在函数中重复初始化
}
```

### 2. 跟踪策略
```rust
// ✅ 跟踪关键的堆分配
let important_data = vec![1, 2, 3];
track_var!(important_data);

// ✅ 跟踪大型分配
let large_buffer = vec![0; 1024 * 1024];
track_var!(large_buffer);

// ❌ 不需要跟踪栈上的简单值
let simple_int = 42; // 不需要跟踪
```

### 3. 导出时机
```rust
// ✅ 在程序结束前导出
fn main() {
    init();
    
    // 程序逻辑...
    
    // 导出分析结果
    let tracker = get_global_tracker();
    tracker.export_to_html("final_analysis.html").unwrap();
}
```

### 4. 错误处理
```rust
// ✅ 适当的错误处理
let tracker = get_global_tracker();
match tracker.export_to_json("analysis") {
    Ok(_) => println!("导出成功"),
    Err(e) => eprintln!("导出失败: {}", e),
}
```

这个基础示例为你提供了使用 memscope-rs 的完整起点。从这里开始，你可以探索更高级的功能！ 🎯