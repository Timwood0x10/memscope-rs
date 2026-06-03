# MemScope-RS Visualization And Precision TODO

> Branch: `dev`
> Scope: Improve dashboard visualization, data correlation, and analysis precision.
> Constraint: Follow `./aim/rules.md`; do not delete files, do not use `rm`, and keep every module behavior-compatible during refactors.

## 1. Goals

- Build a practical visualization roadmap from the existing dashboard and export pipeline.
- Turn the current isolated data models into correlated views: time, thread, task, variable, ownership, unsafe, and FFI.
- Use Rust Nomicon concepts to improve analysis precision for unsafe code, aliasing, lifetimes, drops, layout, and concurrency.
- Keep each change small, testable, and compatible with the current public API.

## 2. Current Baseline

- Dashboard context is built from `Tracker`, `MemoryPassportTracker`, optional async tracker, and event snapshots.
- `MemoryEvent` already contains enough raw material for lifecycle visualization: timestamp, event type, pointer, size, thread id, variable name, type name, and source location.
- Allocation reconstruction currently converts events into allocation records, but the dashboard does not expose the full event timeline to the frontend.
- Thread data, async task data, ownership data, unsafe reports, passport data, and Top-N reports exist, but the dashboard lacks a shared index layer to connect them.
- Task graph support exists, but the graph can be empty; the UI should fall back to `async_tasks` instead of showing an empty task experience.

## 3. Delivery Rules

- [ ] Do not delete any project file during this roadmap.
- [ ] Do not execute `rm` during this roadmap.
- [ ] Do not use git commands during implementation.
- [ ] Keep every Rust source file under 1000 lines; split new logic into modules before crossing the limit.
- [ ] After each Rust module change, run `make fmt`.
- [ ] After each completed module, run the smallest relevant test first, then run `make check` before handoff.
- [ ] Treat warnings as acceptable only when they are unrelated; do not silence warnings with `#[allow(dead_code)]`.
- [ ] Tests must check invariants and hidden bugs, not only happy-path `assert!` calls.

## 4. Phase 1: Dashboard Data Integrity

### 4.1 Fix Existing Data Distortion

- [ ] Replace hardcoded dashboard thread count with the actual aggregated thread count.
- [ ] Review allocation limiting in dashboard context and avoid silently dropping half of allocation data.
- [ ] Add an explicit dashboard sampling policy if large exports need truncation.
- [ ] Surface sampling metadata to the frontend: original count, displayed count, sampling strategy, and warning flag.
- [ ] Ensure total memory, active allocations, leak counts, and thread summaries are computed from consistent input sets.

### 4.2 Export Memory Events To Dashboard JSON

- [ ] Add a lightweight serializable event view for dashboard usage.
- [ ] Include event timestamp, event type, pointer, size, thread id, task id if available, variable name, type name, source file, and source line.
- [ ] Keep raw event export optional or bounded if export size becomes too large.
- [ ] Add tests proving allocate/deallocate/reallocate/move/clone events survive dashboard serialization.
- [ ] Validate that event export does not change existing `export_html` behavior.

### 4.3 Add Frontend DataIndex

- [ ] Build `ptr_to_allocation` index.
- [ ] Build `thread_id_to_allocations` index.
- [ ] Build `thread_id_to_events` index.
- [ ] Build `type_name_to_allocations` index.
- [ ] Build `var_name_to_allocations` index.
- [ ] Build `allocation_ptr_to_passport` index.
- [ ] Build `allocation_ptr_to_unsafe_reports` index.
- [ ] Build `source_location_to_allocations` index.
- [ ] Cache derived indexes once during dashboard initialization.

### 4.4 Acceptance Criteria

- [ ] Dashboard shows correct thread count for multithreaded examples.
- [ ] Dashboard can display whether allocation data is complete or sampled.
- [ ] Frontend can answer thread, pointer, type, variable, and source-location queries without scanning all arrays repeatedly.
- [ ] `make fmt` succeeds after Rust changes.
- [ ] `make check` finishes with 0 errors.

## 5. Phase 2: Time Travel Visualization

### 5.1 Lifecycle Timeline

- [ ] Render allocation lifetime bars from event timestamps.
- [ ] Mark active leaked allocations as open-ended bars.
- [ ] Use color by type category or ownership category.
- [ ] Show tooltip with pointer, type, variable, thread id, size, source location, and lifetime.
- [ ] Add filters for thread, type, variable, source location, leak status, and unsafe status.

### 5.2 Time Slice Snapshot

- [ ] Add a time slider backed by sorted `MemoryEvent` data.
- [ ] Compute active allocations at the selected timestamp.
- [ ] Compute memory-by-thread at the selected timestamp.
- [ ] Compute memory-by-type at the selected timestamp.
- [ ] Show allocation count, live bytes, peak bytes so far, leak candidates, and unsafe boundary count.

### 5.3 Playback Mode

- [ ] Add play, pause, speed, and reset controls.
- [ ] Avoid re-rendering heavy graphs on every tick; update only aggregated stats during playback.
- [ ] Add throttling for large event streams.
- [ ] Defer Canvas/Web Worker work until event count proves DOM rendering is too slow.

### 5.4 Acceptance Criteria

- [ ] Users can locate the first leaked allocation on the timeline.
- [ ] Users can filter all related views by a selected time range.
- [ ] Timeline remains usable on at least 10,000 events.
- [ ] Time-slice statistics match reconstructed active allocations.

## 6. Phase 3: Thread Topology Visualization

