# Installation Guide

This guide will help you properly install and configure memscope-rs in different environments.

## 🚀 Quick Installation

### Basic Installation
Add to your `Cargo.toml`:

```toml
[dependencies]
memscope-rs = "0.1.10"
```

This enables default features including:
- `tracking-allocator` - Global allocator tracking
- All core functionality

### Minimal Installation
For basic functionality only:

```toml
[dependencies]
memscope-rs = { version = "0.1.10", default-features = false }
```

## 🎛️ Feature Configuration

### Available Features

| Feature | Default | Description | Use Case |
|---------|---------|-------------|----------|
| `tracking-allocator` | ✅ | Global allocator tracking | Automatic memory tracking |
| `backtrace` | ❌ | Call stack tracing | Detailed debugging |
| `derive` | ❌ | Derive macro support | Custom type tracking |
| `test` | ❌ | Testing utilities | Unit testing |

### Feature Combinations

**Full functionality**:
```toml
[dependencies]
memscope-rs = {
    version = "0.1.10",
    features = ["tracking-allocator", "backtrace", "derive"]
}
```

**Performance optimized**:
```toml
[dependencies]
memscope-rs = {
    version = "0.1.10",
    features = ["tracking-allocator"]
}
```

**Debug configuration**:
```toml
[dependencies]
memscope-rs = {
    version = "0.1.10",
    features = ["tracking-allocator", "backtrace"]
}
```

## 🏗️ Environment Setup

### Standard Rust Project
```toml
# Cargo.toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"

[dependencies]
memscope-rs = "0.1.10"
```

```rust
// src/main.rs
use memscope_rs::{track_var};

fn main() {
    let memscope = memscope_rs::MemScope::new();

    let data = vec![1, 2, 3];
    track_var!(data);

    memscope.export_html("analysis.html").unwrap();
}
```

### Library Project
```toml
[dependencies]
memscope-rs = { version = "0.1.10", optional = true }

[features]
default = []
memory-analysis = ["memscope-rs"]
```

```rust
#[cfg(feature = "memory-analysis")]
use memscope_rs::track_var;

pub fn process_data(data: Vec<i32>) -> Vec<i32> {
    #[cfg(feature = "memory-analysis")]
    track_var!(data);

    data.into_iter().map(|x| x * 2).collect()
}
```

### no_std Environment
```toml
[dependencies]
memscope-rs = {
    version = "0.1.10",
    default-features = false,
    features = []
}
```

```rust
#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use memscope_rs::MemoryTracker;

fn main() {
    let tracker = MemoryTracker::new();
    // Manual tracking mode...
}
```

## 🔧 Development Environment

### VS Code Configuration
Create `.vscode/settings.json`:

```json
{
    "rust-analyzer.cargo.features": [
        "tracking-allocator",
        "backtrace"
    ],
    "rust-analyzer.checkOnSave.command": "clippy"
}
```

### Cargo Configuration
Create `.cargo/config.toml`:

```toml
[build]
rustflags = ["-C", "debug-assertions=on"]

[env]
RUST_LOG = { value = "memscope_rs=debug", force = true }
RUST_BACKTRACE = { value = "1", force = true }

[alias]
analyze = "run --features backtrace --"
test-memory = "test --features test --"
```

### Environment Variables
```bash
# Development
export RUST_LOG=memscope_rs=debug
export RUST_BACKTRACE=1

# Production
export RUST_LOG=memscope_rs=info
export MEMSCOPE_OUTPUT_DIR=/var/log/memscope
```

## 🐳 Containerized Deployment

### Dockerfile
```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --features "tracking-allocator,backtrace"

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/my-app /usr/local/bin/

RUN mkdir -p /var/log/memscope
ENV MEMSCOPE_OUTPUT_DIR=/var/log/memscope

CMD ["my-app"]
```

## 🧪 Testing Setup

### Unit Tests
```toml
[dev-dependencies]
memscope-rs = { version = "0.1.10", features = ["test"] }
```

```rust
#[cfg(test)]
mod tests {
    use memscope_rs::{track_var};

    #[test]
    fn test_memory_tracking() {
        let memscope = memscope_rs::MemScope::new();

        let data = vec![1, 2, 3];
        track_var!(data);

        let stats = memscope.summary().unwrap();
        assert!(stats.total_allocations > 0);
    }
}
```

### Integration Tests
```rust
// tests/integration_test.rs
use memscope_rs::{track_var};

#[test]
fn integration_test() {
    let memscope = memscope_rs::MemScope::new();

    let large_data = vec![0; 1024 * 1024];
    track_var!(large_data);

    assert!(memscope.export_json("integration_test").is_ok());
}
```

## 🚀 Performance Configuration

### Release Build
```toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### Conditional Compilation
```rust
// Only enable tracking in debug mode
#[cfg(debug_assertions)]
use memscope_rs::{init, track_var};

#[cfg(debug_assertions)]
macro_rules! debug_track {
    ($var:expr) => { track_var!($var) };
}

#[cfg(not(debug_assertions))]
macro_rules! debug_track {
    ($var:expr) => {};
}

fn main() {
    #[cfg(debug_assertions)]
    init();
    
    let data = vec![1, 2, 3];
    debug_track!(data);
}
```

## 🔍 Installation Verification

### Quick Verification Script
```rust
// verify_installation.rs
use memscope_rs::{track_var};

fn main() {
    println!("🔍 Verifying memscope-rs installation...");

    // 1. Initialization test
    let memscope = match std::panic::catch_unwind(|| memscope_rs::MemScope::new()) {
        Ok(m) => {
            println!("✅ Initialization successful");
            m
        }
        Err(_) => {
            println!("❌ Initialization failed");
            return;
        }
    };

    // 2. Tracking test
    let test_data = vec![1, 2, 3];
    track_var!(test_data);
    println!("✅ Variable tracking successful");

    // 3. Statistics test
    match memscope.summary() {
        Ok(stats) => {
            println!("✅ Statistics successful: {} total allocations", stats.total_allocations);
        }
        Err(e) => {
            println!("❌ Statistics failed: {}", e);
            return;
        }
    }

    // 4. Export test
    match memscope.export_json("verification_test") {
        Ok(_) => println!("✅ JSON export successful"),
        Err(e) => println!("⚠️ JSON export failed: {}", e),
    }

    println!("🎉 memscope-rs installation verification complete!");
}
```

Run verification:
```bash
cargo run --bin verify_installation
```

## 📋 Installation Checklist

- [ ] ✅ Added correct dependency to Cargo.toml
- [ ] ✅ Selected appropriate feature configuration
- [ ] ✅ Correctly imported necessary macros and functions
- [ ] ✅ Called init() at the start of main() function
- [ ] ✅ Verification script runs successfully
- [ ] ✅ Can generate and view export files
- [ ] ✅ Test cases pass

## 🆘 Common Installation Issues

If you encounter problems, check [Troubleshooting Guide](../user-guide/troubleshooting.md) or:

1. Confirm Rust version >= 1.70
2. Check network connection and crates.io access
3. Clean build cache: `cargo clean`
4. Update dependencies: `cargo update`

After successful installation, continue with [Quick Start Guide](quick-start.md)! 🎯