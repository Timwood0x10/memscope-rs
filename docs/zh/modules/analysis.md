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