### 6.1 Thread Relationship Graph

- [ ] Render each thread as a node.
- [ ] Size thread nodes by current or peak memory.
- [ ] Color thread nodes by risk: normal, leak, unsafe, FFI, or mixed.
- [ ] Infer edges from shared variable/type patterns, clone relationships, and `Arc`-like smart pointer data.
- [ ] Style edges by inferred relationship type: shared ownership, clone, unsafe sharing, or source-location affinity.

### 6.2 Thread Detail Panel

- [ ] Show memory-by-type chart for selected thread.
- [ ] Show top variables for selected thread.
- [ ] Show top source locations for selected thread.
- [ ] Show unsafe reports linked to selected thread through allocation pointer indexes.
- [ ] Show task candidates for selected thread when task-thread mapping is unavailable.
- [ ] Add navigation from thread detail to timeline and variable graph.

### 6.3 Acceptance Criteria

- [ ] Multithreaded examples render one node per real thread.
- [ ] Selecting a thread filters allocation table, timeline, unsafe list, and variable graph.
- [ ] Thread graph works even when no inferred edges exist.
- [ ] Relationship inference explains its confidence and evidence in the UI.

## 7. Phase 4: Task Hierarchy Visualization

### 7.1 Task Graph Fallback

- [ ] Render `task_graph_json` when graph nodes exist.
- [ ] Fall back to `async_tasks` cards or list when task graph is empty.
- [ ] Show task memory, peak memory, allocation count, duration, completion status, and leak flag.
- [ ] Preserve the existing Gantt timeline and improve empty-state messaging.

### 7.2 Task Correlation

- [ ] Add `thread_id` to task data when available from async tracker internals.
- [ ] Add `allocation_ids` or allocation pointer list when task-attribution data exists.
- [ ] Add `task_id` to allocation or event records when attribution is reliable.
- [ ] Avoid fake task links; label inferred links with confidence.

### 7.3 Acceptance Criteria

- [ ] Async examples show useful task data even with empty task graph.
- [ ] Task selection filters allocations and timeline when correlation data exists.
- [ ] Missing task correlation is visible as “unknown”, not silently guessed.

## 8. Phase 5: Variable And Ownership Graphs

### 8.1 Enhanced Ownership Graph

- [ ] Group graph nodes by thread, type, and ownership category.
- [ ] Color nodes by ownership model: stack, box, rc, arc, raw pointer, FFI handle, and unknown.
- [ ] Highlight clone chains and shared owners.
- [ ] Highlight suspected ownership cycles and leaked cycle members.
- [ ] Add graph filters for thread, type, leak status, unsafe status, and source location.

### 8.2 Leak Path View

- [ ] Build a leak-only subgraph from leaked allocation pointers.
- [ ] Include relationships touching leaked nodes.
- [ ] Display probable root cause: cycle, missing deallocation, FFI ownership transfer, raw pointer escape, or unknown.
- [ ] Link every leak node back to time travel and source location.

### 8.3 Acceptance Criteria

- [ ] Leak graph contains only relevant leaked nodes and neighbors.
- [ ] Ownership graph explains why a node is considered risky.
- [ ] Variable graph remains readable with more than 1,000 relationships through filtering or sampling.

## 9. Phase 6: Unsafe And FFI Visualization

### 9.1 Unsafe Call Stack View

- [ ] Group unsafe reports by source file and source line.
- [ ] Build a call-stack tree when stack trace data is available.
- [ ] Show risk level, risk factors, allocation pointer, variable, type, and lifecycle events.
- [ ] Add source-location navigation data without assuming a specific editor.

### 9.2 FFI Boundary Flow

- [ ] Model each FFI boundary crossing as a flow event.
- [ ] Distinguish Rust-to-C, C-to-Rust, ownership transfer, borrowed pointer, and unknown transfer.
- [ ] Highlight boundaries where allocation and deallocation appear on different sides.
- [ ] Show suspected responsibility mismatch when a pointer enters FFI and never returns or deallocates.

### 9.3 Acceptance Criteria

- [ ] Unsafe reports can be navigated by file, function, pointer, thread, and time.
- [ ] FFI view distinguishes ownership transfer from temporary borrowing when evidence exists.
- [ ] Unknown cases are shown explicitly instead of being reported as certain bugs.

## 10. Nomicon-Based Precision Improvements

### 10.A Source Evidence Used By This Plan

This roadmap is based on existing MemScope-RS source modules and real examples, not only on abstract Nomicon concepts.

#### Ownership Evidence

- `src/analysis/ownership_graph.rs` already defines post-analysis ownership propagation from passport events, with `OwnershipOp::{Create, Drop, RcClone, ArcClone, Move, SharedBorrow, MutBorrow}` and graph edges such as `Owns`, `Contains`, `Borrows`, `RcClone`, `ArcClone`, `Move`, `SharedBorrow`, and `MutBorrow`.
- `src/capture/types/ownership.rs` already models ownership hierarchy, root owners, ownership transfer events, weak references, circular references, and ownership kinds including unique, shared single-threaded, shared multi-threaded, borrowed, weak, and raw.
- `src/capture/types/smart_pointer.rs` and `src/metadata/smart_pointers/*` provide a natural place to improve `Rc`, `Arc`, `Weak`, clone, and smart-pointer metadata collection.
- `src/event_store/event.rs` already contains ownership-relevant event variants: `Move`, `Borrow`, `Return`, `Clone`, plus clone source/target pointer fields.
- `src/analysis/memory_passport_tracker.rs` already records passport lifecycle events and status transitions, including allocated in Rust, handover to FFI, freed by foreign code, reclaimed by Rust, boundary access, and ownership transfer.
- `examples/variable_relationships_showcase.rs` is a real ownership workload: clone-like `Vec<i32>` groups, `Arc` clone chains, `Rc` clone chains, an intentional `Rc<RefCell<Node>>` retain cycle, and a safe linear `Rc` graph.

