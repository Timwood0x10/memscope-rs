# memscope-rs API 使用指南

## 快速开始

memscope-rs 提供了简洁的 API 来追踪 Rust 应用程序的内存使用情况。

### 基本用法

```rust
use memscope_rs::{global_tracker, init_global_tracking, track};

// 初始化（可选，会在首次使用时自动初始化）
init_global_tracking().unwrap();

// 获取全局追踪器
let tracker = global_tracker().unwrap();

// 追踪变量
let data = vec![1, 2, 3, 4, 5];
track!(tracker, data);

let text = String::from("Hello, world!");
track!(tracker, text);

// 导出数据
tracker.export_json("output").unwrap();
tracker.export_html("output").unwrap();
```

## 核心 API

### 1. 初始化

```rust
use memscope_rs::init_global_tracking;

// 使用默认配置初始化
init_global_tracking().unwrap();

// 使用自定义配置初始化
use memscope_rs::GlobalTrackerConfig;
let config = GlobalTrackerConfig {
    tracker: memscope_rs::TrackerConfig {
        max_allocations: 1000000,
        enable_statistics: true,
    },
    ..Default::default()
};
init_global_tracking_with_config(config).unwrap();
```

### 2. 获取追踪器

```rust
use memscope_rs::global_tracker;

// 获取全局追踪器实例
let tracker = global_tracker().unwrap();
```

### 3. 追踪变量

#### 简单追踪

```rust
use memscope_rs::track;

// 使用 track! 宏自动追踪
let data = vec![1, 2, 3];
track!(tracker, data);

let text = String::from("Hello");
track!(tracker, text);
```

#### 带名称的追踪

```rust
use memscope_rs::track;

// 追踪时指定变量名和位置
let important_data = vec![1, 2, 3];
track!(tracker, important_data, "important_data", "my_file.rs", 42);
```

#### 直接追踪方法

```rust
// 使用 tracker.track() 方法
let data = vec![1, 2, 3];
tracker.track(&data);

// 使用 tracker.track_as() 方法指定名称
let data = vec![1, 2, 3];
tracker.track_as(&data, "my_data", "my_file.rs", 42);
```

### 4. 导出数据

#### 导出 JSON

```rust
// 导出所有 JSON 文件到指定目录
tracker.export_json("output")?;

// 这会生成以下文件：
// - memory_snapshots.json        - 内存快照数据
// - memory_passports.json        - 内存护照追踪
// - leak_detection.json          - 内存泄漏检测结果
// - unsafe_ffi_analysis.json    - 不安全/FFI 追踪数据
// - system_resources.json       - 系统资源监控
// - async_analysis.json         - 异步任务内存分析
```

#### 导出 HTML

```rust
// 导出 HTML 仪表板
tracker.export_html("output")?;

// 这会生成一个交互式的 HTML 仪表板文件
```

### 5. 获取统计信息

```rust
// 获取追踪统计信息
let stats = tracker.get_stats();

println!("总分配次数: {}", stats.total_allocations);
println!("活跃分配: {}", stats.active_allocations);
println!("峰值内存: {} bytes", stats.peak_memory_bytes);
println!("当前内存: {} bytes", stats.current_memory_bytes);
println!("内存护照数量: {}", stats.passport_count);
println!("活跃护照: {}", stats.active_passports);
println!("检测到的泄漏: {}", stats.leaks_detected);
println!("异步任务数量: {}", stats.async_task_count);
println!("活跃异步任务: {}", stats.active_async_tasks);
println!("运行时间: {:?}", stats.uptime);
```

## 高级用法

### 不安全代码追踪

```rust
use memscope_rs::global_tracker;

let tracker = global_tracker().unwrap();

// 追踪不安全分配
use std::alloc::{alloc, Layout};
unsafe {
    let layout = Layout::new::<[u32; 100]>();
    let ptr = alloc(layout);

    if !ptr.is_null() {
        // 创建内存护照（返回护照 ID）
        let passport_id = tracker
            .create_passport(ptr as usize, layout.size(), "unsafe_alloc".to_string())
            .expect("Failed to create passport");
        println!("Created passport: {}", passport_id);

        // 记录跨边界事件
        tracker.record_handover(ptr as usize, "ffi_context".to_string(), "malloc".to_string());

        // 使用内存...
        std::ptr::write_bytes(ptr as *mut u8, 0, layout.size());

        // 记录释放事件
        tracker.record_free(ptr as usize, "ffi_context".to_string(), "free".to_string());

        std::alloc::dealloc(ptr, layout);
    }
}
```

