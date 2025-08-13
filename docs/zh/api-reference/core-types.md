# Core Types Reference

Core data structures and types used in memscope-rs for memory tracking and analysis.

## Overview

This document describes the fundamental types that form the foundation of memscope-rs. These types are defined in `src/core/types/mod.rs` and are used throughout the library for tracking memory allocations, storing analysis results, and providing error handling.

## Error Types

### TrackingError

The main error type for all tracking operations.

**Module:** `memscope_rs::core::types`

**Source:** `src/core/types/mod.rs`

Comprehensive error type that covers all possible failure modes in the tracking system.

#### Variants

- **AllocationFailed(String)** - Memory allocation operation failed
- **DeallocationFailed(String)** - Memory deallocation operation failed  
- **TrackingDisabled** - Memory tracking is currently disabled
- **InvalidPointer(String)** - The provided pointer is invalid or null
- **SerializationError(String)** - Error occurred during data serialization
- **VisualizationError(String)** - Error occurred during visualization generation
- **ThreadSafetyError(String)** - Thread safety violation detected
- **ConfigurationError(String)** - Configuration parameter is invalid
- **AnalysisError(String)** - Error occurred during memory analysis
- **ExportError(String)** - Error occurred during data export
- **MemoryCorruption(String)** - Memory corruption detected
- **UnsafeOperationDetected(String)** - Unsafe operation detected and flagged
- **FFIError(String)** - Foreign Function Interface error
- **ScopeError(String)** - Scope management error
- **BorrowCheckError(String)** - Borrow checker violation detected
- **LifetimeError(String)** - Lifetime management error
- **TypeInferenceError(String)** - Type inference failed
- **PerformanceError(String)** - Performance threshold exceeded
- **ResourceExhausted(String)** - System resources exhausted
- **InternalError(String)** - Internal system error
- **IoError(String)** - Input/output operation failed
- **LockError(String)** - Lock acquisition failed
- **ChannelError(String)** - Channel communication error
- **ThreadError(String)** - Thread operation error
- **InitializationError(String)** - Initialization error
- **NotImplemented(String)** - Feature not implemented
- **InvalidOperation(String)** - Invalid operation
- **ValidationError(String)** - Validation error

#### Example Usage

```rust
use memscope_rs::TrackingError;

fn handle_tracking_error(result: Result<(), TrackingError>) {
    match result {
        Ok(()) => println!("Operation successful"),
        Err(TrackingError::AllocationFailed(msg)) => {
            eprintln!("Allocation failed: {}", msg);
        }
        Err(TrackingError::TrackingDisabled) => {
            eprintln!("Memory tracking is disabled");
        }
        Err(e) => eprintln!("Other error: {}", e),
    }
}
```

### TrackingResult<T>

Type alias for `Result<T, TrackingError>`.

**Module:** `memscope_rs::core::types`

```rust
pub type TrackingResult<T> = Result<T, TrackingError>;
```

## Memory Allocation Types

### AllocationInfo

Information about a memory allocation.

**Module:** `memscope_rs::core::types`

**Source:** `src/core/types/mod.rs`

Contains comprehensive information about a single memory allocation, including metadata, timing, and analysis results.

#### Fields

- **ptr**: `usize` (pub) - Memory address of the allocation
- **size**: `usize` (pub) - Size of the allocation in bytes
- **var_name**: `Option<String>` (pub) - Optional variable name associated with this allocation
- **type_name**: `Option<String>` (pub) - Optional type name of the allocated data
- **scope_name**: `Option<String>` (pub) - Optional scope name where the allocation occurred
- **timestamp_alloc**: `u64` (pub) - Timestamp when the allocation was made
- **timestamp_dealloc**: `Option<u64>` (pub) - Optional timestamp when the allocation was deallocated
- **thread_id**: `String` (private) - Thread ID where the allocation occurred
- **borrow_count**: `usize` (pub) - Number of active borrows for this allocation
- **stack_trace**: `Option<Vec<String>>` (pub) - Optional stack trace at the time of allocation
- **is_leaked**: `bool` (pub) - Whether this allocation is considered leaked
- **lifetime_ms**: `Option<u64>` (pub) - Precise lifetime in milliseconds
- **smart_pointer_info**: `Option<SmartPointerInfo>` (pub) - Smart pointer specific information
- **memory_layout**: `Option<MemoryLayoutInfo>` (pub) - Detailed memory layout information
- **generic_info**: `Option<GenericTypeInfo>` (pub) - Generic type information
- **dynamic_type_info**: `Option<DynamicTypeInfo>` (pub) - Dynamic type information (trait objects)
- **runtime_state**: `Option<RuntimeStateInfo>` (pub) - Runtime state information
- **stack_allocation**: `Option<StackAllocationInfo>` (pub) - Stack allocation information
- **temporary_object**: `Option<TemporaryObjectInfo>` (pub) - Temporary object information
- **fragmentation_analysis**: `Option<EnhancedFragmentationAnalysis>` (pub) - Memory fragmentation analysis
- **generic_instantiation**: `Option<GenericInstantiationInfo>` (pub) - Enhanced generic instantiation tracking
- **type_relationships**: `Option<TypeRelationshipInfo>` (pub) - Type relationship information
- **type_usage**: `Option<TypeUsageInfo>` (pub) - Type usage information
- **function_call_tracking**: `Option<FunctionCallTrackingInfo>` (pub) - Function call tracking
- **lifecycle_tracking**: `Option<ObjectLifecycleInfo>` (pub) - Object lifecycle tracking
- **access_tracking**: `Option<MemoryAccessTrackingInfo>` (pub) - Memory access pattern tracking
- **drop_chain_analysis**: `Option<DropChainAnalysis>` (pub) - Drop chain analysis