#### Unsafe And FFI Evidence

- `src/analysis/unsafe_ffi_tracker.rs` explicitly targets unsafe Rust allocation, FFI allocation, cross-boundary transfer, and safety violation detection.
- `src/analysis/unsafe_ffi_tracker.rs` already distinguishes allocation sources: `RustSafe`, `UnsafeRust`, `FfiC`, and `CrossBoundary`.
- `src/analysis/unsafe_ffi_tracker.rs` already models boundary events: `RustToFfi`, `FfiToRust`, `OwnershipTransfer`, and `SharedAccess`.
- `src/analysis/unsafe_ffi_tracker.rs` already models safety violations: double free, invalid free, potential leak, and cross-boundary risk.
- `src/analysis/ffi_function_resolver.rs` already resolves FFI functions into library, function name, optional signature, category, and risk level.
- `examples/unsafe_ffi_demo.rs` is a real FFI workload: Rust `std::alloc` allocation/deallocation, libc `malloc`/`calloc`/`free`, passport creation, and FFI handover recording.
- `examples/merkle_tree.rs` is a real unsafe workload: raw pointer copy, manual buffer allocation, unsafe tree operations, passport creation, unsafe allocation tracking, and unsafe deallocation tracking.

### 10.B Ownership-Driven Collection Strategy

The collection strategy should treat ownership as a first-class signal, not only as a relationship inferred from type names. Nomicon's ownership, aliasing, drop, and interior mutability rules should guide what evidence is collected and how strong each conclusion is.

| Ownership Area | Existing Evidence | Collection Gap | Collection Optimization |
| --- | --- | --- | --- |
| Unique ownership | `OwnershipType::Unique`, `EdgeKind::Owns`, `Box<T>` examples. | Heap allocation and logical owner can be separated. | Record owner object id, pointee object id, allocation generation, and owner transition events. |
| Move semantics | `OwnershipOp::Move`, `MemoryEventType::Move`. | Moves may not be connected to allocation generation and logical owner. | Record source owner, destination owner, moved type, pointer stability, and whether move is logical-only or address-changing. |
| Clone semantics | `OwnershipOp::{RcClone, ArcClone}`, `MemoryEventType::Clone`, clone source/target pointers. | Plain `Clone` can mean deep copy, shallow refcount clone, or handle copy. | Classify clone as `DeepClone`, `RcClone`, `ArcClone`, `HandleClone`, `UnknownClone` with evidence. |
| Shared ownership | `Arc`/`Rc` examples and smart-pointer modules. | Refcount data may be inferred from type names instead of safe counters. | Collect safe strong/weak counts for `Rc`/`Arc` snapshots and link clones to one logical allocation group. |
| Weak references | `WeakReferenceInfo` exists. | Weak edges are easy to miss in graphs and leak analysis. | Record weak count, upgrade attempts if instrumented, and separate weak edges from strong ownership edges. |
| Borrowing | `SharedBorrow`, `MutBorrow`, `BorrowState`. | Borrow events may not capture duration or conflict evidence. | Record borrow start/end, borrow kind, borrower id, borrowed object id, and conflict evidence. |
| Interior mutability | Real examples use `Rc<RefCell<Node>>`. | `Rc<RefCell<T>>` cycles and borrow risks need different treatment from plain `Rc<T>`. | Detect `Cell`, `RefCell`, `Mutex`, `RwLock`, `Atomic`, and `UnsafeCell` containers as ownership modifiers. |
| Drop semantics | `OwnershipOp::Drop`, lifecycle and drop-chain modules. | Missing drop can be intentional for `ManuallyDrop`, `mem::forget`, or FFI transfer. | Record drop expectation: normal, forgotten, manual, FFI-owned, leaked, unknown. |
| Cycles | Circular reference analysis and `Rc` retain-cycle example exist. | Cycle confidence depends on strong vs weak edges. | Detect cycles only through strong ownership edges by default; show weak edges as cycle breakers. |
| Cross-thread ownership | `Arc` and thread data exist. | `Arc` sharing is safe ownership, but inner mutability can still matter. | Record thread ids for owners, clone sites, and shared access; separate shared ownership from data-race suspicion. |

### 10.C Ownership Data Model TODO

- [ ] Add `OwnershipEvidence` with source fields: event store, passport event, smart pointer snapshot, derive metadata, unsafe tracker, or heuristic.
- [ ] Add `OwnershipConfidence`: `Observed`, `InferredStrong`, `InferredWeak`, `Heuristic`, `Unknown`.
- [ ] Add `CloneKind`: `DeepClone`, `RcClone`, `ArcClone`, `WeakClone`, `HandleClone`, `CopyClone`, `UnknownClone`.
- [ ] Add `OwnerKind`: `StackOwner`, `HeapOwner`, `SmartPointerOwner`, `ContainerOwner`, `Borrower`, `ForeignOwner`, `UnknownOwner`.
- [ ] Add `DropExpectation`: `NormalDrop`, `ManualDrop`, `Forgotten`, `ForeignFree`, `RustReclaim`, `NoDropNeeded`, `Unknown`.
- [ ] Add `OwnershipEdgeEvidence` to graph edges so the dashboard can explain why an edge exists.
- [ ] Add allocation generation id to every ownership edge to avoid false links caused by address reuse.
- [ ] Add strong/weak edge type to cycle detection so weak references do not create false leak cycles.
- [ ] Add thread id and task id to ownership events when available.

