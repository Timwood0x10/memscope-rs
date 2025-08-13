# 第一次内存分析

本指南将带你完成第一次完整的内存分析，从数据收集到报告解读，让你快速掌握 memscope-rs 的分析能力。

## 🎯 学习目标

完成本指南后，你将能够：
- 生成完整的内存分析报告
- 解读各种图表和数据
- 识别常见的内存问题
- 使用交互式仪表板进行深入分析

## 📊 完整分析示例

### 创建分析目标程序

首先创建一个包含多种内存模式的示例程序：

```rust
use memscope_rs::{track_var, track_var_smart, get_global_tracker, init};
use std::rc::Rc;
use std::sync::Arc;
use std::collections::HashMap;

fn main() {
    // 1. 初始化跟踪系统
    init();
    println!("🚀 开始内存分析示例");
    
    // 2. 基础数据类型
    basic_types_demo();
    
    // 3. 智能指针演示
    smart_pointers_demo();
    
    // 4. 复杂数据结构
    complex_structures_demo();
    
    // 5. 生成分析报告
    generate_analysis_reports();
    
    println!("✅ 分析完成！查看 MemoryAnalysis/ 目录");
}

fn basic_types_demo() {
    println!("\n📦 基础类型分配...");
    
    // 字符串分配
    let greeting = String::from("Hello, Memory Analysis!");
    track_var!(greeting);
    
    // 向量分配
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    track_var!(numbers);
    
    // 大数组分配
    let large_array = vec![0u8; 1024 * 10]; // 10KB
    track_var!(large_array);
    
    println!("  ✓ 字符串: {} bytes", greeting.len());
    println!("  ✓ 数字向量: {} 个元素", numbers.len());
    println!("  ✓ 大数组: {} bytes", large_array.len());
}

fn smart_pointers_demo() {
    println!("\n🔗 智能指针演示...");
    
    // Box 指针
    let boxed_data = Box::new(vec![1; 100]);
    track_var!(boxed_data);
    
    // Rc 引用计数
    let shared_data = Rc::new(String::from("共享数据"));
    track_var!(shared_data);
    
    let shared_clone1 = Rc::clone(&shared_data);
    track_var!(shared_clone1);
    
    let shared_clone2 = Rc::clone(&shared_data);
    track_var!(shared_clone2);
    
    // Arc 原子引用计数
    let thread_safe_data = Arc::new(vec![1, 2, 3, 4, 5]);
    track_var!(thread_safe_data);
    
    println!("  ✓ Box 数据: {} 个元素", boxed_data.len());
    println!("  ✓ Rc 引用计数: {}", Rc::strong_count(&shared_data));
    println!("  ✓ Arc 数据: {} 个元素", thread_safe_data.len());
}

fn complex_structures_demo() {
    println!("\n🏗️ 复杂数据结构...");
    
    // HashMap
    let mut user_cache = HashMap::new();
    user_cache.insert("user1", "Alice");
    user_cache.insert("user2", "Bob");
    user_cache.insert("user3", "Charlie");
    track_var!(user_cache);
    
    // 嵌套结构
    let matrix = vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ];
    track_var!(matrix);
    
    // 自定义结构体
    #[derive(Debug)]
    struct UserProfile {
        name: String,
        email: String,
        preferences: Vec<String>,
        metadata: HashMap<String, String>,
    }
    
    let profile = UserProfile {
        name: String::from("测试用户"),
        email: String::from("test@example.com"),
        preferences: vec![
            String::from("dark_mode"),
            String::from("notifications"),
        ],
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("created_at".to_string(), "2024-01-01".to_string());
            meta.insert("last_login".to_string(), "2024-01-15".to_string());
            meta
        },
    };
    track_var!(profile);
    
    println!("  ✓ 用户缓存: {} 个条目", user_cache.len());
    println!("  ✓ 矩阵: {}x{}", matrix.len(), matrix[0].len());
    println!("  ✓ 用户配置: {}", profile.name);
}

fn generate_analysis_reports() {
    println!("\n📊 生成分析报告...");
    
    let tracker = get_global_tracker();
    
    // 显示当前统计信息
    if let Ok(stats) = tracker.get_stats() {
        println!("  📈 当前内存统计:");
        println!("    - 活跃分配: {}", stats.active_allocations);
        println!("    - 活跃内存: {} bytes", stats.active_memory);
        println!("    - 总分配次数: {}", stats.total_allocations);
        println!("    - 总释放次数: {}", stats.total_deallocations);
        println!("    - 峰值内存: {} bytes", stats.peak_memory);
    }
    
    // 生成 JSON 报告
    match tracker.export_to_json("first_analysis") {
        Ok(_) => println!("  ✅ JSON 报告生成成功"),
        Err(e) => println!("  ❌ JSON 报告生成失败: {}", e),
    }
    
    // 生成 SVG 图表
    match tracker.export_memory_analysis("first_analysis.svg") {
        Ok(_) => println!("  ✅ SVG 图表生成成功"),
        Err(e) => println!("  ❌ SVG 图表生成失败: {}", e),
    }
    
    // 生成 HTML 交互式报告
    match tracker.export_to_html("first_analysis.html") {
        Ok(_) => println!("  ✅ HTML 报告生成成功"),
        Err(e) => println!("  ❌ HTML 报告生成失败: {}", e),
    }
}
```

