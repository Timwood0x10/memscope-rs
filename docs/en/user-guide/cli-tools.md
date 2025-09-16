# CLI Tools Usage Guide

memscope-rs provides Makefile-based command-line tools for convenient memory report generation and analysis.

## 🚀 Quick Start

### Prerequisites

Ensure you have built the project:

```bash
# Build project
cargo build --release
```

### Basic Usage Flow

```bash
# 1. Run your program to generate data
cargo run --example your_program

# 2. Use make command to generate HTML report
make html DIR=MemoryAnalysis/your_data

# 3. Open the generated report
open memory_report.html
```

## 📊 make html Command

The main command for generating interactive HTML memory analysis reports.

### Basic Syntax

```bash
make html [DIR=directory] [OUTPUT=filename] [BASE=basename] [OPTIONS]
```

### Parameters

| Parameter | Description | Default | Example |
|-----------|-------------|---------|---------|
| `DIR` | JSON files directory | `MemoryAnalysis/basic_usage` | `DIR=MemoryAnalysis/my_app` |
| `OUTPUT` | Output HTML filename | `memory_report.html` | `OUTPUT=my_report.html` |
| `BASE` | JSON files base name | `snapshot` | `BASE=my_analysis` |
| `VERBOSE` | Enable verbose output | None | `VERBOSE=1` |
| `DEBUG` | Enable debug mode | None | `DEBUG=1` |
| `PERFORMANCE` | Enable performance analysis | None | `PERFORMANCE=1` |

### Usage Examples

```bash
# Basic usage - use default settings
make html

# Specify custom directory
make html DIR=MemoryAnalysis/advanced_metrics_demo

# Use correct base name
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo

# Custom output filename
make html DIR=MemoryAnalysis/my_data OUTPUT=custom_report.html BASE=my_data

# Enable verbose output
make html DIR=MemoryAnalysis/my_data BASE=my_data VERBOSE=1

# Enable debug and performance analysis
make html DIR=MemoryAnalysis/my_data BASE=my_data DEBUG=1 PERFORMANCE=1

# Complete example
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo OUTPUT=advanced_report.html VERBOSE=1
```

### File Naming Rules

The HTML generator expects JSON files in this format:

```
{BASE}_memory_analysis.json
{BASE}_lifetime.json
{BASE}_performance.json
{BASE}_unsafe_ffi.json
{BASE}_complex_types.json
```

For example, if `BASE=my_analysis`, it needs:

- `my_analysis_memory_analysis.json`
- `my_analysis_lifetime.json`
- `my_analysis_performance.json`
- `my_analysis_unsafe_ffi.json`
- `my_analysis_complex_types.json`

## 🎯 Real Usage Examples

### Example 1: Basic Usage

```bash
# 1. Run basic example
cargo run --example basic_usage

# 2. Generate HTML report (note: basic_usage generates files with basic_usage prefix)
make html DIR=MemoryAnalysis/basic_usage BASE=basic_usage

# 3. View report
open memory_report.html
```

### Example 2: Advanced Multi-threaded Example

```bash
# 1. Run advanced example
cargo run --example advanced_metrics_demo

# 2. Generate HTML report (use correct base name)
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo

# 3. View report
open memory_report.html
```

### Example 3: Binary Export Example

```bash
# 1. Run binary export example
cargo run --example binary_export_demo

# 2. Generate HTML report
make html DIR=MemoryAnalysis/binary_demo_example BASE=binary_demo_example

# 3. View report
open memory_report.html
```

## 🔧 Other Make Commands

### Build and Test Commands

```bash
# Build project
make build          # Debug build
make release        # Release build

# Run tests
make test           # All tests
make test-unit      # Unit tests
make test-integration  # Integration tests
make test-performance  # Performance tests

# Code quality
make fmt            # Format code
make clippy         # Run Clippy checks
make audit          # Security audit
```

### Example Running Commands

