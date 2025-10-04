# Lockfree数据分析与模板占位符文档

## 概述
本文档分析了lockfree模块中实际可收集的数据结构，以及HTML模板中使用的占位符，为可视化器开发提供依据。

## 🔍 可收集的真实数据结构

### 1. LockfreeAnalysis (主分析结果)
**位置**: `src/lockfree/analysis.rs`
```rust
pub struct LockfreeAnalysis {
    pub thread_stats: HashMap<u64, ThreadStats>,           // 线程统计数据
    pub hottest_call_stacks: Vec<HotCallStack>,           // 最热的调用栈
    pub thread_interactions: Vec<ThreadInteraction>,       // 线程间交互
    pub memory_peaks: Vec<MemoryPeak>,                     // 内存峰值
    pub performance_bottlenecks: Vec<PerformanceBottleneck>, // 性能瓶颈
    pub summary: AnalysisSummary,                          // 总体统计
}
```

### 2. ThreadStats (线程统计)
```rust
pub struct ThreadStats {
    pub thread_id: u64,                    // 线程ID
    pub total_allocations: u64,            // 总分配次数
    pub total_deallocations: u64,          // 总释放次数
    pub peak_memory: usize,                // 峰值内存使用
    pub total_allocated: usize,            // 总分配字节数
    pub allocation_frequency: HashMap<u64, u64>, // 调用栈频率
    pub avg_allocation_size: f64,          // 平均分配大小
    pub timeline: Vec<AllocationEvent>,    // 分配事件时间线
}
```

### 3. AllocationEvent (分配事件)
```rust
pub struct AllocationEvent {
    pub timestamp: u64,           // 时间戳(纳秒)
    pub ptr: usize,               // 内存指针地址
    pub size: usize,              // 分配大小(字节)
    pub call_stack_hash: u64,     // 调用栈哈希
    pub event_type: EventType,    // 事件类型(分配/释放)
    pub thread_id: u64,           // 线程ID
}
```

### 4. HotCallStack (热门调用栈)
```rust
pub struct HotCallStack {
    pub call_stack_hash: u64,     // 调用栈哈希
    pub total_frequency: u64,     // 总频率
    pub total_size: usize,        // 总大小
    pub impact_score: u64,        // 影响分数
    pub threads: Vec<u64>,        // 涉及的线程
}
```

### 5. PlatformResourceMetrics (平台资源指标)
**位置**: `src/lockfree/platform_resources.rs`
```rust
pub struct PlatformResourceMetrics {
    pub timestamp: u64,                                    // 时间戳
    pub cpu_metrics: CpuResourceMetrics,                   // CPU指标
    pub gpu_metrics: Option<GpuResourceMetrics>,           // GPU指标(未实现)
    pub io_metrics: IoResourceMetrics,                     // I/O指标(默认值)
    pub thread_metrics: HashMap<u64, ThreadResourceMetrics>, // 线程资源指标
}
```

### 6. CpuResourceMetrics (CPU资源指标)
```rust
pub struct CpuResourceMetrics {
    pub overall_usage_percent: f32,      // 总体CPU使用率
    pub per_core_usage: Vec<f32>,        // 每个核心使用率
    pub frequency_mhz: Vec<u32>,         // 频率(未实现)
    pub temperature_celsius: Vec<f32>,   // 温度(未实现)
    pub context_switches_per_sec: u64,   // 上下文切换(未实现)
    pub interrupts_per_sec: u64,         // 中断(未实现)
    pub load_average: (f64, f64, f64),   // 负载平均值
}
```

## 📋 模板占位符分析

### 基础指标占位符
- `{{cpu_usage}}` - CPU平均使用率(%)
- `{{cpu_peak}}` - CPU峰值使用率(%)
- `{{cpu_cores}}` - CPU核心数量
- `{{gpu_usage}}` - GPU使用率(%)
- `{{gpu_status}}` - GPU状态
- `{{total_allocations}}` - 总分配次数
- `{{peak_memory}}` - 峰值内存使用
- `{{memory_efficiency}}` - 内存效率(%)
- `{{system_efficiency}}` - 系统效率(%)
- `{{bottleneck_type}}` - 瓶颈类型

### 线程数据占位符
- `{{thread_count}}` - 线程总数
- `{{active_tracked_threads}}` - 活跃跟踪线程数
- `{{total_peak_memory}}` - 总峰值内存
- `{{avg_allocations_per_thread}}` - 平均每线程分配数