#### Methods

##### new

```rust
pub fn new(ptr: usize, size: usize) -> Self
```

Create a new AllocationInfo instance with basic information.

**Parameters:**
- `ptr`: `usize` - Memory address
- `size`: `usize` - Size in bytes

**Returns:** `AllocationInfo`

##### mark_deallocated

```rust
pub fn mark_deallocated(&mut self)
```

Mark this allocation as deallocated with current timestamp.

##### is_active

```rust
pub fn is_active(&self) -> bool
```

Check if this allocation is still active (not deallocated).

**Returns:** `bool` - True if allocation is still active

#### Example Usage

```rust
use memscope_rs::AllocationInfo;

let mut allocation = AllocationInfo::new(0x1000, 1024);
allocation.var_name = Some("my_vector".to_string());
allocation.type_name = Some("Vec<i32>".to_string());

println!("Allocation size: {} bytes", allocation.size);
println!("Is active: {}", allocation.is_active());

allocation.mark_deallocated();
println!("Is active after deallocation: {}", allocation.is_active());
```

### SmartPointerInfo

Smart pointer specific information for Rc/Arc tracking.

**Module:** `memscope_rs::core::types`

**Source:** `src/core/types/mod.rs`

Contains detailed information about smart pointer behavior, reference counting, and clone relationships.

#### Fields

- **data_ptr**: `usize` (pub) - Points to the actual data being shared
- **cloned_from**: `Option<usize>` (pub) - Clone relationship tracking
- **clones**: `Vec<usize>` (pub) - Clones of this smart pointer
- **ref_count_history**: `Vec<RefCountSnapshot>` (pub) - Reference count history
- **weak_count**: `Option<usize>` (pub) - Weak reference information
- **is_weak_reference**: `bool` (pub) - Is this a weak reference?
- **is_data_owner**: `bool` (pub) - Is this the last strong reference?
- **is_implicitly_deallocated**: `bool` (pub) - Was data deallocated when this was dropped?
- **pointer_type**: `SmartPointerType` (pub) - Smart pointer type

#### Methods

##### new_rc_arc

```rust
pub fn new_rc_arc(
    data_ptr: usize,
    pointer_type: SmartPointerType,
    strong_count: usize,
    weak_count: usize,
) -> Self
```

Create new smart pointer info for Rc/Arc.

##### new_weak

```rust
pub fn new_weak(data_ptr: usize, pointer_type: SmartPointerType, weak_count: usize) -> Self
```

Create new smart pointer info for Weak references.

##### record_clone

```rust
pub fn record_clone(&mut self, clone_ptr: usize, source_ptr: usize)
```

Record a clone relationship.

##### update_ref_count

```rust
pub fn update_ref_count(&mut self, strong_count: usize, weak_count: usize)
```

Update reference count.

##### mark_implicitly_deallocated

```rust
pub fn mark_implicitly_deallocated(&mut self)
```

Mark as implicitly deallocated (data was freed when this pointer was dropped).

##### latest_ref_counts

```rust
pub fn latest_ref_counts(&self) -> Option<&RefCountSnapshot>
```

Get the latest reference counts.

### SmartPointerType

Type of smart pointer.

**Module:** `memscope_rs::core::types`

**Source:** `src/core/types/mod.rs`

#### Variants

- **Rc** - Rc smart pointer
- **Arc** - Arc smart pointer  
- **RcWeak** - RcWeak smart pointer
- **ArcWeak** - ArcWeak smart pointer
- **Box** - Box smart pointer

### RefCountSnapshot

Reference count snapshot at a specific time.

