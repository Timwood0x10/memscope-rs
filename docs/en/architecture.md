# Architecture

## Overview

memscope-rs is a comprehensive memory tracking and analysis library for Rust applications. It provides a modular architecture that separates concerns across multiple layers, enabling efficient memory monitoring, analysis, and visualization.

## System Architecture

```
User Code
   ↓
Facade Layer (facade/)
   ↓
Capture Engine (capture/)
   ↓
Event Storage (event_store/)
   ↓
Metadata Engine (metadata/)
   ↓
Snapshot Engine (snapshot/)
   ↓
Query Engine (query/)
   ↓
Analysis Engine (analysis/)
   ↓
Render Engine (render_engine/)
   ↓
Output (JSON/HTML/Binary)
```

## Module Responsibilities

### Core Layer (core/)

**Purpose**: Provides fundamental memory tracking capabilities and type definitions.

**Components**:
- `allocator.rs`: Custom global allocator that intercepts all heap allocations
- `error.rs`: Error type definitions
- `scope_tracker.rs`: Scope tracking for variable lifetimes
- `safe_operations.rs`: Safe operation utilities
- `call_stack_normalizer.rs`: Call stack normalization
- `unwrap_safe.rs`: Safe unwrap utilities

**Key Implementation**:

```rust
// src/core/allocator.rs
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());
            if should_track {
                TRACKING_DISABLED.with(|disabled| disabled.set(true));
                if let Ok(tracker) = std::panic::catch_unwind(crate::core::tracker::get_tracker) {
                    let _ = tracker.track_allocation(ptr as usize, layout.size());
                }
                TRACKING_DISABLED.with(|disabled| disabled.set(false));
            }
        }
        ptr
    }
}
```

**Design Principles**:
- **Zero-overhead**: Uses TLS flags instead of locks
- **Recursive protection**: Prevents infinite tracking loops
- **Panic resilience**: Tracking failures don't crash the application
- **Type inference**: Tracks types based on allocation size

### Tracker Layer (tracker.rs)

**Purpose**: Provides a unified tracking API and macros for users.

**Key Features**:
- Automatic variable name and type capture
- System monitoring (CPU, memory, disk, network)
- Per-thread independent tracking
- Configurable sampling rate
- Automatic hotspot detection
- JSON/HTML export support

**Macro Usage**:

```rust
let tracker = tracker!();

let data = vec![1, 2, 3, 4, 5];
track!(tracker, data);
```

**Sampling Support**:

```rust
pub fn track_as<T: Trackable>(&self, var: &T, name: &str, file: &str, line: u32) {
    if let Ok(cfg) = self.config.lock() {
        if cfg.sampling.sample_rate < 1.0 {
            let hash = compute_hash(timestamp, thread_id, name, file, line);
            let threshold = (cfg.sampling.sample_rate * 1000.0) as u64;
            if (hash % 1000) > threshold {
                return;
            }
        }
    }
    self.track_inner(var, name, file, line);
}
```

### Capture Engine (capture/)

**Purpose**: Captures memory events from the application using various backend strategies.

**Backend Types**:
- `CoreTracker`: Single-threaded, simple, low overhead
- `LockfreeTracker`: Multi-threaded, thread-local storage
- `AsyncTracker`: Async task tracking with task IDs
- `UnifiedTracker`: Automatic detection based on CPU core count

**Key Types**:

```rust
pub enum CaptureBackendType {
    Core,
    Lockfree,
    Async,
    Unified,
}

pub struct AllocationInfo {
    pub ptr: usize,
    pub size: usize,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub timestamp_alloc: u64,
    pub timestamp_dealloc: Option<u64>,
    pub thread_id: ThreadId,
    pub thread_id_u64: u64,
}
```

**Analysis Features**:
- Hotspot analysis
- Bottleneck analysis
- Task-level memory profiling
- Efficiency scoring
- Resource ranking

### Event Storage (event_store/)

**Purpose**: Centralized storage for all memory events using lock-free queues.

**Key Implementation**:

```rust
pub struct EventStore {
    // Lock-free queue for events
}

impl EventStore {
    pub fn record(&self, event: MemoryEvent) {
        // Record event to lock-free queue
    }

    pub fn snapshot(&self) -> Vec<MemoryEvent> {
        // Get event snapshot
    }
}

pub enum MemoryEventType {
    Allocate,
    Deallocate,
    Reallocate,
    Move,
    Borrow,
    Clone,
}
```

**Design Principles**:
- **Lock-free**: High-performance concurrent access
- **Append-only**: Efficient event recording
- **Snapshot support**: Point-in-time state capture

### Metadata Engine (metadata/)

**Purpose**: Centralized management of all metadata including variables, scopes, and threads.

**Components**:
- `VariableRegistry`: Variable metadata management
- `ScopeTracker`: Scope tracking for variable lifetimes
- `ThreadRegistry`: Thread metadata management
- `SmartPointers`: Smart pointer information
- `StackTrace`: Call stack information

**Key Implementation**:

```rust
pub struct MetadataEngine {
    pub variable_registry: Arc<VariableRegistry>,
    pub scope_tracker: Arc<ScopeTracker>,
    pub thread_registry: Arc<ThreadRegistry>,
}

impl MetadataEngine {
    pub fn new() -> Self {
        Self {
            variable_registry: Arc::new(VariableRegistry::new()),
            scope_tracker: Arc::new(ScopeTracker::new()),
            thread_registry: Arc::new(ThreadRegistry::new()),
        }
    }
}
```

### Snapshot Engine (snapshot/)

**Purpose**: Builds memory snapshots from event data.

**Key Implementation**:

```rust
pub struct SnapshotEngine {
    event_store: SharedEventStore,
}

impl SnapshotEngine {
    pub fn build_snapshot(&self) -> MemorySnapshot {
        // Build snapshot from events
    }
}

pub struct MemorySnapshot {
    pub allocations: Vec<ActiveAllocation>,
    pub stats: MemoryStats,
    pub timestamp: u64,
}

pub struct ActiveAllocation {
    pub ptr: usize,
    pub size: usize,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub allocated_at: u64,
    pub thread_id: ThreadId,
    pub thread_id_u64: u64,
}
```

### Query Engine (query/)

**Purpose**: Provides unified query interface for accessing memory data.

**Query Types**:

```rust
pub enum Query {
    Allocation(AllocationQuery),
    Thread(ThreadQuery),
    Summary(SummaryQuery),
}

pub enum QueryResult {
    Allocation(AllocationQueryResult),
    Thread(ThreadQueryResult),
    Summary(SummaryQueryResult),
}
```

### Analysis Engine (analysis/)

**Purpose**: Analyzes memory data to detect issues and provide insights.

**Detectors**:
- `LeakDetector`: Memory leak detection
- `UafDetector`: Use-after-free detection
- `OverflowDetector`: Buffer overflow detection
- `SafetyDetector`: Safety violation detection
- `LifecycleDetector`: Lifecycle analysis

**Detector Interface**:

```rust
pub trait Detector {
    fn name(&self) -> &str;
    fn detect(&self, allocations: &[AllocationInfo]) -> DetectionResult;
    fn get_config(&self) -> &DetectorConfig;
}

pub struct DetectionResult {
    pub issues: Vec<Issue>,
    pub statistics: DetectionStatistics,
}
```

**Additional Analysis**:
- Circular reference detection
- Unsafe FFI tracking
- Async pattern analysis
- Borrow pattern analysis
- Generic type analysis
- Closure analysis

### Timeline Engine (timeline/)

**Purpose**: Time-based memory analysis and replay.

**Components**:
- `TimelineIndex`: Time-based indexing
- `TimelineQuery`: Time-based queries
- `TimelineReplay`: Event replay functionality

### Render Engine (render_engine/)

**Purpose**: Renders output data in various formats.

**Output Formats**:
- JSON
- HTML (interactive dashboard)
- Binary
- SVG

**Export Functions**:

```rust
pub fn export_all_json(output_path: &str, tracker: &Tracker, ...) -> Result<()>
pub fn export_dashboard_html(output_path: &str, tracker: &Tracker, ...) -> Result<()>
pub fn export_snapshot_to_json(snapshot: &MemorySnapshot, path: &Path, options: &ExportJsonOptions) -> Result<()>
pub fn export_leak_detection_json(detection_result: &DetectionResult, path: &Path) -> Result<()>
pub fn export_memory_passports_json(passports: &[MemoryPassport], path: &Path) -> Result<()>
pub fn export_unsafe_ffi_json(stats: &UnsafeFFIStats, path: &Path) -> Result<()>
```

### Facade Layer (facade/)

**Purpose**: Provides a unified facade interface that integrates all engines.

**Key Implementation**:

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

impl MemScope {
    pub fn new() -> Self {
        // Create all engines and connect them
        let event_store = Arc::new(EventStore::new());
        let capture = Arc::new(CaptureEngine::new(CaptureBackendType::Unified, event_store.clone()));
        // ... create other engines

        Self {
            event_store,
            capture,
            metadata,
            snapshot,
            query,
            analysis,
            timeline,
            render,
        }
    }

    pub fn run_leak_detector(&self) -> DetectionResult {
        self.analysis.lock().unwrap().run_detector(LeakDetector::new(...))
    }

    pub fn export_html(&self, path: &str) -> Result<()> {
        self.render.export_html(path, ...)
    }
}
```

### Error Handling (error/)

**Purpose**: Unified error handling with context and recovery strategies.

**Error Types**:

```rust
pub struct MemScopeError {
    pub kind: ErrorKind,
    pub severity: ErrorSeverity,
    pub context: ErrorContext,
    pub source: Option<Box<dyn std::error::Error>>,
}

pub enum ErrorKind {
    AllocationError,
    TrackingError,
    AnalysisError,
    RenderError,
    ExportError,
}

pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
}