### 运行分析

```bash
# 编译并运行
cargo run

# 查看生成的文件
ls -la MemoryAnalysis/first_analysis/
```

## 📈 报告解读指南

### JSON 数据分析

生成的 JSON 文件包含详细的内存分配信息：

```json
{
  "memory_stats": {
    "active_allocations": 8,
    "active_memory": 15432,
    "total_allocations": 12,
    "total_deallocations": 4,
    "peak_memory": 18560
  },
  "allocations": [
    {
      "id": "alloc_001",
      "size": 1024,
      "type_name": "Vec<u8>",
      "location": "src/main.rs:45",
      "timestamp": "2024-01-15T10:30:00Z",
      "status": "active"
    }
  ]
}
```

**关键指标解读**:
- `active_allocations`: 当前未释放的分配数量
- `active_memory`: 当前占用的内存总量
- `peak_memory`: 程序运行期间的内存使用峰值
- `total_allocations`: 总分配次数（包括已释放的）

### SVG 图表分析

SVG 图表提供可视化的内存使用趋势：

**时间线图表**:
- X 轴：时间进度
- Y 轴：内存使用量
- 线条：内存使用趋势
- 峰值点：内存使用高峰

**分配类型饼图**:
- 不同颜色代表不同数据类型
- 扇形大小表示内存占用比例
- 悬停显示详细信息

### HTML 交互式仪表板

HTML 报告提供最丰富的分析功能：

#### 1. 概览面板
```
📊 内存概览
├── 总内存使用: 15.4 KB
├── 活跃分配: 8 个
├── 内存效率: 92.3%
└── 潜在问题: 0 个
```

#### 2. 分配详情表格
| ID | 类型 | 大小 | 位置 | 状态 | 操作 |
|----|------|------|------|------|------|
| #001 | Vec<u8> | 10.0 KB | main.rs:45 | 活跃 | [详情] |
| #002 | String | 23 B | main.rs:32 | 活跃 | [详情] |

#### 3. 交互式图表
- **内存时间线**: 拖拽缩放，查看特定时间段
- **类型分布**: 点击筛选特定类型
- **热力图**: 显示内存热点区域

#### 4. 过滤和搜索
```
🔍 筛选选项:
□ 仅显示活跃分配
□ 大于 1KB 的分配
□ String 类型
□ 最近 1 分钟
```

## 🔍 常见模式识别

### 1. 内存泄漏模式

**特征**:
- `active_allocations` 持续增长
- `active_memory` 不断上升
- 时间线图呈上升趋势

**示例**:
```rust
// ❌ 潜在内存泄漏
fn memory_leak_example() {
    loop {
        let data = vec![0; 1024];
        track_var!(data);
        std::mem::forget(data); // 故意泄漏
    }
}
```

### 2. 内存碎片模式

**特征**:
- 频繁的分配和释放
- 小块内存分配较多
- 内存使用效率较低

