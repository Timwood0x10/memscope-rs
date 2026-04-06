# 分析引擎与检测器

> 可插拔分析 — 五个内置检测器加自定义扩展支持

---

## 概述

分析系统有两层：

1. **AnalysisEngine** (`src/analysis_engine/`) — 编排层，管理分析器和检测器
2. **Detectors** (`src/analysis/detectors/`) — 独立的分析实现

---

## Detector Trait

**文件:** `src/analysis/detectors/mod.rs`

```rust
pub trait Detector: Send + Sync {
    /// 检测器的人类可读名称
    fn name(&self) -> &str;

    /// 在一组分配上运行检测
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult;
}
```

该 trait 是 `Send + Sync` 的，所以检测器可以从任何线程注册和运行。`detect()` 方法接收 `AllocationInfo` 切片并返回 `DetectionResult`。

---

## 检测结果

**文件:** `src/analysis/detectors/types.rs`

```rust
pub struct DetectionResult {
    pub detector_name: String,
    pub issues: Vec<Issue>,
    pub severity: IssueSeverity,
    pub timestamp: u64,
}

pub struct Issue {
    pub description: String,
    pub ptr: usize,
    pub size: usize,
    pub severity: IssueSeverity,
    pub stack_trace: Option<Vec<String>>,
}

pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

---

## 内置检测器

### 1. LeakDetector（泄漏检测）

**文件:** `src/analysis/detectors/leak_detector.rs`

**算法:** 扫描 `active_allocations` 查找没有匹配释放的条目。如果一个分配存在于活跃集合中但从未被释放，则被视为"泄漏"。

```rust
// leak_detector.rs
pub struct LeakDetector {
    config: LeakDetectorConfig,
}

impl Detector for LeakDetector {
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        let mut issues = Vec::new();

        for alloc in allocations {
            // 检查此分配是否存活时间过长
            let age = now() - alloc.allocated_at_ns;
            if age > self.config.max_age_ns {
                issues.push(Issue {
                    description: format!("位于 {:x} 的分配已存活 {}ms",
                        alloc.ptr, age / 1_000_000),
                    ptr: alloc.ptr,
                    size: alloc.size,
                    severity: IssueSeverity::High,
                    stack_trace: alloc.stack_trace.clone(),
                });
            }
        }

        DetectionResult {
            detector_name: "LeakDetector".to_string(),
            issues,
            severity: if issues.is_empty() { IssueSeverity::Low } else { IssueSeverity::High },
            timestamp: now(),
        }
    }
}
```

**配置:**

```rust
pub struct LeakDetectorConfig {
    pub max_age_ns: u64,           // 标记为泄漏前的最大存活时间
    pub min_size: usize,           // 检查的最小分配大小
    pub ignore_patterns: Vec<String>, // 忽略的类型名
}
```

---

### 2. UafDetector（Use-After-Free 检测）

**文件:** `src/analysis/detectors/uaf_detector.rs`

**算法:** 检测指针在释放后被使用的模式。跟踪分配和释放的时间线，然后检查是否有任何访问事件发生在同一指针的释放之后。

---

### 3. OverflowDetector（溢出检测）

**文件:** `src/analysis/detectors/overflow_detector.rs`

**算法:** 检查写入是否超出分配边界。寻找数据写入超出分配大小的模式，这可能表明缓冲区溢出漏洞。

---

### 4. SafetyDetector（安全检测）

**文件:** `src/analysis/detectors/safety_detector.rs`

**算法:** 通用不安全代码安全违规检测。检查以下模式：
- 没有适当验证的裸指针解引用
- 可能导致未定义行为的不安全块使用模式
- FFI 调用安全违规

---

### 5. LifecycleDetector（生命周期检测）

**文件:** `src/analysis/detectors/lifecycle_detector.rs`

**算法:** RAII/Drop 模式分析。检查是否正确调用了应有确定性清理的分配的 `Drop` trait。识别绕过 RAII 模式的分配。

---

## AnalysisEngine

**文件:** `src/analysis_engine/engine.rs`

AnalysisEngine 编排所有分析器和检测器：

```rust
// engine.rs
pub struct AnalysisEngine {
    analyzers: Vec<Box<dyn Analyzer>>,
    detectors: Vec<Box<dyn Detector>>,
}

