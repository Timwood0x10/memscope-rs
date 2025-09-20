# 异步内存模块文档

## 概述

`async_memory` 模块为Rust异步应用程序提供了全面的内存跟踪和资源监控功能。它提供实时性能分析、资源使用跟踪、瓶颈识别和高级可视化功能。

## 架构

```
src/async_memory/
├── mod.rs                 # 模块导出和主要API
├── api.rs                 # 高级用户API
├── buffer.rs              # 内存缓冲区管理
├── error.rs               # 错误类型和处理
├── profile.rs             # 任务性能分析
├── resource_monitor.rs    # 资源监控引擎
├── system_monitor.rs      # 系统级监控
├── task_id.rs            # 任务标识
├── tracker.rs            # 内存跟踪实现
└── visualization.rs      # 报告生成和可视化
```

## 核心组件

### 1. AsyncResourceMonitor (异步资源监控器)

主要的资源监控引擎，跟踪异步任务的CPU、内存、IO、网络和GPU使用情况。

```rust
use memscope_rs::async_memory::AsyncResourceMonitor;

let mut monitor = AsyncResourceMonitor::new();

// 开始监控任务
monitor.start_monitoring(task_id, "我的任务", TaskType::CpuIntensive);

// 在执行期间更新指标
monitor.update_metrics(task_id);

// 完成监控
monitor.finish_monitoring(task_id);

// 获取所有配置文件
let profiles = monitor.get_all_profiles();
```

### 2. 任务内存跟踪器

跟踪单个异步任务的内存分配和释放。

```rust
use memscope_rs::async_memory::{create_tracked, TaskId};

async fn my_task() {
    let task_id = TaskId::new();
    
    // 创建跟踪的future
    let tracked_future = create_tracked(task_id, async {
        // 你的异步代码在这里
        let data = vec![0u8; 1024]; // 跟踪的分配
        process_data(data).await
    });
    
    tracked_future.await
}
```

### 3. 资源监控

具有详细指标的综合系统资源跟踪。

#### CPU指标
```rust
pub struct CpuMetrics {
    pub usage_percent: f64,           // CPU利用率百分比
    pub time_user_ms: f64,           // 用户模式执行时间
    pub time_kernel_ms: f64,         // 内核模式执行时间
    pub context_switches: u64,       // 上下文切换次数
    pub cpu_cycles: u64,             // 消耗的CPU周期
    pub instructions: u64,           // 执行的指令数
    pub cache_misses: u64,           // 缓存未命中次数
    pub branch_misses: u64,          // 分支预测错误次数
    pub core_affinity: Vec<u32>,     // 使用的CPU核心
}
```

#### 内存指标
```rust
pub struct MemoryMetrics {
    pub current_bytes: u64,          // 当前内存使用量
    pub peak_bytes: u64,             // 峰值内存使用量
    pub allocations_count: u64,      // 分配次数
    pub deallocations_count: u64,    // 释放次数
    pub heap_fragmentation: f64,     // 堆碎片化比率
    pub memory_bandwidth: f64,       // 内存带宽使用
}
```

#### IO指标
```rust
pub struct IoMetrics {
    pub bytes_read: u64,             // 总读取字节数
    pub bytes_written: u64,          // 总写入字节数
    pub operations_read: u64,        // 读操作次数
    pub operations_written: u64,     // 写操作次数
    pub average_latency_ms: f64,     // 平均IO延迟
    pub bandwidth_mbps: f64,         // IO带宽(MB/s)
    pub io_wait_percent: f64,        // IO等待百分比
    pub queue_depth: u32,            // IO队列深度
}
```

#### 网络指标
```rust
pub struct NetworkMetrics {
    pub bytes_sent: u64,             // 总发送字节数
    pub bytes_received: u64,         // 总接收字节数
    pub packets_sent: u64,           // 发送包数
    pub packets_received: u64,       // 接收包数
    pub connections_active: u32,     // 活跃连接数
    pub latency_avg_ms: f64,         // 平均网络延迟
    pub throughput_mbps: f64,        // 网络吞吐量
    pub error_count: u64,            // 网络错误数
}
```

#### GPU指标（可选）
```rust
pub struct GpuMetrics {
    pub gpu_utilization: f64,        // GPU利用率百分比
    pub memory_used: u64,            // 已使用GPU内存
    pub memory_total: u64,           // 总GPU内存
    pub compute_units: u32,          // 活跃计算单元
    pub memory_bandwidth: f64,       // GPU内存带宽
    pub temperature: f32,            // GPU温度
    pub power_draw: f32,             // 功耗
    pub clock_speed: u32,            // GPU时钟速度
}
```

