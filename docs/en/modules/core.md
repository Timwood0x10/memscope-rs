# Core Module

## Overview

The core module provides fundamental memory tracking capabilities for the memscope-rs library. It implements the low-level infrastructure needed to intercept, track, and analyze memory allocations in Rust applications.

## Components

### 1. TrackingAllocator

**File**: `src/core/allocator.rs`

**Purpose**: Custom global allocator that intercepts all heap allocations and deallocations in the application.

**Key Features**:
- Implements `GlobalAlloc` trait to override system allocator
- Tracks every heap allocation and deallocation
- Prevents recursive tracking using thread-local storage flags
- Provides type inference based on allocation size
- Panic-safe operation

**Source Code**:

```rust
pub struct TrackingAllocator;

thread_local! {
    static TRACKING_DISABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

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

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());

        if should_track {
            TRACKING_DISABLED.with(|disabled| disabled.set(true));

            if let Ok(tracker) = std::panic::catch_unwind(crate::core::tracker::get_tracker) {
                let _ = tracker.track_deallocation(ptr as usize);
            }

            TRACKING_DISABLED.with(|disabled| disabled.set(false));
        }

        System.dealloc(ptr, layout);
    }
}
```

**Design Philosophy**:

1. **Zero-overhead tracking**: Uses thread-local storage flags instead of locks to minimize performance impact
2. **Recursive protection**: Disables tracking during tracking operations to prevent infinite loops
3. **Panic resilience**: Tracking failures don't crash the application
4. **Type inference**: Provides basic type information based on allocation size patterns

**Type Inference Implementation**:

```rust
fn _infer_type_from_allocation_context(size: usize) -> &'static str {
    match size {
        // Common Rust type sizes
        1 => "u8",
        2 => "u16",
        4 => "u32",
        8 => "u64",
        16 => "u128",

        // String and Vec common sizes
        24 => "String",
        32 => "Vec<T>",
        48 => "HashMap<K,V>",

        // Smart pointer sizes
        size if size == std::mem::size_of::<std::sync::Arc<String>>() => "Arc<T>",
        size if size == std::mem::size_of::<std::rc::Rc<String>>() => "Rc<T>",
        size if size == std::mem::size_of::<Box<String>>() => "Box<T>",

        // Default for other sizes
        _ => "unknown",
    }
}
```

**Usage**:

```rust
// In main.rs or lib.rs
use memscope::core::TrackingAllocator;

#[global_allocator]
static GLOBAL_ALLOCATOR: TrackingAllocator = TrackingAllocator;

fn main() {
    // All heap allocations are now automatically tracked
    let data = vec![1, 2, 3, 4, 5];
    let string = String::from("Hello");
}
```

### 2. Error Handling

**File**: `src/core/error.rs`

**Purpose**: Unified error handling system for the entire memscope-rs library.

**Key Features**:
- Simplified, efficient error types using `Arc<str>` to reduce string cloning
- Error categorization (Memory, Analysis, Export, Configuration, System, Internal)
- Error severity levels (Low, Medium, High, Critical)
- Error recovery mechanisms
- Backward compatibility with old error types

**Error Types**:

```rust
pub enum MemScopeError {
    Memory {
        operation: MemoryOperation,
        message: Arc<str>,
        context: Option<Arc<str>>,
    },

    Analysis {
        analyzer: Arc<str>,
        message: Arc<str>,
        recoverable: bool,
    },

    Export {
        format: Arc<str>,
        message: Arc<str>,
        partial_success: bool,
    },

    Configuration {
        component: Arc<str>,
        message: Arc<str>,
    },

    System {
        error_type: SystemErrorType,
        message: Arc<str>,
        source_message: Option<Arc<str>>,
    },

    Internal {
        message: Arc<str>,
        location: Option<Arc<str>>,
    },
}
```

**Error Severity**:

```rust
pub enum ErrorSeverity {
    Low,      // Warnings, partial failures
    Medium,   // Operational failures
    High,     // Critical analysis failures
    Critical, // Internal errors, bugs
}
```

**Error Recovery**:

```rust
pub enum RecoveryAction {
    Retry { max_attempts: u32, delay_ms: u64 },
    UseDefault { value: String },
    Skip,
    Abort,
    Fallback { strategy: String },
}

pub trait ErrorRecovery {
    fn can_recover(&self, error: &MemScopeError) -> bool;
    fn get_recovery_action(&self, error: &MemScopeError) -> Option<RecoveryAction>;
    fn execute_recovery(&self, action: &RecoveryAction) -> Result<()>;
}
```

**Design Philosophy**:

1. **Performance**: Uses `Arc<str>` instead of `String` to reduce cloning overhead
2. **Categorization**: Clear separation of error types for better handling
3. **Recoverability**: Marks errors as recoverable or not for appropriate response
4. **Backward compatibility**: Converts old error types to new format automatically

**Usage**:

```rust
use memscope::core::error::{MemScopeError, MemoryOperation};

fn allocate_memory(size: usize) -> Result<*mut u8, MemScopeError> {
    if size == 0 {
        return Err(MemScopeError::memory_with_context(
            MemoryOperation::Allocation,
            "zero-sized allocation",
            "in allocate_memory",
        ));
    }

    // ... allocation logic
    Ok(ptr)
}
```

### 3. Scope Tracker

**File**: `src/core/scope_tracker.rs`

**Purpose**: Tracks variable lifetimes and scope hierarchy for memory analysis.

**Key Features**:
- Per-thread scope stack tracking
- Scope hierarchy and relationship tracking
- Variable association with scopes
- Scope lifecycle metrics
- Automatic scope management using RAII

**Core Implementation**:

```rust
pub struct ScopeTracker {
    pub active_scopes: RwLock<HashMap<ScopeId, ScopeInfo>>,
    pub completed_scopes: Mutex<Vec<ScopeInfo>>,
    pub scope_hierarchy: Mutex<ScopeHierarchy>,
    next_scope_id: AtomicU64,
    pub scope_stack: RwLock<HashMap<String, Vec<ScopeId>>>,
}

impl ScopeTracker {
    pub fn enter_scope(&self, name: String) -> TrackingResult<ScopeId> {
        let scope_id = self.allocate_scope_id();
        let thread_id = format!("{:?}", std::thread::current().id());
        let timestamp = current_timestamp();

        // Determine parent scope and depth
        let (parent_scope, depth) = {
            let stack = self.scope_stack.read().unwrap();
            if let Some(thread_stack) = stack.get(&thread_id) {
                if let Some(&parent_id) = thread_stack.last() {
                    let active = self.active_scopes.read().unwrap();
                    if let Some(parent) = active.get(&parent_id) {
                        (Some(parent.name.clone()), parent.depth + 1)
                    } else {
                        (None, 0)
                    }
                } else {
                    (None, 0)
                }
            } else {
                (None, 0)
            }
        };

        // Create and register scope info
        let scope_info = ScopeInfo {
            name: name.clone(),
            parent: parent_scope.clone(),
            children: Vec::new(),
            depth,
            variables: Vec::new(),
            total_memory: 0,
            peak_memory: 0,
            allocation_count: 0,
            lifetime_start: Some(timestamp as u64),
            lifetime_end: None,
            is_active: true,
            start_time: timestamp as u64,
            end_time: None,
            memory_usage: 0,
            child_scopes: Vec::new(),
            parent_scope: parent_scope.clone(),
        };

        self.active_scopes.write().unwrap().insert(scope_id, scope_info);
        self.scope_stack.write().unwrap()
            .entry(thread_id.clone())
            .or_default()
            .push(scope_id);

        Ok(scope_id)
    }
}
```

**RAII Scope Guard**:

```rust
pub struct ScopeGuard {
    scope_id: ScopeId,
    tracker: Arc<ScopeTracker>,
}

impl ScopeGuard {
    pub fn enter(name: &str) -> TrackingResult<Self> {
        let tracker = get_global_scope_tracker();
        let scope_id = tracker.enter_scope(name.to_string())?;
        Ok(Self { scope_id, tracker })
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        let _ = self.tracker.exit_scope(self.scope_id);
    }
}
```

**Design Philosophy**:

1. **Thread-local tracking**: Each thread maintains its own scope stack
2. **Hierarchy awareness**: Tracks parent-child relationships between scopes
3. **Automatic cleanup**: Uses RAII pattern for automatic scope exit
4. **Performance**: Uses atomic operations for scope ID allocation

**Usage**:

```rust
use memscope::core::scope_tracker::ScopeGuard;

fn process_data() {
    // Enter scope with automatic cleanup
    let _guard = ScopeGuard::enter("process_data").unwrap();

    // Variables are automatically associated with this scope
    let data = vec![1, 2, 3, 4, 5];

    // Nested scope
    {
        let _nested_guard = ScopeGuard::enter("inner_scope").unwrap();
        let temp = String::from("temporary");
    } // Inner scope automatically exited

} // Outer scope automatically exited
```

**Macro Usage**:

```rust
// Simple scope tracking
track_scope!("function_name");

// Scope with code block
track_scope!("block_name", {
    let data = vec![1, 2, 3];
    // ... processing
});
```

## Design Principles

### 1. Zero Overhead
The core module is designed to have minimal performance impact:
- Uses thread-local storage instead of locks
- Avoids allocations during tracking operations
- Uses static strings for type inference

### 2. Panic Safety
All operations are panic-safe:
- Tracking failures don't crash the application
- Uses `catch_unwind` to handle panics in tracking code
- Graceful degradation when tracking fails

### 3. Thread Safety
All shared state is properly synchronized:
- Uses `RwLock` for read-heavy data structures
- Uses `Mutex` for write-heavy data structures
- Uses `AtomicU64` for counters

### 4. Type Safety
Strong type system ensures correctness:
- Custom error types prevent invalid operations
- Result types force error handling
- Type inference provides additional context

## Performance Considerations

### Thread-Local Storage
Using thread-local storage flags instead of locks:
- **Advantage**: No lock contention between threads
- **Trade-off**: Each thread has its own tracking state

### Static Strings
Using static strings for type inference:
- **Advantage**: No allocation during tracking
- **Trade-off**: Limited type information

### Atomic Operations
Using atomic operations for counters:
- **Advantage**: Lock-free performance
- **Trade-off**: Limited to simple operations

## Testing

The core module includes comprehensive tests:

```rust
#[test]
fn test_allocation_tracking() {
    let allocator = TrackingAllocator::new();
    let layout = Layout::from_size_align(1024, 8).unwrap();

    unsafe {
        let ptr = allocator.alloc(layout);
        assert!(!ptr.is_null());
        allocator.dealloc(ptr, layout);
    }
}

#[test]
fn test_recursive_allocation_handling() {
    // Verifies that recursive allocations don't cause infinite loops
    let allocator = TrackingAllocator::new();
    let layout = Layout::from_size_align(64, 8).unwrap();

    unsafe {
        let ptr = allocator.alloc(layout);
        if !ptr.is_null() {
            allocator.dealloc(ptr, layout);
        }
    }
}
```

## Integration

The core module integrates with the rest of the memscope-rs library:

```
core/
  ↓
tracker/      (Uses TrackingAllocator)
  ↓
capture/      (Tracks allocation events)
  ↓
analysis/     (Analyzes scope data)
  ↓
render/       (Visualizes scope hierarchy)
```

## Best Practices

1. **Global Allocator**: Set the tracking allocator as the global allocator once at program startup
2. **Error Handling**: Always handle `MemScopeError` appropriately
3. **Scope Management**: Use `ScopeGuard` for automatic cleanup
4. **Performance**: Use sampling to reduce overhead in production

## Limitations

1. **Type Inference**: Limited to common type sizes
2. **Stack Variables**: Only heap allocations are tracked
3. **Static Variables**: Static variables are not tracked
4. **External Memory**: Memory allocated by external libraries is not tracked unless they use the global allocator

## Future Improvements

1. **Better Type Inference**: Integrate with compiler for accurate type information
2. **Stack Tracking**: Track stack allocations
3. **Variable Names**: Capture actual variable names (currently uses inferred names)
4. **Performance**: Further optimize tracking overhead