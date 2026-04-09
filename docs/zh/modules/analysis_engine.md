# 分析引擎模块 (Analysis Engine Module)

## 概述

分析引擎提供可插拔的内存分析功能。它协调多个分析器，检查内存快照以检测泄漏、碎片化和安全违规等问题。

## 组件

### 1. AnalysisEngine

**文件**: `src/analysis_engine/engine.rs`

**用途**: 协调多个分析器并在内存快照上运行它们。

**核心实现**:

```rust
pub struct AnalysisEngine {
    snapshot_engine: SharedSnapshotEngine,
    analyzers: Vec<Box<dyn Analyzer>>,
}

impl AnalysisEngine {
    pub fn new(snapshot_engine: SharedSnapshotEngine) -> Self

    pub fn register_analyzer(&mut self, analyzer: Box<dyn Analyzer>)

    pub fn analyze(&self) -> Vec<AnalysisResult> {
        let snapshot = self.snapshot_engine.build_snapshot();
        self.analyze_snapshot(&snapshot)
    }

    pub fn analyze_snapshot(&self, snapshot: &MemorySnapshot) -> Vec<AnalysisResult> {
        self.analyzers
            .iter()
            .map(|analyzer| analyzer.analyze(snapshot))
            .collect()
    }
}
```

### 2. Analyzer Trait

**文件**: `src/analysis_engine/analyzer.rs`

**用途**: 定义所有分析器的接口。

```rust
pub trait Analyzer: Send + Sync {
    fn name(&self) -> &str;
    fn analyze(&self, snapshot: &MemorySnapshot) -> AnalysisResult;
}
```

### 3. AnalysisResult

```rust
pub struct AnalysisResult {
    pub analyzer_name: String,
    pub issue_count: usize,
    pub severity: Severity,
    pub description: String,
    pub findings: Vec<Finding>,
}

pub struct Finding {
    pub issue_type: String,
    pub description: String,
    pub ptr: Option<usize>,
    pub size: Option<usize>,
    pub context: String,
}

pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}
```

### 4. DetectorToAnalyzer 适配器

**文件**: `src/analysis_engine/detector_adapter.rs`

**用途**: 将遗留的检测器模式适配到分析器接口。

## 设计理念

1. **可插拔**: 分析器在运行时注册
2. **可组合**: 多个分析器按顺序运行
3. **线程安全**: 所有 trait 都是 `Send + Sync`
4. **可扩展**: 易于添加新的分析器

## 使用示例

```rust
use memscope_rs::analysis_engine::{AnalysisEngine, Analyzer};
use memscope_rs::snapshot::engine::SnapshotEngine;

struct MyAnalyzer;

impl Analyzer for MyAnalyzer {
    fn name(&self) -> &str { "my_analyzer" }

    fn analyze(&self, snapshot: &MemorySnapshot) -> AnalysisResult {
        // 分析快照...
        AnalysisResult {
            analyzer_name: self.name().to_string(),
            issue_count: 0,
            severity: Severity::Info,
            description: "分析完成".to_string(),
            findings: vec![],
        }
    }
}

let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
let mut analysis_engine = AnalysisEngine::new(snapshot_engine);
analysis_engine.register_analyzer(Box::new(MyAnalyzer));

let results = analysis_engine.analyze();
```

## 与其他模块的集成

```
捕获引擎
    ↓
事件存储（存储事件）
    ↓
快照引擎（构建快照）
    ↓
分析引擎（运行分析器）
    ↓
渲染引擎（输出结果）
```

## 限制

1. **离线分析**: 基于快照工作，非实时
2. **内存开销**: 所有分析器保存在内存中
3. **无增量分析**: 每次都完整重新分析