### 4. 性能分析

具有瓶颈检测和效率评分的高级性能分析。

#### 任务资源配置文件
```rust
pub struct TaskResourceProfile {
    pub task_id: TaskId,
    pub task_name: String,
    pub task_type: TaskType,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub duration_ms: Option<f64>,
    
    // 资源指标
    pub cpu_metrics: CpuMetrics,
    pub memory_metrics: MemoryMetrics,
    pub io_metrics: IoMetrics,
    pub network_metrics: NetworkMetrics,
    pub gpu_metrics: Option<GpuMetrics>,
    
    // 性能分析
    pub efficiency_score: f64,
    pub resource_balance: f64,
    pub bottleneck_type: BottleneckType,
    
    // 增强功能
    pub source_location: SourceLocation,
    pub hot_metrics: HotMetrics,
    pub efficiency_explanation: EfficiencyExplanation,
}
```

#### 瓶颈检测
```rust
pub enum BottleneckType {
    Cpu,        // CPU密集型工作负载
    Memory,     // 内存密集型工作负载
    Io,         // IO密集型工作负载
    Network,    // 网络密集型工作负载
    Gpu,        // GPU密集型工作负载
    Balanced,   // 资源使用均衡
    Unknown,    // 数据不足以进行分析
}
```

#### 任务类型
```rust
pub enum TaskType {
    CpuIntensive,      // CPU密集型计算
    IoIntensive,       // 文件和存储操作
    NetworkIntensive,  // 网络通信
    MemoryIntensive,   // 内存密集型操作
    GpuCompute,        // GPU计算
    Mixed,             // 混合工作负载
    Streaming,         // 实时流处理
    Background,        // 后台处理
}
```

### 5. 源位置跟踪

跟踪定义任务的源代码位置，用于调试和优化。

```rust
pub struct SourceLocation {
    pub file_path: String,           // 源文件路径
    pub line_number: u32,            // 行号
    pub function_name: String,       // 函数名
    pub module_path: String,         // 模块路径
    pub crate_name: String,          // Crate名称
}
```

### 6. 热点指标分析

用于性能优化的详细热点分析。

```rust
pub struct HotMetrics {
    pub cpu_hotspots: Vec<CpuHotspot>,
    pub memory_hotspots: Vec<MemoryHotspot>,
    pub io_hotspots: Vec<IoHotspot>,
    pub network_hotspots: Vec<NetworkHotspot>,
    pub critical_path_analysis: CriticalPathAnalysis,
}
```

### 7. 可视化和报告

生成包含图表和分析的综合HTML报告。

```rust
use memscope_rs::async_memory::{VisualizationGenerator, VisualizationConfig, Theme};

let config = VisualizationConfig {
    title: "性能分析报告".to_string(),
    theme: Theme::Dark,
    include_charts: true,
    include_baselines: true,
    include_rankings: true,
    include_efficiency_breakdown: true,
};

let visualizer = VisualizationGenerator::with_config(config);
let html_report = visualizer.generate_html_report(&profiles)?;
```

## 数据收集逻辑

### 1. 初始化

```rust
use memscope_rs::async_memory;

// 初始化异步内存跟踪系统
async_memory::initialize()?;
```

### 2. 任务注册

```rust
// 方法1：手动注册
let monitor = AsyncResourceMonitor::new();
monitor.start_monitoring(task_id, "任务名称", TaskType::CpuIntensive);

// 方法2：带源位置
let source_location = SourceLocation {
    file_path: "src/main.rs".to_string(),
    line_number: 42,
    function_name: "process_data".to_string(),
    module_path: "myapp::processor".to_string(),
    crate_name: "myapp".to_string(),
};
monitor.start_monitoring_with_location(task_id, "任务名称", TaskType::CpuIntensive, Some(source_location));
```

### 3. 指标收集

```rust
// 在任务执行期间定期更新指标
loop {
    monitor.update_metrics(task_id);
    
    // 你的任务工作在这里
    do_work().await;
    
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

### 4. 任务完成

```rust
// 任务完成时结束监控
monitor.finish_monitoring(task_id);
```

### 5. 数据检索

```rust
// 获取所有任务配置文件
let profiles = monitor.get_all_profiles();