**Module:** `memscope_rs::core::types`

**Source:** `src/core/types/mod.rs`

#### Fields

- **timestamp**: `u64` (pub) - Timestamp of the snapshot
- **strong_count**: `usize` (pub) - Strong reference count
- **weak_count**: `usize` (pub) - Weak reference count

## Statistics Types

### MemoryStats

Memory statistics and analysis results.

**Module:** `memscope_rs::core::types`

**Source:** `src/core/types/mod.rs`

Comprehensive statistics about memory usage, including allocation counts, sizes, and analysis results.

#### Fields

- **total_allocations**: `usize` (pub) - Total number of allocations made
- **total_allocated**: `usize` (pub) - Total bytes allocated
- **active_allocations**: `usize` (pub) - Number of currently active allocations
- **active_memory**: `usize` (pub) - Total bytes in active allocations
- **peak_allocations**: `usize` (pub) - Peak number of concurrent allocations
- **peak_memory**: `usize` (pub) - Peak memory usage in bytes
- **total_deallocations**: `usize` (pub) - Total number of deallocations performed
- **total_deallocated**: `usize` (pub) - Total bytes deallocated
- **leaked_allocations**: `usize` (pub) - Number of leaked allocations
- **leaked_memory**: `usize` (pub) - Total bytes in leaked allocations
- **fragmentation_analysis**: `FragmentationAnalysis` (pub) - Analysis of memory fragmentation
- **lifecycle_stats**: `ScopeLifecycleMetrics` (pub) - Lifecycle statistics for scopes
- **allocations**: `Vec<AllocationInfo>` (pub) - List of all allocation information
- **system_library_stats**: `SystemLibraryStats` (pub) - Statistics for system library allocations
- **concurrency_analysis**: `ConcurrencyAnalysis` (pub) - Analysis of concurrent memory operations

#### Methods

##### new

```rust
pub fn new() -> Self
```

Create a new empty MemoryStats.

**Returns:** `MemoryStats`

#### Example Usage

```rust
use memscope_rs::{get_global_tracker, MemoryStats};

let tracker = get_global_tracker();
match tracker.get_stats() {
    Ok(stats) => {
        println!("Active allocations: {}", stats.active_allocations);
        println!("Active memory: {} bytes", stats.active_memory);
        println!("Peak memory: {} bytes", stats.peak_memory);
    }
    Err(e) => eprintln!("Failed to get stats: {}", e),
}
```

### FragmentationAnalysis

Analysis of memory fragmentation.

**Module:** `memscope_rs::core::types`

**Source:** `src/core/types/mod.rs`

#### Fields

- **fragmentation_ratio**: `f64` (pub) - Ratio of fragmented to total memory
- **largest_free_block**: `usize` (pub) - Size of the largest free memory block
- **smallest_free_block**: `usize` (pub) - Size of the smallest free memory block
- **free_block_count**: `usize` (pub) - Total number of free memory blocks
- **total_free_memory**: `usize` (pub) - Total amount of free memory
- **external_fragmentation**: `f64` (pub) - External fragmentation percentage
- **internal_fragmentation**: `f64` (pub) - Internal fragmentation percentage

### MemoryTypeInfo

Memory type analysis information.

**Module:** `memscope_rs::core::types`

**Source:** `src/core/types/mod.rs`

#### Fields

- **type_name**: `String` (pub) - Name of the memory type
- **total_size**: `usize` (pub) - Total size in bytes for this type
- **allocation_count**: `usize` (pub) - Number of allocations of this type
- **average_size**: `usize` (pub) - Average size of allocations for this type
- **largest_allocation**: `usize` (pub) - Size of the largest allocation for this type
- **smallest_allocation**: `usize` (pub) - Size of the smallest allocation for this type
- **active_instances**: `usize` (pub) - Number of currently active instances
- **leaked_instances**: `usize` (pub) - Number of leaked instances

## Analysis Types

### SystemLibraryStats

Statistics for system library usage.

**Module:** `memscope_rs::core::types`

**Source:** `src/core/types/mod.rs`

#### Fields

- **std_collections**: `LibraryUsage` (pub) - Usage statistics for standard collections
- **async_runtime**: `LibraryUsage` (pub) - Usage statistics for async runtime
- **network_io**: `LibraryUsage` (pub) - Usage statistics for network I/O
- **file_system**: `LibraryUsage` (pub) - Usage statistics for file system operations
- **serialization**: `LibraryUsage` (pub) - Usage statistics for serialization
- **regex_engine**: `LibraryUsage` (pub) - Usage statistics for regex operations
- **crypto_security**: `LibraryUsage` (pub) - Usage statistics for cryptographic operations
- **database**: `LibraryUsage` (pub) - Usage statistics for database operations
- **graphics_ui**: `LibraryUsage` (pub) - Usage statistics for graphics and UI
- **http_stack**: `LibraryUsage` (pub) - Usage statistics for HTTP operations

