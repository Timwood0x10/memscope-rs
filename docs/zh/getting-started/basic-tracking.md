# 基础跟踪使用

本指南将深入介绍 memscope-rs 的核心跟踪功能，帮你掌握三种跟踪宏的使用方法和最佳实践。

## 🎯 学习目标

完成本指南后，你将能够：
- 理解三种跟踪宏的区别和适用场景
- 掌握不同数据类型的跟踪技巧
- 了解跟踪的性能影响和优化方法
- 学会调试跟踪相关问题

## 📊 三种跟踪宏概览

### `track_var!` - 零开销跟踪 [推荐]

**特点**: 编译时优化，运行时零开销
**适用**: 生产环境、性能敏感场景

```rust
use memscope_rs::{track_var, init, get_global_tracker};

fn main() {
    init();
    
    let my_vec = vec![1, 2, 3, 4, 5];
    track_var!(my_vec);  // 零开销，变量不变
    
    // 变量完全正常使用
    println!("Length: {}", my_vec.len());
    for item in &my_vec {
        println!("Item: {}", item);
    }
}
```

### `track_var_smart!` - 智能跟踪

**特点**: 自动处理不同类型，返回原值
**适用**: 开发调试、混合类型场景

```rust
use memscope_rs::{track_var_smart, init};

fn main() {
    init();
    
    // 自动适配不同类型
    let numbers = track_var_smart!(vec![1, 2, 3]);
    let text = track_var_smart!(String::from("Hello"));
    let boxed = track_var_smart!(Box::new(42));
    
    // 返回值可以直接使用
    println!("Numbers: {:?}", numbers);
}
```

### `track_var_owned!` - 所有权跟踪

**特点**: 获取所有权，精确生命周期控制
**适用**: 生命周期分析、内存泄漏检测

```rust
use memscope_rs::{track_var_owned, init};

fn main() {
    init();
    
    let data = vec![1, 2, 3, 4, 5];
    let tracked_data = track_var_owned!(data);
    // 注意：原 data 变量已被移动
    
    // 使用包装后的数据
    println!("Data: {:?}", *tracked_data);
}
```

## 🔍 数据类型跟踪详解

### 基础类型

```rust
use memscope_rs::{track_var, init};

fn main() {
    init();
    
    // 数值类型
    let number = 42i32;
    track_var!(number);
    
    // 字符串类型
    let text = String::from("Hello, World!");
    track_var!(text);
    
    // 数组和向量
    let array = [1, 2, 3, 4, 5];
    track_var!(array);
    
    let vector = vec!["a", "b", "c"];
    track_var!(vector);
}
```

### 智能指针跟踪

```rust
use memscope_rs::{track_var, init};
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    init();
    
    // Box 指针
    let boxed_data = Box::new(vec![1, 2, 3]);
    track_var!(boxed_data);
    
    // Rc 引用计数指针
    let rc_data = Rc::new(String::from("Shared data"));
    track_var!(rc_data);
    
    let rc_clone = Rc::clone(&rc_data);
    track_var!(rc_clone);  // 跟踪引用计数变化
    
    // Arc 原子引用计数指针
    let arc_data = Arc::new(vec![1, 2, 3, 4]);
    track_var!(arc_data);
}
```

### 复杂数据结构

```rust
use memscope_rs::{track_var, init};
use std::collections::HashMap;

fn main() {
    init();
    
    // HashMap
    let mut map = HashMap::new();
    map.insert("key1", "value1");
    map.insert("key2", "value2");
    track_var!(map);
    
    // 嵌套结构
    let nested = vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ];
    track_var!(nested);
    
    // 自定义结构体
    #[derive(Debug)]
    struct Person {
        name: String,
        age: u32,
        hobbies: Vec<String>,
    }
    
    let person = Person {
        name: String::from("Alice"),
        age: 30,
        hobbies: vec![
            String::from("Reading"),
            String::from("Coding"),
        ],
    };
    track_var!(person);
}
```

## ⚡ 性能考虑

### 零开销原理

`track_var!` 宏在编译时被优化掉，运行时没有任何开销：

```rust
// 编译前
track_var!(my_data);

// 编译后（简化）
// 仅在调试模式下记录元数据，发布模式下完全移除
```

### 性能对比测试

