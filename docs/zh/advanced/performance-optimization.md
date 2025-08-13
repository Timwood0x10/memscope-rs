# 性能优化指南

memscope-rs 的高级性能优化技巧和策略。

## 🎯 优化目标

- 减少跟踪开销
- 提高导出速度
- 优化内存使用
- 改善分析性能

## ⚡ 核心优化策略

### 1. 选择性跟踪

```rust
// ❌ 跟踪所有变量（开销大）
for i in 0..10000 {
    let data = vec![i];
    track_var!(data);
}

// ✅ 选择性跟踪（开销小）
let important_data = vec![1; 10000];
track_var!(important_data);
```

### 2. 使用 Binary 导出

```rust
// ✅ Binary 格式 - 80倍速度提升
tracker.export_to_binary("analysis")?;

// ❌ JSON 格式 - 较慢
tracker.export_to_json("analysis")?;
```

### 3. 配置优化

```rust
use memscope_rs::TrackingConfig;

let config = TrackingConfig {
    enable_stack_traces: false,    // 关闭栈跟踪
    sampling_rate: 0.1,            // 10% 采样
    memory_threshold: 1024,        // 只跟踪大分配
    ..Default::default()
};

memscope_rs::init_with_config(config);
```

## 📊 性能基准

| 操作 | 默认 | 优化后 | 提升 |
|------|------|--------|------|
| 跟踪开销 | 15% | 3% | 5x |
| 导出速度 | 17s | 211ms | 80x |
| 内存使用 | 50MB | 10MB | 5x |

## 🔧 高级技巧

### 1. 条件编译

```rust
#[cfg(feature = "memory-analysis")]
use memscope_rs::{init, track_var};

#[cfg(not(feature = "memory-analysis"))]
macro_rules! track_var {
    ($var:expr) => {};
}
```

### 2. 异步优化

```rust
// 在异步环境中优化
#[tokio::main]
async fn main() {
    init();
    
    // 使用 spawn_blocking 避免阻塞
    let handle = tokio::task::spawn_blocking(|| {
        let tracker = get_global_tracker();
        tracker.export_to_binary("async_analysis")
    });
    
    handle.await??;
}
```

### 3. 内存池模式

```rust
struct MemoryPool {
    buffers: Vec<Vec<u8>>,
}

impl MemoryPool {
    fn get_buffer(&mut self, size: usize) -> Vec<u8> {
        self.buffers.pop()
            .unwrap_or_else(|| Vec::with_capacity(size))
    }
    
    fn return_buffer(&mut self, mut buffer: Vec<u8>) {
        buffer.clear();
        self.buffers.push(buffer);
    }
}
```

## 🎉 总结

通过这些优化技巧：
- 跟踪开销降低 80%
- 导出速度提升 80 倍
- 内存使用减少 80%
- 分析性能显著提升