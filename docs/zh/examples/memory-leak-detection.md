# 内存泄漏检测

本指南展示如何使用 memscope-rs 检测和分析 Rust 程序中的内存泄漏，包括循环引用、忘记释放的资源和长期存活的对象。

## 🎯 学习目标

- 识别不同类型的内存泄漏
- 使用 memscope-rs 检测循环引用
- 分析对象生命周期异常
- 理解 Rust 中内存泄漏的常见原因
- 生成内存泄漏分析报告

## 🚨 内存泄漏类型

| 泄漏类型 | 原因 | 检测方法 | 严重程度 |
|---------|------|----------|----------|
| **循环引用** | Rc/Arc 循环 | 引用计数分析 | 高 |
| **忘记释放** | 手动管理资源 | 生命周期跟踪 | 中 |
| **长期持有** | 全局/静态变量 | 存活时间分析 | 低 |
| **异步泄漏** | Future 未完成 | 异步状态跟踪 | 中 |

## 🚀 完整检测示例

```rust
use memscope_rs::{init, track_var, get_global_tracker};
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    println!("🔍 开始内存泄漏检测...");
    
    // 1. 循环引用检测
    detect_circular_references();
    
    // 2. 长期存活对象检测
    detect_long_lived_objects();
    
    // 3. 资源泄漏检测
    detect_resource_leaks();
    
    // 4. 异步内存泄漏检测
    detect_async_leaks();
    
    // 5. 导出分析结果
    let tracker = get_global_tracker();
    tracker.export_to_binary("memory_leak_detection")?;
    
    println!("✅ 内存泄漏检测完成！");
    println!("运行: make html DIR=MemoryAnalysis/memory_leak_detection BASE=memory_leak_detection");
    
    Ok(())
}
```

## 🔄 循环引用检测

### 经典循环引用示例

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Parent {
    name: String,
    children: RefCell<Vec<Rc<Child>>>,
}

#[derive(Debug)]
struct Child {
    name: String,
    parent: RefCell<Option<Rc<Parent>>>, // 这会造成循环引用！
}

fn detect_circular_references() {
    println!("🔄 检测循环引用...");
    
    // 创建父节点
    let parent = Rc::new(Parent {
        name: "Parent".to_string(),
        children: RefCell::new(vec![]),
    });
    track_var!(parent);
    
    // 创建子节点
    let child = Rc::new(Child {
        name: "Child".to_string(),
        parent: RefCell::new(None),
    });
    track_var!(child);
    
    // 建立循环引用
    parent.children.borrow_mut().push(Rc::clone(&child));
    *child.parent.borrow_mut() = Some(Rc::clone(&parent));
    
    println!("  父节点引用计数: {}", Rc::strong_count(&parent));
    println!("  子节点引用计数: {}", Rc::strong_count(&child));
    
    // 注意：这里会造成内存泄漏！
    println!("  ⚠️ 检测到循环引用 - 内存泄漏！");
}
```

### 正确的循环引用解决方案

```rust
#[derive(Debug)]
struct SafeParent {
    name: String,
    children: RefCell<Vec<Rc<SafeChild>>>,
}

#[derive(Debug)]
struct SafeChild {
    name: String,
    parent: RefCell<Option<Weak<SafeParent>>>, // 使用 Weak 打破循环
}

fn demonstrate_safe_references() {
    println!("✅ 演示安全的引用模式...");
    
    let parent = Rc::new(SafeParent {
        name: "SafeParent".to_string(),
        children: RefCell::new(vec![]),
    });
    track_var!(parent);
    
    let child = Rc::new(SafeChild {
        name: "SafeChild".to_string(),
        parent: RefCell::new(None),
    });
    track_var!(child);
    
    // 建立安全的父子关系
    parent.children.borrow_mut().push(Rc::clone(&child));
    *child.parent.borrow_mut() = Some(Rc::downgrade(&parent));
    
    println!("  父节点引用计数: {}", Rc::strong_count(&parent));
    println!("  子节点引用计数: {}", Rc::strong_count(&child));
    println!("  父节点弱引用计数: {}", Rc::weak_count(&parent));
    
    println!("  ✅ 无循环引用 - 内存安全！");
}
```

## ⏰ 长期存活对象检测

### 模拟长期存活的对象

```rust
use std::time::{Duration, Instant};
use std::thread;

static mut GLOBAL_CACHE: Option<HashMap<String, Vec<u8>>> = None;