```rust
use memscope_rs::{track_var, track_var_smart, track_var_owned, init};
use std::time::Instant;

fn performance_comparison() {
    init();
    
    let iterations = 1_000_000;
    
    // 测试 track_var! 性能
    let start = Instant::now();
    for i in 0..iterations {
        let data = vec![i; 100];
        track_var!(data);
    }
    let track_var_time = start.elapsed();
    
    // 测试 track_var_smart! 性能
    let start = Instant::now();
    for i in 0..iterations {
        let data = track_var_smart!(vec![i; 100]);
    }
    let track_var_smart_time = start.elapsed();
    
    // 测试 track_var_owned! 性能
    let start = Instant::now();
    for i in 0..iterations {
        let data = vec![i; 100];
        let _tracked = track_var_owned!(data);
    }
    let track_var_owned_time = start.elapsed();
    
    println!("性能对比 ({} 次迭代):", iterations);
    println!("track_var!:       {:?}", track_var_time);
    println!("track_var_smart!: {:?}", track_var_smart_time);
    println!("track_var_owned!: {:?}", track_var_owned_time);
}
```

## 🛠️ 最佳实践

### 1. 选择合适的跟踪宏

```rust
// ✅ 生产环境 - 使用 track_var!
fn production_code() {
    let critical_data = load_important_data();
    track_var!(critical_data);  // 零开销
    process_data(&critical_data);
}

// ✅ 开发调试 - 使用 track_var_smart!
fn development_debugging() {
    let test_data = track_var_smart!(generate_test_data());
    run_tests(test_data);
}

// ✅ 生命周期分析 - 使用 track_var_owned!
fn lifecycle_analysis() {
    let data = create_data();
    let tracked = track_var_owned!(data);
    analyze_lifecycle(tracked);
}
```

### 2. 跟踪粒度控制

```rust
use memscope_rs::{track_var, init};

fn granularity_example() {
    init();
    
    // ✅ 跟踪关键数据结构
    let user_cache = create_user_cache();
    track_var!(user_cache);
    
    // ✅ 跟踪大内存分配
    let large_buffer = vec![0u8; 1024 * 1024];  // 1MB
    track_var!(large_buffer);
    
    // ❌ 避免跟踪临时小变量
    // let temp = 42;
    // track_var!(temp);  // 不必要
}
```

### 3. 条件跟踪

```rust
use memscope_rs::{track_var, init};

fn conditional_tracking() {
    init();
    
    let data = vec![1, 2, 3, 4, 5];
    
    // 仅在调试模式下跟踪
    #[cfg(debug_assertions)]
    track_var!(data);
    
    // 基于特性标志跟踪
    #[cfg(feature = "memory-profiling")]
    track_var!(data);
}
```

## 🔧 调试技巧

### 检查跟踪状态

```rust
use memscope_rs::{get_global_tracker, track_var, init};

fn debug_tracking() {
    init();
    
    let data1 = vec![1, 2, 3];
    track_var!(data1);
    
    let data2 = String::from("Hello");
    track_var!(data2);
    
    // 检查当前跟踪状态
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("活跃分配: {}", stats.active_allocations);
        println!("活跃内存: {} bytes", stats.active_memory);
        println!("总分配次数: {}", stats.total_allocations);
    }
}
```

### 跟踪特定作用域

```rust
use memscope_rs::{track_var, get_global_tracker, init};

fn scope_tracking() {
    init();
    
    let tracker = get_global_tracker();
    
    // 记录初始状态
    let initial_stats = tracker.get_stats().unwrap();
    
    {
        // 在特定作用域内跟踪
        let scoped_data = vec![1; 1000];
        track_var!(scoped_data);
        
        let current_stats = tracker.get_stats().unwrap();
        println!("作用域内新增内存: {} bytes", 
                current_stats.active_memory - initial_stats.active_memory);
    }
    
    // 检查作用域结束后的状态
    let final_stats = tracker.get_stats().unwrap();
    println!("作用域结束后内存变化: {} bytes", 
            final_stats.active_memory - initial_stats.active_memory);
}
```

## 🚀 下一步

现在你已经掌握了基础跟踪功能，可以继续学习：

- **[第一次内存分析](first-analysis.md)** - 生成和解读分析报告
- **[导出格式说明](../user-guide/export-formats.md)** - 选择合适的导出格式
- **[跟踪宏详解](../user-guide/tracking-macros.md)** - 深入了解宏的实现细节

## 💡 关键要点

- **`track_var!` 是首选** - 零开销，适合生产环境
- **智能指针自动跟踪引用计数** - Rc/Arc 变化会被记录
- **避免过度跟踪** - 只跟踪关键数据结构
- **使用条件编译** - 在不同环境下启用不同级别的跟踪
- **定期检查统计信息** - 了解内存使用趋势

开始使用这些技巧来优化你的内存使用吧！ 🎯