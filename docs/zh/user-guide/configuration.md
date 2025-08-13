# 配置选项详解

memscope-rs 提供了丰富的配置选项，让你可以根据具体需求调整内存跟踪的行为和性能。

## 🎯 配置概览

| 配置类别 | 用途 | 性能影响 |
|---------|------|----------|
| **跟踪配置** | 控制跟踪行为 | 低到中等 |
| **导出配置** | 自定义导出格式 | 中等 |
| **性能配置** | 优化性能表现 | 显著 |
| **分析配置** | 调整分析深度 | 中等到高 |

## 🔧 基础配置

### 初始化配置

```rust
use memscope_rs::{init_with_config, TrackingConfig};

fn main() {
    // 使用默认配置
    memscope_rs::init();
    
    // 或使用自定义配置
    let config = TrackingConfig {
        enable_stack_traces: true,
        max_tracked_allocations: 10000,
        enable_lifecycle_tracking: true,
        ..Default::default()
    };
    
    init_with_config(config);
}
```

### TrackingConfig 选项

```rust
pub struct TrackingConfig {
    /// 是否启用栈跟踪 (影响性能)
    pub enable_stack_traces: bool,
    
    /// 最大跟踪分配数量 (0 = 无限制)
    pub max_tracked_allocations: usize,
    
    /// 是否启用生命周期跟踪
    pub enable_lifecycle_tracking: bool,
    
    /// 是否启用借用检查分析
    pub enable_borrow_analysis: bool,
    
    /// 是否启用循环引用检测
    pub enable_circular_reference_detection: bool,
    
    /// 是否启用异步分析
    pub enable_async_analysis: bool,
    
    /// 是否启用 FFI 跟踪
    pub enable_ffi_tracking: bool,
    
    /// 内存阈值 (字节)，小于此值的分配可能被忽略
    pub memory_threshold: usize,
    
    /// 采样率 (0.0-1.0)，1.0 = 跟踪所有分配
    pub sampling_rate: f64,
    
    /// 是否启用实时统计
    pub enable_real_time_stats: bool,
}
```

## 📊 导出配置

### ExportOptions 配置

```rust
use memscope_rs::{get_global_tracker, ExportOptions};

fn export_with_config() {
    let tracker = get_global_tracker();
    
    let options = ExportOptions {
        include_stack_traces: true,
        include_lifecycle_data: true,
        include_type_analysis: true,
        compress_output: true,
        max_entries: Some(5000),
        filter_small_allocations: true,
        min_allocation_size: 64,
        ..Default::default()
    };
    
    tracker.export_to_json_with_options("analysis", &options).unwrap();
}
```

### 导出选项详解

```rust
pub struct ExportOptions {
    /// 包含栈跟踪信息
    pub include_stack_traces: bool,
    
    /// 包含生命周期数据
    pub include_lifecycle_data: bool,
    
    /// 包含类型分析信息
    pub include_type_analysis: bool,
    
    /// 压缩输出文件
    pub compress_output: bool,
    
    /// 最大导出条目数
    pub max_entries: Option<usize>,
    
    /// 过滤小分配
    pub filter_small_allocations: bool,
    
    /// 最小分配大小阈值
    pub min_allocation_size: usize,
    
    /// 包含性能指标
    pub include_performance_metrics: bool,
    
    /// 包含内存布局信息
    pub include_memory_layout: bool,
    
    /// 导出格式版本
    pub format_version: String,
    
    /// 自定义元数据
    pub custom_metadata: std::collections::HashMap<String, String>,
}
```

## ⚡ 性能配置

### 高性能配置

```rust
use memscope_rs::TrackingConfig;

// 生产环境配置 - 最小开销
let production_config = TrackingConfig {
    enable_stack_traces: false,          // 关闭栈跟踪
    max_tracked_allocations: 1000,       // 限制跟踪数量
    enable_lifecycle_tracking: false,    // 关闭生命周期跟踪
    enable_borrow_analysis: false,       // 关闭借用分析
    enable_circular_reference_detection: false,
    enable_async_analysis: false,
    enable_ffi_tracking: false,
    memory_threshold: 1024,              // 只跟踪 >1KB 的分配
    sampling_rate: 0.1,                  // 10% 采样率
    enable_real_time_stats: false,
};

// 开发环境配置 - 完整功能
let development_config = TrackingConfig {
    enable_stack_traces: true,
    max_tracked_allocations: 50000,
    enable_lifecycle_tracking: true,
    enable_borrow_analysis: true,
    enable_circular_reference_detection: true,
    enable_async_analysis: true,
    enable_ffi_tracking: true,
    memory_threshold: 0,                 // 跟踪所有分配
    sampling_rate: 1.0,                  // 100% 采样率
    enable_real_time_stats: true,
};

// 调试环境配置 - 最详细信息
let debug_config = TrackingConfig {
    enable_stack_traces: true,
    max_tracked_allocations: 0,          // 无限制
    enable_lifecycle_tracking: true,
    enable_borrow_analysis: true,
    enable_circular_reference_detection: true,
    enable_async_analysis: true,
    enable_ffi_tracking: true,
    memory_threshold: 0,
    sampling_rate: 1.0,
    enable_real_time_stats: true,
};
```

