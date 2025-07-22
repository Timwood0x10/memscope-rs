# JSON导出优化与变量关系图实现任务文档

## 任务概述

基于`.kiro/specs/json-export-optimization`规格文档，实现JSON导出性能优化和变量关系图数据收集功能。核心目标是通过分离式导出策略解决当前JSON文件过大的问题，同时为可交互的HTML内存分析界面提供完整的变量关系数据。

## 核心任务

### 1. JSON导出性能优化

#### 1.1 分离式导出架构实现
- **目标**：将单一大JSON文件分离为4个专门文件
- **输出文件**：
  - `{base_name}_variable_relationships.json` - 变量关系图数据
  - `{base_name}_memory_analysis.json` - 内存统计分析
  - `{base_name}_lifetime_analysis.json` - 生命周期时间线
  - `{base_name}_unsafe_ffi_analysis.json` - 安全分析

#### 1.2 性能优化策略
- **并行处理优化**：
  - 大数据集（>1000分配）使用并行处理
  - 小数据集使用串行处理避免并行开销
  - 智能决策机制根据数据量和可用内存选择策略
- **缓存机制**：
  - 实现类型推断结果缓存（LRU策略）
  - 避免重复计算相同大小的类型推断
- **数据预处理**：
  - 一次性提取所有共享数据，避免重复锁竞争
  - 批量处理减少内存分配开销

#### 1.3 性能监控
- 记录各阶段处理时间
- 报告吞吐量统计（分配数/毫秒）
- 提供详细的性能分析信息
- 压缩比和文件大小报告

### 2. 变量关系图数据收集与存储

#### 2.1 关系图数据结构设计

**节点数据（变量）**：
```rust
pub struct VariableNode {
    pub id: String,           // 唯一标识符（通常是内存地址）
    pub name: String,         // 变量名
    pub type_name: String,    // 类型名
    pub size: usize,          // 内存大小
    pub scope: String,        // 作用域
    pub is_active: bool,      // 是否仍然活跃
    pub smart_pointer_info: Option<SmartPointerInfo>,
    pub category: VariableCategory, // user_variable | system_allocation
}
```

**关系数据（边）**：
```rust
pub struct VariableRelationship {
    pub source: String,       // 源变量ID
    pub target: String,       // 目标变量ID
    pub relationship_type: RelationshipType,
    pub weight: f64,          // 关系强度
    pub metadata: HashMap<String, serde_json::Value>,
}

pub enum RelationshipType {
    References,    // 引用关系
    Owns,         // 拥有关系
    Clones,       // 克隆关系（Rc/Arc）
    Contains,     // 包含关系（作用域）
    DependsOn,    // 依赖关系
}
```

#### 2.2 关系检测算法

**引用关系检测**：
- 基于智能指针的`data_ptr`字段
- 检测Rc/Arc的共享数据关系
- 识别Box指针的拥有关系

**克隆关系检测**：
- 利用现有的`SmartPointerInfo.clones`和`cloned_from`字段
- 构建克隆链图
- 追踪引用计数变化历史

**作用域关系检测**：
- 基于变量名的作用域前缀分析
- 构建作用域层次结构
- 识别父子作用域包含关系

**循环引用检测**：
- 扩展现有`circular_reference.rs`功能
- 集成到关系图构建过程中
- 提供循环引用修复建议

#### 2.3 图形布局优化

**聚类算法**：
- 按作用域进行变量聚类
- 按类型进行相似变量分组
- 提供层次化显示支持

**布局提示**：
- 预计算节点的建议位置
- 提供力导向图的初始参数
- 优化边的路径和权重

## 实现计划

### Phase 1: 基础架构重构
1. 在`src/export/optimized_export.rs`中实现分离式导出框架
2. 定义4个JSON文件的数据结构
3. 实现基础的串行导出功能

### Phase 2: 变量关系图核心功能
1. 扩展`src/analysis/`模块，添加关系检测功能
2. 实现变量节点和关系的数据收集
3. 集成现有的循环引用检测功能

### Phase 3: 性能优化
1. 实现类型推断缓存机制
2. 添加智能并行处理决策
3. 优化内存使用和数据预处理

### Phase 4: 图形布局和可视化支持
1. 实现聚类和布局算法
2. 生成前端友好的数据格式
3. 添加渐进式加载支持

## 验收标准