### 泄漏检测

```rust
use memscope_rs::global_tracker;

let tracker = global_tracker().unwrap();

// 执行泄漏检测（返回 LeakDetectionResult）
let leak_result = tracker.passport_tracker().detect_leaks_at_shutdown();

println!("检测到的泄漏数量: {}", leak_result.total_leaks);
println!("活跃护照数量: {}", leak_result.active_passports);
println!("总护照数量: {}", leak_result.total_passports);
```

### 访问内部追踪器

如果需要访问更底层的追踪功能：

```rust
use memscope_rs::global_tracker;

let tracker = global_tracker().unwrap();

// 访问基础追踪器
let base_tracker = tracker.tracker();
let report = base_tracker.analyze();

// 访问护照追踪器
let passport_tracker = tracker.passport_tracker();
let passport_stats = passport_tracker.get_stats();

// 访问异步追踪器
let async_tracker = tracker.async_tracker();
let async_stats = async_tracker.get_stats();
```

## 支持的类型

以下类型自动支持追踪：

- `Vec<T>`
- `String`
- `Box<T>`
- `Rc<T>`
- `Arc<T>`
- `HashMap<K, V>`
- `BTreeMap<K, V>`
- `VecDeque<T>`
- `RefCell<T>`
- `RwLock<T>`

## 完整示例

```rust
use memscope_rs::{global_tracker, init_global_tracking, track};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化
    init_global_tracking()?;

    // 获取追踪器
    let tracker = global_tracker()?;

    // 追踪各种类型
    let vec_data = vec![1, 2, 3, 4, 5];
    track!(tracker, vec_data);

    let string_data = String::from("Hello, memscope!");
    track!(tracker, string_data);

    let boxed_data = Box::new(42);
    track!(tracker, boxed_data);

    // 获取统计信息
    let stats = tracker.get_stats();
    println!("追踪统计: {:?}", stats);

    // 导出数据
    tracker.export_json("output")?;
    tracker.export_html("output")?;

    println!("数据已导出到 output/ 目录");

    Ok(())
}
```

## 注意事项

1. **线程安全**：全局追踪器是线程安全的，可以在多线程环境中使用
2. **性能影响**：追踪会带来一定的性能开销，建议仅在开发和调试时使用
3. **内存开销**：追踪本身会消耗内存，注意监控内存使用情况
4. **初始化**：虽然可以自动初始化，但建议在程序开始时显式调用 `init_global_tracking()`

## 故障排除

### 错误：AlreadyInitialized

如果你多次调用 `init_global_tracking()`，会收到 `AlreadyInitialized` 错误。这是正常的，因为全局追踪器只能初始化一次。

### 错误：NotInitialized

如果在未初始化的情况下调用 `global_tracker()`，会收到 `NotInitialized` 错误。确保在调用 `global_tracker()` 之前先调用 `init_global_tracking()`。

### 导出失败

确保目标目录存在且有写入权限。如果目录不存在，需要先创建：
```rust
use std::fs;
fs::create_dir_all("output")?;
```

## 更多示例

查看 `examples/` 目录获取更多使用示例：

- `basic_usage.rs` - 基本用法
- `global_tracker_showcase.rs` - 全局追踪器展示
- `complex_lifecycle_showcase.rs` - 复杂生命周期示例
- `complex_multithread_showcase.rs` - 多线程示例
- `comprehensive_async_showcase.rs` - 异步编程示例
- `unsafe_ffi_demo.rs` - 不安全代码和 FFI 示例
- `merkle_tree.rs` - Merkle 树实现示例

## 关系推断系统

memscope-rs 提供了强大的内存关系推断引擎，能够自动检测分配之间的语义关系。

### 支持的关系类型