### 10.D Ownership Visualization TODO

- [ ] Show separate layers for unique ownership, shared ownership, borrow edges, weak edges, move edges, clone edges, and FFI transfer edges.
- [ ] Add an ownership timeline for selected object: create, move, clone, borrow, return, handover, reclaim, drop.
- [ ] Add `Rc`/`Arc` clone-chain view with strong and weak counts over time.
- [ ] Add cycle explanation view that lists only strong edges forming the suspected cycle.
- [ ] Add `Rc<RefCell<T>>` special marker because it combines shared ownership with runtime borrow checking and interior mutability.
- [ ] Add confidence badges on inferred relationships, especially type-name-only clone relationships.
- [ ] Add source-code anchors for ownership events when source file and line are available.

### 10.E Unsafe And FFI Collection Strategy

Unsafe and FFI collection should model contracts and ownership transfer explicitly. A pointer crossing a boundary is not automatically a leak or bug; the report needs to know who owns it, who may access it, who must free it, and which allocator family applies.

| Unsafe/FFI Area | Existing Evidence | Collection Gap | Collection Optimization |
| --- | --- | --- | --- |
| Unsafe allocation | `AllocationSource::UnsafeRust`, unsafe call stack, risk assessment. | Layout, allocator family, and owner responsibility may be missing. | Record allocator API, layout size/alignment, owner context, and expected deallocator. |
| FFI allocation | `AllocationSource::FfiC`, FFI resolver, libc hook info. | Foreign allocator ownership can be confused with Rust ownership. | Record allocator family, library, function, returned ownership state, and expected free function. |
| Boundary transfer | `BoundaryEventType::{RustToFfi, FfiToRust, OwnershipTransfer, SharedAccess}`. | Borrowed pointer vs ownership transfer may be ambiguous. | Require `BoundaryOwnershipMode`: borrowed, shared, transferred, returned, unknown. |
| Passport lifecycle | Passport events already include handover, freed by foreign, reclaimed, ownership transfer. | Passport status may not include evidence strength. | Attach evidence level, boundary mode, allocator family, and validation method to passport events. |
| Invalid free | Safety violation exists. | Address reuse can cause false invalid/double-free reports. | Use allocation generation id and allocator family before reporting. |
| Double free | Safety violation exists. | Rust free vs foreign free need separate families. | Report double free only inside the same allocation generation and expected deallocator family. |
| FFI function risk | Resolver has function category and risk. | Function risk is not the same as actual memory risk. | Combine function risk with pointer ownership, size, layout, and lifecycle evidence. |
| Raw pointer access | Unsafe stack frames and risk factors exist. | Dereference, offset, copy, and lifetime evidence need separation. | Add operation kind: deref, offset, read, write, copy, cast, alloc, dealloc, boundary pass. |

### 10.F Unsafe And FFI Data Model TODO

- [ ] Add `AllocatorFamily`: `RustGlobal`, `System`, `LibcMalloc`, `LibcCalloc`, `LibcRealloc`, `Custom`, `Foreign`, `Unknown`.
- [ ] Add `ExpectedDeallocator`: `RustDealloc`, `LibcFree`, `CustomFunction`, `ForeignOwner`, `NoDeallocator`, `Unknown`.
- [ ] Add `BoundaryOwnershipMode`: `Borrowed`, `SharedAccess`, `TransferredToForeign`, `TransferredToRust`, `ReturnedToRust`, `FreedByForeign`, `Unknown`.
- [ ] Add `UnsafeOperationKind`: `RawDeref`, `RawRead`, `RawWrite`, `PointerOffset`, `PointerCast`, `Alloc`, `Dealloc`, `Realloc`, `Copy`, `FfiCall`, `BoundaryPass`.
- [ ] Add `FfiContractEvidence`: function name, library name, signature, category, risk level, pointer role, size argument, deallocator hint.
- [ ] Link unsafe/FFI events to `MemoryPassport` by allocation pointer and generation id.
- [ ] Link unsafe/FFI events to `TypeLayoutSnapshot` when pointer type or pointee layout is known.

### 10.G Unsafe And FFI Visualization TODO

- [ ] Show boundary flow as Rust owner → FFI call → foreign owner/shared borrower → free/reclaim outcome.
- [ ] Show allocator-family mismatch warnings only when allocator and deallocator evidence is known.
- [ ] Show unknown ownership as yellow/unknown, not as red/confirmed leak.
- [ ] Group unsafe operations by operation kind, source location, pointer, passport, and allocation generation.
- [ ] Show FFI function resolver data in tooltips: library, function, category, signature, and risk level.
- [ ] Add passport lifecycle swimlane for FFI memory: allocated, handed over, accessed, transferred, freed, reclaimed, leaked, unknown.
- [ ] Add raw-pointer operation timeline for unsafe workloads such as the Merkle tree example.

### 10.H Source-Based Validation TODO