### ConcurrencyAnalysis

Analysis of concurrent memory operations.

**Module:** `memscope_rs::core::types`

**Source:** `src/core/types/mod.rs`

#### Fields

- **thread_safety_allocations**: `usize` (pub) - Thread Safety Allocations
- **shared_memory_bytes**: `usize` (pub) - Shared Memory Bytes
- **mutex_protected**: `usize` (pub) - Mutex Protected
- **arc_shared**: `usize` (pub) - Arc Shared
- **rc_shared**: `usize` (pub) - Rc Shared
- **channel_buffers**: `usize` (pub) - Channel Buffers
- **thread_local_storage**: `usize` (pub) - Thread Local Storage
- **atomic_operations**: `usize` (pub) - Atomic Operations
- **lock_contention_risk**: `String` (pub) - Lock Contention Risk

## Scope Analysis Types

### ScopeLifecycleMetrics

Lifecycle metrics for memory scopes.

**Module:** `memscope_rs::core::types`

**Source:** `src/core/types/mod.rs`

#### Fields

- **scope_name**: `String` (pub) - Name of the scope
- **variable_count**: `usize` (pub) - Number of variables in scope
- **average_lifetime_ms**: `f64` (pub) - Average lifetime in milliseconds
- **total_memory_usage**: `usize` (pub) - Total memory used by scope
- **peak_memory_usage**: `usize` (pub) - Peak memory usage in scope
- **allocation_frequency**: `f64` (pub) - Frequency of allocations
- **deallocation_efficiency**: `f64` (pub) - Efficiency of deallocations
- **completed_allocations**: `usize` (pub) - Number of completed allocations
- **memory_growth_events**: `usize` (pub) - Number of memory growth events
- **peak_concurrent_variables**: `usize` (pub) - Peak number of concurrent variables
- **memory_efficiency_ratio**: `f64` (pub) - Memory efficiency ratio
- **ownership_transfer_events**: `usize` (pub) - Number of ownership transfers
- **fragmentation_score**: `f64` (pub) - Fragmentation score
- **instant_allocations**: `usize` (pub) - Number of instant allocations
- **short_term_allocations**: `usize` (pub) - Number of short-term allocations
- **medium_term_allocations**: `usize` (pub) - Number of medium-term allocations
- **long_term_allocations**: `usize` (pub) - Number of long-term allocations
- **suspected_leaks**: `usize` (pub) - Number of suspected memory leaks
- **risk_distribution**: `RiskDistribution` (pub) - Risk distribution analysis
- **scope_metrics**: `Vec<ScopeLifecycleMetrics>` (pub) - Metrics for individual scopes
- **type_lifecycle_patterns**: `Vec<TypeLifecyclePattern>` (pub) - Lifecycle patterns for types

## Usage Examples

### Basic Type Usage

```rust
use memscope_rs::{AllocationInfo, MemoryStats, TrackingError, TrackingResult};

fn example_usage() -> TrackingResult<()> {
    // Create allocation info
    let mut allocation = AllocationInfo::new(0x1000, 1024);
    allocation.var_name = Some("example_vec".to_string());
    allocation.type_name = Some("Vec<i32>".to_string());
    
    // Check if allocation is active
    if allocation.is_active() {
        println!("Allocation is still active");
    }
    
    // Mark as deallocated
    allocation.mark_deallocated();
    
    // Create memory stats
    let stats = MemoryStats::new();
    println!("Initial stats: {} active allocations", stats.active_allocations);
    
    Ok(())
}
```

### Smart Pointer Information

```rust
use memscope_rs::{SmartPointerInfo, SmartPointerType};

fn smart_pointer_example() {
    let mut sp_info = SmartPointerInfo::new_rc_arc(
        0x2000,
        SmartPointerType::Rc,
        1,
        0
    );
    
    // Record a clone
    sp_info.record_clone(0x3000, 0x2000);
    
    // Update reference count
    sp_info.update_ref_count(2, 0);
    
    // Check latest counts
    if let Some(snapshot) = sp_info.latest_ref_counts() {
        println!("Strong count: {}", snapshot.strong_count);
        println!("Weak count: {}", snapshot.weak_count);
    }
}
```

## See Also

- [Tracking API Reference](tracking-api.md) - Functions for memory tracking
- [Analysis API Reference](analysis-api.md) - Memory analysis functions
- [Export API Reference](export-api.md) - Data export functionality