pub enum RecoveryAction {
    Retry,
    Skip,
    Abort,
    Recover,
}
```

### Derive Macro (memscope-derive/)

**Purpose**: Provides procedural macro for automatically implementing the `Trackable` trait.

**Key Implementation**:

```rust
#[proc_macro_derive(Trackable)]
pub fn derive_trackable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        Data::Struct(data_struct) => {
            let heap_ptr_impl = generate_heap_ptr_impl(&data_struct.fields);
            let size_estimate_impl = generate_size_estimate_impl(&data_struct.fields);
            let internal_allocations_impl = generate_internal_allocations_impl(&data_struct.fields);

            quote! {
                impl Trackable for #name {
                    fn get_heap_ptr(&self) -> Option<usize> { #heap_ptr_impl }
                    fn get_type_name(&self) -> &'static str { stringify!(#name) }
                    fn get_size_estimate(&self) -> usize { #size_estimate_impl }
                    fn get_internal_allocations(&self, var_name: &str) -> Vec<(usize, String)> {
                        #internal_allocations_impl
                    }
                }
            }
        }
        Data::Enum(data_enum) => { /* ... */ }
        Data::Union(_) => { /* Not supported */ }
    }
}
```

### Trackable Trait

**Purpose**: Marks types that can be tracked by the memory tracker.

**Trait Definition**:

```rust
pub trait Trackable {
    fn get_heap_ptr(&self) -> Option<usize>;
    fn get_type_name(&self) -> &'static str;
    fn get_size_estimate(&self) -> usize;
    fn get_ref_count(&self) -> Option<usize> { None }
    fn get_data_ptr(&self) -> Option<usize>;
    fn get_data_size(&self) -> Option<usize>;
}
```

**Built-in Implementations**:
- `Vec<T>`
- `String`
- `HashMap<K, V>`
- `Box<T>`
- `Arc<T>`
- `Rc<T>`

## Data Flow

### Tracking Flow

```
1. User code uses track!() macro
   ↓
2. Tracker captures variable information
   ↓
3. CaptureEngine records allocation events
   ↓
4. EventStore stores events in lock-free queue
   ↓
5. MetadataEngine manages metadata
   ↓
6. SnapshotEngine builds snapshots
```

### Analysis Flow

```
1. SnapshotEngine provides memory snapshot
   ↓
2. QueryEngine queries specific data
   ↓
3. AnalysisEngine runs detectors
   ↓
4. Detection results generated
   ↓
5. RenderEngine renders output
   ↓
6. Export to JSON/HTML/Binary
```

### Export Flow

```
1. User calls export function
   ↓
2. RenderEngine retrieves data from SnapshotEngine
   ↓
3. Data formatted according to output format
   ↓
4. Write to file system
```

## Design Patterns

### Facade Pattern
`MemScope` provides a unified interface to all engines, simplifying user interaction.

### Strategy Pattern
`CaptureBackend` supports multiple tracking strategies (Core, Lockfree, Async, Unified).

### Observer Pattern
Event storage and event recording use observer pattern for event handling.

### Factory Pattern
Backend creation and configuration use factory pattern.

### Adapter Pattern
Detectors are adapted to work with the analysis engine.

### Builder Pattern
Configuration objects use builder pattern for flexible construction.

### Singleton Pattern
Global tracker uses singleton pattern for shared state.

## Architecture Principles

### 1. Layered Architecture
Clear separation of concerns across multiple layers, each with a specific responsibility.

### 2. Modular Design
Each module has a single, well-defined responsibility, making the codebase easy to understand and maintain.

### 3. Type Safety
Strong type system ensures memory safety and prevents many common errors.

### 4. Thread Safety
Uses `Arc` and `Mutex` for shared state, ensuring thread-safe concurrent access.

### 5. Zero-Overhead Tracking
Uses TLS flags instead of locks to minimize performance impact.

### 6. Async Support
Designed with async tasks in mind, supporting task ID tracking.

### 7. Extensibility
Easy to add new detectors, backends, and export formats.

## Performance Considerations

### Lock-Free Queues
Event storage uses lock-free queues for high-performance concurrent access.

### Sampling
Supports configurable sampling rates to reduce overhead in production environments.

### TLS Flags
Uses thread-local storage flags instead of locks to prevent recursive tracking.

### Panic Resilience
Tracking failures are handled gracefully without crashing the application.

### Bounded History
Event storage supports bounded history to limit memory usage.

## Security Considerations

### Panic Safety
All operations are panic-safe, ensuring tracking failures don't crash the application.

### Type Safety
Strong type system prevents memory safety violations.

### Thread Safety
All shared state is properly synchronized using `Arc` and `Mutex`.

### Recursive Protection
Tracking is disabled during tracking operations to prevent infinite loops.

## Dependencies

### External Dependencies
- `serde`: Serialization/deserialization
- `serde_json`: JSON support
- `std`: Standard library

### Internal Dependencies
- `memscope-derive`: Procedural macro for `Trackable` trait

## Conclusion

memscope-rs provides a comprehensive, modular architecture for memory tracking and analysis in Rust applications. The layered design ensures clear separation of concerns, while the modular approach allows for easy extension and maintenance. The architecture prioritizes performance, safety, and usability, making it suitable for both development and production environments.