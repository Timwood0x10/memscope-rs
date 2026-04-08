# 分析模块 (Analysis Module)

## 概述

分析模块提供高级内存分析功能，包括泄漏检测、安全分析、生命周期分析和模式检测。它实现了各种检测器和分析器来识别内存问题并提供洞察。

## 组件

### 1. 检测器

**文件**: `src/analysis/detectors/`

**用途**: 常见内存问题的专用检测器。

#### LeakDetector（泄漏检测器）

检测潜在的内存泄漏：

```rust
pub struct LeakDetector {
    config: LeakDetectorConfig,
}

impl Detector for LeakDetector {
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        // 检测没有对应释放的分配
        let leaked: Vec<_> = allocations.iter()
            .filter(|a| a.timestamp_dealloc.is_none())
            .collect();

        DetectionResult {
            detector_name: "LeakDetector".to_string(),
            issues: leaked.len(),
            details: leaked.iter().map(|a| format!("0x{:x}: {} 字节", a.ptr, a.size)).collect(),
        }
    }
}
```

#### UafDetector（释放后使用检测器）

检测释放后使用问题：

```rust
pub struct UafDetector {
    config: UafDetectorConfig,
}

impl Detector for UafDetector {
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        // 检测释放后的访问
        // ... 实现
    }
}
```

#### OverflowDetector（溢出检测器）

检测缓冲区溢出问题：

```rust
pub struct OverflowDetector {
    config: OverflowDetectorConfig,
}

impl Detector for OverflowDetector {
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        // 检测缓冲区溢出
        // ... 实现
    }
}
```

#### SafetyDetector（安全检测器）

检测安全违规：

```rust
pub struct SafetyDetector {
    config: SafetyDetectorConfig,
}

impl Detector for SafetyDetector {
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        // 检测安全违规
        // ... 实现
    }
}
```

#### LifecycleDetector（生命周期检测器）

检测生命周期问题：

```rust
pub struct LifecycleDetector {
    config: LifecycleDetectorConfig,
}

impl Detector for LifecycleDetector {
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        // 检测生命周期问题
        // ... 实现
    }
}
```

### 2. 专用分析器

#### MemoryPassportTracker（内存护照跟踪器）

使用"护照"跟踪内存生命周期：

```rust
pub struct MemoryPassport {
    pub ptr: usize,
    pub size: usize,
    pub created_at: u64,
    pub status: PassportStatus,
    pub events: Vec<PassportEvent>,
}
```

#### CircularReferenceDetector（循环引用检测器）

检测智能指针中的循环引用：

```rust
pub fn detect_circular_references(allocations: &[AllocationInfo]) -> CircularReferenceAnalysis {
    // 检测引用图中的循环
    // ... 实现
}
```

#### VariableRelationshipAnalyzer（变量关系分析器）

分析变量关系：

```rust
pub fn build_variable_relationship_graph(allocations: &[AllocationInfo]) -> VariableRelationshipGraph {
    // 构建关系图
    // ... 实现
}
```

### 3. 模式分析器

#### AsyncAnalyzer（异步分析器）

分析异步模式：

```rust
pub struct AsyncAnalyzer {
    // 分析 Future 状态机
}
```

#### BorrowAnalyzer（借用分析器）

分析借用模式：

```rust
pub struct BorrowAnalyzer {
    // 分析借用检查器模式
}
```

#### GenericAnalyzer（泛型分析器）

分析泛型类型使用：

```rust
pub struct GenericAnalyzer {
    // 分析泛型实例化
}
```

#### ClosureAnalyzer（闭包分析器）

分析闭包捕获：

```rust
pub struct ClosureAnalyzer {
    // 分析闭包生命周期模式
}
```

### 4. AnalysisManager（分析管理器）

**文件**: `src/analysis/mod.rs`

**用途**: 整合所有分析功能。

**核心实现**:

```rust
pub struct AnalysisManager {
    // 整合所有分析功能
}

impl AnalysisManager {
    /// 分析内存碎片
    pub fn analyze_fragmentation(&self, allocations: &[AllocationInfo]) -> FragmentationAnalysis {
        // ... 实现
    }

    /// 分析系统库使用
    pub fn analyze_system_libraries(&self, allocations: &[AllocationInfo]) -> SystemLibraryStats {
        // ... 实现
    }

    /// 分析并发安全性
    pub fn analyze_concurrency_safety(&self, allocations: &[AllocationInfo]) -> ConcurrencyAnalysis {
        // ... 实现
    }

    /// 执行综合分析
    pub fn perform_comprehensive_analysis(
        &self,
        allocations: &[AllocationInfo],
        stats: &MemoryStats,
    ) -> ComprehensiveAnalysisReport {
        // ... 实现
    }
}
```

## 使用示例

### 使用检测器

```rust
use memscope::analysis::detectors::{LeakDetector, LeakDetectorConfig};

// 创建泄漏检测器
let detector = LeakDetector::new(LeakDetectorConfig::default());

// 运行检测
let result = detector.detect(&allocations);

println!("检测到 {} 个潜在泄漏", result.issues);
for detail in &result.details {
    println!("  {}", detail);
}
```

### 综合分析

```rust
use memscope::analysis::AnalysisManager;

let manager = AnalysisManager::new();

// 执行综合分析
let report = manager.perform_comprehensive_analysis(&allocations, &stats);

println!("碎片率: {}", report.fragmentation_analysis.fragmentation_ratio);
println!("不安全 FFI 操作: {}", report.unsafe_ffi_stats.total_operations);
```

