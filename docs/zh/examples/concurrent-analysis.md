# 并发代码内存分析

本指南展示如何使用 memscope-rs 分析多线程程序的内存使用模式，包括共享变量跟踪、竞争条件检测和性能分析。

## 🎯 学习目标

- 跟踪多线程程序中的共享变量
- 分析生产者-消费者模式的内存使用
- 检测工作窃取队列的负载均衡
- 理解原子操作的内存影响
- 生成并发程序的内存分析报告

## 🚀 完整示例

运行我们提供的高级示例：

```bash
# 运行高级多线程内存分析示例
cargo run --example advanced_metrics_demo

# 生成交互式HTML报告
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo

# 打开报告查看结果
open memory_report.html
```

## 📊 示例输出

运行示例后，你会看到类似的输出：

```
🚀 Advanced Memory Metrics Demo
===============================
📊 Creating advanced memory scenarios...
   Creating complex nested data structures...
   Creating smart pointer relationships...
   Creating unsafe code and FFI scenarios...
   Creating multi-threaded scenarios with shared variable tracking...
   ✅ Complex multi-threaded scenarios with shared variable tracking completed
   Creating memory layout optimization examples...
   Creating performance-critical allocations...
   Creating main-thread allocations with clear variable names...
✅ Created advanced allocation scenarios with rich metadata

💾 Exporting to binary format...
✅ Binary export completed in 211ms
📁 Binary file: MemoryAnalysis/advanced_metrics_demo/advanced_metrics_demo.memscope (480KB)

🔄 Converting binary to standard JSON files...
✅ Standard JSON conversion completed in 17.1s
📄 Generated JSON files:
  • advanced_metrics_demo_memory_analysis.json (84KB)
  • advanced_metrics_demo_lifetime.json (69KB)
  • advanced_metrics_demo_performance.json (125KB)
  • advanced_metrics_demo_unsafe_ffi.json (118KB)
  • advanced_metrics_demo_complex_types.json (330KB)

📈 Advanced Performance Analysis:
  📊 Binary export time:     211ms
  📊 Standard JSON time:     17.1s
  🚀 Speed improvement:      80.72x faster
  📁 Binary file size:       480KB
  📁 JSON files size:        728KB (5 files)
  💾 Size reduction:         34.0%

🔍 Advanced Memory Analysis:
  • Total allocations: 289
  • Smart pointer usage: 20
  • Unsafe operations: 0
  • Multi-threaded allocations: 294
  • Complex data structures: 78
```

## 🧵 多线程场景分析

### 1. 生产者-消费者模式

示例中实现了一个复杂的生产者-消费者场景：

- **3个生产者线程** - 向共享缓冲区添加数据
- **2个消费者线程** - 从共享缓冲区取出数据
- **共享统计** - 跟踪生产和消费计数

```rust
// 核心数据结构（来自示例）
let shared_buffer = Arc<Mutex<VecDeque<String>>>;
let buffer_stats = Arc<Mutex<(usize, usize)>>; // (produced, consumed)

// 每个线程的结果都被单独跟踪
let consumer_data = (vec![consumer_id], consumed_items, vec![stats]);
track_var!(consumer_data);
```

### 2. 读写锁缓存访问

模拟了一个高并发的缓存系统：

- **2个写线程** - 更新缓存数据
- **4个读线程** - 并发读取缓存
- **访问统计** - 跟踪读取、写入和缓存未命中

```rust
let shared_cache = Arc<RwLock<HashMap<String, Vec<u8>>>>;
let cache_metrics = Arc<Mutex<(usize, usize, usize)>>; // (reads, writes, misses)
```

### 3. 工作窃取队列

实现了一个工作窃取算法：

- **4个工作线程** - 每个都有自己的工作队列
- **任务窃取** - 空闲线程从其他线程窃取任务
- **负载统计** - 跟踪每个线程完成的工作量

```rust
let work_queues: Vec<Arc<Mutex<VecDeque<String>>>> = (0..4)
    .map(|_| Arc::new(Mutex::new(VecDeque::new())))
    .collect();
```

### 4. 原子操作和无锁结构

展示了原子操作的内存跟踪：

- **原子计数器** - 多线程安全的计数
- **原子标志位** - 线程间状态同步
- **操作历史** - 记录每个原子操作

```rust
let atomic_counter = Arc<AtomicUsize>;
let atomic_flags = Arc<[AtomicBool; 4]>;
```

## 📈 性能分析结果

### 导出性能对比

| 格式 | 导出时间 | 文件大小 | 速度提升 |
|------|---------|---------|----------|
| Binary | 211ms | 480KB | 基准 |
| JSON | 17.1s | 728KB | 80.72x 慢 |

### 内存使用统计

