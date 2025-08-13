# 安装配置

本指南将帮你在不同环境中正确安装和配置 memscope-rs。

## 🚀 快速安装

### 基础安装
在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
memscope-rs = "0.1.4"
```

这将启用默认特性，包括：
- `tracking-allocator` - 全局分配器跟踪
- 所有核心功能

### 最小安装
如果你只需要基础功能：

```toml
[dependencies]
memscope-rs = { version = "0.1.4", default-features = false }
```

## 🎛️ 特性配置

### 可用特性

| 特性 | 默认 | 描述 | 适用场景 |
|------|------|------|----------|
| `tracking-allocator` | ✅ | 全局分配器跟踪 | 自动内存跟踪 |
| `backtrace` | ❌ | 调用栈跟踪 | 详细调试信息 |
| `derive` | ❌ | 派生宏支持 | 自定义类型跟踪 |
| `test` | ❌ | 测试工具 | 单元测试 |

### 特性组合示例

**完整功能配置**:
```toml
[dependencies]
memscope-rs = { 
    version = "0.1.4", 
    features = ["tracking-allocator", "backtrace", "derive"] 
}
```

**性能优化配置**:
```toml
[dependencies]
memscope-rs = { 
    version = "0.1.4", 
    features = ["tracking-allocator"] 
}
```

**调试配置**:
```toml
[dependencies]
memscope-rs = { 
    version = "0.1.4", 
    features = ["tracking-allocator", "backtrace"] 
}
```

**测试配置**:
```toml
[dev-dependencies]
memscope-rs = { 
    version = "0.1.4", 
    features = ["test"] 
}
```

## 🏗️ 环境配置

### 标准 Rust 项目
```toml
# Cargo.toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"

[dependencies]
memscope-rs = "0.1.4"
```

```rust
// src/main.rs
use memscope_rs::{init, track_var, get_global_tracker};

fn main() {
    init();
    
    let data = vec![1, 2, 3];
    track_var!(data);
    
    let tracker = get_global_tracker();
    tracker.export_to_html("analysis.html").unwrap();
}
```

### 库项目配置
```toml
# Cargo.toml
[package]
name = "my-library"
version = "0.1.0"
edition = "2021"

[dependencies]
memscope-rs = { version = "0.1.4", optional = true }

[features]
default = []
memory-analysis = ["memscope-rs"]
```

```rust
// src/lib.rs
#[cfg(feature = "memory-analysis")]
use memscope_rs::track_var;

pub fn process_data(data: Vec<i32>) -> Vec<i32> {
    #[cfg(feature = "memory-analysis")]
    track_var!(data);
    
    // 处理逻辑...
    data.into_iter().map(|x| x * 2).collect()
}
```

### no_std 环境
```toml
[dependencies]
memscope-rs = { 
    version = "0.1.4", 
    default-features = false,
    features = [] 
}
```

```rust
#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use memscope_rs::MemoryTracker;

fn main() {
    let tracker = MemoryTracker::new();
    // 手动跟踪模式...
}
```

## 🔧 开发环境设置

### VS Code 配置
创建 `.vscode/settings.json`:

```json
{
    "rust-analyzer.cargo.features": [
        "tracking-allocator",
        "backtrace"
    ],
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.extraArgs": [
        "--",
        "-W",
        "clippy::all"
    ]
}
```

### Cargo 配置
创建 `.cargo/config.toml`:

```toml
[build]
rustflags = ["-C", "debug-assertions=on"]

[env]
RUST_LOG = { value = "memscope_rs=debug", force = true }
RUST_BACKTRACE = { value = "1", force = true }

[alias]
analyze = "run --features backtrace --"
test-memory = "test --features test --"
```

### 环境变量
```bash
# 开发环境
export RUST_LOG=memscope_rs=debug
export RUST_BACKTRACE=1

# 生产环境
export RUST_LOG=memscope_rs=info
export MEMSCOPE_OUTPUT_DIR=/var/log/memscope
```

## 🐳 容器化部署

### Dockerfile
```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 构建时启用所有特性
RUN cargo build --release --features "tracking-allocator,backtrace"

FROM debian:bullseye-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/my-app /usr/local/bin/

# 创建输出目录
RUN mkdir -p /var/log/memscope
ENV MEMSCOPE_OUTPUT_DIR=/var/log/memscope

CMD ["my-app"]
```

### Docker Compose
```yaml
version: '3.8'
services:
  app:
    build: .
    environment:
      - RUST_LOG=memscope_rs=info
      - MEMSCOPE_OUTPUT_DIR=/data/memscope
    volumes:
      - ./memscope-data:/data/memscope
    ports:
      - "8080:8080"
