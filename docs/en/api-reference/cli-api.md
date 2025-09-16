# CLI API Reference

memscope-rs provides powerful command-line tools for memory analysis, report generation, and data processing.

## 🚀 Overview

The memscope CLI tool provides the following main functionality:

- **analyze** - Analyze program memory usage
- **report** - Generate analysis reports from existing data
- **html-from-json** - Generate interactive HTML reports from JSON files
- **test** - Run enhanced memory tests

## 📋 Basic Usage

```bash
# Basic syntax
memscope <SUBCOMMAND> [OPTIONS]

# View help
memscope --help
memscope <SUBCOMMAND> --help
```

## 🔍 analyze Command

Analyze program memory usage.

### Syntax

```bash
memscope analyze <COMMAND> [OPTIONS]
```

### Arguments

- `<COMMAND>` - Command to execute and analyze (required)

### Options

- `--export <FORMAT>` - Export format: json, html, binary, all
  - Default: `html`
  - Values: `json`, `html`, `binary`, `all`

- `--output <PATH>` - Output file path (without extension)
  - Default: `memory_analysis`

### Examples

```bash
# Analyze Rust program
memscope analyze cargo run --release

# Analyze and export as JSON
memscope analyze --export json --output my_analysis cargo run

# Analyze and export all formats
memscope analyze --export all ./my_program

# Analyze program with arguments
memscope analyze ./my_program arg1 arg2 --flag

# Analyze Python program
memscope analyze python my_script.py

# Analyze Node.js program
memscope analyze node app.js
```

### Output Files

Depending on the export format, the following files will be generated:

```
MemoryAnalysis/
├── my_analysis_memory_analysis.json    # Basic memory analysis
├── my_analysis_lifetime.json           # Lifecycle data
├── my_analysis_performance.json        # Performance metrics
├── my_analysis_unsafe_ffi.json         # Unsafe/FFI tracking
├── my_analysis_complex_types.json      # Complex type analysis
├── my_analysis.html                     # Interactive HTML report
├── my_analysis.svg                      # SVG visualization
└── my_analysis.memscope                 # Binary format
```

## 📊 report Command

Generate memory analysis reports from existing data.

### Syntax

```bash
memscope report --input <INPUT_FILE> --output <OUTPUT_FILE> [OPTIONS]
```

### Arguments

- `--input <INPUT_FILE>` - Input JSON file path (required)
- `--output <OUTPUT_FILE>` - Output report file path (required)

### Options

- `--format <FORMAT>` - Output format
  - Default: `html`
  - Values: `html`, `svg`, `pdf`

### Examples

```bash
# Generate HTML report from JSON
memscope report --input analysis.json --output report.html

# Generate SVG visualization
memscope report --input analysis.json --output chart.svg --format svg

# Use custom template
memscope report --input analysis.json --output custom_report.html --template my_template.html
```

## 🌐 html-from-json Command

Generate interactive HTML reports from exported JSON files, significantly faster than direct tracker export.

### Syntax

```bash
memscope html-from-json --input-dir <DIR> --output <HTML_FILE> [OPTIONS]
```

### Arguments

- `--input-dir <DIR>` - Input directory containing JSON files (required)
- `--output <HTML_FILE>` - Output HTML file path (required)

### Options

- `--base-name <NAME>` - Base name for JSON files
  - Default: `snapshot`

- `--verbose` - Enable verbose output with detailed progress information

- `--debug` - Enable debug mode with detailed logging and timing information

- `--performance` - Enable performance analysis mode with comprehensive timing and memory tracking

- `--validate-only` - Only validate JSON files without generating HTML

### Examples

```bash
# Basic usage
memscope html-from-json --input-dir MemoryAnalysis/my_analysis --output report.html

# Use custom base name
memscope html-from-json --input-dir ./data --output analysis.html --base-name my_snapshot

# Verbose mode
memscope html-from-json --input-dir ./data --output report.html --verbose

# Debug mode
memscope html-from-json --input-dir ./data --output report.html --debug --performance

# Validate JSON files only
memscope html-from-json --input-dir ./data --validate-only

# Process large datasets
memscope html-from-json --input-dir ./large_dataset --output big_report.html --performance
```

### Performance Benefits

The html-from-json command has significant performance advantages over direct HTML export:

| Operation | Direct Export | html-from-json | Performance Gain |
|-----------|---------------|----------------|------------------|
| Small datasets (< 1MB) | 2-5 seconds | 0.5-1 second | 2-5x |
| Medium datasets (1-10MB) | 10-30 seconds | 2-5 seconds | 5-6x |
| Large datasets (> 10MB) | 60+ seconds | 5-15 seconds | 4-12x |

## 🧪 test Command

Run enhanced memory tests.

### Syntax

```bash
memscope test [OPTIONS]
```

### Options

- `--output <PATH>` - Output path
  - Default: `enhanced_memory_test`

### Examples

```bash
# Run basic test
memscope test

# Specify output path
memscope test --output my_test_results

# Run test with verbose output
memscope test --output test_2024 --verbose
```

## 🔧 Global Options

All commands support the following global options:

- `--help` - Show help information
- `--version` - Show version information

## 📁 Output Directory Structure

memscope creates output files in the `MemoryAnalysis/` directory by default:

```
MemoryAnalysis/
├── <base_name>/
│   ├── <base_name>_memory_analysis.json
│   ├── <base_name>_lifetime.json
│   ├── <base_name>_performance.json
│   ├── <base_name>_unsafe_ffi.json
│   ├── <base_name>_complex_types.json
│   └── <base_name>.memscope
├── <base_name>.html
├── <base_name>.svg
└── logs/
    └── memscope.log
```