| 关系类型 | 描述 | 检测方法 |
|----------|------|----------|
| **Owner** | A 拥有或指向 B | 指针扫描 - 在 A 的内存中找到指向 B 的指针 |
| **Slice** | A 是 B 的子视图 | 地址范围检测 - A 的指针在 B 的内部 |
| **Clone** | A 是 B 的克隆 | 内容相似度 + 时间窗口 + 调用栈匹配 |
| **Shared** | A 和 B 共享所有权 | Arc/Rc control block 模式识别 |

### 使用方法

```rust
use memscope_rs::analysis::relation_inference::{RelationGraphBuilder, Relation};

// 从活跃分配构建关系图
let graph = RelationGraphBuilder::build(&allocations, None);

// 查询关系
for edge in graph.edges() {
    println!("{:?}: {} -> {}", edge.relation, edge.from, edge.to);
}

// 检测循环引用
let cycles = graph.detect_cycles();
if !cycles.is_empty() {
    println!("检测到 {} 个循环引用", cycles.len());
}

// 获取所有节点
let nodes = graph.all_nodes();
```

### 准确率数据

基于真实测试数据的准确率：

```
=== Clone 检测准确率 ===
Precision: 100.00%
Recall: 100.00%
F1 Score: 100.00%

=== Owner 检测准确率 ===
✅ Box<Vec> 关系正确检测
✅ 独立 Vec 无误报

=== 性能数据 ===
1000 分配构建时间: ~230ms
```

### 配置选项

```rust
use memscope_rs::analysis::relation_inference::{GraphBuilderConfig, CloneConfig};

let config = GraphBuilderConfig {
    clone_config: CloneConfig {
        min_similarity: 0.8,              // 最小相似度阈值
        min_similarity_no_stack_hash: 0.95, // 无调用栈时的更严格阈值
        max_time_diff_ns: 10_000_000,     // 10ms 时间窗口
        max_clone_edges_per_node: 10,     // 每节点最大克隆边数
        ..Default::default()
    },
};

let graph = RelationGraphBuilder::build(&allocations, Some(config));
```

### 注意事项

1. **Owner 检测**：需要元数据在堆上（如 `Box<Vec>`），栈上的元数据无法被扫描
2. **Clone 检测**：依赖调用栈哈希，相同调用栈的分配会被分组比较
3. **Shared 检测**：依赖 Owner 关系，需要先检测 Owner 后再检测 Shared
4. **性能**：使用滑动时间窗口避免 O(n²) 复杂度

## 异步任务追踪

memscope-rs 提供了完整的异步任务内存追踪功能，可以追踪 tokio 任务的内存使用情况，并检测僵尸任务。

### 核心概念

| 概念 | 说明 |
|------|------|
| **Task ID** | 全局唯一任务 ID，永不回收，确保准确追踪 |
| **Tokio Task ID** | Tokio 运行时的任务 ID，可能被回收，仅作为 metadata |
| **Zombie Task** | 已启动但从未完成的任务，可能表示内存泄漏 |

### API 选择指南

