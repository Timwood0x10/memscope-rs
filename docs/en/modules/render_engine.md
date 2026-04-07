# Render Engine Module

## Overview

The render engine is responsible for outputting memory data in various formats (JSON, HTML, Binary). It provides a pluggable renderer system that can be extended with custom formats.

## Components

### 1. RenderEngine

**File**: `src/render_engine/engine.rs`

**Purpose**: Coordinates multiple renderers and provides unified rendering interface.

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

**File**: `src/render_engine/renderer.rs`

```rust
pub trait Renderer: Send + Sync {
    fn format(&self) -> OutputFormat;
    fn render(&self, snapshot: &MemorySnapshot, config: &RenderConfig) -> Result<RenderResult, String>;
}
```

### 3. Output Formats

```rust
pub enum OutputFormat {
    Json,
    Html,
    Binary,
    Svg,
}
```

### 4. Export Functions

**File**: `src/render_engine/export.rs`

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

### 5. Dashboard Renderer

**File**: `src/render_engine/dashboard/renderer.rs`

**Purpose**: Generates interactive HTML dashboard with JavaScript charts.

## Architecture

```
RenderEngine
    │
    ├── JsonRenderer ───→ JSON files
    ├── HtmlRenderer ──→ HTML dashboard
    ├── BinaryRenderer ─→ Compact binary format
    └── SvgRenderer ────→ SVG charts
```

## Design Decisions

1. **Pluggable renderers**: Easy to add new formats
2. **Snapshot-based**: Renders from memory snapshots
3. **Lazy rendering**: Rendering happens on demand

## Usage Example

```rust
use memscope_rs::render_engine::{RenderEngine, ExportJsonOptions};

let engine = RenderEngine::new(snapshot_engine);

// Export to JSON
let options = ExportJsonOptions { verbose: true };
engine.export_snapshot_to_json(&snapshot, Path::new("output.json"), &options);

// Export to HTML dashboard
engine.export_dashboard_html("dashboard.html", &tracker);
```

## Limitations

1. **Full snapshot in memory**: Rendering requires entire snapshot
2. **No streaming**: Cannot render incrementally
3. **Single output per render**: No batch export
