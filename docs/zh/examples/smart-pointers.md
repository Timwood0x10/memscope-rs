# 智能指针内存分析

本指南展示如何使用 memscope-rs 分析 Rust 智能指针的内存使用模式，包括 `Box`、`Rc`、`Arc`、`RefCell` 等的跟踪和分析。

## 🎯 学习目标

- 跟踪不同类型智能指针的内存分配
- 分析引用计数的变化模式
- 检测循环引用和内存泄漏
- 理解智能指针的性能影响
- 生成智能指针使用的分析报告

## 📦 智能指针类型概览

| 智能指针 | 用途 | 线程安全 | 引用计数 |
|---------|------|----------|----------|
| `Box<T>` | 堆分配 | ❌ | ❌ |
| `Rc<T>` | 共享所有权 | ❌ | ✅ |
| `Arc<T>` | 线程安全共享 | ✅ | ✅ |
| `RefCell<T>` | 内部可变性 | ❌ | ❌ |
| `Mutex<T>` | 线程安全可变 | ✅ | ❌ |

## 🚀 完整示例

### 基础智能指针跟踪

```rust
use memscope_rs::{init, track_var, get_global_tracker};
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    // 1. Box 指针分析
    analyze_box_pointers();
    
    // 2. Rc 引用计数分析
    analyze_rc_pointers();
    
    // 3. Arc 线程安全分析
    analyze_arc_pointers();
    
    // 4. RefCell 内部可变性分析
    analyze_refcell_patterns();
    
    // 5. 导出分析结果
    let tracker = get_global_tracker();
    tracker.export_to_binary("smart_pointer_analysis")?;
    
    println!("✅ 智能指针分析完成！");
    println!("运行: make html DIR=MemoryAnalysis/smart_pointer_analysis BASE=smart_pointer_analysis");
    
    Ok(())
}
```

## 📦 Box 指针分析

### 基础 Box 使用

```rust
fn analyze_box_pointers() {
    println!("📦 分析 Box 指针...");
    
    // 创建不同大小的 Box
    let small_box = Box::new(42i32);
    track_var!(small_box);
    
    let large_box = Box::new(vec![0; 10000]);
    track_var!(large_box);
    
    let string_box = Box::new(String::from("Hello, Box!"));
    track_var!(string_box);
    
    // 嵌套 Box
    let nested_box = Box::new(Box::new(Box::new(100)));
    track_var!(nested_box);
    
    println!("  ✅ Box 分析完成");
}
```

### Box 性能模式

```rust
fn analyze_box_performance() {
    // 大量小 Box（可能低效）
    let mut small_boxes = Vec::new();
    for i in 0..1000 {
        let boxed = Box::new(i);
        small_boxes.push(boxed);
    }
    track_var!(small_boxes);
    
    // 单个大 Box（更高效）
    let large_data = vec![0; 1000];
    let large_box = Box::new(large_data);
    track_var!(large_box);
}
```

## 🔄 Rc 引用计数分析

### 基础 Rc 使用

```rust
fn analyze_rc_pointers() {
    println!("🔄 分析 Rc 引用计数...");
    
    // 创建原始 Rc
    let original = Rc::new(vec![1, 2, 3, 4, 5]);
    track_var!(original);
    println!("  引用计数: {}", Rc::strong_count(&original));
    
    // 创建克隆
    let clone1 = Rc::clone(&original);
    track_var!(clone1);
    println!("  引用计数: {}", Rc::strong_count(&original));
    
    let clone2 = Rc::clone(&original);
    track_var!(clone2);
    println!("  引用计数: {}", Rc::strong_count(&original));
    
    // 弱引用
    let weak_ref = Rc::downgrade(&original);
    track_var!(weak_ref);
    println!("  强引用: {}, 弱引用: {}", 
             Rc::strong_count(&original), 
             Rc::weak_count(&original));
    
    println!("  ✅ Rc 分析完成");
}
```

### Rc 循环引用检测

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Node {
    value: i32,
    children: RefCell<Vec<Rc<Node>>>,
    parent: RefCell<Weak<Node>>,
}

