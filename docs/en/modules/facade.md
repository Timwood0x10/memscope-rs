# Facade Module

## Overview

The facade module provides `MemScope`, a unified interface that integrates all engines into a single, easy-to-use API. It simplifies the user experience by hiding the complexity of coordinating multiple engines.

## MemScope Structure

**File**: `src/facade/facade.rs`

**Purpose**: Unified facade for all engines.

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

## Module Composition

```
┌─────────────────────────────────────────────────────────────┐
│                         MemScope                            │
│  (Facade - Unified interface for all engines)               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │ EventStore  │  │  Capture    │  │  Metadata   │       │
│  │ (Storage)   │←─┤  Engine     │  │  Engine     │       │
│  └─────────────┘  └─────────────┘  └─────────────┘       │
│         │               │               │                 │
│         │         ┌──────┴──────┐       │                 │
│         └────────→│  Snapshot   │←──────┘                 │
│                   │  Engine     │                          │
│                   └──────┬──────┘                          │
│                          │                                 │
│         ┌────────────────┼────────────────┐               │
│         │                │                │               │
│         ▼                ▼                ▼               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Query     │  │  Analysis   │  │  Timeline   │        │
│  │   Engine   │  │   Engine   │  │   Engine   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
│                          │                                 │
│                          ▼                                 │
│                   ┌─────────────┐                         │
│                   │   Render    │                         │
│                   │   Engine    │                         │
│                   └─────────────┘                         │
└─────────────────────────────────────────────────────────────┘
```

## Creation Methods

```rust
impl MemScope {
    /// Create with default settings (Unified backend)
    pub fn new() -> Self

    /// Create with specific backend
    pub fn with_backend(backend_type: CaptureBackendType) -> Self
}
```

## Usage Example

```rust
use memscope_rs::MemScope;

let memscope = MemScope::new();

// Capture events (typically done via track! macro)
// memscope.capture.capture_alloc(ptr, size, thread_id);

// Build and query snapshot
let snapshot = memscope.snapshot.build_snapshot();

// Query allocations
let allocs = memscope.query.query_allocations(&snapshot, ...);

// Analyze
let analysis_results = {
    let analysis = memscope.analysis.lock().unwrap();
    analysis.analyze()
};

// Export
memscope.render.export_json("output.json");
```

## Key Design Decisions

1. **All engines are Arc-wrapped**: Enables shared ownership
2. **Analysis uses Mutex**: Only one analysis at a time
3. **Snapshot is queryable**: Direct access to memory state
4. **Render connected to snapshot**: Decoupled rendering

## Limitations

1. **Interior mutability**: Analysis engine wrapped in Mutex
2. **No back-pressure**: Events queued without limits
3. **Single snapshot view**: No concurrent snapshot views