**示例**:
```rust
// ⚠️ 内存碎片风险
fn fragmentation_example() {
    for i in 0..1000 {
        let small_alloc = vec![i; 10];
        track_var!(small_alloc);
    }
}
```

### 3. 峰值内存模式

**特征**:
- 短时间内大量内存分配
- 峰值远高于平均值
- 可能导致 OOM

**示例**:
```rust
// ⚠️ 内存峰值风险
fn peak_memory_example() {
    let huge_data = vec![0u8; 100 * 1024 * 1024]; // 100MB
    track_var!(huge_data);
    // 处理完后立即释放
}
```

## 🛠️ 分析技巧

### 1. 对比分析

```rust
use memscope_rs::{get_global_tracker, track_var, init};

fn comparative_analysis() {
    init();
    let tracker = get_global_tracker();
    
    // 记录基线
    let baseline = tracker.get_stats().unwrap();
    
    // 执行操作
    {
        let data = vec![0; 1000];
        track_var!(data);
        
        // 记录操作后状态
        let after_op = tracker.get_stats().unwrap();
        println!("操作增加内存: {} bytes", 
                after_op.active_memory - baseline.active_memory);
    }
    
    // 记录清理后状态
    let after_cleanup = tracker.get_stats().unwrap();
    println!("清理后内存变化: {} bytes", 
            after_cleanup.active_memory - baseline.active_memory);
}
```

### 2. 分阶段分析

```rust
fn staged_analysis() {
    init();
    let tracker = get_global_tracker();
    
    // 阶段 1: 初始化
    println!("🔄 阶段 1: 初始化");
    initialization_phase();
    tracker.export_to_json("stage1_init").ok();
    
    // 阶段 2: 数据加载
    println!("🔄 阶段 2: 数据加载");
    data_loading_phase();
    tracker.export_to_json("stage2_loading").ok();
    
    // 阶段 3: 处理
    println!("🔄 阶段 3: 数据处理");
    data_processing_phase();
    tracker.export_to_json("stage3_processing").ok();
    
    // 阶段 4: 清理
    println!("🔄 阶段 4: 清理");
    cleanup_phase();
    tracker.export_to_json("stage4_cleanup").ok();
}
```

### 3. 性能影响分析

```rust
use std::time::Instant;

fn performance_impact_analysis() {
    init();
    
    // 测试无跟踪性能
    let start = Instant::now();
    for i in 0..10000 {
        let data = vec![i; 100];
        // 不跟踪
    }
    let no_tracking_time = start.elapsed();
    
    // 测试有跟踪性能
    let start = Instant::now();
    for i in 0..10000 {
        let data = vec![i; 100];
        track_var!(data);
    }
    let with_tracking_time = start.elapsed();
    
    println!("性能影响分析:");
    println!("  无跟踪: {:?}", no_tracking_time);
    println!("  有跟踪: {:?}", with_tracking_time);
    println!("  开销: {:.2}%", 
            (with_tracking_time.as_nanos() as f64 / no_tracking_time.as_nanos() as f64 - 1.0) * 100.0);
}
```

## 🚀 下一步行动

### 基于分析结果的优化

1. **识别问题**:
   - 查看 HTML 报告中的"潜在问题"部分
   - 关注内存使用峰值
   - 检查长期存活的大对象

2. **制定优化策略**:
   - 减少不必要的分配
   - 优化数据结构选择
   - 改进内存释放时机

3. **验证优化效果**:
   - 重新运行分析
   - 对比优化前后的报告
   - 关注关键指标变化

### 继续学习

- **[导出格式说明](../user-guide/export-formats.md)** - 深入了解各种导出格式
- **[内存分析功能](../user-guide/memory-analysis.md)** - 学习高级分析技巧
- **[性能优化指南](../advanced/performance-optimization.md)** - 系统性优化方法

## 💡 关键要点

- **HTML 报告最全面** - 提供交互式分析功能
- **JSON 数据可编程处理** - 适合自动化分析
- **SVG 图表直观易懂** - 适合报告和演示
- **分阶段分析更精确** - 帮助定位具体问题
- **对比分析显示趋势** - 验证优化效果

恭喜你完成了第一次内存分析！现在你已经具备了基础的内存分析能力。🎉