### 线程卡片循环占位符
```handlebars
{{#each threads}}
  <div class="thread-card">
    <span>{{id}}</span>                    <!-- 线程ID -->
    <span>{{role_icon}}</span>             <!-- 角色图标 -->
    <span>{{role_name}}</span>             <!-- 角色名称 -->
    <span>{{allocations}}</span>           <!-- 分配次数 -->
    <span>{{peak_memory}}</span>           <!-- 峰值内存 -->
    <span>{{cpu_usage}}</span>             <!-- CPU使用率 -->
    <span>{{io_operations}}</span>         <!-- I/O操作数 -->
  </div>
{{/each}}
```

### 性能排名占位符
```handlebars
{{#each top_performing_threads}}
  <tr>
    <td>{{rank}}</td>                     <!-- 排名 -->
    <td>{{thread_id}}</td>                <!-- 线程ID -->
    <td>{{efficiency_score}}</td>          <!-- 效率分数 -->
    <td>{{allocations}}</td>              <!-- 分配次数 -->
    <td>{{memory}}</td>                   <!-- 内存使用 -->
  </tr>
{{/each}}
```

### 内存模式占位符
```handlebars
{{#each memory_patterns}}
  <div class="pattern-card">
    <h4>{{thread_id}}</h4>                <!-- 线程ID -->
    <div>{{allocations}}</div>            <!-- 分配次数 -->
    <div>{{bar_width}}</div>              <!-- 进度条宽度 -->
  </div>
{{/each}}
```

### 资源样本占位符
```handlebars
{{#each resource_samples}}
  <tr>
    <td>{{sample_id}}</td>                <!-- 样本ID -->
    <td>{{timestamp}}</td>                <!-- 时间戳 -->
    <td>{{cpu_usage}}</td>                <!-- CPU使用率 -->
    <td>{{memory_usage}}</td>             <!-- 内存使用 -->
  </tr>
{{/each}}
```

### CPU核心占位符
```handlebars
{{#each cpu_cores}}
  <div class="core-card">
    <div>{{name}}</div>                   <!-- 核心名称 -->
    <div>{{usage}}</div>                  <!-- 使用率 -->
  </div>
{{/each}}
```

## 🎯 实际可用的数据字段

### ✅ 可靠可用的字段
1. **ThreadStats中的字段**:
   - `thread_id`, `total_allocations`, `total_deallocations`
   - `peak_memory`, `total_allocated`, `avg_allocation_size`
   - `timeline` (包含实际分配事件)

2. **CpuResourceMetrics中的字段**:
   - `overall_usage_percent`, `per_core_usage`, `load_average`

3. **AnalysisSummary中的字段**:
   - `total_threads`, `total_allocations`, `total_deallocations`
   - `peak_memory_usage`, `total_memory_allocated`

### ⚠️ 部分可用/需注意的字段
1. **HotCallStack**: 可从`hottest_call_stacks`获取，但可能为空
2. **ThreadInteraction**: 可从`thread_interactions`获取，但可能为空
3. **MemoryPeak**: 可从`memory_peaks`获取，但可能为空
4. **PerformanceBottleneck**: 可从`performance_bottlenecks`获取，但可能为空

### ❌ 不可用的字段
1. **GPU指标**: `gpu_metrics`始终为`None`
2. **I/O指标**: `io_metrics`始终为默认值(全零)
3. **详细的线程资源指标**: 大部分字段未实现

## 🚀 推荐的实现策略

### 1. 核心数据展示
优先展示以下真实数据:
- 线程分配统计和效率
- CPU使用率和核心负载
- 内存峰值和分配时间线
- 总体系统统计

### 2. 条件渲染
对于可能为空的数据结构，使用条件判断:
```rust
if analysis.hottest_call_stacks.is_empty() {
    // 显示空状态或隐藏相关部分
} else {
    // 渲染热门调用栈数据
}
```

### 3. 数据转换建议
- 将字节转换为MB显示: `size as f32 / 1024.0 / 1024.0`
- 计算效率百分比: `(deallocations as f32 / allocations as f32) * 100.0`
- 时间戳转换为人类可读格式

### 4. 性能考虑
- 限制显示的项目数量(例如前10个线程，前20个样本)
- 对于大型数据集，使用分页或滚动加载

这份文档为可视化器的开发提供了清晰的数据映射关系，确保只展示实际可收集的数据。