fn detect_long_lived_objects() {
    println!("⏰ 检测长期存活对象...");
    
    // 1. 全局缓存（可能的内存泄漏源）
    unsafe {
        GLOBAL_CACHE = Some(HashMap::new());
        if let Some(ref mut cache) = GLOBAL_CACHE {
            // 添加大量数据到全局缓存
            for i in 0..1000 {
                let key = format!("key_{}", i);
                let value = vec![i as u8; 1024]; // 1KB per entry
                cache.insert(key, value);
            }
            track_var!(cache);
        }
    }
    
    // 2. 长期持有的大对象
    let long_lived_data = vec![0u8; 10 * 1024 * 1024]; // 10MB
    track_var!(long_lived_data);
    
    // 3. 模拟一些短期对象
    for i in 0..100 {
        let temp_data = vec![i; 100];
        track_var!(temp_data);
        // temp_data 在这里被释放
    }
    
    // 4. 中期存活对象
    let medium_lived = Arc::new(Mutex::new(vec![0; 1000]));
    track_var!(medium_lived);
    
    println!("  ✅ 长期存活对象检测完成");
    
    // 注意：long_lived_data 和 GLOBAL_CACHE 会一直存活到程序结束
}
```

### 生命周期分析

```rust
fn analyze_object_lifetimes() {
    let start_time = Instant::now();
    
    // 创建不同生命周期的对象
    let short_lived = {
        let data = vec![1; 1000];
        track_var!(data);
        data
    }; // data 在这里应该被释放，但我们返回了它
    
    thread::sleep(Duration::from_millis(100));
    
    let medium_lived = vec![2; 1000];
    track_var!(medium_lived);
    
    thread::sleep(Duration::from_millis(200));
    
    let long_lived = Box::leak(Box::new(vec![3; 1000])); // 故意泄漏！
    track_var!(long_lived);
    
    println!("  对象创建耗时: {:?}", start_time.elapsed());
    println!("  ⚠️ 检测到故意的内存泄漏");
}
```

## 💧 资源泄漏检测

### 文件句柄泄漏

```rust
use std::fs::File;
use std::io::Read;

fn detect_resource_leaks() {
    println!("💧 检测资源泄漏...");
    
    // 1. 文件句柄泄漏示例
    let mut leaked_files = Vec::new();
    for i in 0..10 {
        match File::open("Cargo.toml") {
            Ok(file) => {
                leaked_files.push(file); // 文件句柄被持有但可能不会被正确关闭
            }
            Err(_) => continue,
        }
    }
    track_var!(leaked_files);
    
    // 2. 内存分配泄漏
    let mut leaked_memory = Vec::new();
    for i in 0..100 {
        let data = Box::leak(Box::new(vec![i; 1000])); // 故意泄漏内存
        leaked_memory.push(data as *const Vec<i32>);
    }
    track_var!(leaked_memory);
    
    // 3. 线程句柄泄漏
    let mut thread_handles = Vec::new();
    for i in 0..5 {
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(10)); // 长时间运行
            println!("Thread {} finished", i);
        });
        thread_handles.push(handle);
    }
    track_var!(thread_handles);
    // 注意：如果不调用 join()，线程资源可能泄漏
    
    println!("  ⚠️ 检测到多种资源泄漏");
}
```

### 正确的资源管理

```rust
fn demonstrate_proper_resource_management() {
    println!("✅ 演示正确的资源管理...");
    
    // 1. 使用 RAII 自动管理文件
    {
        let _file = File::open("Cargo.toml").expect("Failed to open file");
        // 文件在作用域结束时自动关闭
    }
    
    // 2. 使用 Drop trait 自动清理
    struct ManagedResource {
        data: Vec<u8>,
    }
    
    impl Drop for ManagedResource {
        fn drop(&mut self) {
            println!("  清理资源: {} bytes", self.data.len());
        }
    }
    
    {
        let resource = ManagedResource {
            data: vec![0; 1000],
        };
        track_var!(resource);
        // resource 在这里自动调用 drop
    }
    
    // 3. 正确处理线程
    let handles: Vec<_> = (0..3).map(|i| {
        std::thread::spawn(move || {
            println!("Worker thread {} completed", i);
        })
    }).collect();
    
    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("  ✅ 所有资源正确清理");
}
```

## 🔮 异步内存泄漏检测

### 异步任务泄漏

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct LeakyFuture {
    data: Vec<u8>,
    completed: bool,
}

impl Future for LeakyFuture {
    type Output = ();
    
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completed {
            Poll::Ready(())
        } else {
            // 模拟永远不完成的 Future
            Poll::Pending
        }
    }
}

fn detect_async_leaks() {
    println!("🔮 检测异步内存泄漏...");
    
    // 1. 创建永远不完成的 Future
    let leaky_future = LeakyFuture {
        data: vec![0; 10000], // 10KB 数据
        completed: false,
    };
    track_var!(leaky_future);
    
    // 2. 创建大量异步任务但不等待完成
    let mut pending_futures = Vec::new();
    for i in 0..100 {
        let future = LeakyFuture {
            data: vec![i as u8; 1000],
            completed: false,
        };
        pending_futures.push(Box::pin(future));
    }
    track_var!(pending_futures);
    
    println!("  ⚠️ 检测到异步任务泄漏");
    
    // 注意：这些 Future 永远不会完成，导致内存泄漏
}
```

## 📊 泄漏分析报告

### 生成详细报告

```bash
# 运行检测
cargo run --example memory_leak_detection

# 生成 HTML 报告
make html DIR=MemoryAnalysis/memory_leak_detection BASE=memory_leak_detection

# 分析 JSON 数据
cat MemoryAnalysis/memory_leak_detection/memory_leak_detection_memory_analysis.json | jq '.allocations[] | select(.is_leaked == true)'
```