fn analyze_circular_references() {
    println!("🔄 检测循环引用...");
    
    let parent = Rc::new(Node {
        value: 1,
        children: RefCell::new(vec![]),
        parent: RefCell::new(Weak::new()),
    });
    track_var!(parent);
    
    let child = Rc::new(Node {
        value: 2,
        children: RefCell::new(vec![]),
        parent: RefCell::new(Rc::downgrade(&parent)),
    });
    track_var!(child);
    
    // 建立父子关系
    parent.children.borrow_mut().push(Rc::clone(&child));
    
    println!("  父节点引用计数: {}", Rc::strong_count(&parent));
    println!("  子节点引用计数: {}", Rc::strong_count(&child));
    
    // 注意：这里没有循环引用，因为使用了 Weak
    println!("  ✅ 无循环引用检测完成");
}
```

## 🧵 Arc 线程安全分析

### 多线程 Arc 使用

```rust
use std::sync::Arc;
use std::thread;

fn analyze_arc_pointers() {
    println!("🧵 分析 Arc 线程安全指针...");
    
    let shared_data = Arc::new(vec![1, 2, 3, 4, 5]);
    track_var!(shared_data);
    
    let mut handles = vec![];
    
    // 在多个线程中共享数据
    for thread_id in 0..4 {
        let data_clone = Arc::clone(&shared_data);
        
        let handle = thread::spawn(move || {
            // 在每个线程中跟踪克隆
            track_var!(data_clone);
            
            println!("  线程 {} 访问数据: {:?}", thread_id, data_clone);
            
            // 模拟一些工作
            thread::sleep(std::time::Duration::from_millis(100));
        });
        
        handles.push(handle);
    }
    
    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("  ✅ Arc 分析完成");
}
```

### Arc + Mutex 模式

```rust
use std::sync::{Arc, Mutex};

