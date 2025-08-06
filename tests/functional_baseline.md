# memscope-rs 现有功能完整清单

## 公共API接口

### 核心跟踪功能

- `MemoryTracker` - 主要内存跟踪器
- `get_global_tracker()` - 获取全局跟踪器实例
- `TrackingAllocator` - 自定义全局分配器
- `Trackable` trait - 可跟踪类型的trait

### 跟踪宏

- `track_var!()` - 零成本引用跟踪（推荐）
- `track_var_owned!()` - 拥有权转移的生命周期跟踪
- `track_var_smart!()` - 智能类型检测跟踪
- `TrackedVariable<T>` - 自动生命周期跟踪包装器

### 支持的类型

#### 标准集合类型

- `Vec<T>` - 动态数组
- `String` - 字符串
- `HashMap<K,V,S>` - 哈希映射
- `BTreeMap<K,V>` - B树映射
- `HashSet<T>` - 哈希集合
- `BTreeSet<T>` - B树集合
- `VecDeque<T>` - 双端队列
- `LinkedList<T>` - 链表
- `BinaryHeap<T>` - 二进制堆

#### 智能指针类型

- `Box<T>` - 堆分配指针
- `Rc<T>` - 引用计数指针
- `Arc<T>` - 原子引用计数指针
- `Weak<T>` - 弱引用指针（Rc和Arc）

#### 高级类型

- `RefCell<T>` - 内部可变性
- `Mutex<T>` - 互斥锁
- `RwLock<T>` - 读写锁
- `Cell<T>` - 内部可变性（Copy类型）
- `ManuallyDrop<T>` - 手动析构
- `MaybeUninit<T>` - 可能未初始化
- `Pin<T>` - 固定指针
- 各种原子类型 (`AtomicBool`, `AtomicUsize`, 等)
- `mpsc::Sender<T>` / `Receiver<T>` - 消息传递通道

#### 复合类型

- `Option<T>` - 可选值
- `Result<T,E>` - 结果类型
- 元组类型 `(T1,T2,T3)`
- `CString` - C字符串
- `RandomState` - 哈希状态

## 分析功能

### 内存分析

- `EnhancedMemoryAnalyzer` - 增强内存分析器
- `analyze_memory_with_enhanced_features()` - 增强内存分析
- `analyze_fragmentation()` - 内存碎片分析
- `analyze_system_libraries()` - 系统库使用分析
- `analyze_concurrency_safety()` - 并发安全分析

### 专门分析器

- `UnsafeFFITracker` - 不安全FFI跟踪
- `CircularReferenceAnalysis` - 循环引用检测
- `BorrowAnalyzer` - 借用模式分析
- `GenericAnalyzer` - 泛型类型分析
- `AsyncAnalyzer` - 异步模式分析
- `ClosureAnalyzer` - 闭包分析
- `LifecycleAnalyzer` - 生命周期分析
- `SecurityViolationAnalyzer` - 安全违规分析

### 变量关系分析

- `build_variable_relationship_graph()` - 构建变量关系图
- `VariableRelationshipGraph` - 变量关系图
- `VariableNode` - 变量节点
- `VariableRelationship` - 变量关系
- `VariableCluster` - 变量集群

### 高级类型分析

- `AdvancedTypeAnalysisReport` - 高级类型分析报告
- `analyze_advanced_types()` - 分析高级类型
- `GenericAdvancedTypeAnalyzer` - 泛型高级类型分析器

## 导出功能

### 导出格式

- **JSON导出**
  - `export_to_json()` - 基本JSON导出
  - `export_to_json_with_options()` - 带选项的JSON导出
  - `streaming_json_writer` - 流式JSON写入器
  - `optimized_json_export` - 优化的JSON导出

- **二进制导出**
  - `binary/` 模块 - 二进制格式导出
  - 内存布局序列化
  - 压缩二进制格式

- **HTML导出**
  - `html_export` - HTML可视化导出
  - 交互式内存可视化

- **可视化导出**
  - `export_lifecycle_timeline()` - 生命周期时间线
  - `export_memory_analysis()` - 内存分析可视化
  - SVG图表生成

### 导出选项

- `ExportOptions` - 导出配置
  - `include_system_allocations` - 包含系统分配
  - `verbose_logging` - 详细日志
  - `buffer_size` - 缓冲区大小
  - `compress_output` - 输出压缩

### 性能优化

- `adaptive_performance` - 自适应性能优化
- `batch_processor` - 批处理器
- `parallel_shard_processor` - 并行分片处理
- `high_speed_buffered_writer` - 高速缓冲写入器
- `fast_export_coordinator` - 快速导出协调器

