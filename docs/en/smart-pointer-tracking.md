# Smart Pointer Tracking and Circular Reference Detection

## Overview

Phase 1 focuses on smart pointer tracking and circular reference detection, providing powerful memory leak detection capabilities through automatic detection and simplified API.

## Completed Features

### P1.1 Clone Operation Detection (Macro Layer)

**Goal**: Detect clone operations through macros, recording clone source and target

**Implementation**:
- `track_clone!` macro to record clone operations
- Added `clone_source_ptr` and `clone_target_ptr` fields to `MemoryEvent`
- Maintain `clone_info_map` in `rebuild_allocations_from_events`

**Optimization**:
- Integrated into `track!` macro, no need for separate `track_clone!`
- Auto-detect smart pointer types in `rebuild_allocations_from_events`
- Auto-fill `smart_pointer_info` through type names (Arc, Rc, Box)

**Effects**:
- Automatic detection of smart pointer clone operations
- Simplified API, only need `track!` macro
- Data foundation for circular reference detection

### P1.2 Smart Pointer Opt-in Tracking

**Goal**: Integrate existing circular reference detection, using clone information to build reference graph

**Implementation**:
- Auto-detect smart pointer types in `rebuild_allocations_from_events`
- Fill `smart_pointer_info` field
- Integrate `detect_circular_references` function
- Add `circular_references` field to `DashboardContext`

**Data Structure**:
```rust
pub struct SmartPointerInfo {
    pub data_ptr: usize,
    pub pointer_type: SmartPointerType,
    pub is_data_owner: bool,
    pub ref_count_history: Vec<u64>,
    pub weak_count: Option<u64>,
    pub cloned_from: Option<usize>,
    pub clones: Vec<usize>,
    pub is_implicitly_deallocated: bool,
    pub is_weak_reference: false,
}
```

**Circular Reference Detection**:
- Build `ReferenceGraph` from allocation info
- Use DFS to detect cycles
- Calculate leaked memory and cycle statistics

**Effects**:
- Automatic detection of Arc/Rc circular references
- Provide cycle paths and leak estimates
- Visualized circular reference reports

## Integration Improvements

### Unified Data Source

**Goal**: Ensure all features use single data source (event_store) and single processing flow (rebuild_allocations_from_events)

**Implementation**:
- All data from `event_store.snapshot()`
- Unified processing through `rebuild_allocations_from_events`
- Return `capture::types::AllocationInfo` (includes `smart_pointer_info`)

**Data Flow**:
```
event_store (MemoryEvent)
  ↓
rebuild_allocations_from_events
  ↓
capture::types::AllocationInfo (with smart_pointer_info)
  ↓
Report building functions
  ↓
DashboardContext
  ↓
HTML Dashboard
```

### Automatic Lifecycle Tracking

**Goal**: No manual user operation, automatically record deallocation

**Implementation**:
- Auto-record deallocation events for all active allocations in `Tracker::drop()`
- Auto-calculate lifecycle (lifetime_ms)
- No need for user to manually call `drop()`

**Effects**:
- Fully automatic lifecycle tracking
- No extra user operations required
- Accurate lifecycle data

### API Simplification

**Goal**: Reduce number of macros, simplify usage

**Implementation**:
- Remove `track_clone!` macro dependency
- Auto-detect smart pointers in `rebuild_allocations_from_events`
- Only need `track!` macro

**Usage**:
```rust
// Before: need two macros
track!(tracker, data);
track_clone!(tracker, source, target);

// Now: only need one macro
track!(tracker, data); // Auto-detect smart pointers
```

## Usage Examples

### Basic Smart Pointer Tracking

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};
use std::rc::Rc;
use std::sync::Arc;

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // Smart pointer auto-detection
    let rc_data = Rc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, rc_data);

    let arc_data = Arc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, arc_data);

    // Automatic lifecycle tracking
    // No manual drop needed, auto-recorded on Tracker drop

    // Analysis results
    let mut az = analyzer(&tracker)?;
    let report = az.analyze();
    
    println!("Smart pointers detected: {}", 
        report.circular_references.total_smart_pointers);
    println!("Circular references: {}", 
        report.circular_references.count);
    
    Ok(())
}
```

### Circular Reference Detection

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};
use std::cell::RefCell;
use std::rc::Rc;

struct Node {
    data: i32,
    next: Option<Rc<RefCell<Node>>>,
}

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // Create circular reference
    let node1 = Rc::new(RefCell::new(Node { data: 1, next: None }));
    let node2 = Rc::new(RefCell::new(Node { data: 2, next: None }));

    node1.borrow_mut().next = Some(node2.clone());
    node2.borrow_mut().next = Some(node1.clone());

    track!(tracker, node1);
    track!(tracker, node2);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    if report.circular_references.has_cycles {
        println!("Detected {} circular references!", report.circular_references.count);
        println!("Total leaked memory: {} bytes", report.circular_references.total_leaked_memory);
    }

    Ok(())
}
```

### Mixed Type Tracking

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};
use std::rc::Rc;
use std::sync::Arc;
use std::boxed::Box;

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // Mix different smart pointer types
    let rc_data = Rc::new(vec![1, 2, 3]);
    track!(tracker, rc_data);

    let arc_data = Arc::new(vec![4, 5, 6]);
    track!(tracker, arc_data);

    let boxed_data = Box::new(42);
    track!(tracker, boxed_data);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    println!("Total smart pointers: {}", report.circular_references.total_smart_pointers);

    Ok(())
}
```

## Circular Reference Detection

### Detection Mechanism

1. **Build Reference Graph**:
   - Extract reference relationships from `smart_pointer_info`
   - Build adjacency list and reverse reference mapping
   - Skip weak references (don't create strong cycles)

2. **Detect Cycles**:
   - Use DFS to traverse reference graph
   - Identify cycles with length ≥ 2
   - Analyze cycle paths and leak estimates

3. **Generate Report**:
   - Number of circular references
   - Total leaked memory
   - Number of pointers in cycles
   - Statistics

### Report Fields

```rust
pub struct CircularReferenceReport {
    pub count: usize,                    // Number of circular references
    pub total_leaked_memory: usize,       // Total leaked memory
    pub pointers_in_cycles: usize,        // Pointers in cycles
    pub total_smart_pointers: usize,      // Total smart pointers
    pub has_cycles: bool,                // Whether cycles exist
}
```

## Performance Features

- **Auto-detection**: No need to manually mark smart pointers
- **Zero overhead**: Type detection in data processing stage
- **Unified processing**: Single data source and single processing flow
- **Simplified API**: Only need one `track!` macro

## Limitations

- Can only detect circular references in smart pointers (Arc, Rc, Box)
- Cannot detect circular references in plain references
- Cycle detection based on clone relationships, not real ownership transfer
- Real ownership transfer tracking requires Phase 3 (MIR extraction)