```

## 🧪 测试配置

### 单元测试
```toml
[dev-dependencies]
memscope-rs = { version = "0.1.4", features = ["test"] }
tokio-test = "0.4"
```

```rust
#[cfg(test)]
mod tests {
    use memscope_rs::{init, track_var, get_global_tracker};

    #[test]
    fn test_memory_tracking() {
        init();
        
        let data = vec![1, 2, 3];
        track_var!(data);
        
        let tracker = get_global_tracker();
        let stats = tracker.get_stats().unwrap();
        assert!(stats.active_allocations > 0);
    }
}
```

### 集成测试
```rust
// tests/integration_test.rs
use memscope_rs::{init, track_var, get_global_tracker};

#[test]
fn integration_test() {
    init();
    
    // 模拟真实使用场景
    let large_data = vec![0; 1024 * 1024];
    track_var!(large_data);
    
    let tracker = get_global_tracker();
    
    // 验证导出功能
    assert!(tracker.export_to_json("integration_test").is_ok());
    
    // 验证文件生成
    let path = std::path::Path::new("MemoryAnalysis/integration_test");
    assert!(path.exists());
}
```

### 基准测试
```toml
[[bench]]
name = "memory_tracking"
harness = false

[dev-dependencies]
criterion = "0.5"
```

```rust
// benches/memory_tracking.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memscope_rs::{init, track_var};

fn benchmark_tracking(c: &mut Criterion) {
    init();
    
    c.bench_function("track_var", |b| {
        b.iter(|| {
            let data = black_box(vec![1, 2, 3, 4, 5]);
            track_var!(data);
        })
    });
}

criterion_group!(benches, benchmark_tracking);
criterion_main!(benches);
```

## 🚀 性能优化配置

### 发布构建
```toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### 条件编译
```rust
// 只在调试模式下启用跟踪
#[cfg(debug_assertions)]
use memscope_rs::{init, track_var};

#[cfg(debug_assertions)]
macro_rules! debug_track {
    ($var:expr) => { track_var!($var) };
}

#[cfg(not(debug_assertions))]
macro_rules! debug_track {
    ($var:expr) => {};
}

fn main() {
    #[cfg(debug_assertions)]
    init();
    
    let data = vec![1, 2, 3];
    debug_track!(data);
}
```

## 🔍 验证安装

### 快速验证脚本
```rust
// verify_installation.rs
use memscope_rs::{init, track_var, get_global_tracker};

fn main() {
    println!("🔍 验证 memscope-rs 安装...");
    
    // 1. 初始化测试
    match std::panic::catch_unwind(|| init()) {
        Ok(_) => println!("✅ 初始化成功"),
        Err(_) => {
            println!("❌ 初始化失败");
            return;
        }
    }
    
    // 2. 跟踪测试
    let test_data = vec![1, 2, 3];
    track_var!(test_data);
    println!("✅ 变量跟踪成功");
    
    // 3. 统计测试
    let tracker = get_global_tracker();
    match tracker.get_stats() {
        Ok(stats) => {
            println!("✅ 统计获取成功: {} 个活跃分配", stats.active_allocations);
        }
        Err(e) => {
            println!("❌ 统计获取失败: {}", e);
            return;
        }
    }
    
    // 4. 导出测试
    match tracker.export_to_json("verification_test") {
        Ok(_) => println!("✅ JSON 导出成功"),
        Err(e) => println!("⚠️ JSON 导出失败: {}", e),
    }
    
    println!("🎉 memscope-rs 安装验证完成！");
}
```

运行验证：
```bash
cargo run --bin verify_installation
```

## 📋 安装检查清单

- [ ] ✅ Cargo.toml 中添加了正确的依赖
- [ ] ✅ 选择了合适的特性配置
- [ ] ✅ 代码中正确导入了必要的宏和函数
- [ ] ✅ 在 main() 函数开始处调用了 init()
- [ ] ✅ 验证脚本运行成功
- [ ] ✅ 能够生成和查看导出文件
- [ ] ✅ 测试用例通过

## 🆘 常见安装问题

如果遇到问题，请查看 [常见问题解决](../user-guide/troubleshooting.md) 或：

1. 确认 Rust 版本 >= 1.70
2. 检查网络连接和 crates.io 访问
3. 清理构建缓存：`cargo clean`
4. 更新依赖：`cargo update`

安装成功后，继续阅读 [快速开始](quick-start.md) 开始使用！ 🎯