### 性能调优建议

```rust
// 根据应用类型选择配置
fn get_config_for_app_type(app_type: &str) -> TrackingConfig {
    match app_type {
        "web_server" => TrackingConfig {
            enable_stack_traces: false,
            max_tracked_allocations: 5000,
            sampling_rate: 0.05,  // 5% 采样，减少开销
            memory_threshold: 4096,
            ..Default::default()
        },
        
        "desktop_app" => TrackingConfig {
            enable_stack_traces: true,
            max_tracked_allocations: 20000,
            sampling_rate: 0.5,   // 50% 采样
            memory_threshold: 512,
            enable_lifecycle_tracking: true,
            ..Default::default()
        },
        
        "embedded" => TrackingConfig {
            enable_stack_traces: false,
            max_tracked_allocations: 1000,
            sampling_rate: 0.01,  // 1% 采样，极低开销
            memory_threshold: 8192,
            enable_real_time_stats: false,
            ..Default::default()
        },
        
        _ => TrackingConfig::default(),
    }
}
```

## 🔍 分析配置

### 高级分析配置

```rust
use memscope_rs::analysis::AnalysisConfig;

let analysis_config = AnalysisConfig {
    // 循环引用检测配置
    circular_reference_detection: CircularReferenceConfig {
        enabled: true,
        max_depth: 10,
        check_interval_ms: 1000,
    },
    
    // 生命周期分析配置
    lifecycle_analysis: LifecycleConfig {
        enabled: true,
        track_drop_order: true,
        analyze_scope_relationships: true,
    },
    
    // 异步分析配置
    async_analysis: AsyncConfig {
        enabled: true,
        track_future_states: true,
        analyze_await_points: true,
    },
    
    // FFI 跟踪配置
    ffi_tracking: FFIConfig {
        enabled: true,
        track_c_allocations: true,
        validate_pointer_safety: true,
    },
};
```

## 🌍 环境变量配置

可以通过环境变量覆盖配置：

```bash
# 基础配置
export MEMSCOPE_ENABLE_STACK_TRACES=true
export MEMSCOPE_MAX_TRACKED_ALLOCATIONS=10000
export MEMSCOPE_MEMORY_THRESHOLD=1024
export MEMSCOPE_SAMPLING_RATE=0.5

# 功能开关
export MEMSCOPE_ENABLE_LIFECYCLE_TRACKING=true
export MEMSCOPE_ENABLE_BORROW_ANALYSIS=false
export MEMSCOPE_ENABLE_ASYNC_ANALYSIS=true
export MEMSCOPE_ENABLE_FFI_TRACKING=false

# 导出配置
export MEMSCOPE_EXPORT_COMPRESS=true
export MEMSCOPE_EXPORT_MAX_ENTRIES=5000
export MEMSCOPE_EXPORT_MIN_SIZE=64

# 性能配置
export MEMSCOPE_REAL_TIME_STATS=false
export MEMSCOPE_BACKGROUND_ANALYSIS=true
```

在代码中读取环境变量：

```rust
use memscope_rs::TrackingConfig;

fn config_from_env() -> TrackingConfig {
    TrackingConfig {
        enable_stack_traces: std::env::var("MEMSCOPE_ENABLE_STACK_TRACES")
            .map(|v| v.parse().unwrap_or(false))
            .unwrap_or(false),
            
        max_tracked_allocations: std::env::var("MEMSCOPE_MAX_TRACKED_ALLOCATIONS")
            .map(|v| v.parse().unwrap_or(10000))
            .unwrap_or(10000),
            
        sampling_rate: std::env::var("MEMSCOPE_SAMPLING_RATE")
            .map(|v| v.parse().unwrap_or(1.0))
            .unwrap_or(1.0),
            
        ..Default::default()
    }
}
```

## 📁 配置文件

### TOML 配置文件

创建 `memscope.toml`：

