# 核心跟踪模块

memscope-rs 提供三个专门的跟踪模块，针对不同的并发场景设计，以及一个结合所有功能的混合模式。

## 🎯 模块概览

| 模块 | 使用场景 | 性能特点 | 精确度 | 最适合 |
|------|----------|----------|--------|---------|
| **单线程模块** | 基础跟踪 | 零开销 | 精确 | 开发、调试 |
| **多线程模块 (无锁)** | 高并发 | 采样式 | 近似 | 生产环境、20+线程 |
| **异步模块** | 任务中心 | < 5ns 开销 | 任务级别 | async/await 应用 |
| **混合模块** | 混合场景 | 自适应 | 综合 | 复杂应用 |

## 📦 1. 单线程模块 (默认)

### 核心特性
- **零开销跟踪** 使用 `track_var!` 宏
- **精确生命周期管理** 使用 `track_var_owned!`
- **智能类型检测** 使用 `track_var_smart!`
- **实时分析** 和交互式 HTML 报告

### API 使用
```rust
use memscope_rs::{track_var, track_var_smart, track_var_owned};

fn main() {
    memscope_rs::init();
    
    // 零开销跟踪 (推荐)
    let data = vec![1, 2, 3, 4, 5];
    track_var!(data);
    
    // 智能跟踪 (自动优化)
    let number = 42i32;        // Copy 类型 - 会被复制
    let text = String::new();  // 非 Copy - 引用跟踪
    track_var_smart!(number);
    track_var_smart!(text);
    
    // 所有权跟踪 (精确生命周期)
    let owned_data = vec![1, 2, 3];
    let tracked = track_var_owned!(owned_data);
    println!("数据: {:?}", tracked.get());
    
    // 导出分析
    let tracker = memscope_rs::get_tracker();
    tracker.export_to_json("analysis.json").unwrap();
}
```

### 示例：基础使用
```bash
cargo run --example basic_usage
```

**生成文件：**
- `MemoryAnalysis/basic_usage.json` - 原始跟踪数据
- `MemoryAnalysis/basic_usage.html` - 交互式仪表板

## 🔀 2. 多线程模块 (无锁)

### 核心特性
- **线程本地跟踪** 零共享状态
- **无锁设计** 支持高并发 (100+ 线程)
- **智能采样** 性能优化
- **二进制格式** 高效数据存储
- **综合平台指标** (CPU、GPU、I/O)

### API 使用
```rust
use memscope_rs::lockfree::{trace_all, stop_tracing, export_comprehensive_analysis};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 开始跟踪所有线程
    trace_all("./MemoryAnalysis")?;
    
    // 创建多个线程
    let handles: Vec<_> = (0..30).map(|i| {
        thread::spawn(move || {
            // 线程本地跟踪自动进行
            let data = vec![0u8; 1024 * 1024]; // 1MB 分配
            thread::sleep(std::time::Duration::from_millis(100));
            
            // 模拟工作
            for j in 0..1000 {
                let temp = vec![i, j];
                drop(temp);
            }
        })
    }).collect();
    
    // 等待线程完成
    for handle in handles {
        handle.join().unwrap();
    }
    
    // 停止跟踪并导出
    stop_tracing()?;
    export_comprehensive_analysis("./MemoryAnalysis", "comprehensive_analysis")?;
    
    Ok(())
}
```

### 示例：复杂多线程
```bash
cargo run --example complex_multithread_showcase
```

**生成文件：**
- `MemoryAnalysis/complex_showcase_dashboard.html` - 综合仪表板
- `MemoryAnalysis/*.bin` - 二进制跟踪数据 (高性能)

## ⚡ 3. 异步模块

### 核心特性
- **任务中心跟踪** 针对 async/await 应用
- **零开销任务识别** 使用 waker 地址
- **无锁事件缓冲** 质量监控
- **生产级可靠性** 数据完整性监控

### API 使用
```rust
use memscope_rs::async_memory::{initialize, spawn_tracked, get_memory_snapshot};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化异步跟踪
    initialize().await?;
    
    // 创建跟踪任务
    let task1 = spawn_tracked(async {
        let data = vec![0u8; 1024 * 1024]; // 1MB 分配
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        data.len()
    });
    
    let task2 = spawn_tracked(async {
        let mut results = Vec::new();
        for i in 0..1000 {
            results.push(format!("任务 {}", i));
            tokio::task::yield_now().await;
        }
        results.len()
    });
    
    // 执行任务
    let (result1, result2) = tokio::try_join!(task1, task2)?;
    println!("结果: {}, {}", result1, result2);
    
    // 获取内存快照
    let snapshot = get_memory_snapshot();
    println!("活跃任务: {}", snapshot.active_task_count());
    println!("总内存: {} 字节", snapshot.total_memory_usage());
    
    Ok(())
}
```

### 示例：综合异步
```bash
cargo run --example comprehensive_async_showcase
```