- **总分配数**: 289个
- **智能指针**: 20个（Arc, Rc等）
- **多线程分配**: 294个
- **复杂数据结构**: 78个

## 🔍 分析报告解读

### JSON文件内容

生成的5个JSON文件包含不同方面的数据：

1. **memory_analysis.json** - 基础分配信息
   ```json
   {
     "var_name": "main_thread_buffer",
     "type_name": "alloc::vec::Vec<u8>",
     "size": 1024,
     "thread_id": "ThreadId(1)"
   }
   ```

2. **performance.json** - 性能相关数据
3. **complex_types.json** - 复杂类型分析
4. **unsafe_ffi.json** - Unsafe代码跟踪
5. **lifetime.json** - 生命周期信息

### HTML报告功能

使用 `make html` 生成的交互式报告包含：

- **内存使用时间线** - 显示内存增长趋势
- **线程分析** - 按线程分组的内存使用
- **类型分布** - 不同数据类型的内存占用
- **变量关系图** - 智能指针的引用关系

## 🛠️ 自定义并发分析

### 创建你自己的多线程分析

```rust
use memscope_rs::{get_global_tracker, track_var, init};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    init();
    
    // 1. 创建共享数据结构
    let shared_data = Arc::new(Mutex::new(Vec::new()));
    track_var!(shared_data);
    
    // 2. 启动多个线程
    let mut handles = vec![];
    for thread_id in 0..4 {
        let data_clone = Arc::clone(&shared_data);
        
        let handle = thread::spawn(move || {
            // 3. 在每个线程中跟踪局部数据
            let local_data = vec![thread_id; 100];
            track_var!(local_data);
            
            // 4. 操作共享数据
            {
                let mut data = data_clone.lock().unwrap();
                data.extend_from_slice(&local_data);
            }
        });
        handles.push(handle);
    }
    
    // 5. 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }
    
    // 6. 导出分析结果
    let tracker = get_global_tracker();
    tracker.export_to_binary("my_concurrent_analysis")?;
    
    println!("✅ 并发分析完成！");
    println!("运行: make html DIR=MemoryAnalysis/my_concurrent_analysis BASE=my_concurrent_analysis");
}
```

### 分析特定的并发模式

```rust
// 分析 Channel 通信
use std::sync::mpsc;

let (sender, receiver) = mpsc::channel();
track_var!(sender);
track_var!(receiver);

// 分析 Barrier 同步
use std::sync::Barrier;

let barrier = Arc::new(Barrier::new(4));
track_var!(barrier);

// 分析 Condvar 等待
use std::sync::Condvar;

let condvar = Arc::new(Condvar::new());
track_var!(condvar);
```

## 🎯 最佳实践

### 1. 跟踪策略

- **主线程变量** - 确保有清晰的变量名
- **共享数据** - 在创建时就开始跟踪
- **线程局部数据** - 在每个线程内部跟踪

### 2. 性能考虑

- **使用Binary格式** - 对于大量数据，Binary比JSON快80倍
- **分批分析** - 避免一次跟踪过多变量
- **选择性跟踪** - 只跟踪关键的共享数据

### 3. 报告生成

```bash
# 快速查看 - 使用SVG
tracker.export_memory_analysis("quick_view.svg")?;

# 详细分析 - 使用HTML
make html DIR=MemoryAnalysis/your_analysis BASE=your_analysis

# 数据处理 - 使用JSON
tracker.export_to_json("data_analysis")?;
```

## 🔧 故障排除

### 常见问题

1. **变量名显示为"unknown"**
   - 确保在主线程中有明确命名的变量
   - 使用 `track_var!(variable_name)` 而不是匿名表达式

2. **HTML图表显示错误**
   - 确保使用正确的BASE名称：`make html BASE=your_actual_base_name`
   - 检查JSON文件是否正确生成

3. **性能问题**
   - 优先使用Binary格式导出
   - 避免跟踪过多的临时变量

### 调试技巧

```rust
// 启用详细日志
std::env::set_var("MEMSCOPE_VERBOSE", "1");

// 启用测试模式（更准确的跟踪）
std::env::set_var("MEMSCOPE_TEST_MODE", "1");

// 启用准确跟踪（用于测试）
std::env::set_var("MEMSCOPE_ACCURATE_TRACKING", "1");
```

## 🎉 总结

通过这个并发分析示例，你学会了：

✅ **多线程内存跟踪** - 跟踪共享变量和线程局部数据  
✅ **性能优化** - 使用Binary格式获得80倍速度提升  
✅ **复杂场景分析** - 生产者-消费者、工作窃取、原子操作  
✅ **交互式报告** - 生成专业的HTML分析报告  
✅ **数据分类** - 5个专门的JSON文件便于深度分析  

现在你可以分析任何复杂的多线程Rust程序的内存使用模式了！🚀