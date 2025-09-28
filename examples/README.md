# MemScope Unified Backend Examples

This directory contains comprehensive examples demonstrating the MemScope Unified Backend features.

## Files Overview

- **`unified_backend_demo.rs`** - Comprehensive API demonstration with all backend features
- **`simple_demo_app.rs`** - Simple application with different memory patterns for CLI testing
- **`demo_script.sh`** - Interactive shell script demonstrating CLI features
- **`README.md`** - This documentation file

## Quick Start

### 1. Build the Examples

```bash
# Build all examples
cargo build --examples

# Or build specific examples
cargo build --example unified_backend_demo
cargo build --example simple_demo_app
```

### 2. Run the Comprehensive API Demo

```bash
# Run the full API demonstration
cargo run --example unified_backend_demo
```

This will show:
- ✅ Quick start usage
- 🌍 Environment detection
- ⚙️ Manual strategy configuration
- 🧵 Multi-threaded tracking
- ⚡ Async tracking
- 🔧 System integration testing

### 3. Run CLI Demonstrations

```bash
# Make the script executable
chmod +x examples/demo_script.sh

# Run the full interactive demo
./examples/demo_script.sh

# Or run a quick demo
./examples/demo_script.sh quick
```

## Individual CLI Examples

### Basic Analysis Commands

```bash
# Auto-detection mode (recommended)
cargo run --bin memscope-rs -- analyze --mode auto \
    cargo run --example simple_demo_app single

# Use unified backend explicitly
cargo run --bin memscope-rs -- analyze --mode unified \
    cargo run --example simple_demo_app multi

# Use legacy backend for comparison
cargo run --bin memscope-rs -- analyze --mode legacy \
    cargo run --example simple_demo_app memory-intensive
```

### Strategy-Specific Analysis

```bash
# Single-threaded strategy
cargo run --bin memscope-rs -- analyze --strategy single-thread \
    cargo run --example simple_demo_app single

# Thread-local strategy for multi-threaded apps
cargo run --bin memscope-rs -- analyze --strategy thread-local \
    cargo run --example simple_demo_app multi

# Async strategy for async-like patterns
cargo run --bin memscope-rs -- analyze --strategy async \
    cargo run --example simple_demo_app async

# Hybrid strategy for complex scenarios
cargo run --bin memscope-rs -- analyze --strategy hybrid \
    cargo run --example simple_demo_app leak-simulation
```

### New Run Command Examples

```bash
# Basic run with async tracking
cargo run --bin memscope-rs -- run --track-async \
    cargo run --example simple_demo_app async

# Full featured run
cargo run --bin memscope-rs -- run \
    --track-async \
    --detailed-tracking \
    --performance-monitoring \
    --max-overhead 64 \
    cargo run --example simple_demo_app memory-intensive

# Constrained memory overhead
cargo run --bin memscope-rs -- run --max-overhead 16 \
    cargo run --example simple_demo_app single
```

### Export Format Examples

```bash
# Generate HTML report
cargo run --bin memscope-rs -- analyze --mode unified \
    --export html --output my_analysis \
    cargo run --example simple_demo_app multi

# Generate JSON data
cargo run --bin memscope-rs -- analyze --mode unified \
    --export json --output my_data \
    cargo run --example simple_demo_app single

# Generate SVG visualization
cargo run --bin memscope-rs -- analyze --mode unified \
    --export svg --output my_chart \
    cargo run --example simple_demo_app memory-intensive
```

### Performance Tuning Examples

```bash
# High precision (100% sampling)
cargo run --bin memscope-rs -- analyze --sample-rate 1.0 \
    cargo run --example simple_demo_app leak-simulation

# Balanced performance (80% sampling)
cargo run --bin memscope-rs -- analyze --sample-rate 0.8 \
    cargo run --example simple_demo_app multi

# Low overhead (50% sampling)
cargo run --bin memscope-rs -- analyze --sample-rate 0.5 \
    cargo run --example simple_demo_app memory-intensive
```

## Demo Application Modes

The `simple_demo_app` supports different memory patterns:

### Single-threaded Mode
```bash
cargo run --example simple_demo_app single
```
- Demonstrates basic memory operations
- Creates various data structures
- Shows allocation and deallocation patterns

### Multi-threaded Mode
```bash
cargo run --example simple_demo_app multi
```
- Spawns multiple worker threads
- Each thread performs independent memory operations
- Good for testing thread-local strategy

