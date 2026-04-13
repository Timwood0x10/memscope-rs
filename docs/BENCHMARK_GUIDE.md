# Benchmark 使用指南

## 快速开始

### 运行快速模式（推荐）

```bash
make bench-quick
```

- **运行时间**: \~5分钟
- **适用场景**: 日常开发、快速验证
- **配置**: 10次采样，100ms预热，500ms测量

### 运行完整模式

```bash
make bench
```

- **运行时间**: \~60分钟
- **适用场景**: 发布前验证、性能回归检测
- **配置**: 100次采样，3秒预热，5秒测量

## 分类运行

### 核心功能测试

```bash
# Tracker核心功能
make bench-tracker

# 后端性能
make bench-backend

# 类型分类
make bench-classification
```

### 并发测试

```bash
# 并发追踪
make bench-concurrent

# 压力测试
make bench-stress
```

### 特定场景测试

```bash
# IO操作
make bench-io

# 内存压力
make bench-pressure

# 真实场景
make bench-scenario
```

### 新增测试

```bash
# 内存分配器对比
make bench-allocator

# 长期稳定性
make bench-stability

# 边缘情况
make bench-edge

# 性能回归检测
make bench-regression
```

## 结果保存

### 自动保存结果

```bash
make bench-save
```

- 运行快速模式并保存结果
- 结果文件: `benches/benchmark_results_quick_YYYYMMDD_HHMMSS.log`
- 包含时间戳，方便对比历史数据

### 手动保存

```bash
QUICK_BENCH=1 cargo bench 2>&1 | tee benches/my_benchmark.log
```

## 环境变量

### QUICK\_BENCH

控制benchmark运行模式：

```bash
# 快速模式
QUICK_BENCH=1 cargo bench

# 完整模式（默认）
cargo bench
```

### 自定义配置

```bash
# 自定义采样数
QUICK_BENCH=1 SAMPLE_SIZE=50 cargo bench

# 自定义预热时间（毫秒）
QUICK_BENCH=1 WARM_UP_TIME=200 cargo bench

# 自定义测量时间（毫秒）
QUICK_BENCH=1 MEASUREMENT_TIME=1000 cargo bench
```

## 性能报告

### 查看分析报告

```bash
# 中文版
cat benches/PERFORMANCE_ANALYSIS.md

# 英文版
cat benches/PERFORMANCE_ANALYSIS_EN.md
```

### 历史结果对比

```bash
# 查看所有历史结果
ls -lh benches/benchmark_results_*.log

# 对比两次结果
diff benches/benchmark_results_quick_20260412_193000.log \
     benches/benchmark_results_quick_20260412_200000.log
```

## CI/CD 集成

### GitHub Actions 示例

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
      
      # 快速模式（PR检查）
      - name: Run quick benchmarks
        run: make bench-quick
      
      # 保存结果
      - name: Save benchmark results
        uses: actions/upload-artifact@v2
        with:
          name: benchmark-results
          path: benches/benchmark_results_*.log
```

### 定期完整测试

```yaml
name: Full Benchmark

on:
  schedule:
    - cron: '0 2 * * 0'  # 每周日凌晨2点

jobs:
  full-benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      
      - name: Run full benchmarks
        run: make bench
```

## 性能基线

### 关键指标

| 操作    | 快速模式延迟        | 完整模式延迟        | 目标       |
| ----- | ------------- | ------------- | -------- |
| 单次追踪  | \~535 ns      | \~535 ns      | < 1 µs   |
| 后端分配  | \~21 ns       | \~21 ns       | < 50 ns  |
| 统计查询  | \~250 ns      | \~250 ns      | < 500 ns |
| 小规模分析 | \~540 µs (1k) | \~540 µs (1k) | < 1 ms   |
| 大规模分析 | \~35 ms (50k) | \~35 ms (50k) | < 50 ms  |

### 性能回归检测

```bash
# 运行回归检测
make bench-regression

# 如果发现性能下降超过阈值，会显示警告
```

## 故障排查

### 运行时间过长

```bash
# 检查是否使用了快速模式
echo $QUICK_BENCH

# 确保环境变量设置正确
QUICK_BENCH=1 make bench-quick
```

### 内存不足

```bash
# 减少并发线程数
cargo bench -- concurrent_benches --test-threads=4

# 使用更小的数据集
QUICK_BENCH=1 cargo bench
```

### 结果异常

```bash
# 清理缓存重新运行
cargo clean
make bench-quick
```

## 最佳实践

### 开发阶段

1. 使用 `make bench-quick` 快速验证
2. 关注关键指标变化
3. 定期保存结果对比

### 发布前

1. 运行 `make bench` 完整测试
2. 对比历史基线数据
3. 检查性能回归报告

### 生产环境

1. 使用采样模式减少开销
2. 定期运行稳定性测试
3. 监控长期性能趋势

## 相关文档

- [性能分析报告（中文）](PERFORMANCE_ANALYSIS.md)
- [性能分析报告（英文）](PERFORMANCE_ANALYSIS_EN.md)
- [Benchmark代码](comprehensive_benchmarks.rs)

## 常见问题

**Q: 快速模式和完整模式结果差异大吗？**\
A: 差异很小（<5%），快速模式足够用于日常开发验证。

**Q: 如何选择运行模式？**\
A: 日常开发用快速模式，发布前用完整模式。

**Q: 结果文件可以删除吗？**\
A: 可以，但建议保留历史数据用于性能趋势分析。

**Q: 如何添加新的benchmark？**\
A: 在 `comprehensive_benchmarks.rs` 中添加新函数，然后在 `criterion_main!` 中注册。
