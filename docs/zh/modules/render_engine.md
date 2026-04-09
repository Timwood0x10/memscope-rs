# 渲染引擎模块 (Render Engine Module)

## 概述

渲染引擎负责以各种格式（JSON、HTML、Binary）输出内存数据。它提供了一个可插拔的渲染器系统，可以扩展自定义格式。

## 组件

### 1. RenderEngine

**文件**: `src/render_engine/engine.rs`

**用途**: 协调多个渲染器并提供统一的渲染接口。

```rust
pub struct RenderEngine {
    snapshot_engine: SharedSnapshotEngine,
    renderers: Vec<Box<dyn Renderer>>,
}

impl RenderEngine {
    pub fn new(snapshot_engine: SharedSnapshotEngine) -> Self

    pub fn register_renderer(&mut self, renderer: Box<dyn Renderer>)
}
```

### 2. Renderer Trait

**文件**: `src/render_engine/renderer.rs`

```rust
pub trait Renderer: Send + Sync {
    fn format(&self) -> OutputFormat;
    fn render(&self, snapshot: &MemorySnapshot, config: &RenderConfig) -> Result<RenderResult, String>;
}
```

### 3. 输出格式

```rust
pub enum OutputFormat {
    Json,
    Html,
    Binary,
    Svg,
}
```

### 4. 导出函数

**文件**: `src/render_engine/export.rs`

```rust
pub fn export_snapshot_to_json(
    snapshot: &MemorySnapshot,
    path: &Path,
    options: &ExportJsonOptions,
) -> Result<(), ExportError>

pub fn export_leak_detection_json(
    detection_result: &DetectionResult,
    path: &Path,
) -> Result<(), ExportError>

pub fn export_dashboard_html(
    output_path: &str,
    tracker: &Tracker,
    ...
) -> Result<()>
```

### 5. 仪表盘渲染器

**文件**: `src/render_engine/dashboard/renderer.rs`

**用途**: 生成带 JavaScript 图表的交互式 HTML 仪表盘。

## 架构

```
RenderEngine
    │
    ├── JsonRenderer ───→ JSON 文件
    ├── HtmlRenderer ──→ HTML 仪表盘
    ├── BinaryRenderer ─→ 紧凑二进制格式
    └── SvgRenderer ───→ SVG 图表
```

## 设计决策

1. **可插拔渲染器**: 易添加新格式
2. **基于快照**: 从内存快照渲染
3. **惰性渲染**: 按需渲染

## 使用示例

```rust
use memscope_rs::render_engine::{RenderEngine, ExportJsonOptions};

let engine = RenderEngine::new(snapshot_engine);

// 导出为 JSON
let options = ExportJsonOptions { verbose: true };
engine.export_snapshot_to_json(&snapshot, Path::new("output.json"), &options);

// 导出为 HTML 仪表盘
engine.export_dashboard_html("dashboard.html", &tracker);
```

## 限制

1. **内存中完整快照**: 渲染需要整个快照
2. **无流式处理**: 无法增量渲染
3. **单次输出**: 无批量导出
