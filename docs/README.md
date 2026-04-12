# Documentation Index

Welcome to the memscope-rs documentation. This directory contains comprehensive guides, architecture documentation, and performance analysis reports.

## 📚 Documentation Structure

### Architecture & Design

- **[Architecture Overview](ARCHITECTURE.md)** - Detailed architecture documentation with mermaid diagrams
  - Three-Layer Object Model
  - Unified Node Identity System
  - Modular Backend Architecture
  - Event-Driven Architecture
  - Comparison with master branch

### Performance & Benchmarks

#### Chinese (中文)
- **[性能分析报告](PERFORMANCE_ANALYSIS.md)** - 详细的性能分析报告
  - 核心性能指标
  - 后端性能对比
  - 并发性能分析
  - 性能瓶颈分析
  - 优化建议

- **[Benchmark 使用指南](BENCHMARK_GUIDE.md)** - Benchmark 使用说明
  - 快速开始
  - 分类运行
  - 结果保存
  - CI/CD 集成
  - 最佳实践

#### English
- **[Performance Analysis Report](PERFORMANCE_ANALYSIS_EN.md)** - Detailed performance analysis
  - Core performance metrics
  - Backend performance comparison
  - Concurrency analysis
  - Bottleneck analysis
  - Optimization recommendations

- **[Benchmark Guide](BENCHMARK_GUIDE_EN.md)** - Benchmark usage guide
  - Quick start
  - Categorized execution
  - Result saving
  - CI/CD integration
  - Best practices

### Language-Specific Documentation

#### Chinese (中文)
- **[API 指南](zh/api_guide.md)** - API 使用指南
- **[模块文档](zh/modules/)** - 各模块详细文档
  - [分析器模块](zh/modules/analyzer.md)
  - [视图模块](zh/modules/view.md)

#### English
- **[API Guide](en/api_guide.md)** - API usage guide
- **[Module Documentation](en/modules/)** - Detailed module docs
  - [Analysis Module](en/modules/analysis.md)
  - [Analyzer Module](en/modules/analyzer.md)
  - [Tracker Module](en/modules/tracker.md)
  - [Tracking Module](en/modules/tracking.md)
  - [View Module](en/modules/view.md)
  - [Capture Module](en/modules/capture.md)
  - [Render Engine](en/modules/render_engine.md)
  - [Core Module](en/modules/core.md)

## 🚀 Quick Links

### For New Users
1. Start with [Architecture Overview](ARCHITECTURE.md) to understand the system
2. Read [API Guide](en/api_guide.md) for basic usage
3. Check [Performance Analysis](PERFORMANCE_ANALYSIS_EN.md) for performance expectations

### For Developers
1. Review [Architecture Overview](ARCHITECTURE.md) for system design
2. Run benchmarks with [Benchmark Guide](BENCHMARK_GUIDE_EN.md)
3. Check module documentation for implementation details

### For Performance Analysis
1. Read [Performance Analysis Report](PERFORMANCE_ANALYSIS_EN.md)
2. Run your own benchmarks with `make bench-quick`
3. Compare results with baseline data

## 📊 Test Environment

All performance benchmarks were conducted on:
- **Hardware**: Apple M3 Max
- **OS**: macOS Sonoma
- **Rust**: 1.85+

## 🔍 Key Performance Highlights

### Backend Performance (M3 Max)
- **Core Backend**: 21 ns allocation latency
- **Async Backend**: 21 ns allocation latency
- **Lockfree Backend**: 40 ns allocation latency
- **Unified Backend**: 40 ns allocation latency

### Tracking Overhead
- **Single Track**: 528 ns - 4.72 µs (depending on size)
- **Batch Track**: 1.85 Melem/s throughput
- **Stats Query**: 250 ns (O(1) complexity)

### Concurrency
- **Optimal Threads**: 4-8 threads
- **Max Efficiency**: 139% at 4 threads
- **Scalability**: Good up to 16 threads

## 📈 Architecture Improvements

Compared to the `master` branch:

| Aspect | Master | Improve | Improvement |
|--------|--------|---------|-------------|
| Backend Types | 1 | 4 | +300% |
| Analysis Modules | 3 | 10+ | +233% |
| Detectors | 2 | 5 | +150% |
| Code Coverage | ~60% | ~85% | +42% |
| Performance | Good | Excellent | Up to 80% faster |

See [Architecture Overview](ARCHITECTURE.md) for detailed comparison.

## 🛠️ Running Benchmarks

```bash
# Quick mode (~5 minutes)
make bench-quick

# Full mode (~60 minutes)
make bench

# Save results
make bench-save
```

See [Benchmark Guide](BENCHMARK_GUIDE_EN.md) for more options.

## 📝 Contributing

Found an issue or want to improve documentation?
1. Check existing issues
2. Submit a pull request
3. Follow the documentation style guide

## 📄 License

All documentation is licensed under MIT OR Apache-2.0, same as the main project.

---

**Last Updated**: 2026-04-12  
**Maintainer**: memscope-rs team
