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