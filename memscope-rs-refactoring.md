# memscope-rs 项目重构建议

## 项目精简和合并建议

### 1. 核心模块合并

1. **合并跟踪器相关文件**:
   - 将 `src/tracker.rs` 和 `src/scope_tracker.rs` 合并为一个 `src/tracking.rs` 文件
   - 这两个文件功能相关，都是关于跟踪内存分配和作用域的，合并后可以减少跨文件引用

2. **合并可视化相关文件**:
   - 将 `src/visualization.rs`、`src/advanced_charts.rs` 和 `src/html_export.rs` 合并为一个 `src/visualization.rs` 文件
   - 将 `src/optimized_html_export.rs` 合并到上面的文件中，因为它们都是关于数据可视化的

3. **合并导出功能**:
   - 将 `src/export_enhanced.rs` 和 `src/report_generator.rs` 合并为一个 `src/export.rs` 文件
   - 这些文件都是关于数据导出和报告生成的，功能上有重叠

4. **合并分析功能**:
   - 将 `src/advanced_analysis.rs` 和 `src/unsafe_ffi_visualization.rs` 合并为一个 `src/analysis.rs` 文件
   - 这些都是关于数据分析的功能

### 2. 简化类型定义

1. **精简类型定义**:
   - `src/types.rs` 文件过于庞大（1326行），包含了太多的类型定义
   - 建议将类型按功能分组，拆分为几个更小的文件：
     - `src/types/core.rs`: 核心类型和错误处理
     - `src/types/allocation.rs`: 分配相关类型
     - `src/types/visualization.rs`: 可视化相关类型
     - `src/types/analysis.rs`: 分析相关类型

2. **减少重复的数据结构**:
   - 当前有多个相似的报告结构，如 `ComprehensiveReport`、`MemoryOverview` 等
   - 可以合并这些结构，减少冗余

### 3. 合并示例文件

1. **整合示例文件**:
   - `examples` 目录下有多个相似的示例文件，如 `test_html_dashboard.rs`、`test_fast_dashboard.rs` 等
   - 建议合并为更少的、更有代表性的示例，每个示例展示不同的功能点

2. **创建分类示例**:
   - 基础用法示例：展示核心功能
   - 高级用法示例：展示高级分析功能
   - 可视化示例：展示不同的可视化选项

### 4. 简化模块结构

1. **减少模块层级**:
   - 当前 `lib.rs` 中导出了太多模块，使得API过于复杂
   - 建议将相关功能组合成更少的公共模块，隐藏内部实现细节

2. **重新组织公共API**:
   - 将核心功能集中在几个主要模块中：
     - `tracking`: 内存跟踪功能
     - `analysis`: 分析功能
     - `visualization`: 可视化功能
     - `export`: 导出功能

### 5. 解决重复导出问题

1. **修复重复导出**:
   - `lib.rs` 中有重复导出的问题，如 `MemoryTracker` 被定义多次
   - 需要整理导出，确保每个类型只导出一次

### 6. 具体文件合并建议

```
src/
├── tracking.rs (合并 tracker.rs 和 scope_tracker.rs)
├── analysis.rs (合并 advanced_analysis.rs 和 unsafe_ffi_tracker.rs)
├── visualization.rs (合并 visualization.rs, advanced_charts.rs, html_export.rs 和 optimized_html_export.rs)
├── export.rs (合并 export_enhanced.rs 和 report_generator.rs)
├── allocator.rs (保持不变)
├── types/
│   ├── core.rs
│   ├── allocation.rs
│   ├── visualization.rs
│   └── analysis.rs
└── utils.rs (保持不变)
```

### 7. 精简示例目录

```
examples/
├── basic_usage.rs (基础用法示例)
├── advanced_analysis.rs (合并复杂分析相关示例)
├── visualization_showcase.rs (合并可视化相关示例)
└── unsafe_ffi_demo.rs (保持不变，因为这是特定功能)
```

## 重构后的模块结构

### 1. 核心跟踪模块 (`src/tracking.rs`)

该模块将包含所有与内存跟踪相关的功能：

- 内存分配跟踪
- 作用域跟踪
- 变量关联
- 统计收集

### 2. 分析模块 (`src/analysis.rs`)

该模块将包含所有与数据分析相关的功能：

- 内存使用分析
- 类型分析
- 不安全代码分析
- FFI调用分析
- 性能分析

### 3. 可视化模块 (`src/visualization.rs`)

该模块将包含所有与数据可视化相关的功能：

- SVG图表生成
- HTML仪表板
- 时间线可视化
- 交互式报告

### 4. 导出模块 (`src/export.rs`)

该模块将包含所有与数据导出相关的功能：

- JSON导出
- HTML报告生成
- SVG导出
- 增强型JSON导出

### 5. 类型模块 (`src/types/`)

将大型类型定义文件拆分为更小的、功能聚焦的文件：

- `core.rs`: 错误类型、结果类型、基本接口
- `allocation.rs`: 分配信息、内存统计
- `visualization.rs`: 可视化相关类型
- `analysis.rs`: 分析相关类型

## 重构后的公共API

```rust
// lib.rs
pub mod tracking;
pub mod analysis;
pub mod visualization;
pub mod export;
pub mod types;
pub mod allocator;
pub mod utils;

// 重新导出核心功能，保持向后兼容
pub use allocator::TrackingAllocator;
pub use tracking::{get_global_tracker, MemoryTracker, track_var, init};
pub use types::core::{TrackingError, TrackingResult};
pub use visualization::{export_memory_analysis, export_lifecycle_timeline};
pub use export::generate_interactive_html_report;

// 设置全局分配器
#[cfg(feature = "tracking-allocator")]
#[global_allocator]
pub static GLOBAL: TrackingAllocator = TrackingAllocator::new();
```

## 重构的好处

1. **减少文件数量**：从20多个文件减少到约10个核心文件
2. **提高代码组织**：相关功能集中在同一个文件中
3. **简化API**：更清晰的模块结构和导出
4. **减少重复**：合并相似功能，减少代码重复
5. **提高可维护性**：更容易理解和修改代码
6. **更好的文档**：更集中的功能更容易文档化

## 实施步骤

1. 创建新的文件结构
2. 将相关功能合并到新文件中
3. 更新内部引用路径
4. 更新公共API导出
5. 更新文档和示例
6. 运行测试确保功能正常

这种重构保持了所有现有功能，同时大大简化了代码结构，使项目更易于维护和理解。