- [ ] Use `examples/variable_relationships_showcase.rs` to validate Arc clone chains, Rc clone chains, Rc retain cycle detection, and safe linear Rc graph behavior.
- [ ] Use `examples/unsafe_ffi_demo.rs` to validate Rust allocation, libc allocation, FFI handover, free, passport lifecycle, and boundary flow visualization.
- [ ] Use `examples/merkle_tree.rs` to validate raw pointer operations, unsafe allocation tracking, unsafe deallocation tracking, and source-location grouping.
- [ ] Add tests proving weak references do not form leak cycles.
- [ ] Add tests proving `Rc<RefCell<T>>` cycles are classified as strong-cycle leak candidates, while linear `Rc` graphs are not.
- [ ] Add tests proving foreign-owned memory with unknown ownership is not reported as a confirmed leak.
- [ ] Add tests proving allocator-family mismatch requires known allocator and known deallocator evidence.

### 10.0 Rust Nomicon Reference Map

This section maps Rust Nomicon topics to concrete MemScope-RS precision work. The goal is not to encourage more unsafe code, but to use Nomicon's memory model to reduce false positives and make unsafe diagnostics more exact.

| Nomicon Topic | Precision Problem In MemScope-RS | Concrete Improvement |
| --- | --- | --- |
| `Aliasing` | Shared pointers, clones, borrows, and raw pointers can look like the same ownership relation. | Add alias categories: shared reference, mutable-exclusive candidate, raw pointer alias, clone alias, and unknown alias. |
| `Lifetimes` | Allocation lifetime and Rust borrow lifetime are currently easy to mix together. | Separate allocation lifetime, owner lifetime, borrow lifetime, and FFI exposure lifetime in reports. |
| `Drop Check` | Missing deallocation, early drop, double drop, and container/member drop ordering can be confused. | Build drop-chain evidence from deallocate events, passport events, clone events, and container metadata. |
| `Ownership And Moves` | Move events are not fully used to explain why an allocation changed logical owner. | Record owner transitions and show move edges separately from clone edges. |
| `Send And Sync` | Cross-thread memory use can be reported too broadly as risky. | Classify thread safety using type patterns, smart pointer metadata, and cross-thread event evidence. |
| `Subtyping And Variance` | References hidden in generic containers may have different lifetime expectations. | Add future generic-container hints for `Vec<T>`, `Box<T>`, `Rc<T>`, `Arc<T>`, `RefCell<T>`, and `Mutex<T>`. |
| `Uninitialized Memory` | `MaybeUninit` and raw buffers should not be treated as leaks or invalid reads without evidence. | Add `maybe_uninit` risk state and require access/read evidence before escalating severity. |
| `Working With Unsafe` | Unsafe reports need evidence and contracts, not only risk labels. | Attach safety-contract fields: preconditions, observed evidence, violated invariant, confidence. |
| `Raw Pointers` | Numeric pointer equality can create false relationships after address reuse. | Introduce allocation generation ids and pointer provenance. |
| `FFI` | Rust-owned memory, foreign-owned memory, borrowed FFI memory, and transferred ownership are different. | Add FFI ownership states: borrowed, Rust-owned, foreign-owned, transferred-to-foreign, transferred-to-Rust, unknown. |
| `Data Layout` | Layout mismatch can be mistaken for normal allocation metadata. | Track observed size, expected size, alignment, repr hint, slice length, and trait-object metadata when available. |
| `Exception Safety` | Panic paths can skip cleanup or leave partial lifecycle traces. | Add panic-boundary lifecycle markers when the tracker can observe unwind or shutdown cleanup. |

### 10.0.1 Nomicon-Inspired Diagnostic Principles

- [ ] Separate “undefined behavior risk” from “confirmed memory bug”.
- [ ] Require evidence before escalating an unsafe finding from possible to likely or confirmed.
- [ ] Preserve uncertainty explicitly with `unknown` states instead of guessing ownership or lifetime.
- [ ] Prefer provenance and generation ids over raw pointer numeric equality.
- [ ] Report the violated invariant in Nomicon terms: aliasing, validity, initialization, layout, ownership, thread safety, or FFI contract.
- [ ] Show which data source produced the conclusion: event store, passport tracker, relationship inference, async tracker, source metadata, or unsafe inference.

### 10.0.2 Precision Labels To Add

- [ ] Add `EvidenceLevel`: `Observed`, `Inferred`, `Heuristic`, `Unknown`.
- [ ] Add `RiskConfidence`: `Confirmed`, `Likely`, `Possible`, `Unknown`.
- [ ] Add `UnsafeInvariant`: `Aliasing`, `Validity`, `Initialized`, `Layout`, `Drop`, `ThreadSafety`, `FfiOwnership`, `AllocatorFamily`.
- [ ] Add `PointerProvenance`: `Allocator`, `Stack`, `Reallocation`, `Clone`, `Move`, `FfiInput`, `FfiOutput`, `ContainerMember`, `Unknown`.
- [ ] Add `OwnershipState`: `OwnedByRust`, `BorrowedByRust`, `OwnedByForeign`, `BorrowedByForeign`, `Shared`, `Transferred`, `Released`, `Unknown`.
- [ ] Add `LifetimeKind`: `AllocationLifetime`, `OwnerLifetime`, `BorrowLifetime`, `TaskLifetime`, `ThreadLifetime`, `FfiExposureLifetime`.

### 10.0.3 Nomicon-To-Visualization Mapping

