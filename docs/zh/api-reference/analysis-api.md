# 分析 API

memscope-rs 分析 API 的完整参考文档。

## 核心分析接口

### `MemoryAnalyzer`

主要的内存分析器接口。

```rust
use memscope_rs::analysis::MemoryAnalyzer;

let analyzer = MemoryAnalyzer::new();
let results = analyzer.analyze_memory_patterns()?;
```

## 分析结果类型

### `AnalysisResult`

包含完整分析结果的数据结构。

### `MemoryPattern`

识别的内存使用模式。

## 使用示例

详细的使用示例和最佳实践。