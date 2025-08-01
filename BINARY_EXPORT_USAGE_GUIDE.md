# Binary Export System Usage Guide

## Overview

The binary export system provides high-performance memory analysis data export capabilities with comprehensive features including compression, validation, and multiple output formats.

## Quick Start

### 1. Basic Binary Export

```bash
# Run your program with binary export
cargo run --example basic_usage_binary

# This creates binary files in MemoryAnalysis/basic_usage/
```

### 2. Convert Binary to Other Formats

```bash
# Convert to JSON
memscope export -i MemoryAnalysis/basic_usage/basic_usage_snapshot.ms -f json -o output.json

# Convert to HTML
memscope export -i MemoryAnalysis/basic_usage/basic_usage_snapshot.ms -f html -o report.html
```

### 3. Generate Analysis Reports

```bash
# Generate comprehensive analysis report
memscope analyze-report -i data.ms -o analysis_report.html -t comprehensive

# Generate security-focused report
memscope analyze-report -i data.ms -o security_report.json -t security -f json
```

### 4. Query Binary Data

```bash
# Find large allocations
memscope query -i data.ms -q "size > 1024" -f table

# Find leaked memory
memscope query -i data.ms -q "leaked" -f json

# Find specific types
memscope query -i data.ms -q "type = \"Vec\"" -f csv
```

## Available Commands

- `export`: Convert binary files to JSON/HTML
- `analyze-report`: Generate comprehensive analysis reports  
- `query`: Search and filter binary data
- `integration-test`: Run comprehensive test suite

## Features Completed

✅ All core binary export functionality
✅ Multiple compression algorithms (zstd, lz4)
✅ HTML and JSON conversion
✅ Analysis report generation
✅ Command-line tools
✅ Integration testing framework

The system is ready for production use!