# Benchmark Usage Guide

## Quick Start

### Run Quick Mode (Recommended)
```bash
make bench-quick
```
- **Runtime**: ~5 minutes
- **Use Case**: Daily development, quick validation
- **Configuration**: 10 samples, 100ms warm-up, 500ms measurement

### Run Full Mode
```bash
make bench
```
- **Runtime**: ~60 minutes
- **Use Case**: Pre-release validation, performance regression detection
- **Configuration**: 100 samples, 3s warm-up, 5s measurement

## Categorized Execution

### Core Functionality Tests
```bash
# Tracker core functionality
make bench-tracker

# Backend performance
make bench-backend

# Type classification
make bench-classification
```

### Concurrency Tests
```bash
# Concurrent tracking
make bench-concurrent

# Stress tests
make bench-stress
```

### Specific Scenario Tests
```bash
# IO operations
make bench-io

# Memory pressure
make bench-pressure

# Real-world scenarios
make bench-scenario
```

### New Tests
```bash
# Memory allocator comparison
make bench-allocator

# Long-term stability
make bench-stability

# Edge cases
make bench-edge

# Performance regression detection
make bench-regression
```

## Result Saving

### Auto-save Results
```bash
make bench-save
```
- Runs quick mode and saves results
- Result file: `benches/benchmark_results_quick_YYYYMMDD_HHMMSS.log`
- Includes timestamp for easy historical comparison

### Manual Save
```bash
QUICK_BENCH=1 cargo bench 2>&1 | tee benches/my_benchmark.log
```

## Environment Variables

### QUICK_BENCH
Controls benchmark execution mode:

```bash
# Quick mode
QUICK_BENCH=1 cargo bench

# Full mode (default)
cargo bench
```

### Custom Configuration
```bash
# Custom sample size
QUICK_BENCH=1 SAMPLE_SIZE=50 cargo bench

# Custom warm-up time (milliseconds)
QUICK_BENCH=1 WARM_UP_TIME=200 cargo bench

# Custom measurement time (milliseconds)
QUICK_BENCH=1 MEASUREMENT_TIME=1000 cargo bench
```

## Performance Reports

### View Analysis Reports
```bash
# English version
cat benches/PERFORMANCE_ANALYSIS_EN.md

# Chinese version
cat benches/PERFORMANCE_ANALYSIS.md
```

### Historical Result Comparison
```bash
# View all historical results
ls -lh benches/benchmark_results_*.log

# Compare two results
diff benches/benchmark_results_quick_20260412_193000.log \
     benches/benchmark_results_quick_20260412_200000.log
```

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Benchmark

on: [pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      # Quick mode (PR check)
      - name: Run quick benchmarks
        run: make bench-quick
      
      # Save results
      - name: Save benchmark results
        uses: actions/upload-artifact@v2
        with:
          name: benchmark-results
          path: benches/benchmark_results_*.log
```

### Periodic Full Testing
```yaml
name: Full Benchmark

on:
  schedule:
    - cron: '0 2 * * 0'  # Every Sunday at 2 AM

jobs:
  full-benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      
      - name: Run full benchmarks
        run: make bench
```

## Performance Baselines

### Key Metrics
| Operation | Quick Mode Latency | Full Mode Latency | Target |
|-----------|-------------------|-------------------|--------|
| Single Track | ~535 ns | ~535 ns | < 1 µs |
| Backend Alloc | ~21 ns | ~21 ns | < 50 ns |
| Stats Query | ~250 ns | ~250 ns | < 500 ns |
| Small-scale Analysis | ~540 µs (1k) | ~540 µs (1k) | < 1 ms |
| Large-scale Analysis | ~35 ms (50k) | ~35 ms (50k) | < 50 ms |

### Performance Regression Detection
```bash
# Run regression detection
make bench-regression

# Will show warnings if performance drops beyond threshold
```

## Troubleshooting

### Runtime Too Long
```bash
# Check if quick mode is enabled
echo $QUICK_BENCH

# Ensure environment variable is set correctly
QUICK_BENCH=1 make bench-quick
```

### Insufficient Memory
```bash
# Reduce concurrent thread count
cargo bench -- concurrent_benches --test-threads=4

# Use smaller dataset
QUICK_BENCH=1 cargo bench
```

### Abnormal Results
```bash
# Clean cache and re-run
cargo clean
make bench-quick
```

## Best Practices

### Development Phase
1. Use `make bench-quick` for quick validation
2. Monitor key metric changes
3. Periodically save results for comparison

### Pre-release
1. Run `make bench` for full testing
2. Compare with historical baseline data
3. Check performance regression reports

### Production Environment
1. Use sampling mode to reduce overhead
2. Periodically run stability tests
3. Monitor long-term performance trends

## Related Documentation

- [Performance Analysis Report (English)](PERFORMANCE_ANALYSIS_EN.md)
- [Performance Analysis Report (Chinese)](PERFORMANCE_ANALYSIS.md)
- [Benchmark Code](comprehensive_benchmarks.rs)

## FAQ

**Q: Is there a big difference between quick mode and full mode results?**  
A: Very small difference (<5%), quick mode is sufficient for daily development validation.

**Q: How to choose execution mode?**  
A: Use quick mode for daily development, full mode before release.

**Q: Can result files be deleted?**  
A: Yes, but it's recommended to keep historical data for performance trend analysis.

**Q: How to add new benchmarks?**  
A: Add new functions in `comprehensive_benchmarks.rs`, then register in `criterion_main!`.

## Performance Optimization Tips

### For Faster Execution
1. Use quick mode for development
2. Run specific test categories instead of all tests
3. Reduce sample size for quick checks

### For More Accurate Results
1. Use full mode for final validation
2. Run multiple times and compare results
3. Monitor system resources during execution

### For CI/CD
1. Use quick mode in PR checks
2. Use full mode in scheduled jobs
3. Save and compare results over time

## Advanced Usage

### Run Specific Benchmark Groups
```bash
# Run only tracker benchmarks
cargo bench -- tracker_benches

# Run only concurrent benchmarks
cargo bench -- concurrent_benches

# Run only stress tests
cargo bench -- stress_benches
```

### Filter by Name
```bash
# Run benchmarks containing "alloc"
cargo bench -- alloc

# Run benchmarks containing "concurrent"
cargo bench -- concurrent
```

### Save with Metadata
```bash
# Save with git commit info
QUICK_BENCH=1 cargo bench 2>&1 | \
  tee benches/benchmark_$(date +%Y%m%d_%H%M%S)_$(git rev-parse --short HEAD).log
```

## Performance Monitoring

### Key Metrics to Watch
- **Tracking overhead**: Should remain < 1 µs
- **Backend latency**: Should remain < 50 ns
- **Analysis time**: Should scale linearly
- **Concurrency efficiency**: Should be > 100% at 4-8 threads

### Alert Thresholds
- Tracking overhead > 2 µs: ⚠️ Warning
- Backend latency > 100 ns: ⚠️ Warning
- Analysis time > 2x baseline: 🔴 Critical
- Concurrency efficiency < 50%: ⚠️ Warning

## Support

For issues or questions:
1. Check this guide first
2. Review performance analysis reports
3. Compare with historical results
4. Check system resources and configuration

---

**Last Updated**: 2026-04-12  
**Maintainer**: memscope-rs team