```toml
[tracking]
enable_stack_traces = true
max_tracked_allocations = 10000
enable_lifecycle_tracking = true
enable_borrow_analysis = true
enable_circular_reference_detection = true
enable_async_analysis = false
enable_ffi_tracking = false
memory_threshold = 512
sampling_rate = 1.0
enable_real_time_stats = true

[export]
include_stack_traces = true
include_lifecycle_data = true
include_type_analysis = true
compress_output = false
max_entries = 5000
filter_small_allocations = true
min_allocation_size = 64
include_performance_metrics = true

[analysis.circular_reference]
enabled = true
max_depth = 10
check_interval_ms = 1000

[analysis.lifecycle]
enabled = true
track_drop_order = true
analyze_scope_relationships = true

[analysis.async]
enabled = false
track_future_states = false
analyze_await_points = false

[analysis.ffi]
enabled = false
track_c_allocations = false
validate_pointer_safety = false
```

加载配置文件：

```rust
use memscope_rs::config::load_config_from_file;

fn main() {
    let config = load_config_from_file("memscope.toml")
        .unwrap_or_else(|_| TrackingConfig::default());
    
    memscope_rs::init_with_config(config);
}
```

## 🎛️ 运行时配置

### 动态调整配置

```rust
use memscope_rs::get_global_tracker;

fn adjust_runtime_config() {
    let tracker = get_global_tracker();
    
    // 动态调整采样率
    tracker.set_sampling_rate(0.1);
    
    // 动态调整内存阈值
    tracker.set_memory_threshold(2048);
    
    // 启用/禁用特定功能
    tracker.enable_stack_traces(false);
    tracker.enable_lifecycle_tracking(true);
    
    // 清理旧数据
    tracker.cleanup_old_allocations(Duration::from_secs(300));
}
```

### 条件配置

```rust
fn conditional_config() -> TrackingConfig {
    let mut config = TrackingConfig::default();
    
    // 根据构建类型调整
    #[cfg(debug_assertions)]
    {
        config.enable_stack_traces = true;
        config.enable_lifecycle_tracking = true;
        config.sampling_rate = 1.0;
    }
    
    #[cfg(not(debug_assertions))]
    {
        config.enable_stack_traces = false;
        config.sampling_rate = 0.1;
        config.memory_threshold = 1024;
    }
    
    // 根据目标平台调整
    #[cfg(target_os = "linux")]
    {
        config.enable_ffi_tracking = true;
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        config.max_tracked_allocations = 1000;
        config.enable_real_time_stats = false;
    }
    
    config
}
```

## 📊 配置验证

### 验证配置有效性

```rust
use memscope_rs::config::validate_config;

fn validate_my_config(config: &TrackingConfig) -> Result<(), String> {
    // 检查采样率范围
    if config.sampling_rate < 0.0 || config.sampling_rate > 1.0 {
        return Err("采样率必须在 0.0-1.0 之间".to_string());
    }
    
    // 检查内存阈值
    if config.memory_threshold > 1024 * 1024 {
        return Err("内存阈值不应超过 1MB".to_string());
    }
    
    // 检查功能兼容性
    if config.enable_async_analysis && !config.enable_lifecycle_tracking {
        return Err("异步分析需要启用生命周期跟踪".to_string());
    }
    
    Ok(())
}
```

## 🔧 最佳实践

### 1. 分层配置策略

```rust
// 基础配置
let base_config = TrackingConfig::default();

// 环境特定配置
let env_config = match std::env::var("RUST_ENV").as_deref() {
    Ok("production") => production_overrides(),
    Ok("development") => development_overrides(),
    Ok("testing") => testing_overrides(),
    _ => TrackingConfig::default(),
};

// 合并配置
let final_config = merge_configs(base_config, env_config);
```

### 2. 性能监控配置

```rust
// 监控配置对性能的影响
fn monitor_config_impact() {
    let start = std::time::Instant::now();
    
    // 执行一些操作
    perform_operations();
    
    let duration = start.elapsed();
    println!("操作耗时: {:?}", duration);
    
    // 根据性能调整配置
    if duration > Duration::from_millis(100) {
        // 降低跟踪精度以提高性能
        adjust_for_performance();
    }
}
```

### 3. 配置文档化

```rust
/// 生产环境配置
/// 
/// 特点:
/// - 最小性能开销
/// - 基础内存跟踪
/// - 压缩导出
/// - 采样跟踪
const PRODUCTION_CONFIG: TrackingConfig = TrackingConfig {
    enable_stack_traces: false,
    max_tracked_allocations: 5000,
    sampling_rate: 0.05,
    memory_threshold: 4096,
    enable_lifecycle_tracking: false,
    enable_real_time_stats: false,
};
```

## 🔗 相关文档

- [性能优化指南](../advanced/performance-optimization.md) - 性能调优技巧
- [常见问题](troubleshooting.md) - 配置相关问题解决
- [CLI 工具](cli-tools.md) - 命令行配置选项

---

合理的配置是高效内存分析的基础！ ⚙️