### 关键指标解读

1. **引用计数异常**
   ```json
   {
     "var_name": "circular_parent",
     "type_name": "alloc::rc::Rc<Parent>",
     "reference_count": 2,
     "expected_count": 1,
     "is_leaked": true
   }
   ```

2. **长期存活对象**
   ```json
   {
     "var_name": "long_lived_data",
     "lifetime_ms": 300000,
     "size": 10485760,
     "leak_probability": "high"
   }
   ```

3. **资源句柄泄漏**
   ```json
   {
     "resource_type": "file_handle",
     "count": 10,
     "status": "not_closed"
   }
   ```

## 🛠️ 泄漏检测工具

### 自动检测函数

```rust
use memscope_rs::analysis::detect_memory_leaks;

fn automated_leak_detection() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = get_global_tracker();
    let allocations = tracker.get_active_allocations()?;
    
    // 检测存活超过 5 秒的对象
    let potential_leaks = detect_memory_leaks(&allocations, 5000);
    
    if !potential_leaks.is_empty() {
        println!("🚨 发现 {} 个潜在内存泄漏:", potential_leaks.len());
        
        for leak in &potential_leaks {
            println!("  - {} bytes, 存活 {}ms", leak.size, leak.lifetime_ms);
            if let Some(name) = &leak.var_name {
                println!("    变量名: {}", name);
            }
        }
    } else {
        println!("✅ 未发现内存泄漏");
    }
    
    Ok(())
}
```

### 循环引用检测器

```rust
use memscope_rs::analysis::analyze_circular_references;

fn automated_circular_reference_detection() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = get_global_tracker();
    let allocations = tracker.get_active_allocations()?;
    
    let circular_refs = analyze_circular_references(&allocations)?;
    
    if !circular_refs.is_empty() {
        println!("🔄 发现 {} 个循环引用:", circular_refs.len());
        
        for circular_ref in &circular_refs {
            println!("  - 循环长度: {}", circular_ref.cycle_length);
            println!("    涉及分配: {:?}", circular_ref.involved_allocations);
            println!("    严重程度: {:?}", circular_ref.severity);
        }
    } else {
        println!("✅ 未发现循环引用");
    }
    
    Ok(())
}
```

## 🔧 预防和修复策略

### 1. 使用 Weak 引用

```rust
// ❌ 容易造成循环引用
struct BadNode {
    children: Vec<Rc<BadNode>>,
    parent: Option<Rc<BadNode>>,
}

// ✅ 使用 Weak 打破循环
struct GoodNode {
    children: Vec<Rc<GoodNode>>,
    parent: Option<Weak<GoodNode>>,
}
```

### 2. 实现 Drop trait

```rust
struct ResourceManager {
    resources: Vec<File>,
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        println!("清理 {} 个资源", self.resources.len());
        // 资源会自动清理
    }
}
```

### 3. 使用作用域管理

```rust
fn scoped_resource_management() {
    // 使用作用域限制对象生命周期
    {
        let temp_data = vec![0; 1000];
        track_var!(temp_data);
        // temp_data 在这里自动释放
    }
    
    // 使用 RAII 模式
    let _guard = std::fs::File::open("temp.txt");
    // 文件在 _guard 销毁时自动关闭
}
```

### 4. 定期清理

```rust
fn periodic_cleanup() {
    static mut CLEANUP_COUNTER: usize = 0;
    
    unsafe {
        CLEANUP_COUNTER += 1;
        if CLEANUP_COUNTER % 1000 == 0 {
            // 每 1000 次操作清理一次
            if let Some(ref mut cache) = GLOBAL_CACHE {
                cache.clear();
                println!("清理全局缓存");
            }
        }
    }
}
```

## 🎯 最佳实践

### 1. 设计原则

- **优先使用栈分配** - 避免不必要的堆分配
- **明确所有权** - 使用 Rust 的所有权系统
- **限制生命周期** - 使用作用域控制对象生命周期
- **避免全局状态** - 减少全局变量的使用

### 2. 检测策略

- **定期检测** - 在开发过程中定期运行泄漏检测
- **自动化测试** - 在 CI/CD 中集成内存泄漏检测
- **性能监控** - 监控生产环境的内存使用情况

### 3. 修复流程

1. **识别泄漏** - 使用 memscope-rs 识别泄漏位置
2. **分析原因** - 理解泄漏的根本原因
3. **设计修复** - 选择合适的修复策略
4. **验证修复** - 确认修复后不再泄漏

## 🎉 总结

通过这个内存泄漏检测示例，你学会了：

✅ **泄漏类型识别** - 循环引用、资源泄漏、长期存活对象  
✅ **自动检测工具** - 使用 memscope-rs 的分析功能  
✅ **预防策略** - Weak 引用、RAII、作用域管理  
✅ **修复技巧** - Drop trait、定期清理、所有权设计  
✅ **最佳实践** - 设计原则、检测策略、修复流程  

现在你可以有效地检测和修复 Rust 程序中的内存泄漏了！🚀