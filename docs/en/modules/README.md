# Module Documentation Index

> Detailed source-level documentation for each MemScope module

---

## Available Modules

| Module | Description |
|--------|-------------|
| [Architecture Overview](../architecture/overview.md) | The 9-engine pipeline, data flow, and system architecture |
| [Capture Backends](../capture/backends.md) | Core, Lockfree, Async, Unified — how each backend works |
| [EventStore](../event-store/eventstore.md) | Lock-free event storage and snapshot mechanism |
| [Analysis & Detectors](../analysis/detectors.md) | Pluggable detectors (leak, UAF, overflow, safety, lifecycle) |
| [Unsafe Type Inference](../analysis/unsafe-inference.md) | Heuristic type detection for FFI allocations |
| [Tracker API](../tracker-api/tracker.md) | High-level simplified interface with system monitoring |

---

## Quick Navigation by Task

**I want to understand how MemScope works:**
→ Start with [Architecture Overview](../architecture/overview.md)

**I want to choose the right tracking backend:**
→ Read [Capture Backends](../capture/backends.md)

**I want to understand how events are stored:**
→ Read [EventStore](../event-store/eventstore.md)

**I want to detect memory leaks or other issues:**
→ Read [Analysis & Detectors](../analysis/detectors.md)

**I want to understand type inference for raw pointers:**
→ Read [Unsafe Type Inference](../analysis/unsafe-inference.md)

**I want a simple API for quick tracking:**
→ Read [Tracker API](../tracker-api/tracker.md)

---

## ⚠️ Note on Legacy Documentation

The module-specific pages (`single-threaded.md`, `multithread.md`, `async.md`, `hybrid.md`) in this directory reference the **old API** (`init()`, `track_var!`, `lockfree/` module) that has been superseded by the new architecture. They are preserved for reference but should not be used as the primary documentation source.

For the current architecture, see the links above.