### 质量保证

- `quality_validator` - 质量验证器
- `schema_validator` - 模式验证器
- `error_handling` - 错误处理
- `error_recovery` - 错误恢复

## 工具和实用程序

### 格式化工具

- `format_bytes()` - 字节格式化
- `get_simple_type()` - 获取简单类型名
- `simplify_type_name()` - 简化类型名

### 变量注册

- `variable_registry` - 轻量级HashMap变量跟踪

### 宏支持

- `advanced_trackable_macro` - 高级可跟踪宏
- `impl_advanced_trackable!()` - 实现高级可跟踪trait

## 命令行工具

### 二进制工具

- `allocation_count_diagnostic` - 分配计数诊断
- `core_performance_test` - 核心性能测试
- `large_active_allocations` - 大型活跃分配分析
- `lifecycle_analysis` - 生命周期分析工具
- `performance_only_benchmark` - 纯性能基准测试
- `run_benchmark` - 运行基准测试
- `simple_benchmark` - 简单基准测试
- `test_mode_specific_validation` - 测试模式特定验证

### CLI命令

- `cli/commands/` - 命令行接口模块

## 数据类型和结构

### 核心类型

- `AllocationInfo` - 分配信息
- `MemoryStats` - 内存统计
- `TrackingError` - 跟踪错误
- `TrackingResult<T>` - 跟踪结果类型

### 分析结果类型

- `FragmentationAnalysis` - 碎片分析结果
- `SystemLibraryStats` - 系统库统计
- `ConcurrencyAnalysis` - 并发分析结果
- `ComprehensiveAnalysisReport` - 综合分析报告
- `BorrowPatternAnalysis` - 借用模式分析
- `GenericStatistics` - 泛型统计
- `AsyncPatternAnalysis` - 异步模式分析
- `ClosureAnalysisReport` - 闭包分析报告
- `LifecycleAnalysisReport` - 生命周期分析报告

### 内存信息类型

- `MemoryTypeInfo` - 内存类型信息
- `TypeMemoryUsage` - 类型内存使用
- `AllocatorStateInfo` - 分配器状态信息
- `CachePerformanceInfo` - 缓存性能信息
- `CpuUsageInfo` - CPU使用信息
- `MemoryPressureInfo` - 内存压力信息

### 泛型和类型信息

- `GenericTypeInfo` - 泛型类型信息
- `DynamicTypeInfo` - 动态类型信息
- `MemoryLayoutInfo` - 内存布局信息
- `RuntimeStateInfo` - 运行时状态信息
- `TypeParameter` - 类型参数
- `GenericConstraint` - 泛型约束
- `MonomorphizationInfo` - 单态化信息
- `CodeBloatLevel` - 代码膨胀级别

### 作用域和上下文

- `StackScopeInfo` - 栈作用域信息
- `CreationContext` - 创建上下文
- `SourceLocation` - 源码位置
- `ScopeType` - 作用域类型
- `ExpressionType` - 表达式类型

### 性能和优化

- `PerformanceImpact` - 性能影响
- `VTableInfo` - 虚表信息
- `TypeErasureInfo` - 类型擦除信息
- `DispatchOverhead` - 分发开销
- `EnhancedFragmentationAnalysis` - 增强碎片分析
- `StackAllocationInfo` - 栈分配信息
- `TemporaryObjectInfo` - 临时对象信息

## 特性标志

### 编译时特性

- `tracking-allocator` - 启用全局跟踪分配器
- `derive` - 启用派生宏支持

## 配置选项

### 导出配置

- 系统分配包含控制
- 详细日志控制
- 缓冲区大小配置
- 压缩选项

### 性能配置

- 批处理大小
- 并行处理选项
- 内存阈值设置
- 优化级别控制

## 错误处理

### 错误类型

- `TrackingError` - 主要错误类型
- 错误恢复机制
- 错误日志记录

### 错误处理策略

- 优雅降级
- 错误传播
- 恢复机制

## 测试和验证

### 测试工具

- 单元测试框架
- 集成测试
- 性能基准测试
- 回归测试

### 验证工具

- 数据完整性验证
- 模式验证
- 质量检查

## 向后兼容性保证

所有上述功能在优化过程中必须保持完全可用：

- 所有公共API接口保持不变
- 所有导出格式继续支持
- 所有分析功能保持可用
- 所有配置选项继续有效
- 所有错误处理行为保持一致
- 所有特性标志继续工作