```bash
# Run various examples
make run-basic                    # Basic usage example
make run-ownership               # Ownership patterns demo
make run-unsafe-ffi              # Unsafe/FFI demo
make run-improved-tracking       # Improved tracking showcase
make run-speed-test              # Speed test
make run-memory-stress           # Memory stress test
make run-lifecycle               # Lifecycle example

# Run binary tools
make run-benchmark               # Comprehensive performance benchmark
make run-simple-benchmark        # Simple benchmark
make run-core-performance        # Core performance evaluation
```

### HTML Related Commands

```bash
# HTML generation different modes
make html-verbose               # Verbose output mode
make html-debug                 # Debug mode
make html-performance           # Performance analysis mode
make html-validate              # Only validate JSON files

# Clean HTML files
make html-clean                 # Clean generated HTML files

# Get help
make html-help                  # Show detailed HTML command help
```

## 📈 Demo Workflows

### Quick Demo

```bash
# Complete demo workflow
make demo
# This executes: build → run basic example → generate HTML report
```

### Comprehensive Demo

```bash
# Full-featured demo
make demo-all
# This runs multiple examples and generates reports
```

### Performance Demo

```bash
# Performance evaluation demo
make perf-demo
# Runs performance benchmarks and generates analysis reports
```

## 🚨 Common Issues and Solutions

### Issue 1: JSON Files Not Found

```bash
# Error: No JSON files found in directory
# Solution: Check directory and base name are correct

# Check actual generated files
ls MemoryAnalysis/your_directory/

# Use correct base name
make html DIR=MemoryAnalysis/your_directory BASE=actual_base_name
```

### Issue 2: HTML Report Display Errors

```bash
# If HTML report charts show errors, base name might not match
# Ensure BASE parameter matches actual JSON file prefix

# For example, if files are advanced_metrics_demo_*.json
make html DIR=MemoryAnalysis/advanced_metrics_demo BASE=advanced_metrics_demo
```

### Issue 3: Permission Issues

```bash
# Ensure execution permissions
chmod +x target/release/memscope-rs

# Ensure output directory is writable
mkdir -p reports && chmod +w reports
```

## 💡 Best Practices

### 1. File Organization

```bash
# Recommended directory structure
MemoryAnalysis/
├── basic_usage/
│   ├── basic_usage_*.json
│   └── memory_report.html
├── advanced_demo/
│   ├── advanced_demo_*.json
│   └── memory_report.html
└── performance_test/
    ├── performance_test_*.json
    └── memory_report.html
```

### 2. Naming Conventions

```bash
# Keep consistent naming
cargo run --example my_feature
make html DIR=MemoryAnalysis/my_feature BASE=my_feature
```

### 3. Automation Scripts

```bash
#!/bin/bash
# Automated analysis script

EXAMPLE_NAME="advanced_metrics_demo"

echo "Running example: $EXAMPLE_NAME"
cargo run --example $EXAMPLE_NAME

echo "Generating HTML report"
make html DIR=MemoryAnalysis/$EXAMPLE_NAME BASE=$EXAMPLE_NAME VERBOSE=1

echo "Report generated: memory_report.html"
open memory_report.html
```

### 4. Batch Processing

```bash
#!/bin/bash
# Batch report generation

for dir in MemoryAnalysis/*/; do
    if [ -d "$dir" ]; then
        dirname=$(basename "$dir")
        echo "Processing directory: $dirname"
        make html DIR="$dir" BASE="$dirname" OUTPUT="${dirname}_report.html"
    fi
done
```

## 🔗 Related Documentation

- [Export Formats Guide](export-formats.md) - Understand various export formats
- [Quick Start Guide](../getting-started/quick-start.md) - Basic usage guide
- [Concurrent Analysis Example](../examples/concurrent-analysis.md) - Multi-threaded analysis examples

## 📋 Command Quick Reference

| Task | Command |
|------|---------|
| Run basic example | `cargo run --example basic_usage` |
| Generate HTML report | `make html DIR=path BASE=name` |
| Run advanced example | `cargo run --example advanced_metrics_demo` |
| Clean HTML files | `make html-clean` |
| Get help | `make html-help` |
| Quick demo | `make demo` |
| Build project | `make build` |
| Run tests | `make test` |

---

Use these command-line tools to make memory analysis more efficient and automated! 🎯