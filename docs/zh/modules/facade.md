# 外观层模块 (Facade Module)

## 概述

外观层模块提供 `MemScope`，这是一个统一的接口，将所有引擎集成到一个简单易用的 API 中。它通过隐藏协调多个引擎的复杂性来简化用户体验。

## MemScope 结构

**文件**: `src/facade/facade.rs`

**用途**: 所有引擎的统一外观。

```rust
pub struct MemScope {
    pub event_store: Arc<EventStore>,
    pub capture: Arc<CaptureEngine>,
    pub metadata: Arc<MetadataEngine>,
    pub snapshot: Arc<SnapshotEngine>,
    pub query: Arc<QueryEngine>,
    pub analysis: Arc<Mutex<AnalysisEngine>>,
    pub timeline: Arc<TimelineEngine>,
    pub render: Arc<RenderEngine>,
}
```

## 模块组成

```
┌─────────────────────────────────────────────────────────────┐
│                         MemScope                            │
│  (外观层 - 所有引擎的统一接口)                                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │ EventStore  │  │  捕获      │  │   元数据    │       │
│  │ (存储)      │←─┤  引擎      │  │   引擎      │       │
│  └─────────────┘  └─────────────┘  └─────────────┘       │
│         │               │               │                 │
│         └───────────────┼───────────────┘                 │
│                         │                                 │
│                   ┌─────┴─────┐                         │
│                   │  快照引擎  │                         │
│                   └─────┬─────┘                         │
│                         │                                 │
│         ┌───────────────┼───────────────┐               │
│         │               │               │               │
│         ▼               ▼               ▼               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │   查询引擎   │  │   分析引擎   │  │  时间线引擎  │    │
│  └─────────────┘  └─────────────┘  └─────────────┘    │
│                         │                                 │
│                         ▼                                 │
│                   ┌─────────────┐                       │
│                   │   渲染引擎   │                       │
│                   └─────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

## 创建方法

```rust
impl MemScope {
    /// 使用默认设置创建（统一后端）
    pub fn new() -> Self

    /// 使用特定后端创建
    pub fn with_backend(backend_type: CaptureBackendType) -> Self
}
```

## 使用示例

```rust
use memscope_rs::MemScope;

let memscope = MemScope::new();

// 捕获事件（通常通过 track! 宏完成）
// memscope.capture.capture_alloc(ptr, size, thread_id);

// 构建和查询快照
let snapshot = memscope.snapshot.build_snapshot();

// 查询分配
let allocs = memscope.query.query_allocations(&snapshot, ...);

// 分析
let analysis_results = {
    let analysis = memscope.analysis.lock().unwrap();
    analysis.analyze()
};

// 导出
memscope.render.export_json("output.json");
```

## 关键设计决策

1. **所有引擎都使用 Arc 包装**: 支持共享所有权
2. **分析使用 Mutex**: 同一时间只能有一个分析
3. **快照可查询**: 直接访问内存状态
4. **渲染连接快照**: 解耦的渲染

## 限制

1. **内部可变性**: 分析引擎包装在 Mutex 中
2. **无背压**: 事件排队无限制
3. **单一快照视图**: 无并发快照视图
