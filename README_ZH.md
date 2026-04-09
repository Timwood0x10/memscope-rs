# memscope-rs

[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

高性能 Rust 内存追踪库，采用模块化引擎架构。

## 🚀 v0.2.0 - 重大重构

**最新变更：**
- 🏗️ **架构重构**：从单体架构迁移到模块化引擎架构
- 📉 **代码精简**：减少约 75% 代码（删除 265K 行）
- 🔒 **安全性增强**：消除所有不安全的 `unwrap()` 调用
- ⚡ **性能提升**：并发追踪场景性能提升高达 98%
- 📊 **代码统计**：525 个文件修改，当前代码库：77,641 行

详见 [PR 摘要](PR_SUMMARY.md) 了解详细变更和迁移指南。

## 架构

```mermaid
graph TB
    subgraph "用户代码"
        A[track_var! 宏]
        B[track_scope! 宏]
    end

    subgraph "门面层"
        C[统一追踪 API]
    end

    subgraph "引擎层"
        D[捕获引擎]
        E[分析引擎]
        F[事件存储引擎]
        G[渲染引擎]
        H[快照引擎]
        I[时间轴引擎]
        J[查询引擎]
        K[元数据引擎]
    end

    subgraph "后端"
        L[CoreTracker]
        M[LockfreeTracker]
        N[AsyncTracker]
        O[GlobalTracker]
    end

    A --> C
    B --> C
    C --> D
    D --> L
    D --> M
    D --> N
    D --> O
    D --> F
    E --> F
    E --> G
    G --> J
    H --> F
    I --> F
    J --> K
```

## 数据流

```mermaid
sequenceDiagram
    participant User as 用户代码
    participant Facade as 门面 API
    participant Capture as 捕获引擎
    participant EventStore as 事件存储引擎
    participant Analysis as 分析引擎
    participant Render as 渲染引擎

    User->>Facade: track_var!(data)
    Facade->>Capture: 捕获分配(ptr, size)
    Capture->>EventStore: 存储事件
    User->>Facade: analyze()
    Facade->>Analysis: 检测问题
    Analysis->>EventStore: 读取事件
    Analysis-->>Facade: 返回报告
    User->>Facade: export_json()
    Facade->>Render: 渲染数据
    Render-->>User: 输出文件
```

## 模块概览

```mermaid
graph LR
    subgraph "核心层"
        facade[facade/]
        tracker[tracker/]
    end

    subgraph "引擎层"
        capture[capture/]
        analysis[analysis_engine/]
        event[event_store/]
        render[render_engine/]
        snapshot[snapshot/]
        timeline[timeline/]
        query[query/]
        metadata[metadata/]
    end

    subgraph "分析模块"
        detectors[detectors/]
        safety[safety/]
        classification[classification/]
    end

    facade --> tracker
    tracker --> capture
    capture --> event
    capture --> analysis
    analysis --> detectors
    analysis --> safety
    analysis --> classification
    analysis --> snapshot
    analysis --> timeline
    event --> query
    event --> render
```

## 快速开始

```rust
use memscope_rs::tracker::{track_var, track_scope};

fn main() {
    // 追踪变量
    let data = track_var!(vec![1, 2, 3, 4, 5]);
    
    // 追踪作用域
    {
        let _guard = track_scope!("处理");
        // 你的代码
    }
    
    // 分析内存使用
    let tracker = memscope_rs::tracker::get_tracker();
    let report = tracker.analyze();
    println!("分配次数: {}", report.total_allocations);
}
```

## 追踪后端

| 后端          | 使用场景            | 性能    | 说明                          |
| ------------- | ------------------- | ------- | ----------------------------- |
| CoreTracker   | 单线程             | ~23ns   | 简单，低开销                  |
| LockfreeTracker | 多线程           | ~39ns   | 无锁，线程本地存储            |
| AsyncTracker  | 异步任务           | ~23ns   | 任务 ID 追踪                 |
| GlobalTracker | 全局追踪           | 可变    | 跨线程共享                    |

## 引擎能力

### 分析引擎
- **内存泄漏检测** - 查找未释放的分配
- **释放后使用检测** - 检测 UAF 模式
- **缓冲区溢出检测** - 查找边界违规
- **安全性分析** - 不安全代码风险评估
- **循环引用检测** - 检测引用循环
- **关系推断** - 追踪变量关系

### 捕获引擎
- **多后端支持** - Core、Lockfree、Async、Global
- **智能指针追踪** - Rc/Arc/Box/Weak 支持
- **线程本地存储** - 高效并发追踪
- **FFI 边界追踪** - FFI 调用的内存护照

### 事件存储引擎
- **无锁队列** - 高吞吐量事件存储
- **快照支持** - 时间点视图
- **线程安全** - 并发读写访问

### 渲染引擎
- **JSON 导出** - 人类可读格式
- **HTML 仪表板** - 交互式可视化
- **二进制导出** - 大数据集的紧凑格式

### 其他引擎
- **快照引擎** - 内存快照构建
- **时间轴引擎** - 基于时间的内存分析
- **查询引擎** - 统一查询接口
- **元数据引擎** - 集中式元数据管理

## 性能

| 指标                      | 性能           | 改进       |
| ------------------------- | -------------- | ---------- |
| 并发追踪 (1线程)          | 98µs           | -98% ⚡     |
| 并发追踪 (64线程)         | 1.9ms          | -25% ⚡     |
| 分析操作 (100元素)        | 30µs           | -91% ⚡     |
| Lockfree 分配             | 39ns           | -46% ⚡     |
| 类型分类                  | 40-56ns        | 1-21% ⚡    |

详见 [benchmarks/run.log](benches/run.log) 查看详细性能数据。

## 安装

```toml
[dependencies]
memscope-rs = "0.2.0"
```

## 从 v0.1.x 迁移

**重要破坏性变更：**
- 追踪 API 移至 `memscope_rs::tracker` 模块
- 错误处理系统完全重构
- 部分内部模块重新组织

**快速迁移：**
```rust
// 旧 API (v0.1.x)
use memscope_rs::{track, tracker};

// 新 API (v0.2.0)
use memscope_rs::tracker::{track_var, track_scope};
```

详见 [PR 摘要](PR_SUMMARY_CN.md) 查看详细迁移指南。

## 示例

```bash
# 基本用法
cargo run --example basic_usage

# 多线程
cargo run --example complex_multithread_showcase

# 异步
cargo run --example comprehensive_async_showcase

# 完整演示与仪表板
cargo run --example global_tracker_showcase

# Merkle 树示例
cargo run --example merkle_tree

# 变量关系
cargo run --example variable_relationships_showcase

# Unsafe FFI 演示
cargo run --example unsafe_ffi_demo
```

## 文档

- [API 指南（中文）](docs/zh/api_guide.md)
- [架构文档（中文）](docs/zh/architecture.md)
- [模块文档（中文）](docs/zh/modules/)

### 关键模块

- [分析模块](docs/zh/modules/analysis.md) - 泄漏检测、关系推断、安全分析
- [追踪器模块](docs/zh/modules/tracker.md) - 核心追踪 API
- [捕获模块](docs/zh/modules/capture.md) - 内存捕获后端
- [渲染引擎](docs/zh/modules/render_engine.md) - 导出和可视化
- [核心模块](docs/zh/modules/core.md) - 核心类型和工具

## 项目结构

```
src/
├── analysis/           # 分析模块
│   ├── detectors/      # 泄漏、UAF、溢出检测器
│   ├── safety/         # 安全分析器
│   ├── classification/  # 类型分类
│   └── ...            # 其他分析模块
├── analysis_engine/    # 分析引擎编排
├── capture/            # 捕获引擎和后端
│   ├── backends/       # Core、Lockfree、Async、Global 追踪器
│   ├── types/          # 捕获数据类型
│   └── platform/       # 平台特定实现
├── core/               # 核心类型和工具
├── error/              # 统一错误处理
├── event_store/        # 事件存储引擎
├── render_engine/      # 输出渲染
│   └── dashboard/      # HTML 模板
├── snapshot/           # 快照引擎
├── timeline/           # 时间轴引擎
├── query/              # 查询引擎
├── metadata/           # 元数据引擎
├── tracker/            # 统一追踪器 API
├── facade/             # 门面 API
└── lib.rs              # 公共 API
```

## 与其他工具对比

| 功能                  | memscope-rs | Valgrind      | AddressSanitizer | Heaptrack |
| --------------------- | ----------- | ------------- | ---------------- | --------- |
| **语言**              | Rust 原生  | C/C++         | C/C++/Rust       | C/C++     |
| **运行时**            | 进程内     | 外部          | 进程内           | 外部      |
| **开销**              | 低         | 高 (10-50x)   | 中等 (2x)        | 中等      |
| **变量名**            | ✅           | ❌             | ❌                | ❌         |
| **源码位置**          | ✅           | ✅             | ✅                | ✅         |
| **泄漏检测**          | ✅           | ✅             | ✅                | ✅         |
| **UAF 检测**          | ✅           | ✅             | ✅                | ⚠️        |
| **缓冲区溢出**        | ⚠️          | ✅             | ✅                | ❌         |
| **线程分析**          | ✅           | ✅             | ✅                | ✅         |
| **异步支持**          | ✅           | ❌             | ❌                | ❌         |
| **FFI 追踪**          | ✅           | ⚠️            | ⚠️               | ⚠️        |
| **HTML 仪表板**       | ✅           | ❌             | ❌                | ⚠️        |
| **生产环境就绪**      | ⚠️          | ❌             | ❌                | ⚠️        |

### 何时使用 memscope-rs

**适合场景：**

- 需要变量级别追踪的 Rust 项目
- 异步/await 应用程序
- 开发和调试
- 理解内存模式
- 智能指针分析

**考虑替代方案：**

- **Valgrind** - 深度内存调试，成熟工具
- **AddressSanitizer** - 生产级 UAF/溢出检测
- **Heaptrack** - C/C++ 项目，成熟分析器

### 限制

- 缓冲区溢出检测基于模式，不是运行时强制
- 不能替代生产环境中的 ASAN/Valgrind
- 需要代码插桩（track! 宏）
- 性能开销因用例而异
- 大数据集分析可能有性能影响（详见 PR 摘要）

## 贡献

欢迎贡献！请阅读我们的贡献指南并向我们的仓库提交拉取请求。

## 许可证

采用 MIT OR Apache-2.0 许可证。

## 致谢

- 用 ❤️ 为 Rust 社区构建
- 灵感来自现有内存追踪工具
- 特别感谢所有贡献者