## 🌍 Environment Variables

Configure memscope behavior through environment variables:

```bash
# Enable memory tracking
export MEMSCOPE_ENABLED=1

# Auto export
export MEMSCOPE_AUTO_EXPORT=1

# Export format
export MEMSCOPE_EXPORT_FORMAT=json

# Export path
export MEMSCOPE_EXPORT_PATH=my_analysis

# Auto tracking
export MEMSCOPE_AUTO_TRACK=1

# Wait for completion
export MEMSCOPE_WAIT_COMPLETION=1

# Log level
export RUST_LOG=memscope_rs=debug
```

## 📊 Performance Comparison

Performance characteristics of different commands:

### analyze Command
- **Overhead**: 5-15% program execution time
- **Memory**: Additional 10-50MB memory usage
- **Use case**: Development and testing phases

### html-from-json Command
- **Speed**: 4-12x faster than direct HTML export
- **Memory**: Low memory footprint, supports large files
- **Use case**: Production report generation

### report Command
- **Speed**: Fast report generation
- **Flexibility**: Supports multiple output formats
- **Use case**: Automated reporting workflows

## 🔍 Advanced Usage

### 1. Batch Analysis

```bash
#!/bin/bash
# Batch analyze multiple programs

programs=("./app1" "./app2" "./app3")

for program in "${programs[@]}"; do
    echo "Analyzing $program..."
    memscope analyze --export all --output "analysis_$(basename $program)" "$program"
done

# Generate summary report
memscope html-from-json --input-dir MemoryAnalysis --output summary.html
```

### 2. Continuous Integration

```yaml
# .github/workflows/memory-analysis.yml
name: Memory Analysis

on: [push, pull_request]

jobs:
  memory-analysis:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install memscope-rs
        run: cargo install memscope-rs
      
      - name: Run memory analysis
        run: |
          memscope analyze --export json cargo test
          memscope html-from-json --input-dir MemoryAnalysis --output memory-report.html
      
      - name: Upload report
        uses: actions/upload-artifact@v2
        with:
          name: memory-analysis-report
          path: memory-report.html
```

### 3. Performance Monitoring

```bash
#!/bin/bash
# Performance monitoring script

# Run analysis
echo "Starting memory analysis..."
time memscope analyze --export binary --output perf_test ./my_app

# Generate fast report
echo "Generating HTML report..."
time memscope html-from-json --input-dir MemoryAnalysis/perf_test --output perf_report.html --performance

# Show file sizes
echo "Output files:"
ls -lh MemoryAnalysis/perf_test/
ls -lh perf_report.html
```

## ❌ Error Handling

### Common Errors and Solutions

#### 1. "Command not found"
```bash
# Ensure memscope is in PATH
which memscope

# If not found, add to PATH or use full path
export PATH="$HOME/.cargo/bin:$PATH"
```

#### 2. "Permission denied"
```bash
# Check output directory permissions
ls -la MemoryAnalysis/

# Create directory and set permissions
mkdir -p MemoryAnalysis
chmod 755 MemoryAnalysis
```

#### 3. "JSON files not found"
```bash
# Check if files exist
ls -la MemoryAnalysis/my_analysis/

# Verify file name pattern
memscope html-from-json --input-dir MemoryAnalysis/my_analysis --validate-only
```

#### 4. "Out of memory"
```bash
# For large files, use performance mode
memscope html-from-json --input-dir ./large_data --output report.html --performance

# Or increase system memory limits
ulimit -v 8388608  # 8GB
```

## 🔧 Command Chaining

### Sequential Analysis Pipeline

```bash
# Complete analysis pipeline
memscope analyze --export json ./my_app && \
memscope html-from-json --input-dir MemoryAnalysis/memory_analysis --output final_report.html --performance && \
echo "Analysis complete! Report: final_report.html"
```

### Conditional Processing

```bash
# Only generate HTML if JSON export succeeds
if memscope analyze --export json ./my_app; then
    echo "JSON export successful, generating HTML..."
    memscope html-from-json --input-dir MemoryAnalysis/memory_analysis --output report.html
else
    echo "Analysis failed!"
    exit 1
fi
```

## 📊 Output Format Comparison

| Format | Size | Generation Time | Use Case |
|--------|------|----------------|----------|
| **JSON** | Medium | Fast | Data processing, APIs |
| **HTML** | Large | Medium | Interactive analysis |
| **SVG** | Small | Fast | Quick visualization |
| **Binary** | Small | Very Fast | Storage, automation |

## 🎯 Best Practices

### 1. Development Workflow
```bash
# Quick development check
memscope analyze --export json cargo test

# Detailed analysis for debugging
memscope analyze --export all ./debug_build
memscope html-from-json --input-dir MemoryAnalysis/memory_analysis --output debug_report.html --debug
```

### 2. Production Monitoring
```bash
# Lightweight production analysis
memscope analyze --export binary ./production_app
memscope html-from-json --input-dir MemoryAnalysis/memory_analysis --output prod_report.html --performance
```

### 3. CI/CD Integration
```bash
# Fast CI analysis
memscope analyze --export json cargo test --release
if [ $? -eq 0 ]; then
    memscope html-from-json --input-dir MemoryAnalysis/memory_analysis --output ci_report.html
    # Upload report to artifact storage
fi
```

## 🔗 Related Documentation

- [Tracking API Reference](tracking-api.md) - In-program tracking interfaces
- [Export API Reference](export-api.md) - Data export functionality
- [CLI Tools Guide](../user-guide/cli-tools.md) - CLI usage guide
- [Export Formats Guide](../user-guide/export-formats.md) - Output format details

---

CLI tools make memory analysis simple and efficient! 🎯