- [ ] Aliasing view: show raw pointer aliases, shared aliases, mutable-exclusive candidates, and conflicting evidence.
- [ ] Drop view: show allocation, owner transitions, destructor/deallocation, leaks, double-free candidates, and post-drop use candidates.
- [ ] Provenance view: show pointer origin, allocation generation, reallocation transitions, FFI input/output, and address reuse.
- [ ] Layout view: show expected size, observed size, alignment, type category, repr hint, and layout mismatch risk.
- [ ] Thread-safety view: show cross-thread ownership edges and separate `Arc` sharing from non-thread-safe sharing.
- [ ] FFI contract view: show ownership transfer, borrowed pointer windows, allocator family, and unmatched boundary crossings.

### 10.0.4 Type-Layout-Driven Collection Strategy

This section focuses on the Rust Nomicon's low-level type layout model. The collection strategy should stop treating every allocation as only `{ ptr, size, type_name }`; instead, it should collect layout-aware facts that explain what the bytes mean.

| Nomicon Layout Topic | Current Collection Risk | Collection Optimization |
| --- | --- | --- |
| `repr(Rust)` has unspecified field order | Field-level layout cannot be guessed from source order. | Only report field offsets when emitted by derive/proc-macro metadata or explicit layout instrumentation. |
| `repr(C)` has stable C-compatible order | FFI structs need stronger layout confidence than normal Rust structs. | Add `repr_hint` and mark `repr(C)` records as layout-stable for FFI diagnostics. |
| Alignment and padding | Heap block size alone cannot explain wasted bytes or false fragmentation. | Collect `size_of::<T>()`, `align_of::<T>()`, observed allocation size, and padding estimate when type metadata exists. |
| Zero-sized types | ZST values can have non-null dangling-like addresses and no heap size. | Add `is_zst` and avoid treating zero-byte logical values as allocator leaks. |
| Dynamically sized types | Slices, `str`, and trait objects are fat pointers, not simple thin pointers. | Collect DST metadata: slice length, string length, dynamic size from `size_of_val`, and trait-object marker when safely available. |
| Niche optimization | `Option<&T>`, `Option<Box<T>>`, and some enums can have the same size as pointers. | Add enum/niche-aware classification to avoid misreading compact enum layouts as missing fields. |
| Enum discriminants | Enum payload size and discriminant layout vary by representation. | Record enum category and only infer discriminant cost when representation metadata is known. |
| `Vec<T>` layout | Allocator sees buffer bytes, while logical object has pointer, length, and capacity. | Collect safe `len`, `capacity`, `element_size`, `element_align`, used bytes, reserved bytes, and utilization ratio. |
| `String` layout | String allocation must distinguish bytes used from capacity reserved. | Collect safe `len`, `capacity`, UTF-8 byte usage, reserved bytes, and growth pattern. |
| `Box<T>` layout | The box value is small, but the pointee has the real layout. | Record pointee `size_of::<T>()`, `align_of::<T>()`, and heap allocation generation. |
| `Rc<T>` and `Arc<T>` layout | The observed pointer points into an allocation with strong/weak counters and payload. | Collect strong count, weak count when safe, smart pointer kind, pointee type, and clone lineage. |
| `UnsafeCell<T>` and interior mutability | Shared references can still mutate through legal interior mutability. | Mark interior-mutability containers to reduce false aliasing and data-race reports. |
| `MaybeUninit<T>` | Uninitialized storage is valid allocation but not initialized value. | Record initialized-state as `unknown` unless instrumentation observes initialization. |
| `ManuallyDrop<T>` | Drop may be intentionally suppressed. | Mark manual-drop ownership so missing destructor evidence does not immediately become a leak. |
| `Pin<P>` | Address stability matters for move analysis. | Mark pinned values so move events are checked as stronger invariants. |

### 10.0.5 Layout Metadata To Collect

- [ ] Add `TypeLayoutSnapshot` for values tracked through explicit APIs or derive macros.
- [ ] Record `type_name`, `type_id_hash`, `size_of_t`, `align_of_t`, `size_of_val`, and `is_sized` when available.
- [ ] Record `repr_hint`: `Rust`, `C`, `Transparent`, `Packed`, `Align`, `Unknown`.
- [ ] Record `layout_kind`: `Primitive`, `Struct`, `Enum`, `Union`, `Tuple`, `Array`, `Slice`, `Str`, `TraitObject`, `Pointer`, `SmartPointer`, `Container`, `Zst`, `Unknown`.
- [ ] Record `pointer_width`: `Thin`, `FatSlice`, `FatTraitObject`, `Function`, `Unknown`.
- [ ] Record `logical_size_bytes`, `allocated_size_bytes`, `used_size_bytes`, and `reserved_size_bytes` separately.
- [ ] Record `alignment`, `observed_layout_alignment`, and `alignment_mismatch` for allocator/layout consistency checks.
- [ ] Record `container_len`, `container_capacity`, `element_size`, and `element_align` for `Vec`, `String`, arrays, and slices.
- [ ] Record `strong_count`, `weak_count`, and `pointee_layout` for `Rc` and `Arc` when using safe APIs.
- [ ] Record `interior_mutability`: `None`, `Cell`, `RefCell`, `Mutex`, `RwLock`, `Atomic`, `UnsafeCell`, `Unknown`.
- [ ] Record `drop_semantics`: `Normal`, `NeedsDrop`, `Copy`, `ManuallyDrop`, `MaybeUninit`, `Pinned`, `Unknown`.

### 10.0.6 Collection Layers

