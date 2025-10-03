---
name: Performance Issue
about: Report performance problems or unexpected overhead
title: '[PERFORMANCE] '
labels: 'performance'
assignees: ''
---

## ⚡ Performance Issue

**Performance problem description**
A clear description of the performance issue you're experiencing.

**Expected performance**
What performance you expected based on documentation or benchmarks.

**Actual performance**
What performance you're actually seeing.

## 📊 Measurements

**Benchmarking setup**
```rust
// Your benchmarking code
use memscope_rs::*;

fn main() {
    // Code that demonstrates the performance issue
}
```

**Performance metrics**
- **CPU overhead**: ___% (expected: ___%)
- **Memory overhead**: ___MB (expected: ___MB)
- **Latency impact**: ___ms per operation
- **Throughput impact**: ___ops/sec reduction

**Profiling results** (if available)
```
Paste output from cargo flamegraph, perf, or other profiling tools
```

## 🖥️ Environment

**System specs:**
- CPU: [e.g. Intel i7-12700K, Apple M2, AMD Ryzen 9 5900X]
- RAM: [e.g. 32GB DDR4-3200]
- Storage: [e.g. NVMe SSD, HDD]
- OS: [e.g. Ubuntu 22.04, macOS 13.0, Windows 11]

**Rust toolchain:**
- Rust version: [output of `rustc --version`]
- Compilation flags: [e.g. --release, target-cpu=native]
- memscope-rs version: [e.g. 0.1.6]

## 🎯 Workload Details

**Tracking strategy used:**
- [ ] Core (single-threaded)
- [ ] Lock-free (multi-threaded) - threads: ___
- [ ] Async task-aware - concurrent tasks: ___
- [ ] Unified backend

**Application characteristics:**
- **Allocation rate**: ~___ allocations/second
- **Allocation sizes**: mostly ___KB, max ___MB
- **Runtime duration**: ___ minutes/hours
- **Concurrency level**: ___ threads/tasks

**Configuration:**
```toml
[dependencies]
memscope-rs = { version = "0.1.6", features = ["..."] }
```

## 🔍 Analysis

**Comparison data** (if available)
- Performance without memscope-rs: ___
- Performance with memscope-rs: ___
- Performance with other tools (Valgrind, etc.): ___

**Bottleneck location** (if identified)
- [ ] Allocation tracking
- [ ] Data export/serialization
- [ ] Lock contention
- [ ] Memory usage by tracker itself
- [ ] Other: ___________

## 📋 Additional Context

**Impact on your use case:**
- [ ] Minor inconvenience
- [ ] Significant slowdown
- [ ] Blocking production use
- [ ] Making tool unusable

**Temporary workarounds:**
Describe any workarounds you've found or configuration changes that help.

**Additional information:**
Any other details that might help diagnose the performance issue.

---

**Your performance feedback helps make memscope-rs faster for everyone! 🚀**