```
┌─────────────────────────────────────────────────────────────────────┐
│                        何时使用哪个 API                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  场景 1: 在 async 代码中追踪                                         │
│  ─────────────────────────────────────                              │
│  ✅ 使用 track_in_tokio_task()                                       │
│     自动检测 tokio task ID，无需手动设置                              │
│                                                                     │
│  let (task_id, result) = tracker.track_in_tokio_task(               │
│      "my_task".to_string(),                                          │
│      async move {                                                    │
│          // 你的 async 代码                                          │
│      }                                                               │
│  ).await;                                                           │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  场景 2: 在同步代码中手动设置 task context                            │
│  ─────────────────────────────────────                              │
│  ✅ 使用 enter_task() 或 with_task()                                  │
│     手动设置 task ID，通常用于测试或特殊场景                           │
│                                                                     │
│  let task_id = 42u64;                                               │
│  let guard = AsyncTracker::enter_task(task_id);                       │
│  // 或                                                              │
│  AsyncTracker::with_task(task_id, || {                               │
│      // 同步代码                                                     │
│  });                                                                │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  场景 3: 在 tokio 上下文中自动工作                                    │
│  ─────────────────────────────────────                              │
│  ✅ track_as() 在 tokio 上下文中自动获取 task ID                      │
│     但这需要用户在 async 入口处调用 enter_task() 设置 context          │
│                                                                     │
│  // 在 async handler 入口                                            │
│  let task_id = generate_unique_task_id();                            │
│  let _guard = AsyncTracker::enter_task(task_id);                     │
│                                                                     │
│  // 之后所有 track_as() 调用会自动关联到这个 task                      │
│  tracker.track_as(&data, "my_data", file, line);                    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 使用示例

#### 示例 1: track_in_tokio_task (推荐)

```rust
use memscope_rs::{global_tracker, init_global_tracking};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // 使用 track_in_tokio_task 自动追踪
    let (task_id, result) = tracker
        .async_tracker()
        .track_in_tokio_task("http_handler".to_string(), async {
            // 模拟处理请求
            let data = vec![1u8; 1024];
            tracker.track_as(&data, "request_data", file!(), line!());

            // 模拟一些延迟
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            "response"
        })
        .await;

    println!("Task ID: {}, Result: {}", task_id, result);

    // 导出数据
    tracker.export_json("output")?;

    Ok(())
}
```

#### 示例 2: 手动设置 task context

```rust
use memscope_rs::{global_tracker, init_global_tracking, capture::backends::async_tracker::AsyncTracker};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // 生成唯一 task ID
    let task_id = generate_unique_task_id();

    // 设置 task context
    let _guard = AsyncTracker::enter_task(task_id);

    // 在这个 task context 中，所有 track_as() 调用都会关联到这个 task
    let data = vec![1u8; 1024];
    tracker.track_as(&data, "my_data", file!(), line!());

    // 清理
    drop(_guard);

    Ok(())
}
```

#### 示例 3: 检测僵尸任务

```rust
use memscope_rs::{global_tracker, init_global_tracking};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // 启动一些任务...
    // ...

    // 在程序结束时检测僵尸任务
    let zombies = tracker.async_tracker().detect_zombie_tasks();

    if !zombies.is_empty() {
        println!("警告: 发现 {} 个僵尸任务!", zombies.len());
        for task_id in &zombies {
            println!("  - Task ID: {}", task_id);
        }
    }

    // 获取僵尸任务统计
    let (zombie_count, total_count) = tracker.async_tracker().zombie_task_stats();
    println!(
        "僵尸任务: {}/{} ({:.1}%)",
        zombie_count,
        total_count,
        if total_count > 0 {
            (zombie_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        }
    );

    Ok(())
}
```

### 数据流

```
track_in_tokio_task() 或 track_as()
            │
            ▼
AsyncTracker::get_current_task()
            │
            ▼
track_allocation_with_location()
            │
            ▼
┌───────────────────────────────────────┐
│         AsyncTracker 内部状态          │
│  - allocations HashMap                │
│  - profiles HashMap (TaskMemoryProfile)│
│  - stats                              │
└───────────────────────────────────────┘
            │
            ▼
export_all_json() → async_analysis.json
            │
            ▼
输出文件:
  - task_profiles: 每个任务的内存使用情况
  - summary: 统计摘要
```

### 任务 ID 设计说明

 Tokio 的 task ID 会在任务完成后被回收复用，这可能导致 ID 冲突。
 memscope-rs 使用双重 ID 系统解决这个问题：

```rust
struct TaskMemoryProfile {
    task_id: u64,        // 全局唯一，永不回收 (TASK_COUNTER)
    tokio_task_id: u64, // Tokio 运行时 ID，可能回收 (作为 metadata)
}
```

这样既保证了追踪的准确性（使用永不回收的 task_id），又保留了调试信息（tokio_task_id 可以关联到 tokio runtime）。

### 注意事项

1. **性能影响**：异步追踪会带来一定开销，建议仅在开发和调试时启用
2. **Task ID 唯一性**：使用 `generate_unique_task_id()` 生成的任务 ID 永不重复
3. **生命周期**：确保 `track_task_end()` 在任务完成时被调用，以正确计算任务持续时间
4. **僵尸任务**：未完成的任务会被标记为僵尸任务，可能表示资源泄漏