### 模式分析

```rust
// 分析异步模式
let async_analyzer = get_global_async_analyzer();
let async_analysis = async_analyzer.analyze_async_patterns();

// 分析借用模式
let borrow_analyzer = get_global_borrow_analyzer();
let borrow_analysis = borrow_analyzer.analyze_borrow_patterns();

// 分析闭包模式
let closure_analyzer = get_global_closure_analyzer();
let closure_analysis = closure_analyzer.analyze_closure_patterns(&allocations);
```

## 设计原则

### 1. 模块化
检测器和分析器是独立的：
- **优势**: 易于添加新分析
- **权衡**: 协调更复杂

### 2. 可扩展性
易于添加自定义检测器：
- **优势**: 针对特定用例灵活
- **权衡**: API 复杂性

### 3. 高效性
针对高效分析进行优化：
- **优势**: 快速分析
- **权衡**: 可能使用更多内存

## 最佳实践

1. **检测器选择**: 根据用例选择合适的检测器
2. **配置**: 自定义检测器配置以获得更好的结果
3. **错误处理**: 始终处理分析错误
4. **性能**: 尽可能缓存分析结果

## 限制

1. **误报**: 检测器可能报告误报
2. **性能**: 复杂分析可能较慢
3. **内存使用**: 分析可能消耗大量内存
4. **上下文**: 某些分析需要额外的上下文

## 未来改进

1. **更好的准确性**: 减少误报
2. **更多检测器**: 添加更多专用检测器
3. **性能**: 提高分析性能
4. **机器学习**: 使用 ML 进行模式检测
5. **实时分析**: 支持实时分析

## 关系推断引擎

### 概述

关系推断引擎能够自动检测内存分配之间的语义关系，帮助理解程序的内存结构和所有权模型。

### 支持的关系类型

| 关系类型 | 描述 | 检测方法 |
|----------|------|----------|
| **Owner** | A 拥有或指向 B | 指针扫描 - 在 A 的内存中找到指向 B 的指针 |
| **Slice** | A 是 B 的子视图 | 地址范围检测 - A 的指针在 B 的内部 |
| **Clone** | A 是 B 的克隆 | 内容相似度 + 时间窗口 + 调用栈匹配 |
| **Shared** | A 和 B 共享所有权 | Arc/Rc control block 模式识别 |

### 使用方法

```rust
use memscope_rs::analysis::relation_inference::{RelationGraphBuilder, Relation};

// 从活跃分配构建关系图
let graph = RelationGraphBuilder::build(&allocations, None);

// 查询关系
for edge in &graph.edges {
    println!("{:?}: {} -> {}", edge.relation, edge.from, edge.to);
}

// 检测循环引用
let cycles = graph.detect_cycles();
if !cycles.is_empty() {
    println!("检测到 {} 个循环引用", cycles.len());
}

// 获取所有节点
let nodes = graph.all_nodes();
```

### 准确率数据

基于真实测试数据的准确率：

```
=== Clone 检测准确率 ===
Precision: 100.00%
Recall: 100.00%
F1 Score: 100.00%

=== Owner 检测准确率 ===
✅ Box<Vec> 关系正确检测
✅ 独立 Vec 无误报

=== 性能数据 ===
1000 分配构建时间: ~230ms
```

### 配置选项

```rust
use memscope_rs::analysis::relation_inference::{GraphBuilderConfig, CloneConfig};

let config = GraphBuilderConfig {
    clone_config: CloneConfig {
        min_similarity: 0.8,              // 最小相似度阈值
        min_similarity_no_stack_hash: 0.95, // 无调用栈时的更严格阈值
        max_time_diff_ns: 10_000_000,     // 10ms 时间窗口
        max_clone_edges_per_node: 10,     // 每节点最大克隆边数
        ..Default::default()
    },
};

let graph = RelationGraphBuilder::build(&allocations, Some(config));
```

### 检测算法详解

#### Owner 检测

扫描分配内存中的指针值，如果找到指向其他分配的指针，则建立 Owner 关系。

```rust
// 检测流程
1. 读取分配内存内容
2. 扫描每个 8 字节对齐位置
3. 解析为指针值
4. 在 RangeMap 中查找目标分配
5. 建立 Owner 边
```

#### Slice 检测

检测一个分配的指针是否落在另一个分配的内部（非起始位置）。

```rust
// 检测条件
1. A.ptr 在 B 的地址范围内 (B.ptr < A.ptr < B.ptr + B.size)
2. A.size <= 256 (Slice 元数据通常较小)
3. A 的完整范围在 B 内
```

#### Clone 检测

基于内容相似度和时间窗口检测克隆关系。

```rust
// 检测条件
1. 相同的 TypeKind 和 size
2. 相同的 call_stack_hash (或都为 None)
3. 分配时间差在窗口内
4. 内容相似度 >= 阈值
```

#### Shared 检测

检测 Arc/Rc 共享所有权。

```rust
// 检测条件
1. 多个 Owner 边指向同一目标
2. 目标内存布局符合 ArcInner 结构
3. strong_count 和 weak_count 在合理范围内
```

### 注意事项

1. **Owner 检测**：需要元数据在堆上（如 `Box<Vec>`），栈上的元数据无法被扫描
2. **Clone 检测**：依赖调用栈哈希，相同调用栈的分配会被分组比较
3. **Shared 检测**：依赖 Owner 关系，需要先检测 Owner 后再检测 Shared
4. **性能**：使用滑动时间窗口避免 O(n²) 复杂度