# Analysis Engine & Detectors

> Pluggable analysis — five built-in detectors plus custom extension support

---

## Overview

The analysis system has two layers:

1. **AnalysisEngine** (`src/analysis_engine/`) — The orchestration layer that manages analyzers and detectors
2. **Detectors** (`src/analysis/detectors/`) — Individual analysis implementations

---

## The Detector Trait

**File:** `src/analysis/detectors/mod.rs`

```rust
pub trait Detector: Send + Sync {
    /// Human-readable name for this detector
    fn name(&self) -> &str;

    /// Run detection on a set of allocations
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult;
}
```

The trait is `Send + Sync` so detectors can be registered and run from any thread. The `detect()` method receives a slice of `AllocationInfo` and returns a `DetectionResult`.

---

## Detection Result

**File:** `src/analysis/detectors/types.rs`

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

## Built-in Detectors

### 1. LeakDetector

**File:** `src/analysis/detectors/leak_detector.rs`

**Algorithm:** Scans `active_allocations` for entries that have no matching deallocation. An allocation is considered "leaked" if it exists in the active set but was never freed.

```rust
// leak_detector.rs
pub struct LeakDetector {
    config: LeakDetectorConfig,
}

impl Detector for LeakDetector {
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        let mut issues = Vec::new();

        for alloc in allocations {
            // Check if this allocation has been alive for too long
            let age = now() - alloc.allocated_at_ns;
            if age > self.config.max_age_ns {
                issues.push(Issue {
                    description: format!("Allocation at {:x} alive for {}ms",
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

**Configuration:**

```rust
pub struct LeakDetectorConfig {
    pub max_age_ns: u64,           // Maximum age before flagging as leak
    pub min_size: usize,           // Minimum allocation size to check
    pub ignore_patterns: Vec<String>, // Type names to ignore
}
```

---

### 2. UafDetector (Use-After-Free)

**File:** `src/analysis/detectors/uaf_detector.rs`

**Algorithm:** Detects access patterns where a pointer is used after it has been freed. Tracks the timeline of allocations and deallocations, then checks for any access events that occur after a deallocation of the same pointer.

```rust
// uaf_detector.rs
pub struct UafDetector {
    config: UafDetectorConfig,
}

impl Detector for UafDetector {
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult {
        // Build a timeline of alloc/dealloc events
        // Check for access-after-free patterns
        // Return issues with severity based on confidence
    }
}
```

---

### 3. OverflowDetector

**File:** `src/analysis/detectors/overflow_detector.rs`

**Algorithm:** Checks if writes exceed allocation bounds. Looks for patterns where data is written beyond the allocated size, which could indicate buffer overflow vulnerabilities.

---

### 4. SafetyDetector

**File:** `src/analysis/detectors/safety_detector.rs`

**Algorithm:** General unsafe code safety violation detection. Checks for patterns like:
- Raw pointer dereferences without proper validation
- Unsafe block usage patterns that could lead to undefined behavior
- FFI call safety violations

---

### 5. LifecycleDetector

**File:** `src/analysis/detectors/lifecycle_detector.rs`

**Algorithm:** RAII/Drop pattern analysis. Checks if the `Drop` trait is properly called for allocations that should have deterministic cleanup. Identifies allocations that bypass RAII patterns.

---

## AnalysisEngine

**File:** `src/analysis_engine/engine.rs`

The AnalysisEngine orchestrates all analyzers and detectors:

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

### Detector Adapter

**File:** `src/analysis_engine/detector_adapter.rs`

Bridges the `Detector` trait to the `Analyzer` trait, allowing detectors to be used as analyzers:

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

## Running Detectors via MemScope Facade

```rust
use memscope_rs::facade::MemScope;

let memscope = MemScope::new();

// Run individual detectors
memscope.run_leak_detector()?;
memscope.run_uaf_detector()?;
memscope.run_overflow_detector()?;
memscope.run_safety_detector()?;
memscope.run_lifecycle_detector()?;

// Register a custom detector
memscope.register_detector(MyCustomDetector::new())?;

// Run all registered detectors
memscope.run_detectors()?;
```

---

## Additional Analysis Modules

Beyond the core detectors, the `src/analysis/` directory contains specialized analysis modules:

| Module | File | Purpose |
|--------|------|---------|
| **Async Analysis** | `async_analysis.rs` | Task memory profiling, efficiency scoring |
| **Borrow Analysis** | `borrow_analysis.rs` | Mutable/immutable borrow pattern detection |
| **Generic Analysis** | `generic/` | Generic type instantiation statistics |
| **Closure Analysis** | `closure/` | Closure capture and lifetime analysis |
| **Memory Passport** | `memory_passport_tracker.rs` | FFI boundary ownership tracking |
| **Unsafe Inference** | `unsafe_inference/` | Heuristic type detection for raw allocations |
| **Lifecycle** | `lifecycle/` | Ownership history, borrow/clone tracking |
| **Classification** | `classification/` | Type classification with rule engine |
| **Estimation** | `estimation/` | Size estimation for allocations |
| **Metrics** | `metrics/` | Performance metrics collection and reporting |
| **Quality** | `quality/` | Code quality analysis and validation |
| **Safety** | `safety/` | Risk assessment and violation tracking |
| **Security** | `security/` | Security violation analysis |
| **Unknown** | `unknown/` | Unknown memory region analysis |

---

## Detector Performance

| Detector | Complexity | Memory Cost | Typical Runtime |
|----------|------------|-------------|-----------------|
| LeakDetector | O(n) | O(issues) | < 1ms for 10K allocations |
| UafDetector | O(n log n) | O(events) | < 5ms for 10K allocations |
| OverflowDetector | O(n) | O(issues) | < 1ms for 10K allocations |
| SafetyDetector | O(n) | O(issues) | < 2ms for 10K allocations |
| LifecycleDetector | O(n) | O(issues) | < 1ms for 10K allocations |

---

## Custom Detector Example

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
                    description: format!("Large allocation: {} bytes at {:x}",
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

// Usage
let memscope = MemScope::new();
memscope.register_detector(Box::new(LargeAllocationDetector { threshold: 1_000_000 }))?;
memscope.run_detectors()?;
```
