---
name: Bug Report
about: Report a bug to help improve memscope-rs
title: '[BUG] '
labels: 'bug'
assignees: ''
---

## 🐛 Bug Description

**Describe the bug**
A clear and concise description of what the bug is.

**Expected behavior**
What you expected to happen.

**Actual behavior**
What actually happened.

## 🔍 Reproduction Steps

1. Go to '...'
2. Run command '....'
3. Set configuration '....'
4. See error

**Minimal reproducible example**
```rust
// Please provide a minimal code example that reproduces the issue
use memscope_rs::*;

fn main() {
    // Your reproduction code here
}
```

## 🖥️ Environment

**System Information:**
- OS: [e.g. Ubuntu 22.04, macOS 13.0, Windows 11]
- Rust version: [output of `rustc --version`]
- memscope-rs version: [e.g. 0.1.6]
- Cargo.toml dependencies: [relevant dependencies and versions]

**Tracking Strategy:**
- [ ] Core (single-threaded)
- [ ] Lock-free (multi-threaded)
- [ ] Async task-aware
- [ ] Unified backend

## 📋 Additional Context

**Error output**
```
Paste any error messages, stack traces, or log output here
```

**Performance impact** (if applicable)
- Memory usage: 
- CPU overhead: 
- Thread count: 

**Configuration** (if applicable)
```toml
# Your Cargo.toml [dependencies] section
[dependencies]
memscope-rs = { version = "0.1.6", features = ["..."] }
```

**Additional context**
Add any other context about the problem here, such as:
- Does this happen consistently or intermittently?
- Did this work in a previous version?
- Any workarounds you've found?

---

**Thank you for taking the time to report this issue! 🙏**