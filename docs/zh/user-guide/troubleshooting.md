# 常见问题解决

本指南收集了使用 memscope-rs 时最常遇到的问题和解决方案。

## 🚨 编译错误

### 问题 1: "cannot find macro `track_var` in this scope"

**错误信息**:
```
error: cannot find macro `track_var` in this scope
 --> src/main.rs:5:5
  |
5 |     track_var!(my_vec);
  |     ^^^^^^^^^
```

**原因**: 没有正确导入宏

**解决方案**:
```rust
// ✅ 正确的导入方式
use memscope_rs::{track_var, init, get_global_tracker};

// 或者使用完整路径
memscope_rs::track_var!(my_vec);
```

### 问题 2: "no global memory allocator found"

**错误信息**:
```
error: no global memory allocator found but one is required; 
link to std or add `extern crate alloc` and `#[global_allocator]` as appropriate
```

**原因**: 在 `no_std` 环境中使用，或者分配器配置冲突

**解决方案**:
```rust
// 方案 1: 确保使用 std
// 在 Cargo.toml 中不要设置 default-features = false

// 方案 2: 如果必须使用 no_std，禁用 tracking-allocator 特性
[dependencies]
memscope-rs = { version = "0.1.4", default-features = false }
```

### 问题 3: "feature `tracking-allocator` is required"

**错误信息**:
```
error: the feature `tracking-allocator` is required to use the global allocator
```

**解决方案**:
```toml
# 在 Cargo.toml 中确保启用特性
[dependencies]
memscope-rs = { version = "0.1.4", features = ["tracking-allocator"] }

# 或者使用默认特性（已包含 tracking-allocator）
[dependencies]
memscope-rs = "0.1.4"
```

## 🏃‍♂️ 运行时错误

### 问题 4: "failed to initialize memory tracker"

**错误信息**:
```
thread 'main' panicked at 'failed to initialize memory tracker: AlreadyInitialized'
```

**原因**: 多次调用 `init()`

**解决方案**:
```rust
// ✅ 只在程序开始时调用一次
fn main() {
    memscope_rs::init(); // 只调用一次
    
    // 程序逻辑...
}

// ❌ 避免重复初始化
fn some_function() {
    // memscope_rs::init(); // 不要在这里调用
}
```

### 问题 5: "export directory creation failed"

**错误信息**:
```
Error: export directory creation failed: Permission denied (os error 13)
```

**原因**: 没有写入权限或目录被占用

**解决方案**:
```rust
// 方案 1: 检查当前目录权限
// 确保程序有写入当前目录的权限

// 方案 2: 指定自定义输出目录
use memscope_rs::ExportOptions;

let options = ExportOptions::new()
    .with_output_directory("/tmp/memscope_analysis") // 使用有权限的目录
    .with_create_subdirectory(true);

tracker.export_to_json_with_options("analysis", &options)?;

// 方案 3: 使用相对路径
let options = ExportOptions::new()
    .with_output_directory("./reports");
```

### 问题 6: "memory tracking not working"

**症状**: 调用 `get_stats()` 返回全零

**可能原因和解决方案**:

```rust
// 原因 1: 忘记调用 init()
fn main() {
    memscope_rs::init(); // ← 必须调用
    
    let data = vec![1, 2, 3];
    memscope_rs::track_var!(data);
}

// 原因 2: 没有启用 tracking-allocator 特性
// 检查 Cargo.toml:
[dependencies]
memscope-rs = { version = "0.1.4", features = ["tracking-allocator"] }

// 原因 3: 在 no_std 环境中
// 使用手动跟踪模式:
#[cfg(not(feature = "tracking-allocator"))]
fn manual_tracking_example() {
    use memscope_rs::MemoryTracker;
    
    let tracker = MemoryTracker::new();
    // 手动记录分配...
}
```

## 📊 性能问题

### 问题 7: "程序运行变慢"

**症状**: 启用 memscope-rs 后程序明显变慢

**诊断和解决**:

```rust
// 检查 1: 确认使用零开销宏
// ✅ 零开销
track_var!(data);

// ❌ 有开销
let tracked = track_var_owned!(data);

// 检查 2: 避免过度跟踪
// ✅ 只跟踪重要的分配
let important_data = vec![1; 1000000];
track_var!(important_data);

// ❌ 避免跟踪大量小对象
for i in 0..10000 {
    let small_data = vec![i]; // 不要每个都跟踪
    // track_var!(small_data); // 避免这样做
}

// 检查 3: 使用快速导出模式
use memscope_rs::ExportOptions;

let fast_options = ExportOptions::new()
    .with_fast_mode(true)
    .with_minimal_analysis(true);

tracker.export_to_json_with_options("fast_export", &fast_options)?;
```

### 问题 8: "内存使用过高"

**症状**: 程序内存使用异常增长

**解决方案**:

```rust
// 方案 1: 定期清理跟踪数据
let tracker = get_global_tracker();
tracker.clear_deallocated_entries(); // 清理已释放的条目

// 方案 2: 使用采样跟踪
static mut TRACK_COUNTER: usize = 0;

macro_rules! sample_track {
    ($var:expr) => {
        unsafe {
            TRACK_COUNTER += 1;
            if TRACK_COUNTER % 100 == 0 { // 只跟踪 1% 的分配
                track_var!($var);
            }
        }
    };
}