// 获取特定任务配置文件
if let Some(profile) = monitor.get_profile(task_id) {
    println!("任务效率：{:.1}%", profile.efficiency_score * 100.0);
}
```

## 显示内容

### 1. 性能概览

- **汇总统计**：总任务数、平均资源使用、整体效率
- **性能趋势**：显示资源分布的可视化图表
- **系统健康**：整体系统资源利用率

### 2. 任务分析

- **单个任务卡片**：每个监控任务的详细指标
- **基线对比**：每个任务与平均性能的比较
- **类别排名**：在任务类型类别中的排名
- **效率分解**：组件效率分析

### 3. 资源监控

- **CPU使用**：利用率百分比、核心分布、上下文切换
- **内存使用**：当前/峰值使用、分配模式、碎片化
- **IO性能**：带宽、延迟、操作计数
- **网络活动**：吞吐量、连接计数、错误率
- **GPU利用率**：GPU使用、内存消耗、计算单元

### 4. 优化洞察

- **瓶颈识别**：主要性能瓶颈
- **热点**：性能关键代码段
- **优化建议**：可操作的改进建议
- **关键路径分析**：执行流程分析

## 示例

### 基本用法

```rust
use memscope_rs::async_memory::{self, AsyncResourceMonitor, TaskType, TaskId};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化
    async_memory::initialize()?;
    let monitor = Arc::new(Mutex::new(AsyncResourceMonitor::new()));
    
    // 启动带监控的任务
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let task_id = i as TaskId;
        let monitor_clone = Arc::clone(&monitor);
        
        // 注册任务
        {
            let mut mon = monitor_clone.lock().unwrap();
            mon.start_monitoring(task_id, format!("任务-{}", i), TaskType::CpuIntensive);
        }
        
        // 生成监控任务
        let handle = tokio::spawn(async move {
            execute_monitored_task(task_id, monitor_clone).await
        });
        
        handles.push((task_id, handle));
    }
    
    // 等待完成
    for (task_id, handle) in handles {
        handle.await?;
        
        let mut mon = monitor.lock().unwrap();
        mon.finish_monitoring(task_id);
    }
    
    // 生成报告
    let profiles = {
        let mon = monitor.lock().unwrap();
        mon.get_all_profiles().clone()
    };
    
    let visualizer = VisualizationGenerator::new();
    let html_report = visualizer.generate_html_report(&profiles)?;
    std::fs::write("performance_report.html", html_report)?;
    
    println!("报告已生成：performance_report.html");
    Ok(())
}

