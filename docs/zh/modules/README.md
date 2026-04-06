# 模块文档索引

> MemScope 各模块的源码级详细文档

---

## 可用模块

| 模块 | 描述 |
|------|------|
| [架构概览](../architecture/overview.md) | 九引擎流水线、数据流、系统架构 |
| [捕获后端](../capture/backends.md) | Core、Lockfree、Async、Unified — 每个后端的工作原理 |
| [EventStore](../event-store/eventstore.md) | 无锁事件存储与快照机制 |
| [分析与检测器](../analysis/detectors.md) | 可插拔检测器（泄漏、UAF、溢出、安全、生命周期） |
| [Unsafe 类型推断](../analysis/unsafe-inference.md) | FFI 分配的启发式类型检测 |
| [Tracker API](../tracker-api/tracker.md) | 高级简化接口，含系统监控 |

---

## 按任务快速导航

**我想了解 MemScope 的工作原理：**
→ 从 [架构概览](../architecture/overview.md) 开始

**我想选择合适的追踪后端：**
→ 阅读 [捕获后端](../capture/backends.md)

**我想了解事件如何存储：**
→ 阅读 [EventStore](../event-store/eventstore.md)

**我想检测内存泄漏或其他问题：**
→ 阅读 [分析与检测器](../analysis/detectors.md)

**我想了解裸指针的类型推断：**
→ 阅读 [Unsafe 类型推断](../analysis/unsafe-inference.md)

**我想要一个简单的 API 用于快速追踪：**
→ 阅读 [Tracker API](../tracker-api/tracker.md)

---

## ⚠️ 关于旧版文档的说明

本目录中的模块专属页面（`single-threaded.md`、`multithread.md`、`async.md`、`hybrid.md`）引用的是**旧版 API**（`init()`、`track_var!`、`lockfree/` 模块），已被新架构取代。它们保留供参考，但不应作为主要文档来源。

当前架构请参见上方链接。