- [ ] Allocator layer: collect raw pointer, requested layout size, requested alignment, allocation generation, thread id, and timestamp.
- [ ] Explicit tracking macro layer: collect `size_of::<T>()`, `align_of::<T>()`, `type_name::<T>()`, source location, and ownership category.
- [ ] Derive macro layer: collect field names, declared field types, optional field offsets, repr hints, and container member relationships.
- [ ] Smart pointer layer: collect safe `Rc`/`Arc` counts, pointee type, clone source, clone target, and weak-reference evidence.
- [ ] Container layer: collect `Vec` and `String` length/capacity/utilization through safe public APIs.
- [ ] DST layer: collect slice length, `str` length, dynamic value size, and fat-pointer category without relying on unstable internals.
- [ ] FFI layer: collect ABI boundary, repr hint, pointer ownership state, allocator family, and expected layout if declared.

### 10.0.7 False-Positive Reduction Rules

- [ ] Do not infer field layout for `repr(Rust)` structs unless instrumentation provides trusted offsets.
- [ ] Do not treat ZST tracking records as heap leaks only because they have pointer-like values.
- [ ] Do not equate `Vec<T>` object size with its heap buffer size; always separate handle size, length, capacity, and buffer allocation.
- [ ] Do not report `MaybeUninit<T>` as invalid without read/access evidence.
- [ ] Do not report `ManuallyDrop<T>` as a leak without ownership-release evidence.
- [ ] Do not treat `UnsafeCell<T>` as automatically unsafe; use it to explain legal interior mutability first.
- [ ] Do not infer trait object concrete type from vtable pointer unless a safe or explicitly instrumented source provides it.
- [ ] Do not merge allocations by pointer address without allocation generation because allocator address reuse is valid.

### 10.0.8 Data Collection Implementation TODO

- [ ] Design `TypeLayoutSnapshot` in a small module under capture types, keeping the file below 1000 lines.
- [ ] Add helper constructors that accept generic `T` and record `size_of::<T>()`, `align_of::<T>()`, and `type_name::<T>()`.
- [ ] Add safe container snapshot helpers for `Vec<T>`, `String`, slices, arrays, `Box<T>`, `Rc<T>`, and `Arc<T>`.
- [ ] Extend derive macro output to include repr hints and field metadata where possible.
- [ ] Link `TypeLayoutSnapshot` to allocation events by pointer and allocation generation id.
- [ ] Export layout snapshots to dashboard JSON as a separate indexed collection.
- [ ] Add layout-aware dashboard panels: type layout card, container utilization card, padding estimate, and smart-pointer counter card.
- [ ] Add tests for ZST, Vec spare capacity, String capacity, Box pointee size, Rc/Arc counts, repr(C) metadata, and address reuse.

### 10.1 Aliasing And Borrow Precision

- [ ] Add an aliasing classifier inspired by Nomicon aliasing rules.
- [ ] Track whether multiple live references or pointer-like records target the same allocation.
- [ ] Distinguish shared aliasing from mutable-exclusive access when event metadata allows it.
- [ ] Treat raw pointers as unknown aliasing unless provenance evidence exists.
- [ ] Report aliasing findings with confidence levels: confirmed, likely, possible, or unknown.
- [ ] Avoid marking aliasing as a violation unless the evidence includes conflicting mutable access or unsafe boundary misuse.

### 10.2 Lifetime And Drop Precision

- [ ] Derive object lifetime from allocate, move, clone, metadata, and deallocate events.
- [ ] Detect missing deallocation as a leak candidate only after considering ownership transfer and container metadata.
- [ ] Detect suspicious early drop when a later event references the same pointer.
- [ ] Detect double-drop or double-free candidates from repeated deallocation events.
- [ ] Preserve deallocation ordering so drop-chain visualization can explain container/member lifetimes.

### 10.3 Pointer Provenance Precision

- [ ] Track pointer origin: allocator, stack metadata, clone target, FFI input, FFI output, reallocation target, or unknown.
- [ ] Preserve old and new pointers for reallocation events.
- [ ] Avoid merging two pointer identities only because their numeric address is reused at different times.
- [ ] Introduce allocation generation ids to distinguish address reuse.
- [ ] Add tests for address reuse after deallocation to prevent false ownership links.

### 10.4 Layout And Type Precision

- [ ] Classify layout-sensitive operations: casts, FFI structs, trait objects, slices, strings, and manually managed buffers.
- [ ] Record known alignment and size when available.
- [ ] Flag impossible or suspicious size/type pairs, such as zero-sized type confusion or buffer-size mismatch.
- [ ] Treat unknown layout as uncertainty, not as a confirmed bug.
- [ ] Add future support for `repr(C)` and `repr(Rust)` hints when metadata can capture them.

### 10.5 Uninitialized And MaybeUninit Precision

- [ ] Add risk category for possibly uninitialized memory.
- [ ] Detect `MaybeUninit`-like type names and raw allocation patterns.
- [ ] Avoid claiming uninitialized reads unless read/access evidence exists.
- [ ] Visualize initialization lifecycle if future events can record initialize/read transitions.

### 10.6 Send, Sync, And Cross-Thread Precision

- [ ] Classify cross-thread allocations and ownership transfers.
- [ ] Highlight `Rc`, `RefCell`, `Cell`, raw pointers, and non-thread-safe patterns crossing thread boundaries.
- [ ] Treat `Arc` as shared ownership but still inspect inner mutability risks.
- [ ] Add confidence-based reports for suspected data race patterns.
- [ ] Add concurrency stress tests for thread aggregation and event ordering.

### 10.7 FFI Ownership Precision

