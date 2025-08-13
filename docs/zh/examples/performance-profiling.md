# 性能分析示例

使用 memscope-rs 进行性能分析和优化的实用指南。

## 🎯 目标

- 识别内存分配热点
- 分析分配模式和频率
- 优化内存使用效率
- 对比不同实现的性能

## 🚀 快速示例

```rust
use memscope_rs::{init, track_var, get_global_tracker};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    
    // 性能测试
    benchmark_allocations();
    
    // 导出分析
    let tracker = get_global_tracker();
    tracker.export_to_binary("performance_analysis")?;
    
    println!("运行: make html DIR=MemoryAnalysis/performance_analysis BASE=performance_analysis");
    Ok(())
}

fn benchmark_allocations() {
    // 测试1: 频繁小分配
    let start = Instant::now();
    for i in 0..10000 {
        let data = vec![i; 10];
        track_var!(data);
    }
    println!("小分配耗时: {:?}", start.elapsed());
    
    // 测试2: 少量大分配
    let start = Instant::now();
    let large_data = vec![0; 100000];
    track_var!(large_data);
    println!("大分配耗时: {:?}", start.elapsed());
}
```

## 📊 分析结果

- **小分配**: 高频率，低效率
- **大分配**: 低频率，高效率
- **建议**: 使用内存池或预分配

## 🔧 优化策略

1. **预分配容量**: `Vec::with_capacity()`
2. **内存池**: 重用分配
3. **批量操作**: 减少分配次数
4. **栈分配**: 优先使用栈

## 🎉 总结

通过性能分析，可以：
- 识别瓶颈
- 优化分配策略
- 提升程序性能
- 减少内存开销