impl AnalysisEngine {
    pub fn register_analyzer(&mut self, analyzer: Box<dyn Analyzer>) { ... }
    pub fn register_detector(&mut self, detector: Box<dyn Detector>) { ... }
    pub fn run_all(&self, snapshot: &MemorySnapshot) -> AnalysisReport { ... }
}
```

### 检测器适配器

**文件:** `src/analysis_engine/detector_adapter.rs`

将 `Detector` trait 桥接到 `Analyzer` trait，允许检测器作为分析器使用：

```rust
// detector_adapter.rs
pub struct DetectorAdapter {
    detector: Box<dyn Detector>,
}

impl Analyzer for DetectorAdapter {
    fn analyze(&self, snapshot: &MemorySnapshot) -> AnalysisResult {
        let allocations = snapshot.get_active_allocations();
        let result = self.detector.detect(&allocations);
        AnalysisResult::from_detection(result)
    }
}
```

---

## 通过 MemScope Facade 运行检测器

```rust
use memscope_rs::facade::MemScope;

let memscope = MemScope::new();

// 运行单个检测器
memscope.run_leak_detector()?;
memscope.run_uaf_detector()?;
memscope.run_overflow_detector()?;
memscope.run_safety_detector()?;
memscope.run_lifecycle_detector()?;

// 注册自定义检测器
memscope.register_detector(MyCustomDetector::new())?;

// 运行所有已注册检测器
memscope.run_detectors()?;
```

---

## 额外分析模块

除了核心检测器，`src/analysis/` 目录还包含专门的分析模块：

| 模块 | 文件 | 用途 |
|------|------|------|
| **Async 分析** | `async_analysis.rs` | 任务内存分析、效率评分 |
| **Borrow 分析** | `borrow_analysis.rs` | 可变/不可变借用模式检测 |
| **泛型分析** | `generic/` | 泛型类型实例化统计 |
| **Closure 分析** | `closure/` | 闭包捕获和生命周期分析 |
| **Memory Passport** | `memory_passport_tracker.rs` | FFI 边界所有权追踪 |
| **Unsafe 推断** | `unsafe_inference/` | 原始分配的启发式类型检测 |
| **生命周期** | `lifecycle/` | 所有权历史、borrow/clone 追踪 |
| **分类** | `classification/` | 带规则引擎的类型分类 |
| **估算** | `estimation/` | 分配大小估算 |
| **指标** | `metrics/` | 性能指标收集和报告 |
| **质量** | `quality/` | 代码质量分析和验证 |
| **安全** | `safety/` | 风险评估和违规追踪 |
| **安全违规** | `security/` | 安全违规分析 |
| **未知** | `unknown/` | 未知内存区域分析 |

---

## 检测器性能

| 检测器 | 复杂度 | 内存成本 | 典型运行时间 |
|--------|--------|----------|-------------|
| LeakDetector | O(n) | O(issues) | 10K 分配 < 1ms |
| UafDetector | O(n log n) | O(events) | 10K 分配 < 5ms |
| OverflowDetector | O(n) | O(issues) | 10K 分配 < 1ms |
| SafetyDetector | O(n) | O(issues) | 10K 分配 < 2ms |
| LifecycleDetector | O(n) | O(issues) | 10K 分配 < 1ms |

---

## 自定义检测器示例

```rust
use memscope_rs::analysis::detectors::{Detector, DetectionResult, Issue, IssueSeverity};
use memscope_rs::capture::types::AllocationInfo;

struct LargeAllocationDetector {
    threshold: usize,
}

impl Detector for LargeAllocationDetector {
    fn name(&self) -> &str {
        "LargeAllocationDetector"
    }

    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        let mut issues = Vec::new();

        for alloc in allocations {
            if alloc.size > self.threshold {
                issues.push(Issue {
                    description: format!("大分配: {} 字节, 位于 {:x}",
                        alloc.size, alloc.ptr),
                    ptr: alloc.ptr,
                    size: alloc.size,
                    severity: if alloc.size > self.threshold * 10 {
                        IssueSeverity::High
                    } else {
                        IssueSeverity::Medium
                    },
                    stack_trace: alloc.stack_trace.clone(),
                });
            }
        }

        DetectionResult {
            detector_name: self.name().to_string(),
            issues,
            severity: if issues.is_empty() {
                IssueSeverity::Low
            } else {
                IssueSeverity::Medium
            },
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        }
    }
}

// 使用
let memscope = MemScope::new();
memscope.register_detector(Box::new(LargeAllocationDetector { threshold: 1_000_000 }))?;
memscope.run_detectors()?;
```