async fn execute_monitored_task(
    task_id: TaskId,
    monitor: Arc<Mutex<AsyncResourceMonitor>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 定期指标更新
    let update_handle = {
        let monitor_clone = Arc::clone(&monitor);
        tokio::spawn(async move {
            for _ in 0..10 {
                {
                    let mut mon = monitor_clone.lock().unwrap();
                    mon.update_metrics(task_id);
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        })
    };
    
    // 模拟工作
    for i in 0..1000000u32 {
        let _ = i.wrapping_mul(i) % 12345;
        if i % 100000 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    update_handle.await?;
    Ok(())
}
```

### 高级用法与自定义配置

```rust
use memscope_rs::async_memory::{
    VisualizationGenerator, VisualizationConfig, Theme,
    resource_monitor::SourceLocation
};

// 创建自定义可视化配置
let viz_config = VisualizationConfig {
    title: "生产环境性能分析".to_string(),
    theme: Theme::Dark,
    include_charts: true,
    include_baselines: true,
    include_rankings: true,
    include_efficiency_breakdown: true,
};

// 带源跟踪的监控任务
let source_location = SourceLocation {
    file_path: file!().to_string(),
    line_number: line!(),
    function_name: "advanced_task".to_string(),
    module_path: module_path!().to_string(),
    crate_name: env!("CARGO_PKG_NAME").to_string(),
};

monitor.start_monitoring_with_location(
    task_id, 
    "高级任务", 
    TaskType::Mixed, 
    Some(source_location)
);
```

### 内存跟踪示例

```rust
use memscope_rs::async_memory::{create_tracked, TaskId};

async fn memory_intensive_task() -> Result<(), Box<dyn std::error::Error>> {
    let task_id = TaskId::new();
    
    let result = create_tracked(task_id, async {
        // 内存分配被跟踪
        let large_buffer = vec![0u8; 10 * 1024 * 1024]; // 10MB
        
        // 处理数据
        process_buffer(large_buffer).await?;
        
        // 分配和释放都被监控
        let cache = build_cache().await?;
        
        Ok::<_, Box<dyn std::error::Error>>("任务完成")
    }).await?;
    
    println!("结果：{}", result);
    Ok(())
}
```

### 与现有应用程序集成

```rust
// 在现有异步应用程序中
async fn integrate_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化监控
    async_memory::initialize()?;
    let monitor = Arc::new(Mutex::new(AsyncResourceMonitor::new()));
    
    // 现有任务生成逻辑
    let mut task_handles = Vec::new();
    
    for (i, config) in task_configs.iter().enumerate() {
        let task_id = i as TaskId;
        let monitor_clone = Arc::clone(&monitor);
        
        // 为现有任务添加监控
        {
            let mut mon = monitor_clone.lock().unwrap();
            mon.start_monitoring(task_id, config.name.clone(), config.task_type);
        }
        
        // 使用监控包装器生成
        let handle = tokio::spawn(async move {
            let result = execute_existing_task(config).await;
            
            // 完成监控
            {
                let mut mon = monitor_clone.lock().unwrap();
                mon.finish_monitoring(task_id);
            }
            
            result
        });
        
        task_handles.push(handle);
    }
    
    // 等待完成并生成报告
    for handle in task_handles {
        handle.await?;
    }
    
    generate_performance_report(&monitor).await?;
    Ok(())
}
```

## 性能特征

### 内存开销
- **每个任务**：约1-5KB元数据
- **全局状态**：监控基础设施约10-50KB
- **缩放**：与任务数量线性O(n)

### CPU开销
- **指标收集**：<1% CPU开销
- **报告生成**：O(n)，其中n是任务数量
- **实时更新**：可配置频率（默认100ms）

### 存储需求
- **内存配置文件**：每个任务约2-10KB
- **HTML报告**：最终输出每个任务约2-5KB
- **最小磁盘使用**：仅用于生成的报告

## 最佳实践

### 1. 任务命名和分类
```rust
// 使用描述性任务名称
monitor.start_monitoring(task_id, "图像处理管道", TaskType::CpuIntensive);

// 按类型分组相关任务
monitor.start_monitoring(task_id, "数据库查询", TaskType::IoIntensive);
monitor.start_monitoring(task_id, "API请求", TaskType::NetworkIntensive);
```

### 2. 监控生命周期
```rust
// 始终配对start/finish调用
monitor.start_monitoring(task_id, name, task_type);
// ... 任务执行 ...
monitor.finish_monitoring(task_id); // 重要：不要忘记这个！
```

### 3. 源位置跟踪
```rust
// 使用宏进行自动源跟踪
macro_rules! track_task {
    ($monitor:expr, $task_id:expr, $name:expr, $task_type:expr) => {
        $monitor.start_monitoring_with_location(
            $task_id,
            $name,
            $task_type,
            Some(SourceLocation {
                file_path: file!().to_string(),
                line_number: line!(),
                function_name: "unknown".to_string(), // 可以用更多宏增强
                module_path: module_path!().to_string(),
                crate_name: env!("CARGO_PKG_NAME").to_string(),
            })
        );
    };
}
```

### 4. 错误处理
```rust
use memscope_rs::async_memory::AsyncError;

match monitor.start_monitoring(task_id, name, task_type) {
    Ok(_) => { /* 监控已开始 */ }
    Err(AsyncError::TaskAlreadyExists) => {
        // 处理重复任务ID
    }
    Err(e) => {
        eprintln!("监控错误：{}", e);
    }
}
```

### 5. 性能优化
```rust
// 批量指标更新以提高效率
let update_interval = Duration::from_millis(500); // 根据需要调整

// 使用定期更新而不是连续更新
tokio::spawn(async move {
    let mut interval = tokio::time::interval(update_interval);
    loop {
        interval.tick().await;
        monitor.update_all_metrics(); // 批量更新所有任务
    }
});
```

## 故障排除

### 常见问题

#### 1. 缺少任务数据
```rust
// 问题：报告中没有数据
// 解决：确保监控生命周期完整
monitor.start_monitoring(task_id, name, task_type);
// ... 执行任务 ...
monitor.finish_monitoring(task_id); // <- 不要忘记这个！
```

#### 2. 内存泄漏
```rust
// 问题：内存使用随时间增长
// 解决：正确清理已完成的任务
monitor.cleanup_finished_tasks(); // 定期清理
```

#### 3. 性能影响
```rust
// 问题：监控开销过高
// 解决：减少更新频率
let config = MonitoringConfig {
    update_interval_ms: 1000, // 从默认100ms增加
    batch_updates: true,       // 启用批处理
};
```

#### 4. 大报告文件
```rust
// 问题：HTML报告过大
// 解决：过滤或分页数据
let filtered_profiles: HashMap<_, _> = profiles
    .into_iter()
    .filter(|(_, profile)| profile.efficiency_score < 0.8) // 仅效率低的任务
    .collect();

let html_report = visualizer.generate_html_report(&filtered_profiles)?;
```

### 调试模式

```rust
// 启用调试日志
env_logger::init();

// 使用调试方法
monitor.debug_print_all_tasks();
monitor.validate_consistency();
```

## 迁移指南

### 从基本监控到异步内存

```rust
// 之前：基本计时
let start = Instant::now();
execute_task().await;
let duration = start.elapsed();
println!("任务耗时：{:?}", duration);

// 之后：综合监控
let task_id = TaskId::new();
monitor.start_monitoring(task_id, "我的任务", TaskType::CpuIntensive);

let update_handle = tokio::spawn({
    let monitor = monitor.clone();
    async move {
        loop {
            monitor.update_metrics(task_id);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
});

execute_task().await;

update_handle.abort();
monitor.finish_monitoring(task_id);

// 获取详细分析
let profile = monitor.get_profile(task_id).unwrap();
println!("效率：{:.1}%", profile.efficiency_score * 100.0);
println!("瓶颈：{:?}", profile.bottleneck_type);
```

## API 总结

### 主要导出
```rust
pub use async_memory::{
    // 核心监控
    AsyncResourceMonitor,
    TaskResourceProfile,
    TaskType,
    BottleneckType,
    
    // 指标
    CpuMetrics,
    MemoryMetrics,
    IoMetrics,
    NetworkMetrics,
    GpuMetrics,
    
    // 可视化
    VisualizationGenerator,
    VisualizationConfig,
    Theme,
    
    // 跟踪
    create_tracked,
    TaskId,
    
    // 分析
    PerformanceBaselines,
    CategoryRanking,
    PerformanceComparison,
    
    // 错误
    AsyncError,
    VisualizationError,
};
```

## 动态特性说明

### 自动任务缩放

该模块**完全支持动态任务数量**：

#### 小规模场景（1-10个任务）
- 所有任务清晰显示在报告中
- 图表展示每个任务的详细条形
- 排名和对比一目了然

#### 中等规模场景（10-100个任务）  
- 响应式布局自动调整网格
- 图表自动缩放以容纳所有数据
- 滚动浏览保持良好用户体验

#### 大规模场景（100+个任务）
- 自动优化渲染性能
- 图表保持可读性
- 报告文件大小合理控制

#### 超大规模场景（1000+个任务）
- 建议使用过滤或分批报告
- 内存使用线性增长O(n)
- 生成时间可接受范围内

### 实时适应示例

```rust
// 这个函数可以处理任意数量的任务
async fn monitor_dynamic_tasks(task_count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let monitor = Arc::new(Mutex::new(AsyncResourceMonitor::new()));
    let mut handles = Vec::new();
    
    // 动态生成任意数量的任务
    for i in 0..task_count {
        let task_id = i as TaskId;
        let monitor_clone = Arc::clone(&monitor);
        
        // 每个任务都会被正确监控和显示
        {
            let mut mon = monitor_clone.lock().unwrap();
            mon.start_monitoring(task_id, format!("动态任务-{}", i), determine_task_type(i));
        }
        
        let handle = tokio::spawn(async move {
            execute_task_with_monitoring(task_id, monitor_clone).await
        });
        
        handles.push((task_id, handle));
    }
    
    // 等待所有任务完成
    for (task_id, handle) in handles {
        handle.await?;
        let mut mon = monitor.lock().unwrap();
        mon.finish_monitoring(task_id);
    }
    
    // 生成包含所有任务的报告
    let profiles = {
        let mon = monitor.lock().unwrap();
        mon.get_all_profiles().clone()
    };
    
    let visualizer = VisualizationGenerator::new();
    let html_report = visualizer.generate_html_report(&profiles)?;
    
    // 报告自动适应任务数量
    std::fs::write(format!("report_{}_tasks.html", task_count), html_report)?;
    
    println!("✅ 成功为 {} 个任务生成报告", task_count);
    Ok(())
}

// 使用示例
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 测试不同规模
    monitor_dynamic_tasks(5).await?;    // 小规模
    monitor_dynamic_tasks(50).await?;   // 中等规模  
    monitor_dynamic_tasks(200).await?;  // 大规模
    
    Ok(())
}
```

这个综合性模块为Rust异步应用提供了企业级的任务监控、性能分析和优化能力，无论应用规模大小都能提供卓越的监控体验。