**生成文件：**
- `AsyncAnalysis/async_dashboard.html` - 任务中心分析
- `AsyncAnalysis/task_profiles.json` - 单个任务指标

## 🔄 4. 混合模块

### 核心特性
- **综合分析** 来自所有三个模块
- **统一仪表板** 跨模块洞察
- **自动优化** 基于工作负载模式
- **丰富可视化** 性能关联分析

### API 使用
```rust
use memscope_rs::export::fixed_hybrid_template::{
    FixedHybridTemplate, create_sample_hybrid_data, RenderMode
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建结合所有模块的混合分析
    let thread_count = 30;
    let task_count = 100;
    
    // 生成综合混合数据
    let hybrid_data = create_sample_hybrid_data(thread_count, task_count);
    
    // 创建 HTML 仪表板
    let template = FixedHybridTemplate::new(RenderMode::Complete);
    let html_content = template.generate_hybrid_dashboard(&hybrid_data)?;
    
    // 写入综合仪表板
    std::fs::write("hybrid_dashboard.html", html_content)?;
    
    println!("✅ 混合分析完成: hybrid_dashboard.html");
    
    Ok(())
}
```

### 示例：增强型 30 线程演示
```bash
cargo run --example enhanced_30_thread_demo
```

**生成文件：**
- `hybrid_dashboard.html` - 统一分析仪表板
- 结合线程、任务和单线程洞察

## 🎛️ 配置选项

### 单线程配置
```rust
// 测试模式快速运行
std::env::set_var("MEMSCOPE_TEST_MODE", "1");

// 程序退出时自动导出
memscope_rs::enable_auto_export(Some("final_analysis"));
```

### 多线程配置
```rust
use memscope_rs::lockfree::SamplingConfig;

// 自定义采样配置
let config = SamplingConfig {
    sample_rate: 0.1,        // 10% 采样率
    max_events: 1000000,     // 每线程 1M 事件
    buffer_size: 64 * 1024,  // 64KB 缓冲区
};
```

### 异步配置
```rust
use memscope_rs::async_memory::VisualizationConfig;

let config = VisualizationConfig {
    max_tracked_tasks: 10000,
    buffer_size: 1024 * 1024,  // 每线程 1MB
    enable_task_hierarchy: true,
};
```

## 📊 性能特征

### 导出性能 (实际测试数据)

| 模块 | 导出时间 | 文件大小 | 使用场景 |
|------|----------|----------|----------|
| 单线程 | 1.3s | 1.2MB | 开发分析 |
| 多线程 | 211ms | 480KB | 生产监控 |
| 异步 | 800ms | 800KB | 任务性能分析 |
| 混合 | 2.1s | 2.5MB | 综合分析 |

*基于示例应用的实际测试结果*

### 内存开销

| 模块 | 每线程开销 | 跟踪开销 | 运行时影响 |
|------|------------|----------|------------|
| 单线程 | ~100KB | 零 (基于引用) | < 0.1% |
| 多线程 | ~64KB | 基于采样 | < 0.5% |
| 异步 | ~1MB | < 5ns 每次分配 | < 0.1% |
| 混合 | 可变 | 自适应 | < 1% |

## 🔧 选择合适的模块

### 使用单线程模块当：
- ✅ 开发和调试
- ✅ 单线程应用
- ✅ 需要精确精度
- ✅ 需要实时分析

### 使用多线程模块当：
- ✅ 高并发 (20+ 线程)
- ✅ 性能至关重要
- ✅ 生产监控
- ✅ 近似跟踪可接受

### 使用异步模块当：
- ✅ async/await 应用
- ✅ 需要任务级分析
- ✅ 复杂异步模式
- ✅ 需要任务层次洞察

### 使用混合模块当：
- ✅ 复杂应用混合模式
- ✅ 需要综合分析
- ✅ 比较不同方法
- ✅ 高级性能优化

## 🚀 快速开始命令

```bash
# 尝试每个模块:
cargo run --example basic_usage                    # 单线程
cargo run --example complex_multithread_showcase   # 多线程  
cargo run --example comprehensive_async_showcase   # 异步
cargo run --example enhanced_30_thread_demo        # 混合

# 生成 HTML 报告:
make html DIR=MemoryAnalysis BASE=basic_usage
```

## 💡 实用建议

### 开发阶段
1. 使用**单线程模块**进行精确调试
2. 用 `track_var_smart!` 进行快速原型
3. 启用自动导出检查内存泄漏

### 测试阶段
1. 使用**多线程模块**测试并发性能
2. 配置适当的采样率
3. 监控线程间内存竞争

### 生产环境
1. 根据应用类型选择模块
2. 使用二进制格式减少开销
3. 定期导出分析数据

### 性能优化
1. 使用**混合模块**综合分析
2. 对比不同模块的结果
3. 根据瓶颈调整配置

---

**下一步：** [单线程模块详解](single-threaded.md) | [多线程模块详解](multithread.md) | [异步模块详解](async.md)