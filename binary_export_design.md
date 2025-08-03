# 精简高效Binary导出模块设计方案

## 概述

基于对现有JSON/HTML导出模块的分析，设计一个务实、高效的binary导出模块，专注解决实际问题，避免过度工程化。

## 1. 设计原则

### 1.1 核心原则
- **务实简单**：不搞花哨功能，专注核心需求
- **无锁设计**：避免并发复杂性，使用单线程顺序写入
- **兼容现有**：与JSON/HTML导出接口保持一致
- **高效紧凑**：二进制格式，最小化文件大小和导出时间

### 1.2 技术约束
- 不使用锁机制
- 暂时不实现高并发
- 不依赖外部序列化库
- 保持与现有导出模块的接口一致性

## 2. 架构设计

### 2.1 模块结构
```
src/export/binary.rs
├── BinaryExportConfig     // 导出配置
├── BinaryExporter        // 核心导出器
├── BinaryExportStats     // 导出统计信息
├── MemoryTracker扩展     // 为现有tracker添加binary导出方法
└── 便捷函数              // 兼容现有接口的便捷函数
```

### 2.2 依赖关系
```
binary.rs
├── crate::core::types::{AllocationInfo, MemoryStats, TrackingResult}
├── crate::core::tracker::MemoryTracker
├── std::fs::File
├── std::io::{BufWriter, Write}
└── std::path::Path
```

## 3. 文件格式设计

### 3.1 整体结构
```
Binary文件结构：
┌─────────────────────────┐
│ 文件头 (24字节)          │ <- 魔数+版本+标志+时间戳
├─────────────────────────┤
│ 统计段 (60字节)          │ <- 核心内存统计数据
├─────────────────────────┤
│ 分配段 (变长)            │ <- 分配记录数组
│ ├─ 段标识 (4字节)        │
│ ├─ 记录数量 (4字节)      │
│ └─ 分配记录列表 (变长)   │
└─────────────────────────┘
```

### 3.2 文件头格式 (24字节)
```
偏移  大小  字段          说明
0     8     魔数          "MEMSCOPE" (8字节)
8     4     版本号        格式版本 (u32, 小端序)
12    4     标志位        配置标志 (u32, 小端序)
16    8     时间戳        导出时间 (u64, 小端序)
```

### 3.3 统计段格式 (60字节)
```
偏移  大小  字段                    说明
0     4     段标识                  "STAT" (4字节)
4     8     total_allocations       总分配次数
12    8     total_deallocations     总释放次数
20    8     active_allocations      活跃分配数
28    8     active_memory          活跃内存大小
36    8     peak_memory            峰值内存大小
44    8     total_allocated        总分配内存
52    8     total_deallocated      总释放内存
```

### 3.4 分配记录格式
#### 基础模式 (24字节/记录)
```
偏移  大小  字段              说明
0     8     ptr               指针地址 (u64)
8     8     size              分配大小 (u64)
16    8     timestamp_alloc   分配时间戳 (u64)
```

#### 详细模式 (变长)
```
基础24字节 +
├─ var_name_len (2字节) + var_name (变长)
├─ type_name_len (2字节) + type_name (变长)
└─ flags (1字节)
```

## 4. 配置系统

### 4.1 配置结构
```rust
#[derive(Debug, Clone)]
pub struct BinaryExportConfig {
    /// 缓冲区大小 (默认: 64KB)
    pub buffer_size: usize,
    
    /// 是否包含详细信息 (默认: false，只导出核心数据)
    pub include_details: bool,
    
    /// 压缩级别 (0=无压缩, 1-9=压缩级别，默认: 0)
    pub compression_level: u8,
}
```

### 4.2 配置构建器
```rust
impl BinaryExportConfig {
    pub fn new() -> Self
    pub fn buffer_size(mut self, size: usize) -> Self
    pub fn include_details(mut self, include: bool) -> Self
    pub fn compression_level(mut self, level: u8) -> Self
}
```

## 5. 核心实现

### 5.1 导出器结构
```rust
pub struct BinaryExporter {
    config: BinaryExportConfig,
}

impl BinaryExporter {
    pub fn new(config: BinaryExportConfig) -> Self
    pub fn export<P: AsRef<Path>>(&self, tracker: &MemoryTracker, path: P) -> TrackingResult<BinaryExportStats>
    
    // 私有方法
    fn write_header<W: Write>(&self, writer: &mut W) -> TrackingResult<()>
    fn write_stats<W: Write>(&self, writer: &mut W, stats: &MemoryStats) -> TrackingResult<usize>
    fn write_allocations<W: Write>(&self, writer: &mut W, allocations: &[AllocationInfo]) -> TrackingResult<usize>
    fn write_single_allocation<W: Write>(&self, writer: &mut W, alloc: &AllocationInfo) -> TrackingResult<usize>
}
```

### 5.2 统计信息
```rust
#[derive(Debug, Clone)]
pub struct BinaryExportStats {
    /// 导出耗时（毫秒）
    pub export_time_ms: u64,
    /// 文件大小（字节）
    pub file_size: usize,
    /// 分配记录数量
    pub allocations_count: usize,
    /// 统计信息大小
    pub stats_size: usize,
    /// 分配信息大小
    pub allocations_size: usize,
}

impl BinaryExportStats {
    /// 计算压缩比（相对于JSON）
    pub fn compression_ratio_vs_json(&self, json_size: usize) -> f64
    /// 获取每个分配记录的平均大小
    pub fn avg_allocation_size(&self) -> f64
}
```