- [ ] Track whether FFI receives borrowed memory, takes ownership, returns ownership, or only observes a pointer.
- [ ] Separate Rust allocator ownership from foreign allocator ownership.
- [ ] Detect mismatched allocation/free families when evidence exists.
- [ ] Detect leaked FFI-owned memory only when ownership is known or strongly inferred.
- [ ] Add explicit unknown state for FFI ownership to reduce false positives.

### 10.8 Unsafe Contract Documentation

- [ ] Require every internal `unsafe` block touched by this roadmap to have a nearby safety explanation.
- [ ] Add `# Safety` docs to every public `unsafe fn` touched by this roadmap.
- [ ] Add tests around unsafe wrappers that verify null pointer, invalid alignment, zero-size, and boundary-size behavior.
- [ ] Prefer safe abstractions around raw pointer handling and keep unsafe blocks small.

### 10.9 Precision Acceptance Criteria

- [ ] Reports distinguish confirmed bugs from inferred risks.
- [ ] Every inferred risk includes evidence fields and confidence level.
- [ ] Address reuse does not create false ownership relationships.
- [ ] FFI unknown ownership does not become a false leak by default.
- [ ] Cross-thread shared ownership is separated from data-race suspicion.
- [ ] Unsafe-related tests cover positive, negative, and concurrency cases where applicable.

## 11. Data Model Additions

### 11.1 Near-Term Additions

- [ ] Add dashboard event DTO instead of exposing internal event structs directly.
- [ ] Add `event_count`, `is_sampled`, and `sampling_strategy` to dashboard JSON.
- [ ] Add allocation generation id in reconstructed frontend state if backend generation ids are not ready.
- [ ] Add confidence and evidence fields to inferred frontend relationships.

### 11.2 Rust-Side Additions

- [ ] Add `thread_id` to async task view data.
- [ ] Add `task_id` to allocation or event data when attribution exists.
- [ ] Add allocation generation id to distinguish address reuse.
- [ ] Add pointer provenance enum for allocator, stack, clone, reallocation, FFI, and unknown origins.
- [ ] Add optional stack trace frames to dashboard-safe allocation views.
- [ ] Add optional layout metadata: alignment, declared size, observed size, and representation hint.

### 11.3 Compatibility Rules

- [ ] New fields must be optional or have safe defaults when exported to existing dashboard templates.
- [ ] Existing examples must keep exporting `dashboard.html` and JSON files successfully.
- [ ] Public API changes require documentation and migration notes.

## 12. Testing Strategy

### 12.1 Unit Tests

- [ ] Test event-to-allocation reconstruction with allocate/deallocate pairs.
- [ ] Test address reuse with two allocation generations using the same pointer value.
- [ ] Test clone relationship reconstruction with source and target pointers.
- [ ] Test reallocation lifecycle preserving old and new pointer identity.
- [ ] Test thread aggregation with multiple thread ids.
- [ ] Test dashboard serialization with empty and non-empty event streams.

### 12.2 Integration Tests

- [ ] Export dashboard for a simple single-thread example.
- [ ] Export dashboard for a multithreaded example.
- [ ] Export dashboard for an async example with task data.
- [ ] Export dashboard for an unsafe/FFI example.
- [ ] Verify generated dashboard JSON contains expected top-level fields and stable defaults.

### 12.3 Stress And Safety Tests

- [ ] Add concurrent event ingestion tests with at least 50 threads where relevant.
- [ ] Add Miri checks for modules that touch raw pointers or manual memory logic.
- [ ] Add property-based tests for pointer, size, alignment, and event ordering if new parser logic is introduced.
- [ ] Do not add coverage runs as part of this roadmap.

## 13. Implementation Order

1. [ ] Fix thread count and allocation sampling metadata.
2. [ ] Add dashboard event DTO and serialize events into `json_data`.
3. [ ] Add frontend `DataIndex` and wire it into existing views without changing layout heavily.
4. [ ] Add time-slice statistics and lifecycle timeline.
5. [ ] Add thread detail filtering and thread topology graph.
6. [ ] Add task graph fallback from `async_tasks`.
7. [ ] Add leak path subgraph and enhanced ownership grouping.
8. [ ] Add unsafe call-stack and FFI boundary views.
9. [ ] Add backend precision fields: generation id, provenance, task/thread attribution, layout metadata.
10. [ ] Add Nomicon-inspired precision classifiers with evidence and confidence levels.

## 14. Definition Of Done

- [ ] Dashboard provides cross-view filtering by time, thread, task, variable, pointer, source location, unsafe status, and leak status.
- [ ] Analysis reports separate exact facts from inferred risks.
- [ ] Nomicon-inspired checks improve precision without increasing false positives by default.
- [ ] All new Rust code follows the naming, error handling, ownership, and documentation rules in `./aim/rules.md`.
- [ ] All new tests include meaningful assertions and invariant comments.
- [ ] `make fmt` succeeds.
- [ ] `make check` reports 0 errors.


## check list
- [ ] File is under 1000 lines
- [ ] Code is simple and straightforward
- [ ] All comments are in English
- [ ] Code-to-comment ratio is approximately 7:3
- [ ] Tests include boundary cases
- [ ] No files were deleted without permission
- [ ] Naming conventions are followed
- [ ] Code is formatted with `make fmt`
- [ ] All tests pass
- [ ] Public APIs have doc comments
- [ ] Error handling is appropriate
- [ ] Memory management is correct
- [ ] Changes are surgical and minimal