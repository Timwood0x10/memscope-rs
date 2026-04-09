# Analysis Engine Module

## Overview

The analysis engine provides pluggable memory analysis capabilities. It coordinates multiple analyzers that examine memory snapshots to detect issues like leaks, fragmentation, and safety violations.

## Components

### 1. AnalysisEngine

**File**: `src/analysis_engine/engine.rs`

**Purpose**: Coordinates multiple analyzers and runs them on memory snapshots.

**Core Implementation**:

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

**File**: `src/analysis_engine/analyzer.rs`

**Purpose**: Defines the interface for all analyzers.

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

### 4. DetectorToAnalyzer Adapter

**File**: `src/analysis_engine/detector_adapter.rs`

**Purpose**: Adapts the legacy detector pattern to the analyzer interface.

## Design Philosophy

1. **Pluggable**: Analyzers are registered at runtime
2. **Composable**: Multiple analyzers run in sequence
3. **Thread-safe**: All traits are `Send + Sync`
4. **Extensible**: Easy to add new analyzers

## Usage Example

```rust
use memscope_rs::analysis_engine::{AnalysisEngine, Analyzer};
use memscope_rs::snapshot::engine::SnapshotEngine;

struct MyAnalyzer;

impl Analyzer for MyAnalyzer {
    fn name(&self) -> &str { "my_analyzer" }

    fn analyze(&self, snapshot: &MemorySnapshot) -> AnalysisResult {
        // Analyze the snapshot...
        AnalysisResult {
            analyzer_name: self.name().to_string(),
            issue_count: 0,
            severity: Severity::Info,
            description: "Analysis complete".to_string(),
            findings: vec![],
        }
    }
}

let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
let mut analysis_engine = AnalysisEngine::new(snapshot_engine);
analysis_engine.register_analyzer(Box::new(MyAnalyzer));

let results = analysis_engine.analyze();
```

## Integration with Other Modules

```
Capture Engine
    ↓
Event Store (stores events)
    ↓
Snapshot Engine (builds snapshots)
    ↓
Analysis Engine (runs analyzers)
    ↓
Render Engine (outputs results)
```

## Limitations

1. **Analysis is offline**: Works on snapshots, not real-time
2. **Memory overhead**: All analyzers held in memory
3. **No incremental analysis**: Full re-analysis each time