## 6. 接口兼容性

### 6.1 MemoryTracker扩展
```rust
impl MemoryTracker {
    /// 导出到binary格式
    pub fn export_to_binary<P: AsRef<Path>>(&self, path: P) -> TrackingResult<BinaryExportStats>
    
    /// 使用自定义配置导出到binary格式
    pub fn export_to_binary_with_config<P: AsRef<Path>>(
        &self, 
        path: P, 
        config: BinaryExportConfig
    ) -> TrackingResult<BinaryExportStats>
}
```

### 6.2 便捷函数
```rust
/// 便捷函数：导出到binary格式
pub fn export_to_binary<P: AsRef<Path>>(tracker: &MemoryTracker, path: P) -> TrackingResult<BinaryExportStats>

/// 便捷函数：使用配置导出到binary格式
pub fn export_to_binary_with_config<P: AsRef<Path>>(
    tracker: &MemoryTracker, 
    path: P, 
    config: BinaryExportConfig
) -> TrackingResult<BinaryExportStats>
```

### 6.3 使用示例
```rust
// 基本使用
let tracker = get_global_tracker();
let stats = tracker.export_to_binary("output.bin")?;

// 自定义配置
let config = BinaryExportConfig::new()
    .buffer_size(128 * 1024)
    .include_details(true);
let stats = tracker.export_to_binary_with_config("detailed.bin", config)?;

// 便捷函数
let stats = export_to_binary(&tracker, "simple.bin")?;
```

## 7. 性能优化策略

### 7.1 I/O优化
- 使用`BufWriter`进行批量写入
- 可配置缓冲区大小（默认64KB）
- 顺序写入，避免随机访问

### 7.2 数据格式优化
- 小端序二进制格式，避免字节序转换
- 字符串长度前缀编码，减少解析开销
- 紧凑的数据布局，最小化填充字节

### 7.3 内存优化
- 避免大量临时对象分配
- 直接从源数据写入，减少中间拷贝
- 不使用序列化库，减少依赖和开销

## 8. 实现计划

### 8.1 阶段1：核心功能 (优先级：高)
- [x] 基本文件格式定义
- [x] 文件头和统计段写入
- [x] 基础分配记录写入
- [x] 配置系统实现
- [x] 基本错误处理
- [x] 单元测试

### 8.2 阶段2：完善功能 (优先级：中)
- [ ] 详细信息导出（变量名、类型名）
- [ ] 错误处理优化和恢复机制
- [ ] 性能基准测试
- [ ] 与JSON导出的对比测试
- [ ] 文档和使用示例

### 8.3 阶段3：扩展功能 (优先级：低)
- [ ] 简单压缩支持（可选）
- [ ] 二进制文件读取器
- [ ] 格式验证和完整性检查
- [ ] 版本兼容性处理

## 9. 预期性能收益

### 9.1 文件大小
- **基础模式**：每条记录24字节 vs JSON ~100-200字节
- **详细模式**：每条记录约40-80字节 vs JSON ~200-400字节
- **预计压缩比**：相比JSON减少60-80%文件大小

### 9.2 导出速度
- **无序列化开销**：直接二进制写入
- **批量I/O**：使用缓冲写入器
- **预计性能提升**：比JSON导出快2-3倍

### 9.3 内存使用
- **减少临时对象**：避免JSON序列化中间对象
- **流式写入**：不需要在内存中构建完整数据结构
- **预计内存节省**：减少30-50%内存使用

## 10. 风险评估与缓解

### 10.1 低风险项
- **文件格式简单**：二进制格式不易出错
- **无并发复杂性**：单线程顺序写入
- **接口兼容性**：遵循现有模式，集成简单

### 10.2 需要注意的风险
- **字节序兼容性**
  - 风险：不同平台字节序差异
  - 缓解：统一使用小端序格式

- **文件完整性**
  - 风险：写入过程中断导致文件损坏
  - 缓解：使用缓冲写入器，确保flush操作

- **版本兼容性**
  - 风险：格式变更导致旧文件无法读取
  - 缓解：预留版本字段，实现向后兼容

- **大文件处理**
  - 风险：内存不足或写入超时
  - 缓解：流式写入，可配置缓冲区大小

## 11. 测试策略

### 11.1 单元测试
```rust
#[cfg(test)]
mod tests {
    #[test] fn test_binary_export_basic()
    #[test] fn test_binary_export_with_details()
    #[test] fn test_binary_config_builder()
    #[test] fn test_file_format_compatibility()
    #[test] fn test_error_handling()
}
```

### 11.2 集成测试
- 与现有JSON导出结果对比
- 大数据集性能测试
- 错误场景测试（磁盘满、权限不足等）

### 11.3 性能基准
- 导出时间对比（binary vs JSON）
- 文件大小对比
- 内存使用对比

## 12. 总结

这个设计方案专注于解决实际问题：

1. **务实简单**：避免过度工程化，专注核心功能
2. **高效紧凑**：二进制格式显著减少文件大小和导出时间
3. **兼容现有**：无缝集成到现有导出系统
4. **易于维护**：简单的文件格式和清晰的代码结构

该方案能够快速实现并投入使用，为用户提供更高效的数据导出选择。