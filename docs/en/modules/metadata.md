# Metadata Module

## Overview

The metadata module provides centralized metadata management for the memscope-rs system. It manages all metadata including variables, scopes, threads, types, and pointers across the entire system, providing a unified interface for accessing and updating metadata.

## Components

### 1. MetadataEngine

**File**: `src/metadata/engine.rs`

**Purpose**: Centralized metadata management engine that coordinates all metadata components.

**Key Features**:
- Centralized: Single source of truth for all metadata
- Thread-safe: All operations are thread-safe via Arc
- Efficient: Optimized for fast lookups and updates
- Unified: Single interface for all metadata types

**Core Implementation**:

```rust
pub struct MetadataEngine {
    /// Variable registry
    pub variable_registry: Arc<VariableRegistry>,
    /// Scope tracker
    pub scope_tracker: Arc<ScopeTracker>,
    /// Thread registry
    pub thread_registry: Arc<ThreadRegistry>,
}

impl MetadataEngine {
    /// Create a new MetadataEngine
    pub fn new() -> Self {
        Self {
            variable_registry: Arc::new(VariableRegistry::new()),
            scope_tracker: Arc::new(ScopeTracker::new()),
            thread_registry: Arc::new(ThreadRegistry::new()),
        }
    }

    /// Get the variable registry
    pub fn variables(&self) -> &Arc<VariableRegistry> {
        &self.variable_registry
    }

    /// Get the scope tracker
    pub fn scopes(&self) -> &Arc<ScopeTracker> {
        &self.scope_tracker
    }

    /// Get the thread registry
    pub fn threads(&self) -> &Arc<ThreadRegistry> {
        &self.thread_registry
    }
}
```

### 2. VariableRegistry

**File**: `src/metadata/registry.rs`

**Purpose**: Manages variable metadata across the system.

**Key Features**:
- Variable tracking and lifecycle management
- Type information storage
- Relationship tracking
- Performance metrics

**Variable Metadata**:

```rust
pub struct VariableInfo {
    pub variable_id: VariableId,
    pub name: String,
    pub type_name: String,
    pub ptr: usize,
    pub size: usize,
    pub scope_id: ScopeId,
    pub thread_id: ThreadId,
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u64,
    pub is_active: bool,
}
```

### 3. ScopeTracker

**File**: `src/metadata/scope.rs`

**Purpose**: Tracks scope hierarchy and lifecycle for memory analysis.

**Key Features**:
- Scope hierarchy tracking
- Parent-child relationships
- Lifecycle metrics
- Automatic scope management

**Scope Metadata**:

```rust
pub struct ScopeInfo {
    pub scope_id: ScopeId,
    pub name: String,
    pub parent_scope: Option<ScopeId>,
    pub depth: u32,
    pub variables: Vec<VariableId>,
    pub total_memory: usize,
    pub peak_memory: usize,
    pub allocation_count: usize,
    pub lifetime_start: Option<u64>,
    pub lifetime_end: Option<u64>,
    pub is_active: bool,
}
```

### 4. ThreadRegistry

**File**: `src/metadata/thread.rs`

**Purpose**: Manages thread metadata and activity tracking.

**Key Features**:
- Thread registration and tracking
- Memory usage per thread
- Activity monitoring
- Thread relationships

**Thread Metadata**:

```rust
pub struct ThreadInfo {
    pub thread_id: ThreadId,
    pub thread_id_u64: u64,
    pub name: Option<String>,
    pub created_at: u64,
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub current_memory: usize,
    pub peak_memory: usize,
    pub is_active: bool,
}
```

## Design Principles

### 1. Centralized Management
All metadata is managed centrally:
- **Benefits**: Single source of truth, consistent state
- **Trade-off**: Centralized bottleneck potential

### 2. Thread Safety
All operations are thread-safe:
- **Benefits**: Safe for concurrent access
- **Trade-off**: Synchronization overhead

### 3. Efficient Lookups
Optimized for fast lookups:
- **Benefits**: High performance for queries
- **Trade-off**: Higher memory usage for indexes

### 4. Unified Interface
Single interface for all metadata types:
- **Benefits**: Consistent API, easier to use
- **Trade-off**: Less flexibility for specific use cases

## Usage Examples

### Basic Usage

```rust
use memscope::metadata::MetadataEngine;

// Create metadata engine
let engine = MetadataEngine::new();

// Access registries
let variable_registry = engine.variables();
let scope_tracker = engine.scopes();
let thread_registry = engine.threads();
```

### Variable Tracking

```rust
use memscope::metadata::registry::VariableRegistry;

let registry = VariableRegistry::new();

// Register a variable
let var_id = registry.register_variable(
    "my_variable",
    "Vec<u8>",
    0x1000,
    1024,
    scope_id,
    thread_id,
);

// Get variable info
if let Some(var_info) = registry.get_variable(var_id) {
    println!("Variable: {}, Size: {}", var_info.name, var_info.size);
}

// Update variable access
registry.record_access(var_id);
```

### Scope Tracking

```rust
use memscope::metadata::scope::ScopeTracker;

let tracker = ScopeTracker::new();

// Enter a scope
let scope_id = tracker.enter_scope("function_name".to_string()).unwrap();

// Exit a scope
tracker.exit_scope(scope_id).unwrap();

// Get scope hierarchy
let hierarchy = tracker.get_scope_hierarchy();
```

### Thread Tracking

```rust
use memscope::metadata::thread::ThreadRegistry;

let registry = ThreadRegistry::new();

// Register a thread
let thread_id = registry.register_thread(
    std::thread::current().id(),
    "main".to_string(),
);

// Update thread stats
registry.record_allocation(thread_id, 1024);
registry.record_deallocation(thread_id, 512);

// Get thread info
if let Some(thread_info) = registry.get_thread(thread_id) {
    println!("Thread: {}, Memory: {}", thread_info.name.unwrap_or_default(), thread_info.current_memory);
}
```

## Integration with Other Modules

```
Capture Engine
    ↓
Event Store (records events)
    ↓
Metadata Engine (manages metadata)
    ↓
Snapshot Engine (builds snapshots with metadata)
    ↓
Query Engine (queries with metadata context)
    ↓
Analysis Engine (analyzes with metadata)
```

## Performance Considerations

### Centralized Access
All metadata access goes through the engine:
- **Benefits**: Consistent state management
- **Trade-off**: Potential bottleneck for high concurrency

### Memory Usage
Metadata is stored in memory:
- **Benefits**: Fast access
- **Trade-off**: Memory grows with tracked items

### Synchronization
Thread-safe operations require synchronization:
- **Benefits**: Safe concurrent access
- **Trade-off**: Performance overhead

## Best Practices

1. **Metadata Registration**: Register variables and scopes as early as possible
2. **Cleanup**: Clean up inactive metadata periodically
3. **Thread Safety**: Use Arc<MetadataEngine> for shared access
4. **Error Handling**: Always handle registration errors

## Limitations

1. **Memory Growth**: Metadata grows until cleaned up
2. **Centralized Bottleneck**: All access goes through engine
3. **Synchronization Overhead**: Thread safety has performance cost
4. **Complexity**: Managing all metadata types can be complex

## Future Improvements

1. **Metadata Compression**: Compress stored metadata
2. **Lazy Loading**: Load metadata on demand
3. **Persistence**: Add disk persistence for metadata
4. **Distributed Support**: Support distributed metadata management
5. **Better Indexing**: Improve indexing for faster lookups