fn analyze_arc_mutex_pattern() {
    let shared_counter = Arc::new(Mutex::new(0));
    track_var!(shared_counter);
    
    let mut handles = vec![];
    
    for _ in 0..4 {
        let counter_clone = Arc::clone(&shared_counter);
        
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                let mut num = counter_clone.lock().unwrap();
                *num += 1;
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let final_count = *shared_counter.lock().unwrap();
    println!("  最终计数: {}", final_count);
}
```

## 🔄 RefCell 内部可变性分析

### 基础 RefCell 使用

```rust
use std::cell::RefCell;

fn analyze_refcell_patterns() {
    println!("🔄 分析 RefCell 内部可变性...");
    
    let data = RefCell::new(vec![1, 2, 3]);
    track_var!(data);
    
    // 不可变借用
    {
        let borrowed = data.borrow();
        println!("  数据长度: {}", borrowed.len());
        track_var!(borrowed);
    }
    
    // 可变借用
    {
        let mut borrowed_mut = data.borrow_mut();
        borrowed_mut.push(4);
        track_var!(borrowed_mut);
    }
    
    println!("  ✅ RefCell 分析完成");
}
```

### Rc + RefCell 组合模式

```rust
fn analyze_rc_refcell_combination() {
    let shared_data = Rc::new(RefCell::new(vec![1, 2, 3]));
    track_var!(shared_data);
    
    let clone1 = Rc::clone(&shared_data);
    let clone2 = Rc::clone(&shared_data);
    
    // 通过不同的克隆修改数据
    clone1.borrow_mut().push(4);
    clone2.borrow_mut().push(5);
    
    println!("  最终数据: {:?}", shared_data.borrow());
    println!("  引用计数: {}", Rc::strong_count(&shared_data));
}
```

## 📊 性能分析和优化

### 智能指针性能对比

```rust
use std::time::Instant;

fn benchmark_smart_pointers() {
    let iterations = 100000;
    
    // Box 性能测试
    let start = Instant::now();
    for i in 0..iterations {
        let boxed = Box::new(i);
        std::hint::black_box(boxed);
    }
    let box_time = start.elapsed();
    
    // Rc 性能测试
    let start = Instant::now();
    let rc_data = Rc::new(0);
    for _ in 0..iterations {
        let cloned = Rc::clone(&rc_data);
        std::hint::black_box(cloned);
    }
    let rc_time = start.elapsed();
    
    println!("📊 性能对比:");
    println!("  Box 创建: {:?}", box_time);
    println!("  Rc 克隆: {:?}", rc_time);
}
```

### 内存使用模式分析

```rust
fn analyze_memory_patterns() {
    // 模式 1: 深度嵌套
    let deep_nested = Box::new(Box::new(Box::new(Box::new(42))));
    track_var!(deep_nested);
    
    // 模式 2: 广度共享
    let shared = Rc::new(vec![1; 1000]);
    let mut clones = Vec::new();
    for _ in 0..10 {
        clones.push(Rc::clone(&shared));
    }
    track_var!(clones);
    
    // 模式 3: 混合使用
    let mixed = Arc::new(Mutex::new(RefCell::new(Box::new(vec![1, 2, 3]))));
    track_var!(mixed);
}
```

## 🔍 分析报告解读

### 生成详细报告

```bash
# 导出所有格式
cargo run --example smart_pointer_analysis
memscope analyze --export all ./target/debug/examples/smart_pointer_analysis

# 生成 HTML 报告
make html DIR=MemoryAnalysis/smart_pointer_analysis BASE=smart_pointer_analysis

# 查看 JSON 数据
cat MemoryAnalysis/smart_pointer_analysis/smart_pointer_analysis_memory_analysis.json | jq .
```

### 关键指标解读

1. **引用计数变化**
   ```json
   {
     "var_name": "shared_rc",
     "type_name": "alloc::rc::Rc<alloc::vec::Vec<i32>>",
     "reference_count": 3,
     "weak_count": 1
   }
   ```

2. **内存分布**
   - Box: 直接堆分配
   - Rc: 引用计数 + 数据
   - Arc: 原子引用计数 + 数据

3. **生命周期模式**
   - 短期：临时 Box
   - 中期：共享 Rc
   - 长期：全局 Arc

## 🛠️ 最佳实践

### 1. 选择合适的智能指针

```rust
// 单一所有权 -> Box
let unique_data = Box::new(expensive_computation());

// 单线程共享 -> Rc
let shared_config = Rc::new(load_configuration());

// 多线程共享 -> Arc
let thread_safe_data = Arc::new(Mutex::new(shared_state));

// 内部可变性 -> RefCell
let mutable_in_immutable = RefCell::new(counter);
```

### 2. 避免常见陷阱

```rust
// ❌ 避免：不必要的 Box
let unnecessary = Box::new(42); // 直接用 i32 即可

// ✅ 推荐：只在需要时使用 Box
let necessary = Box::new(large_struct);

// ❌ 避免：循环引用
// parent -> child -> parent (使用 Rc)

// ✅ 推荐：使用 Weak 打破循环
// parent -> child, child -> Weak<parent>
```

### 3. 性能优化技巧

```rust
// 预分配容量
let mut data = Vec::with_capacity(1000);
let boxed_data = Box::new(data);

// 批量操作
let batch_data = (0..1000).collect::<Vec<_>>();
let shared_batch = Rc::new(batch_data);

// 减少克隆
let data = Rc::new(expensive_data);
// 传递引用而不是克隆
process_data(&data);
```

## 🔧 故障排除

### 常见问题

1. **引用计数不减少**
   ```rust
   // 检查是否有循环引用
   println!("强引用: {}", Rc::strong_count(&data));
   println!("弱引用: {}", Rc::weak_count(&data));
   ```

2. **内存使用过高**
   ```rust
   // 检查是否有内存泄漏
   let tracker = get_global_tracker();
   let stats = tracker.get_stats()?;
   println!("活跃分配: {}", stats.active_allocations);
   ```

3. **性能问题**
   ```rust
   // 使用 Arc 而不是 Mutex<Rc<T>>
   // ❌ 低效
   let bad = Mutex::new(Rc::new(data));
   
   // ✅ 高效
   let good = Arc::new(Mutex::new(data));
   ```

## 🎉 总结

通过这个智能指针分析示例，你学会了：

✅ **智能指针跟踪** - 跟踪 Box、Rc、Arc、RefCell 等  
✅ **引用计数分析** - 理解引用计数的变化模式  
✅ **循环引用检测** - 识别和避免内存泄漏  
✅ **性能优化** - 选择合适的智能指针类型  
✅ **最佳实践** - 避免常见陷阱和性能问题  

现在你可以有效地分析和优化 Rust 程序中的智能指针使用了！🚀