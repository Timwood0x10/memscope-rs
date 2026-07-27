# Why Build This Wheel

> The Rust ecosystem doesn't lack memory analysis tools — it lacks tools that *understand* Rust. Valgrind sees raw pointer allocation and deallocation; it has no idea what `Arc<Rc<Box<...>>>` means. AddressSanitizer's stack traces always point to some `alloc::vec!` internals, never your business logic. So I decided to build my own.

***

## The Problem: Rust's Memory Analysis Gap

It started with a debugging session.

I was investigating a Rust service with persistently high CPU. My gut said it was memory-related — either an `Arc` cycle keeping refcounts from ever dropping to zero, or some async task holding onto memory indefinitely.

I reached for the usual toolbelt:

- **Valgrind**: Slowed the service down 30x, and the report was full of `je_malloc`, `pthread_create`, and similar noise. It couldn't tell me "this `Arc` was cloned 47 times."
- **AddressSanitizer**: Faster to start, but same problem — it sees raw memory operations, not Rust semantics. It doesn't know the difference between a `Vec` and a `HashMap`.
- **Heaptrack**: Nicer GUI than Valgrind. Still doesn't understand Rust. `Arc::clone()` looks like a regular allocation to it.

My thought was simple: **Is there a tool that analyzes Rust as Rust — that actually understands the memory semantics of `Arc`, `Rc`, `Vec`, `Box`, `String` — rather than treating Rust like C?**

The answer was no. So I started writing one.

```mermaid
graph LR
    subgraph "Existing Tools vs Rust Needs"
        A[Valgrind] -->|"only sees alloc/free"| X["Doesn't understand<br/>Arc/Rc semantics"]
        B[AddressSanitizer] -->|"raw memory ops"| X
        C[Heaptrack] -->|"alloc/free timeline"| X
        D[perf / dtrace] -->|"system-level sampling"| X
    end

    subgraph "Rust-specific Needs"
        X --> Y["Arc/Rc clone tracking"]
        X --> Z["Ownership propagation"]
        X --> W["Async task attribution"]
        X --> V["Cycle detection"]
    end
```

## First Attempt: It Seemed Simple

I thought this would be straightforward — just hook `GlobalAlloc`, record every allocation's pointer, size, and stack trace, then check which pointers leaked at exit.

I threw together a first version in a weekend:

```rust
// Naive first version, ~June 2025
struct NaiveAllocator;

unsafe impl GlobalAlloc for NaiveAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = std::alloc::System.alloc(layout);
        TRACKER.lock().unwrap().record_alloc(ptr as usize, layout.size());
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        TRACKER.lock().unwrap().record_dealloc(ptr as usize);
        std::alloc::System.dealloc(ptr, layout);
    }
}
```

It worked. `Vec::push`, `Box::new`, `String::clone` — all captured. But this was just **raw memory operation logging**, light-years from "understanding Rust semantics."

The list of problems grew:

1. **`Arc::clone()` doesn't allocate** — it just increments a refcount. GlobalAlloc hooks are completely blind to it. Yet `Arc` cloning is one of the most common causes of memory leaks in Rust.
2. **`Vec` reallocation vs actual data growth** — when `Vec` grows, the old pointer is freed and a new one allocated. I needed to link them to the same logical object.
3. **`HashMap` has no fixed heap pointer** — its internal structure is complex; a simple `ptr → object` mapping doesn't work.
4. **Cross-thread and async tasks** — memory allocated in thread A, freed in thread B, passed through several async tasks. Who "owns" it?
5. **FFI boundaries** — once memory is handed to C, who frees it? Who reclaims it?

Each problem was a design decision. Some were right; some were wrong; some are still being revised.

## Key Technical Decision: Real vs Complete

This was the most important — and most painful — decision in the entire project.

Rust's ownership system is compile-time. `move`, `borrow`, `lifetime` are verified in MIR and gone by the time machine code is generated. The runtime can't see them.

```mermaid
flowchart LR
    A["Source Code<br/>let x = vec![1,2,3];<br/>let y = x; // move"] --> B["HIR"]
    B --> C["THIR"]
    C --> D["MIR<br/>← move/borrow checked<br/>& resolved here"]
    D --> E["LLVM IR<br/>→ move/borrow info<br/>already gone"]
    E --> F["Machine Code"]
    
    G["Runtime Tracking --/--> can only see<br/>alloc/free/realloc"]
```

I spent a long time trying to recover compile-time information at runtime. I tried:

- **Parsing DWARF debug info** to infer types → too slow, and not all allocations have debug symbols
- **Stack trace pattern matching** to distinguish `Arc::clone` from `Rc::clone` → brittle, breaks with Rust version updates
- **Hooking mprotect** to detect memory access → performance disaster

Eventually I accepted the truth:

> **I can never 100% reconstruct Rust's compile-time semantics at runtime. So I won't pretend I can.**

This decision drove the entire architecture:

```mermaid
flowchart TD
    subgraph "Trackable (Real Data)"
        A1["GlobalAlloc Hook<br/>alloc/free/realloc"]
        A2["Stack address tracking<br/>Arc/Rc on-stack pointers"]
        A3["Thread ID & Timestamp"]
        A4["Task ID registration<br/>(requires annotation)"]
        A5["Heap content read<br/>(safe scanning)"]
    end
    
    subgraph "Not Directly Trackable"
        B1["Borrow & Move semantics"]
        B2["Type layout information"]
        B3["Ownership transfer paths"]
        B4["Lifetime relationships"]
    end
    
    subgraph "Solution"
        C1["Real data shown directly"]
        C2["Inference engine + honest labels<br/>_source: inferred<br/>_confidence: low"]
        C3["Honest limitation annotations"]
    end

    A1 --> C1
    A2 --> C1
    A3 --> C1
    A4 --> C1
    A5 --> C1
    
    B1 --> C2
    B2 --> C2
    B3 --> C2
    B4 --> C2
```

Every inferred field carries its own "provenance":

```json
{
  "ptr": 0x600001234560,
  "size": 1024,
  "borrow_info": {
    "_source": "inferred",
    "_confidence": "low",
    "immutable_borrows": 3
  }
}
```

## 75% Code Deletion: A Second Birth

The v0.1.x releases were full of beginner mistakes. The worst: **feature stacking**.

Every new feature meant another file crammed into the main module. Analysis, rendering, tracking — all mixed together. By v0.1.10, the codebase had bloated to roughly 265,000 lines.

There were days I'd open the project and immediately want to close it. Not because I didn't want to maintain it — because I didn't know where to start.

In v0.2.0, I did something drastic — I scrapped the entire `src/` directory and started over.

```mermaid
flowchart LR
    subgraph "v0.1.x (Chaos)"
        S1["src/ 270K lines"]
        S1 --> S2["Tracking logic scattered everywhere"]
        S1 --> S3["Analysis mixed with rendering"]
        S1 --> S4["unwrap() all over the place"]
        S1 --> S5["One error crashes the process"]
    end
    
    subgraph "v0.2.0 (Modular)"
        T1["src/ 77K lines"]
        T1 --> T2["8 independent engines"]
        T1 --> T3["Unified error handling"]
        T1 --> T4["Clear module boundaries"]
        T1 --> T5["Error recovery mechanisms"]
    end

    S1 -->|"tear down and rebuild"| T1
```

The refactored 8-engine architecture:

```mermaid
graph TB
    subgraph "MemScope-RS Core"
        direction TB
        E1["Capture Engine"] --> E2["Event Store"]
        E2 --> E3["Analysis Engine"]
        E3 --> E4["Render Engine"]
        
        E5["Snapshot Engine"] --> E2
        E6["Timeline Engine"] --> E2
        E7["Query Engine"] --> E3
        E8["Metadata Engine"] --> E3
    end

    subgraph "User Layer"
        U1["track! / track_var! macros"]
        U2["Unified Tracker API"]
        U3["Dashboard HTML"]
    end

    U1 --> E1
    U2 --> E1
    E4 --> U3
```

This refactoring taught me a few things:

1. **Feature stacking doesn't produce architecture** — architecture is actively designed, not passively accumulated.
2. **Deleting code is harder than writing it, but more valuable** — after removing 75% of the code, the remaining 25% could do more.
3. **Error handling isn't an afterthought** — `unwrap()` is tolerable in prototypes, but not in tooling. A panic in an allocation tracker crashes the very process it's monitoring. That's unacceptable.

## Honest Section

As of writing this (June 2026), here are the parts I know are imperfect:

- **`docs/ARCHITECTURE.md` describes a different branch** — that doc reflects the `improve` branch's vision, which is ahead of `dev` by a significant margin. A documentation management failure.
- **`ownership_analyzer.rs` has both rustdoc JSON parsing and syn AST analysis as TODOs** — currently only 8 common types are hardcoded. The static analysis potential is largely untapped.
- **`src/core/types/mod.rs` is 4,161 lines** — a historical relic that should be split into smaller modules, but every refactor risks breaking existing APIs.
- **The inference engine's confidence is still a coarse label** — `low`, `medium`, `high` are too rough. v0.2.4 introduced `EvidenceLevel` and `RiskConfidence`, but that's just the first step.

## Reflection

Looking back, what I'm most satisfied with isn't a particular technical solution — it's **honesty**.

I've seen too many tools claim "automatic memory leak detection," only to reveal they just flag "not freed at exit = leak." That's roughly true in C, but completely false in Rust — `Arc` cycle memory can stay "alive" until shutdown without being a leak, while a `Box` that was `mem::forget`-ted is a real leak.

memscope-rs chose a different path: **report only what I'm certain I can see, and honestly label the rest.**

This isn't a technical decision; it's a values decision. It affects:

- **Output format** — every inferred field carries `_source` and `_confidence`
- **Architecture** — engines communicate through the event store, sharing no internal state
- **Documentation** — `LIMITATIONS.md` isn't a disclaimer; it's a core design document
- **Version evolution** — deleting and simplifying happens more often than adding features

This path is harder. Users will never complain about a tool that pretends to be perfect — they'll only complain about an honest tool being "not smart enough." But in the long run, honesty builds trust.

And trust is the only thing that matters between a tool and its user.

***

**Next article**: [GlobalAlloc Hook — Where Everything Begins](02-global-alloc-hook.md)

The next article dives into memscope-rs's lowest layer: how hooking `GlobalAlloc` intercepts every allocation with <5% overhead, and the design trade-offs of the four tracking backends (Core, LockFree, Async, Unified).