### 功能性要求
- ✅ 生成4个分离的JSON文件，每个文件包含对应功能的完整数据
- ✅ 变量关系图包含完整的节点和边信息
- ✅ 支持所有类型的变量关系检测（引用、拥有、克隆、包含）
- ✅ 保持与现有HTML导出功能的完全兼容性

### 性能要求
- ✅ 大数据集处理速度提升至少50%
- ✅ 内存使用优化，避免不必要的数据重复
- ✅ 提供详细的性能监控和报告
- ✅ 支持压缩优化减少文件大小

### 数据完整性要求
- ✅ 所有内存分配记录（用户和系统）完整保留
- ✅ 生命周期分析、释放模式、内存泄漏分析等统计信息完整
- ✅ 变量注册表、作用域信息、类型统计等核心指标不丢失
- ✅ 向后兼容性，现有分析工具能正常解析

## 技术实现要点

### 数据流优化
```rust
// 优化前：多次访问tracker，重复计算
tracker.get_active_allocations() // 调用4次
tracker.get_stats()             // 调用4次

// 优化后：一次性提取，共享使用
let shared_data = SharedExportData::extract_from(tracker);
// 4个JSON生成器共享这个数据
```

### 关系图构建
```rust
// 基于现有数据构建关系图
let relationship_builder = VariableRelationshipBuilder::new();
relationship_builder
    .add_allocations(&allocations)
    .detect_references()
    .detect_clones()
    .detect_scope_relationships()
    .detect_circular_references()
    .build_graph()
```

### 智能并行决策
```rust
let export_strategy = if allocations.len() > 1000 && available_memory > threshold {
    ExportStrategy::Parallel
} else {
    ExportStrategy::Sequential
};
```

## 具体实现文件

### 新增文件
- `src/analysis/variable_relationships.rs` - 变量关系检测和图构建
- `src/export/relationship_export.rs` - 关系图JSON导出
- `src/export/performance_monitor.rs` - 性能监控模块

### 修改文件
- `src/export/optimized_export.rs` - 完善分离式导出实现
- `src/analysis/circular_reference.rs` - 扩展关系检测功能
- `src/variable_registry.rs` - 优化数据提取性能

## 数据格式规范

### variable_relationships.json
```json
{
  "relationship_graph": {
    "nodes": [
      {
        "id": "ptr_123456",
        "name": "my_vec", 
        "type": "Vec<i32>",
        "size": 1024,
        "scope": "main",
        "category": "user_variable",
        "is_active": true,
        "smart_pointer_info": {...}
      }
    ],
    "relationships": [
      {
        "source": "ptr_123456",
        "target": "ptr_789012", 
        "type": "references",
        "weight": 1.0,
        "metadata": {}
      }
    ],
    "clusters": [
      {
        "id": "scope_main",
        "type": "scope",
        "variables": ["ptr_123456", "ptr_789012"],
        "layout_hint": {"x": 100, "y": 200}
      }
    ]
  },
  "statistics": {
    "total_nodes": 150,
    "total_relationships": 89,
    "circular_references": 2,
    "largest_cluster_size": 25
  },
  "metadata": {
    "timestamp": 1234567890,
    "processing_time_ms": 45,
    "export_type": "variable_relationships"
  }
}
```

### memory_analysis.json
```json
{
  "memory_statistics": {
    "total_allocated": 1048576,
    "active_allocations": 150,
    "peak_memory": 2097152
  },
  "allocations_summary": [...],
  "memory_by_type": [...],
  "performance_metrics": {...}
}
```

### lifetime_analysis.json
```json
{
  "timeline_events": [...],
  "temporal_relationships": [...],
  "lifecycle_patterns": [...],
  "allocation_timeline": [...]
}
```

### unsafe_ffi_analysis.json
```json
{
  "unsafe_relationships": [...],
  "circular_references": [...],
  "safety_violations": [...],
  "risk_assessment": {...}
}
```

## 性能目标

- **处理速度**：大数据集（>10000分配）处理时间减少50%以上
- **内存使用**：峰值内存使用减少30%
- **文件大小**：通过分离和压缩，单个文件大小减少60%
- **加载速度**：前端页面数据加载时间减少70%

这个任务文档涵盖了规格文档中的所有核心需求，特别关注了变量关系图的数据收集和存储，以及JSON导出的性能优化。实现将分阶段进行，确保每个阶段都有明确的验收标准和技术实现要点。