// 方案 3: 限制跟踪的数据大小
fn should_track<T>(data: &T) -> bool {
    std::mem::size_of_val(data) > 1024 // 只跟踪大于 1KB 的分配
}

let large_data = vec![0; 2048];
if should_track(&large_data) {
    track_var!(large_data);
}
```

## 🔧 导出问题

### 问题 9: "JSON 导出文件过大"

**解决方案**:

```rust
use memscope_rs::ExportOptions;

// 方案 1: 启用压缩
let options = ExportOptions::new()
    .with_compression(true)
    .with_minimal_analysis(true);

// 方案 2: 过滤小分配
let options = ExportOptions::new()
    .with_size_threshold(1024) // 只导出大于 1KB 的分配
    .with_exclude_system_allocations(true);

// 方案 3: 使用二进制格式
tracker.export_to_binary("compact_data.memscope")?;
```

### 问题 10: "HTML 报告无法打开"

**症状**: 生成的 HTML 文件在浏览器中显示空白

**解决方案**:

```rust
// 检查 1: 确保文件完整生成
use std::fs;

let html_path = "MemoryAnalysis/report.html";
if let Ok(metadata) = fs::metadata(html_path) {
    if metadata.len() == 0 {
        println!("HTML 文件为空，重新生成...");
        tracker.export_to_html("report.html")?;
    }
} else {
    println!("HTML 文件不存在");
}

// 检查 2: 使用绝对路径
let current_dir = std::env::current_dir()?;
let html_path = current_dir.join("MemoryAnalysis/report.html");
println!("HTML 文件位置: {}", html_path.display());

// 检查 3: 验证浏览器兼容性
// 使用现代浏览器（Chrome, Firefox, Safari, Edge）
```

## 🧵 多线程问题

### 问题 11: "多线程环境下数据不一致"

**解决方案**:

```rust
use std::sync::Arc;
use std::thread;

// ✅ 正确的多线程跟踪
fn multithreaded_tracking() {
    memscope_rs::init();
    
    let handles: Vec<_> = (0..4).map(|i| {
        thread::spawn(move || {
            let data = vec![i; 1000];
            track_var!(data); // 线程安全的跟踪
            
            // 处理数据...
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // 在主线程中导出
    let tracker = get_global_tracker();
    tracker.export_to_html("multithreaded_analysis.html").unwrap();
}
```

### 问题 12: "Arc/Rc 引用计数跟踪异常"

**解决方案**:

```rust
use std::sync::Arc;
use std::rc::Rc;

// ✅ 正确跟踪共享指针
fn shared_pointer_tracking() {
    // Arc - 多线程安全
    let arc_data = Arc::new(vec![1, 2, 3]);
    track_var!(arc_data);
    
    let arc_clone = Arc::clone(&arc_data);
    track_var!(arc_clone); // 自动跟踪引用计数变化
    
    // Rc - 单线程
    let rc_data = Rc::new(String::from("test"));
    track_var!(rc_data);
    
    let rc_clone = Rc::clone(&rc_data);
    track_var!(rc_clone); // 自动跟踪引用计数变化
}
```

## 🔍 调试技巧

### 启用详细日志
```rust
// 在程序开始时设置日志级别
std::env::set_var("RUST_LOG", "memscope_rs=debug");
env_logger::init();

// 或者使用 tracing
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

### 验证跟踪状态
```rust
fn debug_tracking_status() {
    let tracker = get_global_tracker();
    
    // 检查跟踪器状态
    if let Ok(stats) = tracker.get_stats() {
        println!("跟踪器状态:");
        println!("  活跃分配: {}", stats.active_allocations);
        println!("  总分配: {}", stats.total_allocations);
        println!("  峰值内存: {}", stats.peak_memory);
    } else {
        println!("⚠️ 跟踪器未正确初始化");
    }
    
    // 检查特性启用状态
    #[cfg(feature = "tracking-allocator")]
    println!("✅ tracking-allocator 特性已启用");
    
    #[cfg(not(feature = "tracking-allocator"))]
    println!("⚠️ tracking-allocator 特性未启用");
}
```

### 最小复现示例
```rust
// 创建最小的问题复现示例
fn minimal_reproduction() {
    println!("开始最小复现测试...");
    
    // 1. 初始化
    memscope_rs::init();
    println!("✅ 初始化完成");
    
    // 2. 简单跟踪
    let test_data = vec![1, 2, 3];
    memscope_rs::track_var!(test_data);
    println!("✅ 跟踪完成");
    
    // 3. 获取统计
    let tracker = memscope_rs::get_global_tracker();
    match tracker.get_stats() {
        Ok(stats) => {
            println!("✅ 统计获取成功: {} 个分配", stats.active_allocations);
        }
        Err(e) => {
            println!("❌ 统计获取失败: {}", e);
        }
    }
    
    // 4. 导出测试
    match tracker.export_to_json("test_export") {
        Ok(_) => println!("✅ 导出成功"),
        Err(e) => println!("❌ 导出失败: {}", e),
    }
}
```

## 📞 获取帮助

如果以上解决方案都不能解决你的问题：

1. **检查版本兼容性** - 确保使用最新版本
2. **查看示例代码** - 参考 `examples/` 目录中的工作示例
3. **提交 Issue** - 在 GitHub 上提供最小复现示例
4. **查看文档** - 阅读 [API 文档](https://docs.rs/memscope-rs)

记住：大多数问题都有简单的解决方案！ 🎯