### Async Simulation Mode
```bash
cargo run --example simple_demo_app async
```
- Simulates async-like behavior with delays
- Shows task switching patterns
- Ideal for testing async strategy

### Memory-intensive Mode
```bash
cargo run --example simple_demo_app memory-intensive
```
- Creates large allocations progressively
- Demonstrates allocation/deallocation cycles
- Tests memory overhead limits

### Leak Simulation Mode
```bash
cargo run --example simple_demo_app leak-simulation
```
- Simulates memory leak patterns (controlled)
- Shows growing memory usage over time
- Demonstrates leak detection capabilities

## Real-world Usage Examples

### Web Server Analysis
```bash
# Analyze a Tokio-based web server
cargo run --bin memscope-rs -- analyze --strategy async \
    --export html --output web_server_analysis \
    cargo run --bin your_web_server

# Monitor with performance limits
cargo run --bin memscope-rs -- run --track-async \
    --performance-monitoring --max-overhead 32 \
    cargo run --bin your_web_server
```

### Batch Processing Analysis
```bash
# Analyze CPU-intensive batch job
cargo run --bin memscope-rs -- analyze --strategy thread-local \
    --sample-rate 0.8 --export json \
    your_batch_processor --input large_dataset.csv

# Monitor with detailed tracking
cargo run --bin memscope-rs -- run --detailed-tracking \
    --max-overhead 128 \
    your_batch_processor --parallel --jobs 8
```

### Game Engine Profiling
```bash
# Profile game loop with hybrid strategy
cargo run --bin memscope-rs -- analyze --strategy hybrid \
    --sample-rate 1.0 --export svg \
    your_game_engine --level test_map

# Real-time monitoring with strict limits
cargo run --bin memscope-rs -- run --performance-monitoring \
    --max-overhead 16 \
    your_game_client --graphics high
```

## Understanding Output

### JSON Output
```json
{
  "session_id": "session_12345",
  "duration_ms": 1250,
  "strategy_used": "AsyncOptimized",
  "total_allocations": 142,
  "peak_memory_mb": 8.5,
  "data_size_bytes": 2048
}
```

### HTML Reports
- Interactive memory usage charts
- Allocation timeline visualization  
- Strategy performance metrics
- Detailed allocation breakdown
- Clickable elements for exploration

### SVG Visualizations
- Memory usage over time graphs
- Allocation pattern diagrams
- Performance hotspot indicators
- Scalable vector graphics for presentations

## Troubleshooting

### Common Issues

1. **"Command not found" errors**
   ```bash
   # Make sure MemScope is built
   cargo build --release --bin memscope-rs
   
   # Or use the full path
   cargo run --bin memscope-rs -- --help
   ```

2. **High memory overhead warnings**
   ```bash
   # Reduce sampling rate
   --sample-rate 0.5
   
   # Lower overhead limit
   --max-overhead 32
   ```

3. **Async tracking not working**
   ```bash
   # Ensure async flag is set
   --track-async
   
   # Use async strategy explicitly
   --strategy async
   ```

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin memscope-rs -- analyze your_program

# Trace level for maximum detail
RUST_LOG=trace cargo run --bin memscope-rs -- run --detailed-tracking your_program
```

## Performance Expectations

| Mode | Memory Overhead | CPU Overhead | Best For |
|------|----------------|--------------|----------|
| `auto` | 2-5% | 1-3% | General applications |
| `unified` | 2-5% | 1-3% | Modern applications |
| `legacy` | 3-7% | 2-5% | Compatibility testing |

| Strategy | Memory Overhead | CPU Overhead | Best For |
|----------|----------------|--------------|----------|
| `single-thread` | 1-2% | 0.5-1% | Simple programs |
| `thread-local` | 2-4% | 1-2% | Multi-threaded apps |
| `async` | 2-3% | 1-2% | Async applications |
| `hybrid` | 3-5% | 2-3% | Complex mixed workloads |

## Next Steps

1. **Try with your own applications**:
   ```bash
   memscope analyze --mode unified your_program
   ```

2. **Explore different strategies**:
   ```bash
   memscope analyze --strategy auto your_program
   ```

3. **Generate reports for analysis**:
   ```bash
   memscope analyze --export html --output analysis your_program
   ```

4. **Read the full documentation**:
   - English: `docs/en/unified-backend-guide.md`
   - 中文: `docs/zh/unified-backend-guide.md`

## Contributing

Found issues or want to add more examples? Please:
1. Check existing issues in the repository
2. Add new example applications in this directory
3. Update this README with your examples
4. Submit a pull request

Happy memory tracking! 🚀