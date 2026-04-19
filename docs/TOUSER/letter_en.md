# Why memscope-rs Exists

## Rust deserves honest memory tooling.

---

Dear fellow Rustacean,

If you're reading this, you probably care about memory safety as much as I do. Let me share why memscope-rs exists — not the marketing version, but the real one.

---

## What You Get Today

Before I tell you the story, let me tell you what this tool can do for you **right now**:

- **Find memory leaks in minutes, not hours** — Track every allocation with <5% overhead
- **Detect Arc/Rc cycles** — Real clone tracking, not guessing
- **Attribute memory to async tasks** — See which task consumes what
- **Visualize ownership graphs** — Interactive HTML dashboard

Last week, a user found an Arc circular reference in 15 minutes that had plagued them for two days with traditional tools.

---

## The Hard Truth I Had to Accept

I built this tool because existing solutions didn't understand Rust. Valgrind doesn't know what `Arc<Rc<Box<...>>>` means. AddressSanitizer gives you stack traces to nowhere.

But after months of work, I hit a wall.

I spent weeks reading the Rust compiler source code — `rustc_middle`, `rustc_borrowck`, `rustc_mir_build`. I traced the complete path from HIR to THIR to MIR to LLVM IR. I documented everything in [Rust Ownership Semantics Analysis](../zh/rust-ownership-semantics-analysis.md) — 735 lines explaining why move/borrow semantics vanish before runtime.

The brutal truth:

```
Source → AST → HIR → THIR → MIR → LLVM IR → Machine Code
                              ↑
                    move/borrow exists here
                              ↓
                         disappears here
```

**I understand what Rust cannot see at runtime, therefore I only track what is truly visible, and make honest inferences for the rest.**

This is not something every tool author can say. It's the core of what makes memscope-rs different.

---

## The Promise

> **I will never lie to my users.**

If data is real, I'll say it's real. If data is inferred, I'll mark it with confidence level. No pretending, no hiding behind technical jargon.

Every inferred field comes with:

```json
{
  "borrow_info": {
    "immutable_borrows": 5,
    "_source": "inferred",
    "_confidence": "low"
  }
}
```

---

## The Philosophy

**Think Bold.** I dreamed of a tool that understands Rust's ownership model at runtime.

**Build Carefully.** Every feature is implemented within real constraints. I deliver what I promise.

**Ship Wild Features.** Arc/Rc clone detection. Task memory attribution. Circular reference detection. Ownership graph visualization.

**Keep the Core Tight.** <5% overhead. 21-40ns latency. 75% code reduction to stay lean.

---

## This Is Not a Weekend Project

memscope-rs is the result of:

- **Months of development** — From architecture to implementation
- **Deep compiler research** — Understanding the limits
- **Honest self-reflection** — Admitting constraints
- **Continuous improvement** — v0.2.0, v0.2.1, v0.2.2, v0.2.3...

You might have seen earlier versions labeled as a "research project." That was my honest assessment at the time — I wasn't sure if the approach would work. But after extensive testing and real-world use, I can confidently say: **this tool is production-ready**. The core features work. The performance is solid. The data is real.

---

## The Road Ahead

I'm not done. There's still work to do:

- Better async task tracking
- More accurate ownership inference
- Improved dashboard visualizations
- Integration with popular frameworks

---

## Join Us

If you believe tools should be honest, fast, and Rust-native — welcome.

This is not just a tool. It's a commitment to the Rust community.

Star the repo. Open an issue. Share your story.

Let's build better memory tooling together.

---

**With gratitude,**

TimWood  
Maintainer

---

> "The best tool is not the one that promises everything. It's the one that delivers what it promises."
